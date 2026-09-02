use std::{
    io::ErrorKind,
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};

use super::*;

const ALLOW_NATIVE_ZEROMQ_SKIP: &str = "RUM_ALLOW_NATIVE_ZEROMQ_SKIP";

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
    runtime: &tokio::runtime::Runtime,
    heartbeat_endpoint: &str,
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
) {
    let mut heartbeat = runtime.block_on(async {
        let mut heartbeat = ReqSocket::new();
        heartbeat
            .connect(heartbeat_endpoint)
            .await
            .expect("iopub subscription barrier should connect");
        heartbeat
    });
    let barrier = b"iopub-subscription-ready".to_vec();

    runtime.block_on(async {
        heartbeat
            .send(ZmqMessage::from(barrier.clone()))
            .await
            .expect("iopub subscription barrier should send");
    });
    wait_for_socket_event(
        driver,
        transport,
        crate::KernelSocketPollEvent::Heartbeat,
    );

    let reply = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), heartbeat.recv())
            .await
            .expect("iopub subscription barrier reply should arrive before timeout")
            .expect("iopub subscription barrier reply should decode")
    });
    assert_eq!(zmq_message_to_frames(reply), vec![barrier]);
}

pub(super) fn wait_for_socket_event(
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
    expected: crate::KernelSocketPollEvent,
) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let event = driver
            .pump_once(transport)
            .expect("socket driver should keep pumping");
        if event == Some(expected) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for socket event {expected:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

pub(super) fn wait_for_pending_socket_channels(
    transport: &ZeroMqSocketSession<NativeZeroMqSocket>,
    channels: &[KernelChannel],
) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if channels
            .iter()
            .all(|&channel| transport.has_pending_channel(channel))
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for pending socket channels {channels:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

pub(super) fn wait_for_next_socket_event(
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
) -> crate::KernelSocketPollEvent {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(event) = driver
            .pump_once(transport)
            .expect("socket driver should keep pumping")
        {
            return event;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for the next socket event"
        );
        thread::sleep(Duration::from_millis(10));
    }
}
