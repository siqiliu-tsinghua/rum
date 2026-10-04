//! Cold freeze and immutable-frame resume. The runtime keeps the queued Arc
//! Session-rooted while its temporary working copy executes.
use super::{
    PullBudget,
    realization::drive_items,
    state::{
        AwaitCursor, DriveCursor, PullContext, PullMachine, PullSource, PullStage, SavedPullLimits,
    },
};
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::FrameRootVisitor;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{
    ContinuationValue, EvalError, FrameAction, FrameRootTrace, PureFrameAbandon, Session,
    SuspensionFrame, system_expr, truthiness_symbol,
};
use std::sync::Arc;

#[derive(FrameRootTrace, PureFrameAbandon)]
pub(super) struct LazyRealizeSuspensionFrame {
    pub(super) machine: PullMachine,
    pub(super) items: SharedArgs,
    pub(super) budget: PullBudget,
    pub(super) limits: SavedPullLimits,
    pub(super) cursor: AwaitCursor,
}

#[cold]
#[inline(never)]
pub(super) fn suspend_realization(
    session: &mut Session,
    machine: PullMachine,
    items: Vec<Expr>,
    budget: PullBudget,
    mut ctx: PullContext,
    error: EvalError,
) -> EvalError {
    if let EvalError::InputRequired { .. } = &error {
        let Some(cursor) = ctx.pending.take() else {
            return invalid_cursor();
        };
        rum_runtime::resume_or_suspend!(insert session, LazyRealizeSuspensionFrame {
            machine,
            items: items.into(),
            budget,
            limits: ctx.limits,
            cursor: *cursor,
        });
    }
    error
}

impl SuspensionFrame for LazyRealizeSuspensionFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let answer = match outcome {
            Ok(answer) => answer,
            Err(error) => return FrameAction::complete(Err(error)),
        };
        FrameAction::complete(resume_realization(&self, session, answer))
    }
}

#[cold]
#[inline(never)]
fn resume_realization(
    frame: &LazyRealizeSuspensionFrame,
    session: &mut Session,
    answer: Expr,
) -> Result<Expr, EvalError> {
    let mut ctx = PullContext::restore(session, frame.limits);
    // Vec Clone iterates. No stage owns another node. Both frozen and working
    // owners remain live here; root tracing does not discharge retention costs.
    let (mut machine, mut items, cursor) = working_copy(frame);
    let cursor = apply_answer(&mut machine, cursor, answer, &ctx)?;
    match drive_items(
        &mut machine,
        session,
        &mut ctx,
        &mut items,
        frame.budget,
        cursor,
    ) {
        Ok(()) => Ok(Some(Expr::call(system_expr("List"), items)).into_continuation_expr()),
        Err(error) => Err(suspend_realization(
            session,
            machine,
            items,
            frame.budget,
            ctx,
            error,
        )),
    }
}

pub(super) fn working_copy(
    frame: &LazyRealizeSuspensionFrame,
) -> (PullMachine, Vec<Expr>, AwaitCursor) {
    (
        frame.machine.clone(),
        frame.items.to_vec(),
        frame.cursor.clone(),
    )
}

fn apply_answer(
    machine: &mut PullMachine,
    cursor: AwaitCursor,
    answer: Expr,
    ctx: &PullContext,
) -> Result<DriveCursor, EvalError> {
    // This is the resumed callback's completion checkpoint, not a resume-entry poll.
    ctx.observe_cancellation()?;
    match cursor {
        AwaitCursor::Map { stage } => {
            let Some(PullStage::Map { .. }) = machine.stages.get(stage) else {
                return Err(invalid_cursor());
            };
            Ok(DriveCursor::Up {
                boundary: stage,
                result: Some(answer),
            })
        }
        AwaitCursor::Select { stage, candidate } => {
            let Some(PullStage::Select { .. }) = machine.stages.get(stage) else {
                return Err(invalid_cursor());
            };
            let keep = truthiness_symbol(&answer) == Some(true);
            drop(answer);
            if keep {
                Ok(DriveCursor::Up {
                    boundary: stage,
                    result: Some(candidate),
                })
            } else {
                drop(candidate);
                Ok(DriveCursor::Down { at: stage + 1 })
            }
        }
        AwaitCursor::Fold { stage } => {
            let Some(PullStage::FoldList {
                accumulator,
                emitted_init: true,
                ..
            }) = machine.stages.get_mut(stage)
            else {
                return Err(invalid_cursor());
            };
            *accumulator = answer;
            Ok(DriveCursor::Up {
                boundary: stage,
                result: Some(accumulator.clone()),
            })
        }
        AwaitCursor::Nest { yield_value } => {
            let PullSource::NestList { current, .. } = &mut machine.source else {
                return Err(invalid_cursor());
            };
            *current = answer;
            Ok(DriveCursor::Up {
                boundary: machine.stages.len(),
                result: Some(yield_value),
            })
        }
    }
}

fn invalid_cursor() -> EvalError {
    EvalError::ExceptionThrown {
        exception: Expr::string("invalid internal lazy realization cursor"),
    }
}
