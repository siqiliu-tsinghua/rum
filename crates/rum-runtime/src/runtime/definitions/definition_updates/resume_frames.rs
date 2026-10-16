use super::*;
use crate::runtime::kernel_api::{ContinueToken, TypedAction, TypedFrame};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishHeldPartSelectorEvaluationSuspensionFrame {
    pub(super) operation: crate::runtime::definitions::definition_updates::HeldUpdateOperation,
    pub(super) target: Symbol,
    pub(super) args: SharedArgs,
}

impl TypedFrame for FinishHeldPartSelectorEvaluationSuspensionFrame {
    type Input = EvaluatedPartSelectors;
    type Output = Expr;
    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<EvaluatedPartSelectors, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let operation = self.operation;
        let target = self.target;
        let args = self.args.clone();
        let resumed = (|| {
            let evaluated_selectors = outcome?;
            session.finish_suspended_held_part_selector_evaluation(
                operation,
                target,
                args,
                evaluated_selectors,
            )
        })();
        TypedAction::complete_work(resumed.map_err(crate::runtime::kernel_api::ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishHeldUpdateSuspensionFrame {
    pub(super) operator: &'static str,
    pub(super) target: crate::runtime::definitions::definition_updates::SuspendedHeldUpdateTarget,
    pub(super) return_value: crate::runtime::definitions::definition_updates::HeldUpdateReturn,
}

impl TypedFrame for FinishHeldUpdateSuspensionFrame {
    type Input = Expr;
    type Output = Expr;
    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
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
        TypedAction::complete(resumed)
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend_held_part_selectors(
    session: &mut Session,
    operation: HeldUpdateOperation,
    target: Symbol,
    args: &[Expr],
    witness: crate::runtime::kernel_api::SlotWitness<EvaluatedPartSelectors>,
    request: RuntimeInputRequest,
) -> Result<HeldUpdateCurrent, EvalError> {
    witness
        .suspend(
            session,
            request,
            FinishHeldPartSelectorEvaluationSuspensionFrame {
                operation,
                target,
                args: SharedArgs::new(args.to_vec()),
            },
        )
        .map(HeldUpdateCurrent::Immediate)
}

#[cold]
#[inline(never)]
pub(super) fn suspend_held_update(
    session: &mut Session,
    operator: &'static str,
    target: HeldUpdateTarget,
    return_value: HeldUpdateReturn,
    witness: crate::runtime::kernel_api::SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    let target = SuspendedHeldUpdateTarget::freeze(target);
    witness.suspend(
        session,
        request,
        FinishHeldUpdateSuspensionFrame {
            operator,
            target,
            return_value,
        },
    )
}
