use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{ContinueToken, TypedAction, TypedFrame};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct FinishHeldAssociationPartSelectorEvaluationSuspensionFrame {
    pub(super) operation: AssociationPartMutationOperation,
    pub(super) target_expr: Expr,
    pub(super) target: Symbol,
    pub(super) spec: Expr,
}

impl TypedFrame for FinishHeldAssociationPartSelectorEvaluationSuspensionFrame {
    type Input = EvaluatedPartSelectors;
    type Output = Expr;
    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<EvaluatedPartSelectors, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let operation = self.operation;
        let target_expr = self.target_expr.clone();
        let target = self.target;
        let spec = self.spec.clone();
        let resumed = (|| {
            let evaluated_selectors = outcome?;
            finish_held_association_part_mutation(
                session,
                operation,
                target_expr,
                target,
                evaluated_selectors,
                spec,
            )
        })();
        TypedAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct EvalAssociateToSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) prepared: PreparedDefinitionLhs,
}

impl TypedFrame for EvalAssociateToSuspensionFrame {
    type Input = Expr;
    type Output = Expr;
    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut prepared = self.prepared.clone();
        let resumed = match outcome {
            Ok(expr) => (|| finish_associate_to_value(session, args, prepared, expr))(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct EvalKeyDropFromSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) prepared: PreparedDefinitionLhs,
}

impl TypedFrame for EvalKeyDropFromSuspensionFrame {
    type Input = Expr;
    type Output = Expr;
    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut prepared = self.prepared.clone();
        let resumed = match outcome {
            Ok(expr) => (|| finish_key_drop_from_value(session, args, prepared, expr))(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}
