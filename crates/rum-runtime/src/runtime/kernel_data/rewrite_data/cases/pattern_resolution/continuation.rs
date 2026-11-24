use super::*;
use crate::runtime::suspension::{FrameAction, SuspensionFrame};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatcherPatternSuspensionFrame {
    pub(super) state: MatcherPatternState,
    pub(super) pending: MatcherPatternAwait,
}

impl SuspensionFrame for MatcherPatternSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_15d(self, session, outcome);
        }

        FrameAction::complete(match outcome {
            Ok(value) => resume_matcher_pattern(session, &self, value),
            Err(error) => Err(error),
        })
    }
}
impl MatcherPatternSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_15d(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        if crate::cm740_probe::frame("P0936") {
            crate::cm740_probe::hit("P0936", "OmitActualPatternParentAcceptance");
            return FrameAction::complete(outcome);
        }
        FrameAction::complete(match outcome {
            Ok(value) => resume_matcher_pattern(session, &self, value),
            Err(error) => Err(error),
        })
    }
}

#[cold]
fn resume_matcher_pattern(
    session: &mut Session,
    frame: &MatcherPatternSuspensionFrame,
    value: Expr,
) -> Result<Expr, EvalError> {
    // The immutable queued frame remains rooted while this working copy runs.
    let mut state = matcher_pattern_working_copy(&frame.state);
    let destination = match frame.pending {
        MatcherPatternAwait::Evaluate { destination, .. }
        | MatcherPatternAwait::PreparedCall { destination } => destination,
    };
    let step = accept_matcher_pattern_value(&mut state, destination, value);
    session.drive_matcher_pattern(state, step, None)
}

#[cold]
pub(super) fn matcher_pattern_working_copy(state: &MatcherPatternState) -> MatcherPatternState {
    state.clone()
}
