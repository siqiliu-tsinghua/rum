use super::*;

#[test]
fn native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let harness = crate::zeromq::tests::support::NativeTestHarness::new();
    let codec = bootstrap.wire_codec();
    let frontend_identity =
        PeerIdentity::from_str("frontend-zmq-multi-client").expect("identity should parse");
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let stdin_endpoint = backend.transport_layout().stdin.uri();
    let control_endpoint = backend.transport_layout().control.uri();
    let iopub_endpoint = backend.transport_layout().iopub.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell_options = SocketOptions::default();
    shell_options.peer_identity(frontend_identity.clone());
    let mut shell = DealerSocket::with_options(shell_options);
    harness.block_on_result(
        &runtime,
        "session-flow Shell Dealer/Router connection setup",
        shell.connect(&shell_endpoint),
    );
    let mut stdin_options = SocketOptions::default();
    stdin_options.peer_identity(frontend_identity);
    let mut stdin = DealerSocket::with_options(stdin_options);
    harness.block_on_result(
        &runtime,
        "session-flow Stdin Dealer/Router connection setup",
        stdin.connect(&stdin_endpoint),
    );
    let mut control = harness.connect_dealer(
        &runtime,
        &control_endpoint,
        "session-flow Control Dealer/Router connection setup",
    );
    let mut iopub = SubSocket::new();
    harness.block_on_result(
        &runtime,
        "session-flow IOPub SUB connection setup",
        iopub.connect(&iopub_endpoint),
    );
    harness.block_on_result(
        &runtime,
        "session-flow IOPub SUB subscription request",
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
        "iopub-barrier-zmq-session-flow",
    );

    let first_execute_header = KernelMessageHeader::new(
        "execute-zmq-multi-1",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let first_execute = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: first_execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
        })
        .expect("first execute request should encode");
    harness.block_on_result(
        &runtime,
        "session-flow first execute request send",
        shell.send(frames_to_zmq_message(
            wire_message_to_frames(first_execute).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        "session-flow first execute request delivery",
    );
    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
        "session-flow first execute worker completion",
    );

    let first_reply_wire = harness.block_on_result(
        &runtime,
        "session-flow first execute Shell reply",
        shell.recv(),
    );
    let first_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(first_reply_wire),
        )
        .expect("first shell reply frames should convert"),
    )
    .expect("first shell reply should decode");
    match first_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected first execute reply, got {other:?}"),
    }

    let first_observed_iopub = harness.collect_iopub_for_parent(
        &runtime,
        &mut iopub,
        &codec,
        &bootstrap.wire_config().delimiter,
        &first_execute_header,
        "session-flow first execute IOPub collection",
    );
    harness.assert_exact_execute_iopub_sequence(
        &first_observed_iopub,
        "session-flow first execute IOPub sequence",
    );
    let first_iopub = first_observed_iopub
        .into_iter()
        .map(|(_, envelope)| envelope)
        .collect::<Vec<_>>();
    assert!(first_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "1" && input.execution_count == Some(1)
    )));
    assert!(first_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteResult(result)
            if result.data.get("text/plain") == Some("1")
    )));

    let second_execute_header = KernelMessageHeader::new(
        "execute-zmq-multi-2",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let second_execute = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: second_execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new(
                "InputString[\"Name: \"]",
            )),
        })
        .expect("second execute request should encode");
    harness.block_on_result(
        &runtime,
        "session-flow second execute request send",
        shell.send(frames_to_zmq_message(
            wire_message_to_frames(second_execute).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        "session-flow second execute request delivery",
    );
    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
        "session-flow suspension worker completion",
    );

    let stdin_request_wire = harness.block_on_result(
        &runtime,
        "session-flow second execute input request",
        stdin.recv(),
    );
    let stdin_request_wire = frames_to_wire_message(
        KernelChannel::Stdin,
        &bootstrap.wire_config().delimiter,
        zmq_message_to_frames(stdin_request_wire),
    )
    .expect("stdin request frames should convert");
    let stdin_request = crate::decode_input_request_message(&stdin_request_wire)
        .expect("stdin request should decode");
    assert_eq!(stdin_request.request.prompt, "Name: ");

    let input_reply = codec
        .encode_input_reply_envelope(&crate::KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "input-zmq-multi-1",
                "frontend-session",
                "frontend",
                "input_reply",
            ),
            parent_header: Some(stdin_request.header.clone()),
            metadata: KernelMetadata::new(),
            reply: crate::KernelInputReply::new("Ada"),
        })
        .expect("stdin reply should encode");
    harness.block_on_result(
        &runtime,
        "session-flow second execute input reply send",
        stdin.send(frames_to_zmq_message(
            wire_message_to_frames(input_reply).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Stdin),
        "session-flow input reply delivery",
    );
    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
        "session-flow resume worker completion",
    );

    let second_reply_wire = harness.block_on_result(
        &runtime,
        "session-flow second execute Shell reply",
        shell.recv(),
    );
    let second_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(second_reply_wire),
        )
        .expect("second shell reply frames should convert"),
    )
    .expect("second shell reply should decode");
    match second_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(2));
        }
        other => panic!("expected second execute reply, got {other:?}"),
    }

    let second_observed_iopub = harness.collect_iopub_for_parent(
        &runtime,
        &mut iopub,
        &codec,
        &bootstrap.wire_config().delimiter,
        &second_execute_header,
        "session-flow second execute IOPub collection",
    );
    harness.assert_exact_execute_iopub_sequence(
        &second_observed_iopub,
        "session-flow second execute IOPub sequence",
    );
    let second_iopub = second_observed_iopub
        .into_iter()
        .map(|(_, envelope)| envelope)
        .collect::<Vec<_>>();
    assert!(second_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "InputString[\"Name: \"]" && input.execution_count == Some(2)
    )));
    assert!(second_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteResult(result)
            if result.data.get("text/plain") == Some("\"Ada\"")
    )));

    let third_execute_header = KernelMessageHeader::new(
        "execute-zmq-multi-3",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let third_execute = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: third_execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("2")),
        })
        .expect("third execute request should encode");
    harness.block_on_result(
        &runtime,
        "session-flow third execute request send",
        shell.send(frames_to_zmq_message(
            wire_message_to_frames(third_execute).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        "session-flow third execute request delivery",
    );
    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
        "session-flow third execute worker completion",
    );

    let third_reply_wire = harness.block_on_result(
        &runtime,
        "session-flow third execute Shell reply",
        shell.recv(),
    );
    let third_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(third_reply_wire),
        )
        .expect("third shell reply frames should convert"),
    )
    .expect("third shell reply should decode");
    match third_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(3));
        }
        other => panic!("expected third execute reply, got {other:?}"),
    }

    let third_observed_iopub = harness.collect_iopub_for_parent(
        &runtime,
        &mut iopub,
        &codec,
        &bootstrap.wire_config().delimiter,
        &third_execute_header,
        "session-flow third execute IOPub collection",
    );
    harness.assert_exact_execute_iopub_sequence(
        &third_observed_iopub,
        "session-flow third execute IOPub sequence",
    );
    let third_iopub = third_observed_iopub
        .into_iter()
        .map(|(_, envelope)| envelope)
        .collect::<Vec<_>>();
    assert!(third_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "2" && input.execution_count == Some(3)
    )));
    assert!(third_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteResult(result)
            if result.data.get("text/plain") == Some("2")
    )));

    let shutdown_header = KernelMessageHeader::new(
        "shutdown-zmq-multi-1",
        "frontend-session",
        "frontend",
        "shutdown_request",
    );
    let shutdown_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Control,
            header: shutdown_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Shutdown(crate::KernelShutdownRequest { restart: false }),
        })
        .expect("shutdown request should encode");
    harness.block_on_result(
        &runtime,
        "session-flow shutdown request send",
        control.send(frames_to_zmq_message(
            wire_message_to_frames(shutdown_request).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Control),
        "session-flow shutdown request delivery",
    );

    let shutdown_reply_wire = harness.block_on_result(
        &runtime,
        "session-flow shutdown Control reply",
        control.recv(),
    );
    let shutdown_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Control,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(shutdown_reply_wire),
        )
        .expect("shutdown reply frames should convert"),
    )
    .expect("shutdown reply should decode");
    match shutdown_reply.response {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(!response.restart);
            assert_eq!(
                response.lifecycle_state,
                crate::KernelLifecycleState::ShutdownRequested
            );
        }
        other => panic!("expected shutdown reply, got {other:?}"),
    }
}
