use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateProfiledFixedPointSuspensionFrame {
    pub(super) state: crate::runtime::resolution::SuspendedFixedPointState,
}

impl EvaluateProfiledFixedPointSuspensionFrame {
    #[cold]
    #[inline(never)]
    pub(super) fn freeze(current: Expr, evaluation_passes: usize, seen: Vec<Expr>) -> Self {
        Self {
            state: SuspendedFixedPointState {
                current,
                evaluation_passes,
                seen: FixedPointHistory::freeze(seen),
            },
        }
    }
}

impl crate::runtime::suspension::TypedFrame for EvaluateProfiledFixedPointSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::suspension::ContinueToken<'resume, Self>,
    ) -> crate::runtime::suspension::TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let next = outcome?;
            session.resume_fixed_point_from(state, next)
        })();
        crate::runtime::suspension::TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeFixedPointFromSuspensionFrame {
    pub(super) state: crate::runtime::resolution::SuspendedFixedPointState,
}

impl crate::runtime::suspension::TypedFrame for ResumeFixedPointFromSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::suspension::ContinueToken<'resume, Self>,
    ) -> crate::runtime::suspension::TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let next = outcome?;
            session.resume_fixed_point_from(state, next)
        })();
        crate::runtime::suspension::TypedAction::complete(resumed)
    }
}
