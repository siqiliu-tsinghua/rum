use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeWhileSequenceFromIndexSuspensionFrame {
    pub(super) sequence: crate::functional_ops::sequence::window::SuspendedSequenceLikeExpr,
    pub(super) criterion: rum_core::Expr,
    pub(super) index: usize,
    pub(super) mode: crate::functional_ops::sequence::window::WhileSequenceMode,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeWhileSequenceFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut sequence = self.sequence.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut mode = self.mode.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_while_sequence_with_value(
                    session, sequence, criterion, index, mode, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedWhileSequenceFromIndexSuspensionFrame {
    pub(super) sequence: crate::functional_ops::sequence::window::SuspendedSequenceLikeExpr,
    pub(super) criterion: rum_core::Expr,
    pub(super) index: usize,
    pub(super) mode: crate::functional_ops::sequence::window::WhileSequenceMode,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedWhileSequenceFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut sequence = self.sequence.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut mode = self.mode.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_while_sequence_with_value(
                    session, sequence, criterion, index, mode, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeAccumulateFromIndexSuspensionFrame {
    pub(super) sequence: crate::functional_ops::sequence::window::SuspendedSequenceLikeExpr,
    pub(super) index: usize,
    pub(super) accumulated:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeAccumulateFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut sequence = self.sequence.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut accumulated = self.accumulated.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_accumulate_with_value(session, sequence, index, accumulated, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedAccumulateFromIndexSuspensionFrame {
    pub(super) sequence: crate::functional_ops::sequence::window::SuspendedSequenceLikeExpr,
    pub(super) index: usize,
    pub(super) accumulated:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAccumulateFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut sequence = self.sequence.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut accumulated = self.accumulated.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_accumulate_with_value(session, sequence, index, accumulated, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeBlockMapFromIndexSuspensionFrame {
    pub(super) state: crate::functional_ops::sequence::window::SuspendedBlockMapEvalState,
    pub(super) index: usize,
    pub(super) results:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeBlockMapFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut results = self.results.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_block_map_with_value(session, state, index, results, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedBlockMapFromIndexSuspensionFrame {
    pub(super) state: crate::functional_ops::sequence::window::SuspendedBlockMapEvalState,
    pub(super) index: usize,
    pub(super) results:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedBlockMapFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut results = self.results.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_block_map_with_value(session, state, index, results, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
