use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeTallyTestFromCountSuspensionFrame {
    pub(super) state_test: rum_core::Expr,
    pub(super) state_index: usize,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) counts: crate::structure_ops::suspended_state::PersistentStructureCounts,
    pub(super) count_index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeTallyTestFromCountSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state_test = self.state_test.clone();
        #[allow(unused_mut)]
        let mut state_index = self.state_index.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut counts = self.counts.clone();
        #[allow(unused_mut)]
        let mut count_index = self.count_index.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_tally_test_with_value(
                    session,
                    state_test,
                    items,
                    state_index,
                    counts,
                    count_index,
                    value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedTallyTestFromCountSuspensionFrame {
    pub(super) test: rum_core::Expr,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) counts: crate::structure_ops::suspended_state::PersistentStructureCounts,
    pub(super) count_index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedTallyTestFromCountSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut test = self.test.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut counts = self.counts.clone();
        #[allow(unused_mut)]
        let mut count_index = self.count_index.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_tally_test_with_value(
                    session,
                    test,
                    items,
                    index,
                    counts,
                    count_index,
                    value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeGatherByFunctionListFromPositionSuspensionFrame {
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) functions:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) item_index: usize,
    pub(super) function_index: usize,
    pub(super) keys:
        crate::structure_ops::suspended_state::PersistentStructureSequence<(usize, rum_core::Expr)>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeGatherByFunctionListFromPositionSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut functions = self.functions.clone();
        #[allow(unused_mut)]
        let mut item_index = self.item_index.clone();
        #[allow(unused_mut)]
        let mut function_index = self.function_index.clone();
        #[allow(unused_mut)]
        let mut keys = self.keys.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_gather_by_function_list_with_key(
                    session,
                    items,
                    functions,
                    item_index,
                    function_index,
                    keys,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedGatherByFunctionListFromPositionSuspensionFrame {
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) functions:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) item_index: usize,
    pub(super) function_index: usize,
    pub(super) keys:
        crate::structure_ops::suspended_state::PersistentStructureSequence<(usize, rum_core::Expr)>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedGatherByFunctionListFromPositionSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut functions = self.functions.clone();
        #[allow(unused_mut)]
        let mut item_index = self.item_index.clone();
        #[allow(unused_mut)]
        let mut function_index = self.function_index.clone();
        #[allow(unused_mut)]
        let mut keys = self.keys.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_gather_by_function_list_with_key(
                    session,
                    items,
                    functions,
                    item_index,
                    function_index,
                    keys,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDecorateByFromIndexSuspensionFrame {
    pub(super) state: crate::structure_ops::collection_stats::SuspendedDecorateByState,
    pub(super) index: usize,
    pub(super) decorated: crate::structure_ops::suspended_state::PersistentStructureSequence<
        crate::structure_ops::collection_stats::KeyedEntry,
    >,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeDecorateByFromIndexSuspensionFrame {
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
        let mut decorated = self.decorated.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_decorate_by_with_key(session, state, index, decorated, item, key)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDecorateByFromIndexSuspensionFrame {
    pub(super) state: crate::structure_ops::collection_stats::SuspendedDecorateByState,
    pub(super) index: usize,
    pub(super) decorated: crate::structure_ops::suspended_state::PersistentStructureSequence<
        crate::structure_ops::collection_stats::KeyedEntry,
    >,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDecorateByFromIndexSuspensionFrame
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
        let mut decorated = self.decorated.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_decorate_by_with_key(session, state, index, decorated, item, key)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
