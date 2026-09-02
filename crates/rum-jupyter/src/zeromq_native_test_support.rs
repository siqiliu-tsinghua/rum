use std::{
    fmt::Debug,
    future::Future,
    io::ErrorKind,
    net::TcpListener,
    time::Duration,
};

use super::*;

const ALLOW_NATIVE_ZEROMQ_SKIP: &str = "RUM_ALLOW_NATIVE_ZEROMQ_SKIP";
const NATIVE_TEST_HARNESS_STOP_LOSS: Duration = Duration::from_secs(30);
const NATIVE_TEST_POLL_INTERVAL: Duration = Duration::from_millis(10);

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
        let probe_header = KernelMessageHeader::new(
            probe_message_id,
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
        let send_stage = format!("{stage}: connection probe send");
        self.block_on_result(
            runtime,
            &send_stage,
            dealer.send(frames_to_zmq_message(
                wire_message_to_frames(probe).frames,
            )),
        );

        let delivery_stage = format!("{stage}: connection probe request delivery");
        wait_for_socket_event(
            self,
            driver,
            transport,
            crate::KernelSocketPollEvent::Channel(channel),
            &delivery_stage,
        );

        let reply_stage = format!("{stage}: connection probe reply");
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
            Some(probe_header),
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
    heartbeat_endpoint: &str,
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
) {
    let mut heartbeat = harness.connect_req(
        runtime,
        heartbeat_endpoint,
        "IOPub subscription heartbeat connection",
    );
    let barrier = b"iopub-subscription-ready".to_vec();
    harness.heartbeat_round_trip(
        runtime,
        &mut heartbeat,
        driver,
        transport,
        barrier,
        "IOPub subscription heartbeat round trip",
    );
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
}
