use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeNestFromIterationSuspensionFrame {
    pub(super) state: crate::functional_ops::iteration::NestEvalState,
    pub(super) iteration: usize,
    pub(super) items:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeNestFromIterationSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut iteration = self.iteration.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_nest_with_value(session, state, iteration, items, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedNestFromIterationSuspensionFrame {
    pub(super) state: crate::functional_ops::iteration::NestEvalState,
    pub(super) iteration: usize,
    pub(super) items:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedNestFromIterationSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut iteration = self.iteration.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_nest_with_value(session, state, iteration, items, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeFixedPointFromIterationSuspensionFrame {
    pub(super) state: crate::functional_ops::iteration::FixedPointEvalState,
    pub(super) iteration: usize,
    pub(super) current: rum_core::Expr,
    pub(super) seen:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
    pub(super) items:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeFixedPointFromIterationSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut iteration = self.iteration.clone();
        #[allow(unused_mut)]
        let mut current = self.current.clone();
        #[allow(unused_mut)]
        let mut seen = self.seen.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_fixed_point_with_value(
                    session, state, iteration, current, seen, items, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedFixedPointFromIterationSuspensionFrame {
    pub(super) state: crate::functional_ops::iteration::FixedPointEvalState,
    pub(super) iteration: usize,
    pub(super) current: rum_core::Expr,
    pub(super) seen:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
    pub(super) items:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedFixedPointFromIterationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut iteration = self.iteration.clone();
        #[allow(unused_mut)]
        let mut current = self.current.clone();
        #[allow(unused_mut)]
        let mut seen = self.seen.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_fixed_point_with_value(
                    session, state, iteration, current, seen, items, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeNestWhileTestFromIterationSuspensionFrame {
    pub(super) state: crate::functional_ops::iteration::NestWhileEvalState,
    pub(super) iteration: usize,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeNestWhileTestFromIterationSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut iteration = self.iteration.clone();
        #[allow(unused_mut)]
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(test_value) => (|| {
                resume_suspended_nest_while_with_test_value(
                    session, state, iteration, history, test_value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedNestWhileTestFromIterationSuspensionFrame {
    pub(super) state: crate::functional_ops::iteration::NestWhileEvalState,
    pub(super) iteration: usize,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedNestWhileTestFromIterationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut iteration = self.iteration.clone();
        #[allow(unused_mut)]
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(test_value) => (|| {
                resume_suspended_nest_while_with_test_value(
                    session, state, iteration, history, test_value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeNestWhileWithTestValueSuspensionFrame {
    pub(super) state: crate::functional_ops::iteration::NestWhileEvalState,
    pub(super) iteration: usize,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeNestWhileWithTestValueSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut iteration = self.iteration.clone();
        #[allow(unused_mut)]
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_nest_while_with_step_value(
                    session, state, iteration, history, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedNestWhileWithTestValueSuspensionFrame {
    pub(super) state: crate::functional_ops::iteration::NestWhileEvalState,
    pub(super) iteration: usize,
    pub(super) history:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedNestWhileWithTestValueSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut iteration = self.iteration.clone();
        #[allow(unused_mut)]
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_nest_while_with_step_value(
                    session, state, iteration, history, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
