use super::*;

#[test]
fn native_zeromq_backend_frontend_style_control_interrupt_round_trip() {
    let Some(bootstrap) = loopback_connection() else {
        return;
    };
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let control_endpoint = backend.transport_layout().control.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut control = runtime.block_on(async {
        let mut control = DealerSocket::new();
        control
            .connect(&control_endpoint)
            .await
            .expect("control dealer should connect");
        control
    });

    runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

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
    runtime.block_on(async {
        control
            .send(frames_to_zmq_message(
                wire_message_to_frames(interrupt_request).frames,
            ))
            .await
            .expect("control dealer should send interrupt request");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Control),
    );

    let reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), control.recv())
            .await
            .expect("control reply should arrive before timeout")
            .expect("control reply should decode")
    });
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
    let codec = bootstrap.wire_codec();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let shell_endpoint = backend.transport_layout().shell.uri();
    let control_endpoint = backend.transport_layout().control.uri();
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = runtime.block_on(async {
        let mut shell = DealerSocket::new();
        shell
            .connect(&shell_endpoint)
            .await
            .expect("shell dealer should connect");
        shell
    });
    let mut control = runtime.block_on(async {
        let mut control = DealerSocket::new();
        control
            .connect(&control_endpoint)
            .await
            .expect("control dealer should connect");
        control
    });

    runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

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

    runtime.block_on(async {
        control
            .send(frames_to_zmq_message(
                wire_message_to_frames(shutdown_request).frames,
            ))
            .await
            .expect("control dealer should send shutdown request");
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(execute_request).frames,
            ))
            .await
            .expect("shell dealer should send queued execute request");
    });

    wait_for_pending_socket_channels(
        backend.transport(),
        &[KernelChannel::Control, KernelChannel::Shell],
    );
    let first_event = wait_for_next_socket_event(&mut driver, backend.transport_mut());
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
    assert!(driver.event_pump().adapter().shutdown_requested());
    assert_eq!(
        driver.event_pump().adapter().lifecycle_state(),
        crate::KernelLifecycleState::Shutdown
    );

    let control_reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), control.recv())
            .await
            .expect("shutdown reply should arrive before timeout")
            .expect("shutdown reply should decode")
    });
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

    let shell_timeout = runtime
        .block_on(async { tokio::time::timeout(Duration::from_millis(200), shell.recv()).await });
    assert!(
        shell_timeout.is_err(),
        "queued shell work should remain pending after shutdown"
    );
}
