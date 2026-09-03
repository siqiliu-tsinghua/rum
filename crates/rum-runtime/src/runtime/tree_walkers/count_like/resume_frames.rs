use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalCountSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalCountSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_count_after_target(target, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalMemberQSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalMemberQSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_member_q_after_target(target, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalFreeQSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalFreeQSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_free_q_after_target(target, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeCountAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeCountAfterTargetSuspensionFrame {
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
            Ok(pattern) => (|| session.resume_count_after_pattern(target, pattern, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeMemberQAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeMemberQAfterTargetSuspensionFrame {
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
            Ok(pattern) => (|| session.resume_member_q_after_pattern(target, pattern, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeFreeQAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeFreeQAfterTargetSuspensionFrame {
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
            Ok(pattern) => (|| session.resume_free_q_after_pattern(target, pattern, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame for FinishCountLikeAfterPatternSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut builtin_name = self.builtin_name;
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        #[allow(unused_mut)]
        let mut pattern = self.pattern.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut finish = self.finish.clone();
        let resumed = (|| {
            let traversal = TraversalWithoutLimitResolution::from_continuation_expr(outcome?);
            session.finish_count_like_after_traversal(
                builtin_name,
                target,
                pattern,
                &args,
                finish,
                traversal,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishCountLikeAfterTraversalSuspensionFrame {
    pub(super) finish: crate::runtime::tree_walkers::count_like::CountLikeFinish,
}

impl crate::runtime::suspension::SuspensionFrame for FinishCountLikeAfterTraversalSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut finish = self.finish.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <crate::runtime::TraversalCountOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            Ok(finish.finish(outcome.count))
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueCountTraversalSuspensionFrame {
    pub(super) state: crate::runtime::tree_walkers::count_like::SuspendedCountTraversal,
}

impl crate::runtime::suspension::SuspensionFrame for ContinueCountTraversalSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session
                .resume_count_traversal(state, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
