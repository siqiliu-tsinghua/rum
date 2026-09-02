use super::*;

#[test]
fn native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages() {
    let Some(bootstrap) = signed_loopback_connection() else {
        return;
    };
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("signed native zeromq backend should build");
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

    let request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "execute-zmq-signed-1",
                "frontend-session",
                "frontend",
                "execute_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
        })
        .expect("signed execute request should encode");
    assert!(request.signature.is_some());

    runtime.block_on(async {
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(request).frames,
            ))
            .await
            .expect("shell dealer should send signed execute request");
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

    let reply_wire = frames_to_wire_message(
        KernelChannel::Shell,
        &bootstrap.wire_config().delimiter,
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), shell.recv())
                .await
                .expect("signed shell reply should arrive before timeout")
                .expect("signed shell reply should decode")
                .into_vec()
                .into_iter()
                .map(|frame| frame.to_vec())
                .collect::<Vec<_>>()
        }),
    )
    .expect("signed shell reply frames should convert");
    assert!(reply_wire.signature.is_some());
    let reply = codec
        .decode_response_message(&reply_wire)
        .expect("signed shell reply should validate and decode");
    match reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected signed execute reply, got {other:?}"),
    }

    let mut saw_signed_execute_result = false;
    while !saw_signed_execute_result {
        let message = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                .await
                .expect("signed iopub message should arrive before timeout")
                .expect("signed iopub message should decode")
        });
        let wire = frames_to_wire_message(
            KernelChannel::IoPub,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(message),
        )
        .expect("signed iopub frames should convert");
        assert!(wire.signature.is_some());
        let envelope = codec
            .decode_iopub_message(&wire)
            .expect("signed iopub message should validate and decode");
        if matches!(
            envelope.payload,
            crate::KernelIoPubPayload::ExecuteResult(ref result)
                if result.data.get("text/plain") == Some("1")
        ) {
            saw_signed_execute_result = true;
        }
    }
}

#[test]
fn native_zeromq_backend_rejects_tampered_signed_requests_end_to_end() {
    let Some(bootstrap) = signed_loopback_connection() else {
        return;
    };
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("signed native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = runtime.block_on(async {
        let mut shell = DealerSocket::new();
        shell
            .connect(&shell_endpoint)
            .await
            .expect("shell dealer should connect");
        shell
    });

    runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

    let mut tampered_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "execute-zmq-signed-bad-1",
                "frontend-session",
                "frontend",
                "execute_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
        })
        .expect("signed request should encode before tampering");
    tampered_request.signature = Some("bad-signature".to_string());

    runtime.block_on(async {
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(tampered_request).frames,
            ))
            .await
            .expect("shell dealer should send tampered signed request");
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match driver.pump_once(backend.transport_mut()) {
            Err(crate::KernelWireError::InvalidSignature) => break,
            Ok(None) | Ok(Some(_)) => {
                assert!(
                    Instant::now() < deadline,
                    "timed out waiting for invalid-signature failure"
                );
                thread::sleep(Duration::from_millis(10));
            }
            Err(other) => panic!("expected invalid signature error, got {other:?}"),
        }
    }

    let shell_timeout = runtime
        .block_on(async { tokio::time::timeout(Duration::from_millis(200), shell.recv()).await });
    assert!(
        shell_timeout.is_err(),
        "tampered signed request should not produce a shell reply"
    );
}
