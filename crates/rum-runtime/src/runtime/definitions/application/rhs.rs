use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn evaluate_definition_rhs(
        &mut self,
        option_scope: Option<OptionValueScope>,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        let profile_start = lexical_profile_start();
        let eval_cache_profile_start =
            evaluation_cache_profile_source_timing_start("definition_rhs_eval_by_source");
        let runtime_profile_total = runtime_path_timing_start("definition.rhs.total");
        let result = if option_scope.is_none() && expr_is_self_evaluating_atom(&rhs) {
            Ok(rhs)
        } else {
            let runtime_profile_evaluate = runtime_path_timing_start("definition.rhs.evaluate");
            let rhs_result = self.evaluate_with_option_value_scope(option_scope, rhs);
            record_profile_runtime_path_timing(
                runtime_profile_evaluate,
                runtime_profile_outcome_for_expr_result(&rhs_result),
            );
            let outcome = match rhs_result {
                Ok(value) => Ok(value),
                Err(EvalError::InputRequired { request }) => {
                    self.suspend_definition_rhs_evaluation(request)
                }
                Err(error) => Err(error),
            };

            match outcome {
                Err(EvalError::InputRequired { request }) => {
                    Err(EvalError::InputRequired { request })
                }
                outcome => self.finish_definition_rhs_evaluation(outcome),
            }
        };
        record_profile_definition_rhs_eval(profile_start);
        record_profile_definition_rhs_eval_by_source(eval_cache_profile_start);
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    #[cold]
    #[inline(never)]
    fn suspend_definition_rhs_evaluation(
        &mut self,
        request: RuntimeInputRequest,
    ) -> Result<Expr, EvalError> {
        crate::resume_or_suspend!(insert self, EvaluateDefinitionRhsSuspensionFrame);
        Err(EvalError::InputRequired { request })
    }

    fn finish_definition_rhs_evaluation(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_finish = runtime_path_timing_start("definition.rhs.finish");
        let profile_outcome = runtime_profile_outcome_for_expr_result(&outcome);
        let result = match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Return { value }) => Ok(value),
            Err(error) => Err(error),
        };
        record_profile_runtime_path_timing(runtime_profile_finish, profile_outcome);
        result
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvaluateDefinitionRhsSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for EvaluateDefinitionRhsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = (|| session.finish_definition_rhs_evaluation(outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
