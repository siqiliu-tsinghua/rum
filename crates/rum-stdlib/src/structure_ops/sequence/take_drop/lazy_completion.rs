//! Complete the original evaluated Take call after lazy realization.
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::FrameRootVisitor;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{
    ContinuationValue, EvalError, FrameAction, FrameRootTrace, PureFrameAbandon, Session,
    SuspensionFrame,
};
use std::sync::Arc;

#[derive(FrameRootTrace, PureFrameAbandon)]
pub(super) struct TakeLazyCompletionFrame {
    pub(super) head: Expr,
    pub(super) args: SharedArgs,
}

impl SuspensionFrame for TakeLazyCompletionFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(complete(&self, outcome))
    }
}

#[cold]
fn complete(
    frame: &TakeLazyCompletionFrame,
    outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    let encoded = outcome?;
    Ok(match Option::<Expr>::from_continuation_expr(encoded) {
        Some(value) => value,
        None => Expr::call(frame.head.clone(), frame.args.to_vec()),
    })
}

#[cold]
#[inline(never)]
pub(super) fn capture(
    session: &mut Session,
    error: EvalError,
    head: &Expr,
    args: &[Expr],
) -> Result<Option<Expr>, EvalError> {
    rum_runtime::resume_or_suspend!(
        session,
        Err::<Option<Expr>, EvalError>(error),
        ok(value) => Ok(value),
        suspend => TakeLazyCompletionFrame { head: head.clone(), args: args.to_vec().into() },
    )
}
