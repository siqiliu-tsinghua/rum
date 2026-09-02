use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeOrderedQComparatorSuspensionFrame {
    pub(super) state: crate::predicates::ordering::SuspendedOrderedQComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeOrderedQComparatorSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_ordered_q_comparator_with_value(session, state, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedOrderedQComparatorSuspensionFrame {
    pub(super) state: crate::predicates::ordering::SuspendedOrderedQComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedOrderedQComparatorSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_ordered_q_comparator_with_value(session, state, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeLexicographicListComparatorSuspensionFrame1 {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeLexicographicListComparatorSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_forward_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeLexicographicListComparatorSuspensionFrame2 {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeLexicographicListComparatorSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_reverse_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedLexicographicListComparatorSuspensionFrame1 {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedLexicographicListComparatorSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_forward_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedLexicographicListComparatorSuspensionFrame2 {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedLexicographicListComparatorSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_reverse_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedLexicographicListComparatorWithForwardValueSuspensionFrame {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedLexicographicListComparatorWithForwardValueSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_reverse_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeLexicographicPairComparatorSuspensionFrame {
    pub(super) state: crate::predicates::ordering::LexicographicPairComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeLexicographicPairComparatorSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_lexicographic_pair_comparator_with_forward_value(session, state, value))(
                )
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeLexicographicPairComparatorWithForwardValueSuspensionFrame {
    pub(super) state: crate::predicates::ordering::LexicographicPairComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeLexicographicPairComparatorWithForwardValueSuspensionFrame
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
                (|| resume_lexicographic_pair_comparator_with_reverse_value(session, state, value))(
                )
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
