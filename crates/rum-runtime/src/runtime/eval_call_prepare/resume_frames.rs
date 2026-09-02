use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame {
    pub(super) state: crate::runtime::eval_call_prepare::SuspendedCallArgumentPreparation,
    pub(super) allow_sequence_splice: bool,
}

impl crate::runtime::suspension::SuspensionFrame
    for PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut allow_sequence_splice = self.allow_sequence_splice.clone();
        let value = match outcome {
            Ok(value) => value,
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_prepared_call_action(session.resume_suspended_call_after_argument(
            state,
            value,
            allow_sequence_splice,
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedCallArgumentPreparationSuspensionFrame {
    pub(super) state: crate::runtime::eval_call_prepare::SuspendedCallArgumentPreparation,
    pub(super) allow_sequence_splice: bool,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedCallArgumentPreparationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut allow_sequence_splice = self.allow_sequence_splice.clone();
        let value = match outcome {
            Ok(value) => value,
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_prepared_call_action(session.resume_suspended_call_after_argument(
            state,
            value,
            allow_sequence_splice,
        ))
    }
}

fn finish_prepared_call_action(
    result: Result<SuspendedPreparedCallArguments, EvalError>,
) -> crate::runtime::suspension::FrameAction {
    match result {
        Ok(state) => crate::runtime::suspension::FrameAction::finalize(
            FinishPreparedCallArgumentsFinalizer { state },
        ),
        Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
    }
}

struct FinishPreparedCallArgumentsFinalizer {
    state: SuspendedPreparedCallArguments,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishPreparedCallArgumentsFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(finish_suspended_prepared_call(self.state))
    }
}
