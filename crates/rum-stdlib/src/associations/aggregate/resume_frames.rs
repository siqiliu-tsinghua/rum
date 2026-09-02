use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeCountsAssociationEntriesFromIndexSuspensionFrame {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) index: usize,
    pub(super) values:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
    pub(super) spec_keys: std::option::Option<
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    >,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeCountsAssociationEntriesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut values = self.values.clone();
        #[allow(unused_mut)]
        let mut spec_keys = self.spec_keys.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_counts_after_association_value(
                    session, entries, index, values, spec_keys, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedCountsAssociationEntriesFromIndexSuspensionFrame {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) index: usize,
    pub(super) values:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
    pub(super) spec_keys: std::option::Option<
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    >,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedCountsAssociationEntriesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut values = self.values.clone();
        #[allow(unused_mut)]
        let mut spec_keys = self.spec_keys.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_counts_after_association_value(
                    session, entries, index, values, spec_keys, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeCatenateDirectAssociationEntriesFromIndexSuspensionFrame {
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) index: usize,
    pub(super) values:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeCatenateDirectAssociationEntriesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut values = self.values.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_catenate_direct_association_value(
                    session, call_args, entries, index, values, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedCatenateDirectAssociationEntriesFromIndexSuspensionFrame {
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) index: usize,
    pub(super) values:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedCatenateDirectAssociationEntriesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut values = self.values.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_catenate_direct_association_value(
                    session, call_args, entries, index, values, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeCatenateStepsFromIndexSuspensionFrame {
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) steps: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::aggregate::CatenateStep,
    >,
    pub(super) index: usize,
    pub(super) flattened:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeCatenateStepsFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut steps = self.steps.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut flattened = self.flattened.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_catenate_after_association_value(
                    session, call_args, steps, index, flattened, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedCatenateStepsFromIndexSuspensionFrame {
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) steps: crate::associations::suspended_state::FrozenAssociationSequence<
        crate::associations::aggregate::CatenateStep,
    >,
    pub(super) index: usize,
    pub(super) flattened:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedCatenateStepsFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut steps = self.steps.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut flattened = self.flattened.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_catenate_after_association_value(
                    session, call_args, steps, index, flattened, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
