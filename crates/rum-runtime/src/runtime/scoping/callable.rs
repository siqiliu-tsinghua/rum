use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn try_apply_function(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let applied = if let Some(function) = parse_function(head) {
            match function {
                FunctionSpec::SlotBased { body, .. } => substitute_slots(body, args),
                FunctionSpec::Explicit { params, body, .. } => {
                    if args.len() < params.len() {
                        self.push_message(
                            "Function",
                            "argx",
                            format!(
                                "expected at least {} argument(s), got {}",
                                params.len(),
                                args.len()
                            ),
                        );
                        return Ok(Some(Expr::call(head.clone(), args.to_vec())));
                    }

                    let bindings = params.into_iter().zip(args.iter().cloned()).collect();
                    substitute_lexical_bindings(body, &bindings, &BTreeSet::new())
                }
            }
        } else if let Some(composition) = parse_composition(head) {
            apply_composition_to_args(&composition, args)
        } else if let Some(status) = self.apply_status_object_to_args(head, args)? {
            status
        } else if let Some(value) = self.apply_association_to_args(head, args) {
            value
        } else if let Some(invalid_key_application) =
            self.invalid_callable_key_application(head, args)
        {
            return Ok(Some(invalid_key_application));
        } else if let Some(wrapper) = apply_callable_operator_wrapper_to_args(head, args) {
            wrapper
        } else if let Some(symbol) = invalid_callable_operator_wrapper_argx_symbol(head, args) {
            self.push_message(
                symbol,
                "argx",
                format!(
                    "{head} called with {} arguments; 1 argument is expected.",
                    args.len()
                ),
            );
            return Ok(Some(Expr::call(head.clone(), args.to_vec())));
        } else if let Some(wrapper) = apply_callable_failsafe_wrapper_to_args(head, args) {
            wrapper
        } else {
            return Ok(None);
        };

        let runtime_profile_evaluate = runtime_path_timing_start("function.apply.evaluate");
        let applied_result = self.evaluate(applied);
        record_profile_runtime_path_timing(
            runtime_profile_evaluate,
            runtime_profile_outcome_for_expr_result(&applied_result),
        );
        let outcome = crate::resume_or_suspend!(
            self,
            applied_result,
            ok(value) => Ok(value),
            suspend => TryApplyFunctionSuspensionFrame,
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_function_application(outcome).map(Some),
        }
    }

    fn finish_function_application(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_finish = runtime_path_timing_start("function.apply.finish");
        let profile_outcome = runtime_profile_outcome_for_expr_result(&outcome);
        let result = match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Return { value }) => Ok(value),
            Err(error) => Err(error),
        };
        record_profile_runtime_path_timing(runtime_profile_finish, profile_outcome);
        result
    }

    fn invalid_callable_key_application(&mut self, head: &Expr, args: &[Expr]) -> Option<Expr> {
        let Expr::Call {
            head: key_head,
            args: key_args,
            ..
        } = head
        else {
            return None;
        };
        if !is_system_symbol_name(key_head, "Key") || key_args.len() != 1 {
            return None;
        }

        let held = Expr::call(head.clone(), args.to_vec());
        let [expr] = args else {
            self.push_message(
                "Key",
                "argx",
                format!(
                    "{head} called with {} arguments; 1 argument is expected.",
                    args.len()
                ),
            );
            return Some(held);
        };

        if association_entries_from_expr(expr).is_none() {
            self.push_message("Key", "invak", invalid_association_message(expr));
            return Some(held);
        }

        None
    }
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

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct TryApplyFunctionSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for TryApplyFunctionSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = (|| session.finish_function_application(outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
