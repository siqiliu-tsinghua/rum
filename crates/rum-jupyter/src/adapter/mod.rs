use rum_host::{
    HostCompletenessStatus, HostCompletionRequest, HostEvalRequest, HostInspectRequest,
    HostIsCompleteRequest, HostSession,
};

use crate::{
    KernelChannel, KernelCommInfoRequest, KernelCommInfoResponse, KernelCompleteRequest,
    KernelCompleteResponse, KernelDebugRequest, KernelDebugResponse, KernelDispatchResult,
    KernelDisplayOutput, KernelErrorOutput, KernelExecuteInput, KernelExecuteOutcome,
    KernelExecutePayload, KernelExecuteRequest, KernelExecuteResponse, KernelExecuteResult,
    KernelExecutionState, KernelHelpLink, KernelHistoryRequest, KernelHistoryResponse,
    KernelIgnoredRequest, KernelInfoRequest, KernelInfoResponse, KernelInputReply,
    KernelInputReplyEnvelope, KernelInputRequest, KernelInputRequestEnvelope, KernelInspectRequest,
    KernelInspectResponse, KernelInterruptRequest, KernelInterruptResponse, KernelIoPubEnvelope,
    KernelIoPubPayload, KernelIsCompleteRequest, KernelIsCompleteResponse, KernelIsCompleteStatus,
    KernelLanguageInfo, KernelLifecycleState, KernelMessageHeader, KernelMetadata,
    KernelMimeBundle, KernelOutput, KernelReplyStatus, KernelRequest, KernelRequestEnvelope,
    KernelResponse, KernelResponseEnvelope, KernelShutdownRequest, KernelShutdownResponse,
    KernelStatusOutput, KernelStdinError, KernelStreamName, KernelStreamOutput,
};

mod input_retirement;
mod iopub;
mod render;
mod worker;

use self::render::{
    build_execute_response, build_suspended_outputs, complete_metadata, input_request_from_outcome,
    input_request_metadata, inspect_metadata, inspect_mime_bundle, kernel_input_request,
    response_metadata, response_msg_type,
};
#[cfg(test)]
pub(crate) use self::worker::ExecuteWorkerGate;
use self::worker::{
    ExecuteContext, ExecuteState, force_cancelled_success, host_outcome_to_kernel,
    outcome_is_cancelled, outcome_requires_session_reset,
};
#[cfg(test)]
use self::worker::{ExecuteWorkerGateWait, execute_worker_gate};

pub struct KernelAdapter {
    session: Option<HostSession>,
    execute_state: ExecuteState,
    kernel_username: String,
    execution_count: u64,
    next_message_id: u64,
    execution_state: KernelExecutionState,
    lifecycle_state: KernelLifecycleState,
    pending_input: Option<KernelInputRequestEnvelope>,
    pending_execute: Option<ExecuteContext>,
    last_input_reply: Option<KernelInputReply>,
    #[cfg(test)]
    next_execute_worker_gate: Option<ExecuteWorkerGateWait>,
}

impl KernelAdapter {
    pub fn new(session: HostSession) -> Self {
        Self::with_username(session, "kernel")
    }

    pub fn with_username(session: HostSession, username: impl Into<String>) -> Self {
        Self {
            session: Some(session),
            execute_state: ExecuteState::idle(),
            kernel_username: username.into(),
            execution_count: 0,
            next_message_id: 0,
            execution_state: KernelExecutionState::Idle,
            lifecycle_state: KernelLifecycleState::Running,
            pending_input: None,
            pending_execute: None,
            last_input_reply: None,
            #[cfg(test)]
            next_execute_worker_gate: None,
        }
    }

    pub fn execution_count(&self) -> u64 {
        self.execution_count
    }

    pub fn execution_state(&self) -> KernelExecutionState {
        self.execution_state
    }

    pub fn lifecycle_state(&self) -> KernelLifecycleState {
        self.lifecycle_state
    }

    pub fn interrupt_requested(&self) -> bool {
        self.lifecycle_state == KernelLifecycleState::InterruptRequested
    }

    pub fn shutdown_requested(&self) -> bool {
        matches!(
            self.lifecycle_state,
            KernelLifecycleState::ShutdownRequested | KernelLifecycleState::Shutdown
        )
    }

    pub fn awaiting_stdin(&self) -> bool {
        self.pending_input.is_some()
    }

    pub fn execute_running(&self) -> bool {
        self.execute_state.is_running()
    }

    #[cfg(test)]
    pub(crate) fn gate_next_execute_worker(&mut self) -> ExecuteWorkerGate {
        assert!(
            self.next_execute_worker_gate.is_none(),
            "only one execute worker gate may be pending"
        );
        let (gate, wait) = execute_worker_gate();
        self.next_execute_worker_gate = Some(wait);
        gate
    }

    #[cfg(test)]
    pub(crate) fn wait_for_execute_worker_started(&mut self) {
        self.execute_state.wait_until_started();
    }

    #[cfg(test)]
    pub(crate) fn wait_for_execute_worker_finished(&mut self) {
        self.execute_state.wait_until_finished();
    }

    pub fn pending_input_request(&self) -> Option<&KernelInputRequestEnvelope> {
        self.pending_input.as_ref()
    }

    pub fn take_last_input_reply(&mut self) -> Option<KernelInputReply> {
        self.last_input_reply.take()
    }

    /// Evaluates immediately and returns a terminal response.
    ///
    /// This API cannot supply input, even when `request.allow_stdin` is true.
    /// An admitted input request is cancelled and retired, running guest cleanup,
    /// before returning the original `InputRequired` error. Its outputs include
    /// the prefix and cleanup once; the abandoned remainder is not evaluated.
    /// A request refused at admission leaves the existing request untouched.
    /// Use [`Self::dispatch_request`] or [`crate::KernelEventPump`] for notebook
    /// transport with asynchronous stdin and completion handling.
    pub fn execute(&mut self, request: KernelExecuteRequest) -> KernelExecuteResponse {
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

    pub fn kernel_info(&self, _request: KernelInfoRequest) -> KernelInfoResponse {
        KernelInfoResponse {
            status: KernelReplyStatus::Ok,
            protocol_version: "5.3".to_string(),
            implementation: "rum".to_string(),
            implementation_version: env!("CARGO_PKG_VERSION").to_string(),
            banner: "rum experimental kernel adapter".to_string(),
            language_info: KernelLanguageInfo {
                name: "rum".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                mimetype: "text/x-rum".to_string(),
                file_extension: ".rum".to_string(),
                pygments_lexer: "text".to_string(),
                codemirror_mode: "text".to_string(),
            },
            help_links: vec![KernelHelpLink {
                text: "rum repository".to_string(),
                url: "https://github.com/siqiliu-tsinghua/rum".to_string(),
            }],
        }
    }

    pub fn shutdown(&mut self, request: KernelShutdownRequest) -> KernelShutdownResponse {
        self.lifecycle_state = KernelLifecycleState::ShutdownRequested;
        KernelShutdownResponse {
            status: KernelReplyStatus::Ok,
            restart: request.restart,
            lifecycle_state: self.lifecycle_state,
        }
    }

    pub fn interrupt(&mut self, _request: KernelInterruptRequest) -> KernelInterruptResponse {
        self.lifecycle_state = KernelLifecycleState::InterruptRequested;
        self.execute_state.cancel();
        KernelInterruptResponse {
            status: KernelReplyStatus::Ok,
            interrupt_requested: true,
            lifecycle_state: self.lifecycle_state,
        }
    }

    pub fn inspect(&mut self, request: KernelInspectRequest) -> KernelInspectResponse {
        let result = self
            .session_mut()
            .inspect(HostInspectRequest::new(request.code, request.cursor_offset));
        let target = result.target.clone();
        let found = result.found;
        let data = if found {
            inspect_mime_bundle(&result)
        } else {
            KernelMimeBundle::new()
        };
        let metadata = inspect_metadata(target.as_deref(), found);

        KernelInspectResponse {
            status: KernelReplyStatus::Ok,
            target,
            found,
            data,
            metadata,
            result: found.then_some(result),
        }
    }

    pub fn complete(&mut self, request: KernelCompleteRequest) -> KernelCompleteResponse {
        let completion = self.session_mut().complete(HostCompletionRequest::new(
            request.code,
            request.cursor_offset,
        ));
        let prefix = completion.prefix;
        let replacement_start = completion.replacement_start;
        let replacement_end = completion.replacement_end;
        let matches = completion.matches;

        KernelCompleteResponse {
            status: KernelReplyStatus::Ok,
            replacement_start,
            replacement_end,
            matches,
            metadata: complete_metadata(&prefix, replacement_start, replacement_end),
            prefix,
        }
    }

    pub fn is_complete(&mut self, request: KernelIsCompleteRequest) -> KernelIsCompleteResponse {
        let response = self
            .session_mut()
            .is_complete(HostIsCompleteRequest::new(request.code));

        KernelIsCompleteResponse {
            reply_status: KernelReplyStatus::Ok,
            completion_status: match response.status {
                HostCompletenessStatus::Complete => KernelIsCompleteStatus::Complete,
                HostCompletenessStatus::Incomplete => KernelIsCompleteStatus::Incomplete,
                HostCompletenessStatus::Invalid => KernelIsCompleteStatus::Invalid,
            },
            indent: response.indent,
        }
    }

    pub fn history(&self, _request: KernelHistoryRequest) -> KernelHistoryResponse {
        KernelHistoryResponse {
            status: KernelReplyStatus::Ok,
            history: Vec::new(),
        }
    }

    pub fn comm_info(&self, _request: KernelCommInfoRequest) -> KernelCommInfoResponse {
        KernelCommInfoResponse {
            status: KernelReplyStatus::Ok,
        }
    }

    pub fn debug(&self, request: KernelDebugRequest) -> KernelDebugResponse {
        KernelDebugResponse {
            status: KernelReplyStatus::Error,
            request_seq: request.request_seq,
            command: request.command,
            message: "rum does not implement the Jupyter debug protocol".to_string(),
        }
    }

    pub fn request_stdin_for_execute(
        &mut self,
        parent_header: &KernelMessageHeader,
        request: &KernelExecuteRequest,
        input: KernelInputRequest,
    ) -> Result<KernelInputRequestEnvelope, KernelStdinError> {
        if !request.allow_stdin {
            return Err(KernelStdinError::StdinNotAllowed);
        }

        if self.pending_input.is_some() {
            return Err(KernelStdinError::AlreadyAwaitingInput);
        }

        let metadata = input_request_metadata(parent_header, request, &input);
        let envelope = KernelInputRequestEnvelope {
            channel: KernelChannel::Stdin,
            header: self.next_response_header(parent_header, "input_request"),
            parent_header: Some(parent_header.clone()),
            metadata,
            request: input,
        };

        self.pending_input = Some(envelope.clone());

        Ok(envelope)
    }

    pub fn handle_input_reply(
        &mut self,
        envelope: KernelInputReplyEnvelope,
    ) -> Result<Option<KernelDispatchResult>, KernelStdinError> {
        let pending = self
            .pending_input
            .as_ref()
            .ok_or(KernelStdinError::NoPendingInputRequest)?;
        if let Some(parent_header) = envelope.parent_header.as_ref() {
            let found = parent_header.msg_id.clone();
            if found != pending.header.msg_id {
                return Err(KernelStdinError::ReplyParentMismatch {
                    expected: pending.header.msg_id.clone(),
                    found,
                });
            }
        }

        self.last_input_reply = Some(envelope.reply);
        self.pending_input = None;

        let Some(pending_execute) = self.pending_execute.take() else {
            return Ok(None);
        };

        Ok(Some(self.resume_pending_execute(pending_execute)))
    }

    pub fn poll_execute_completion(&mut self) -> Option<KernelDispatchResult> {
        let worker = self.execute_state.take_finished()?;
        let (context, mut session, mut outcome, interrupted) = worker.join();
        if interrupted {
            outcome = force_cancelled_success(outcome);
        }

        if outcome_requires_session_reset(&outcome) {
            session.reset();
        }
        if outcome_is_cancelled(&outcome)
            && self.lifecycle_state == KernelLifecycleState::InterruptRequested
        {
            self.lifecycle_state = KernelLifecycleState::Running;
        }
        self.session = Some(session);

        Some(self.finish_execute_outcome(context, outcome))
    }

    pub fn dispatch_request(&mut self, envelope: KernelRequestEnvelope) -> KernelDispatchResult {
        let parent_header = envelope.header.clone();
        match envelope.request {
            KernelRequest::Execute(request) => {
                self.dispatch_execute_request(parent_header, request)
            }
            KernelRequest::Inspect(request) => {
                let response = KernelResponse::Inspect(self.inspect(request));
                self.finish_non_execute_request(envelope.channel, parent_header, response)
            }
            KernelRequest::Complete(request) => {
                let response = KernelResponse::Complete(self.complete(request));
                self.finish_non_execute_request(envelope.channel, parent_header, response)
            }
            KernelRequest::IsComplete(request) => {
                let response = KernelResponse::IsComplete(self.is_complete(request));
                self.finish_non_execute_request(envelope.channel, parent_header, response)
            }
            KernelRequest::KernelInfo(request) => self.finish_non_execute_request(
                envelope.channel,
                parent_header,
                KernelResponse::KernelInfo(self.kernel_info(request)),
            ),
            KernelRequest::History(request) => self.finish_non_execute_request(
                envelope.channel,
                parent_header,
                KernelResponse::History(self.history(request)),
            ),
            KernelRequest::CommInfo(request) => self.finish_non_execute_request(
                envelope.channel,
                parent_header,
                KernelResponse::CommInfo(self.comm_info(request)),
            ),
            KernelRequest::Debug(request) => self.finish_non_execute_request(
                envelope.channel,
                parent_header,
                KernelResponse::Debug(self.debug(request)),
            ),
            KernelRequest::Ignored(request) => self.ignore_request(request),
            KernelRequest::Interrupt(request) => {
                let response = KernelResponse::Interrupt(self.interrupt(request));
                self.finish_non_execute_request(envelope.channel, parent_header, response)
            }
            KernelRequest::Shutdown(request) => {
                let response = KernelResponse::Shutdown(self.shutdown(request));
                self.finish_non_execute_request(envelope.channel, parent_header, response)
            }
        }
    }

    fn ignore_request(&mut self, _request: KernelIgnoredRequest) -> KernelDispatchResult {
        KernelDispatchResult {
            reply: None,
            iopub: Vec::new(),
            stdin: Vec::new(),
        }
    }

    /// Returns only the reply portion of [`Self::dispatch_request`].
    ///
    /// For `Execute`, this starts asynchronous work and returns `None` while
    /// pending. The caller must poll [`Self::poll_execute_completion`], service
    /// stdin through [`Self::handle_input_reply`], and deliver completion output.
    /// This wrapper drops the initial IOPub prologue (busy and execute input).
    /// Prefer full dispatch or [`crate::KernelEventPump`] for notebook transport.
    pub fn handle_shell_request(
        &mut self,
        envelope: KernelRequestEnvelope,
    ) -> Option<KernelResponseEnvelope> {
        self.dispatch_request(envelope).reply
    }

    fn dispatch_execute_request(
        &mut self,
        parent_header: KernelMessageHeader,
        request: KernelExecuteRequest,
    ) -> KernelDispatchResult {
        let execution_count = self.next_execution_count(request.store_history);
        self.execution_state = KernelExecutionState::Busy;
        let iopub = self.execute_prologue_iopub(&parent_header, execution_count, &request.code);
        let context = ExecuteContext {
            parent_header,
            request,
            execution_count,
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

    fn resume_pending_execute(&mut self, pending: ExecuteContext) -> KernelDispatchResult {
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

    fn finish_non_execute_request(
        &mut self,
        channel: KernelChannel,
        parent_header: KernelMessageHeader,
        response: KernelResponse,
    ) -> KernelDispatchResult {
        if self.execute_running() {
            return KernelDispatchResult {
                reply: Some(self.response_envelope(channel, &parent_header, response)),
                iopub: Vec::new(),
                stdin: Vec::new(),
            };
        }

        self.execution_state = KernelExecutionState::Busy;
        let mut iopub = vec![self.status_iopub(&parent_header, KernelExecutionState::Busy)];
        self.execution_state = KernelExecutionState::Idle;

        KernelDispatchResult {
            reply: Some(self.response_envelope(channel, &parent_header, response)),
            iopub: {
                iopub.push(self.status_iopub(&parent_header, KernelExecutionState::Idle));
                iopub
            },
            stdin: Vec::new(),
        }
    }

    fn response_envelope(
        &mut self,
        channel: KernelChannel,
        parent_header: &KernelMessageHeader,
        response: KernelResponse,
    ) -> KernelResponseEnvelope {
        let response_metadata = response_metadata(
            &response,
            channel,
            parent_header,
            self.execution_state,
            self.lifecycle_state,
        );
        let response_header =
            self.next_response_header(parent_header, response_msg_type(&response));

        let reply = KernelResponseEnvelope {
            channel,
            header: response_header,
            parent_header: Some(parent_header.clone()),
            metadata: response_metadata,
            response,
        };

        if matches!(reply.response, KernelResponse::Shutdown(_)) {
            self.lifecycle_state = KernelLifecycleState::Shutdown;
        }

        reply
    }

    fn next_response_header(
        &mut self,
        parent_header: &KernelMessageHeader,
        msg_type: &str,
    ) -> KernelMessageHeader {
        self.next_message_id += 1;
        let mut header = KernelMessageHeader::new(
            format!("rum-msg-{}", self.next_message_id),
            &parent_header.session_id,
            &self.kernel_username,
            msg_type,
        );
        header.protocol_version = parent_header.protocol_version.clone();
        header
    }

    fn session_mut(&mut self) -> &mut HostSession {
        self.session
            .as_mut()
            .expect("host session is only borrowed on the adapter while execute is idle")
    }

    fn take_idle_session(&mut self) -> HostSession {
        self.session
            .take()
            .expect("host session moves to the execute worker only while running")
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
