use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishPositionalPartSelectorEvaluationSuspensionFrame {
    pub(super) owner: Symbol,
    pub(super) rhs: Expr,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishPositionalPartSelectorEvaluationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let owner = self.owner;
        let rhs = self.rhs.clone();
        let resumed = (|| {
            let evaluated_selectors = EvaluatedPartSelectors::from_continuation_expr(outcome?);
            session.finish_positional_part_assignment(owner, evaluated_selectors, rhs)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
