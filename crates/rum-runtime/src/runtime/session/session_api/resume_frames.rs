use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvaluateWithRandomStateRestoredSuspensionFrame {
    pub(super) saved: crate::runtime::random::PrngState,
}

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateWithRandomStateRestoredSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut saved = self.saved.clone();
        let resumed = (|| {
            session.prng_state = saved;
            outcome
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvaluateWithRandomStateRestoredSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_prng_state(self.saved);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}
