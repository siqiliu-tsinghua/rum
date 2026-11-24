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

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime) fn evaluate_definition_rhs_cm740_driven(
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
            let rhs_result = crate::cm740_probe::around("P0287", || {
                self.evaluate_with_option_value_scope(option_scope, rhs)
            });
            let witness = slot_witness(&rhs_result);
            record_profile_runtime_path_timing_adopted(self, runtime_profile_evaluate, &rhs_result);
            let outcome = match rhs_result {
                Ok(value) => Ok(value),
                Err(EvalError::InputRequired { request }) => {
                    #[inline(never)]
                    fn cm740_cold_P0289(
                        session: &mut Session,
                        witness: SlotWitness<Expr>,
                        request: RuntimeInputRequest,
                    ) -> Result<Expr, EvalError> {
                        let _guard = crate::cm740_probe::enter("P0289");
                        let owner_before = session.pending_suspension_generation();
                        let original_request = request.clone();
                        let actual = session.suspend_definition_rhs_evaluation(witness, request);
                        if crate::cm740_probe::class_lease::wants_input_return("P0289") {
                            if let Err(crate::EvalError::InputRequired { .. }) = &actual {
                                _guard.input_return(&actual);
                                crate::cm740_probe::class_lease::note_input_return("P0289", 0);
                            }
                        }

                        if let Err(EvalError::InputRequired { request: returned }) = &actual {
                            assert!(owner_before.is_some());
                            assert_eq!(owner_before, session.pending_suspension_generation());
                            assert_eq!(&original_request, returned);
                            assert_eq!(session.pending_input_request(), Some(returned));
                            eprintln!(
                                "C17_RETURNED_OWNERSHIP row=P0289 generation={:?} request={:?} witness_consumed=true owner_preserved=true",
                                owner_before, returned
                            );
                            crate::cm740_probe::returned("P0289", &returned.prompt);
                        }
                        if crate::cm740_probe::frame("P0289") {
                            if let Err(error) = actual {
                                crate::cm740_probe::hit("P0289", "OmitActualRhsReturnedAccounting");
                                return Err(error);
                            }
                        }
                        actual
                    }
                    cm740_cold_P0289(self, witness, request)
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
    #[allow(dead_code)]
    pub(super) fn evaluate_definition_application_rhs_cm740_driven(
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
            let rhs_result = crate::cm740_probe::around_ok(
                "P0292",
                "ActualTypedCarrierBeforeOuterTail",
                false,
                || self.evaluate_with_option_value_scope(option_scope, rhs),
            );
            let witness = slot_witness(&rhs_result);
            record_profile_runtime_path_timing_adopted(self, runtime_profile_evaluate, &rhs_result);
            match rhs_result {
                Err(EvalError::InputRequired { request }) => {
                    let result = {
                        #[inline(never)]
                        fn cm740_cold_P0294(
                            session: &mut Session,
                            witness: SlotWitness<Expr>,
                            request: RuntimeInputRequest,
                        ) -> Result<Option<Expr>, EvalError> {
                            let _guard = crate::cm740_probe::enter("P0294");
                            let owner_before = session.pending_suspension_generation();
                            let original_request = request.clone();
                            let actual = session
                                .suspend_definition_application_rhs_evaluation(witness, request);
                            if crate::cm740_probe::class_lease::wants_input_return("P0294") {
                                if let Err(crate::EvalError::InputRequired { .. }) = &actual {
                                    _guard.input_return(&actual);
                                    crate::cm740_probe::class_lease::note_input_return("P0294", 0);
                                }
                            }

                            if let Err(EvalError::InputRequired { request: returned }) = &actual {
                                assert!(owner_before.is_some());
                                assert_eq!(owner_before, session.pending_suspension_generation());
                                assert_eq!(&original_request, returned);
                                assert_eq!(session.pending_input_request(), Some(returned));
                                eprintln!(
                                    "C17_RETURNED_OWNERSHIP row=P0294 generation={:?} request={:?} witness_consumed=true owner_preserved=true",
                                    owner_before, returned
                                );
                                crate::cm740_probe::returned("P0294", &returned.prompt);
                            }
                            if crate::cm740_probe::frame("P0294") {
                                if let Err(error) = actual {
                                    crate::cm740_probe::hit(
                                        "P0294",
                                        "OmitActualRhsReturnedAccounting",
                                    );
                                    return Err(error);
                                }
                            }
                            actual
                        }
                        cm740_cold_P0294(self, witness, request)
                    };
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
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_20be(self, session, outcome, _token);
        }

        let resumed = (|| session.finish_definition_rhs_evaluation(outcome))();
        crate::runtime::kernel_api::TypedAction::complete(resumed)
    }
}
impl EvaluateDefinitionRhsSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_20be<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::kernel_api::ContinueToken<'resume, Self>,
    ) -> crate::runtime::kernel_api::TypedAction<Expr> {
        if crate::cm740_probe::frame("P0287") {
            crate::cm740_probe::hit("P0287", "ActualExprBeforeRhsReturnFinish");
            return crate::runtime::kernel_api::TypedAction::complete(outcome);
        }
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
