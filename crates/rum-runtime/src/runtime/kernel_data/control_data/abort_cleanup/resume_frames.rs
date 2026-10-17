use super::*;
use crate::runtime::kernel_api::{ContinueToken, SlotWitness, TypedAction, TypedFrame};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalCheckAbortSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for EvalCheckAbortSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                session.resume_check_abort_with_propagation(
                    &args,
                    check_abort_propagation_from_value(value),
                )
            })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCheckAbortWithPropagationSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) propagate:
        crate::runtime::kernel_data::control_data::abort_cleanup::PropagateAbortsMode,
    pub(super) pending_checkpoint: usize,
}

impl TypedFrame for ResumeCheckAbortWithPropagationSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut propagate = self.propagate;
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint;
        let resumed = (|| {
            session.resume_check_abort_after_primary(&args, propagate, pending_checkpoint, outcome)
        })();
        TypedAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeCheckAbortWithPropagationSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_abort_checkpoint(self.pending_checkpoint);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalAbortProtectSuspensionFrame {
    pub(super) pending_checkpoint: usize,
}

impl TypedFrame for EvalAbortProtectSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint;
        let resumed = (|| session.resume_abort_protect_after_body(pending_checkpoint, outcome))();
        TypedAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalAbortProtectSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_abort_protect_scope(self.pending_checkpoint);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCheckAbortAfterPrimarySuspensionFrame1 {
    pub(super) propagate:
        crate::runtime::kernel_data::control_data::abort_cleanup::PropagateAbortsMode,
    pub(super) pending_checkpoint: usize,
}

impl TypedFrame for ResumeCheckAbortAfterPrimarySuspensionFrame1 {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut propagate = self.propagate;
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint;
        let resumed = (|| {
            session.resume_check_abort_after_fallback(propagate, true, pending_checkpoint, outcome)
        })();
        TypedAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeCheckAbortAfterPrimarySuspensionFrame1 {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_abort_checkpoint(self.pending_checkpoint);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCheckAbortAfterPrimarySuspensionFrame2 {
    pub(super) propagate:
        crate::runtime::kernel_data::control_data::abort_cleanup::PropagateAbortsMode,
    pub(super) protected: bool,
    pub(super) pending_checkpoint: usize,
}

impl TypedFrame for ResumeCheckAbortAfterPrimarySuspensionFrame2 {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut propagate = self.propagate;
        #[allow(unused_mut)]
        let mut protected = self.protected;
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint;
        let resumed = (|| {
            session.resume_check_abort_after_fallback(
                propagate,
                protected,
                pending_checkpoint,
                outcome,
            )
        })();
        TypedAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeCheckAbortAfterPrimarySuspensionFrame2 {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_abort_checkpoint(self.pending_checkpoint);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend_check_abort_with_propagation(
    session: &mut Session,
    args: &SharedArgs,
    propagate: PropagateAbortsMode,
    pending_checkpoint: usize,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        ResumeCheckAbortWithPropagationSuspensionFrame {
            args: args.clone(),
            propagate,
            pending_checkpoint,
        },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_abort_protect(
    session: &mut Session,
    pending_checkpoint: usize,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        EvalAbortProtectSuspensionFrame { pending_checkpoint },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_check_abort_after_primary_pending(
    session: &mut Session,
    propagate: PropagateAbortsMode,
    pending_checkpoint: usize,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        ResumeCheckAbortAfterPrimarySuspensionFrame1 {
            propagate,
            pending_checkpoint,
        },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_check_abort_after_primary_aborted(
    session: &mut Session,
    propagate: PropagateAbortsMode,
    protected: bool,
    pending_checkpoint: usize,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        ResumeCheckAbortAfterPrimarySuspensionFrame2 {
            propagate,
            protected,
            pending_checkpoint,
        },
    )
}
