//! Cold capture and resume of the eager bounds, before building the held plan.
use super::finish_lazy_table;
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::{
    AbandonmentContext, EvalError, FrameAbandon, FrameAbandonAction, FrameAction, FrameRootTrace,
    FrameRootVisitor, PureFrameAbandon, Session, SuspensionFrame,
};
use std::sync::Arc;

#[derive(Clone, Copy, FrameRootTrace)]
pub(super) enum BoundSlot {
    Lower,
    Upper,
    Step,
}

#[derive(FrameRootTrace, PureFrameAbandon)]
struct LazyTableBoundsFrame {
    args: SharedArgs,
    iterator: Expr,
    // Completed values precede the pending slot; normalized expressions follow.
    bounds: [Expr; 3],
    pending_slot: BoundSlot,
}

#[cold]
#[inline(never)]
pub(super) fn suspend_bounds(
    session: &mut Session,
    args: &SharedArgs,
    iterator: &Expr,
    bounds: [Expr; 3],
    pending_slot: BoundSlot,
    error: EvalError,
) -> EvalError {
    if let EvalError::InputRequired { .. } = &error {
        rum_runtime::resume_or_suspend!(insert session, LazyTableBoundsFrame {
            args: args.clone(),
            iterator: iterator.clone(),
            bounds,
            pending_slot,
        });
    }
    error
}

impl SuspensionFrame for LazyTableBoundsFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let answer = match outcome {
            Ok(answer) => answer,
            Err(error) => return FrameAction::complete(Err(error)),
        };
        FrameAction::complete(resume_bounds(&self, session, answer))
    }
}

#[cold]
#[inline(never)]
fn resume_bounds(
    frame: &LazyTableBoundsFrame,
    session: &mut Session,
    answer: Expr,
) -> Result<Expr, EvalError> {
    // The immutable frame stays Session-rooted while this cold copy executes.
    let [mut imin, mut imax, mut step] = frame.bounds.clone();
    match frame.pending_slot {
        BoundSlot::Lower => imin = answer,
        BoundSlot::Upper => imax = answer,
        BoundSlot::Step => step = answer,
    }
    if let BoundSlot::Lower = frame.pending_slot {
        imax = match session.evaluate(imax.clone()) {
            Ok(value) => value,
            Err(error) => {
                return Err(suspend_bounds(
                    session,
                    &frame.args,
                    &frame.iterator,
                    [imin, imax, step],
                    BoundSlot::Upper,
                    error,
                ));
            }
        };
    }
    if let BoundSlot::Lower | BoundSlot::Upper = frame.pending_slot {
        step = match session.evaluate(step.clone()) {
            Ok(value) => value,
            Err(error) => {
                return Err(suspend_bounds(
                    session,
                    &frame.args,
                    &frame.iterator,
                    [imin, imax, step],
                    BoundSlot::Step,
                    error,
                ));
            }
        };
    }
    finish_lazy_table(
        session,
        &frame.args,
        &frame.iterator,
        &frame.args[0],
        imin,
        imax,
        step,
    )
}
