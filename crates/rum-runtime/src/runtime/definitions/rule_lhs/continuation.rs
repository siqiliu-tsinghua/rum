use super::*;
use crate::runtime::suspension::{FrameAction, SuspensionFrame};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct RulePatternSuspensionFrame {
    pub(super) state: RuleLhsState,
    pub(super) pending: RuleLhsAwait,
}

impl SuspensionFrame for RulePatternSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(match outcome {
            Ok(value) => resume_rule_pattern(session, &self, value),
            Err(error) => Err(error),
        })
    }
}

#[cold]
fn resume_rule_pattern(
    session: &mut Session,
    frame: &RulePatternSuspensionFrame,
    value: Expr,
) -> Result<Expr, EvalError> {
    // The immutable queued frame remains rooted until this working copy retires.
    let mut state = rule_lhs_working_copy(&frame.state);
    let step = accept_rule_lhs_value(&mut state, frame.pending.destination, value);
    session.drive_rule_lhs(state, step)
}

#[cold]
pub(super) fn rule_lhs_working_copy(state: &RuleLhsState) -> RuleLhsState {
    state.clone()
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(in crate::runtime::definitions) struct RuleLhsCompletionFrame {
    pub(in crate::runtime::definitions) rhs: Expr,
    pub(in crate::runtime::definitions) delayed: bool,
}

impl SuspensionFrame for RuleLhsCompletionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(match outcome {
            Ok(lhs) => session.finish_rule_lhs(lhs, self.rhs.clone(), self.delayed),
            Err(error) => Err(error),
        })
    }
}
