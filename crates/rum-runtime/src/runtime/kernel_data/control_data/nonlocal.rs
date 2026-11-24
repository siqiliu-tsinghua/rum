use super::*;
use crate::runtime::kernel_api::{
    ContinueToken, SlotWitness, TypedAction, TypedFrame, slot_witness,
};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_return(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let runtime_profile = runtime_path_timing_start("return.eval");
        let result = match args {
            [] => Err(EvalError::Return {
                value: system_expr("Null"),
            }),
            [value] => Err(EvalError::Return {
                value: value.clone(),
            }),
            _ => Ok(self.invalid_call_with_arg_range("Return", args, 0, 1)),
        };
        record_profile_runtime_path_timing(
            runtime_profile,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    pub(in crate::runtime) fn eval_throw(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let runtime_profile = runtime_path_timing_start("throw.eval");
        let result = match args {
            [value] => Err(EvalError::Thrown {
                value: value.clone(),
                tag: system_expr("None"),
                explicit_tag: false,
            }),
            [value, tag] => Err(EvalError::Thrown {
                value: value.clone(),
                tag: tag.clone(),
                explicit_tag: true,
            }),
            _ => Ok(self.invalid_call_with_arg_range("Throw", args, 1, 2)),
        };
        record_profile_runtime_path_timing(
            runtime_profile,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    pub(in crate::runtime) fn eval_break(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Break", args, 0));
        }

        Err(EvalError::Break)
    }

    pub(in crate::runtime) fn eval_continue(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Continue", args, 0));
        }

        Err(EvalError::Continue)
    }

    pub(in crate::runtime) fn eval_catch(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let runtime_profile_total = runtime_path_timing_start("catch.eval.total");
        if !(1..=3).contains(&args.len()) {
            let result = Ok(self.invalid_call_with_arg_range("Catch", args, 1, 3));
            self.record_profile_runtime_path_timing_adopted_boundary(
                runtime_profile_total,
                &result,
            );
            return result;
        }

        let form = args.get(1).cloned();
        let handler = args.get(2).cloned();
        let runtime_profile_body = runtime_path_timing_start("catch.body.evaluate");
        let body_result = self.evaluate(args[0].clone());
        let witness = slot_witness(&body_result);
        self.record_profile_runtime_path_timing_adopted_boundary(
            runtime_profile_body,
            &body_result,
        );
        let result = match body_result {
            Err(EvalError::InputRequired { request }) => {
                suspend_catch(self, form, handler, witness, request)
            }
            Ok(value) => self.resume_catch_after_body(form, handler, Ok(value)),
            Err(error) => self.resume_catch_after_body(form, handler, Err(error)),
        };
        self.record_profile_runtime_path_timing_adopted_boundary(runtime_profile_total, &result);
        result
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime) fn eval_catch_cm740_driven(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let runtime_profile_total = runtime_path_timing_start("catch.eval.total");
        if !(1..=3).contains(&args.len()) {
            let result = Ok(self.invalid_call_with_arg_range("Catch", args, 1, 3));
            self.record_profile_runtime_path_timing_adopted_boundary(
                runtime_profile_total,
                &result,
            );
            return result;
        }

        let form = args.get(1).cloned();
        let handler = args.get(2).cloned();
        let runtime_profile_body = runtime_path_timing_start("catch.body.evaluate");
        let body_result = self.evaluate_cm740_driven(args[0].clone());
        let witness = slot_witness(&body_result);
        self.record_profile_runtime_path_timing_adopted_boundary(
            runtime_profile_body,
            &body_result,
        );
        let result = match body_result {
            Err(EvalError::InputRequired { request }) => {
                let _guard = crate::cm740_probe::enter("P0812");
                let owner_before = self.pending_suspension_generation();
                let original_request = request.clone();
                let actual = suspend_catch_cm740_driven(self, form, handler, witness, request);
                if crate::cm740_probe::class_lease::wants_input_return("P0812") {
                    if let Err(crate::EvalError::InputRequired { .. }) = &actual {
                        _guard.input_return(&actual);
                        crate::cm740_probe::class_lease::note_input_return("P0812", 0);
                    }
                }

                if let Err(EvalError::InputRequired { request: returned }) = &actual {
                    assert!(owner_before.is_some());
                    assert_eq!(owner_before, self.pending_suspension_generation());
                    assert_eq!(&original_request, returned);
                    assert_eq!(self.pending_input_request(), Some(returned));
                    eprintln!(
                        "C18_RETURNED_OWNERSHIP row=P0812 generation={:?} request={:?} witness_consumed=true owner_preserved=true",
                        owner_before, returned
                    );
                    crate::cm740_probe::returned("P0812", &returned.prompt);
                }
                if crate::cm740_probe::frame("P0812") {
                    crate::cm740_probe::hit("P0812", "OmitActualCatchReturnedAccounting");
                    return actual;
                }
                actual
            }
            Ok(value) => crate::cm740_probe::around("P0813", || {
                self.resume_catch_after_body_cm740_driven(form, handler, Ok(value))
            }),
            Err(error) => match crate::cm740_probe::around_ok_profile_return(
                "P0814",
                "OrdinaryActualSuccessBeforeTail",
                || self.resume_catch_after_body_cm740_driven(form, handler, Err(error)),
            ) {
                crate::cm740_probe::OkAction::ReturnOk(value) => return Ok(value),
                crate::cm740_probe::OkAction::Continue(result) => result,
            },
        };
        self.record_profile_runtime_path_timing_adopted_boundary(runtime_profile_total, &result);
        result
    }

    fn resume_catch_after_body(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let runtime_profile =
            self.runtime_path_timing_start_adopted_incoming("catch.resume_after_body", &outcome);
        let profile_outcome = runtime_profile_outcome_for_expr_result(&outcome);
        let result = match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::ThrownBypassingCatch {
                value,
                tag,
                explicit_tag,
                cleanup_value,
                cleanup_tag,
                cleanup_explicit_tag,
            }) => self.resume_catch_for_bypassing_throw(
                form,
                value,
                tag,
                explicit_tag,
                cleanup_value,
                cleanup_tag,
                cleanup_explicit_tag,
            ),
            Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            }) => self.resume_catch_for_throw(form, handler, value, tag, explicit_tag),
            Err(error) => Err(error),
        };
        self.record_profile_runtime_path_timing_adopted_incoming(
            runtime_profile,
            profile_outcome,
            &result,
        );
        result
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_catch_after_body_cm740_driven(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let runtime_profile =
            self.runtime_path_timing_start_adopted_incoming("catch.resume_after_body", &outcome);
        let profile_outcome = runtime_profile_outcome_for_expr_result(&outcome);
        let result = match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::ThrownBypassingCatch {
                value,
                tag,
                explicit_tag,
                cleanup_value,
                cleanup_tag,
                cleanup_explicit_tag,
            }) => {
                let _c5_guard = crate::cm740_probe::enter("P0817");
                let c5_child = self.resume_catch_for_bypassing_throw_cm740_driven(
                    form,
                    value,
                    tag,
                    explicit_tag,
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag,
                );
                if crate::cm740_probe::ordinary("P0817") {
                    crate::cm740_probe::hit(
                        "P0817",
                        "ActualCompleteResultIncludingBypassingThrowBeforeProfileFinish",
                    );
                    return c5_child;
                }
                c5_child
            }
            Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            }) => crate::cm740_probe::around_ok(
                "P0818",
                "ActualTypedCarrierBeforeOuterTail",
                false,
                || {
                    self.resume_catch_for_throw_cm740_driven(
                        form,
                        handler,
                        value,
                        tag,
                        explicit_tag,
                    )
                },
            ),
            Err(error) => Err(error),
        };
        self.record_profile_runtime_path_timing_adopted_incoming(
            runtime_profile,
            profile_outcome,
            &result,
        );
        result
    }

    fn resume_catch_for_throw(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        let Some(form) = form else {
            return self.finish_catch_match(handler, value, tag);
        };

        let form_result = self.resolve_matcher_pattern_expr_owned(form);
        let form = crate::resume_typed_or_suspend!(
            self,
            form_result,
            ok(value) => Ok(value),
            suspend => {
                let handler = handler.clone();
                let value = value.clone();
                let tag = tag.clone();
                ResumeCatchForThrowSuspensionFrame { handler, value, tag, explicit_tag }
            },
        )?;

        self.finish_catch_for_throw_with_form(form, handler, value, tag, explicit_tag)
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_catch_for_throw_cm740_driven(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        let Some(form) = form else {
            return self.finish_catch_match(handler, value, tag);
        };

        let form_result = crate::cm740_probe::around("P0820", || {
            self.resolve_matcher_pattern_expr_owned_cm740_driven(form)
        });
        let form = crate::resume_typed_or_suspend!(
            self,
            form_result,
            ok(value) => Ok(value),
            suspend => {
                let handler = handler.clone();
                let value = value.clone();
                let tag = tag.clone();
                ResumeCatchForThrowSuspensionFrame { handler, value, tag, explicit_tag }
            },
        )?;

        self.finish_catch_for_throw_with_form_cm740_driven(form, handler, value, tag, explicit_tag)
    }

    fn finish_catch_for_throw_with_form(
        &mut self,
        form: Expr,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        if let Some(matched) = exact_catch_tag_match(&form, &tag) {
            return if matched {
                self.finish_catch_match(handler, value, tag)
            } else {
                Err(EvalError::Thrown {
                    value,
                    tag,
                    explicit_tag,
                })
            };
        }

        let match_result = self.match_expr_owned(form, tag.clone(), SymbolExprMap::default());
        crate::resume_typed_or_suspend!(
            self,
            match_result,
            ok(outcome) => {
                let session = self;
                (|| {
                    if outcome.matched {
                        session.finish_catch_match(handler, value, tag)
                    } else {
                        Err(EvalError::Thrown {
                            value,
                            tag,
                            explicit_tag,
                        })
                    }
                })()
            },
            suspend => FinishCatchForThrowWithFormSuspensionFrame { handler, value, tag, explicit_tag },
        )
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn finish_catch_for_throw_with_form_cm740_driven(
        &mut self,
        form: Expr,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        if let Some(matched) = exact_catch_tag_match(&form, &tag) {
            return if matched {
                self.finish_catch_match(handler, value, tag)
            } else {
                Err(EvalError::Thrown {
                    value,
                    tag,
                    explicit_tag,
                })
            };
        }

        let match_result = crate::cm740_probe::around("P0823", || {
            self.match_expr_owned_cm740_driven(form, tag.clone(), SymbolExprMap::default())
        });
        crate::resume_typed_or_suspend!(
            self,
            match_result,
            ok(outcome) => {
                let session = self;
                (|| {
                    if outcome.matched {
                        session.finish_catch_match(handler, value, tag)
                    } else {
                        Err(EvalError::Thrown {
                            value,
                            tag,
                            explicit_tag,
                        })
                    }
                })()
            },
            suspend => FinishCatchForThrowWithFormSuspensionFrame { handler, value, tag, explicit_tag },
        )
    }

    fn resume_catch_for_bypassing_throw(
        &mut self,
        form: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
        cleanup_value: Expr,
        cleanup_tag: Expr,
        cleanup_explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        let Some(form) = form else {
            return Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            });
        };

        let form_result = self.resolve_matcher_pattern_expr_owned(form);
        let form = crate::resume_typed_or_suspend!(
            self,
            form_result,
            ok(value) => Ok(value),
            suspend => {
                let value = value.clone();
                let tag = tag.clone();
                let cleanup_value = cleanup_value.clone();
                let cleanup_tag = cleanup_tag.clone();
                ResumeCatchForBypassingThrowSuspensionFrame {
                    value,
                    tag,
                    explicit_tag,
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag,
                }
            },
        )?;

        self.finish_catch_for_bypassing_throw_with_form(
            form,
            value,
            tag,
            explicit_tag,
            cleanup_value,
            cleanup_tag,
            cleanup_explicit_tag,
        )
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_catch_for_bypassing_throw_cm740_driven(
        &mut self,
        form: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
        cleanup_value: Expr,
        cleanup_tag: Expr,
        cleanup_explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        let Some(form) = form else {
            return Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            });
        };

        let form_result = crate::cm740_probe::around("P0825", || {
            self.resolve_matcher_pattern_expr_owned_cm740_driven(form)
        });
        let form = crate::resume_typed_or_suspend!(
            self,
            form_result,
            ok(value) => Ok(value),
            suspend => {
                let value = value.clone();
                let tag = tag.clone();
                let cleanup_value = cleanup_value.clone();
                let cleanup_tag = cleanup_tag.clone();
                ResumeCatchForBypassingThrowSuspensionFrame {
                    value,
                    tag,
                    explicit_tag,
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag,
                }
            },
        )?;

        self.finish_catch_for_bypassing_throw_with_form_cm740_driven(
            form,
            value,
            tag,
            explicit_tag,
            cleanup_value,
            cleanup_tag,
            cleanup_explicit_tag,
        )
    }

    fn finish_catch_for_bypassing_throw_with_form(
        &mut self,
        form: Expr,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
        cleanup_value: Expr,
        cleanup_tag: Expr,
        cleanup_explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        if let Some(matched) = exact_catch_tag_match(&form, &tag) {
            return if matched {
                Err(EvalError::Thrown {
                    value,
                    tag,
                    explicit_tag,
                })
            } else {
                Err(EvalError::ThrownBypassingCatch {
                    value,
                    tag,
                    explicit_tag,
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag,
                })
            };
        }

        let match_result = self.match_expr_owned(form, tag.clone(), SymbolExprMap::default());
        crate::resume_typed_or_suspend!(
            self,
            match_result,
            ok(outcome) => {
                (|| {
                    if outcome.matched {
                        Err(EvalError::Thrown {
                            value,
                            tag,
                            explicit_tag,
                        })
                    } else {
                        Err(EvalError::ThrownBypassingCatch {
                            value,
                            tag,
                            explicit_tag,
                            cleanup_value,
                            cleanup_tag,
                            cleanup_explicit_tag,
                        })
                    }
                })()
            },
            suspend => FinishCatchForBypassingThrowWithFormSuspensionFrame {
                value,
                tag,
                explicit_tag,
                cleanup_value,
                cleanup_tag,
                cleanup_explicit_tag,
            },
        )
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn finish_catch_for_bypassing_throw_with_form_cm740_driven(
        &mut self,
        form: Expr,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
        cleanup_value: Expr,
        cleanup_tag: Expr,
        cleanup_explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        if let Some(matched) = exact_catch_tag_match(&form, &tag) {
            return if matched {
                Err(EvalError::Thrown {
                    value,
                    tag,
                    explicit_tag,
                })
            } else {
                Err(EvalError::ThrownBypassingCatch {
                    value,
                    tag,
                    explicit_tag,
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag,
                })
            };
        }

        let match_result = crate::cm740_probe::around("P0827", || {
            self.match_expr_owned_cm740_driven(form, tag.clone(), SymbolExprMap::default())
        });
        crate::resume_typed_or_suspend!(
            self,
            match_result,
            ok(outcome) => {
                (|| {
                    if outcome.matched {
                        Err(EvalError::Thrown {
                            value,
                            tag,
                            explicit_tag,
                        })
                    } else {
                        Err(EvalError::ThrownBypassingCatch {
                            value,
                            tag,
                            explicit_tag,
                            cleanup_value,
                            cleanup_tag,
                            cleanup_explicit_tag,
                        })
                    }
                })()
            },
            suspend => FinishCatchForBypassingThrowWithFormSuspensionFrame {
                value,
                tag,
                explicit_tag,
                cleanup_value,
                cleanup_tag,
                cleanup_explicit_tag,
            },
        )
    }

    fn finish_catch_match(
        &mut self,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        match handler {
            Some(handler) => self.evaluate(Expr::call(handler, vec![value, tag])),
            None => Ok(value),
        }
    }
}

fn exact_catch_tag_match(form: &Expr, tag: &Expr) -> Option<bool> {
    matches!(form, Expr::Atom(_)).then(|| form == tag)
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
struct EvalCatchSuspensionFrame {
    form: std::option::Option<rum_core::Expr>,
    handler: std::option::Option<rum_core::Expr>,
}

impl TypedFrame for EvalCatchSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_3531(self, session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut form = self.form.clone();
        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        let resumed = (|| session.resume_catch_after_body(form, handler, outcome))();
        TypedAction::complete(resumed)
    }
}
impl EvalCatchSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_3531<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut form = self.form.clone();
        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        let resumed = if crate::cm740_probe::frame("P0813") {
            crate::cm740_probe::hit("P0813", "ForwardActualCatchBodyBeforeDispatch");
            outcome
        } else {
            session.resume_catch_after_body_cm740_driven(form, handler, outcome)
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeCatchForThrowSuspensionFrame {
    handler: std::option::Option<rum_core::Expr>,
    value: rum_core::Expr,
    tag: rum_core::Expr,
    explicit_tag: bool,
}

impl TypedFrame for ResumeCatchForThrowSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_3927(self, session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag;
        let resumed = match outcome {
            Ok(form) => (|| {
                session.finish_catch_for_throw_with_form(form, handler, value, tag, explicit_tag)
            })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}
impl ResumeCatchForThrowSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_3927<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::frame("P0820") {
            crate::cm740_probe::hit("P0820", "ActualAttachedFrameInputBeforeTail");
            return crate::runtime::kernel_api::TypedAction::complete(outcome);
        }

        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag;
        let resumed = match outcome {
            Ok(form) => (|| {
                session.finish_catch_for_throw_with_form_cm740_driven(
                    form,
                    handler,
                    value,
                    tag,
                    explicit_tag,
                )
            })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishCatchForThrowWithFormSuspensionFrame {
    handler: std::option::Option<rum_core::Expr>,
    value: rum_core::Expr,
    tag: rum_core::Expr,
    explicit_tag: bool,
}

impl TypedFrame for FinishCatchForThrowWithFormSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_3e6b(self, session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag;
        let resumed = match outcome {
            Ok(outcome) => (|| {
                if outcome.matched {
                    session.finish_catch_match(handler, value, tag)
                } else {
                    Err(EvalError::Thrown {
                        value,
                        tag,
                        explicit_tag,
                    })
                }
            })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}
impl FinishCatchForThrowWithFormSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_3e6b<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::frame("P0823") {
            crate::cm740_probe::hit("P0823", "OmitActualCatchTagClassification");
            return TypedAction::complete(match outcome {
                Ok(_) => Err(EvalError::Thrown {
                    value: self.value.clone(),
                    tag: self.tag.clone(),
                    explicit_tag: self.explicit_tag,
                }),
                Err(error) => Err(error),
            });
        }
        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag;
        let resumed = match outcome {
            Ok(outcome) => (|| {
                if outcome.matched {
                    session.finish_catch_match(handler, value, tag)
                } else {
                    Err(EvalError::Thrown {
                        value,
                        tag,
                        explicit_tag,
                    })
                }
            })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeCatchForBypassingThrowSuspensionFrame {
    value: rum_core::Expr,
    tag: rum_core::Expr,
    explicit_tag: bool,
    cleanup_value: rum_core::Expr,
    cleanup_tag: rum_core::Expr,
    cleanup_explicit_tag: bool,
}

impl TypedFrame for ResumeCatchForBypassingThrowSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_453e(self, session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag;
        #[allow(unused_mut)]
        let mut cleanup_value = self.cleanup_value.clone();
        #[allow(unused_mut)]
        let mut cleanup_tag = self.cleanup_tag.clone();
        #[allow(unused_mut)]
        let mut cleanup_explicit_tag = self.cleanup_explicit_tag;
        let resumed = match outcome {
            Ok(form) => (|| {
                session.finish_catch_for_bypassing_throw_with_form(
                    form,
                    value,
                    tag,
                    explicit_tag,
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag,
                )
            })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}
impl ResumeCatchForBypassingThrowSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_453e<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::frame("P0825") {
            crate::cm740_probe::hit("P0825", "ActualAttachedFrameInputBeforeTail");
            return crate::runtime::kernel_api::TypedAction::complete(outcome);
        }

        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag;
        #[allow(unused_mut)]
        let mut cleanup_value = self.cleanup_value.clone();
        #[allow(unused_mut)]
        let mut cleanup_tag = self.cleanup_tag.clone();
        #[allow(unused_mut)]
        let mut cleanup_explicit_tag = self.cleanup_explicit_tag;
        let resumed = match outcome {
            Ok(form) => (|| {
                session.finish_catch_for_bypassing_throw_with_form_cm740_driven(
                    form,
                    value,
                    tag,
                    explicit_tag,
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag,
                )
            })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishCatchForBypassingThrowWithFormSuspensionFrame {
    value: rum_core::Expr,
    tag: rum_core::Expr,
    explicit_tag: bool,
    cleanup_value: rum_core::Expr,
    cleanup_tag: rum_core::Expr,
    cleanup_explicit_tag: bool,
}

impl TypedFrame for FinishCatchForBypassingThrowWithFormSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_4c58(self, _session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag;
        #[allow(unused_mut)]
        let mut cleanup_value = self.cleanup_value.clone();
        #[allow(unused_mut)]
        let mut cleanup_tag = self.cleanup_tag.clone();
        #[allow(unused_mut)]
        let mut cleanup_explicit_tag = self.cleanup_explicit_tag;
        let resumed = match outcome {
            Ok(outcome) => (|| {
                if outcome.matched {
                    Err(EvalError::Thrown {
                        value,
                        tag,
                        explicit_tag,
                    })
                } else {
                    Err(EvalError::ThrownBypassingCatch {
                        value,
                        tag,
                        explicit_tag,
                        cleanup_value,
                        cleanup_tag,
                        cleanup_explicit_tag,
                    })
                }
            })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}
impl FinishCatchForBypassingThrowWithFormSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_4c58<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::frame("P0827") {
            crate::cm740_probe::hit("P0827", "OmitActualCatchTagClassification");
            return TypedAction::complete(match outcome {
                Ok(_) => Err(EvalError::ThrownBypassingCatch {
                    value: self.value.clone(),
                    tag: self.tag.clone(),
                    explicit_tag: self.explicit_tag,
                    cleanup_value: self.cleanup_value.clone(),
                    cleanup_tag: self.cleanup_tag.clone(),
                    cleanup_explicit_tag: self.cleanup_explicit_tag,
                }),
                Err(error) => Err(error),
            });
        }
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag;
        #[allow(unused_mut)]
        let mut cleanup_value = self.cleanup_value.clone();
        #[allow(unused_mut)]
        let mut cleanup_tag = self.cleanup_tag.clone();
        #[allow(unused_mut)]
        let mut cleanup_explicit_tag = self.cleanup_explicit_tag;
        let resumed = match outcome {
            Ok(outcome) => (|| {
                if outcome.matched {
                    Err(EvalError::Thrown {
                        value,
                        tag,
                        explicit_tag,
                    })
                } else {
                    Err(EvalError::ThrownBypassingCatch {
                        value,
                        tag,
                        explicit_tag,
                        cleanup_value,
                        cleanup_tag,
                        cleanup_explicit_tag,
                    })
                }
            })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[cold]
#[inline(never)]
fn suspend_catch(
    session: &mut Session,
    form: Option<Expr>,
    handler: Option<Expr>,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(session, request, EvalCatchSuspensionFrame { form, handler })
}

#[cold]
#[inline(never)]
#[allow(dead_code)]
fn suspend_catch_cm740_driven(
    session: &mut Session,
    form: Option<Expr>,
    handler: Option<Expr>,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    crate::cm740_probe::returned("P0813", &request.prompt);
    witness.suspend(session, request, EvalCatchSuspensionFrame { form, handler })
}
