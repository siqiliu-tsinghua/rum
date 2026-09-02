use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_monitor(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_arg_range("Monitor", args, 2, usize::MAX));
        }

        let explicit_options = if args.len() > 2 {
            let resolved = self.evaluate(Expr::call(system_expr("List"), args[2..].to_vec()))?;
            let options = match self.compile_assigned_options_from_resolved(&resolved) {
                Ok(options) => options,
                Err(_) => {
                    return Ok(self.invalid_call_with_arg_form(
                        "Monitor",
                        args,
                        "expected the optional third argument to be a rule or list of rules in the current first-pass subset",
                    ));
                }
            };
            Some(options)
        } else {
            None
        };

        if let Some(options) = explicit_options.as_ref() {
            for (name, _, _) in options.iter() {
                if !matches!(name.name(), "EvaluationMonitor" | "StepMonitor") {
                    return Ok(self.invalid_call_with_arg_form(
                        "Monitor",
                        args,
                        "expected only EvaluationMonitor and StepMonitor option rules in the current first-pass subset",
                    ));
                }
            }
        }

        let evaluation_monitor =
            self.monitor_option_expr(explicit_options.as_ref(), "EvaluationMonitor")?;
        let step_monitor = self.monitor_option_expr(explicit_options.as_ref(), "StepMonitor")?;

        self.monitor_scopes.push(MonitorScope {
            display_expr: args[1].clone(),
            evaluation_monitor,
            step_monitor,
            last_display: None,
        });
        let last_index = self.monitor_scopes.len().saturating_sub(1);
        self.refresh_monitor_scope_display(last_index);

        let body_result = self.evaluate(args[0].clone());
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => EvalMonitorSuspensionFrame,
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_monitor(outcome),
        }
    }

    fn monitor_option_expr(
        &mut self,
        explicit: Option<&SymbolOptions>,
        name: &str,
    ) -> Result<Option<Expr>, EvalError> {
        let resolved =
            self.runtime_option_expr_for_symbol(explicit, &Symbol::system("Monitor"), name)?;

        Ok(match resolved {
            Some(expr) if is_system_symbol_name(&expr, "None") => None,
            other => other,
        })
    }

    fn finish_monitor(&mut self, outcome: Result<Expr, EvalError>) -> Result<Expr, EvalError> {
        let last_index = self.monitor_scopes.len().saturating_sub(1);
        self.refresh_monitor_scope_display(last_index);
        let _ = self.monitor_scopes.pop();
        outcome
    }

    pub(in crate::runtime) fn handle_monitor_trace_event(&mut self, event: &TraceEvent) {
        if self.monitor_scopes.is_empty() || !self.monitor_callbacks_enabled {
            return;
        }

        let scopes = self.monitor_scopes.clone();
        self.monitor_callbacks_enabled = false;

        for (index, scope) in scopes.into_iter().enumerate() {
            self.refresh_monitor_scope_display(index);

            if let Some(callback) = scope.step_monitor {
                self.run_monitor_callback_best_effort(callback);
            }

            if matches!(event, TraceEvent::EvaluationPass { .. }) {
                if let Some(callback) = scope.evaluation_monitor {
                    self.run_monitor_callback_best_effort(callback);
                }
            }
        }

        self.monitor_callbacks_enabled = true;
    }

    fn run_monitor_callback_best_effort(&mut self, callback: Expr) {
        match self.evaluate(callback) {
            Ok(_) => {}
            Err(EvalError::InputRequired { .. }) => {
                self.abandon_pending_suspension(AbandonReason::HelperBestEffortDiscard);
            }
            Err(_) => {}
        }
    }

    fn refresh_monitor_scope_display(&mut self, index: usize) {
        let Some(display_expr) = self
            .monitor_scopes
            .get(index)
            .map(|scope| scope.display_expr.clone())
        else {
            return;
        };

        match self.evaluate(display_expr) {
            Ok(value) => {
                let rendered = render_echo_fragment(&value);
                let should_emit = self
                    .monitor_scopes
                    .get(index)
                    .and_then(|current| current.last_display.as_ref())
                    .map_or(true, |current| current != &rendered);

                if should_emit {
                    self.push_print(rendered.clone());
                }

                if let Some(current) = self.monitor_scopes.get_mut(index) {
                    current.last_display = Some(rendered);
                }
            }
            Err(EvalError::InputRequired { .. }) => {
                self.abandon_pending_suspension(AbandonReason::HelperBestEffortDiscard);
            }
            Err(_) => {}
        }
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct EvalMonitorSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for EvalMonitorSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = (|| session.finish_monitor(outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalMonitorSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.pop_monitor_scope();
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}
