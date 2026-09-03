use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalPositionSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalPositionSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_position_after_target(target, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumePositionAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumePositionAfterTargetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(pattern) => (|| session.resume_position_after_pattern(target, pattern, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumePositionAfterPatternSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) pattern: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumePositionAfterPatternSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        #[allow(unused_mut)]
        let mut pattern = self.pattern.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = (|| {
            let traversal = CasesTraversalResolution::from_continuation_expr(outcome?);
            session.finish_position_after_resolved_traversal(target, pattern, traversal, &args)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishPositionAfterResolvedTraversalSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame
    for FinishPositionAfterResolvedTraversalSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <crate::runtime::PositionCollectionOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            Ok(Expr::call(system_expr("List"), outcome.positions))
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinuePositionTraversalSuspensionFrame {
    pub(super) state: crate::runtime::tree_walkers::position::SuspendedPositionTraversal,
}

impl crate::runtime::suspension::SuspensionFrame for ContinuePositionTraversalSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let state = self.state.clone();
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(outcome?);
            session
                .resume_position_traversal(state, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
