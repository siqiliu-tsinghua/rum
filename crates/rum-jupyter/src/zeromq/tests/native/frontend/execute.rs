use super::*;

#[test]
fn native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let iopub_endpoint = backend.transport_layout().iopub.uri();
    let heartbeat_endpoint = backend.transport_layout().heartbeat.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = runtime.block_on(async {
        let mut shell = DealerSocket::new();
        shell
            .connect(&shell_endpoint)
            .await
            .expect("shell dealer should connect");
        shell
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

    let mut iopub_messages = Vec::new();
    while iopub_messages.len() < 4 {
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
        iopub_messages.push(envelope);
    }

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
