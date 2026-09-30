//! Admission, suspension and terminal completion of the sole execute owner.
use super::*;

impl KernelAdapter {
    /// Evaluates immediately and returns a terminal response.
    ///
    /// This API cannot supply input, even when `request.allow_stdin` is true.
    /// An admitted input request is cancelled and retired, running guest cleanup,
    /// before returning the original `InputRequired` error. Its outputs include
    /// the prefix and cleanup once; the abandoned remainder is not evaluated.
    /// A request refused at admission leaves the existing request and execution
    /// count untouched, including while an asynchronous worker is running.
    /// Use [`Self::dispatch_request`] or [`crate::KernelEventPump`] for notebook
    /// transport with asynchronous stdin and completion handling.
    pub fn execute(&mut self, request: KernelExecuteRequest) -> KernelExecuteResponse {
        if self.has_pending_execute() {
            return overlapping_execute_response();
        }
        let execution_count = self.next_execution_count(request.store_history);
        let outcome = run_execute_request(self.session_mut(), &request);
        build_execute_response(execution_count, outcome)
    }

    fn next_execution_count(&mut self, store_history: bool) -> Option<u64> {
        if store_history {
            self.execution_count += 1;
            Some(self.execution_count)
        } else {
            None
        }
    }

    pub(super) fn has_pending_execute(&self) -> bool {
        self.execute_state.is_running() || self.pending_execute.is_some()
    }

    pub(super) fn start_input_retirement(&mut self, context: ExecuteContext) {
        let session = self.take_idle_session();
        self.execute_state.start_retirement(
            session,
            context,
            #[cfg(test)]
            self.next_execute_worker_gate.take(),
        );
    }

    /// Polls one worker segment. Input waits retain the admitted execute;
    /// only its terminal completion releases the context and publishes idle.
    pub fn poll_execute_completion(&mut self) -> Option<KernelDispatchResult> {
        self.poll_execute_completion_with_routing()
            .map(|(dispatch, _)| dispatch)
    }

    pub(crate) fn poll_execute_completion_with_routing(
        &mut self,
    ) -> Option<(KernelDispatchResult, Vec<Vec<u8>>)> {
        let worker = self.execute_state.take_finished()?;
        let (context, mut session, mut outcome) = worker.join();
        let routing_ids = context.routing_ids.clone();
        // Stdin-denied InputRequired is already terminal after worker retirement.
        if context.interrupted
            && context.request.allow_stdin
            && matches!(outcome, KernelExecuteOutcome::InputRequired(_))
        {
            self.session = Some(session);
            // The worker may have suspended just before interrupt arrived.
            // Do not publish an input request for already cancelled work.
            self.start_input_retirement(context);
            return Some((
                KernelDispatchResult {
                    reply: None,
                    iopub: Vec::new(),
                    stdin: Vec::new(),
                },
                routing_ids,
            ));
        }
        if context.interrupted {
            outcome = force_cancelled_success(outcome);
        }

        if outcome_requires_session_reset(&outcome) {
            session.reset();
        }
        if (context.interrupted || outcome_is_cancelled(&outcome))
            && self.lifecycle_state == KernelLifecycleState::InterruptRequested
        {
            self.lifecycle_state = KernelLifecycleState::Running;
        }
        self.session = Some(session);

        Some((self.finish_execute_outcome(context, outcome), routing_ids))
    }

    pub(super) fn dispatch_execute_request(
        &mut self,
        parent_header: KernelMessageHeader,
        request: KernelExecuteRequest,
        routing_ids: Vec<Vec<u8>>,
    ) -> KernelDispatchResult {
        if self.has_pending_execute() {
            let response = overlapping_execute_response();
            let iopub = response
                .outputs
                .iter()
                .map(|output| self.output_iopub(&parent_header, output))
                .collect();
            return KernelDispatchResult {
                reply: Some(self.response_envelope(
                    KernelChannel::Shell,
                    &parent_header,
                    KernelResponse::Execute(response),
                )),
                iopub,
                stdin: Vec::new(),
            };
        }
        let cancel = self.session_mut().cancel_handle();
        let execution_count = self.next_execution_count(request.store_history);
        self.execution_state = KernelExecutionState::Busy;
        let iopub = self.execute_prologue_iopub(&parent_header, execution_count, &request.code);
        let context = ExecuteContext {
            parent_header,
            request,
            execution_count,
            cancel,
            routing_ids,
            interrupted: false,
        };

        let session = self.take_idle_session();
        self.execute_state.start_eval(
            session,
            context,
            #[cfg(test)]
            self.next_execute_worker_gate.take(),
        );

        KernelDispatchResult {
            reply: None,
            iopub,
            stdin: Vec::new(),
        }
    }

    fn finish_execute_outcome(
        &mut self,
        context: ExecuteContext,
        outcome: KernelExecuteOutcome,
    ) -> KernelDispatchResult {
        if let Some(input) = input_request_from_outcome(&outcome) {
            if context.request.allow_stdin {
                let mut iopub = Vec::new();
                iopub.extend(
                    build_suspended_outputs(&outcome)
                        .iter()
                        .map(|output| self.output_iopub(&context.parent_header, output)),
                );
                let stdin = self
                    .request_stdin_for_execute(
                        &context.parent_header,
                        &context.request,
                        kernel_input_request(&input),
                    )
                    .expect("execute request should be able to stage stdin exactly once");
                self.pending_execute = Some(context);

                return KernelDispatchResult {
                    reply: None,
                    iopub,
                    stdin: vec![stdin],
                };
            }
        }

        self.finish_execute_request(
            &context.parent_header,
            &context.request.code,
            context.execution_count,
            outcome,
            false,
        )
    }

    pub(super) fn resume_pending_execute(
        &mut self,
        pending: ExecuteContext,
    ) -> KernelDispatchResult {
        let input = self
            .last_input_reply
            .as_ref()
            .map(|reply| reply.value.clone())
            .expect("pending execute resumes only after an input reply");
        self.execution_state = KernelExecutionState::Busy;
        let session = self.take_idle_session();
        self.execute_state.start_resume(
            session,
            pending,
            input,
            #[cfg(test)]
            self.next_execute_worker_gate.take(),
        );

        KernelDispatchResult {
            reply: None,
            iopub: Vec::new(),
            stdin: Vec::new(),
        }
    }

    fn finish_execute_request(
        &mut self,
        parent_header: &KernelMessageHeader,
        code: &str,
        execution_count: Option<u64>,
        outcome: KernelExecuteOutcome,
        emit_prologue: bool,
    ) -> KernelDispatchResult {
        let response = build_execute_response(execution_count, outcome);
        let mut iopub = if emit_prologue {
            self.execute_prologue_iopub(parent_header, execution_count, code)
        } else {
            Vec::new()
        };

        iopub.extend(
            response
                .outputs
                .iter()
                .map(|output| self.output_iopub(parent_header, output)),
        );

        self.execution_state = KernelExecutionState::Idle;
        iopub.push(self.status_iopub(parent_header, KernelExecutionState::Idle));

        let reply = self.response_envelope(
            KernelChannel::Shell,
            parent_header,
            KernelResponse::Execute(response),
        );

        KernelDispatchResult {
            reply: Some(reply),
            iopub,
            stdin: Vec::new(),
        }
    }
}

fn run_execute_request(
    session: &mut HostSession,
    request: &KernelExecuteRequest,
) -> KernelExecuteOutcome {
    let cancel = session.cancel_handle();
    let outcome = session.eval(HostEvalRequest::new(request.code.clone()));
    host_outcome_to_kernel(match outcome {
        rum_host::HostEvalOutcome::InputRequired(suspension) => {
            input_retirement::retire_input_request(session, &cancel, suspension)
        }
        other => other,
    })
}

fn overlapping_execute_response() -> KernelExecuteResponse {
    build_execute_response(
        None,
        KernelExecuteOutcome::Failure(rum_host::HostFailure {
            code: rum_host::HostFailureCode::Other,
            exit_status: None,
            message: "cannot start a new evaluation while an execute request is pending"
                .to_string(),
            syntax_rendered: None,
            messages: Vec::new(),
            prints: Vec::new(),
            trace: Vec::new(),
            outputs: Vec::new(),
        }),
    )
}
