use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::KernelWireError;

use super::*;

#[derive(Debug, Default)]
struct FakeZeroMqSocketState {
    bound: Vec<String>,
    connected: Vec<String>,
    incoming: VecDeque<Vec<Vec<u8>>>,
    outgoing: Vec<Vec<Vec<u8>>>,
}

#[derive(Clone, Debug, Default)]
struct FakeZeroMqSocketHandle(Arc<Mutex<FakeZeroMqSocketState>>);

impl FakeZeroMqSocketHandle {
    fn push_incoming(&self, frames: Vec<Vec<u8>>) {
        self.0
            .lock()
            .expect("fake zeromq socket lock should not be poisoned")
            .incoming
            .push_back(frames);
    }

    fn take_outgoing(&self) -> Vec<Vec<Vec<u8>>> {
        let mut state = self
            .0
            .lock()
            .expect("fake zeromq socket lock should not be poisoned");
        std::mem::take(&mut state.outgoing)
    }

    fn bound_endpoints(&self) -> Vec<String> {
        self.0
            .lock()
            .expect("fake zeromq socket lock should not be poisoned")
            .bound
            .clone()
    }
}

fn wait_for_fake_socket_event<S: ZeroMqSocket>(
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<S>,
    expected: crate::KernelSocketPollEvent,
) {
    driver
        .event_pump_mut()
        .adapter_mut()
        .wait_for_execute_worker_finished();
    assert_eq!(
        driver
            .pump_once(transport)
            .expect("finished fake socket execute should pump"),
        Some(expected)
    );
}

struct FakeZeroMqSocket {
    state: FakeZeroMqSocketHandle,
}

impl ZeroMqSocket for FakeZeroMqSocket {
    fn bind(&mut self, endpoint: &str) -> Result<(), String> {
        self.state
            .0
            .lock()
            .expect("fake zeromq socket lock should not be poisoned")
            .bound
            .push(endpoint.to_string());
        Ok(())
    }

    fn connect(&mut self, endpoint: &str) -> Result<(), String> {
        self.state
            .0
            .lock()
            .expect("fake zeromq socket lock should not be poisoned")
            .connected
            .push(endpoint.to_string());
        Ok(())
    }

    fn send_multipart(&mut self, frames: Vec<Vec<u8>>) {
        self.state
            .0
            .lock()
            .expect("fake zeromq socket lock should not be poisoned")
            .outgoing
            .push(frames);
    }

    fn try_recv_multipart(&mut self) -> Option<Vec<Vec<u8>>> {
        self.state
            .0
            .lock()
            .expect("fake zeromq socket lock should not be poisoned")
            .incoming
            .pop_front()
    }
}

#[derive(Default)]
struct FakeZeroMqFactory {
    router_sockets: Vec<FakeZeroMqSocketHandle>,
    pub_sockets: Vec<FakeZeroMqSocketHandle>,
    rep_sockets: Vec<FakeZeroMqSocketHandle>,
}

impl ZeroMqSocketFactory for FakeZeroMqFactory {
    type Socket = FakeZeroMqSocket;

    fn router_socket(&mut self) -> Result<Self::Socket, String> {
        let handle = FakeZeroMqSocketHandle::default();
        self.router_sockets.push(handle.clone());
        Ok(FakeZeroMqSocket { state: handle })
    }

    fn pub_socket(&mut self) -> Result<Self::Socket, String> {
        let handle = FakeZeroMqSocketHandle::default();
        self.pub_sockets.push(handle.clone());
        Ok(FakeZeroMqSocket { state: handle })
    }

    fn rep_socket(&mut self) -> Result<Self::Socket, String> {
        let handle = FakeZeroMqSocketHandle::default();
        self.rep_sockets.push(handle.clone());
        Ok(FakeZeroMqSocket { state: handle })
    }
}

fn test_connection() -> KernelBootstrapConfig {
    KernelBootstrapConfig::from_connection_file(
        KernelConnectionFile::parse_json(
            r#"{
                "transport": "tcp",
                "ip": "127.0.0.1",
                "shell_port": 57503,
                "iopub_port": 40885,
                "stdin_port": 52597,
                "control_port": 50160,
                "hb_port": 42540,
                "key": "",
                "signature_scheme": "",
                "kernel_name": "rum"
            }"#,
        )
        .expect("connection file should parse"),
    )
    .with_username("rum-kernel")
    .with_session_validation("frontend-session")
}

#[test]
fn zeromq_backend_binds_expected_socket_endpoints() {
    let bootstrap = test_connection();
    let mut factory = FakeZeroMqFactory::default();
    let backend = ZeroMqTransportBackend::from_factory(bootstrap.clone(), &mut factory)
        .expect("backend should build");

    assert_eq!(
        backend.socket_session_config().shell.kind.as_str(),
        "router"
    );
    assert_eq!(
        factory.router_sockets[0].bound_endpoints(),
        vec!["tcp://127.0.0.1:57503"]
    );
    assert_eq!(
        factory.router_sockets[1].bound_endpoints(),
        vec!["tcp://127.0.0.1:50160"]
    );
    assert_eq!(
        factory.router_sockets[2].bound_endpoints(),
        vec!["tcp://127.0.0.1:52597"]
    );
    assert_eq!(
        factory.pub_sockets[0].bound_endpoints(),
        vec!["tcp://127.0.0.1:40885"]
    );
    assert_eq!(
        factory.rep_sockets[0].bound_endpoints(),
        vec!["tcp://127.0.0.1:42540"]
    );
}

#[test]
fn zeromq_backend_round_trips_execute_over_fake_router_socket() {
    let bootstrap = test_connection();
    let codec = bootstrap.wire_codec();
    let mut factory = FakeZeroMqFactory::default();
    let mut backend = ZeroMqTransportBackend::from_factory(bootstrap.clone(), &mut factory)
        .expect("backend should build");
    let mut driver = backend.socket_driver(Session::new());

    let request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "execute-zmq-1",
                "frontend-session",
                "frontend",
                "execute_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1 + 1")),
        })
        .expect("request should encode");
    factory.router_sockets[0].push_incoming(wire_message_to_frames(request).frames);

    assert_eq!(
        driver
            .pump_once(backend.transport_mut())
            .expect("driver should pump"),
        Some(crate::KernelSocketPollEvent::Channel(KernelChannel::Shell))
    );
    wait_for_fake_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
    );

    let reply_frames = factory.router_sockets[0].take_outgoing();
    assert_eq!(reply_frames.len(), 1);
    let reply_wire = frames_to_wire_message(
        KernelChannel::Shell,
        &bootstrap.wire_config().delimiter,
        reply_frames[0].clone(),
    )
    .expect("reply frames should convert");
    let reply = decode_response_message(&reply_wire).expect("reply should decode");
    match reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected execute response, got {other:?}"),
    }
}

#[test]
fn zeromq_backend_reports_malformed_shell_frames_as_structured_wire_errors() {
    let bootstrap = test_connection();
    let mut factory = FakeZeroMqFactory::default();
    let mut backend = ZeroMqTransportBackend::from_factory(bootstrap, &mut factory)
        .expect("backend should build");
    let mut driver = backend.socket_driver(Session::new());

    factory.router_sockets[0].push_incoming(vec![b"frontend".to_vec(), b"<IDS|MSG>".to_vec()]);

    let error = driver
        .pump_once(backend.transport_mut())
        .expect_err("malformed frames should surface as a wire error");

    assert_eq!(
        error,
        KernelWireError::InvalidWireFrames {
            channel: KernelChannel::Shell,
            message: "missing required signed payload frames".to_string(),
        }
    );
}
