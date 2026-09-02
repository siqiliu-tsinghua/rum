use super::super::*;

#[derive(Clone, Copy, Debug)]
struct RequestCheckpoint {
    message_checkpoint: usize,
    print_checkpoint: usize,
    handled_error_checkpoint: usize,
    trace_checkpoint: usize,
}

impl RequestCheckpoint {
    fn new(session: &Session) -> Self {
        Self {
            message_checkpoint: session.messages.len(),
            print_checkpoint: session.prints.len(),
            handled_error_checkpoint: session.handled_non_loader_eval_errors.len(),
            trace_checkpoint: session.trace_events.len(),
        }
    }
}

impl From<RequestCheckpoint> for PendingEvalRequest {
    fn from(checkpoint: RequestCheckpoint) -> Self {
        Self {
            message_checkpoint: checkpoint.message_checkpoint,
            print_checkpoint: checkpoint.print_checkpoint,
            handled_error_checkpoint: checkpoint.handled_error_checkpoint,
            trace_checkpoint: checkpoint.trace_checkpoint,
        }
    }
}

impl From<PendingEvalRequest> for RequestCheckpoint {
    fn from(state: PendingEvalRequest) -> Self {
        Self {
            message_checkpoint: state.message_checkpoint,
            print_checkpoint: state.print_checkpoint,
            handled_error_checkpoint: state.handled_error_checkpoint,
            trace_checkpoint: state.trace_checkpoint,
        }
    }
}

impl Session {
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
        let expr = self
            .lower_syntax_expr(parsed.syntax())
            .map_err(|err| self.finish_request_failure(checkpoint, err))?;

        Ok(SessionLowerResponse { expr })
    }

    pub fn eval_request(
        &mut self,
        input: &str,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        self.reject_overlapping_evaluation_request()?;
        self.begin_top_level_interrupt_budget();
        let checkpoint = RequestCheckpoint::new(self);
        let parsed = parse_with_diagnostics(input).map_err(request_syntax_failure)?;
        let expr = self
            .lower_syntax_expr(parsed.syntax())
            .map_err(|err| self.finish_request_failure(checkpoint, err))?;

        self.eval_expr_request_from_checkpoint(expr, checkpoint)
    }

    /// Evaluates an already-built expression through the top-level request protocol.
    ///
    /// The expression is evaluated as-is: no lowering pass runs, so callers are
    /// responsible for supplying resolved symbols.
    pub fn eval_expr_request(
        &mut self,
        expr: Expr,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        self.reject_overlapping_evaluation_request()?;
        self.begin_top_level_interrupt_budget();
        let checkpoint = RequestCheckpoint::new(self);

        self.eval_expr_request_from_checkpoint(expr, checkpoint)
    }

    fn eval_expr_request_from_checkpoint(
        &mut self,
        expr: Expr,
        checkpoint: RequestCheckpoint,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let previous_message_list_checkpoint = self
            .current_message_list_checkpoint
            .replace(checkpoint.message_checkpoint);
        let previous_message_list_dirty =
            std::mem::replace(&mut self.current_message_list_dirty, false);
        let outcome = self.evaluate(expr);
        let finalized =
            self.finalize_top_level_eval_value(outcome, checkpoint.handled_error_checkpoint);
        self.current_message_list_checkpoint = previous_message_list_checkpoint;
        self.current_message_list_dirty = previous_message_list_dirty;
        let (value, termination) = match finalized {
            Ok(finalized) => finalized,
            Err(EvalError::InputRequired { .. }) => {
                self.pending_eval_request = Some(checkpoint.into());
                return Err(self.finish_request_failure(
                    checkpoint,
                    EvalError::InputRequired {
                        request: self
                            .pending_input_request()
                            .cloned()
                            .expect("input suspension should record the pending request"),
                    },
                ));
            }
            Err(err) => return Err(self.finish_request_failure(checkpoint, err)),
        };

        let messages = self.messages.split_off(checkpoint.message_checkpoint);
        let prints = self.prints.split_off(checkpoint.print_checkpoint);
        let trace = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        self.last_message_list = messages.clone();
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);
        self.pending_eval_request = None;

        Ok(SessionEvalResponse {
            value,
            termination,
            messages,
            prints,
            trace,
        })
    }

    pub fn resume_eval_request_with_input(
        &mut self,
        input: impl Into<String>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let input = input.into();
        let checkpoint = self
            .pending_eval_request
            .take()
            .map(RequestCheckpoint::from)
            .ok_or_else(|| SessionRequestFailure {
                error: SessionRequestError::Other {
                    message: "no suspended evaluation is waiting for host input".to_string(),
                },
                messages: Vec::new(),
                prints: Vec::new(),
                trace: Vec::new(),
            })?;

        let previous_message_list_checkpoint = self
            .current_message_list_checkpoint
            .replace(checkpoint.message_checkpoint);
        let previous_message_list_dirty = std::mem::replace(
            &mut self.current_message_list_dirty,
            self.messages.len() > checkpoint.message_checkpoint,
        );
        let outcome = self.resume_suspended_evaluation(input.clone());
        let finalized =
            self.finalize_top_level_eval_value(outcome, checkpoint.handled_error_checkpoint);
        self.current_message_list_checkpoint = previous_message_list_checkpoint;
        self.current_message_list_dirty = previous_message_list_dirty;
        let (value, termination) = match finalized {
            Ok(finalized) => finalized,
            Err(EvalError::InputRequired { .. }) => {
                self.pending_eval_request = Some(checkpoint.into());
                return Err(self.finish_request_failure(
                    checkpoint,
                    EvalError::InputRequired {
                        request: self
                            .pending_input_request()
                            .cloned()
                            .expect("nested input suspension should record the pending request"),
                    },
                ));
            }
            Err(err @ EvalError::Syntax(_)) => {
                return Err(self.finish_completed_request_failure(checkpoint, Some(&input), err));
            }
            Err(err) => return Err(self.finish_request_failure(checkpoint, err)),
        };

        let messages = self.messages.split_off(checkpoint.message_checkpoint);
        let prints = self.prints.split_off(checkpoint.print_checkpoint);
        let trace = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        self.last_message_list = messages.clone();
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);

        Ok(SessionEvalResponse {
            value,
            termination,
            messages,
            prints,
            trace,
        })
    }

    pub fn eval_script_request(
        &mut self,
        source: &str,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        self.reject_overlapping_evaluation_request()?;
        let checkpoint = RequestCheckpoint::new(self);

        match self.eval_script(source) {
            Ok(outcome) => Ok(SessionEvalResponse {
                value: outcome.value,
                termination: outcome.termination,
                messages: outcome.messages,
                prints: outcome.prints,
                trace: self.trace_events[checkpoint.trace_checkpoint..].to_vec(),
            }),
            Err(err) => Err(self.finish_completed_request_failure(checkpoint, Some(source), err)),
        }
    }

    pub fn eval_file_request(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        self.reject_overlapping_evaluation_request()?;
        let checkpoint = RequestCheckpoint::new(self);
        let path = path.as_ref();

        match self.eval_file(path) {
            Ok(outcome) => Ok(SessionEvalResponse {
                value: outcome.value,
                termination: outcome.termination,
                messages: outcome.messages,
                prints: outcome.prints,
                trace: self.trace_events[checkpoint.trace_checkpoint..].to_vec(),
            }),
            Err(err) => {
                let source = match err {
                    EvalError::Syntax(_) => self
                        .file_io()
                        .and_then(|file_io| file_io.read_to_string(path).ok()),
                    _ => None,
                };

                Err(self.finish_completed_request_failure(checkpoint, source.as_deref(), err))
            }
        }
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

    fn finish_request_failure(
        &mut self,
        checkpoint: RequestCheckpoint,
        err: EvalError,
    ) -> SessionRequestFailure {
        self.emit_non_loader_eval_message(checkpoint.handled_error_checkpoint, &err);
        self.emit_top_level_control_flow_messages(&err);

        let messages = self.messages[checkpoint.message_checkpoint..].to_vec();
        let prints = self.prints[checkpoint.print_checkpoint..].to_vec();
        let trace = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        self.last_message_list = messages.clone();
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);

        SessionRequestFailure {
            error: session_request_error_from_eval_error(&err),
            messages,
            prints,
            trace,
        }
    }

    fn finish_completed_request_failure(
        &mut self,
        checkpoint: RequestCheckpoint,
        source: Option<&str>,
        err: EvalError,
    ) -> SessionRequestFailure {
        let messages = self.messages[checkpoint.message_checkpoint..].to_vec();
        let prints = self.prints[checkpoint.print_checkpoint..].to_vec();
        let trace = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        self.last_message_list = messages.clone();
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);

        SessionRequestFailure {
            error: session_request_error_from_eval_error_with_source(&err, source),
            messages,
            prints,
            trace,
        }
    }
}

fn request_syntax_failure(diagnostic: rum_syntax::SourceDiagnostic) -> SessionRequestFailure {
    SessionRequestFailure {
        error: SessionRequestError::Syntax(diagnostic.structured()),
        messages: Vec::new(),
        prints: Vec::new(),
        trace: Vec::new(),
    }
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

fn session_request_error_from_eval_error(err: &EvalError) -> SessionRequestError {
    session_request_error_from_eval_error_with_source(err, None)
}

fn session_request_error_from_eval_error_with_source(
    err: &EvalError,
    source: Option<&str>,
) -> SessionRequestError {
    match err {
        EvalError::Syntax(error) => source
            .map(|source| {
                SessionRequestError::Syntax(
                    rum_syntax::SourceDiagnostic::new(
                        rum_syntax::SourceText::new(source),
                        error.clone(),
                    )
                    .structured(),
                )
            })
            .unwrap_or_else(|| SessionRequestError::Other {
                message: error.to_string(),
            }),
        EvalError::InvalidInteger { literal } => SessionRequestError::InvalidInteger {
            literal: literal.clone(),
        },
        EvalError::Io { path, source } => SessionRequestError::Io {
            path: path.clone(),
            message: source.to_string(),
        },
        EvalError::LoadPathNotFound { target, searched } => SessionRequestError::LoadPathNotFound {
            target: target.clone(),
            searched: searched.clone(),
        },
        EvalError::NoFileIoProvider => SessionRequestError::Other {
            message: "no file provider installed".to_string(),
        },
        EvalError::EvaluationCycleDetected { expr } => {
            SessionRequestError::EvaluationCycleDetected { expr: expr.clone() }
        }
        EvalError::RecursionLimitExceeded { limit } => {
            SessionRequestError::RecursionLimitExceeded { limit: *limit }
        }
        EvalError::StackLimitExceeded { limit } => {
            SessionRequestError::StackLimitExceeded { limit: *limit }
        }
        EvalError::MatchRecursionLimitExceeded { limit } => {
            SessionRequestError::MatchRecursionLimitExceeded { limit: *limit }
        }
        EvalError::Cancelled => SessionRequestError::Cancelled,
        EvalError::TimeConstraintExceeded => SessionRequestError::TimeConstraintExceeded,
        EvalError::MemoryConstraintExceeded { limit } => {
            SessionRequestError::MemoryConstraintExceeded { limit: *limit }
        }
        EvalError::GeneratedElementLimitExceeded { limit } => {
            SessionRequestError::GeneratedElementLimitExceeded { limit: *limit }
        }
        EvalError::IntegerWorkLimitExceeded { limit } => {
            SessionRequestError::IntegerWorkLimitExceeded { limit: *limit }
        }
        EvalError::IterationWorkLimitExceeded { limit } => {
            SessionRequestError::IterationWorkLimitExceeded { limit: *limit }
        }
        EvalError::IterationLimitExceeded { limit } => {
            SessionRequestError::IterationLimitExceeded { limit: *limit }
        }
        EvalError::EvaluationPassLimitExceeded { limit } => {
            SessionRequestError::EvaluationPassLimitExceeded { limit: *limit }
        }
        EvalError::Exit { code } => SessionRequestError::Exit { code: *code },
        EvalError::Return { value } => SessionRequestError::Other {
            message: format!("uncaught top-level Return resolved to {value}"),
        },
        EvalError::Thrown { value, tag, .. } => SessionRequestError::Other {
            message: format!("uncaught throw: {value} with tag {tag}"),
        },
        EvalError::ThrownBypassingCatch {
            value,
            tag,
            cleanup_value,
            cleanup_tag,
            ..
        } => SessionRequestError::Other {
            message: format!(
                "uncaught throw: {cleanup_value} with tag {cleanup_tag}; uncaught throw: {value} with tag {tag}"
            ),
        },
        EvalError::Abort | EvalError::ProtectedAbort => SessionRequestError::Other {
            message: "uncaught Abort[]".to_string(),
        },
        EvalError::Break => SessionRequestError::Other {
            message: "uncaught Break[]".to_string(),
        },
        EvalError::Continue => SessionRequestError::Other {
            message: "uncaught Continue[]".to_string(),
        },
        EvalError::InputRequired { request } => SessionRequestError::InputRequired {
            request: request.clone(),
        },
        EvalError::ExceptionThrown { exception } => SessionRequestError::Other {
            message: format!("uncaught exception: {exception}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_error_mapper_preserves_non_request_surface_limit_codes() {
        let memory = session_request_error_from_eval_error_with_source(
            &EvalError::MemoryConstraintExceeded { limit: 20 },
            Some("MemoryConstrained[Table[x, 100], 20]"),
        );
        assert!(matches!(
            memory,
            SessionRequestError::MemoryConstraintExceeded { limit: 20 }
        ));

        let pass = session_request_error_from_eval_error_with_source(
            &EvalError::EvaluationPassLimitExceeded { limit: 2 },
            Some("loopA"),
        );
        assert!(matches!(
            pass,
            SessionRequestError::EvaluationPassLimitExceeded { limit: 2 }
        ));
    }
}
