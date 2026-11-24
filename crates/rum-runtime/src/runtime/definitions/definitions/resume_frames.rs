use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalSetSuspensionFrame {
    pub(super) lhs: rum_core::Expr,
}

impl crate::runtime::kernel_api::TypedFrame for EvalSetSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::kernel_api::ContinueToken<'resume, Self>,
    ) -> crate::runtime::kernel_api::TypedAction<Expr> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_168(self, session, outcome, _token);
        }

        #[allow(unused_mut)]
        let mut lhs = self.lhs.clone();
        let resumed = match outcome {
            Ok(rhs) => {
                crate::runtime::profiling::logical::open_logical_phase(session, "set.finish");
                let result = (|| session.resume_set_with_rhs(lhs, rhs))();
                session.complete_profile_runtime_path_boundary("set.finish", &result);
                result
            }
            Err(error) => Err(error),
        };
        crate::runtime::kernel_api::TypedAction::complete(resumed)
    }
}
impl EvalSetSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_168<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::kernel_api::ContinueToken<'resume, Self>,
    ) -> crate::runtime::kernel_api::TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut lhs = self.lhs.clone();
        let resumed = match outcome {
            Ok(rhs) => {
                crate::runtime::profiling::logical::open_logical_phase(session, "set.finish");
                let result = (|| {
                    if crate::cm740_probe::driven() {
                        let _class_guard = crate::cm740_probe::enter("P0408");
                        let class_produced =
                            (|| session.resume_set_with_rhs_cm740_driven(lhs, rhs))();
                        if crate::cm740_probe::class_lease::wants_input_return("P0408") {
                            if let Err(crate::EvalError::InputRequired { .. }) = &class_produced {
                                _class_guard.input_return(&class_produced);
                                crate::cm740_probe::class_lease::note_input_return("P0408", 0);
                            }
                        }
                        class_produced
                    } else {
                        (|| session.resume_set_with_rhs_cm740_driven(lhs, rhs))()
                    }
                })();
                if crate::cm740_probe::frame("P0408") {
                    crate::cm740_probe::hit("P0408", "OmitOuterSetFinishAfterActualClosure");
                    return crate::runtime::kernel_api::TypedAction::complete(result);
                }
                session.complete_profile_runtime_path_boundary("set.finish", &result);
                result
            }
            Err(error) => Err(error),
        };
        crate::runtime::kernel_api::TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalRuleSuspensionFrame {
    pub(super) lhs: rum_core::Expr,
}

impl crate::runtime::kernel_api::TypedFrame for EvalRuleSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::kernel_api::ContinueToken<'resume, Self>,
    ) -> crate::runtime::kernel_api::TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut lhs = self.lhs.clone();
        let resumed = match outcome {
            Ok(rhs) => (|| session.resume_rule_with_rhs(lhs, rhs))(),
            Err(error) => Err(error),
        };
        crate::runtime::kernel_api::TypedAction::complete(resumed)
    }
}
