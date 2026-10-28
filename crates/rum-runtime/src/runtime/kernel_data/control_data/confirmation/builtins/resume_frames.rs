use super::*;
use crate::runtime::model::OutputMark;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalEncloseSuspensionFrame {
    pub(super) handler: std::option::Option<rum_core::Expr>,
    pub(super) form: std::option::Option<rum_core::Expr>,
    pub(super) lexical_tag: std::option::Option<rum_core::Expr>,
}

impl crate::runtime::suspension::SuspensionFrame for EvalEncloseSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut handler = self.handler.clone();
        #[allow(unused_mut)]
        let mut form = self.form.clone();
        #[allow(unused_mut)]
        let mut lexical_tag = self.lexical_tag.clone();
        let resumed = (|| session.resume_enclose_after_body(handler, form, lexical_tag, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalConfirmSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalConfirmSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(value) => (|| session.resume_confirm_after_value(&args, value))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalConfirmBySuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalConfirmBySuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(value) => (|| session.resume_confirm_by_after_value(&args, value))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalConfirmMatchSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalConfirmMatchSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(value) => (|| session.resume_confirm_match_after_value(&args, value))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalConfirmQuietSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) selectors:
        crate::runtime::kernel_data::control_data::confirmation::SuspendedConfirmQuietSelectors,
    pub(super) message_checkpoint: OutputMark,
}

impl crate::runtime::suspension::SuspensionFrame for EvalConfirmQuietSuspensionFrame {
    fn output_boundary(&self) -> crate::runtime::suspension::OutputBoundary {
        use crate::runtime::suspension::OutputBoundary;
        OutputBoundary::messages(self.message_checkpoint)
    }

    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut selectors = self.selectors.clone();
        #[allow(unused_mut)]
        let mut message_checkpoint = self.message_checkpoint.clone();
        let resumed = (|| {
            session.resume_suspended_confirm_quiet_after_body(
                &args,
                &selectors,
                message_checkpoint,
                outcome,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalConfirmAssertSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalConfirmAssertSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(test) => (|| session.resume_confirm_assert_after_test(&args, test))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeEncloseForFailureSuspensionFrame {
    pub(super) handler: std::option::Option<rum_core::Expr>,
    pub(super) value: rum_core::Expr,
    pub(super) tag: rum_core::Expr,
    pub(super) explicit_tag: bool,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeEncloseForFailureSuspensionFrame {
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
                session.finish_enclose_handler(handler, value)
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
pub(super) struct ResumeConfirmAfterValueSuspensionFrame {
    pub(super) raw_expr: rum_core::Expr,
    pub(super) value: rum_core::Expr,
    pub(super) tag: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeConfirmAfterValueSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut raw_expr = self.raw_expr.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let info = <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            session.finish_confirm_failure(raw_expr, value, info, tag)
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishConfirmFailureSuspensionFrame {
    pub(super) raw_expr: rum_core::Expr,
    pub(super) info: std::option::Option<rum_core::Expr>,
    pub(super) tag: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for FinishConfirmFailureSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut raw_expr = self.raw_expr.clone();
        #[allow(unused_mut)]
        let mut info = self.info.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let message = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                Err(EvalError::Thrown {
                    value: build_confirm_failure_from_failure(raw_expr, message, info),
                    tag,
                    explicit_tag: true,
                })
            })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeConfirmByAfterValueSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) value: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeConfirmByAfterValueSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        let resumed = match outcome {
            Ok(test) => (|| session.resume_confirm_by_after_test(&args, value, test))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeConfirmByAfterTestSuspensionFrame {
    pub(super) raw_expr: rum_core::Expr,
    pub(super) raw_function: rum_core::Expr,
    pub(super) tag: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeConfirmByAfterTestSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut raw_expr = self.raw_expr.clone();
        #[allow(unused_mut)]
        let mut raw_function = self.raw_function.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let info = <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            session.finish_confirm_by_failure(raw_expr, raw_function, info, tag)
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeConfirmMatchAfterValueSuspensionFrame1 {
    pub(super) value: rum_core::Expr,
    pub(super) info_expr: std::option::Option<rum_core::Expr>,
    pub(super) raw_expr: rum_core::Expr,
    pub(super) raw_pattern: rum_core::Expr,
    pub(super) tag: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeConfirmMatchAfterValueSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut info_expr = self.info_expr.clone();
        #[allow(unused_mut)]
        let mut raw_expr = self.raw_expr.clone();
        #[allow(unused_mut)]
        let mut raw_pattern = self.raw_pattern.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            if outcome.matched {
                Ok(value)
            } else {
                let info_result = evaluate_optional_confirmation_info(session, info_expr);
                crate::resume_or_suspend!(
    session,
    info_result,
    ok(info) => { (|| {
                    session.finish_confirm_match_failure(raw_expr, raw_pattern, info, tag)
                })() },
    suspend => ResumeConfirmMatchAfterValueSuspensionFrame2 { raw_expr, raw_pattern, tag },
)
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
pub(super) struct ResumeConfirmMatchAfterValueSuspensionFrame2 {
    pub(super) raw_expr: rum_core::Expr,
    pub(super) raw_pattern: rum_core::Expr,
    pub(super) tag: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeConfirmMatchAfterValueSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut raw_expr = self.raw_expr.clone();
        #[allow(unused_mut)]
        let mut raw_pattern = self.raw_pattern.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let info = <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                    session.finish_confirm_match_failure(raw_expr, raw_pattern, info, tag)
                })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeConfirmQuietAfterBodySuspensionFrame {
    pub(super) raw_expr: rum_core::Expr,
    pub(super) tag: rum_core::Expr,
    pub(super) message: crate::runtime::RuntimeMessage,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeConfirmQuietAfterBodySuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut raw_expr = self.raw_expr.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        #[allow(unused_mut)]
        let mut message = self.message.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let info = <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            session.finish_confirm_quiet_failure(raw_expr, info, tag, message)
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeConfirmAssertAfterTestSuspensionFrame {
    pub(super) raw_test: rum_core::Expr,
    pub(super) tag: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeConfirmAssertAfterTestSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut raw_test = self.raw_test.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let info = <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            session.finish_confirm_assert_failure(raw_test, info, tag)
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
