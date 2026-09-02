use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct FinishHeldAssociationPartSelectorEvaluationSuspensionFrame {
    pub(super) operation: AssociationPartMutationOperation,
    pub(super) target_expr: Expr,
    pub(super) target: Symbol,
    pub(super) spec: Expr,
    pub(super) continuation: EvaluatedPartSelectorsContinuation,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for FinishHeldAssociationPartSelectorEvaluationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let operation = self.operation;
        let target_expr = self.target_expr.clone();
        let target = self.target;
        let spec = self.spec.clone();
        let resumed = (|| {
            let evaluated_selectors = self.continuation.finish(outcome)?;
            finish_held_association_part_mutation(
                session,
                operation,
                target_expr,
                target,
                evaluated_selectors,
                spec,
            )
        })();
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct EvalAssociateToSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) prepared: PreparedDefinitionLhs,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalAssociateToSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut prepared = self.prepared.clone();
        let resumed = match outcome {
            Ok(expr) => (|| finish_associate_to_value(session, args, prepared, expr))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct EvalKeyDropFromSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) prepared: PreparedDefinitionLhs,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalKeyDropFromSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut prepared = self.prepared.clone();
        let resumed = match outcome {
            Ok(expr) => (|| finish_key_drop_from_value(session, args, prepared, expr))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
