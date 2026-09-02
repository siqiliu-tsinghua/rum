use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeAssociationGroupByFromIndexWithGroupsSuspensionFrame1 {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) index: usize,
    pub(super) grouped: crate::associations::suspended_state::PersistentAssociationGroups<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeAssociationGroupByFromIndexWithGroupsSuspensionFrame1
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
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_association_group_by_with_key(
                    session, head, call_args, specs, spec_index, reducer, entries, index, grouped,
                    entry, key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeAssociationGroupByFromIndexWithGroupsSuspensionFrame2 {
    pub(super) entry_key: rum_core::Expr,
    pub(super) entry_delayed: bool,
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) index: usize,
    pub(super) grouped: crate::associations::suspended_state::PersistentAssociationGroups<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeAssociationGroupByFromIndexWithGroupsSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entry_key = self.entry_key.clone();
        #[allow(unused_mut)]
        let mut entry_delayed = self.entry_delayed.clone();
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
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                let transformed_entry = AssociationEntryForm {
                    key: entry_key,
                    value,
                    delayed: entry_delayed,
                };
                resume_suspended_association_group_by_with_entry(
                    session,
                    head,
                    call_args,
                    specs,
                    spec_index,
                    reducer,
                    entries,
                    index,
                    grouped,
                    transformed_entry,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedAssociationGroupByFromIndexSuspensionFrame1 {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) index: usize,
    pub(super) grouped: crate::associations::suspended_state::PersistentAssociationGroups<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAssociationGroupByFromIndexSuspensionFrame1
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
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_association_group_by_with_key(
                    session, head, call_args, specs, spec_index, reducer, entries, index, grouped,
                    entry, key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedAssociationGroupByFromIndexSuspensionFrame2 {
    pub(super) entry_key: rum_core::Expr,
    pub(super) entry_delayed: bool,
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) index: usize,
    pub(super) grouped: crate::associations::suspended_state::PersistentAssociationGroups<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAssociationGroupByFromIndexSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entry_key = self.entry_key.clone();
        #[allow(unused_mut)]
        let mut entry_delayed = self.entry_delayed.clone();
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
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                let transformed_entry = AssociationEntryForm {
                    key: entry_key,
                    value,
                    delayed: entry_delayed,
                };
                resume_suspended_association_group_by_with_entry(
                    session,
                    head,
                    call_args,
                    specs,
                    spec_index,
                    reducer,
                    entries,
                    index,
                    grouped,
                    transformed_entry,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedAssociationGroupByWithKeySuspensionFrame {
    pub(super) entry_key: rum_core::Expr,
    pub(super) entry_delayed: bool,
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) index: usize,
    pub(super) grouped: crate::associations::suspended_state::PersistentAssociationGroups<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAssociationGroupByWithKeySuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entry_key = self.entry_key.clone();
        #[allow(unused_mut)]
        let mut entry_delayed = self.entry_delayed.clone();
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
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                let transformed_entry = AssociationEntryForm {
                    key: entry_key,
                    value,
                    delayed: entry_delayed,
                };
                resume_suspended_association_group_by_with_entry(
                    session,
                    head,
                    call_args,
                    specs,
                    spec_index,
                    reducer,
                    entries,
                    index,
                    grouped,
                    transformed_entry,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeAssociationGroupByNestedFromIndexSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_runtime::kernel_api::AssociationEntryForm>,
    )>,
    pub(super) index: usize,
    pub(super) nested: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeAssociationGroupByNestedFromIndexSuspensionFrame
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
                resume_suspended_association_group_by_nested_with_value(
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
pub(super) struct ResumeSuspendedAssociationGroupByNestedFromIndexSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) specs: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::grouping::GroupBySpec,
    >,
    pub(super) spec_index: usize,
    pub(super) reducer: std::option::Option<rum_core::Expr>,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_runtime::kernel_api::AssociationEntryForm>,
    )>,
    pub(super) index: usize,
    pub(super) nested: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAssociationGroupByNestedFromIndexSuspensionFrame
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
                resume_suspended_association_group_by_nested_with_value(
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
pub(super) struct ResumeAssociationGroupByReduceFromIndexSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) reducer: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_runtime::kernel_api::AssociationEntryForm>,
    )>,
    pub(super) index: usize,
    pub(super) reduced: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeAssociationGroupByReduceFromIndexSuspensionFrame
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
                resume_suspended_association_group_by_reduce_with_value(
                    session, head, call_args, reducer, grouped, index, reduced, key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedAssociationGroupByReduceFromIndexSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) reducer: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_runtime::kernel_api::AssociationEntryForm>,
    )>,
    pub(super) index: usize,
    pub(super) reduced: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAssociationGroupByReduceFromIndexSuspensionFrame
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
                resume_suspended_association_group_by_reduce_with_value(
                    session, head, call_args, reducer, grouped, index, reduced, key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
