use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyReplacePartRulesOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::replace_part::SuspendedReplacePartRules,
}

impl crate::runtime::suspension::SuspensionFrame for ApplyReplacePartRulesOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let targets = Vec::<ReplacePartTarget>::from_continuation_expr(outcome?);
            session.resume_suspended_replace_part_rules(state, targets)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectRuleTargetsOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::replace_part::SuspendedReplacePartRuleTargets,
}

impl crate::runtime::suspension::SuspensionFrame for CollectRuleTargetsOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let targets = match outcome {
            Ok(value) => Vec::<ReplacePartTarget>::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_part_targets_action(
            session.resume_suspended_replace_part_rule_targets(self.state.clone(), targets),
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectPatternPositionTargetsOwnedSuspensionFrame {
    pub(super) expr: rum_core::Expr,
    pub(super) rhs: rum_core::Expr,
    pub(super) delayed: bool,
    pub(super) heads: crate::runtime::kernel_data::structure_data::replace::ReplacePartHeadsMode,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectPatternPositionTargetsOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        #[allow(unused_mut)]
        let mut rhs = self.rhs.clone();
        #[allow(unused_mut)]
        let mut delayed = self.delayed.clone();
        #[allow(unused_mut)]
        let mut heads = self.heads.clone();
        let value = outcome;
        let resumed = (|| {
            session
                .finish_replace_part_pattern_resolution(expr, rhs, delayed, heads, value?)
                .map(|targets| targets.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectPatternPositionTargetsFromCandidatesOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::replace_part::SuspendedReplacePartPatternCandidates,
    pub(super) path: std::sync::Arc<[isize]>,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectPatternPositionTargetsFromCandidatesOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(value) => CompiledMatchOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_part_targets_action(
            session.resume_suspended_replace_part_pattern_candidates(
                self.state.clone(),
                self.path.clone(),
                outcome,
            ),
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedReplacePartRulesSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::replace_part::SuspendedReplacePartRules,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedReplacePartRulesSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let targets = Vec::<ReplacePartTarget>::from_continuation_expr(value?);
            session.resume_suspended_replace_part_rules(state, targets)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedReplacePartRuleTargetsSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::replace_part::SuspendedReplacePartRuleTargets,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedReplacePartRuleTargetsSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let targets = match outcome {
            Ok(value) => Vec::<ReplacePartTarget>::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_part_targets_action(
            session.resume_suspended_replace_part_rule_targets(self.state.clone(), targets),
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedReplacePartPatternCandidatesSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::replace_part::SuspendedReplacePartPatternCandidates,
    pub(super) path: std::sync::Arc<[isize]>,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedReplacePartPatternCandidatesSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(value) => CompiledMatchOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_part_targets_action(
            session.resume_suspended_replace_part_pattern_candidates(
                self.state.clone(),
                self.path.clone(),
                outcome,
            ),
        )
    }
}

fn finish_suspended_replace_part_targets_action(
    result: Result<SuspendedReplacePartTargetsCompletion, EvalError>,
) -> crate::runtime::suspension::FrameAction {
    match result {
        Ok(completion) => crate::runtime::suspension::FrameAction::finalize(
            FinishSuspendedReplacePartTargetsFinalizer { completion },
        ),
        Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
    }
}

struct FinishSuspendedReplacePartTargetsFinalizer {
    completion: SuspendedReplacePartTargetsCompletion,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishSuspendedReplacePartTargetsFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(self.completion.finish().into_continuation_expr())
    }
}
