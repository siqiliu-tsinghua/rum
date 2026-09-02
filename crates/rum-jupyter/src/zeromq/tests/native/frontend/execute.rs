use super::*;

#[test]
fn native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let harness = crate::zeromq::tests::support::NativeTestHarness::new();
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let iopub_endpoint = backend.transport_layout().iopub.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = harness.connect_dealer(
        &runtime,
        &shell_endpoint,
        "frontend execute Shell Dealer/Router connection setup",
    );
    let mut iopub = SubSocket::new();
    harness.block_on_result(
        &runtime,
        "frontend execute IOPub SUB connection setup",
        iopub.connect(&iopub_endpoint),
    );
    harness.block_on_result(
        &runtime,
        "frontend execute IOPub SUB subscription request",
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
        "iopub-barrier-zmq-metadata",
    );

    let execute_header = KernelMessageHeader::new(
        "execute-zmq-metadata-1",
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
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
        })
        .expect("execute request should encode");
    harness.block_on_result(
        &runtime,
        "frontend execute request send",
        shell.send(frames_to_zmq_message(
            wire_message_to_frames(execute_request).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        "frontend execute request delivery",
    );
    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
        "frontend execute worker completion",
    );

    let shell_reply_wire =
        harness.block_on_result(&runtime, "frontend execute Shell reply", shell.recv());
    let shell_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(shell_reply_wire),
        )
        .expect("shell reply frames should convert"),
    )
    .expect("shell reply should decode");

    assert_eq!(shell_reply.header.username, "rum-kernel");
    assert_eq!(
        shell_reply.header.protocol_version,
        execute_header.protocol_version
    );
    assert_eq!(shell_reply.parent_header, Some(execute_header.clone()));
    assert_eq!(shell_reply.metadata.get("channel"), Some("shell"));
    assert_eq!(
        shell_reply.metadata.get("parent_msg_type"),
        Some("execute_request")
    );

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
        "frontend execute IOPub collection",
    );
    harness.assert_exact_execute_iopub_sequence(&observed_iopub, "frontend execute IOPub sequence");
    let iopub_messages = observed_iopub
        .into_iter()
        .map(|(_, envelope)| envelope)
        .collect::<Vec<_>>();

    for message in &iopub_messages {
        assert_eq!(message.header.username, "rum-kernel");
        assert_eq!(
            message.header.protocol_version,
            execute_header.protocol_version
        );
        assert_eq!(message.parent_header, Some(execute_header.clone()));
        assert_eq!(message.metadata.get("channel"), Some("iopub"));
        assert_eq!(
            message.metadata.get("parent_msg_type"),
            Some("execute_request")
        );
    }

    assert!(iopub_messages.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::Status(status)
            if status.execution_state == crate::KernelExecutionState::Busy
                && status.lifecycle_state == crate::KernelLifecycleState::Running
    )));
    assert!(iopub_messages.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "1" && input.execution_count == Some(1)
    )));
    assert!(iopub_messages.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteResult(result) if result.data.get("text/plain").is_some()
    )));
    assert!(iopub_messages.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::Status(status)
            if status.execution_state == crate::KernelExecutionState::Idle
                && status.lifecycle_state == crate::KernelLifecycleState::Running
    )));
}
