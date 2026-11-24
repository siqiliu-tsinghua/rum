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
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_337(self, session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let next = outcome?;
            session.resume_fixed_point_from(state, next)
        })();
        crate::runtime::suspension::TypedAction::complete(resumed)
    }
}
impl EvaluateProfiledFixedPointSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_337<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::suspension::ContinueToken<'resume, Self>,
    ) -> crate::runtime::suspension::TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let next = outcome?;
            crate::cm740_probe::around("P1470", || {
                session.resume_fixed_point_from_cm740_driven(state, next, false)
            })
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
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_6d8(self, session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let next = outcome?;
            session.resume_fixed_point_from(state, next)
        })();
        crate::runtime::suspension::TypedAction::complete(resumed)
    }
}
impl ResumeFixedPointFromSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_6d8<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::suspension::ContinueToken<'resume, Self>,
    ) -> crate::runtime::suspension::TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let next = outcome?;
            {
                let _cm740_producer = crate::cm740_probe::enter("P1471");
                session.resume_fixed_point_from_cm740_driven(
                    state,
                    next,
                    crate::cm740_probe::frame("P1476"),
                )
            }
        })();
        crate::runtime::suspension::TypedAction::complete(resumed)
    }
}
