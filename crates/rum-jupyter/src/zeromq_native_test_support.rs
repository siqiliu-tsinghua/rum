use std::{
    fmt::Debug,
    future::Future,
    io::ErrorKind,
    net::TcpListener,
};

use futures::FutureExt;

use super::*;
use crate::{
    KernelExecutionState, KernelIoPubPayload, KernelLifecycleState, KernelStatusOutput,
};

const ALLOW_NATIVE_ZEROMQ_SKIP: &str = "RUM_ALLOW_NATIVE_ZEROMQ_SKIP";
const NATIVE_TEST_HARNESS_STOP_LOSS: Duration = Duration::from_secs(30);
const NATIVE_TEST_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParentScopedIoPub {
    Excluded,
    Included,
    TerminalIdle,
}

fn classify_parent_scoped_iopub(
    envelope: &crate::KernelIoPubEnvelope,
    parent: &KernelMessageHeader,
) -> ParentScopedIoPub {
    if envelope.parent_header.as_ref() != Some(parent) {
        return ParentScopedIoPub::Excluded;
    }

    if matches!(
        envelope.payload,
        KernelIoPubPayload::Status(ref status)
            if status.execution_state == KernelExecutionState::Idle
    ) {
        ParentScopedIoPub::TerminalIdle
    } else {
        ParentScopedIoPub::Included
    }
}

fn decode_observed_iopub(
    codec: &crate::KernelWireCodec,
    delimiter: &str,
    message: ZmqMessage,
    stage: &str,
) -> (crate::KernelWireMessage, crate::KernelIoPubEnvelope) {
    let wire = frames_to_wire_message(
        KernelChannel::IoPub,
        delimiter,
        zmq_message_to_frames(message),
    )
    .unwrap_or_else(|error| panic!("native ZeroMQ stage `{stage}` failed: {error:?}"));
    let envelope = codec
        .decode_iopub_message(&wire)
        .unwrap_or_else(|error| panic!("native ZeroMQ stage `{stage}` failed: {error:?}"));
    (wire, envelope)
}

pub(super) struct NativeTestHarness {
    deadline: Instant,
}

impl NativeTestHarness {
    pub(super) fn new() -> Self {
        Self {
            deadline: Instant::now() + NATIVE_TEST_HARNESS_STOP_LOSS,
        }
    }

    pub(super) fn block_on_result<F, T, E>(
        &self,
        runtime: &tokio::runtime::Runtime,
        stage: &str,
        future: F,
    ) -> T
    where
        F: Future<Output = Result<T, E>>,
        E: Debug,
    {
        match runtime
            .block_on(async { tokio::time::timeout(self.remaining(stage), future).await })
        {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => panic!("native ZeroMQ stage `{stage}` failed: {error:?}"),
            Err(_) => self.harness_timeout(stage),
        }
    }

    pub(super) fn connect_dealer(
        &self,
        runtime: &tokio::runtime::Runtime,
        endpoint: &str,
        stage: &str,
    ) -> DealerSocket {
        let mut dealer = DealerSocket::new();
        self.block_on_result(runtime, stage, dealer.connect(endpoint));
        dealer
    }

    pub(super) fn connect_req(
        &self,
        runtime: &tokio::runtime::Runtime,
        endpoint: &str,
        stage: &str,
    ) -> ReqSocket {
        let mut request = ReqSocket::new();
        self.block_on_result(runtime, stage, request.connect(endpoint));
        request
    }

    pub(super) fn establish_dealer_connection(
        &self,
        runtime: &tokio::runtime::Runtime,
        dealer: &mut DealerSocket,
        codec: &crate::KernelWireCodec,
        delimiter: &str,
        driver: &mut KernelSocketDriver,
        transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
        channel: KernelChannel,
        probe_message_id: &str,
        stage: &str,
    ) {
        self.kernel_info_round_trip(
            runtime,
            dealer,
            codec,
            delimiter,
            driver,
            transport,
            channel,
            probe_message_id,
            stage,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn kernel_info_round_trip(
        &self,
        runtime: &tokio::runtime::Runtime,
        dealer: &mut DealerSocket,
        codec: &crate::KernelWireCodec,
        delimiter: &str,
        driver: &mut KernelSocketDriver,
        transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
        channel: KernelChannel,
        message_id: &str,
        stage: &str,
    ) -> KernelMessageHeader {
        let probe_header = KernelMessageHeader::new(
            message_id,
            "frontend-session",
            "frontend",
            "kernel_info_request",
        );
        let probe = codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel,
                header: probe_header.clone(),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::KernelInfo(crate::KernelInfoRequest),
            })
            .unwrap_or_else(|error| panic!("native ZeroMQ stage `{stage}` failed: {error:?}"));
        let send_stage = format!("{stage}: request send");
        self.block_on_result(
            runtime,
            &send_stage,
            dealer.send(frames_to_zmq_message(
                wire_message_to_frames(probe).frames,
            )),
        );

        let delivery_stage = format!("{stage}: request delivery");
        wait_for_socket_event(
            self,
            driver,
            transport,
            crate::KernelSocketPollEvent::Channel(channel),
            &delivery_stage,
        );

        let reply_stage = format!("{stage}: reply");
        let reply = self.block_on_result(runtime, &reply_stage, dealer.recv());
        let reply_wire = frames_to_wire_message(channel, delimiter, zmq_message_to_frames(reply))
            .unwrap_or_else(|error| {
                panic!("native ZeroMQ stage `{reply_stage}` failed: {error:?}")
            });
        let reply = codec
            .decode_response_message(&reply_wire)
            .unwrap_or_else(|error| {
                panic!("native ZeroMQ stage `{reply_stage}` failed: {error:?}")
            });
        assert_eq!(
            reply.parent_header,
            Some(probe_header.clone()),
            "native ZeroMQ stage `{reply_stage}` returned the wrong parent header"
        );
        match reply.response {
            KernelResponse::KernelInfo(response) => assert_eq!(
                response.status,
                KernelReplyStatus::Ok,
                "native ZeroMQ stage `{reply_stage}` returned a failed kernel-info probe"
            ),
            other => panic!(
                "native ZeroMQ stage `{reply_stage}` returned the wrong response: {other:?}"
            ),
        }
        probe_header
    }

    pub(super) fn heartbeat_round_trip(
        &self,
        runtime: &tokio::runtime::Runtime,
        heartbeat: &mut ReqSocket,
        driver: &mut KernelSocketDriver,
        transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
        payload: Vec<u8>,
        stage: &str,
    ) {
        let send_stage = format!("{stage}: request send");
        self.block_on_result(
            runtime,
            &send_stage,
            heartbeat.send(ZmqMessage::from(payload.clone())),
        );
        let delivery_stage = format!("{stage}: request delivery");
        wait_for_socket_event(
            self,
            driver,
            transport,
            crate::KernelSocketPollEvent::Heartbeat,
            &delivery_stage,
        );
        let reply_stage = format!("{stage}: reply");
        let reply = self.block_on_result(runtime, &reply_stage, heartbeat.recv());
        assert_eq!(
            zmq_message_to_frames(reply),
            vec![payload],
            "native ZeroMQ stage `{reply_stage}` returned the wrong heartbeat payload"
        );
    }

    pub(super) fn wait_for_invalid_signature(
        &self,
        driver: &mut KernelSocketDriver,
        transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
        stage: &str,
    ) {
        loop {
            match driver.pump_once(transport) {
                Err(crate::KernelWireError::InvalidSignature) => return,
                Ok(_) => self.pause(stage),
                Err(other) => {
                    panic!("native ZeroMQ stage `{stage}` failed with the wrong error: {other:?}")
                }
            }
        }
    }

    pub(super) fn collect_iopub_for_parent(
        &self,
        runtime: &tokio::runtime::Runtime,
        iopub: &mut SubSocket,
        codec: &crate::KernelWireCodec,
        delimiter: &str,
        parent: &KernelMessageHeader,
        stage: &str,
    ) -> Vec<(crate::KernelWireMessage, crate::KernelIoPubEnvelope)> {
        let receive_stage = format!("{stage}: SUB receive");
        let decode_stage = format!("{stage}: frame decode");
        let mut collected = Vec::new();

        loop {
            let message = self.block_on_result(runtime, &receive_stage, iopub.recv());
            let observed = decode_observed_iopub(codec, delimiter, message, &decode_stage);
            match classify_parent_scoped_iopub(&observed.1, parent) {
                ParentScopedIoPub::Excluded => {}
                ParentScopedIoPub::Included => collected.push(observed),
                ParentScopedIoPub::TerminalIdle => {
                    collected.push(observed);
                    return collected;
                }
            }
        }
    }

    pub(super) fn assert_exact_execute_iopub_sequence(
        &self,
        messages: &[(crate::KernelWireMessage, crate::KernelIoPubEnvelope)],
        stage: &str,
    ) {
        let sequence = messages
            .iter()
            .map(|(_, message)| match &message.payload {
                KernelIoPubPayload::Status(status)
                    if status.lifecycle_state == KernelLifecycleState::Running =>
                {
                    status.execution_state.as_str()
                }
                KernelIoPubPayload::ExecuteInput(_) => "input",
                KernelIoPubPayload::ExecuteResult(_) => "result",
                _ => "other",
            })
            .collect::<Vec<_>>();
        assert_eq!(
            sequence,
            ["busy", "input", "result", "idle"],
            "native ZeroMQ stage `{stage}` returned the wrong target-parent IOPub count or sequence"
        );
    }

    fn remaining(&self, stage: &str) -> Duration {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .unwrap_or_else(|| self.harness_timeout(stage))
    }

    fn pause(&self, stage: &str) {
        thread::park_timeout(self.remaining(stage).min(NATIVE_TEST_POLL_INTERVAL));
    }

    fn harness_timeout(&self, stage: &str) -> ! {
        panic!(
            "native ZeroMQ harness timeout while waiting for stage `{stage}`; this is a harness \
             timeout, not a latency contract"
        )
    }
}

fn loopback_bind_available() -> bool {
    match TcpListener::bind("127.0.0.1:0") {
        Ok(_probe) => true,
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            if std::env::var_os(ALLOW_NATIVE_ZEROMQ_SKIP).is_some() {
                eprintln!(
                    "skipping native ZeroMQ test: loopback socket binding is not permitted; \
                     {ALLOW_NATIVE_ZEROMQ_SKIP} explicitly allows this skip"
                );
                false
            } else {
                panic!(
                    "native ZeroMQ test cannot bind loopback: {error}; set \
                     {ALLOW_NATIVE_ZEROMQ_SKIP}=1 to explicitly allow this test to skip"
                );
            }
        }
        Err(error) => panic!("native ZeroMQ loopback capability probe failed: {error}"),
    }
}

fn loopback_connection_with_signature(
    key: &str,
    signature_scheme: crate::KernelSignatureScheme,
) -> Option<KernelBootstrapConfig> {
    if !loopback_bind_available() {
        return None;
    }

    Some(KernelBootstrapConfig::from_connection_file(KernelConnectionFile {
        transport: crate::KernelTransportKind::Tcp,
        ip: "127.0.0.1".to_string(),
        shell_port: 0,
        iopub_port: 0,
        stdin_port: 0,
        control_port: 0,
        hb_port: 0,
        key: key.to_string(),
        signature_scheme,
        kernel_name: Some("rum".to_string()),
    })
    .with_username("rum-kernel")
    .with_session_validation("frontend-session"))
}

pub(super) fn loopback_connection() -> Option<KernelBootstrapConfig> {
    loopback_connection_with_signature("", crate::KernelSignatureScheme::None)
}

pub(super) fn signed_loopback_connection() -> Option<KernelBootstrapConfig> {
    loopback_connection_with_signature(
        "secret",
        crate::KernelSignatureScheme::HmacSha256,
    )
}

pub(super) fn test_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime should build")
}

pub(super) fn acknowledge_iopub_subscription(
    harness: &NativeTestHarness,
    runtime: &tokio::runtime::Runtime,
    shell: &mut DealerSocket,
    iopub: &mut SubSocket,
    codec: &crate::KernelWireCodec,
    delimiter: &str,
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
    sentinel_id: &str,
) {
    let adapter_state_before = {
        let adapter = driver.event_pump().adapter();
        (
            adapter.execution_count(),
            adapter.execution_state(),
            adapter.lifecycle_state(),
        )
    };
    let mut sentinels = Vec::new();
    let mut attempt = 0_u64;

    loop {
        attempt += 1;
        let stage = format!("IOPub subscription barrier attempt {attempt}: Shell sentinel");
        let parent = harness.kernel_info_round_trip(
            runtime,
            shell,
            codec,
            delimiter,
            driver,
            transport,
            KernelChannel::Shell,
            &format!("{sentinel_id}-{attempt}"),
            &stage,
        );
        sentinels.push(parent.clone());

        let observed = runtime.block_on(async { iopub.recv().now_or_never() });
        let message = match observed {
            Some(Ok(message)) => message,
            Some(Err(zeromq::ZmqError::NoMessage)) | None => continue,
            Some(Err(error)) => panic!(
                "native ZeroMQ IOPub subscription barrier SUB observation failed: {error:?}"
            ),
        };
        let observation_stage =
            format!("IOPub subscription barrier attempt {attempt}: SUB observation");
        let (_, envelope) = decode_observed_iopub(codec, delimiter, message, &observation_stage);
        if !sentinels
            .iter()
            .any(|sentinel| envelope.parent_header.as_ref() == Some(sentinel))
        {
            continue;
        }

        let adapter = driver.event_pump().adapter();
        assert_eq!(
            (
                adapter.execution_count(),
                adapter.execution_state(),
                adapter.lifecycle_state(),
            ),
            adapter_state_before,
            "IOPub subscription barrier kernel-info sentinels must preserve adapter state"
        );
        return;
    }
}

pub(super) fn wait_for_socket_event(
    harness: &NativeTestHarness,
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
    expected: crate::KernelSocketPollEvent,
    stage: &str,
) {
    loop {
        let event = driver
            .pump_once(transport)
            .unwrap_or_else(|error| panic!("native ZeroMQ stage `{stage}` failed: {error:?}"));
        if event == Some(expected) {
            return;
        }
        harness.pause(stage);
    }
}

pub(super) fn wait_for_pending_socket_channels(
    harness: &NativeTestHarness,
    transport: &ZeroMqSocketSession<NativeZeroMqSocket>,
    channels: &[KernelChannel],
    stage: &str,
) {
    loop {
        if channels
            .iter()
            .all(|&channel| transport.has_pending_channel(channel))
        {
            return;
        }
        harness.pause(stage);
    }
}

pub(super) fn wait_for_next_socket_event(
    harness: &NativeTestHarness,
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
    stage: &str,
) -> crate::KernelSocketPollEvent {
    loop {
        if let Some(event) = driver
            .pump_once(transport)
            .unwrap_or_else(|error| panic!("native ZeroMQ stage `{stage}` failed: {error:?}"))
        {
            return event;
        }
        harness.pause(stage);
    }
}

#[cfg(test)]
mod harness_tests {
    use std::{
        any::Any,
        panic::{AssertUnwindSafe, catch_unwind},
    };

    use super::*;

    fn harness_with_stop_loss(stop_loss: Duration) -> NativeTestHarness {
        NativeTestHarness {
            deadline: Instant::now() + stop_loss,
        }
    }

    fn panic_message(payload: Box<dyn Any + Send>) -> String {
        if let Some(message) = payload.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = payload.downcast_ref::<&str>() {
            (*message).to_string()
        } else {
            "non-string panic payload".to_string()
        }
    }

    #[test]
    fn block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets() {
        let runtime = test_runtime();
        let harness = harness_with_stop_loss(Duration::from_secs(1));

        let value = harness.block_on_result(&runtime, "success outcome", async {
            Ok::<_, &'static str>(42)
        });
        assert_eq!(value, 42);

        let inner_error = catch_unwind(AssertUnwindSafe(|| {
            harness.block_on_result(&runtime, "inner error outcome", async {
                Err::<(), _>("inner failure")
            });
        }))
        .expect_err("an inner error should fail the named harness stage");
        assert_eq!(
            panic_message(inner_error),
            "native ZeroMQ stage `inner error outcome` failed: \"inner failure\""
        );

        let timeout_harness = harness_with_stop_loss(Duration::from_millis(100));
        let timeout = catch_unwind(AssertUnwindSafe(|| {
            timeout_harness.block_on_result(
                &runtime,
                "timeout outcome",
                std::future::pending::<Result<(), &'static str>>(),
            );
        }))
        .expect_err("a permanently pending future should hit the harness stop-loss");
        assert_eq!(
            panic_message(timeout),
            "native ZeroMQ harness timeout while waiting for stage `timeout outcome`; this is a \
             harness timeout, not a latency contract"
        );
    }

    fn status_iopub(
        parent: &KernelMessageHeader,
        execution_state: KernelExecutionState,
    ) -> crate::KernelIoPubEnvelope {
        crate::KernelIoPubEnvelope {
            channel: KernelChannel::IoPub,
            header: KernelMessageHeader::new("status", "kernel-session", "rum-kernel", "status"),
            parent_header: Some(parent.clone()),
            metadata: KernelMetadata::new(),
            payload: KernelIoPubPayload::Status(KernelStatusOutput {
                execution_state,
                lifecycle_state: KernelLifecycleState::Running,
            }),
        }
    }

    #[test]
    fn parent_scope_excludes_sentinels_including_late_duplicates_and_stops_at_target_idle() {
        let sentinel = KernelMessageHeader::new(
            "iopub-barrier-sentinel-1",
            "frontend-session",
            "frontend",
            "kernel_info_request",
        );
        let target = KernelMessageHeader::new(
            "execute-target-1",
            "frontend-session",
            "frontend",
            "execute_request",
        );
        let classify = |parent, state| {
            classify_parent_scoped_iopub(&status_iopub(parent, state), &target)
        };

        assert_eq!(
            [
                classify(&sentinel, KernelExecutionState::Busy),
                classify(&target, KernelExecutionState::Busy),
                classify(&sentinel, KernelExecutionState::Idle),
                classify(&target, KernelExecutionState::Idle),
            ],
            [
                ParentScopedIoPub::Excluded,
                ParentScopedIoPub::Included,
                ParentScopedIoPub::Excluded,
                ParentScopedIoPub::TerminalIdle,
            ]
        );
    }
}
