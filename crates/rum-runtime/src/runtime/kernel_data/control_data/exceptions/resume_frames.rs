use super::*;
use crate::runtime::kernel_api::{ContinueToken, ResumeStop, SlotWitness, TypedAction, TypedFrame};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

pub(super) trait ExceptionResumeRoute<F: TypedFrame<Output = Expr>>: Sized {
    fn suspend(
        self,
        session: &mut Session,
        witness: SlotWitness<F::Input>,
        request: RuntimeInputRequest,
        fresh: F,
    ) -> Result<Expr, ResumeStop<Expr>>;
}

impl<F: TypedFrame<Output = Expr>> ExceptionResumeRoute<F> for () {
    fn suspend(
        self,
        session: &mut Session,
        witness: SlotWitness<F::Input>,
        request: RuntimeInputRequest,
        fresh: F,
    ) -> Result<Expr, ResumeStop<Expr>> {
        witness
            .suspend(session, request, fresh)
            .map_err(ResumeStop::failure)
    }
}

impl<F: TypedFrame<Output = Expr>> ExceptionResumeRoute<F> for ContinueToken<'_, F> {
    fn suspend(
        self,
        session: &mut Session,
        witness: SlotWitness<F::Input>,
        request: RuntimeInputRequest,
        fresh: F,
    ) -> Result<Expr, ResumeStop<Expr>> {
        Err(self.suspend(session, witness, request, fresh))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalCatchExceptionsSuspensionFrame {
    pub(super) handlers: std::option::Option<
        std::sync::Arc<[crate::runtime::kernel_data::control_data::exceptions::ExceptionHandler]>,
    >,
}

impl TypedFrame for EvalCatchExceptionsSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut handlers = self.handlers.clone();
        let resumed =
            (|| session.resume_suspended_catch_exceptions_after_body(handlers, outcome))();
        TypedAction::complete_work(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DispatchExceptionHandlersOwnedSuspensionFrame1 {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl TypedFrame for DispatchExceptionHandlersOwnedSuspensionFrame1 {
    type Input = CompiledMatchOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_975(self, session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let outcome = outcome.map_err(ResumeStop::failure)?;
            session.resume_suspended_exception_guard_outcome(state, outcome, (), ())
        })();
        TypedAction::complete_work(resumed)
    }
}
impl DispatchExceptionHandlersOwnedSuspensionFrame1 {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_975<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::frame("P0767") {
            crate::cm740_probe::hit("P0767", "OmitActualExceptionRhsGuardDispatch");
            return TypedAction::complete(match outcome {
                Ok(_) => Err(EvalError::ExceptionThrown {
                    exception: self.state.exception.clone(),
                }),
                Err(error) => Err(error),
            });
        }
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let outcome = outcome.map_err(ResumeStop::failure)?;
            session.resume_suspended_exception_guard_outcome_cm740_driven(state, outcome, (), ())
        })();
        TypedAction::complete_work_cm740_driven(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DispatchExceptionHandlersOwnedSuspensionFrame2 {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl TypedFrame for DispatchExceptionHandlersOwnedSuspensionFrame2 {
    type Input = CompiledMatchOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_e2d(self, session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let outcome = outcome.map_err(ResumeStop::failure)?;
            session.resume_suspended_exception_match_outcome(state, outcome, (), ())
        })();
        TypedAction::complete_work(resumed)
    }
}
impl DispatchExceptionHandlersOwnedSuspensionFrame2 {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_e2d<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::frame("P0765") {
            crate::cm740_probe::hit("P0765", "OmitActualExceptionDispatch");
            return TypedAction::complete(match outcome {
                Ok(_) => Err(EvalError::ExceptionThrown {
                    exception: self.state.exception.clone(),
                }),
                Err(error) => Err(error),
            });
        }
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let outcome = outcome.map_err(ResumeStop::failure)?;
            session.resume_suspended_exception_match_outcome_cm740_driven(state, outcome, (), ())
        })();
        TypedAction::complete_work_cm740_driven(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DispatchSuspendedExceptionHandlersSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl TypedFrame for DispatchSuspendedExceptionHandlersSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_1330(self, session, outcome, token);
        }

        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let outcome = outcome.map_err(ResumeStop::failure)?;
            session.resume_suspended_exception_match_outcome(state, outcome, token, ())
        })();
        TypedAction::complete_work(resumed)
    }
}
impl DispatchSuspendedExceptionHandlersSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_1330<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::frame("P0769") {
            crate::cm740_probe::hit("P0769", "OmitActualExceptionDispatch");
            return TypedAction::complete(match outcome {
                Ok(_) => Err(EvalError::ExceptionThrown {
                    exception: self.state.exception.clone(),
                }),
                Err(error) => Err(error),
            });
        }
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let outcome = outcome.map_err(ResumeStop::failure)?;
            session.resume_suspended_exception_match_outcome_cm740_driven(state, outcome, token, ())
        })();
        TypedAction::complete_work_cm740_driven(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedExceptionMatchOutcomeSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl TypedFrame for ResumeSuspendedExceptionMatchOutcomeSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_1839(self, session, outcome, token);
        }

        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let outcome = outcome.map_err(ResumeStop::failure)?;
            session.resume_suspended_exception_guard_outcome(state, outcome, (), token)
        })();
        TypedAction::complete_work(resumed)
    }
}
impl ResumeSuspendedExceptionMatchOutcomeSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_1839<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        if crate::cm740_probe::frame("P0773") {
            crate::cm740_probe::hit("P0773", "OmitActualExceptionRhsGuardDispatch");
            return TypedAction::complete(match outcome {
                Ok(_) => Err(EvalError::ExceptionThrown {
                    exception: self.state.exception.clone(),
                }),
                Err(error) => Err(error),
            });
        }
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let outcome = outcome.map_err(ResumeStop::failure)?;
            session.resume_suspended_exception_guard_outcome_cm740_driven(state, outcome, (), token)
        })();
        TypedAction::complete_work_cm740_driven(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateExceptionHandlerRhsSuspensionFrame;

impl TypedFrame for EvaluateExceptionHandlerRhsSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let resumed = match outcome {
            Ok(value) => (|| Ok(value))(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend_catch_exceptions(
    session: &mut Session,
    handlers: Option<Vec<ExceptionHandler>>,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        EvalCatchExceptionsSuspensionFrame {
            handlers: handlers.map(Arc::<[ExceptionHandler]>::from),
        },
    )
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub(super) fn suspend_exception_handlers_owned_guard(
    session: &mut Session,
    exception: Expr,
    payload: Expr,
    type_chain: Vec<String>,
    handlers: Vec<ExceptionHandler>,
    handler_index: usize,
    ancestor_index: usize,
    witness: SlotWitness<CompiledMatchOutcome>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    let state = SuspendedExceptionDispatch {
        exception,
        payload,
        type_chain: Arc::from(type_chain),
        handlers: Arc::from(handlers),
        handler_index,
        ancestor_index,
    };
    witness.suspend(
        session,
        request,
        DispatchExceptionHandlersOwnedSuspensionFrame1 { state },
    )
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub(super) fn suspend_exception_handlers_owned_match(
    session: &mut Session,
    exception: Expr,
    payload: Expr,
    type_chain: Vec<String>,
    handlers: Vec<ExceptionHandler>,
    handler_index: usize,
    ancestor_index: usize,
    witness: SlotWitness<CompiledMatchOutcome>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    let state = SuspendedExceptionDispatch {
        exception,
        payload,
        type_chain: Arc::from(type_chain),
        handlers: Arc::from(handlers),
        handler_index,
        ancestor_index,
    };
    witness.suspend(
        session,
        request,
        DispatchExceptionHandlersOwnedSuspensionFrame2 { state },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_suspended_exception_handlers<Route>(
    session: &mut Session,
    state: SuspendedExceptionDispatch,
    witness: SlotWitness<CompiledMatchOutcome>,
    request: RuntimeInputRequest,
    route: Route,
) -> Result<Expr, ResumeStop<Expr>>
where
    Route: ExceptionResumeRoute<DispatchSuspendedExceptionHandlersSuspensionFrame>,
{
    route.suspend(
        session,
        witness,
        request,
        DispatchSuspendedExceptionHandlersSuspensionFrame { state },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_suspended_exception_match_outcome<Route>(
    session: &mut Session,
    state: SuspendedExceptionDispatch,
    witness: SlotWitness<CompiledMatchOutcome>,
    request: RuntimeInputRequest,
    route: Route,
) -> Result<Expr, ResumeStop<Expr>>
where
    Route: ExceptionResumeRoute<ResumeSuspendedExceptionMatchOutcomeSuspensionFrame>,
{
    route.suspend(
        session,
        witness,
        request,
        ResumeSuspendedExceptionMatchOutcomeSuspensionFrame { state },
    )
}
