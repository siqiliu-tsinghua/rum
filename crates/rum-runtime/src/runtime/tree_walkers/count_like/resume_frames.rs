use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalCountSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for EvalCountSuspensionFrame {
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
            Ok(target) => session.resume_count_after_target(target, &args),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalMemberQSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for EvalMemberQSuspensionFrame {
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
            Ok(target) => session.resume_member_q_after_target(target, &args),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalFreeQSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for EvalFreeQSuspensionFrame {
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
            Ok(target) => session.resume_free_q_after_target(target, &args),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeCountAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for ResumeCountAfterTargetSuspensionFrame {
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
            Ok(pattern) => session.resume_count_after_pattern(target, pattern, &args),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeMemberQAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for ResumeMemberQAfterTargetSuspensionFrame {
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
            Ok(pattern) => session.resume_member_q_after_pattern(target, pattern, &args),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeFreeQAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl TypedFrame for ResumeFreeQAfterTargetSuspensionFrame {
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
            Ok(pattern) => session.resume_free_q_after_pattern(target, pattern, &args),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishCountLikeAfterPatternSuspensionFrame {
    pub(super) builtin_name: &'static str,
    pub(super) target: rum_core::Expr,
    pub(super) pattern: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
    pub(super) finish: crate::runtime::tree_walkers::count_like::CountLikeFinish,
}

impl TypedFrame for FinishCountLikeAfterPatternSuspensionFrame {
    type Input = TraversalWithoutLimitResolution;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<TraversalWithoutLimitResolution, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let builtin_name = self.builtin_name;
        let target = self.target.clone();
        let pattern = self.pattern.clone();
        let args = self.args.clone();
        let finish = self.finish;
        let resumed = match outcome {
            Ok(traversal) => session.finish_count_like_after_traversal(
                builtin_name,
                target,
                pattern,
                &args,
                finish,
                traversal,
            ),
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishCountLikeAfterTraversalSuspensionFrame {
    pub(super) finish: crate::runtime::tree_walkers::count_like::CountLikeFinish,
}

impl TypedFrame for FinishCountLikeAfterTraversalSuspensionFrame {
    type Input = TraversalCountOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<TraversalCountOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let finish = self.finish;
        let resumed = match outcome {
            Ok(outcome) => Ok(finish.finish(outcome.count)),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueCountTraversalSuspensionFrame {
    pub(super) state: crate::runtime::tree_walkers::count_like::SuspendedCountTraversal,
}

impl TypedFrame for ContinueCountTraversalSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = TraversalCountOutcome;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<TraversalCountOutcome> {
        let state = self.state.clone();
        let resumed = match outcome {
            Ok(outcome) => session.resume_count_traversal(state, outcome, token),
            Err(error) => Err(ResumeStop::failure(error)),
        };
        TypedAction::complete_work(resumed)
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend_count_like_pattern(
    session: &mut Session,
    witness: SlotWitness<TraversalWithoutLimitResolution>,
    request: RuntimeInputRequest,
    builtin_name: &'static str,
    target: Expr,
    pattern: Expr,
    args: &SharedArgs,
    finish: CountLikeFinish,
) -> Result<Expr, EvalError> {
    let args = args.clone();
    witness.suspend(
        session,
        request,
        FinishCountLikeAfterPatternSuspensionFrame {
            builtin_name,
            target,
            pattern,
            args,
            finish,
        },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_count_traversal<
    Route: DefinitionResumeRoute<ContinueCountTraversalSuspensionFrame>,
>(
    session: &mut Session,
    witness: SlotWitness<CompiledMatchOutcome>,
    request: RuntimeInputRequest,
    state: SuspendedCountTraversal,
    route: Route,
) -> Result<TraversalCountOutcome, Route::Error> {
    route.suspend(
        session,
        witness,
        request,
        ContinueCountTraversalSuspensionFrame { state },
    )
}
