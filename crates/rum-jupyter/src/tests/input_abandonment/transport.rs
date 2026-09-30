use super::*;
use crate::{KernelSocketDriver, LocalSocketSession, PriorityKernelSocketPoller};

#[test]
fn red_event_pump_without_stdin_abandons() {
    let codec = KernelWireCodec::new(KernelWireConfig::default());
    let mut pump = KernelEventPump::new(KernelAdapter::new(Session::new()), codec.clone());
    let mut transport = MockSocketTransport::default();
    let mut wire = codec
        .encode_request_envelope(&request("first", cleanup_source(), false))
        .unwrap();
    wire.routing_ids = vec![b"original".to_vec()];
    transport.queue_incoming(wire);
    pump.pump_until_idle(&mut transport).unwrap();
    assert!(transport.stdin.is_empty());
    let terminal = codec.decode_response_message(&transport.shell[0]).unwrap();
    assert_wire_input_error(&terminal.response);
    assert_eq!(transport.shell[0].routing_ids, vec![b"original".to_vec()]);
    let stdout: Vec<_> = transport
        .iopub
        .iter()
        .filter_map(
            |wire| match codec.decode_iopub_message(wire).unwrap().payload {
                KernelIoPubPayload::Stream(stream) if stream.name == KernelStreamName::Stdout => {
                    Some(stream.text)
                }
                _ => None,
            },
        )
        .collect();
    assert_eq!(stdout, ["prefix\n", "cleanup\n"]);
    transport.queue_incoming(
        codec
            .encode_request_envelope(&request("next", "1+1", false))
            .unwrap(),
    );
    pump.pump_until_idle(&mut transport).unwrap();
    assert_wire_two(&codec, &transport.iopub);
    assert_reusable(pump.adapter_mut());
}

#[test]
fn red_local_driver_without_stdin_abandons() {
    let config = KernelBootstrapConfig::from_connection_file(test_connection_file());
    let codec = config.wire_codec();
    let mut driver = KernelSocketDriver::new(
        config.event_pump(Session::new()),
        PriorityKernelSocketPoller,
    );
    let (mut sockets, frontend) = LocalSocketSession::new(config.socket_session_config());
    frontend.queue_incoming(
        codec
            .encode_request_envelope(&request("first", "InputString[\"p: \"]", false))
            .unwrap(),
    );
    driver.pump_until_idle(&mut sockets).unwrap();
    assert_eq!(frontend.outgoing_len(KernelChannel::Stdin), 0);
    let terminal = codec
        .decode_response_message(&frontend.pop_outgoing(KernelChannel::Shell).unwrap())
        .unwrap();
    assert_wire_input_error(&terminal.response);
    frontend.queue_incoming(
        codec
            .encode_request_envelope(&request("next", "1+1", false))
            .unwrap(),
    );
    driver.pump_until_idle(&mut sockets).unwrap();
    let mut iopub = Vec::new();
    while let Some(wire) = frontend.pop_outgoing(KernelChannel::IoPub) {
        iopub.push(wire);
    }
    assert_wire_two(&codec, &iopub);
    assert_reusable(driver.event_pump_mut().adapter_mut());
}

fn assert_wire_input_error(response: &KernelResponse) {
    let KernelResponse::Execute(response) = response else {
        panic!("expected execute reply")
    };
    assert_eq!(response.status, KernelReplyStatus::Error);
    let error = response.error.as_ref().unwrap();
    assert_eq!(error.error_name, "InputRequired");
    assert_eq!(error.error_value, "input required: p: ");
}

fn assert_wire_two(codec: &KernelWireCodec, iopub: &[crate::KernelWireMessage]) {
    assert!(
        iopub.iter().any(|wire| matches!(
            codec.decode_iopub_message(wire).unwrap().payload,
            KernelIoPubPayload::ExecuteResult(result) if result.data.get("text/plain") == Some("2")
        )),
        "following execute must evaluate 1+1 to 2"
    );
}
