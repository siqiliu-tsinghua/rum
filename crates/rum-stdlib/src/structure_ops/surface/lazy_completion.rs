//! Complete the original evaluated First call after lazy realization.
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::FrameRootVisitor;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{
    ContinuationValue, EvalError, FrameAction, FrameRootTrace, PureFrameAbandon, Session,
    SuspensionFrame,
};
use std::sync::Arc;

#[derive(FrameRootTrace, PureFrameAbandon)]
pub(super) struct FirstLazyCompletionFrame {
    pub(super) args: SharedArgs,
}

impl SuspensionFrame for FirstLazyCompletionFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(complete(&self, session, outcome))
    }
}

#[cold]
fn complete(
    frame: &FirstLazyCompletionFrame,
    session: &mut Session,
    outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    if rum_runtime::cm740_probe::driven() {
        return complete_cm740_driven(frame, session, outcome);
    }

    let encoded = outcome?;
    Ok(match Option::<Expr>::from_continuation_expr(encoded) {
        Some(value) => {
            super::resolve_first_like(session, &value, frame.args.get(1), "First", "nofirst", true)
        }
        None => Expr::call(
            rum_runtime::kernel_api::system_expr("First"),
            frame.args.to_vec(),
        ),
    })
}

#[cold]
#[inline(never)]
#[allow(dead_code)]
fn complete_cm740_driven(
    frame: &FirstLazyCompletionFrame,
    session: &mut Session,
    outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    let encoded = outcome?;
    Ok(match Option::<Expr>::from_continuation_expr(encoded) {
        Some(value) => {
            if rum_runtime::cm740_probe::frame("P2593") {
                rum_runtime::cm740_probe::hit("P2593", "ActualOptionalSuccessBeforeConcreteTail");
                return Ok(value);
            }
            super::resolve_first_like(session, &value, frame.args.get(1), "First", "nofirst", true)
        }
        None => Expr::call(
            rum_runtime::kernel_api::system_expr("First"),
            frame.args.to_vec(),
        ),
    })
}

#[cold]
#[inline(never)]
pub(super) fn capture(
    session: &mut Session,
    error: EvalError,
    args: &[Expr],
) -> Result<Option<Expr>, EvalError> {
    rum_runtime::resume_or_suspend!(
        session,
        Err::<Option<Expr>, EvalError>(error),
        ok(value) => Ok(value),
        suspend => FirstLazyCompletionFrame { args: args.to_vec().into() },
    )
}
