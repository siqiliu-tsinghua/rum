use super::*;

#[test]
fn native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let codec = bootstrap.wire_codec();
    let frontend_identity =
        PeerIdentity::from_str("frontend-zmq-stdin-client").expect("identity should parse");
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let stdin_endpoint = backend.transport_layout().stdin.uri();
    let iopub_endpoint = backend.transport_layout().iopub.uri();
    let heartbeat_endpoint = backend.transport_layout().heartbeat.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = runtime.block_on(async {
        let mut options = SocketOptions::default();
        options.peer_identity(frontend_identity.clone());
        let mut shell = DealerSocket::with_options(options);
        shell
            .connect(&shell_endpoint)
            .await
            .expect("shell dealer should connect");
        shell
    });
    let mut stdin = runtime.block_on(async {
        let mut options = SocketOptions::default();
        options.peer_identity(frontend_identity);
        let mut stdin = DealerSocket::with_options(options);
        stdin
            .connect(&stdin_endpoint)
            .await
            .expect("stdin dealer should connect");
        stdin
    });
    let mut iopub = runtime.block_on(async {
        let mut iopub = SubSocket::new();
        iopub
            .connect(&iopub_endpoint)
            .await
            .expect("iopub sub should connect");
        iopub
            .subscribe("")
            .await
            .expect("iopub sub should subscribe");
        iopub
    });

    acknowledge_iopub_subscription(
        &runtime,
        &heartbeat_endpoint,
        &mut driver,
        backend.transport_mut(),
    );

    let execute_header = KernelMessageHeader::new(
        "execute-zmq-stdin-1",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let execute_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new(
                "InputString[\"Name: \"]",
            )),
        })
        .expect("execute request should encode");
    runtime.block_on(async {
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(execute_request).frames,
            ))
            .await
            .expect("shell dealer should send execute request");
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

    let stdin_request_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), stdin.recv())
            .await
            .expect("stdin request should arrive before timeout")
            .expect("stdin request should decode")
    });
    let stdin_request_wire = frames_to_wire_message(
        KernelChannel::Stdin,
        &bootstrap.wire_config().delimiter,
        zmq_message_to_frames(stdin_request_wire),
    )
    .expect("stdin request frames should convert");
    let stdin_request = crate::decode_input_request_message(&stdin_request_wire)
        .expect("stdin request should parse");
    assert_eq!(stdin_request.request.prompt, "Name: ");

    let input_reply = codec
        .encode_input_reply_envelope(&crate::KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "input-zmq-reply-1",
                "frontend-session",
                "frontend",
                "input_reply",
            ),
            parent_header: Some(stdin_request.header.clone()),
            metadata: KernelMetadata::new(),
            reply: crate::KernelInputReply::new("Ada"),
        })
        .expect("input reply should encode");
    runtime.block_on(async {
        stdin
            .send(frames_to_zmq_message(
                wire_message_to_frames(input_reply).frames,
            ))
            .await
            .expect("stdin dealer should send input reply");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Stdin),
    );
    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
    );

    let shell_reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), shell.recv())
            .await
            .expect("shell reply should arrive before timeout")
            .expect("shell reply should decode")
    });
    let shell_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(shell_reply_wire),
        )
        .expect("shell reply frames should convert"),
    )
    .expect("shell reply should decode");
    match shell_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    let mut iopub_payloads = Vec::new();
    while iopub_payloads.len() < 4 {
        let message = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                .await
                .expect("iopub message should arrive before timeout")
                .expect("iopub message should decode")
        });
        let envelope = crate::decode_iopub_message(
            &frames_to_wire_message(
                KernelChannel::IoPub,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(message),
            )
            .expect("iopub frames should convert"),
        )
        .expect("iopub message should parse");
        iopub_payloads.push(envelope.payload);
    }

    assert!(iopub_payloads.iter().any(|payload| matches!(
        payload,
        crate::KernelIoPubPayload::Status(status)
            if status.execution_state == crate::KernelExecutionState::Busy
    )));
    assert!(iopub_payloads.iter().any(|payload| matches!(
        payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "InputString[\"Name: \"]"
    )));
    assert!(iopub_payloads.iter().any(|payload| matches!(
        payload,
        crate::KernelIoPubPayload::ExecuteResult(result)
            if result.data.get("text/plain") == Some("\"Ada\"")
    )));
    assert!(iopub_payloads.iter().any(|payload| matches!(
        payload,
        crate::KernelIoPubPayload::Status(status)
            if status.execution_state == crate::KernelExecutionState::Idle
    )));
}
