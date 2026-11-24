use super::*;
use crate::runtime::kernel_api::{ContinueToken, ResumeStop, TypedAction, TypedFrame};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct RulePatternSuspensionFrame {
    pub(super) state: RuleLhsState,
    pub(super) pending: RuleLhsAwait,
}

impl TypedFrame for RulePatternSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_185(self, session, outcome, token);
        }

        TypedAction::complete_work(match outcome {
            Ok(value) => resume_rule_pattern(session, &self, value, token),
            Err(error) => Err(ResumeStop::failure(error)),
        })
    }
}
impl RulePatternSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_185<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::frame("P0470") {
            crate::cm740_probe::hit("P0470", "OmitActualRuleLhsParentAcceptance");
            return TypedAction::complete(outcome);
        }
        TypedAction::complete_work_cm740_driven(match outcome {
            Ok(value) => resume_rule_pattern(session, &self, value, token),
            Err(error) => Err(ResumeStop::failure(error)),
        })
    }
}

#[cold]
fn resume_rule_pattern(
    session: &mut Session,
    frame: &RulePatternSuspensionFrame,
    value: Expr,
    token: ContinueToken<'_, RulePatternSuspensionFrame>,
) -> Result<Expr, ResumeStop<Expr>> {
    // The immutable queued frame remains rooted until this working copy retires.
    let mut state = rule_lhs_working_copy(&frame.state);
    let step = accept_rule_lhs_value(&mut state, frame.pending.destination, value);
    session.drive_rule_lhs(state, step, token)
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

impl crate::runtime::kernel_api::TypedFrame for RuleLhsCompletionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::kernel_api::ContinueToken<'resume, Self>,
    ) -> crate::runtime::kernel_api::TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_77e(self, session, outcome, _token);
        }

        TypedAction::complete(match outcome {
            Ok(lhs) => session.finish_rule_lhs(lhs, self.rhs.clone(), self.delayed),
            Err(error) => Err(error),
        })
    }
}
impl RuleLhsCompletionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_77e<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::kernel_api::ContinueToken<'resume, Self>,
    ) -> crate::runtime::kernel_api::TypedAction<Expr> {
        if crate::cm740_probe::frame("P0416") {
            crate::cm740_probe::hit("P0416", "ActualAttachedFrameInputBeforeTail");
            return crate::runtime::kernel_api::TypedAction::complete(outcome);
        }

        TypedAction::complete(match outcome {
            Ok(lhs) => session.finish_rule_lhs(lhs, self.rhs.clone(), self.delayed),
            Err(error) => Err(error),
        })
    }
}
