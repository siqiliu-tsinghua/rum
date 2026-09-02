use super::*;

#[test]
fn native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages() {
    let Some(bootstrap) = signed_loopback_connection() else {
        return;
    };
    let harness = crate::zeromq::tests::support::NativeTestHarness::new();
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("signed native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let iopub_endpoint = backend.transport_layout().iopub.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = harness.connect_dealer(
        &runtime,
        &shell_endpoint,
        "signed Shell Dealer/Router connection setup",
    );
    harness.establish_dealer_connection(
        &runtime,
        &mut shell,
        &codec,
        &bootstrap.wire_config().delimiter,
        &mut driver,
        backend.transport_mut(),
        KernelChannel::Shell,
        "connect-zmq-signed-1",
        "signed Shell Dealer/Router connection readiness",
    );
    let mut iopub = SubSocket::new();
    harness.block_on_result(
        &runtime,
        "signed IOPub SUB connection setup",
        iopub.connect(&iopub_endpoint),
    );
    harness.block_on_result(
        &runtime,
        "signed IOPub SUB subscription request",
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
        "iopub-barrier-zmq-signed",
    );

    let execute_header = KernelMessageHeader::new(
        "execute-zmq-signed-1",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
        })
        .expect("signed execute request should encode");
    assert!(request.signature.is_some());

    harness.block_on_result(
        &runtime,
        "signed execute request send",
        shell.send(frames_to_zmq_message(
            wire_message_to_frames(request).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
        "signed execute request delivery",
    );
    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::ExecuteCompletion,
        "signed execute worker completion",
    );

    let reply_wire = frames_to_wire_message(
        KernelChannel::Shell,
        &bootstrap.wire_config().delimiter,
        harness
            .block_on_result(&runtime, "signed execute Shell reply", shell.recv())
            .into_vec()
            .into_iter()
            .map(|frame| frame.to_vec())
            .collect::<Vec<_>>(),
    )
    .expect("signed shell reply frames should convert");
    assert!(reply_wire.signature.is_some());
    let reply = codec
        .decode_response_message(&reply_wire)
        .expect("signed shell reply should validate and decode");
    assert_eq!(reply.parent_header, Some(execute_header.clone()));
    match reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected signed execute reply, got {other:?}"),
    }

    let iopub_messages = harness.collect_iopub_for_parent(
        &runtime,
        &mut iopub,
        &codec,
        &bootstrap.wire_config().delimiter,
        &execute_header,
        "signed execute IOPub collection",
    );
    harness.assert_exact_execute_iopub_sequence(&iopub_messages, "signed execute IOPub sequence");

    let mut saw_signed_execute_result = false;
    for (wire, envelope) in &iopub_messages {
        assert!(wire.signature.is_some());
        if matches!(
            envelope.payload,
            crate::KernelIoPubPayload::ExecuteResult(ref result)
                if result.data.get("text/plain") == Some("1")
        ) {
            saw_signed_execute_result = true;
        }
    }
    assert!(
        saw_signed_execute_result,
        "signed target-parent IOPub sequence should contain the execute result"
    );
}

#[test]
fn native_zeromq_backend_rejects_tampered_signed_requests_end_to_end() {
    let Some(bootstrap) = signed_loopback_connection() else {
        return;
    };
    let harness = crate::zeromq::tests::support::NativeTestHarness::new();
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("signed native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = harness.connect_dealer(
        &runtime,
        &shell_endpoint,
        "tampered-request Shell Dealer/Router connection setup",
    );
    harness.establish_dealer_connection(
        &runtime,
        &mut shell,
        &codec,
        &bootstrap.wire_config().delimiter,
        &mut driver,
        backend.transport_mut(),
        KernelChannel::Shell,
        "connect-zmq-signed-bad-1",
        "tampered-request Shell Dealer/Router connection readiness",
    );
    let adapter_state_before = {
        let adapter = driver.event_pump().adapter();
        assert!(
            !adapter.execute_running(),
            "tampered-request absence precondition: connection probe must leave no worker running"
        );
        (
            adapter.execution_count(),
            adapter.execution_state(),
            adapter.lifecycle_state(),
        )
    };

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

    harness.block_on_result(
        &runtime,
        "tampered signed request send",
        shell.send(frames_to_zmq_message(
            wire_message_to_frames(tampered_request).frames,
        )),
    );
    harness.wait_for_invalid_signature(
        &mut driver,
        backend.transport_mut(),
        "tampered signed request delivery and signature rejection",
    );

    // Execute dispatch synchronously changes this state and leaves a worker installed until a
    // later pump. Unchanged idle state therefore proves rejection happened before dispatch and
    // before any execute reply could be enqueued.
    let adapter = driver.event_pump().adapter();
    assert_eq!(
        (
            adapter.execution_count(),
            adapter.execution_state(),
            adapter.lifecycle_state(),
        ),
        adapter_state_before,
        "tampered-request absence: rejection must not dispatch or enqueue a reply"
    );
    assert!(
        !adapter.execute_running(),
        "tampered-request absence: rejection must not start an execute worker"
    );
    assert!(
        !backend
            .transport()
            .has_pending_channel(KernelChannel::Shell),
        "tampered-request absence: rejected Shell input must be consumed without dispatch"
    );
}
