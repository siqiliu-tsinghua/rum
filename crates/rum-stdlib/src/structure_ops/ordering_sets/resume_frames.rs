use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSetSameTestSuspensionFrame1 {
    pub(super) state: crate::structure_ops::ordering_sets::suspended::SuspendedSetSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSetSameTestSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_set_same_test_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSetSameTestSuspensionFrame2 {
    pub(super) state: crate::structure_ops::ordering_sets::suspended::SuspendedSetSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSetSameTestSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_set_same_test_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSetSameTestSuspensionFrame3 {
    pub(super) state: crate::structure_ops::ordering_sets::suspended::SuspendedSetSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSetSameTestSuspensionFrame3 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_set_same_test_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSortByFromIndexSuspensionFrame {
    pub(super) expr_head: rum_core::Expr,
    pub(super) function: rum_core::Expr,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) decorated: crate::structure_ops::suspended_state::PersistentStructureSequence<
        crate::structure_ops::ordering_sets::SortByEntry,
    >,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSortByFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut expr_head = self.expr_head.clone();
        #[allow(unused_mut)]
        let mut function = self.function.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut decorated = self.decorated.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_sort_by_with_key(
                    session, expr_head, function, items, index, decorated, item, key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSortByOrderingFromIndexSuspensionFrame {
    pub(super) expr_head: rum_core::Expr,
    pub(super) function: rum_core::Expr,
    pub(super) ordering: rum_core::Expr,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) decorated: crate::structure_ops::suspended_state::PersistentStructureSequence<
        crate::structure_ops::ordering_sets::SortByEntry,
    >,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSortByOrderingFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut expr_head = self.expr_head.clone();
        #[allow(unused_mut)]
        let mut function = self.function.clone();
        #[allow(unused_mut)]
        let mut ordering = self.ordering.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut decorated = self.decorated.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_sort_by_ordering_with_key(
                    session, expr_head, function, ordering, items, index, decorated, item, key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
