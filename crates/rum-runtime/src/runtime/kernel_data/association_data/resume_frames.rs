use super::*;
use crate::runtime::kernel_api::{ContinueToken, TypedAction, TypedFrame};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishAssociationSetPartSelectorEvaluationSuspensionFrame {
    pub(super) symbol: Symbol,
    pub(super) direct_single_key: bool,
    pub(super) rhs: Expr,
}

impl TypedFrame for FinishAssociationSetPartSelectorEvaluationSuspensionFrame {
    type Input = EvaluatedPartSelectors;
    type Output = Expr;
    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<EvaluatedPartSelectors, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let symbol = self.symbol;
        let direct_single_key = self.direct_single_key;
        let rhs = self.rhs.clone();
        let resumed = (|| {
            let evaluated_selectors = outcome?;
            session.finish_association_set_part_lvalue(
                symbol,
                direct_single_key,
                evaluated_selectors,
                rhs,
            )
        })();
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame {
    pub(super) suspended:
        crate::runtime::kernel_data::association_data::SuspendedAssociationSetLValue,
}

impl TypedFrame for ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame {
    type Input = Expr;
    type Output = Expr;
    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut suspended = self.suspended.clone();
        let resumed = (|| {
            let selector = outcome?;
            session.resume_suspended_association_set_lvalue(suspended, selector, ())
        })();
        TypedAction::complete_work(resumed.map_err(crate::runtime::kernel_api::ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedAssociationSetLvalueSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::association_data::SuspendedAssociationSetLValue,
}

impl TypedFrame for ResumeSuspendedAssociationSetLvalueSuspensionFrame {
    type Input = Expr;
    type Output = Expr;
    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let selector = outcome.map_err(crate::runtime::kernel_api::ResumeStop::failure)?;
            session.resume_suspended_association_set_lvalue(state, selector, token)
        })();
        TypedAction::complete_work(resumed)
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend_association_set_lvalue_selectors(
    session: &mut Session,
    symbol: Symbol,
    selectors: Vec<Expr>,
    rhs: Expr,
    index: usize,
    evaluated: Vec<Expr>,
    witness: crate::runtime::kernel_api::SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    let suspended = SuspendedAssociationSetLValue::freeze(symbol, selectors, rhs, index, evaluated);
    witness.suspend(
        session,
        request,
        ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame { suspended },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_suspended_association_set_lvalue<Route>(
    session: &mut Session,
    state: SuspendedAssociationSetLValue,
    witness: crate::runtime::kernel_api::SlotWitness<Expr>,
    request: RuntimeInputRequest,
    route: Route,
) -> Result<Expr, Route::Error>
where
    Route: DefinitionResumeRoute<ResumeSuspendedAssociationSetLvalueSuspensionFrame>,
{
    route.suspend(
        session,
        witness,
        request,
        ResumeSuspendedAssociationSetLvalueSuspensionFrame { state },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_association_set_part_selectors(
    session: &mut Session,
    symbol: Symbol,
    direct_single_key: bool,
    rhs: Expr,
    witness: crate::runtime::kernel_api::SlotWitness<EvaluatedPartSelectors>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        FinishAssociationSetPartSelectorEvaluationSuspensionFrame {
            symbol,
            direct_single_key,
            rhs,
        },
    )
}
