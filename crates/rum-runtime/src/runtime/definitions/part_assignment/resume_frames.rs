use super::*;
use crate::runtime::kernel_api::{ContinueToken, TypedAction, TypedFrame};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishPositionalPartSelectorEvaluationSuspensionFrame {
    pub(super) owner: Symbol,
    pub(super) rhs: Expr,
}

impl TypedFrame for FinishPositionalPartSelectorEvaluationSuspensionFrame {
    type Input = EvaluatedPartSelectors;
    type Output = Expr;
    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<EvaluatedPartSelectors, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let owner = self.owner;
        let rhs = self.rhs.clone();
        let resumed = (|| {
            let evaluated_selectors = outcome?;
            session.finish_positional_part_assignment(owner, evaluated_selectors, rhs)
        })();
        TypedAction::complete(resumed)
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend_positional_part_selectors(
    session: &mut Session,
    owner: Symbol,
    rhs: Expr,
    witness: crate::runtime::kernel_api::SlotWitness<EvaluatedPartSelectors>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        FinishPositionalPartSelectorEvaluationSuspensionFrame { owner, rhs },
    )
}
