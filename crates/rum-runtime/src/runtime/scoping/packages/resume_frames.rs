use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalRestoreSealedReferencesSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for EvalRestoreSealedReferencesSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let _value = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> { Ok(system_expr("Null")) })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumePackageInstallSuspensionFrame {
    pub(super) state: crate::runtime::scoping::packages::SuspendedPackageInstall,
}

impl crate::runtime::suspension::SuspensionFrame for ResumePackageInstallSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(_) => {
                crate::runtime::suspension::FrameAction::finalize(PreparePackageInstallFinalizer {
                    state: self.state.clone(),
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Ok(
                session.fail_package_install(self.state.plan.as_ref(), error)
            )),
        }
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumePackageInstallSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.rollback_package_install(self.state.plan.as_ref());
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

struct PreparePackageInstallFinalizer {
    state: SuspendedPackageInstall,
}

impl crate::runtime::suspension::PureFrameFinalizer for PreparePackageInstallFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let plan = std::sync::Arc::try_unwrap(self.state.plan)
            .unwrap_or_else(|_| panic!("completed package install state must be unique"));
        Ok(plan.into_continuation_expr(self.state.index + 1))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinuePackageInstallFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinuePackageInstallFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let (plan, index) = PackageInstallPlan::from_continuation_expr(payload);
            session.resume_package_install(plan, index)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
