use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalSingleSymbolBlockSuspensionFrame {
    pub(super) state: SuspendedSingleBlockScope,
}

impl crate::runtime::suspension::SuspensionFrame for EvalSingleSymbolBlockSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        crate::runtime::suspension::FrameAction::finalize(PrepareSingleBlockInitializerFinalizer {
            state: self.state.clone(),
            outcome,
        })
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalSingleSymbolBlockSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_single_block_scope(
            self.state.symbol,
            self.state.saved_state.as_ref().clone(),
            self.state.dynamic_depth,
        );
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

struct PrepareSingleBlockInitializerFinalizer {
    state: SuspendedSingleBlockScope,
    outcome: Result<Expr, EvalError>,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareSingleBlockInitializerFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(self.state.into_continuation_expr(self.outcome))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueSingleBlockInitializerFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueSingleBlockInitializerFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let (symbol, body, saved_state, dynamic_depth, initializer_outcome) =
                SuspendedSingleBlockScope::from_continuation_expr(payload);
            session.resume_single_symbol_block_after_initializer(
                symbol,
                body,
                saved_state,
                dynamic_depth,
                initializer_outcome,
            )
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeBlockInitializersSuspensionFrame {
    pub(super) state: SuspendedBlockInitializers,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeBlockInitializersSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        crate::runtime::suspension::FrameAction::finalize(PrepareBlockInitializerFinalizer {
            state: self.state.clone(),
            outcome,
        })
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeBlockInitializersSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_block_scope(
            self.state.saved_state.as_ref().clone(),
            self.state.dynamic_depth,
        );
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

struct PrepareBlockInitializerFinalizer {
    state: SuspendedBlockInitializers,
    outcome: Result<Expr, EvalError>,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareBlockInitializerFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(self.state.into_continuation_expr(self.outcome))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueBlockInitializerFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueBlockInitializerFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let (items, body, saved_state, dynamic_depth, index, initializer_outcome) =
                SuspendedBlockInitializers::from_continuation_expr(payload);
            session.resume_block_after_initializer(
                items,
                body,
                saved_state,
                dynamic_depth,
                index,
                initializer_outcome,
            )
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeSingleSymbolBlockBodySuspensionFrame {
    pub(super) state: SuspendedSingleBlockScope,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeSingleSymbolBlockBodySuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        crate::runtime::suspension::FrameAction::finalize(PrepareSingleBlockBodyFinalizer {
            state: self.state.clone(),
            outcome,
        })
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeSingleSymbolBlockBodySuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_single_block_scope(
            self.state.symbol,
            self.state.saved_state.as_ref().clone(),
            self.state.dynamic_depth,
        );
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

struct PrepareSingleBlockBodyFinalizer {
    state: SuspendedSingleBlockScope,
    outcome: Result<Expr, EvalError>,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareSingleBlockBodyFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(self.state.into_continuation_expr(self.outcome))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueSingleBlockBodyFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueSingleBlockBodyFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let (symbol, _body, saved_state, dynamic_depth, body_outcome) =
                SuspendedSingleBlockScope::from_continuation_expr(payload);
            session.finish_single_symbol_block_scope(
                symbol,
                saved_state,
                dynamic_depth,
                body_outcome,
            )
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeBlockBodySuspensionFrame {
    pub(super) state: SuspendedBlockScope,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeBlockBodySuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        crate::runtime::suspension::FrameAction::finalize(PrepareBlockBodyFinalizer {
            state: self.state.clone(),
            outcome,
        })
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeBlockBodySuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_block_scope(
            self.state.saved_state.as_ref().clone(),
            self.state.dynamic_depth,
        );
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

struct PrepareBlockBodyFinalizer {
    state: SuspendedBlockScope,
    outcome: Result<Expr, EvalError>,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareBlockBodyFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(self.state.into_continuation_expr(self.outcome))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueBlockBodyFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueBlockBodyFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let (saved_state, dynamic_depth, body_outcome) =
                SuspendedBlockScope::from_continuation_expr(payload);
            session.finish_block_scope(saved_state, dynamic_depth, body_outcome)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
