use super::*;
use crate::runtime::suspension::trace::FrameRootVisitor;
use std::sync::OnceLock;

const GENSYM_GC_THRESHOLD_ENV: &str = "RUM_GENSYM_GC_THRESHOLD";
const DEFAULT_GENSYM_GC_THRESHOLD: usize = 200_000;

impl Session {
    pub(in crate::runtime) fn collect_and_sweep_dead_gensyms(&mut self, extra_roots: &[Expr]) {
        if self.gc_candidate_gensyms.is_empty() {
            return;
        }

        #[cfg(test)]
        {
            self.gensym_gc_sweep_count = self.gensym_gc_sweep_count.saturating_add(1);
        }

        let mut live = std::collections::HashSet::new();
        let mut worklist = Vec::new();

        for activation in &self.module_activations {
            for gensym in &activation.gensyms {
                if self.gc_candidate_gensyms.contains(gensym) && live.insert(*gensym) {
                    worklist.push(*gensym);
                }
            }
        }

        for root in extra_roots {
            seed_live_gensyms_from_expr(root, &self.gc_candidate_gensyms, &mut live, &mut worklist);
        }

        self.for_each_root_expr(&mut |root| {
            seed_live_gensyms_from_expr(root, &self.gc_candidate_gensyms, &mut live, &mut worklist);
        });

        {
            let mut visitor = GensymSuspensionRootVisitor {
                candidates: &self.gc_candidate_gensyms,
                live: &mut live,
                worklist: &mut worklist,
            };
            for suspension in [
                self.pending_suspension.as_ref(),
                self.resuming_suspension.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                for entry in &suspension.active_stack {
                    visitor.visit_expr(&entry.expr);
                }
                suspension.trace_frame_roots(&mut visitor);
            }
        }

        while let Some(gensym) = worklist.pop() {
            let Some(definition) = self.definitions.get(&gensym) else {
                continue;
            };

            for_each_definition_expr(definition, &mut |expr| {
                seed_live_gensyms_from_expr(
                    expr,
                    &self.gc_candidate_gensyms,
                    &mut live,
                    &mut worklist,
                );
            });
        }

        let dead = self
            .gc_candidate_gensyms
            .iter()
            .filter(|gensym| !live.contains(*gensym))
            .copied()
            .collect::<Vec<_>>();
        if !dead.is_empty() {
            self.in_gensym_sweep = true;
            for gensym in dead {
                self.clear_symbol_state(&gensym, true, true, true, true, true);
                self.gc_candidate_gensyms.remove(&gensym);
            }
            self.in_gensym_sweep = false;
        }

        self.rearm_gensym_gc_watermark();
    }

    pub(in crate::runtime) fn remove_gc_candidate_gensym(&mut self, gensym: &Symbol) {
        if self.gc_candidate_gensyms.remove(gensym) {
            self.rearm_gensym_gc_watermark();
        }
    }

    pub(in crate::runtime) fn remove_gc_candidate_gensyms(&mut self, gensyms: &[Symbol]) {
        let previous_len = self.gc_candidate_gensyms.len();
        for gensym in gensyms {
            self.gc_candidate_gensyms.remove(gensym);
        }
        if self.gc_candidate_gensyms.len() != previous_len {
            self.rearm_gensym_gc_watermark();
        }
    }

    pub(in crate::runtime) fn restore_gc_candidate_gensyms(
        &mut self,
        candidates: std::collections::HashSet<Symbol>,
    ) {
        self.gc_candidate_gensyms = candidates;
        self.rearm_gensym_gc_watermark();
    }

    fn rearm_gensym_gc_watermark(&mut self) {
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::GcCandidateGensyms,
        );
        self.gensym_gc_watermark =
            gensym_gc_watermark(self.gc_candidate_gensyms.len(), gensym_gc_threshold());
    }

    #[cfg(test)]
    pub(in crate::runtime) fn gc_candidate_gensym_count(&self) -> usize {
        self.gc_candidate_gensyms.len()
    }

    #[cfg(test)]
    pub(in crate::runtime) fn retained_candidate_definition_count(&self) -> usize {
        self.gc_candidate_gensyms
            .iter()
            .filter(|candidate| self.definitions.contains_key(candidate))
            .count()
    }

    pub(in crate::runtime) fn for_each_root_expr(&self, visit: &mut dyn FnMut(&Expr)) {
        // Keep in sync with `current_memory_proxy_bytes`; both must cover every
        // Expr-holding Session field. Candidate definitions are not roots: they
        // are followed transitively only after a real root marks the candidate.
        for (symbol, definition) in &self.definitions {
            if self.gc_candidate_gensyms.contains(symbol) {
                continue;
            }
            for_each_definition_expr(definition, visit);
        }

        for options in self.options.values() {
            for (_, value, _) in options.iter() {
                visit(value);
            }
        }

        for expr in self.syntax_information.values() {
            visit(expr);
        }

        for definition in self.message_definitions.values() {
            if let Some(expr) = &definition.immediate {
                visit(expr);
            }
            if let Some(expr) = &definition.delayed {
                visit(expr);
            }
        }

        for expr in self.once_values.values() {
            visit(expr);
        }

        for scope in &self.option_value_scopes {
            for (_, value, _) in scope.explicit().iter() {
                visit(value);
            }
        }

        for frame in &self.dynamic_own_values {
            if let Some(expr) = &frame.value {
                visit(expr);
            }
        }

        for entry in &self.evaluation_stack {
            visit(&entry.expr);
        }

        for scope in &self.reap_scopes {
            for pattern in &scope.patterns {
                visit(pattern);
            }
            if let Some(handler) = &scope.handler {
                visit(handler);
            }
            for bucket in scope.buckets_by_pattern.iter().flatten() {
                visit(&bucket.tag);
                for value in &bucket.values {
                    visit(value);
                }
            }
        }

        for scope in &self.monitor_scopes {
            visit(&scope.display_expr);
            if let Some(expr) = &scope.evaluation_monitor {
                visit(expr);
            }
            if let Some(expr) = &scope.step_monitor {
                visit(expr);
            }
        }

        // A rollback can drop the session copy of a trace event after it was
        // published. The reported copy stays reachable until the request ends.
        if let Some(pending) = &self.pending_request {
            for event in &pending.report().reported.trace {
                match event {
                    crate::runtime::model::TraceEvent::EvaluationPass { before, after } => {
                        visit(before);
                        visit(after);
                    }
                    crate::runtime::model::TraceEvent::DefinitionStored { value, .. } => {
                        visit(value);
                    }
                    crate::runtime::model::TraceEvent::RuleApplied {
                        input,
                        rule,
                        result,
                        ..
                    } => {
                        visit(input);
                        visit(rule);
                        visit(result);
                    }
                }
            }
        }
    }
}

struct GensymSuspensionRootVisitor<'a> {
    candidates: &'a std::collections::HashSet<Symbol>,
    live: &'a mut std::collections::HashSet<Symbol>,
    worklist: &'a mut Vec<Symbol>,
}

impl FrameRootVisitor for GensymSuspensionRootVisitor<'_> {
    fn visit_expr(&mut self, expr: &Expr) {
        seed_live_gensyms_from_expr(expr, self.candidates, self.live, self.worklist);
    }

    fn visit_symbol(&mut self, symbol: &Symbol) {
        if self.candidates.contains(symbol) && self.live.insert(*symbol) {
            self.worklist.push(*symbol);
        }
    }
}

pub(in crate::runtime) fn gensym_gc_threshold() -> usize {
    static THRESHOLD: OnceLock<usize> = OnceLock::new();
    *THRESHOLD.get_or_init(|| {
        std::env::var(GENSYM_GC_THRESHOLD_ENV)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_GENSYM_GC_THRESHOLD)
    })
}

pub(in crate::runtime) fn gensym_gc_watermark(candidate_count: usize, threshold: usize) -> usize {
    candidate_count.saturating_add(threshold)
}

pub(in crate::runtime) fn gensym_gc_extra_roots_for_outcome(
    outcome: &Result<Expr, EvalError>,
) -> Vec<Expr> {
    match outcome {
        Ok(expr) => vec![expr.clone()],
        Err(EvalError::Return { value }) => vec![value.clone()],
        Err(EvalError::Thrown { value, tag, .. }) => vec![value.clone(), tag.clone()],
        Err(EvalError::ThrownBypassingCatch {
            value,
            tag,
            cleanup_value,
            cleanup_tag,
            ..
        }) => vec![
            value.clone(),
            tag.clone(),
            cleanup_value.clone(),
            cleanup_tag.clone(),
        ],
        Err(_) => Vec::new(),
    }
}

pub(in crate::runtime) fn for_each_symbol_in_expr(expr: &Expr, visit: &mut dyn FnMut(&Symbol)) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => visit(symbol),
        Expr::Atom(_) => {}
        Expr::Call { head, args } => {
            for_each_symbol_in_expr(head, visit);
            for arg in args {
                for_each_symbol_in_expr(arg, visit);
            }
        }
    }
}

pub(in crate::runtime) fn seed_live_gensyms_from_expr(
    expr: &Expr,
    candidates: &std::collections::HashSet<Symbol>,
    live: &mut std::collections::HashSet<Symbol>,
    worklist: &mut Vec<Symbol>,
) {
    for_each_symbol_in_expr(expr, &mut |symbol| {
        if candidates.contains(symbol) && live.insert(*symbol) {
            worklist.push(*symbol);
        }
    });
}

pub(in crate::runtime) fn for_each_definition_expr(
    definition: &SymbolDefinition,
    visit: &mut dyn FnMut(&Expr),
) {
    if let Some(expr) = &definition.own_value {
        visit(expr);
    }
    if let Some(expr) = &definition.delayed_own_value {
        visit(expr);
    }
    for guard in definition.own_value_guards() {
        visit(guard);
    }
    for value in &definition.up_values {
        visit(&value.lhs);
        visit(&value.rhs);
        for guard in &value.guards {
            visit(guard);
        }
    }
    for value in &definition.down_values {
        visit(&value.lhs);
        visit(&value.rhs);
        for guard in &value.guards {
            visit(guard);
        }
    }
    for value in &definition.sub_values {
        visit(&value.lhs);
        visit(&value.rhs);
        for guard in &value.guards {
            visit(guard);
        }
    }
}
