use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct SuspendedCleanupPrimaryOutcome(Arc<Result<Expr, EvalError>>);

impl Session {
    pub(in crate::runtime) fn eval_with_cleanup(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr, cleanup] => {
                let primary_result = self.evaluate(expr.clone());
                self.drive_with_cleanup_primary(cleanup.clone(), primary_result)
            }
            [init, expr, cleanup] => {
                let init_result = self.evaluate(init.clone());
                let outcome = crate::resume_or_suspend!(
                    self,
                    init_result,
                    ok(value) => Ok(value),
                    suspend => {
                let expr = expr.clone();
                                    let cleanup = cleanup.clone();
                EvalWithCleanupSuspensionFrame { expr, cleanup }
                },
                );

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => {
                        self.resume_with_cleanup_after_init(expr.clone(), cleanup.clone(), outcome)
                    }
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
            Ok(_) => {
                let primary_result = self.evaluate(expr);
                self.drive_with_cleanup_primary(cleanup, primary_result)
            }
            Err(error) => self.resume_with_cleanup_after_primary(cleanup, Err(error)),
        }
    }

    fn drive_with_cleanup_primary(
        &mut self,
        cleanup: Expr,
        primary_result: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let outcome = crate::resume_or_suspend!(
            self,
            primary_result,
            ok(value) => Ok(value),
            suspend => {
        let cleanup = cleanup.clone();
        DriveWithCleanupPrimarySuspensionFrame { cleanup }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_with_cleanup_after_primary(cleanup, outcome),
        }
    }

    fn resume_with_cleanup_after_primary(
        &mut self,
        cleanup: Expr,
        primary_outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let cleanup_result = self.evaluate(cleanup.clone());
        match cleanup_result {
            Ok(value) => finish_with_cleanup_outcomes(primary_outcome, Ok(value)),
            Err(EvalError::InputRequired { request }) => {
                let primary_outcome = SuspendedCleanupPrimaryOutcome(Arc::new(primary_outcome));
                crate::resume_or_suspend!(insert self, ResumeWithCleanupAfterPrimarySuspensionFrame { primary_outcome });
                Err(EvalError::InputRequired { request })
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

impl crate::runtime::suspension::SuspensionFrame for EvalWithCleanupSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        #[allow(unused_mut)]
        let mut cleanup = self.cleanup.clone();
        let resumed = (|| session.resume_with_cleanup_after_init(expr, cleanup, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame for DriveWithCleanupPrimarySuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut cleanup = self.cleanup.clone();
        let primary_outcome = outcome;
        let resumed = (|| session.resume_with_cleanup_after_primary(cleanup, primary_outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame for ResumeWithCleanupAfterPrimarySuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        crate::runtime::suspension::FrameAction::finalize(FinishWithCleanupFinalizer {
            primary_outcome: self.primary_outcome.clone(),
            cleanup_outcome: outcome,
        })
    }
}

struct FinishWithCleanupFinalizer {
    primary_outcome: SuspendedCleanupPrimaryOutcome,
    cleanup_outcome: Result<Expr, EvalError>,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishWithCleanupFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let primary_outcome = Arc::try_unwrap(self.primary_outcome.0)
            .unwrap_or_else(|_| panic!("completed cleanup outcome state must be unique"));
        finish_with_cleanup_outcomes(primary_outcome, self.cleanup_outcome)
    }
}
