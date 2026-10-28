use super::*;
use crate::runtime::model::OutputMark;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_evaluation_data(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, properties) = match args {
            [expr] => (expr, None),
            [expr, properties] => (expr, Some(properties)),
            _ => return Ok(self.invalid_call_with_arg_range("EvaluationData", args, 1, 2)),
        };

        let checkpoint = DebugCaptureCheckpoint::new(self);
        let prior_tracing = self.tracing_enabled;
        self.tracing_enabled = true;
        let started = Instant::now();
        let input_string = expr.to_string();
        let input_expr = expr.clone();

        let body_result = self.evaluate(expr.clone());
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => {
        let input_string = input_string.clone();
                    let input_expr = input_expr.clone();
        EvalEvaluationDataSuspensionFrame {
            checkpoint,
            properties: properties.cloned(),
            prior_tracing,
            started,
            input_expr,
            input_string: std::sync::Arc::new(input_string),
        }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_evaluation_data_capture(
                checkpoint,
                prior_tracing,
                started,
                &input_expr,
                &input_string,
                outcome,
                properties.cloned(),
            ),
        }
    }

    fn finish_evaluation_data_capture(
        &mut self,
        checkpoint: DebugCaptureCheckpoint,
        prior_tracing: bool,
        started: Instant,
        input_expr: &Expr,
        input_string: &str,
        outcome: Result<Expr, EvalError>,
        properties: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        self.tracing_enabled = prior_tracing;

        if let Err(error) = &outcome {
            if Self::terminated_evaluation_expr_for_error(error).is_some() {
                self.emit_non_loader_eval_message(checkpoint.mark, error);
            }
            self.emit_top_level_control_flow_messages(error);
        }

        let messages = self.split_accounted_messages(checkpoint.mark);
        let prints = self.split_accounted_prints(checkpoint.mark);
        let (events, depths) = self.trace_pairs_after(checkpoint.mark);
        let trace_root = self.structured_trace_expr_from_events(input_expr, &events, &depths);

        if !prior_tracing {
            self.truncate_trace_from(checkpoint.mark);
        }
        self.handled_non_loader_eval_errors
            .truncate_after(checkpoint.mark);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
        );

        let elapsed = elapsed_duration_expr(started.elapsed());
        let (result_expr, success, failure_type) = match outcome {
            Ok(value) => (value, true, system_expr("None")),
            Err(error) if Self::terminated_evaluation_expr_for_error(&error).is_some() => (
                Self::terminated_evaluation_expr_for_error(&error)
                    .expect("termination value should exist after guard"),
                false,
                Expr::string(
                    Self::terminated_evaluation_reason_for_error(&error)
                        .expect("termination reason should exist after guard")
                        .tag(),
                ),
            ),
            Err(EvalError::Abort | EvalError::ProtectedAbort) => {
                (system_expr("$Aborted"), false, Expr::string("Abort"))
            }
            Err(EvalError::TimeConstraintExceeded) => (
                system_expr("$Aborted"),
                false,
                Expr::string("TimeConstraint"),
            ),
            Err(EvalError::MemoryConstraintExceeded { .. }) => (
                system_expr("$Aborted"),
                false,
                Expr::string("MemoryConstraint"),
            ),
            Err(EvalError::Break) => (system_expr("$Failed"), false, Expr::string("Break")),
            Err(EvalError::Continue) => (system_expr("$Failed"), false, Expr::string("Continue")),
            Err(EvalError::Return { .. }) => {
                (system_expr("$Failed"), false, Expr::string("Return"))
            }
            Err(EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. }) => {
                (system_expr("$Failed"), false, Expr::string("Throw"))
            }
            Err(EvalError::InputRequired { request }) => {
                return Err(EvalError::InputRequired { request });
            }
            Err(EvalError::Exit { code }) => return Err(EvalError::Exit { code }),
            Err(error @ EvalError::ContinuationSlotMismatch(_)) => return Err(error),
            Err(_) => (system_expr("$Failed"), false, Expr::string("Error")),
        };

        let data = render_association(vec![
            (
                Expr::string("Input"),
                Expr::call(system_expr("Hold"), vec![input_expr.clone()]),
            ),
            (Expr::string("Result"), result_expr),
            (Expr::string("Success"), debug_bool_expr(success)),
            (Expr::string("FailureType"), failure_type),
            (
                Expr::string("Messages"),
                Expr::call(
                    system_expr("List"),
                    messages
                        .iter()
                        .cloned()
                        .map(runtime_message_name_expr)
                        .collect(),
                ),
            ),
            (
                Expr::string("MessagesExpressions"),
                Expr::call(
                    system_expr("List"),
                    messages
                        .iter()
                        .cloned()
                        .map(runtime_message_emit_expr)
                        .collect(),
                ),
            ),
            (
                Expr::string("MessagesText"),
                Expr::call(
                    system_expr("List"),
                    messages
                        .iter()
                        .map(|message| Expr::string(message.text.clone()))
                        .collect(),
                ),
            ),
            (
                Expr::string("OutputLog"),
                Expr::call(
                    system_expr("List"),
                    prints.into_iter().map(Expr::string).collect(),
                ),
            ),
            (Expr::string("Trace"), trace_root),
            (Expr::string("Timing"), elapsed.clone()),
            (Expr::string("AbsoluteTiming"), elapsed),
            (
                Expr::string("InputString"),
                Expr::string(input_string.to_string()),
            ),
        ]);
        self.project_evaluation_data_properties(data, properties)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, rum_derive::FrameRootTrace)]
struct DebugCaptureCheckpoint {
    mark: OutputMark,
}

impl DebugCaptureCheckpoint {
    fn new(session: &Session) -> Self {
        Self {
            mark: session.output_mark(),
        }
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct EvalEvaluationDataSuspensionFrame {
    properties: Option<Expr>,
    pub(super) checkpoint: crate::runtime::debug_tools::evaluation_data::DebugCaptureCheckpoint,
    pub(super) prior_tracing: bool,
    pub(super) started: std::time::Instant,
    pub(super) input_expr: rum_core::Expr,
    pub(super) input_string: std::sync::Arc<std::string::String>,
}

impl crate::runtime::suspension::SuspensionFrame for EvalEvaluationDataSuspensionFrame {
    fn output_boundary(&self) -> crate::runtime::suspension::OutputBoundary {
        use crate::runtime::suspension::OutputBoundary;
        OutputBoundary::all(
            self.checkpoint.mark,
            self.checkpoint.mark,
            self.checkpoint.mark,
        )
    }

    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut checkpoint = self.checkpoint.clone();
        #[allow(unused_mut)]
        let mut prior_tracing = self.prior_tracing.clone();
        #[allow(unused_mut)]
        let mut started = self.started.clone();
        #[allow(unused_mut)]
        let mut input_expr = self.input_expr.clone();
        #[allow(unused_mut)]
        let mut input_string = self.input_string.clone();
        let resumed = (|| {
            session.finish_evaluation_data_capture(
                checkpoint,
                prior_tracing,
                started,
                &input_expr,
                input_string.as_str(),
                outcome,
                self.properties.clone(),
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalEvaluationDataSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_evaluation_data_capture(self.checkpoint.mark, self.prior_tracing);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}
