//! Application pauses own the same finishing phase as the ordinary route.
use super::*;
use crate::runtime::kernel_api::{
    ContinueToken, FrameRootTrace, FrameRootVisitor, ResumeStop, TypedAction, TypedFrame,
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ReplaceAllApplicationFrame;

impl TypedFrame for ReplaceAllApplicationFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        // A value-phase pause installs its existing legacy frame in the shared
        // finish, before this application frame completes through ResumeStop.
        let resumed = match outcome {
            Ok(replaced) => session.finish_replace_all_application(replaced),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ReplaceAllUnheldApplicationFrame;

impl TypedFrame for ReplaceAllUnheldApplicationFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        // A value-phase pause installs its existing legacy frame in the shared
        // finish, before this application frame completes through ResumeStop.
        let resumed = match outcome {
            Ok(replaced) => session.finish_replace_all_unheld_application(replaced),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}
