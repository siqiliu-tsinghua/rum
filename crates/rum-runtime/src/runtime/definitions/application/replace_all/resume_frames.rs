use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyReplaceAllSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for ApplyReplaceAllSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <ReplaceAllTraversalOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> { Ok(outcome.expr) })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyReplaceAllWithPlanSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for ApplyReplaceAllWithPlanSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <ReplaceAllTraversalOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> { Ok(outcome.expr) })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyReplaceAllOwnedSuspensionFrame {
    pub(super) expr: rum_core::Expr,
    pub(super) rules: crate::runtime::RewriteRulePlan,
    pub(super) source: crate::runtime::RuleApplicationSource,
}

impl crate::runtime::suspension::SuspensionFrame for ApplyReplaceAllOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        #[allow(unused_mut)]
        let mut rules = self.rules.clone();
        #[allow(unused_mut)]
        let mut source = self.source.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<ReplaceAllTraversalOutcome, EvalError> { match outcome {
            Some(replaced) => Ok(ReplaceAllTraversalOutcome {
                expr: replaced,
                changed: true,
            }),
            None => session.apply_replace_all_after_miss(expr, rules, source),
        } })()
                    .map(|value| <ReplaceAllTraversalOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyReplaceAllAfterMissSuspensionFrame {
    pub(super) remaining_args: rum_core::SharedArgs,
    pub(super) rules: crate::runtime::RewriteRulePlan,
    pub(super) source: crate::runtime::RuleApplicationSource,
    pub(super) hold_attribute: crate::runtime::metadata::HoldAttribute,
}

impl crate::runtime::suspension::SuspensionFrame for ApplyReplaceAllAfterMissSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut remaining_args = self.remaining_args.clone();
        #[allow(unused_mut)]
        let mut rules = self.rules.clone();
        #[allow(unused_mut)]
        let mut source = self.source.clone();
        #[allow(unused_mut)]
        let mut hold_attribute = self.hold_attribute.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let head_outcome = <ReplaceAllTraversalOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<ReplaceAllTraversalOutcome, EvalError> {
                    let rewritten_args = Vec::with_capacity(remaining_args.len());
                    session.apply_replace_all_args(
                        head_outcome.expr,
                        rewritten_args,
                        remaining_args,
                        rules,
                        source,
                        head_outcome.changed,
                        hold_attribute,
                        0,
                        None,
                    )
                })()
                    .map(|value| <ReplaceAllTraversalOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyReplaceAllArgsSuspensionFrame {
    pub(super) state:
        crate::runtime::definitions::application::replace_all::SuspendedReplaceAllArgs,
    pub(super) source: crate::runtime::RuleApplicationSource,
    pub(super) hold_attribute: crate::runtime::metadata::HoldAttribute,
}

impl crate::runtime::suspension::SuspensionFrame for ApplyReplaceAllArgsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut source = self.source.clone();
        #[allow(unused_mut)]
        let mut hold_attribute = self.hold_attribute.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = ReplaceAllTraversalOutcome::from_continuation_expr(value?);
            session
                .resume_suspended_replace_all_args(state, source, hold_attribute, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedReplaceAllArgsSuspensionFrame {
    pub(super) state:
        crate::runtime::definitions::application::replace_all::SuspendedReplaceAllArgs,
    pub(super) source: crate::runtime::RuleApplicationSource,
    pub(super) hold_attribute: crate::runtime::metadata::HoldAttribute,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeSuspendedReplaceAllArgsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut source = self.source.clone();
        #[allow(unused_mut)]
        let mut hold_attribute = self.hold_attribute.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = ReplaceAllTraversalOutcome::from_continuation_expr(value?);
            session
                .resume_suspended_replace_all_args(state, source, hold_attribute, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyRulesOnceFromListSuspensionFrame {
    pub(super) state:
        crate::runtime::definitions::application::replace_all::SuspendedRewriteRuleMatch,
    pub(super) source: crate::runtime::RuleApplicationSource,
    pub(super) cursor: RewriteRuleScanCursor,
    pub(super) index: usize,
    pub(super) would_reject: bool,
}

impl crate::runtime::suspension::SuspensionFrame for ApplyRulesOnceFromListSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut source = self.source.clone();
        #[allow(unused_mut)]
        let mut cursor = self.cursor.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut would_reject = self.would_reject.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session
                .finish_suspended_rewrite_rule_match(
                    state,
                    source,
                    cursor,
                    index,
                    would_reject,
                    outcome,
                )
                .map(|result| result.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
