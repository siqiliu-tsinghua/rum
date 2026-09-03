mod match_normalization;
mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct FixedPointHistory {
    prefix: SharedArgs,
    tail: Option<Arc<FixedPointHistoryNode>>,
}

#[derive(FrameRootTrace)]
struct FixedPointHistoryNode {
    previous: Option<Arc<FixedPointHistoryNode>>,
    value: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedFixedPointState {
    current: Expr,
    evaluation_passes: usize,
    seen: FixedPointHistory,
}

impl FixedPointHistory {
    fn freeze(values: Vec<Expr>) -> Self {
        Self {
            prefix: SharedArgs::new(values),
            tail: None,
        }
    }

    fn contains(&self, value: &Expr) -> bool {
        if self.prefix.iter().any(|previous| previous == value) {
            return true;
        }

        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            if &current.value == value {
                return true;
            }
            node = current.previous.as_deref();
        }
        false
    }

    fn appended(&self, value: Expr) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: Some(Arc::new(FixedPointHistoryNode {
                previous: self.tail.clone(),
                value,
            })),
        }
    }
}

impl Session {
    /// Lowers parsed syntax using the session's current context resolution.
    pub fn lower_syntax_expr(&self, syntax: &SyntaxExpr) -> Result<Expr, EvalError> {
        lower_syntax_expr_with(syntax, &mut |raw| self.resolve_symbol_name(raw, true))
    }

    pub(super) fn builtin_spec_for_symbol(&self, symbol: &Symbol) -> Option<&BuiltinSpec> {
        symbol
            .is_system()
            .then(|| self.builtins.get(symbol))
            .flatten()
    }

    pub(super) fn builtin_dispatch_spec_for_symbol(&self, symbol: &Symbol) -> Option<BuiltinSpec> {
        if !symbol.is_system() {
            return None;
        }

        self.builtin_spec_for_symbol(symbol).copied()
    }

    pub(super) fn cached_builtin_dispatch_spec_for_symbol(
        &mut self,
        symbol: &Symbol,
    ) -> Option<BuiltinSpec> {
        if let Some(spec) = self.builtin_dispatch_spec_cache.get(symbol).copied() {
            return spec;
        }

        let spec = self.builtin_dispatch_spec_for_symbol(symbol);
        self.builtin_dispatch_spec_cache
            .insert(symbol.clone(), spec);
        spec
    }

    /// Resolves a symbol name through the active context path.
    pub fn resolve_symbol_name(&self, raw: &str, create_if_missing: bool) -> Option<Symbol> {
        if let Some((context, name)) = split_symbol_name(raw) {
            let context = normalize_context_name(context);
            return Some(self.active_symbol_for_name(&context, name));
        }

        let mut seen_contexts = BTreeSet::new();

        for context in self
            .context_path
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(self.current_context.as_str()))
        {
            if !seen_contexts.insert(context) {
                continue;
            }

            let symbol = self.active_symbol_for_name(context, raw);

            if self.symbol_exists(&symbol) {
                return Some(symbol);
            }
        }

        create_if_missing.then(|| self.active_symbol_for_name(&self.current_context, raw))
    }

    pub(super) fn active_symbol_for_name(&self, context: &str, name: &str) -> Symbol {
        if context == SYSTEM_CONTEXT {
            return Symbol::system(name);
        }

        self.symbol_generations
            .get(&symbol_generation_key(context, name))
            .copied()
            .map(|identity| Symbol::with_identity(context, name, identity))
            .unwrap_or_else(|| Symbol::new(context, name))
    }

    pub(super) fn symbol_exists(&self, symbol: &Symbol) -> bool {
        self.definitions.contains_key(symbol)
            || self.declared_stubs.contains(symbol)
            || self
                .attributes
                .get(symbol)
                .copied()
                .is_some_and(|attributes| !attributes.is_empty())
            || self
                .options
                .get(symbol)
                .is_some_and(|options| !options.is_empty())
            || self.syntax_information.contains_key(symbol)
            || self.message_symbol_counts.contains_key(symbol)
            || self.builtin_spec_for_symbol(symbol).is_some()
            || is_known_system_symbol(symbol)
    }

    pub(super) fn discount_message_symbol(&mut self, symbol: &Symbol) {
        if let Some(count) = self.message_symbol_counts.get_mut(symbol) {
            *count -= 1;
            if *count == 0 {
                self.message_symbol_counts.remove(symbol);
            }
        }
    }

    pub(super) fn rebuild_message_symbol_counts(&mut self) {
        self.message_symbol_counts.clear();
        let mut counts = std::mem::take(&mut self.message_symbol_counts);
        for key in self.message_definitions.keys() {
            *counts.entry(key.symbol).or_insert(0) += 1;
        }
        self.message_symbol_counts = counts;
    }

    pub(super) fn attribute_set_for_symbol(&self, symbol: &Symbol) -> AttributeSet {
        self.attributes
            .get(symbol)
            .copied()
            .unwrap_or_else(|| default_system_attribute_set(symbol))
    }

    pub(super) fn cached_attribute_set_for_symbol(&mut self, symbol: &Symbol) -> AttributeSet {
        if let Some(attributes) = self.attribute_set_cache.get(symbol).copied() {
            return attributes;
        }

        let attributes = self.attribute_set_for_symbol(symbol);
        self.attribute_set_cache.insert(symbol.clone(), attributes);
        attributes
    }

    pub(super) fn set_attribute_set_for_symbol(
        &mut self,
        symbol: Symbol,
        attributes: AttributeSet,
    ) {
        let previous = self.attribute_set_for_symbol(&symbol);
        if previous == attributes {
            return;
        }

        self.bump_attribute_revision();
        let default = default_system_attribute_set(&symbol);

        if attributes == default {
            self.attributes.remove(&symbol);
        } else {
            self.attributes.insert(symbol, attributes);
        }
    }

    pub(super) fn bump_attribute_revision(&mut self) {
        self.attribute_revision = self.attribute_revision.saturating_add(1);
        self.attribute_set_cache.clear();
        self.bump_evaluation_epoch();
    }

    pub(super) fn resolve_load_target(&self, spec: &str) -> Result<PathBuf, EvalError> {
        self.find_load_target(spec)
            .map(|(path, _)| path)
            .ok_or_else(|| {
                let (_, searched) = self.load_target_candidates(spec);
                EvalError::LoadPathNotFound {
                    target: spec.to_string(),
                    searched,
                }
            })
    }

    pub(super) fn load_search_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();

        if let Some(parent) = self
            .load_stack
            .last()
            .and_then(|path| path.parent().map(Path::to_path_buf))
        {
            roots.push(parent);
        }

        roots.extend(self.package_search_paths.iter().cloned());
        dedup_paths(roots)
    }

    pub(super) fn find_load_target(&self, spec: &str) -> Option<(PathBuf, Vec<String>)> {
        let (candidates, searched) = self.load_target_candidates(spec);
        let file_io = self.file_io()?;

        for candidate in candidates {
            let probe = file_io.probe(&candidate);
            if probe.is_file {
                let path = probe
                    .resolved_path
                    .unwrap_or_else(|| normalize_found_path(&candidate));
                return Some((strip_windows_verbatim_prefix(path), searched));
            }
        }

        None
    }

    pub(super) fn load_target_candidates(&self, spec: &str) -> (Vec<PathBuf>, Vec<String>) {
        let specs = if spec.ends_with('`') {
            package_candidates_for_context(spec)
        } else {
            vec![PathBuf::from(spec)]
        };

        let mut candidates = Vec::new();
        let mut searched = Vec::new();

        for spec in specs {
            if spec.is_absolute() {
                searched.push(spec.display().to_string());
                candidates.push(spec);
                continue;
            }

            for root in self.load_search_roots() {
                let path = root.join(&spec);
                searched.push(path.display().to_string());
                candidates.push(path);
            }
        }

        (candidates, searched)
    }

    /// Evaluates an expression to a fixed point under the current session state.
    pub fn evaluate(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        if self.evaluation_depth == 0 {
            let stack_marker = 0_u8;
            self.native_stack_base = (&stack_marker as *const u8) as usize;
            self.native_stack_next_check_depth = NATIVE_STACK_DEPTH_POLL_STRIDE;
        }
        self.record_profile_evaluate_entry();
        let runtime_profile = runtime_path_timing_start("evaluate.total");
        let result = self.evaluate_profiled_fixed_point(expr);
        record_profile_runtime_path_timing(
            runtime_profile,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    fn evaluate_profiled_fixed_point(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        let mut current = expr;
        let mut evaluation_passes = 0;
        let mut fixed_point_seen = Vec::new();

        loop {
            if evaluation_passes >= self.resource_limits.evaluation_pass_limit {
                return Err(EvalError::EvaluationPassLimitExceeded {
                    limit: self.resource_limits.evaluation_pass_limit,
                });
            }
            evaluation_passes += 1;

            self.enforce_active_resource_limits_for_expr(&current)?;
            let runtime_profile_stability = runtime_path_timing_start("evaluate.stability_check");
            let stable = self.cached_expr_is_evaluation_stable(&current);
            record_profile_runtime_path_timing(
                runtime_profile_stability,
                RuntimeProfileOutcome::Ok,
            );
            if stable {
                return Ok(current);
            }

            let track_cycle = should_track_evaluation_cycle(&current);
            if track_cycle {
                if fixed_point_seen.iter().any(|previous| previous == &current) {
                    return Err(EvalError::EvaluationCycleDetected {
                        expr: current.to_string(),
                    });
                }
                fixed_point_seen.push(current.clone());
            }

            let runtime_profile_once = runtime_path_timing_start("evaluate.once_unstable");
            let next_result = self.evaluate_once_unstable_after_input_check_with_cycle_tracking(
                current.clone(),
                track_cycle,
            );
            record_profile_runtime_path_timing(
                runtime_profile_once,
                runtime_profile_outcome_for_expr_result(&next_result),
            );
            let next = match next_result {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedFixedPointState {
                        current,
                        evaluation_passes,
                        seen: FixedPointHistory::freeze(fixed_point_seen),
                    };
                    crate::resume_or_suspend!(insert self, EvaluateProfiledFixedPointSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            };

            let runtime_profile_equal = runtime_path_timing_start("evaluate.fixed_point_equal");
            let fixed_point = next == current;
            record_profile_runtime_path_timing(runtime_profile_equal, RuntimeProfileOutcome::Ok);
            if fixed_point {
                return Ok(next);
            }

            let runtime_profile_update = runtime_path_timing_start("evaluate.trace_update");
            self.push_evaluation_pass_trace(&current, &next);

            current = next;
            record_profile_runtime_path_timing(runtime_profile_update, RuntimeProfileOutcome::Ok);
        }
    }

    fn resume_fixed_point_from(
        &mut self,
        mut state: SuspendedFixedPointState,
        next: Expr,
    ) -> Result<Expr, EvalError> {
        if next == state.current {
            return Ok(next);
        }

        self.push_evaluation_pass_trace(&state.current, &next);
        state.current = next;

        loop {
            if state.evaluation_passes >= self.resource_limits.evaluation_pass_limit {
                return Err(EvalError::EvaluationPassLimitExceeded {
                    limit: self.resource_limits.evaluation_pass_limit,
                });
            }
            state.evaluation_passes += 1;

            self.enforce_active_resource_limits_for_expr(&state.current)?;
            if self.cached_expr_is_evaluation_stable(&state.current) {
                return Ok(state.current);
            }

            let track_cycle = should_track_evaluation_cycle(&state.current);
            if track_cycle {
                if state.seen.contains(&state.current) {
                    return Err(EvalError::EvaluationCycleDetected {
                        expr: state.current.to_string(),
                    });
                }
                state.seen = state.seen.appended(state.current.clone());
            }

            let next_result = self.evaluate_once_unstable_after_input_check_with_cycle_tracking(
                state.current.clone(),
                track_cycle,
            );
            let next = match next_result {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeFixedPointFromSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            };

            if next == state.current {
                return Ok(next);
            }

            self.push_evaluation_pass_trace(&state.current, &next);
            state.current = next;
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FixedPointHistory>();
    assert_traces::<SuspendedFixedPointState>();
};

const EVALUATION_CYCLE_HISTORY_NODE_LIMIT: usize = 8;

pub(super) fn should_track_evaluation_cycle(expr: &Expr) -> bool {
    expr_node_count_with_limit(expr, EVALUATION_CYCLE_HISTORY_NODE_LIMIT).is_some()
}

pub(super) fn expr_node_count_with_limit(expr: &Expr, limit: usize) -> Option<usize> {
    fn walk(expr: &Expr, remaining: &mut usize) -> bool {
        if *remaining == 0 {
            return false;
        }
        *remaining -= 1;

        match expr {
            Expr::Atom(_) => true,
            Expr::Call { head, args, .. } => {
                if !walk(head, remaining) {
                    return false;
                }
                args.iter().all(|arg| walk(arg, remaining))
            }
        }
    }

    let mut remaining = limit;
    walk(expr, &mut remaining).then_some(limit - remaining)
}

pub(super) fn symbol_generation_key(context: &str, name: &str) -> (String, String) {
    (context.to_string(), name.to_string())
}

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}
