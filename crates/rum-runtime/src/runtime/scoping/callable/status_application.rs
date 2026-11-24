//! The status branch owns both phases of its complete Function-stage result.
use super::*;
use crate::runtime::kernel_api::{
    ContinueToken, ResumeStop, SlotWitness, TypedAction, TypedFrame, slot_witness,
};
use crate::runtime::kernel_data::control_data::StatusObjectProjection;

impl Session {
    #[inline(never)]
    pub(super) fn apply_status_function(
        &mut self,
        status: &Expr,
        projection: StatusObjectProjection<'_>,
    ) -> Result<Option<Expr>, EvalError> {
        let applied = match projection {
            StatusObjectProjection::Pure(value) => value,
            StatusObjectProjection::VirtualMessage { property, styled } => {
                let projected = self.project_status_function_message(status, property, styled);
                let witness = slot_witness(&projected);
                match projected {
                    Err(EvalError::InputRequired { request }) => {
                        return self.suspend_status_function_projection(witness, request);
                    }
                    Ok(value) => value,
                    Err(error) => return Err(error),
                }
            }
        };
        self.evaluate_status_function_value(applied)
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(super) fn apply_status_function_cm740_driven(
        &mut self,
        status: &Expr,
        projection: StatusObjectProjection<'_>,
    ) -> Result<Option<Expr>, EvalError> {
        let applied = match projection {
            StatusObjectProjection::Pure(value) => value,
            StatusObjectProjection::VirtualMessage { property, styled } => {
                let projected = {
                    let _c4_guard = crate::cm740_probe::enter("P1477");
                    let c4_child = self.project_status_function_message(status, property, styled);
                    if crate::cm740_probe::ordinary("P1477") {
                        crate::cm740_probe::hit(
                            "P1477",
                            "ActualExprKeepingSomeConversionBeforeTail",
                        );
                        return c4_child.map(Some);
                    }
                    c4_child
                };
                let witness = slot_witness(&projected);
                match projected {
                    Err(EvalError::InputRequired { request }) => {
                        return self
                            .suspend_status_function_projection_cm740_driven(witness, request);
                    }
                    Ok(value) => value,
                    Err(error) => return Err(error),
                }
            }
        };
        self.evaluate_status_function_value_cm740_driven(applied)
    }

    fn evaluate_status_function_value(&mut self, applied: Expr) -> Result<Option<Expr>, EvalError> {
        let runtime_profile_evaluate = runtime_path_timing_start("function.apply.evaluate");
        let applied_result = self.evaluate(applied);
        let witness = slot_witness(&applied_result);
        self.record_profile_runtime_path_timing_adopted_boundary(
            runtime_profile_evaluate,
            &applied_result,
        );
        let outcome = match applied_result {
            Err(EvalError::InputRequired { request }) => {
                return self.suspend_status_function_value(witness, request);
            }
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        self.finish_function_application(outcome).map(Some)
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn evaluate_status_function_value_cm740_driven(
        &mut self,
        applied: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        let runtime_profile_evaluate = runtime_path_timing_start("function.apply.evaluate");
        let applied_result = {
            let _guard = crate::cm740_probe::enter("P1480");
            let actual = self.evaluate_cm740_driven(applied);
            if crate::cm740_probe::class_lease::wants_input_return("P1480") {
                if let Err(crate::EvalError::InputRequired { .. }) = &actual {
                    _guard.input_return(&actual);
                    crate::cm740_probe::class_lease::note_input_return("P1480", 0);
                }
            }

            if crate::cm740_probe::frame("P1480") {
                crate::cm740_probe::hit("P1480", "OmitActualStatusValueAccountingAndReturnFinish");
                return actual.map(Some);
            }
            actual
        };
        let witness = slot_witness(&applied_result);
        self.record_profile_runtime_path_timing_adopted_boundary(
            runtime_profile_evaluate,
            &applied_result,
        );
        let outcome = match applied_result {
            Err(EvalError::InputRequired { request }) => {
                return self.suspend_status_function_value_cm740_driven(witness, request);
            }
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        if crate::cm740_probe::control::active("G15A") && crate::cm740_probe::frame("G15A") {
            crate::cm740_probe::hit("G15A", "shared-value-stage-finish");
            return outcome.map(Some);
        }
        self.finish_function_application(outcome).map(Some)
    }

    #[cold]
    #[inline(never)]
    fn suspend_status_function_projection(
        &mut self,
        witness: SlotWitness<Expr>,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        let _source =
            crate::cm740_probe::control::source::<StatusFunctionProjectionFrame>(&["Some"], false);
        witness.suspend(self, request, StatusFunctionProjectionFrame)
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn suspend_status_function_projection_cm740_driven(
        &mut self,
        witness: SlotWitness<Expr>,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        let _source = crate::cm740_probe::control::source::<StatusFunctionProjectionFrame>(
            &["G15A", "Some"],
            false,
        );
        crate::cm740_probe::arm_control("G15A", "status.projection.producer-pause");
        crate::cm740_probe::arm_control("Some", "StatusFunctionProjectionFrame.producer-pause");
        witness.suspend(self, request, StatusFunctionProjectionFrame)
    }

    #[cold]
    #[inline(never)]
    fn suspend_status_function_value(
        &mut self,
        witness: SlotWitness<Expr>,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        witness.suspend(self, request, StatusFunctionValueFrame)
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn suspend_status_function_value_cm740_driven(
        &mut self,
        witness: SlotWitness<Expr>,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        let _source =
            crate::cm740_probe::control::source::<StatusFunctionValueFrame>(&["G15A"], false);
        crate::cm740_probe::arm_control("G15A", "status.value.producer-pause");
        witness.suspend(self, request, StatusFunctionValueFrame)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct StatusFunctionProjectionFrame;

impl TypedFrame for StatusFunctionProjectionFrame {
    type Input = Expr;
    type Output = Option<Expr>;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Option<Expr>> {
        // Projection errors propagate before the Function Return boundary.
        // A second pause installs the distinct value frame with its own witness.
        let resumed = match outcome {
            Ok(value) => {
                if crate::cm740_probe::driven() {
                    session.evaluate_status_function_value_cm740_driven(value)
                } else {
                    session.evaluate_status_function_value(value)
                }
            }
            Err(error) => Err(error),
        };
        TypedAction::complete_work(resumed.map_err(ResumeStop::failure))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct StatusFunctionValueFrame;

impl TypedFrame for StatusFunctionValueFrame {
    type Input = Expr;
    type Output = Option<Expr>;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Option<Expr>> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_1179(self, session, outcome, _token);
        }

        TypedAction::complete(session.finish_function_application(outcome).map(Some))
    }
}
impl StatusFunctionValueFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_1179<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Option<Expr>> {
        if crate::cm740_probe::control::active("G15A") && crate::cm740_probe::frame("G15A") {
            crate::cm740_probe::hit("G15A", "value-frame-finish");
            return TypedAction::complete(outcome.map(Some));
        }
        TypedAction::complete(session.finish_function_application(outcome).map(Some))
    }
}
