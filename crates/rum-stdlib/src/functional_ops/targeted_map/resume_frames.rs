use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeMapAtFromIndexSuspensionFrame {
    pub(super) state: crate::functional_ops::targeted_map::SuspendedMapAtEvalState,
    pub(super) index: usize,
    pub(super) current: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeMapAtFromIndexSuspensionFrame {
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
        let mut current = self.current.clone();
        let resumed = match outcome {
            Ok(mapped) => {
                (|| resume_suspended_map_at_with_value(session, state, index, current, mapped))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedMapAtFromIndexSuspensionFrame {
    pub(super) state: crate::functional_ops::targeted_map::SuspendedMapAtEvalState,
    pub(super) index: usize,
    pub(super) current: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedMapAtFromIndexSuspensionFrame {
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
        let mut current = self.current.clone();
        let resumed = match outcome {
            Ok(mapped) => {
                (|| resume_suspended_map_at_with_value(session, state, index, current, mapped))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct FinishSubsetMapFromMappedSuspensionFrame;

impl rum_runtime::kernel_api::SuspensionFrame for FinishSubsetMapFromMappedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct FinishSuspendedSubsetMapFromMappedSuspensionFrame;

impl rum_runtime::kernel_api::SuspensionFrame
    for FinishSuspendedSubsetMapFromMappedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ApplySubsetMapSuspensionFrame {
    pub(super) suspended_state: crate::functional_ops::targeted_map::SuspendedSubsetMapEvalState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ApplySubsetMapSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut suspended_state = self.suspended_state.clone();
        let resumed = match outcome {
            Ok(mapped) => {
                (|| finish_suspended_subset_map_from_mapped(session, suspended_state, mapped))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
