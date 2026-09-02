use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeTensorCallbackFromIndexSuspensionFrame {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedTensorCallbackEvalState,
    pub(super) index: usize,
    pub(super) outputs:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeTensorCallbackFromIndexSuspensionFrame {
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
        let mut outputs = self.outputs.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_tensor_callback_with_value(session, state, index, outputs, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedTensorCallbackFromIndexSuspensionFrame {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedTensorCallbackEvalState,
    pub(super) index: usize,
    pub(super) outputs:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedTensorCallbackFromIndexSuspensionFrame
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
        let mut outputs = self.outputs.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_tensor_callback_with_value(session, state, index, outputs, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeInnerFromIndicesSuspensionFrame1 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedInnerEvalState,
    pub(super) output_index: usize,
    pub(super) contract_index: usize,
    pub(super) terms:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
    pub(super) output_scalars:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeInnerFromIndicesSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut output_index = self.output_index.clone();
        #[allow(unused_mut)]
        let mut contract_index = self.contract_index.clone();
        #[allow(unused_mut)]
        let mut terms = self.terms.clone();
        #[allow(unused_mut)]
        let mut output_scalars = self.output_scalars.clone();
        let resumed = match outcome {
            Ok(term) => (|| {
                resume_suspended_inner_term_with_value(
                    session,
                    state,
                    output_index,
                    contract_index,
                    terms,
                    output_scalars,
                    term,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeInnerFromIndicesSuspensionFrame2 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedInnerEvalState,
    pub(super) output_index: usize,
    pub(super) output_scalars:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeInnerFromIndicesSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut output_index = self.output_index.clone();
        #[allow(unused_mut)]
        let mut output_scalars = self.output_scalars.clone();
        let resumed = match outcome {
            Ok(reduced) => (|| {
                resume_suspended_inner_reduced_with_value(
                    session,
                    state,
                    output_index,
                    output_scalars,
                    reduced,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedInnerFromIndicesSuspensionFrame1 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedInnerEvalState,
    pub(super) output_index: usize,
    pub(super) contract_index: usize,
    pub(super) terms:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
    pub(super) output_scalars:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedInnerFromIndicesSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut output_index = self.output_index.clone();
        #[allow(unused_mut)]
        let mut contract_index = self.contract_index.clone();
        #[allow(unused_mut)]
        let mut terms = self.terms.clone();
        #[allow(unused_mut)]
        let mut output_scalars = self.output_scalars.clone();
        let resumed = match outcome {
            Ok(term) => (|| {
                resume_suspended_inner_term_with_value(
                    session,
                    state,
                    output_index,
                    contract_index,
                    terms,
                    output_scalars,
                    term,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedInnerFromIndicesSuspensionFrame2 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedInnerEvalState,
    pub(super) output_index: usize,
    pub(super) output_scalars:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedInnerFromIndicesSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut output_index = self.output_index.clone();
        #[allow(unused_mut)]
        let mut output_scalars = self.output_scalars.clone();
        let resumed = match outcome {
            Ok(reduced) => (|| {
                resume_suspended_inner_reduced_with_value(
                    session,
                    state,
                    output_index,
                    output_scalars,
                    reduced,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
