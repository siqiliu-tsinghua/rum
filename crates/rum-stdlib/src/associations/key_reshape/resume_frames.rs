use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeKeyGroupByFromIndexSuspensionFrame {
    pub(super) function: rum_core::Expr,
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

impl rum_runtime::kernel_api::SuspensionFrame for ResumeKeyGroupByFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut function = self.function.clone();
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
            Ok(group_key) => (|| {
                resume_suspended_key_group_by_with_key(
                    session, function, reducer, entries, index, grouped, entry, group_key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedKeyGroupByFromIndexSuspensionFrame {
    pub(super) function: rum_core::Expr,
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
    for ResumeSuspendedKeyGroupByFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut function = self.function.clone();
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
            Ok(group_key) => (|| {
                resume_suspended_key_group_by_with_key(
                    session, function, reducer, entries, index, grouped, entry, group_key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeKeyGroupByReduceFromIndexSuspensionFrame {
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
    pub(super) group_key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeKeyGroupByReduceFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut reduced = self.reduced.clone();
        #[allow(unused_mut)]
        let mut group_key = self.group_key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_key_group_by_reduce_with_value(
                    session, reducer, grouped, index, reduced, group_key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedKeyGroupByReduceFromIndexSuspensionFrame {
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
    pub(super) group_key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedKeyGroupByReduceFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut reducer = self.reducer.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut reduced = self.reduced.clone();
        #[allow(unused_mut)]
        let mut group_key = self.group_key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_key_group_by_reduce_with_value(
                    session, reducer, grouped, index, reduced, group_key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
