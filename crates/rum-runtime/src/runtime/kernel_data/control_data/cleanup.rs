use super::*;
use crate::runtime::kernel_api::{
    ContinueToken, FrameRootTrace, FrameRootVisitor, SlotFinalizer, SlotWitness, TypedAction,
    TypedFrame, slot_witness,
};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct SuspendedCleanupPrimaryOutcome(Arc<Result<Expr, EvalError>>);

impl Session {
    pub(in crate::runtime) fn eval_with_cleanup(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr, cleanup] => self.drive_with_cleanup_primary(expr.clone(), cleanup.clone()),
            [init, expr, cleanup] => {
                let init_result = self.evaluate(init.clone());
                let witness = slot_witness(&init_result);
                match init_result {
                    Err(EvalError::InputRequired { request }) => suspend_with_cleanup_init(
                        self,
                        expr.clone(),
                        cleanup.clone(),
                        witness,
                        request,
                    ),
                    Ok(value) => self.resume_with_cleanup_after_init(
                        expr.clone(),
                        cleanup.clone(),
                        Ok(value),
                    ),
                    Err(error) => self.resume_with_cleanup_after_init(
                        expr.clone(),
                        cleanup.clone(),
                        Err(error),
                    ),
                }
            }
            _ => Ok(self.invalid_call_with_arg_range("WithCleanup", args, 2, 3)),
        }
    }

    fn resume_with_cleanup_after_init(
        &mut self,
        expr: Expr,
        cleanup: Expr,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) => self.drive_with_cleanup_primary_after_init(expr, cleanup),
            Err(error) => self.resume_with_cleanup_after_primary(cleanup, Err(error)),
        }
    }

    // Kept out of line: inlined into the init path, the whole-Result handoff
    // in `drive_with_cleanup_primary` costs a 136-byte copy per call.
    #[inline(never)]
    fn drive_with_cleanup_primary_after_init(
        &mut self,
        expr: Expr,
        cleanup: Expr,
    ) -> Result<Expr, EvalError> {
        self.drive_with_cleanup_primary(expr, cleanup)
    }

    fn drive_with_cleanup_primary(&mut self, expr: Expr, cleanup: Expr) -> Result<Expr, EvalError> {
        let primary_result = self.evaluate(expr);
        let witness = slot_witness(&primary_result);
        match primary_result {
            Err(EvalError::InputRequired { request }) => {
                suspend_with_cleanup_primary(self, cleanup, witness, request)
            }
            outcome => self.resume_with_cleanup_after_primary(cleanup, outcome),
        }
    }

    fn resume_with_cleanup_after_primary(
        &mut self,
        cleanup: Expr,
        primary_outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let cleanup_result = self.evaluate(cleanup.clone());
        let witness = slot_witness(&cleanup_result);
        match cleanup_result {
            Ok(value) => finish_with_cleanup_outcomes(primary_outcome, Ok(value)),
            Err(EvalError::InputRequired { request }) => {
                suspend_with_cleanup_after_primary(self, primary_outcome, witness, request)
            }
            Err(error) => finish_with_cleanup_outcomes(primary_outcome, Err(error)),
        }
    }
}

fn finish_with_cleanup_outcomes(
    primary_outcome: Result<Expr, EvalError>,
    cleanup_outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    if let (
        Err(EvalError::Thrown {
            value,
            tag,
            explicit_tag,
        }),
        Err(EvalError::Thrown {
            value: cleanup_value,
            tag: cleanup_tag,
            explicit_tag: cleanup_explicit_tag,
        }),
    ) = (&primary_outcome, &cleanup_outcome)
    {
        if tag == cleanup_tag {
            return Err(EvalError::ThrownBypassingCatch {
                value: value.clone(),
                tag: tag.clone(),
                explicit_tag: *explicit_tag,
                cleanup_value: cleanup_value.clone(),
                cleanup_tag: cleanup_tag.clone(),
                cleanup_explicit_tag: *cleanup_explicit_tag,
            });
        }
    }

    if matches!(
        primary_outcome,
        Err(EvalError::Abort | EvalError::ProtectedAbort)
    ) && matches!(
        cleanup_outcome,
        Err(EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. })
    ) {
        return primary_outcome;
    }

    match cleanup_outcome {
        Ok(_) | Err(EvalError::Return { .. }) => primary_outcome,
        Err(error) => Err(error),
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedCleanupPrimaryOutcome>();
};

#[derive(rum_derive::FrameRootTrace)]
struct EvalWithCleanupSuspensionFrame {
    expr: rum_core::Expr,
    cleanup: rum_core::Expr,
}

impl TypedFrame for EvalWithCleanupSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        #[allow(unused_mut)]
        let mut cleanup = self.cleanup.clone();
        let resumed = (|| session.resume_with_cleanup_after_init(expr, cleanup, outcome))();
        TypedAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalWithCleanupSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        _context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        crate::runtime::suspension::FrameAbandonAction::evaluate_cleanup(self.cleanup.clone())
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct DriveWithCleanupPrimarySuspensionFrame {
    cleanup: rum_core::Expr,
}

impl TypedFrame for DriveWithCleanupPrimarySuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut cleanup = self.cleanup.clone();
        let primary_outcome = outcome;
        let resumed = (|| session.resume_with_cleanup_after_primary(cleanup, primary_outcome))();
        TypedAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for DriveWithCleanupPrimarySuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        _context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        crate::runtime::suspension::FrameAbandonAction::evaluate_cleanup(self.cleanup.clone())
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeWithCleanupAfterPrimarySuspensionFrame {
    pub(super) primary_outcome:
        crate::runtime::kernel_data::control_data::cleanup::SuspendedCleanupPrimaryOutcome,
}

impl TypedFrame for ResumeWithCleanupAfterPrimarySuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        TypedAction::finalize(FinishWithCleanupFinalizer {
            primary_outcome: self.primary_outcome.clone(),
            cleanup_outcome: outcome,
        })
    }
}

struct FinishWithCleanupFinalizer {
    primary_outcome: SuspendedCleanupPrimaryOutcome,
    cleanup_outcome: Result<Expr, EvalError>,
}

impl SlotFinalizer for FinishWithCleanupFinalizer {
    type Output = Expr;

    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let primary_outcome = Arc::try_unwrap(self.primary_outcome.0)
            .unwrap_or_else(|_| panic!("completed cleanup outcome state must be unique"));
        finish_with_cleanup_outcomes(primary_outcome, self.cleanup_outcome)
    }
}

#[cold]
#[inline(never)]
fn suspend_with_cleanup_init(
    session: &mut Session,
    expr: Expr,
    cleanup: Expr,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        EvalWithCleanupSuspensionFrame { expr, cleanup },
    )
}

#[cold]
#[inline(never)]
fn suspend_with_cleanup_primary(
    session: &mut Session,
    cleanup: Expr,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        DriveWithCleanupPrimarySuspensionFrame { cleanup },
    )
}

#[cold]
#[inline(never)]
fn suspend_with_cleanup_after_primary(
    session: &mut Session,
    primary_outcome: Result<Expr, EvalError>,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    let primary_outcome = SuspendedCleanupPrimaryOutcome(Arc::new(primary_outcome));
    witness.suspend(
        session,
        request,
        ResumeWithCleanupAfterPrimarySuspensionFrame { primary_outcome },
    )
}
