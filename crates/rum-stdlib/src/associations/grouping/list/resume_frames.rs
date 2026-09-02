use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeGroupByFromIndexWithGroupsSuspensionFrame1 {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) grouped:
        crate::associations::suspended_state::PersistentAssociationGroups<rum_core::Expr>,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeGroupByFromIndexWithGroupsSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut specs = self.specs.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_group_by_with_key(
                    session, head, call_args, specs, spec_index, reducer, items, index, grouped,
                    item, key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeGroupByFromIndexWithGroupsSuspensionFrame2 {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) grouped:
        crate::associations::suspended_state::PersistentAssociationGroups<rum_core::Expr>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeGroupByFromIndexWithGroupsSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut specs = self.specs.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_group_by_with_item_value(
                    session, head, call_args, specs, spec_index, reducer, items, index, grouped,
                    key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedGroupByFromIndexSuspensionFrame1 {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) grouped:
        crate::associations::suspended_state::PersistentAssociationGroups<rum_core::Expr>,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedGroupByFromIndexSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut specs = self.specs.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_group_by_with_key(
                    session, head, call_args, specs, spec_index, reducer, items, index, grouped,
                    item, key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedGroupByFromIndexSuspensionFrame2 {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) grouped:
        crate::associations::suspended_state::PersistentAssociationGroups<rum_core::Expr>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedGroupByFromIndexSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut specs = self.specs.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_group_by_with_item_value(
                    session, head, call_args, specs, spec_index, reducer, items, index, grouped,
                    key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedGroupByWithKeySuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) grouped:
        crate::associations::suspended_state::PersistentAssociationGroups<rum_core::Expr>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedGroupByWithKeySuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut specs = self.specs.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_group_by_with_item_value(
                    session, head, call_args, specs, spec_index, reducer, items, index, grouped,
                    key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeGroupByNestedFromIndexSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) nested: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeGroupByNestedFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut specs = self.specs.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut nested = self.nested.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_group_by_nested_with_value(
                    session, head, call_args, specs, spec_index, reducer, grouped, index, nested,
                    key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedGroupByNestedFromIndexSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) nested: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedGroupByNestedFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut specs = self.specs.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut nested = self.nested.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_group_by_nested_with_value(
                    session, head, call_args, specs, spec_index, reducer, grouped, index, nested,
                    key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeGroupByReduceFromIndexSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) reducer: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) reduced: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeGroupByReduceFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut reduced = self.reduced.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_group_by_reduce_with_value(
                    session, head, call_args, reducer, grouped, index, reduced, key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedGroupByReduceFromIndexSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) reducer: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) reduced: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedGroupByReduceFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut reduced = self.reduced.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_group_by_reduce_with_value(
                    session, head, call_args, reducer, grouped, index, reduced, key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
