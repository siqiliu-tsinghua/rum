use super::*;

#[test]
fn native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let harness = crate::zeromq::tests::support::NativeTestHarness::new();
    let codec = bootstrap.wire_codec();
    let frontend_identity =
        PeerIdentity::from_str("frontend-zmq-stdin-client").expect("identity should parse");
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let stdin_endpoint = backend.transport_layout().stdin.uri();
    let iopub_endpoint = backend.transport_layout().iopub.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell_options = SocketOptions::default();
    shell_options.peer_identity(frontend_identity.clone());
    let mut shell = DealerSocket::with_options(shell_options);
    harness.block_on_result(
        &runtime,
        "stdin flow Shell Dealer/Router connection setup",
        shell.connect(&shell_endpoint),
    );
    let mut stdin_options = SocketOptions::default();
    stdin_options.peer_identity(frontend_identity);
    let mut stdin = DealerSocket::with_options(stdin_options);
    harness.block_on_result(
        &runtime,
        "stdin flow Stdin Dealer/Router connection setup",
        stdin.connect(&stdin_endpoint),
    );
    let mut iopub = SubSocket::new();
    harness.block_on_result(
        &runtime,
        "stdin flow IOPub SUB connection setup",
        iopub.connect(&iopub_endpoint),
    );
    harness.block_on_result(
        &runtime,
        "stdin flow IOPub SUB subscription request",
        iopub.subscribe(""),
    );

    acknowledge_iopub_subscription(
        &harness,
        &runtime,
        &mut shell,
        &mut iopub,
        &codec,
        &bootstrap.wire_config().delimiter,
        &mut driver,
        backend.transport_mut(),
        "iopub-barrier-zmq-stdin",
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
    harness.block_on_result(
        &runtime,
        "stdin flow execute request send",
        shell.send(frames_to_zmq_message(
            wire_message_to_frames(execute_request).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        "stdin flow execute request delivery",
    );
    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
        "stdin flow suspension worker completion",
    );

    let stdin_request_wire =
        harness.block_on_result(&runtime, "stdin flow input request", stdin.recv());
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
    harness.block_on_result(
        &runtime,
        "stdin flow input reply send",
        stdin.send(frames_to_zmq_message(
            wire_message_to_frames(input_reply).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Stdin),
        "stdin flow input reply delivery",
    );
    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
        "stdin flow resume worker completion",
    );

    let shell_reply_wire =
        harness.block_on_result(&runtime, "stdin flow execute Shell reply", shell.recv());
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

    let observed_iopub = harness.collect_iopub_for_parent(
        &runtime,
        &mut iopub,
        &codec,
        &bootstrap.wire_config().delimiter,
        &execute_header,
        "stdin flow execute IOPub collection",
    );
    harness
        .assert_exact_execute_iopub_sequence(&observed_iopub, "stdin flow execute IOPub sequence");
    let iopub_payloads = observed_iopub
        .into_iter()
        .map(|(_, envelope)| envelope.payload)
        .collect::<Vec<_>>();

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
