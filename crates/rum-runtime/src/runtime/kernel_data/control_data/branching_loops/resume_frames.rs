use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalIfSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalIfSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(condition) => (|| session.resume_if_after_condition(&args, condition))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalSwitchSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalSwitchSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(subject) => (|| session.resume_switch_after_subject(&args, subject))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeWhileLoopSuspensionFrame1 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeWhileLoopSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = (|| session.resume_while_loop(&args, WhilePhase::Test, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeWhileLoopSuspensionFrame2 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeWhileLoopSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = (|| session.resume_while_loop(&args, WhilePhase::Body, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeUntilLoopSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) phase: crate::runtime::kernel_data::control_data::branching_loops::UntilPhase,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeUntilLoopSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut phase = self.phase.clone();
        let resumed = (|| session.resume_until_loop(&args, phase, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeForLoopSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) phase: crate::runtime::kernel_data::control_data::branching_loops::ForPhase,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeForLoopSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut phase = self.phase.clone();
        let resumed = (|| session.resume_for_loop(&args, phase, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeCompoundExpressionFromIndexSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeCompoundExpressionFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        let resumed = (|| session.resume_compound_expression_after_item(&args, index, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeWhichFromIndexSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeWhichFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        let resumed = match outcome {
            Ok(condition) => (|| session.resume_which_after_condition(&args, index, condition))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSwitchFromIndexSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) subject: rum_core::Expr,
    pub(super) index: usize,
    pub(super) branch: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeSwitchFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut subject = self.subject.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut branch = self.branch.clone();
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(outcome?);
            session.resume_switch_after_match(&args, subject, index, branch, outcome.matched)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
