use super::*;
use crate::runtime::model::OutputMark;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalStructuredTraceCaptureExprSuspensionFrame {
    pub(super) suspended_state: crate::runtime::debug_tools::trace::TraceEvalState,
    pub(super) trace_checkpoint: OutputMark,
    pub(super) prior_enabled: bool,
}

impl crate::runtime::suspension::SuspensionFrame for EvalStructuredTraceCaptureExprSuspensionFrame {
    fn output_boundary(&self) -> crate::runtime::suspension::OutputBoundary {
        use crate::runtime::suspension::OutputBoundary;
        OutputBoundary::trace(self.trace_checkpoint)
    }

    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended_state = self.suspended_state.clone();
        #[allow(unused_mut)]
        let mut trace_checkpoint = self.trace_checkpoint.clone();
        #[allow(unused_mut)]
        let mut prior_enabled = self.prior_enabled.clone();
        let resumed = (|| {
            session.finish_structured_trace_capture_expr(
                suspended_state,
                trace_checkpoint,
                prior_enabled,
                outcome,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalStructuredTraceCaptureExprSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_trace_capture(self.prior_enabled);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct TraceFilterSuspensionFrame {
    pub(super) state: filter::TraceFilterState,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl crate::runtime::suspension::FrameAbandon for TraceFilterSuspensionFrame {
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

impl crate::runtime::suspension::SuspensionFrame for TraceFilterSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let _class_row = crate::cm740_probe::class_lease::begin_row(&self);
        crate::runtime::suspension::FrameAction::complete(match outcome {
            Ok(value) => filter::resume_trace_filter(session, &self.state, value),
            Err(error) => Err(error),
        })
    }
}
