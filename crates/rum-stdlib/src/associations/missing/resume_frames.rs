use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingListAtTargetLevelSuspensionFrame1 {
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingListAtTargetLevelSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_list_after_child(
                    session, items, level, scan, index, kept, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingListAtTargetLevelSuspensionFrame2 {
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingListAtTargetLevelSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_list_after_child(
                    session, items, level, scan, index, kept, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteMissingListAtTargetLevelSuspensionFrame {
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteMissingListAtTargetLevelSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_list_after_child(
                    session, items, level, scan, index, kept, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame1 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_predicate(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame2 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_child(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame3 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame3
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_child(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame1 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_predicate(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame2 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_child(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
