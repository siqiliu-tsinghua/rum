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
    pub(super) state:
        crate::runtime::definitions::application::replace_all::ReplaceAllTraversalState,
}

impl crate::runtime::suspension::SuspensionFrame for ApplyReplaceAllOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let expr = self.expr.clone();
        let state = self.state.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                session.resume_replace_all_traversal(state, expr, outcome)
                    .map(|value| <ReplaceAllTraversalOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
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
