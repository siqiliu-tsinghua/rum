use super::*;
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
            record_profile_runtime_path_timing(
                runtime_profile_total,
                runtime_profile_outcome_for_expr_result(&result),
            );
            return result;
        }

        let form = args.get(1).cloned();
        let handler = args.get(2).cloned();
        let runtime_profile_body = runtime_path_timing_start("catch.body.evaluate");
        let body_result = self.evaluate(args[0].clone());
        record_profile_runtime_path_timing(
            runtime_profile_body,
            runtime_profile_outcome_for_expr_result(&body_result),
        );
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => {
        let form = form.clone();
                    let handler = handler.clone();
        EvalCatchSuspensionFrame { form, handler }
        },
        );

        let result = match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_catch_after_body(form, handler, outcome),
        };
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    fn resume_catch_after_body(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let runtime_profile = runtime_path_timing_start("catch.resume_after_body");
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
        record_profile_runtime_path_timing(runtime_profile, profile_outcome);
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
        let form = crate::resume_or_suspend!(
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
        crate::resume_or_suspend!(
            self,
            match_result,
            ok(outcome) => { let session = self; (|| {
                    if outcome.matched {
                        session.finish_catch_match(handler, value, tag)
                    } else {
                        Err(EvalError::Thrown {
                            value,
                            tag,
                            explicit_tag,
                        })
                    }
                })() },
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
        let form = crate::resume_or_suspend!(
            self,
            form_result,
            ok(value) => Ok(value),
            suspend => {
        let value = value.clone();
                    let tag = tag.clone();
                    let cleanup_value = cleanup_value.clone();
                    let cleanup_tag = cleanup_tag.clone();
        ResumeCatchForBypassingThrowSuspensionFrame { value, tag, explicit_tag, cleanup_value, cleanup_tag, cleanup_explicit_tag }
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
        crate::resume_or_suspend!(
            self,
            match_result,
            ok(outcome) => { (|| {
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
                })() },
            suspend => FinishCatchForBypassingThrowWithFormSuspensionFrame { value, tag, explicit_tag, cleanup_value, cleanup_tag, cleanup_explicit_tag },
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

impl crate::runtime::suspension::SuspensionFrame for EvalCatchSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut form = self.form.clone();
        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        let resumed = (|| session.resume_catch_after_body(form, handler, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeCatchForThrowSuspensionFrame {
    handler: std::option::Option<rum_core::Expr>,
    value: rum_core::Expr,
    tag: rum_core::Expr,
    explicit_tag: bool,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeCatchForThrowSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag.clone();
        let resumed = match outcome {
            Ok(form) => (|| {
                session.finish_catch_for_throw_with_form(form, handler, value, tag, explicit_tag)
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishCatchForThrowWithFormSuspensionFrame {
    handler: std::option::Option<rum_core::Expr>,
    value: rum_core::Expr,
    tag: rum_core::Expr,
    explicit_tag: bool,
}

impl crate::runtime::suspension::SuspensionFrame for FinishCatchForThrowWithFormSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
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
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame for ResumeCatchForBypassingThrowSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag.clone();
        #[allow(unused_mut)]
        let mut cleanup_value = self.cleanup_value.clone();
        #[allow(unused_mut)]
        let mut cleanup_tag = self.cleanup_tag.clone();
        #[allow(unused_mut)]
        let mut cleanup_explicit_tag = self.cleanup_explicit_tag.clone();
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
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame
    for FinishCatchForBypassingThrowWithFormSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut explicit_tag = self.explicit_tag.clone();
        #[allow(unused_mut)]
        let mut cleanup_value = self.cleanup_value.clone();
        #[allow(unused_mut)]
        let mut cleanup_tag = self.cleanup_tag.clone();
        #[allow(unused_mut)]
        let mut cleanup_explicit_tag = self.cleanup_explicit_tag.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
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
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
