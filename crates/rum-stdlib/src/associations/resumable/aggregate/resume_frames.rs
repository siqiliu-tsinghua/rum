use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeMergeNestedFromIndexSuspensionFrame {
    pub(super) combiner: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) merged: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeMergeNestedFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut combiner = self.combiner.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut merged = self.merged.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_merge_nested_with_value(
                    session, combiner, grouped, index, merged, key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedMergeNestedFromIndexSuspensionFrame {
    pub(super) combiner: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) merged: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedMergeNestedFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut combiner = self.combiner.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut merged = self.merged.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_merge_nested_with_value(
                    session, combiner, grouped, index, merged, key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeMergeFromIndexSuspensionFrame {
    pub(super) combiner: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) merged: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeMergeFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut combiner = self.combiner.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut merged = self.merged.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(combined) => (|| {
                resume_suspended_merge_with_value(
                    session, combiner, grouped, index, merged, key, combined,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedMergeFromIndexSuspensionFrame {
    pub(super) combiner: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) merged: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedMergeFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut combiner = self.combiner.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut merged = self.merged.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(combined) => (|| {
                resume_suspended_merge_with_value(
                    session, combiner, grouped, index, merged, key, combined,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeCountsByFromIndexSuspensionFrame {
    pub(super) function: rum_core::Expr,
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) counts: crate::associations::suspended_state::PersistentAssociationGroups<()>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeCountsByFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut function = self.function.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut counts = self.counts.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_counts_by_with_value(session, function, items, index, counts, key)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedCountsByFromIndexSuspensionFrame {
    pub(super) function: rum_core::Expr,
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) counts: crate::associations::suspended_state::PersistentAssociationGroups<()>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedCountsByFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut function = self.function.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut counts = self.counts.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_counts_by_with_value(session, function, items, index, counts, key)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
