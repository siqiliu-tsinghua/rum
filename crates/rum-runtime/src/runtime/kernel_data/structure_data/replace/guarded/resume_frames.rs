use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct PlanGuardedReplacePartTargetsOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPlanning,
}

impl crate::runtime::suspension::SuspensionFrame
    for PlanGuardedReplacePartTargetsOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed =
            (|| session.plan_guarded_replace_part_pattern_targets_suspended(state, value?))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchGuardedReplacePartPatternTargetsOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPatternMatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for MatchGuardedReplacePartPatternTargetsOwnedSuspensionFrame
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
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session.match_guarded_replace_part_pattern_targets_suspended(state, Some(outcome))
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct SelectGuardedReplacePartTargetsOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartSelection,
}

impl crate::runtime::suspension::SuspensionFrame
    for SelectGuardedReplacePartTargetsOwnedSuspensionFrame
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
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session.select_guarded_replace_part_targets_suspended(state, Some(outcome.matched))
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct PlanGuardedReplacePartTargetsSuspendedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPlanning,
}

impl crate::runtime::suspension::SuspensionFrame
    for PlanGuardedReplacePartTargetsSuspendedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed =
            (|| session.plan_guarded_replace_part_pattern_targets_suspended(state, value?))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchGuardedReplacePartPatternTargetsSuspendedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPatternMatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for MatchGuardedReplacePartPatternTargetsSuspendedSuspensionFrame
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
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session.match_guarded_replace_part_pattern_targets_suspended(state, Some(outcome))
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct SelectGuardedReplacePartTargetsSuspendedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartSelection,
}

impl crate::runtime::suspension::SuspensionFrame
    for SelectGuardedReplacePartTargetsSuspendedSuspensionFrame
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
            let matched = bool::from_continuation_expr(value?);
            session.select_guarded_replace_part_targets_suspended(state, Some(matched))
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
