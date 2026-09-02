use super::*;

#[test]
fn native_zeromq_backend_round_trips_execute_over_loopback_sockets() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let harness = crate::zeromq::tests::support::NativeTestHarness::new();
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut dealer = harness.connect_dealer(
        &runtime,
        &shell_endpoint,
        "Shell Dealer/Router connection setup",
    );
    harness.establish_dealer_connection(
        &runtime,
        &mut dealer,
        &codec,
        &bootstrap.wire_config().delimiter,
        &mut driver,
        backend.transport_mut(),
        KernelChannel::Shell,
        "connect-zmq-real-1",
        "Shell Dealer/Router connection readiness",
    );

    let request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "execute-zmq-real-1",
                "frontend-session",
                "frontend",
                "execute_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1 + 1")),
        })
        .expect("request should encode");
    harness.block_on_result(
        &runtime,
        "execute request send",
        dealer.send(frames_to_zmq_message(
            wire_message_to_frames(request).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        "execute request delivery",
    );
    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
        "execute worker completion",
    );

    let reply_frames = harness
        .block_on_result(&runtime, "execute reply", dealer.recv())
        .into_vec()
        .into_iter()
        .map(|frame| frame.to_vec())
        .collect::<Vec<_>>();
    let reply_wire = frames_to_wire_message(
        KernelChannel::Shell,
        &bootstrap.wire_config().delimiter,
        reply_frames,
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
fn native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let harness = crate::zeromq::tests::support::NativeTestHarness::new();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let heartbeat_endpoint = backend.transport_layout().heartbeat.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut heartbeat = harness.connect_req(
        &runtime,
        &heartbeat_endpoint,
        "heartbeat Req/Rep connection setup",
    );
    harness.heartbeat_round_trip(
        &runtime,
        &mut heartbeat,
        &mut driver,
        backend.transport_mut(),
        b"heartbeat-connection-ready".to_vec(),
        "heartbeat Req/Rep connection readiness",
    );
    harness.heartbeat_round_trip(
        &runtime,
        &mut heartbeat,
        &mut driver,
        backend.transport_mut(),
        b"hb".to_vec(),
        "heartbeat echo",
    );
}
