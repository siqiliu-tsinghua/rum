use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalCheckAbortSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalCheckAbortSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
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
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCheckAbortWithPropagationSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) propagate:
        crate::runtime::kernel_data::control_data::abort_cleanup::PropagateAbortsMode,
    pub(super) pending_checkpoint: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeCheckAbortWithPropagationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut propagate = self.propagate.clone();
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint.clone();
        let resumed = (|| {
            session.resume_check_abort_after_primary(&args, propagate, pending_checkpoint, outcome)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame for EvalAbortProtectSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint.clone();
        let resumed = (|| session.resume_abort_protect_after_body(pending_checkpoint, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame for ResumeCheckAbortAfterPrimarySuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut propagate = self.propagate.clone();
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint.clone();
        let resumed = (|| {
            session.resume_check_abort_after_fallback(propagate, true, pending_checkpoint, outcome)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame for ResumeCheckAbortAfterPrimarySuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut propagate = self.propagate.clone();
        #[allow(unused_mut)]
        let mut protected = self.protected.clone();
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint.clone();
        let resumed = (|| {
            session.resume_check_abort_after_fallback(
                propagate,
                protected,
                pending_checkpoint,
                outcome,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
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
