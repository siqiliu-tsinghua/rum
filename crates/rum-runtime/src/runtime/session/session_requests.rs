use super::super::*;
use super::request_errors::{
    no_suspended_evaluation_failure, request_syntax_failure, session_request_error_from_eval_error,
    session_request_error_from_eval_error_with_source,
};
use crate::runtime::model::OutputMark;

/// One clock stamps every session output list, so one mark names the cut on
/// all four channels.
#[derive(Clone, Copy, Debug)]
struct RequestCheckpoint {
    mark: OutputMark,
}

impl RequestCheckpoint {
    fn new(session: &Session) -> Self {
        Self {
            mark: session.output_mark(),
        }
    }
}

impl Session {
    fn publish_reported_now(&self, reported: &mut ReportedOutput) {
        let through = self.output_mark();
        reported.publish(
            &self.messages,
            &self.prints,
            &self.trace_events,
            through,
            through,
            through,
        );
    }

    fn publish_reported_boundary(&self, reported: &mut ReportedOutput, boundary: OutputBoundary) {
        let now = self.output_mark();
        reported.publish(
            &self.messages,
            &self.prints,
            &self.trace_events,
            boundary.messages.unwrap_or(now),
            boundary.prints.unwrap_or(now),
            boundary.trace.unwrap_or(now),
        );
    }

    /// Removes the waiting report, whichever variant stored it. Every request
    /// route stores one when it returns `InputRequired`. A suspension that the
    /// non-request `eval_text`, `eval_script` or `eval_file` left has none: no
    /// response reported anything for it, so its retirement admits fresh at
    /// this clock (`cancelled_request_retires_a_non_request_suspension`).
    fn take_pending_report(&mut self) -> (RequestCheckpoint, ReportedOutput) {
        let taken = match self.pending_request.take() {
            Some(pending) => {
                let report = pending.into_report();
                (RequestCheckpoint { mark: report.mark }, report.reported)
            }
            None => {
                let checkpoint = RequestCheckpoint::new(self);
                (checkpoint, ReportedOutput::at(checkpoint.mark))
            }
        };
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingRequest,
        );
        taken
    }

    fn store_pending_request(&mut self, request: PendingRequest) {
        self.pending_request = Some(request);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingRequest,
        );
    }

    #[doc(hidden)]
    pub fn finish_cancelled_pending_evaluation_request(
        &mut self,
    ) -> Result<Option<SessionEvalResponse>, SessionRequestFailure> {
        self.activate_pending_accepted_request()?;
        if !self.active_accepted_request_is_cancelled()
            || self.pending_suspension_generation().is_none()
        {
            return Ok(None);
        }

        let (checkpoint, reported) = self.take_pending_report();
        self.abandon_pending_suspension(AbandonReason::CancelledRequest);
        let accepted = self
            .take_active_accepted_request()
            .expect("a qualified cancellation retains its accepted ticket");
        let cancellation_won = accepted.finish();
        debug_assert!(cancellation_won);

        let response = self.finish_cancelled_eval_response(checkpoint, reported);
        self.package_eval_request_outcome(Ok(response)).map(Some)
    }

    #[doc(hidden)]
    pub fn reject_overlapping_evaluation_request(&self) -> Result<(), SessionRequestFailure> {
        if self.pending_suspension_generation().is_none() {
            return Ok(());
        }

        Err(SessionRequestFailure {
            error: SessionRequestError::Other {
                message: "cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first"
                    .to_string(),
            },
            messages: Vec::new(),
            prints: Vec::new(),
            trace: Vec::new(),
        })
    }

    pub fn complete_request(&self, prefix: &str) -> SessionCompletionResponse {
        let prefix = prefix.trim().to_string();
        let matches = self.complete_symbol_prefix(&prefix);

        SessionCompletionResponse { prefix, matches }
    }

    pub fn is_complete_request(&self, input: &str) -> SessionIsCompleteResponse {
        if input.trim().is_empty() {
            return SessionIsCompleteResponse {
                status: SessionCompletenessStatus::Complete,
                indent: None,
                diagnostic: None,
            };
        }

        match parse_with_diagnostics(input) {
            Ok(_) => SessionIsCompleteResponse {
                status: SessionCompletenessStatus::Complete,
                indent: None,
                diagnostic: None,
            },
            Err(diagnostic) => {
                let diagnostic = diagnostic.structured();

                if diagnostic.is_incomplete {
                    SessionIsCompleteResponse {
                        status: SessionCompletenessStatus::Incomplete,
                        indent: Some(suggest_continuation_indent(input)),
                        diagnostic: Some(diagnostic),
                    }
                } else {
                    SessionIsCompleteResponse {
                        status: SessionCompletenessStatus::Invalid,
                        indent: None,
                        diagnostic: Some(diagnostic),
                    }
                }
            }
        }
    }

    pub fn parse_request(
        &self,
        input: &str,
    ) -> Result<SessionParseResponse, SessionRequestFailure> {
        let parsed = parse_with_diagnostics(input).map_err(request_syntax_failure)?;
        Ok(SessionParseResponse { parsed })
    }

    pub fn lower_request(
        &mut self,
        input: &str,
    ) -> Result<SessionLowerResponse, SessionRequestFailure> {
        let checkpoint = RequestCheckpoint::new(self);
        let parsed = parse_with_diagnostics(input).map_err(request_syntax_failure)?;
        let expr = self.lower_syntax_expr(parsed.syntax()).map_err(|err| {
            self.finish_request_failure(checkpoint, ReportedOutput::at(checkpoint.mark), err)
        })?;

        Ok(SessionLowerResponse { expr })
    }

    pub fn eval_request(
        &mut self,
        input: &str,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        if let Some(cancelled) = self.finish_cancelled_pending_evaluation_request()? {
            return Ok(cancelled);
        }
        self.reject_overlapping_evaluation_request()?;
        self.begin_top_level_interrupt_budget();
        let checkpoint = RequestCheckpoint::new(self);
        let reported = ReportedOutput::at(checkpoint.mark);
        if self.active_accepted_request_is_cancelled() {
            return self.finish_cancelled_active_request(checkpoint, reported);
        }
        let result = (|| {
            let parsed = parse_with_diagnostics(input).map_err(request_syntax_failure)?;
            let expr = self.lower_syntax_expr(parsed.syntax()).map_err(|err| {
                self.finish_request_failure(checkpoint, ReportedOutput::at(checkpoint.mark), err)
            })?;

            self.eval_expr_request_from_checkpoint(expr, checkpoint, reported)
        })();
        self.finish_accepted_value_request(result)
    }

    /// Evaluates an already-built expression through the top-level request protocol.
    ///
    /// The expression is evaluated as-is: no lowering pass runs, so callers are
    /// responsible for supplying resolved symbols.
    pub fn eval_expr_request(
        &mut self,
        expr: Expr,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        if let Some(cancelled) = self.finish_cancelled_pending_evaluation_request()? {
            return Ok(cancelled);
        }
        self.reject_overlapping_evaluation_request()?;
        self.begin_top_level_interrupt_budget();
        let checkpoint = RequestCheckpoint::new(self);
        let reported = ReportedOutput::at(checkpoint.mark);
        if self.active_accepted_request_is_cancelled() {
            return self.finish_cancelled_active_request(checkpoint, reported);
        }

        let result = self.eval_expr_request_from_checkpoint(expr, checkpoint, reported);
        self.finish_accepted_value_request(result)
    }

    /// Evaluates source text under a previously accepted request identity.
    pub fn eval_accepted_request(
        &mut self,
        accepted: AcceptedRequest,
        input: &str,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        self.install_accepted_request(accepted)?;
        self.eval_request(input)
    }

    /// Evaluates an expression under a previously accepted request identity.
    pub fn eval_accepted_expr_request(
        &mut self,
        accepted: AcceptedRequest,
        expr: Expr,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        self.install_accepted_request(accepted)?;
        self.eval_expr_request(expr)
    }

    fn eval_expr_request_from_checkpoint(
        &mut self,
        expr: Expr,
        checkpoint: RequestCheckpoint,
        reported: ReportedOutput,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let previous_message_list_checkpoint = self
            .current_message_list_checkpoint
            .replace(checkpoint.mark);
        let previous_message_list_dirty =
            std::mem::replace(&mut self.current_message_list_dirty, false);
        let outcome = self.evaluate(expr);
        let finalized = if self.has_continuation_fault() {
            Err(self.continuation_fault_outcome(outcome))
        } else {
            self.finalize_top_level_eval_value(outcome, checkpoint.mark)
        };
        self.current_message_list_checkpoint = previous_message_list_checkpoint;
        self.current_message_list_dirty = previous_message_list_dirty;
        let (value, termination) = match finalized {
            Ok(finalized) => finalized,
            Err(EvalError::InputRequired { .. }) => {
                return Err(
                    self.finish_input_required_request(PendingRequest::Resumable(PendingReport {
                        mark: checkpoint.mark,
                        reported,
                    })),
                );
            }
            Err(err) => return Err(self.finish_request_failure(checkpoint, reported, err)),
        };

        Ok(self.finish_reported_success(checkpoint, reported, value, termination))
    }

    pub fn resume_eval_request_with_input(
        &mut self,
        input: impl Into<String>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        if self.pending_suspension_generation().is_some() {
            if let Some(cancelled) = self.finish_cancelled_pending_evaluation_request()? {
                return Ok(cancelled);
            }
        }
        // A batch stays installed. Only `Resumable` can be resumed.
        let pending = match self.pending_request.take() {
            Some(PendingRequest::Resumable(report)) => {
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::PendingRequest,
                );
                report
            }
            other => {
                let restore = other.is_some();
                self.pending_request = other;
                if restore {
                    self.refresh_memory_owner(
                        crate::runtime::debug_tools::resources::MemoryOwner::PendingRequest,
                    );
                }
                return Err(no_suspended_evaluation_failure());
            }
        };
        let result = self.resume_eval_request_from_checkpoint(input.into(), pending);
        self.package_eval_request_outcome(result)
    }

    fn resume_eval_request_from_checkpoint(
        &mut self,
        input: String,
        pending: PendingReport,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let checkpoint = RequestCheckpoint { mark: pending.mark };
        let reported = pending.reported;
        let previous_message_list_checkpoint = self
            .current_message_list_checkpoint
            .replace(checkpoint.mark);
        let previous_message_list_dirty = std::mem::replace(
            &mut self.current_message_list_dirty,
            self.messages.has_after(checkpoint.mark),
        );
        let outcome = self.resume_suspended_evaluation(input.clone());
        let outcome = match outcome {
            Err(error @ EvalError::InputRequired { .. }) => {
                if self.active_accepted_request_is_cancelled() {
                    self.store_pending_request(PendingRequest::Resumable(PendingReport {
                        mark: checkpoint.mark,
                        reported,
                    }));
                    return self
                        .finish_cancelled_pending_evaluation_request()?
                        .ok_or_else(|| SessionRequestFailure {
                            error: SessionRequestError::Other {
                                message: "cancelled suspension was not available for abandonment"
                                    .to_string(),
                            },
                            messages: Vec::new(),
                            prints: Vec::new(),
                            trace: Vec::new(),
                        });
                }
                Err(error)
            }
            outcome => {
                if self.finish_active_accepted_request() {
                    Err(EvalError::Cancelled)
                } else {
                    outcome
                }
            }
        };
        let finalized = if self.has_continuation_fault() {
            Err(self.continuation_fault_outcome(outcome))
        } else {
            self.finalize_top_level_eval_value(outcome, checkpoint.mark)
        };
        self.current_message_list_checkpoint = previous_message_list_checkpoint;
        self.current_message_list_dirty = previous_message_list_dirty;
        match finalized {
            Ok((value, termination)) => {
                Ok(self.finish_reported_success(checkpoint, reported, value, termination))
            }
            Err(EvalError::InputRequired { .. }) => Err(self.finish_input_required_request(
                PendingRequest::Resumable(PendingReport {
                    mark: checkpoint.mark,
                    reported,
                }),
            )),
            Err(err @ EvalError::Syntax(_)) => {
                Err(self.finish_completed_request_failure(checkpoint, reported, Some(&input), err))
            }
            Err(err) => Err(self.finish_request_failure(checkpoint, reported, err)),
        }
    }

    pub fn eval_script_request(
        &mut self,
        source: &str,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        if let Some(cancelled) = self.finish_cancelled_pending_evaluation_request()? {
            return Ok(cancelled);
        }
        self.reject_overlapping_evaluation_request()?;
        let checkpoint = RequestCheckpoint::new(self);
        let reported = ReportedOutput::at(checkpoint.mark);

        let result = match self.eval_script_retaining_output(source) {
            Ok((value, termination)) => {
                Ok(self.finish_reported_success(checkpoint, reported, value, termination))
            }
            Err(EvalError::InputRequired { .. }) => Err(self.finish_input_required_request(
                PendingRequest::Batch(PendingReport {
                    mark: checkpoint.mark,
                    reported,
                }),
            )),
            Err(err) => {
                Err(self.finish_completed_request_failure(checkpoint, reported, Some(source), err))
            }
        };
        self.finish_accepted_value_request(result)
    }

    /// Evaluates a script under one previously accepted request identity.
    pub fn eval_accepted_script_request(
        &mut self,
        accepted: AcceptedRequest,
        source: &str,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        self.install_accepted_request(accepted)?;
        self.eval_script_request(source)
    }

    pub fn eval_file_request(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        if let Some(cancelled) = self.finish_cancelled_pending_evaluation_request()? {
            return Ok(cancelled);
        }
        self.reject_overlapping_evaluation_request()?;
        let checkpoint = RequestCheckpoint::new(self);
        let reported = ReportedOutput::at(checkpoint.mark);
        let path = path.as_ref();

        let result = match self.eval_file_retaining_output(path) {
            Ok((value, termination)) => {
                Ok(self.finish_reported_success(checkpoint, reported, value, termination))
            }
            Err(EvalError::InputRequired { .. }) => Err(self.finish_input_required_request(
                PendingRequest::Batch(PendingReport {
                    mark: checkpoint.mark,
                    reported,
                }),
            )),
            Err(err) => {
                let source = match &err {
                    EvalError::Syntax(_) => self
                        .file_io()
                        .and_then(|file_io| file_io.read_to_string(path).ok()),
                    _ => None,
                };

                Err(self.finish_completed_request_failure(
                    checkpoint,
                    reported,
                    source.as_deref(),
                    err,
                ))
            }
        };
        self.finish_accepted_file_request(result)
    }

    /// Evaluates a file under one previously accepted request identity.
    pub fn eval_accepted_file_request(
        &mut self,
        accepted: AcceptedRequest,
        path: impl AsRef<Path>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        self.install_accepted_request(accepted)?;
        self.eval_file_request(path)
    }

    pub fn inspect_request(&self, name: &str) -> Option<SessionInspectResponse> {
        let info = self.inspect(name)?;
        let definitions = self.inspect_definition_catalog(name).unwrap_or_default();
        let message_catalog = self.inspect_message_catalog(Some(name)).unwrap_or_default();

        Some(SessionInspectResponse {
            info,
            definitions,
            message_catalog,
        })
    }

    /// Publishes through the current mark, then splits the session lists once
    /// at the admission mark. `$MessageList` keeps that session suffix.
    fn finish_reported_success(
        &mut self,
        checkpoint: RequestCheckpoint,
        mut reported: ReportedOutput,
        value: Expr,
        termination: Option<EvalTerminationReason>,
    ) -> SessionEvalResponse {
        self.publish_reported_now(&mut reported);
        self.release_request_session_output(checkpoint.mark);
        SessionEvalResponse {
            value,
            termination,
            messages: reported.messages,
            prints: reported.prints,
            trace: reported.trace,
        }
    }

    /// A pause publishes each channel through its own boundary and keeps the
    /// reported prefix for a later completion or cancellation.
    fn finish_input_required_request(
        &mut self,
        mut request: PendingRequest,
    ) -> SessionRequestFailure {
        // One walk of the pending frames serves the publication and `$MessageList`.
        let boundary = self.pending_output_boundary();
        let report = request.report_mut();
        self.publish_reported_boundary(&mut report.reported, boundary);
        let messages = report.reported.messages.clone();
        let prints = report.reported.prints.clone();
        let trace = report.reported.trace.clone();
        let mark = report.mark;
        self.last_message_list = self.messages_through(mark, boundary.messages);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );
        self.store_pending_request(request);
        SessionRequestFailure {
            error: SessionRequestError::InputRequired {
                request: self
                    .pending_input_request()
                    .cloned()
                    .expect("input suspension should record the pending request"),
            },
            messages,
            prints,
            trace,
        }
    }

    fn finish_request_failure(
        &mut self,
        checkpoint: RequestCheckpoint,
        mut reported: ReportedOutput,
        err: EvalError,
    ) -> SessionRequestFailure {
        self.emit_non_loader_eval_message(checkpoint.mark, &err);
        self.emit_top_level_control_flow_messages(&err);
        self.publish_reported_now(&mut reported);
        self.record_unsplit_request_view(checkpoint.mark);

        SessionRequestFailure {
            error: session_request_error_from_eval_error(&err),
            messages: reported.messages,
            prints: reported.prints,
            trace: reported.trace,
        }
    }

    fn finish_completed_request_failure(
        &mut self,
        checkpoint: RequestCheckpoint,
        mut reported: ReportedOutput,
        source: Option<&str>,
        err: EvalError,
    ) -> SessionRequestFailure {
        self.publish_reported_now(&mut reported);
        self.record_unsplit_request_view(checkpoint.mark);

        SessionRequestFailure {
            error: session_request_error_from_eval_error_with_source(&err, source),
            messages: reported.messages,
            prints: reported.prints,
            trace: reported.trace,
        }
    }

    fn record_unsplit_request_view(&mut self, mark: OutputMark) {
        self.last_message_list = self.messages.after_vec(mark);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );
        self.handled_non_loader_eval_errors.truncate_after(mark);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
        );
    }

    fn release_request_session_output(&mut self, mark: OutputMark) {
        let session_messages = self.split_accounted_messages(mark);
        self.last_message_list = session_messages;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );
        let _prints = self.split_accounted_prints(mark);
        self.handled_non_loader_eval_errors.truncate_after(mark);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
        );
        self.pending_request = None;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingRequest,
        );
    }

    fn finish_accepted_value_request(
        &mut self,
        result: Result<SessionEvalResponse, SessionRequestFailure>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        if matches!(
            &result,
            Err(SessionRequestFailure {
                error: SessionRequestError::InputRequired { .. },
                ..
            })
        ) {
            if !self.active_accepted_request_is_cancelled() {
                return result;
            }

            // Abandonment clears the stored request before the drain. Take the
            // published prefix first, or the input outcome's items are lost.
            let (checkpoint, reported) = self.take_pending_report();
            self.abandon_pending_suspension(AbandonReason::CancelledRequest);
            let accepted = self
                .take_active_accepted_request()
                .expect("cancelled input suspension should retain its accepted ticket");
            let cancellation_won = accepted.finish();
            debug_assert!(cancellation_won);
            let response = self.finish_cancelled_eval_response(checkpoint, reported);
            return self.package_eval_request_outcome(Ok(response));
        }

        let result = if self.finish_active_accepted_request() {
            self.force_cancelled_eval_result(result)
        } else {
            result
        };
        self.package_eval_request_outcome(result)
    }

    fn finish_accepted_file_request(
        &mut self,
        result: Result<SessionEvalResponse, SessionRequestFailure>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        if matches!(
            &result,
            Err(SessionRequestFailure {
                error: SessionRequestError::InputRequired { .. },
                ..
            })
        ) {
            return self.finish_accepted_value_request(result);
        }
        let result = if self.finish_active_accepted_request() {
            self.force_cancelled_file_result(result)
        } else {
            result
        };
        self.package_eval_request_outcome(result)
    }

    /// One terminal packaging boundary for evaluation, batch, restore and
    /// cancellation outcomes. Admission rejections never reach this boundary:
    /// they must leave the unrelated, already pending request installed.
    #[inline]
    pub(super) fn package_eval_request_outcome(
        &self,
        result: Result<SessionEvalResponse, SessionRequestFailure>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        #[cfg(any(debug_assertions, test))]
        assert!(
            matches!(
                &result,
                Err(SessionRequestFailure {
                    error: SessionRequestError::InputRequired { .. },
                    ..
                })
            ) || (self.pending_suspension.is_none()
                && self.resuming_suspension.is_none()
                && self.abandonment_worklist.is_empty()
                && !self.abandonment_active
                && self.pending_input_request.is_none()
                && self.pending_request.is_none()),
            "terminal request outcome retained a suspension owner"
        );
        result
    }

    fn finish_cancelled_active_request(
        &mut self,
        checkpoint: RequestCheckpoint,
        reported: ReportedOutput,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let cancellation_won = self.finish_active_accepted_request();
        debug_assert!(cancellation_won);
        let response = self.finish_cancelled_eval_response(checkpoint, reported);
        self.package_eval_request_outcome(Ok(response))
    }

    fn force_cancelled_eval_result(
        &mut self,
        result: Result<SessionEvalResponse, SessionRequestFailure>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let (mut messages, prints, trace) = match result {
            Ok(response) => (response.messages, response.prints, response.trace),
            Err(failure) => (failure.messages, failure.prints, failure.trace),
        };
        append_cancel_message(&mut messages);
        self.last_message_list = messages.clone();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );

        Ok(SessionEvalResponse {
            value: Expr::call(
                system_expr("TerminatedEvaluation"),
                vec![Expr::string(EvalTerminationReason::Cancelled.tag())],
            ),
            termination: Some(EvalTerminationReason::Cancelled),
            messages,
            prints,
            trace,
        })
    }

    fn force_cancelled_file_result(
        &mut self,
        result: Result<SessionEvalResponse, SessionRequestFailure>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let (mut messages, prints, trace) = match result {
            Ok(response) => (response.messages, response.prints, response.trace),
            Err(failure) => (failure.messages, failure.prints, failure.trace),
        };
        append_cancel_message(&mut messages);
        self.last_message_list = messages.clone();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );

        Err(SessionRequestFailure {
            error: SessionRequestError::Cancelled,
            messages,
            prints,
            trace,
        })
    }

    fn finish_cancelled_eval_response(
        &mut self,
        checkpoint: RequestCheckpoint,
        mut reported: ReportedOutput,
    ) -> SessionEvalResponse {
        let (value, termination) = self
            .finalize_top_level_eval_value(Err(EvalError::Cancelled), checkpoint.mark)
            .expect("host cancellation is a terminated-evaluation value");
        self.publish_reported_now(&mut reported);
        self.release_request_session_output(checkpoint.mark);

        SessionEvalResponse {
            value,
            termination,
            messages: reported.messages,
            prints: reported.prints,
            trace: reported.trace,
        }
    }
}

fn append_cancel_message(messages: &mut Vec<RuntimeMessage>) {
    if messages
        .iter()
        .any(|message| message.symbol == "General" && message.tag == "cancel")
    {
        return;
    }

    messages.push(
        Session::runtime_message_for_non_loader_eval_error(&EvalError::Cancelled)
            .expect("host cancellation has a runtime message"),
    );
}

fn suggest_continuation_indent(input: &str) -> String {
    let base_indent = input
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .take_while(|ch| ch.is_whitespace())
                .collect::<String>()
        })
        .unwrap_or_default();

    format!("{base_indent}  ")
}
