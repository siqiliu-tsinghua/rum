use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalPositionSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for EvalPositionSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let args = self.args.clone();
        let resumed = match outcome {
            Ok(target) => session.resume_position_after_target(target, &args),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumePositionAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for ResumePositionAfterTargetSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let target = self.target.clone();
        let args = self.args.clone();
        let resumed = match outcome {
            Ok(pattern) => session.resume_position_after_pattern(target, pattern, &args),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumePositionAfterPatternSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) pattern: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for ResumePositionAfterPatternSuspensionFrame {
    type Input = CasesTraversalResolution;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CasesTraversalResolution, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let target = self.target.clone();
        let pattern = self.pattern.clone();
        let args = self.args.clone();
        let resumed = match outcome {
            Ok(traversal) => {
                session.finish_position_after_resolved_traversal(target, pattern, traversal, &args)
            }
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishPositionAfterResolvedTraversalSuspensionFrame;

impl TypedFrame for FinishPositionAfterResolvedTraversalSuspensionFrame {
    type Input = PositionCollectionOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<PositionCollectionOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let resumed = match outcome {
            Ok(outcome) => Ok(Expr::call(system_expr("List"), outcome.positions)),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinuePositionTraversalSuspensionFrame {
    pub(super) state: crate::runtime::tree_walkers::position::SuspendedPositionTraversal,
}

impl TypedFrame for ContinuePositionTraversalSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = PositionCollectionOutcome;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<PositionCollectionOutcome> {
        let state = self.state.clone();
        let resumed = match outcome {
            Ok(outcome) => session.resume_position_traversal(state, outcome, token),
            Err(error) => Err(ResumeStop::failure(error)),
        };
        TypedAction::complete_work(resumed)
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend_position_collection(
    session: &mut Session,
    witness: SlotWitness<CompiledMatchOutcome>,
    request: RuntimeInputRequest,
    walker: PostOrderPathTraversal,
    pattern: Expr,
    traversal: CasesTraversalConfig,
    positions: Vec<Expr>,
    current_path: Option<usize>,
    current_depth: usize,
) -> Result<PositionCollectionOutcome, EvalError> {
    let state = SuspendedPositionTraversal {
        walker,
        pattern,
        traversal,
        positions: PersistentPositionAccumulator::freeze(positions),
        current_path,
        current_depth,
    };
    witness.suspend(
        session,
        request,
        ContinuePositionTraversalSuspensionFrame { state },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_position_traversal<'resume>(
    session: &mut Session,
    witness: SlotWitness<CompiledMatchOutcome>,
    request: RuntimeInputRequest,
    state: SuspendedPositionTraversal,
    route: ContinueToken<'resume, ContinuePositionTraversalSuspensionFrame>,
) -> Result<PositionCollectionOutcome, ResumeStop<PositionCollectionOutcome>> {
    Err(route.suspend(
        session,
        witness,
        request,
        ContinuePositionTraversalSuspensionFrame { state },
    ))
}
