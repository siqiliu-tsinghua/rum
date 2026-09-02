use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalCatchExceptionsSuspensionFrame {
    pub(super) handlers: std::option::Option<
        std::sync::Arc<[crate::runtime::kernel_data::control_data::exceptions::ExceptionHandler]>,
    >,
}

impl crate::runtime::suspension::SuspensionFrame for EvalCatchExceptionsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut handlers = self.handlers.clone();
        let resumed =
            (|| session.resume_suspended_catch_exceptions_after_body(handlers, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DispatchExceptionHandlersOwnedSuspensionFrame1 {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for DispatchExceptionHandlersOwnedSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_exception_guard(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DispatchExceptionHandlersOwnedSuspensionFrame2 {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for DispatchExceptionHandlersOwnedSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_exception_match(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DispatchSuspendedExceptionHandlersSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for DispatchSuspendedExceptionHandlersSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_exception_match(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedExceptionMatchOutcomeSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedExceptionMatchOutcomeSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_exception_guard(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateExceptionHandlerRhsSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for EvaluateExceptionHandlerRhsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(value) => (|| Ok(value))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
