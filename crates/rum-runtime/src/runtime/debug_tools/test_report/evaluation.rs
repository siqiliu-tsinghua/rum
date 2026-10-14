use super::model::{EvalCapturedOutcome, TestCaseSpec, TestReportOptions};
use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl ContinuationValue for EvalCapturedOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationEvalCapturedOutcome"),
            vec![
                self.value,
                self.failure_type
                    .map(Expr::string)
                    .unwrap_or_else(|| system_expr("Null")),
                Expr::call(
                    system_expr("List"),
                    self.messages
                        .into_iter()
                        .map(|message| {
                            Expr::call(
                                system_expr("List"),
                                vec![
                                    Expr::string(message.symbol),
                                    Expr::string(message.tag),
                                    Expr::string(message.text),
                                ],
                            )
                        })
                        .collect(),
                ),
                Expr::call(
                    system_expr("List"),
                    self.prints.into_iter().map(Expr::string).collect(),
                ),
                self.timing,
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args } = expr else {
            return EvalCapturedOutcome {
                value: expr,
                failure_type: Some("Error"),
                messages: Vec::new(),
                prints: Vec::new(),
                timing: Expr::integer(BigInt::from(0_u8)),
            };
        };
        if !is_system_symbol_name(&head, "InternalContinuationEvalCapturedOutcome")
            || args.len() != 5
        {
            return EvalCapturedOutcome {
                value: Expr::call_shared(head, args),
                failure_type: Some("Error"),
                messages: Vec::new(),
                prints: Vec::new(),
                timing: Expr::integer(BigInt::from(0_u8)),
            };
        }

        let mut items = args.into_iter();
        let value = items.next().unwrap_or_else(|| system_expr("$Failed"));
        let failure_type = items
            .next()
            .and_then(|expr| expr_as_string(&expr).map(failure_type_from_string));
        let messages = items
            .next()
            .and_then(runtime_messages_from_continuation_expr)
            .unwrap_or_default();
        let prints = items
            .next()
            .and_then(strings_from_continuation_expr)
            .unwrap_or_default();
        let timing = items
            .next()
            .unwrap_or_else(|| Expr::integer(BigInt::from(0_u8)));

        EvalCapturedOutcome {
            value,
            failure_type,
            messages,
            prints,
            timing,
        }
    }
}

impl Session {
    pub(in crate::runtime::debug_tools::test_report) fn test_case_output_matches(
        &mut self,
        test_case: &TestCaseSpec,
        report_same_test: Option<Expr>,
        actual: Expr,
        expected: Expr,
    ) -> Result<bool, EvalError> {
        self.test_case_output_matches_parts(
            test_case.same_test.clone(),
            report_same_test,
            actual,
            expected,
        )
    }

    pub(in crate::runtime::debug_tools::test_report) fn test_case_output_matches_parts(
        &mut self,
        case_same_test: Option<Expr>,
        report_same_test: Option<Expr>,
        actual: Expr,
        expected: Expr,
    ) -> Result<bool, EvalError> {
        let same_test = case_same_test
            .or(report_same_test)
            .unwrap_or_else(|| system_expr("SameQ"));
        let applied = Expr::call(same_test, vec![actual, expected]);
        let resolved_result = self.evaluate(applied);
        crate::resume_or_suspend!(
            self,
            resolved_result,
            ok(resolved_expr) => { let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resolved_expr); (|| {
                    Ok(is_system_symbol_name(&resolved, "True"))
                })() },
            suspend => TestCaseOutputMatchesPartsSuspensionFrame,
        )
    }

    pub(in crate::runtime::debug_tools::test_report) fn normalize_expected_message_exprs(
        &mut self,
        expected: Option<Vec<Expr>>,
    ) -> Result<Option<Vec<Expr>>, EvalError> {
        let Some(expected) = expected else {
            return Ok(None);
        };

        let mut normalized = Vec::with_capacity(expected.len());
        for expr in expected {
            let key = match self.normalize_message_emit_target(&expr) {
                Ok(key) => key,
                Err(_) => return Ok(None),
            };
            normalized.push(runtime_message_name_expr(RuntimeMessage {
                symbol: key.symbol.to_string(),
                tag: key.tag,
                text: String::new(),
            }));
        }
        Ok(Some(normalized))
    }

    pub(in crate::runtime::debug_tools::test_report) fn evaluate_test_case_input(
        &mut self,
        input: Expr,
        options: &TestReportOptions,
    ) -> Result<EvalCapturedOutcome, EvalError> {
        let wrapped = wrap_test_case_with_constraints(input, options);
        let started = Instant::now();
        let outcome_result = self.eval_expr_with_stream_capture(wrapped);
        crate::resume_or_suspend!(
            self,
            outcome_result,
            ok(outcome) => { (|| { Ok(finish_test_case_input_outcome(started, outcome)) })() },
            suspend => EvaluateTestCaseInputSuspensionFrame { started },
        )
    }

    fn render_test_outcome(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<(Expr, Option<&'static str>), EvalError> {
        match outcome {
            Ok(value) => Ok((value, None)),
            Err(error) if Self::terminated_evaluation_expr_for_error(&error).is_some() => Ok((
                Self::terminated_evaluation_expr_for_error(&error)
                    .expect("termination value should exist after guard"),
                Self::terminated_evaluation_reason_for_error(&error).map(|reason| reason.tag()),
            )),
            Err(EvalError::Abort | EvalError::ProtectedAbort) => {
                Ok((system_expr("$Aborted"), Some("Abort")))
            }
            Err(EvalError::TimeConstraintExceeded) => {
                Ok((system_expr("$Aborted"), Some("TimeConstraint")))
            }
            Err(EvalError::MemoryConstraintExceeded { .. }) => {
                Ok((system_expr("$Aborted"), Some("MemoryConstraint")))
            }
            Err(EvalError::Break) => Ok((system_expr("$Failed"), Some("Break"))),
            Err(EvalError::Continue) => Ok((system_expr("$Failed"), Some("Continue"))),
            Err(EvalError::Return { .. }) => Ok((system_expr("$Failed"), Some("Return"))),
            Err(EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. }) => {
                Ok((system_expr("$Failed"), Some("Throw")))
            }
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            Err(EvalError::Exit { code }) => Err(EvalError::Exit { code }),
            Err(error @ EvalError::ContinuationSlotMismatch(_)) => Err(error),
            Err(_) => Ok((system_expr("$Failed"), Some("Error"))),
        }
    }

    fn eval_expr_with_stream_capture(
        &mut self,
        expr: Expr,
    ) -> Result<EvalCapturedOutcome, EvalError> {
        let started = Instant::now();
        let message_checkpoint = self.messages.len();
        let print_checkpoint = self.prints.len();
        let trace_checkpoint = self.trace_events.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let evaluated = self.evaluate(expr);
        let result = self.finalize_top_level_control_flow(evaluated);

        match result {
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, EvalExprWithStreamCaptureSuspensionFrame { started, message_checkpoint, print_checkpoint, trace_checkpoint, handled_error_checkpoint });
                Err(EvalError::InputRequired { request })
            }
            result => self.finish_eval_expr_with_stream_capture(
                started,
                message_checkpoint,
                print_checkpoint,
                handled_error_checkpoint,
                result,
            ),
        }
    }

    fn finish_eval_expr_with_stream_capture(
        &mut self,
        started: Instant,
        message_checkpoint: usize,
        print_checkpoint: usize,
        handled_error_checkpoint: usize,
        result: Result<Expr, EvalError>,
    ) -> Result<EvalCapturedOutcome, EvalError> {
        match result {
            Err(error @ EvalError::ContinuationSlotMismatch(_)) => Err(error),
            Ok(value) => {
                let messages = self.split_accounted_messages(message_checkpoint);
                let prints = self.split_accounted_prints(print_checkpoint);
                self.last_message_list = messages.clone();
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
                );
                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
                );
                Ok(EvalCapturedOutcome {
                    value,
                    failure_type: None,
                    messages,
                    prints,
                    timing: elapsed_duration_expr(started.elapsed()),
                })
            }
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);
                self.emit_top_level_control_flow_messages(&err);
                let messages = self.messages[message_checkpoint..].to_vec();
                let prints = self.prints[print_checkpoint..].to_vec();
                self.last_message_list = messages.clone();
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
                );
                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
                );

                let (value, failure_type) = self.render_test_outcome(Err(err))?;
                Ok(EvalCapturedOutcome {
                    value,
                    failure_type,
                    messages,
                    prints,
                    timing: elapsed_duration_expr(started.elapsed()),
                })
            }
        }
    }
}

fn runtime_messages_from_continuation_expr(expr: Expr) -> Option<Vec<RuntimeMessage>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(&head, "List") {
        return None;
    }

    args.into_iter()
        .map(|item| {
            let Expr::Call { head, args } = item else {
                return None;
            };
            if !is_system_symbol_name(&head, "List") || args.len() != 3 {
                return None;
            }
            let mut fields = args.into_iter();
            Some(RuntimeMessage {
                symbol: expr_as_string(&fields.next()?)?.to_string(),
                tag: expr_as_string(&fields.next()?)?.to_string(),
                text: expr_as_string(&fields.next()?)?.to_string(),
            })
        })
        .collect()
}

fn strings_from_continuation_expr(expr: Expr) -> Option<Vec<String>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(&head, "List") {
        return None;
    }

    args.into_iter()
        .map(|item| expr_as_string(&item).map(str::to_owned))
        .collect()
}

fn failure_type_from_string(value: &str) -> &'static str {
    match value {
        "Abort" => "Abort",
        "Break" => "Break",
        "Continue" => "Continue",
        "Error" => "Error",
        "MemoryConstraint" => "MemoryConstraint",
        "Return" => "Return",
        "Throw" => "Throw",
        "TimeConstraint" => "TimeConstraint",
        _ => "Error",
    }
}

fn wrap_test_case_with_constraints(input: Expr, options: &TestReportOptions) -> Expr {
    let mut wrapped = input;

    if let Some(limit) = options.time_constraint.clone() {
        wrapped = Expr::call(
            system_expr("TimeConstrained"),
            vec![wrapped, limit, test_report_time_constraint_sentinel()],
        );
    }

    if let Some(limit) = options.memory_constraint.clone() {
        wrapped = Expr::call(
            system_expr("MemoryConstrained"),
            vec![wrapped, limit, test_report_memory_constraint_sentinel()],
        );
    }

    wrapped
}

fn finish_test_case_input_outcome(
    started: Instant,
    outcome: EvalCapturedOutcome,
) -> EvalCapturedOutcome {
    let elapsed = elapsed_duration_expr(started.elapsed());

    if outcome.value == test_report_time_constraint_sentinel() {
        return EvalCapturedOutcome {
            value: system_expr("$Aborted"),
            failure_type: Some("TimeConstraint"),
            messages: outcome.messages,
            prints: outcome.prints,
            timing: elapsed,
        };
    }

    if outcome.value == test_report_memory_constraint_sentinel() {
        return EvalCapturedOutcome {
            value: system_expr("$Aborted"),
            failure_type: Some("MemoryConstraint"),
            messages: outcome.messages,
            prints: outcome.prints,
            timing: elapsed,
        };
    }

    EvalCapturedOutcome {
        value: outcome.value,
        failure_type: None,
        messages: outcome.messages,
        prints: outcome.prints,
        timing: elapsed,
    }
}

fn test_report_time_constraint_sentinel() -> Expr {
    Expr::call(
        system_expr("HoldComplete"),
        vec![Expr::string("__RumTestReportTimeConstraint__")],
    )
}

fn test_report_memory_constraint_sentinel() -> Expr {
    Expr::call(
        system_expr("HoldComplete"),
        vec![Expr::string("__RumTestReportMemoryConstraint__")],
    )
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct TestCaseOutputMatchesPartsSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for TestCaseOutputMatchesPartsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<bool, EvalError> {
            Ok(is_system_symbol_name(&resolved, "True"))
        })()
                    .map(|value| <bool as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvaluateTestCaseInputSuspensionFrame {
    started: std::time::Instant,
}

impl crate::runtime::suspension::SuspensionFrame for EvaluateTestCaseInputSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut started = self.started.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <EvalCapturedOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<EvalCapturedOutcome, EvalError> { Ok(finish_test_case_input_outcome(started, outcome)) })()
                    .map(|value| <EvalCapturedOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct EvalExprWithStreamCaptureSuspensionFrame {
    started: std::time::Instant,
    message_checkpoint: usize,
    print_checkpoint: usize,
    trace_checkpoint: usize,
    handled_error_checkpoint: usize,
}

impl crate::runtime::suspension::SuspensionFrame for EvalExprWithStreamCaptureSuspensionFrame {
    fn output_boundary(&self) -> crate::runtime::suspension::OutputBoundary {
        use crate::runtime::suspension::OutputBoundary;
        OutputBoundary::all(
            self.print_checkpoint,
            self.message_checkpoint,
            self.trace_checkpoint,
        )
    }

    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut started = self.started.clone();
        #[allow(unused_mut)]
        let mut message_checkpoint = self.message_checkpoint.clone();
        #[allow(unused_mut)]
        let mut print_checkpoint = self.print_checkpoint.clone();
        #[allow(unused_mut)]
        let mut handled_error_checkpoint = self.handled_error_checkpoint.clone();
        let resumed = (|| {
            let result = session.finalize_top_level_control_flow(outcome);
            let captured = session.finish_eval_expr_with_stream_capture(
                started,
                message_checkpoint,
                print_checkpoint,
                handled_error_checkpoint,
                result,
            )?;
            Ok(captured.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalExprWithStreamCaptureSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_stream_capture(self.handled_error_checkpoint);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}
