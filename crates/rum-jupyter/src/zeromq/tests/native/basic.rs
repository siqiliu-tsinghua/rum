use super::*;

#[test]
fn native_zeromq_backend_round_trips_execute_over_loopback_sockets() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut dealer = runtime.block_on(async {
        let mut dealer = DealerSocket::new();
        dealer
            .connect(&shell_endpoint)
            .await
            .expect("dealer should connect");
        dealer
    });

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
    runtime.block_on(async {
        dealer
            .send(frames_to_zmq_message(
                wire_message_to_frames(request).frames,
            ))
            .await
            .expect("dealer should send request");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
    );
    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
    );

    let reply_frames = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), dealer.recv())
            .await
            .expect("dealer should receive reply before timeout")
            .expect("dealer reply should decode")
            .into_vec()
            .into_iter()
            .map(|frame| frame.to_vec())
            .collect::<Vec<_>>()
    });
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
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let heartbeat_endpoint = backend.transport_layout().heartbeat.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut heartbeat = runtime.block_on(async {
        let mut heartbeat = ReqSocket::new();
        heartbeat
            .connect(&heartbeat_endpoint)
            .await
            .expect("req socket should connect");
        heartbeat
    });
    runtime.block_on(async {
        heartbeat
            .send(ZmqMessage::from(Vec::from("hb")))
            .await
            .expect("heartbeat request should send");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Heartbeat,
    );

    let reply = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), heartbeat.recv())
            .await
            .expect("heartbeat reply should arrive before timeout")
            .expect("heartbeat reply should decode")
    });
    assert_eq!(zmq_message_to_frames(reply), vec![b"hb".to_vec()]);
}
