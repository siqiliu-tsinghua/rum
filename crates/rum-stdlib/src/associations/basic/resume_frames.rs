use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeAssociationKeyEvaluationSuspensionFrame {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) evaluated_entries:
        crate::associations::suspended_state::PersistentAssociationSequence<
            rum_runtime::kernel_api::AssociationEntryForm,
        >,
    pub(super) index: usize,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeAssociationKeyEvaluationSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut evaluated_entries = self.evaluated_entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_association_key_evaluation_with_key(
                    session,
                    entries,
                    evaluated_entries,
                    index,
                    entry,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedAssociationKeyEvaluationSuspensionFrame {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) evaluated_entries:
        crate::associations::suspended_state::PersistentAssociationSequence<
            rum_runtime::kernel_api::AssociationEntryForm,
        >,
    pub(super) index: usize,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAssociationKeyEvaluationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut evaluated_entries = self.evaluated_entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_association_key_evaluation_with_key(
                    session,
                    entries,
                    evaluated_entries,
                    index,
                    entry,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct EvalLookupSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) key: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalLookupSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        let resumed = match outcome {
            Ok(expr) => (|| eval_lookup_with_evaluated_data(session, head, expr, key, call_args))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct EvalLookupWithEvaluatedDataSuspensionFrame {
    pub(super) head: rum_core::Expr,
    pub(super) expr: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalLookupWithEvaluatedDataSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        let resumed = match outcome {
            Ok(key) => (|| finish_eval_lookup(session, head, expr, key, call_args))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct FinishEvalLookupSuspensionFrame {
    pub(super) lookup: crate::associations::suspended_state::FrozenAssociationValue<
        crate::associations::basic::PreparedLookup,
    >,
    pub(super) wrapper: std::option::Option<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for FinishEvalLookupSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut lookup = self.lookup.clone();
        #[allow(unused_mut)]
        let mut wrapper = self.wrapper.clone();
        let resumed = match outcome {
            Ok(default) => {
                (|| finish_suspended_prepared_lookup(session, lookup, Some(default), wrapper))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct FinishPreparedLookupSuspensionFrame {
    pub(super) lookup: crate::associations::suspended_state::FrozenAssociationValue<
        crate::associations::basic::PreparedLookup,
    >,
    pub(super) default: std::option::Option<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for FinishPreparedLookupSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut lookup = self.lookup.clone();
        #[allow(unused_mut)]
        let mut default = self.default.clone();
        let resumed = match outcome {
            Ok(wrapper) => (|| {
                Ok(lookup
                    .as_ref()
                    .outcome
                    .to_expr(default.as_ref(), Some(&wrapper)))
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct FinishSuspendedPreparedLookupSuspensionFrame {
    pub(super) lookup: crate::associations::suspended_state::FrozenAssociationValue<
        crate::associations::basic::PreparedLookup,
    >,
    pub(super) default: std::option::Option<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for FinishSuspendedPreparedLookupSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut lookup = self.lookup.clone();
        #[allow(unused_mut)]
        let mut default = self.default.clone();
        let resumed = match outcome {
            Ok(wrapper) => (|| {
                Ok(lookup
                    .as_ref()
                    .outcome
                    .to_expr(default.as_ref(), Some(&wrapper)))
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
