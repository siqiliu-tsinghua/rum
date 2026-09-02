use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishAssociationSetPartSelectorEvaluationSuspensionFrame {
    pub(super) symbol: Symbol,
    pub(super) direct_single_key: bool,
    pub(super) rhs: Expr,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishAssociationSetPartSelectorEvaluationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let symbol = self.symbol;
        let direct_single_key = self.direct_single_key;
        let rhs = self.rhs.clone();
        let resumed = (|| {
            let evaluated_selectors = EvaluatedPartSelectors::from_continuation_expr(outcome?);
            session.finish_association_set_part_lvalue(
                symbol,
                direct_single_key,
                evaluated_selectors,
                rhs,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame {
    pub(super) suspended:
        crate::runtime::kernel_data::association_data::SuspendedAssociationSetLValue,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended = self.suspended.clone();
        let resumed = (|| {
            let selector = outcome?;
            session.resume_suspended_association_set_lvalue(suspended, selector)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedAssociationSetLvalueSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::association_data::SuspendedAssociationSetLValue,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedAssociationSetLvalueSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let selector = outcome?;
            session.resume_suspended_association_set_lvalue(state, selector)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
