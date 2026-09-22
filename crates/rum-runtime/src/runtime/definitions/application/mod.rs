use super::super::*;

mod default_values;
mod down_values;
mod fast_match;
mod replace_all;
mod rhs;
mod sub_values;
mod up_values;

pub(in crate::runtime) use replace_all::RewriteRulePlan;

impl Session {
    pub(in crate::runtime) fn push_trace(&mut self, event: TraceEvent) {
        let depth = self.evaluation_stack.len().saturating_sub(2);
        self.push_trace_with_depth(event, depth);
    }

    pub(in crate::runtime) fn push_trace_transition(&mut self, event: TraceEvent) {
        let depth = self.evaluation_stack.len();
        self.push_trace_with_depth(event, depth);
    }

    pub(in crate::runtime) fn push_definition_stored_trace<F>(
        &mut self,
        symbol: &Symbol,
        kind: DefinitionKind,
        value: F,
    ) where
        F: FnOnce() -> Expr,
    {
        if !self.tracing_enabled
            && (self.monitor_scopes.is_empty() || !self.monitor_callbacks_enabled)
        {
            return;
        }

        self.refresh_definition_memory();

        let event = TraceEvent::DefinitionStored {
            symbol: symbol.to_string(),
            kind,
            value: value(),
        };

        self.push_trace(event);
    }

    pub(in crate::runtime) fn push_rule_applied_trace(
        &mut self,
        source: RuleApplicationSource,
        input: &Expr,
        rule: &Expr,
        result: &Expr,
    ) {
        if !self.tracing_enabled
            && (self.monitor_scopes.is_empty() || !self.monitor_callbacks_enabled)
        {
            return;
        }

        let event = TraceEvent::RuleApplied {
            source,
            input: input.clone(),
            rule: rule.clone(),
            result: result.clone(),
        };

        self.push_trace(event);
    }

    #[inline]
    pub(in crate::runtime) fn push_evaluation_pass_trace(&mut self, before: &Expr, after: &Expr) {
        if !self.tracing_enabled
            && (self.monitor_scopes.is_empty() || !self.monitor_callbacks_enabled)
        {
            return;
        }

        self.push_evaluation_pass_trace_slow(before, after);
    }

    #[cold]
    #[inline(never)]
    fn push_evaluation_pass_trace_slow(&mut self, before: &Expr, after: &Expr) {
        let event = TraceEvent::EvaluationPass {
            before: before.clone(),
            after: after.clone(),
        };

        self.push_trace_transition(event);
    }

    fn push_trace_with_depth(&mut self, event: TraceEvent, depth: usize) {
        if self.tracing_enabled {
            self.push_accounted_trace_events(event.clone());
            self.push_accounted_trace_event_depths(depth);
        }

        self.handle_monitor_trace_event(&event);
    }
}

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    runtime_profile_outcome_for_result(result)
}

fn runtime_profile_outcome_for_option_result<T>(
    result: &Result<Option<T>, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(Some(_)) => RuntimeProfileOutcome::Some,
        Ok(None) => RuntimeProfileOutcome::None,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

fn runtime_profile_outcome_for_internal_result<T>(
    result: &Result<T, EvalError>,
) -> RuntimeProfileOutcome {
    runtime_profile_outcome_for_result(result)
}

fn runtime_profile_outcome_for_result<T>(result: &Result<T, EvalError>) -> RuntimeProfileOutcome {
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

fn runtime_profile_outcome_for_optional_fast_match(
    outcome: &Option<CompiledMatchOutcome>,
) -> RuntimeProfileOutcome {
    if outcome.is_some() {
        RuntimeProfileOutcome::Some
    } else {
        RuntimeProfileOutcome::None
    }
}

fn default_value_lhs(target: Expr, position: Option<usize>) -> Expr {
    let mut args = vec![target];
    if let Some(position) = position {
        args.push(Expr::integer(BigInt::from(position)));
    }
    Expr::call(system_expr(heads::patterns::DEFAULT), args)
}

fn exact_down_value_rhs(down_values: &[DownValue], lhs: &Expr) -> Option<Expr> {
    down_values
        .iter()
        .find(|down_value| down_value.lhs == *lhs)
        .map(|down_value| down_value.rhs.clone())
}
