use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateProfiledFixedPointSuspensionFrame {
    pub(super) state: crate::runtime::resolution::SuspendedFixedPointState,
}

impl crate::runtime::suspension::SuspensionFrame for EvaluateProfiledFixedPointSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let next = outcome?;
            session.resume_fixed_point_from(state, next)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeFixedPointFromSuspensionFrame {
    pub(super) state: crate::runtime::resolution::SuspendedFixedPointState,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeFixedPointFromSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let next = outcome?;
            session.resume_fixed_point_from(state, next)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
