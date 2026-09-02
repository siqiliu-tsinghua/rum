use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDuplicateFreeSameTestSuspensionFrame {
    pub(super) state: crate::predicates::collections::SuspendedDuplicateFreeSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeDuplicateFreeSameTestSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_duplicate_free_same_test_with_value(session, state, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDuplicateFreeSameTestSuspensionFrame {
    pub(super) state: crate::predicates::collections::SuspendedDuplicateFreeSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDuplicateFreeSameTestSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_duplicate_free_same_test_with_value(session, state, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSameTestMatrixSuspensionFrame {
    pub(super) state: crate::predicates::collections::SuspendedSameTestMatrixState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSameTestMatrixSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_same_test_matrix_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedSameTestMatrixSuspensionFrame {
    pub(super) state: crate::predicates::collections::SuspendedSameTestMatrixState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedSameTestMatrixSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_same_test_matrix_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
