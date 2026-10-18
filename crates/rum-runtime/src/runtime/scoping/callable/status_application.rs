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

    fn evaluate_status_function_value(&mut self, applied: Expr) -> Result<Option<Expr>, EvalError> {
        let runtime_profile_evaluate = runtime_path_timing_start("function.apply.evaluate");
        let applied_result = self.evaluate(applied);
        let witness = slot_witness(&applied_result);
        record_profile_runtime_path_timing(
            runtime_profile_evaluate,
            runtime_profile_outcome_for_expr_result(&applied_result),
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
    fn suspend_status_function_projection(
        &mut self,
        witness: SlotWitness<Expr>,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
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
            Ok(value) => session.evaluate_status_function_value(value),
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
        TypedAction::complete(session.finish_function_application(outcome).map(Some))
    }
}
