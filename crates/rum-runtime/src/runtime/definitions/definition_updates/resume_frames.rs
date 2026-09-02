use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishHeldPartSelectorEvaluationSuspensionFrame {
    pub(super) operation: crate::runtime::definitions::definition_updates::HeldUpdateOperation,
    pub(super) target: Symbol,
    pub(super) args: SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishHeldPartSelectorEvaluationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let operation = self.operation;
        let target = self.target;
        let args = self.args.clone();
        let resumed = (|| {
            let evaluated_selectors = EvaluatedPartSelectors::from_continuation_expr(outcome?);
            session.finish_suspended_held_part_selector_evaluation(
                operation,
                target,
                args,
                evaluated_selectors,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishHeldUpdateSuspensionFrame {
    pub(super) operator: &'static str,
    pub(super) target: crate::runtime::definitions::definition_updates::SuspendedHeldUpdateTarget,
    pub(super) return_value: crate::runtime::definitions::definition_updates::HeldUpdateReturn,
}

impl crate::runtime::suspension::SuspensionFrame for FinishHeldUpdateSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut operator = self.operator;
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        #[allow(unused_mut)]
        let mut return_value = self.return_value.clone();
        let resumed = (|| {
            let updated = outcome?;
            session.store_suspended_held_update(operator, target, updated, return_value)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
