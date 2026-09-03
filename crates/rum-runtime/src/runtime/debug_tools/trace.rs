mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod builder;

use builder::{StructuredTraceBuilder, trace_expr_path_depth};

#[derive(Clone, FrameRootTrace)]
struct TraceEvalState {
    original: Expr,
    pattern: Option<Expr>,
    levels: Option<CasesLevelSpec>,
    include_heads: bool,
    output: TraceOutput,
}

#[derive(Clone, Copy, PartialEq, Eq, FrameRootTrace)]
enum TraceOutput {
    Structured,
    Detail,
    Print,
}

impl Session {
    pub(in crate::runtime) fn eval_trace(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_trace_named(args, "Trace", TraceOutput::Structured)
    }

    pub(in crate::runtime) fn eval_trace_detail(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_trace_named(args, "TraceDetail", TraceOutput::Detail)
    }

    pub(in crate::runtime) fn eval_trace_print(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_trace_named(args, "TracePrint", TraceOutput::Print)
    }

    fn eval_trace_named(
        &mut self,
        args: &[Expr],
        name: &'static str,
        output: TraceOutput,
    ) -> Result<Expr, EvalError> {
        let default_include_heads = self.system_option_bool(name, "Heads", false)?;

        if let [expr] = args {
            return self.eval_structured_trace_capture_expr(TraceEvalState {
                original: expr.clone(),
                pattern: None,
                levels: None,
                include_heads: default_include_heads,
                output,
            });
        }

        let (expr, pattern, levels, include_heads) = match args {
            [expr, pattern] => (expr, Some(pattern.clone()), None, default_include_heads),
            [expr, pattern, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr, Some(pattern.clone()), None, include_heads)
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return Ok(self.invalid_call_with_arg_form(
                            name,
                            args,
                            "expected a standard levelspec or Heads -> True|False in the current structured-trace subset",
                        ));
                    };
                    (
                        expr,
                        Some(pattern.clone()),
                        Some(levels),
                        default_include_heads,
                    )
                }
            }
            [expr, pattern, levels_expr, option] => {
                let Some(levels) = parse_standard_level_spec(levels_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        name,
                        args,
                        "expected a standard levelspec as the third argument in the current structured-trace subset",
                    ));
                };
                let Some(include_heads) =
                    parse_heads_option_with_default(option, default_include_heads)
                else {
                    return Ok(self.invalid_call_with_arg_form(
                        name,
                        args,
                        "expected Heads -> True|False as the optional fourth argument in the current structured-trace subset",
                    ));
                };
                (expr, Some(pattern.clone()), Some(levels), include_heads)
            }
            _ => return Ok(self.invalid_call_with_arg_range(name, args, 2, 4)),
        };

        self.eval_structured_trace_capture_expr(TraceEvalState {
            original: expr.clone(),
            pattern,
            levels,
            include_heads,
            output,
        })
    }

    fn eval_structured_trace_capture_expr(
        &mut self,
        state: TraceEvalState,
    ) -> Result<Expr, EvalError> {
        let prior_enabled = self.tracing_enabled;
        let trace_checkpoint = self.trace_events.len();
        self.tracing_enabled = true;

        let value_result = self.evaluate(state.original.clone());
        let outcome = crate::resume_or_suspend!(
            self,
            value_result,
            ok(value) => Ok(value),
            suspend => {
        let suspended_state = state.clone();
        EvalStructuredTraceCaptureExprSuspensionFrame { suspended_state, trace_checkpoint, prior_enabled }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_structured_trace_capture_expr(
                state,
                trace_checkpoint,
                prior_enabled,
                outcome,
            ),
        }
    }

    fn finish_structured_trace_capture_expr(
        &mut self,
        state: TraceEvalState,
        trace_checkpoint: usize,
        prior_enabled: bool,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let value = match outcome {
            Ok(value) => value,
            Err(error) => {
                self.restore_trace_capture_on_error(trace_checkpoint, prior_enabled);
                return Err(error);
            }
        };

        let (value, events, depths) =
            self.finish_trace_event_capture(trace_checkpoint, prior_enabled, value)?;
        let trace = if state.output == TraceOutput::Detail {
            let trace =
                self.structured_trace_detail_expr_from_events(&state.original, &events, &depths);
            self.filter_structured_trace_detail_expr(
                trace,
                state.pattern,
                state.levels,
                state.include_heads,
            )?
        } else {
            let trace = self.structured_trace_expr_from_events(&state.original, &events, &depths);
            self.filter_structured_trace_expr(
                trace,
                state.pattern,
                state.levels,
                state.include_heads,
            )?
        };

        if state.output == TraceOutput::Print {
            for line in render_structured_trace_lines(&trace) {
                self.push_print(line);
            }
            Ok(value)
        } else {
            Ok(trace)
        }
    }

    fn finish_trace_event_capture(
        &mut self,
        trace_checkpoint: usize,
        prior_enabled: bool,
        value: Expr,
    ) -> Result<(Expr, Vec<TraceEvent>, Vec<usize>), EvalError> {
        let events = self.trace_events[trace_checkpoint..].to_vec();
        let depths = self.trace_event_depths[trace_checkpoint..].to_vec();
        self.tracing_enabled = prior_enabled;

        if !prior_enabled {
            self.truncate_trace_from(trace_checkpoint);
        }

        Ok((value, events, depths))
    }

    fn restore_trace_capture_on_error(&mut self, trace_checkpoint: usize, prior_enabled: bool) {
        self.tracing_enabled = prior_enabled;
        if !prior_enabled {
            self.truncate_trace_from(trace_checkpoint);
        }
    }

    pub(in crate::runtime) fn abandon_trace_capture(
        &mut self,
        trace_checkpoint: usize,
        prior_enabled: bool,
    ) {
        self.restore_trace_capture_on_error(trace_checkpoint, prior_enabled);
    }

    pub(in crate::runtime::debug_tools) fn truncate_trace_from(&mut self, trace_checkpoint: usize) {
        self.trace_events.truncate(trace_checkpoint);
        self.trace_event_depths.truncate(trace_checkpoint);
    }

    pub(in crate::runtime::debug_tools) fn structured_trace_expr_from_events(
        &self,
        original: &Expr,
        events: &[TraceEvent],
        depths: &[usize],
    ) -> Expr {
        let mut builder = StructuredTraceBuilder::new();
        builder.push_expr(0, original.clone());
        let mut active_exprs = vec![original.clone()];

        for (event, raw_depth) in events.iter().zip(depths.iter().copied()) {
            match event {
                TraceEvent::EvaluationPass { before, after } => {
                    let depth = trace_expr_path_depth(&active_exprs, before).unwrap_or(raw_depth);
                    active_exprs.truncate(depth + 1);
                    builder.push_evaluation_pass(depth, before.clone(), after.clone());
                    if active_exprs.len() <= depth {
                        active_exprs.resize(depth + 1, original.clone());
                    }
                    active_exprs[depth] = after.clone();
                }
                TraceEvent::DefinitionStored { value, .. } => {
                    let depth = raw_depth;
                    builder.push_expr(depth, value.clone());
                }
                TraceEvent::RuleApplied { input, result, .. } => {
                    let depth = raw_depth;
                    active_exprs.truncate(depth + 1);
                    if active_exprs.len() <= depth {
                        active_exprs.resize(depth + 1, original.clone());
                    }
                    active_exprs[depth] = input.clone();
                    builder.push_expr(depth, input.clone());
                    builder.push_expr(depth.saturating_add(1), result.clone());
                    if active_exprs.len() <= depth + 1 {
                        active_exprs.push(result.clone());
                    } else {
                        active_exprs.truncate(depth + 2);
                        active_exprs[depth + 1] = result.clone();
                    }
                }
            }
        }

        builder.finish()
    }

    fn structured_trace_detail_expr_from_events(
        &self,
        original: &Expr,
        events: &[TraceEvent],
        depths: &[usize],
    ) -> Expr {
        let mut builder = StructuredTraceBuilder::new();
        let mut active_exprs = vec![original.clone()];

        for (event, raw_depth) in events.iter().zip(depths.iter().copied()) {
            match event {
                TraceEvent::EvaluationPass { before, after } => {
                    let depth = trace_expr_path_depth(&active_exprs, before).unwrap_or(raw_depth);
                    active_exprs.truncate(depth + 1);
                    builder.push_item(depth, trace_detail_evaluation_pass(before, after));
                    if active_exprs.len() <= depth {
                        active_exprs.resize(depth + 1, original.clone());
                    }
                    active_exprs[depth] = after.clone();
                }
                TraceEvent::DefinitionStored {
                    symbol,
                    kind,
                    value,
                } => {
                    builder.push_item(
                        raw_depth,
                        trace_detail_definition_stored(symbol, *kind, value),
                    );
                }
                TraceEvent::RuleApplied {
                    source,
                    input,
                    rule,
                    result,
                } => {
                    let depth = raw_depth;
                    active_exprs.truncate(depth + 1);
                    if active_exprs.len() <= depth {
                        active_exprs.resize(depth + 1, original.clone());
                    }
                    active_exprs[depth] = input.clone();
                    builder.push_item(
                        depth,
                        trace_detail_rule_applied(*source, input, rule, result),
                    );
                    if active_exprs.len() <= depth + 1 {
                        active_exprs.push(result.clone());
                    } else {
                        active_exprs.truncate(depth + 2);
                        active_exprs[depth + 1] = result.clone();
                    }
                }
            }
        }

        builder.finish()
    }

    fn filter_structured_trace_expr(
        &mut self,
        trace: Expr,
        pattern: Option<Expr>,
        levels: Option<CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Expr, EvalError> {
        let Expr::Call { head, args, .. } = trace else {
            return Ok(trace);
        };
        if !is_system_symbol_name(&head, "List") {
            return Ok(Expr::call_shared(head, args));
        }

        let filtered = self.filter_structured_trace_items(
            args.into_vec(),
            pattern.as_ref(),
            levels.as_ref(),
            include_heads,
        )?;
        Ok(Expr::call(system_expr("List"), filtered))
    }

    fn filter_structured_trace_detail_expr(
        &mut self,
        trace: Expr,
        pattern: Option<Expr>,
        levels: Option<CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Expr, EvalError> {
        let Expr::Call { head, args, .. } = trace else {
            return Ok(trace);
        };
        if !is_system_symbol_name(&head, "List") {
            return Ok(Expr::call_shared(head, args));
        }

        let filtered = self.filter_structured_trace_detail_items(
            args.into_vec(),
            pattern.as_ref(),
            levels.as_ref(),
            include_heads,
        )?;
        Ok(Expr::call(system_expr("List"), filtered))
    }

    fn filter_structured_trace_detail_items(
        &mut self,
        items: Vec<Expr>,
        pattern: Option<&Expr>,
        levels: Option<&CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Vec<Expr>, EvalError> {
        let mut filtered = Vec::new();

        for item in items {
            match item {
                Expr::Call { head, args, .. } if is_system_symbol_name(&head, "List") => {
                    let nested = self.filter_structured_trace_detail_items(
                        args.into_vec(),
                        pattern,
                        levels,
                        include_heads,
                    )?;
                    if !nested.is_empty() {
                        if nested.len() == 1 {
                            if let Expr::Call { head, args, .. } = &nested[0] {
                                if is_system_symbol_name(head, "List") {
                                    filtered.push(Expr::call_with_shared_args(
                                        system_expr("List"),
                                        args.clone(),
                                    ));
                                    continue;
                                }
                            }
                        }
                        filtered.push(Expr::call(system_expr("List"), nested));
                    }
                }
                other => {
                    if self.trace_detail_item_matches(&other, pattern, levels, include_heads)? {
                        filtered.push(other);
                    }
                }
            }
        }

        Ok(filtered)
    }

    fn trace_detail_item_matches(
        &mut self,
        item: &Expr,
        pattern: Option<&Expr>,
        levels: Option<&CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<bool, EvalError> {
        let Some(exprs) = trace_detail_underlying_exprs(item) else {
            return Ok(pattern.is_none() && levels.is_none());
        };

        for expr in exprs {
            if self.trace_detail_underlying_expr_matches(&expr, pattern, levels, include_heads)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn trace_detail_underlying_expr_matches(
        &mut self,
        expr: &Expr,
        pattern: Option<&Expr>,
        levels: Option<&CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<bool, EvalError> {
        match (pattern, levels) {
            (Some(pattern), Some(levels)) => {
                let patterns = normalize_trace_patterns(pattern);
                let traversal = CasesTraversalConfig {
                    levels: levels.clone(),
                    max_matches: None,
                    include_heads,
                };

                for candidate in collect_level_parts_owned(expr, traversal, 0) {
                    if self.trace_detail_candidate_matches(&patterns, &candidate)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            (Some(pattern), None) if include_heads => {
                let patterns = normalize_trace_patterns(pattern);
                let traversal = CasesTraversalConfig {
                    levels: CasesLevelSpec::range(0, None),
                    max_matches: None,
                    include_heads: true,
                };

                for candidate in collect_level_parts_owned(expr, traversal, 0) {
                    if self.trace_detail_candidate_matches(&patterns, &candidate)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            (Some(pattern), None) => {
                let patterns = normalize_trace_patterns(pattern);
                self.trace_detail_candidate_matches(&patterns, expr)
            }
            (None, Some(levels)) => {
                let traversal = CasesTraversalConfig {
                    levels: levels.clone(),
                    max_matches: Some(1),
                    include_heads,
                };
                Ok(!collect_level_parts_owned(expr, traversal, 0).is_empty())
            }
            (None, None) => Ok(true),
        }
    }

    fn trace_detail_candidate_matches(
        &mut self,
        patterns: &[Expr],
        candidate: &Expr,
    ) -> Result<bool, EvalError> {
        for pattern in patterns {
            let outcome = self.match_expr_owned(
                pattern.clone(),
                candidate.clone(),
                SymbolExprMap::default(),
            )?;
            if outcome.matched {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn filter_structured_trace_items(
        &mut self,
        items: Vec<Expr>,
        pattern: Option<&Expr>,
        levels: Option<&CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Vec<Expr>, EvalError> {
        let mut filtered = Vec::new();

        for item in items {
            match item {
                Expr::Call { head, args, .. } if is_system_symbol_name(&head, "List") => {
                    let nested = self.filter_structured_trace_items(
                        args.into_vec(),
                        pattern,
                        levels,
                        include_heads,
                    )?;
                    if !nested.is_empty() {
                        if nested.len() == 1 {
                            if let Expr::Call { head, args, .. } = &nested[0] {
                                if is_system_symbol_name(head, "List") {
                                    filtered.push(Expr::call_with_shared_args(
                                        system_expr("List"),
                                        args.clone(),
                                    ));
                                    continue;
                                }
                            }
                        }
                        filtered.push(Expr::call(system_expr("List"), nested));
                    }
                }
                other => {
                    for held in
                        self.filter_structured_trace_leaf(other, pattern, levels, include_heads)?
                    {
                        push_unique_trace_item(&mut filtered, held);
                    }
                }
            }
        }

        Ok(filtered)
    }

    fn filter_structured_trace_leaf(
        &mut self,
        item: Expr,
        pattern: Option<&Expr>,
        levels: Option<&CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Vec<Expr>, EvalError> {
        let Some(expr) = unwrap_trace_hold(&item) else {
            return Ok(vec![item]);
        };

        match (pattern, levels) {
            (Some(pattern), Some(levels)) => {
                let patterns = normalize_trace_patterns(pattern);
                let traversal = CasesTraversalConfig {
                    levels: levels.clone(),
                    max_matches: None,
                    include_heads,
                };
                let mut matched = Vec::new();

                for candidate in collect_level_parts_owned(&expr, traversal, 0) {
                    let mut is_match = false;
                    for pattern in &patterns {
                        let outcome = self.match_expr_owned(
                            pattern.clone(),
                            candidate.clone(),
                            SymbolExprMap::default(),
                        )?;
                        if outcome.matched {
                            is_match = true;
                            break;
                        }
                    }
                    if is_match {
                        push_unique_trace_expr(&mut matched, candidate);
                    }
                }

                Ok(matched
                    .into_iter()
                    .map(|candidate| Expr::call(system_expr("Hold"), vec![candidate]))
                    .collect())
            }
            (Some(pattern), None) => {
                if include_heads {
                    let all_levels = CasesLevelSpec::range(0, None);
                    self.filter_structured_trace_leaf(
                        Expr::call(system_expr("Hold"), vec![expr]),
                        Some(pattern),
                        Some(&all_levels),
                        true,
                    )
                } else {
                    let mut is_match = false;
                    for pattern in normalize_trace_patterns(pattern) {
                        let outcome =
                            self.match_expr_owned(pattern, expr.clone(), SymbolExprMap::default())?;
                        if outcome.matched {
                            is_match = true;
                            break;
                        }
                    }
                    Ok(if is_match {
                        vec![Expr::call(system_expr("Hold"), vec![expr])]
                    } else {
                        Vec::new()
                    })
                }
            }
            (None, Some(levels)) => {
                let traversal = CasesTraversalConfig {
                    levels: levels.clone(),
                    max_matches: None,
                    include_heads,
                };
                let mut selected = Vec::new();
                for candidate in collect_level_parts_owned(&expr, traversal, 0) {
                    push_unique_trace_expr(&mut selected, candidate);
                }
                Ok(selected
                    .into_iter()
                    .map(|candidate| Expr::call(system_expr("Hold"), vec![candidate]))
                    .collect())
            }
            (None, None) => Ok(vec![Expr::call(system_expr("Hold"), vec![expr])]),
        }
    }
}

fn trace_detail_rule_applied(
    source: RuleApplicationSource,
    input: &Expr,
    rule: &Expr,
    result: &Expr,
) -> Expr {
    trace_detail_association(vec![
        ("Step", Expr::string("RuleApplied")),
        ("Source", Expr::string(rule_application_source_name(source))),
        ("Input", trace_detail_hold(input)),
        ("RuleLeft", trace_detail_hold(rule)),
        ("Result", trace_detail_hold(result)),
    ])
}

fn trace_detail_evaluation_pass(before: &Expr, after: &Expr) -> Expr {
    trace_detail_association(vec![
        ("Step", Expr::string("EvaluationPass")),
        ("Before", trace_detail_hold(before)),
        ("After", trace_detail_hold(after)),
    ])
}

fn trace_detail_definition_stored(symbol: &str, kind: DefinitionKind, value: &Expr) -> Expr {
    let symbol = if symbol.contains('`') {
        symbol.to_string()
    } else {
        format!("{GLOBAL_CONTEXT}{symbol}")
    };
    trace_detail_association(vec![
        ("Step", Expr::string("DefinitionStored")),
        ("Symbol", Expr::string(symbol)),
        ("Kind", Expr::string(definition_kind_name(kind))),
        ("Value", trace_detail_hold(value)),
    ])
}

fn trace_detail_association(entries: Vec<(&'static str, Expr)>) -> Expr {
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(|(key, value)| Expr::call(system_expr("Rule"), vec![Expr::string(key), value]))
            .collect(),
    )
}

fn trace_detail_hold(expr: &Expr) -> Expr {
    Expr::call(system_expr("Hold"), vec![expr.clone()])
}

fn trace_detail_underlying_exprs(item: &Expr) -> Option<Vec<Expr>> {
    let entries = association_entries_from_expr(item)?;
    let step = trace_detail_association_value(&entries, "Step")?;
    let Expr::Atom(Atom::String(step)) = step else {
        return None;
    };
    let fields: &[&str] = match step.as_str() {
        "RuleApplied" => &["Input", "Result"],
        "EvaluationPass" => &["Before", "After"],
        "DefinitionStored" => &["Value"],
        _ => return None,
    };
    fields
        .iter()
        .map(|field| unwrap_trace_hold(trace_detail_association_value(&entries, field)?))
        .collect()
}

fn trace_detail_association_value<'a>(entries: &'a [(Expr, Expr)], key: &str) -> Option<&'a Expr> {
    entries
        .iter()
        .find_map(|(entry_key, value)| match entry_key {
            Expr::Atom(Atom::String(name)) if name == key => Some(value),
            _ => None,
        })
}

fn definition_kind_name(kind: DefinitionKind) -> &'static str {
    match kind {
        DefinitionKind::OwnValue => "OwnValue",
        DefinitionKind::DelayedOwnValue => "DelayedOwnValue",
        DefinitionKind::UpValue => "UpValue",
        DefinitionKind::DownValue => "DownValue",
        DefinitionKind::SubValue => "SubValue",
    }
}

fn rule_application_source_name(source: RuleApplicationSource) -> &'static str {
    match source {
        RuleApplicationSource::UpValue => "UpValue",
        RuleApplicationSource::DownValue => "DownValue",
        RuleApplicationSource::SubValue => "SubValue",
        RuleApplicationSource::Replace => "Replace",
        RuleApplicationSource::ReplaceList => "ReplaceList",
        RuleApplicationSource::ReplaceAt => "ReplaceAt",
        RuleApplicationSource::ReplaceAll => "ReplaceAll",
        RuleApplicationSource::ReplaceAllUnheld => "ReplaceAllUnheld",
        RuleApplicationSource::ReplaceRepeated => "ReplaceRepeated",
    }
}

fn push_unique_trace_expr(trace: &mut Vec<Expr>, expr: Expr) {
    if trace.last().is_some_and(|current| current == &expr) {
        return;
    }
    trace.push(expr);
}

fn push_unique_trace_item(items: &mut Vec<Expr>, item: Expr) {
    if items.last().is_some_and(|current| current == &item) {
        return;
    }
    items.push(item);
}

fn unwrap_trace_hold(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "Hold") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn render_structured_trace_lines(trace: &Expr) -> Vec<String> {
    let mut lines = Vec::new();
    collect_structured_trace_lines(trace, 0, &mut lines);
    let min_depth = lines.iter().map(|(depth, _)| *depth).min().unwrap_or(0);
    lines
        .into_iter()
        .map(|(depth, line)| format!("{}{}", "  ".repeat(depth.saturating_sub(min_depth)), line))
        .collect()
}

fn collect_structured_trace_lines(trace: &Expr, depth: usize, lines: &mut Vec<(usize, String)>) {
    match trace {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
            for item in args {
                match item {
                    Expr::Call { head, args: _, .. } if is_system_symbol_name(head, "List") => {
                        collect_structured_trace_lines(item, depth + 1, lines);
                    }
                    other => {
                        if let Some(expr) = unwrap_trace_hold(other) {
                            lines.push((depth, render_echo_fragment(&expr)));
                        } else {
                            lines.push((depth, render_echo_fragment(other)));
                        }
                    }
                }
            }
        }
        other => {
            if let Some(expr) = unwrap_trace_hold(other) {
                lines.push((depth, render_echo_fragment(&expr)));
            } else {
                lines.push((depth, render_echo_fragment(other)));
            }
        }
    }
}
fn normalize_trace_patterns(expr: &Expr) -> Vec<Expr> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => args.to_vec(),
        other => vec![other.clone()],
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<TraceEvalState>();
    assert_traces::<TraceOutput>();
};
