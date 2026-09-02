use super::*;

#[test]
fn native_zeromq_backend_frontend_style_control_interrupt_round_trip() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let harness = crate::zeromq::tests::support::NativeTestHarness::new();
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let control_endpoint = backend.transport_layout().control.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut control = harness.connect_dealer(
        &runtime,
        &control_endpoint,
        "interrupt Control Dealer/Router connection setup",
    );
    harness.establish_dealer_connection(
        &runtime,
        &mut control,
        &codec,
        &bootstrap.wire_config().delimiter,
        &mut driver,
        backend.transport_mut(),
        KernelChannel::Control,
        "connect-zmq-control-interrupt-1",
        "interrupt Control Dealer/Router connection readiness",
    );

    let interrupt_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Control,
            header: KernelMessageHeader::new(
                "interrupt-zmq-1",
                "frontend-session",
                "frontend",
                "interrupt_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Interrupt(crate::KernelInterruptRequest),
        })
        .expect("interrupt request should encode");
    harness.block_on_result(
        &runtime,
        "interrupt request send",
        control.send(frames_to_zmq_message(
            wire_message_to_frames(interrupt_request).frames,
        )),
    );

    wait_for_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Control),
        "interrupt request delivery",
    );

    let reply_wire = harness.block_on_result(&runtime, "interrupt reply", control.recv());
    let reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Control,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(reply_wire),
        )
        .expect("control reply frames should convert"),
    )
    .expect("control reply should decode");
    match reply.response {
        KernelResponse::Interrupt(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(
                response.lifecycle_state,
                crate::KernelLifecycleState::InterruptRequested
            );
        }
        other => panic!("expected interrupt reply, got {other:?}"),
    }
}

#[test]
fn native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let harness = crate::zeromq::tests::support::NativeTestHarness::new();
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let control_endpoint = backend.transport_layout().control.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = harness.connect_dealer(
        &runtime,
        &shell_endpoint,
        "shutdown Shell Dealer/Router connection setup",
    );
    let mut control = harness.connect_dealer(
        &runtime,
        &control_endpoint,
        "shutdown Control Dealer/Router connection setup",
    );
    harness.establish_dealer_connection(
        &runtime,
        &mut shell,
        &codec,
        &bootstrap.wire_config().delimiter,
        &mut driver,
        backend.transport_mut(),
        KernelChannel::Shell,
        "connect-zmq-shutdown-shell-1",
        "shutdown Shell Dealer/Router connection readiness",
    );
    harness.establish_dealer_connection(
        &runtime,
        &mut control,
        &codec,
        &bootstrap.wire_config().delimiter,
        &mut driver,
        backend.transport_mut(),
        KernelChannel::Control,
        "connect-zmq-shutdown-control-1",
        "shutdown Control Dealer/Router connection readiness",
    );

    let execute_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "execute-zmq-shutdown-queued-1",
                "frontend-session",
                "frontend",
                "execute_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
        })
        .expect("queued execute request should encode");
    let shutdown_header = KernelMessageHeader::new(
        "shutdown-zmq-1",
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
            request: KernelRequest::Shutdown(crate::KernelShutdownRequest { restart: true }),
        })
        .expect("shutdown request should encode");

    harness.block_on_result(
        &runtime,
        "shutdown request send",
        control.send(frames_to_zmq_message(
            wire_message_to_frames(shutdown_request).frames,
        )),
    );
    harness.block_on_result(
        &runtime,
        "queued Shell execute request send",
        shell.send(frames_to_zmq_message(
            wire_message_to_frames(execute_request).frames,
        )),
    );

    wait_for_pending_socket_channels(
        &harness,
        backend.transport(),
        &[KernelChannel::Control, KernelChannel::Shell],
        "Control and Shell request delivery precondition",
    );
    let first_event = wait_for_next_socket_event(
        &harness,
        &mut driver,
        backend.transport_mut(),
        "Control-before-Shell priority dispatch",
    );
    assert_eq!(
        first_event,
        crate::KernelSocketPollEvent::Channel(KernelChannel::Control)
    );

    let report = driver
        .pump_until_idle(backend.transport_mut())
        .expect("driver should stop cleanly after shutdown");
    assert_eq!(report.control_messages, 0);
    assert_eq!(report.shell_messages, 0);
    assert_eq!(report.stdin_messages, 0);
    assert_eq!(report.heartbeat_messages, 0);
    assert!(report.stopped_for_shutdown);
    assert!(
        backend
            .transport()
            .has_pending_channel(KernelChannel::Shell),
        "post-shutdown absence: queued Shell request must remain pending"
    );
    assert_eq!(
        driver.event_pump().adapter().execution_count(),
        0,
        "post-shutdown absence: queued Shell execute request must remain unprocessed"
    );
    assert!(
        !driver.event_pump().adapter().execute_running(),
        "post-shutdown absence: queued Shell execute request must not start a worker"
    );
    assert!(driver.event_pump().adapter().shutdown_requested());
    assert_eq!(
        driver.event_pump().adapter().lifecycle_state(),
        crate::KernelLifecycleState::Shutdown
    );

    let control_reply_wire = harness.block_on_result(&runtime, "shutdown reply", control.recv());
    let control_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Control,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(control_reply_wire),
        )
        .expect("shutdown reply frames should convert"),
    )
    .expect("shutdown reply should decode");
    assert_eq!(control_reply.header.username, "rum-kernel");
    assert_eq!(
        control_reply.header.protocol_version,
        shutdown_header.protocol_version
    );
    assert_eq!(control_reply.parent_header, Some(shutdown_header.clone()));

    match control_reply.response {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(response.restart);
            assert_eq!(
                response.lifecycle_state,
                crate::KernelLifecycleState::ShutdownRequested
            );
        }
        other => panic!("expected shutdown reply, got {other:?}"),
    }
}
