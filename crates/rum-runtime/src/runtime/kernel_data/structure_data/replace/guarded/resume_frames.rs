use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct PlanGuardedReplacePartTargetsOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPlanning,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl crate::runtime::suspension::FrameAbandon
    for PlanGuardedReplacePartTargetsOwnedSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
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

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct MatchGuardedReplacePartPatternTargetsOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPatternMatch,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl crate::runtime::suspension::FrameAbandon
    for MatchGuardedReplacePartPatternTargetsOwnedSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
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

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct SelectGuardedReplacePartTargetsOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartSelection,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl crate::runtime::suspension::FrameAbandon
    for SelectGuardedReplacePartTargetsOwnedSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

impl crate::runtime::suspension::SuspensionFrame
    for SelectGuardedReplacePartTargetsOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let _class_row = crate::cm740_probe::class_lease::begin_row(&self);
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            if crate::cm740_probe::driven() {
                session.select_guarded_replace_part_targets_suspended_cm740_driven(
                    state,
                    Some(outcome.matched),
                )
            } else {
                session.select_guarded_replace_part_targets_suspended(state, Some(outcome.matched))
            }
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct PlanGuardedReplacePartTargetsSuspendedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPlanning,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl crate::runtime::suspension::FrameAbandon
    for PlanGuardedReplacePartTargetsSuspendedSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
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

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct MatchGuardedReplacePartPatternTargetsSuspendedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPatternMatch,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl crate::runtime::suspension::FrameAbandon
    for MatchGuardedReplacePartPatternTargetsSuspendedSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
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

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct SelectGuardedReplacePartTargetsSuspendedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartSelection,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl crate::runtime::suspension::FrameAbandon
    for SelectGuardedReplacePartTargetsSuspendedSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
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
