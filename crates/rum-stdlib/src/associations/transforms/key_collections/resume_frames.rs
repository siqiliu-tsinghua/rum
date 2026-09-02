use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeKeyUnionMissingValuesFromIndexSuspensionFrame {
    pub(super) fill_function: rum_core::Expr,
    pub(super) plan: crate::associations::suspended_state::FrozenAssociationValue<
        crate::associations::transforms::key_collections::KeyUnionPlan,
    >,
    pub(super) missing_keys:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) fill_values: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeKeyUnionMissingValuesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut fill_function = self.fill_function.clone();
        #[allow(unused_mut)]
        let mut plan = self.plan.clone();
        #[allow(unused_mut)]
        let mut missing_keys = self.missing_keys.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut fill_values = self.fill_values.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(fill_value) => (|| {
                resume_suspended_key_union_missing_value(
                    session,
                    fill_function,
                    plan,
                    missing_keys,
                    index,
                    fill_values,
                    key,
                    fill_value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedKeyUnionMissingValuesFromIndexSuspensionFrame {
    pub(super) fill_function: rum_core::Expr,
    pub(super) plan: crate::associations::suspended_state::FrozenAssociationValue<
        crate::associations::transforms::key_collections::KeyUnionPlan,
    >,
    pub(super) missing_keys:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) fill_values: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedKeyUnionMissingValuesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut fill_function = self.fill_function.clone();
        #[allow(unused_mut)]
        let mut plan = self.plan.clone();
        #[allow(unused_mut)]
        let mut missing_keys = self.missing_keys.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut fill_values = self.fill_values.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(fill_value) => (|| {
                resume_suspended_key_union_missing_value(
                    session,
                    fill_function,
                    plan,
                    missing_keys,
                    index,
                    fill_values,
                    key,
                    fill_value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
