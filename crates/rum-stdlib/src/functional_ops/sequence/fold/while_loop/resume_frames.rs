use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeFoldWhileFromIndexSuspensionFrame1 {
    pub(super) state:
        crate::functional_ops::sequence::fold::while_loop::SuspendedFoldWhileEvalState,
    pub(super) index: usize,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeFoldWhileFromIndexSuspensionFrame1 {
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
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_fold_while_with_step_value(session, state, index, history, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeFoldWhileFromIndexSuspensionFrame2 {
    pub(super) state:
        crate::functional_ops::sequence::fold::while_loop::SuspendedFoldWhileEvalState,
    pub(super) next_index: usize,
    pub(super) value: rum_core::Expr,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeFoldWhileFromIndexSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut next_index = self.next_index.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(test_value) => (|| {
                resume_suspended_fold_while_with_test_value(
                    session, state, next_index, value, history, test_value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedFoldWhileFromIndexSuspensionFrame1 {
    pub(super) state:
        crate::functional_ops::sequence::fold::while_loop::SuspendedFoldWhileEvalState,
    pub(super) index: usize,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedFoldWhileFromIndexSuspensionFrame1
{
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
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_fold_while_with_step_value(session, state, index, history, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedFoldWhileFromIndexSuspensionFrame2 {
    pub(super) state:
        crate::functional_ops::sequence::fold::while_loop::SuspendedFoldWhileEvalState,
    pub(super) next_index: usize,
    pub(super) value: rum_core::Expr,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedFoldWhileFromIndexSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut next_index = self.next_index.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(test_value) => (|| {
                resume_suspended_fold_while_with_test_value(
                    session, state, next_index, value, history, test_value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedFoldWhileWithStepValueSuspensionFrame {
    pub(super) state:
        crate::functional_ops::sequence::fold::while_loop::SuspendedFoldWhileEvalState,
    pub(super) next_index: usize,
    pub(super) value: rum_core::Expr,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedFoldWhileWithStepValueSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut next_index = self.next_index.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        #[allow(unused_mut)]
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(test_value) => (|| {
                resume_suspended_fold_while_with_test_value(
                    session, state, next_index, value, history, test_value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeFoldWhileTailFromIndexSuspensionFrame {
    pub(super) state:
        crate::functional_ops::sequence::fold::while_loop::SuspendedFoldWhileEvalState,
    pub(super) index: usize,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
    pub(super) remaining: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeFoldWhileTailFromIndexSuspensionFrame {
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
        let mut history = self.history.clone();
        #[allow(unused_mut)]
        let mut remaining = self.remaining.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_fold_while_tail_with_value(
                    session, state, index, history, remaining, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedFoldWhileTailFromIndexSuspensionFrame {
    pub(super) state:
        crate::functional_ops::sequence::fold::while_loop::SuspendedFoldWhileEvalState,
    pub(super) index: usize,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
    pub(super) remaining: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedFoldWhileTailFromIndexSuspensionFrame
{
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
        let mut history = self.history.clone();
        #[allow(unused_mut)]
        let mut remaining = self.remaining.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_fold_while_tail_with_value(
                    session, state, index, history, remaining, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
