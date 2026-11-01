use super::*;
use crate::runtime::kernel_api::{SlotWitness, slot_witness};
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
        let runtime_profile_total = runtime_path_timing_start_adopted(self, "definition.rhs.total");
        let result = if option_scope.is_none() && expr_is_self_evaluating_atom(&rhs) {
            Ok(rhs)
        } else {
            let runtime_profile_evaluate =
                runtime_path_timing_start_adopted(self, "definition.rhs.evaluate");
            let rhs_result = self.evaluate_with_option_value_scope(option_scope, rhs);
            let witness = slot_witness(&rhs_result);
            record_profile_runtime_path_timing_adopted(self, runtime_profile_evaluate, &rhs_result);
            let outcome = match rhs_result {
                Ok(value) => Ok(value),
                Err(EvalError::InputRequired { request }) => {
                    self.suspend_definition_rhs_evaluation(witness, request)
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
        record_profile_runtime_path_timing_adopted(self, runtime_profile_total, &result);
        result
    }

    // Definition application owns the Some on both completion paths. Keeping
    // the OwnValue entry separate preserves its Expr slot and one frame per pause.
    pub(super) fn evaluate_definition_application_rhs(
        &mut self,
        option_scope: Option<OptionValueScope>,
        rhs: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        let profile_start = lexical_profile_start();
        let eval_cache_profile_start =
            evaluation_cache_profile_source_timing_start("definition_rhs_eval_by_source");
        let runtime_profile_total = runtime_path_timing_start_adopted(self, "definition.rhs.total");
        let result = if option_scope.is_none() && expr_is_self_evaluating_atom(&rhs) {
            Ok(rhs)
        } else {
            let runtime_profile_evaluate =
                runtime_path_timing_start_adopted(self, "definition.rhs.evaluate");
            let rhs_result = self.evaluate_with_option_value_scope(option_scope, rhs);
            let witness = slot_witness(&rhs_result);
            record_profile_runtime_path_timing_adopted(self, runtime_profile_evaluate, &rhs_result);
            match rhs_result {
                Err(EvalError::InputRequired { request }) => {
                    let result =
                        self.suspend_definition_application_rhs_evaluation(witness, request);
                    record_profile_definition_rhs_eval(profile_start);
                    record_profile_definition_rhs_eval_by_source(eval_cache_profile_start);
                    record_profile_runtime_path_timing_adopted(
                        self,
                        runtime_profile_total,
                        &result,
                    );
                    return result;
                }
                outcome => self.finish_definition_rhs_evaluation(outcome),
            }
        };
        record_profile_definition_rhs_eval(profile_start);
        record_profile_definition_rhs_eval_by_source(eval_cache_profile_start);
        record_profile_runtime_path_timing_adopted(self, runtime_profile_total, &result);
        result.map(Some)
    }

    #[cold]
    #[inline(never)]
    fn suspend_definition_rhs_evaluation(
        &mut self,
        witness: SlotWitness<Expr>,
        request: RuntimeInputRequest,
    ) -> Result<Expr, EvalError> {
        witness.suspend(self, request, EvaluateDefinitionRhsSuspensionFrame)
    }

    #[cold]
    #[inline(never)]
    fn suspend_definition_application_rhs_evaluation(
        &mut self,
        witness: SlotWitness<Expr>,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        witness.suspend(
            self,
            request,
            EvaluateDefinitionApplicationRhsSuspensionFrame,
        )
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

impl crate::runtime::kernel_api::TypedFrame for EvaluateDefinitionRhsSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::kernel_api::ContinueToken<'resume, Self>,
    ) -> crate::runtime::kernel_api::TypedAction<Expr> {
        let resumed = (|| session.finish_definition_rhs_evaluation(outcome))();
        crate::runtime::kernel_api::TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvaluateDefinitionApplicationRhsSuspensionFrame;

impl crate::runtime::kernel_api::TypedFrame for EvaluateDefinitionApplicationRhsSuspensionFrame {
    type Input = Expr;
    type Output = Option<Expr>;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::kernel_api::ContinueToken<'resume, Self>,
    ) -> crate::runtime::kernel_api::TypedAction<Option<Expr>> {
        crate::runtime::kernel_api::TypedAction::complete(
            session.finish_definition_rhs_evaluation(outcome).map(Some),
        )
    }
}
