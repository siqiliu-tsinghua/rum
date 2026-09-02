use super::*;

#[test]
fn connection_file_parser_loads_jupyter_connection_shapes() {
    let connection = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 57503,
            "iopub_port": 40885,
            "stdin_port": 52597,
            "control_port": 50160,
            "hb_port": 42540,
            "key": "secret",
            "signature_scheme": "hmac-sha256",
            "kernel_name": "rum"
        }"#,
    )
    .expect("connection file should parse");

    assert_eq!(connection.transport, KernelTransportKind::Tcp);
    assert_eq!(connection.ip, "127.0.0.1");
    assert_eq!(connection.shell_port, 57503);
    assert_eq!(
        connection.channel_endpoint(KernelChannel::Shell),
        "tcp://127.0.0.1:57503"
    );
    assert_eq!(connection.heartbeat_endpoint(), "tcp://127.0.0.1:42540");
    assert_eq!(
        connection.signature_scheme,
        KernelSignatureScheme::HmacSha256
    );
    assert_eq!(connection.kernel_name.as_deref(), Some("rum"));
    assert_eq!(
        connection.signature_policy(),
        KernelSignaturePolicy::Required("secret".to_string())
    );
}

#[test]
fn connection_file_parser_reports_missing_required_fields() {
    let error = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1"
        }"#,
    )
    .expect_err("incomplete connection file should fail");

    assert_eq!(
        error,
        KernelConnectionFileError::MissingField {
            field: "shell_port"
        }
    );
}

#[test]
fn bootstrap_config_derives_endpoints_and_wire_policy_from_connection_file() {
    let connection = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 57503,
            "iopub_port": 40885,
            "stdin_port": 52597,
            "control_port": 50160,
            "hb_port": 42540,
            "key": "secret",
            "signature_scheme": "hmac-sha256"
        }"#,
    )
    .expect("connection file should parse");

    let bootstrap = KernelBootstrapConfig::from_connection_file(connection)
        .with_username("rum-kernel")
        .with_session_validation("frontend-session");
    let layout = bootstrap.transport_layout();
    let wire = bootstrap.wire_config();

    assert_eq!(bootstrap.username, "rum-kernel");
    assert_eq!(layout.shell.uri(), "tcp://127.0.0.1:57503");
    assert_eq!(layout.control.uri(), "tcp://127.0.0.1:50160");
    assert_eq!(layout.stdin.uri(), "tcp://127.0.0.1:52597");
    assert_eq!(layout.iopub.uri(), "tcp://127.0.0.1:40885");
    assert_eq!(layout.heartbeat.uri(), "tcp://127.0.0.1:42540");
    assert_eq!(wire.session_id, Some("frontend-session".to_string()));
    assert_eq!(
        wire.signature_policy,
        KernelSignaturePolicy::Required("secret".to_string())
    );
}

#[test]
fn connection_file_signature_policy_respects_declared_signature_scheme() {
    let connection = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 57503,
            "iopub_port": 40885,
            "stdin_port": 52597,
            "control_port": 50160,
            "hb_port": 42540,
            "key": "secret",
            "signature_scheme": "hmac-sha512"
        }"#,
    )
    .expect("connection file should parse");

    assert_eq!(
        connection.signature_scheme,
        KernelSignatureScheme::Other("hmac-sha512".to_string())
    );
    assert_eq!(
        connection.signature_policy(),
        KernelSignaturePolicy::Disabled
    );
}

#[test]
fn connection_file_parser_round_trips_unicode_string_fields() {
    let connection = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 57503,
            "iopub_port": 40885,
            "stdin_port": 52597,
            "control_port": 50160,
            "hb_port": 42540,
            "key": "密钥🔒",
            "signature_scheme": "hmac-sha256",
            "kernel_name": "若木内核"
        }"#,
    )
    .expect("unicode connection file should parse");

    assert_eq!(connection.key, "密钥🔒");
    assert_eq!(connection.kernel_name.as_deref(), Some("若木内核"));
}

#[test]
fn mock_transport_backend_exposes_bootstrap_boundary_for_future_real_transports() {
    let connection = KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 57503,
            "iopub_port": 40885,
            "stdin_port": 52597,
            "control_port": 50160,
            "hb_port": 42540,
            "key": "",
            "signature_scheme": ""
        }"#,
    )
    .expect("connection file should parse");

    let bootstrap = KernelBootstrapConfig::from_connection_file(connection);
    let backend = MockTransportBackend::new(bootstrap.clone());

    assert_eq!(backend.bootstrap_config(), &bootstrap);
    assert_eq!(
        backend.transport_layout().shell.uri(),
        "tcp://127.0.0.1:57503"
    );
    assert!(backend.transport().shell.is_empty());

    let pump = backend.event_pump(Session::new());
    assert_eq!(pump.codec().config().session_id, None);
    assert_eq!(
        pump.codec().config().signature_policy,
        KernelSignaturePolicy::Disabled
    );
}

#[test]
fn local_socket_transport_backend_exposes_frontend_boundary_for_frontend_style_flows() {
    let bootstrap = KernelBootstrapConfig::from_connection_file(test_connection_file())
        .with_username("rum-kernel")
        .with_session_validation("frontend-session");
    let backend = LocalSocketTransportBackend::new(bootstrap.clone());
    let frontend = backend.frontend();

    assert_eq!(backend.bootstrap_config(), &bootstrap);
    assert_eq!(
        backend.transport_layout().shell.uri(),
        "tcp://127.0.0.1:57503"
    );
    assert_eq!(
        backend.transport_layout().heartbeat.uri(),
        "tcp://127.0.0.1:42540"
    );
    assert_eq!(frontend.outgoing_len(KernelChannel::Shell), 0);
    assert!(!frontend.has_pending_outgoing());
}

#[test]
fn frontend_style_execute_round_trip_uses_local_socket_backend() {
    let bootstrap = KernelBootstrapConfig::from_connection_file(test_connection_file())
        .with_username("rum-kernel")
        .with_session_validation("frontend-session");
    let codec = bootstrap.wire_codec();
    let mut backend = LocalSocketTransportBackend::new(bootstrap);
    let frontend = backend.frontend();
    let mut pump = backend.event_pump(Session::new());
    let request_header = KernelMessageHeader::new(
        "execute-1",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: request_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("1 + 1")),
        })
        .expect("request should encode");
    frontend.queue_incoming(request);

    let report = pump
        .pump_until_idle(backend.transport_mut())
        .expect("local backend should pump execute request");
    assert_eq!(report.shell_messages, 1);

    let reply_wire = frontend
        .pop_outgoing(KernelChannel::Shell)
        .expect("frontend should receive a shell reply");
    let reply = decode_response_message(&reply_wire).expect("reply should decode");
    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    let iopub = frontend.drain_outgoing(KernelChannel::IoPub);
    assert_eq!(iopub.len(), 4);
    let execute_result = decode_iopub_message(&iopub[2]).expect("execute result should decode");
    assert_eq!(execute_result.header.session_id, "frontend-session");
    assert_eq!(execute_result.header.username, "rum-kernel");
    assert_eq!(reply.parent_header, Some(request_header.clone()));
    assert_eq!(execute_result.parent_header, Some(request_header));

    let reencoded_reply = codec
        .decode_response_message(&reply_wire)
        .expect("reply should still validate against bootstrap wire config");
    assert_eq!(reencoded_reply.header.session_id, "frontend-session");
}

#[test]
fn frontend_style_stdin_round_trip_uses_local_socket_backend() {
    let bootstrap = KernelBootstrapConfig::from_connection_file(test_connection_file())
        .with_session_validation("frontend-session");
    let codec = bootstrap.wire_codec();
    let mut backend = LocalSocketTransportBackend::new(bootstrap);
    let frontend = backend.frontend();
    let mut pump = backend.event_pump(Session::new());
    let request_header = KernelMessageHeader::new(
        "execute-stdin-1",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    frontend.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: request_header.clone(),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new(
                    "InputString[\"Name: \"]",
                )),
            })
            .expect("request should encode"),
    );

    let report = pump
        .pump_until_idle(backend.transport_mut())
        .expect("initial stdin-producing execute should pump");
    assert_eq!(report.shell_messages, 1);
    assert_eq!(report.execute_completions, 1);

    let stdin_wire = frontend
        .pop_outgoing(KernelChannel::Stdin)
        .expect("frontend should receive stdin request");
    let stdin_request =
        decode_input_request_message(&stdin_wire).expect("stdin request should decode");
    assert_eq!(stdin_request.request.prompt, "Name: ");

    frontend.queue_incoming(
        codec
            .encode_input_reply_envelope(&KernelInputReplyEnvelope {
                channel: KernelChannel::Stdin,
                header: KernelMessageHeader::new(
                    "input-reply-1",
                    "frontend-session",
                    "frontend",
                    "input_reply",
                ),
                parent_header: Some(stdin_request.header.clone()),
                metadata: KernelMetadata::new(),
                reply: KernelInputReply::new("Ada"),
            })
            .expect("stdin reply should encode"),
    );

    let report = pump
        .pump_until_idle(backend.transport_mut())
        .expect("stdin resume should complete");
    assert_eq!(report.stdin_messages, 1);

    let reply_wire = frontend
        .pop_outgoing(KernelChannel::Shell)
        .expect("frontend should receive resumed shell reply");
    let reply = decode_response_message(&reply_wire).expect("reply should decode");
    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
            assert_eq!(reply.parent_header, Some(request_header.clone()));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    let iopub = frontend.drain_outgoing(KernelChannel::IoPub);
    let execute_result = iopub
        .iter()
        .find_map(|message| match decode_iopub_message(message) {
            Ok(envelope) => match envelope.payload {
                KernelIoPubPayload::ExecuteResult(result) => Some(result),
                _ => None,
            },
            Err(_) => None,
        })
        .expect("resume should publish execute_result over iopub");
    assert_eq!(execute_result.data.get("text/plain"), Some("\"Ada\""));
}

#[test]
fn local_socket_transport_backend_echoes_heartbeat_payloads() {
    let bootstrap = KernelBootstrapConfig::from_connection_file(test_connection_file());
    let mut backend = LocalSocketTransportBackend::new(bootstrap);
    let frontend = backend.frontend();

    frontend.send_heartbeat(b"ping".to_vec());
    frontend.send_heartbeat(b"pong".to_vec());

    assert!(backend.pump_heartbeat_once());
    assert_eq!(frontend.pop_heartbeat_reply(), Some(b"ping".to_vec()));

    assert_eq!(backend.pump_heartbeats_until_idle(), 1);
    assert_eq!(frontend.pop_heartbeat_reply(), Some(b"pong".to_vec()));
    assert!(!backend.pump_heartbeat_once());
}

#[test]
fn socket_session_config_derives_zero_mq_facing_socket_shapes() {
    let bootstrap = KernelBootstrapConfig::from_connection_file(test_connection_file());
    let session = bootstrap.socket_session_config();

    assert_eq!(session.shell.kind, KernelSocketKind::Router);
    assert_eq!(session.control.kind, KernelSocketKind::Router);
    assert_eq!(session.stdin.kind, KernelSocketKind::Router);
    assert_eq!(session.iopub.kind, KernelSocketKind::Pub);
    assert_eq!(session.heartbeat.kind, KernelSocketKind::Rep);

    assert_eq!(session.shell.bind_mode, KernelSocketBindMode::Bind);
    assert_eq!(session.heartbeat.bind_mode, KernelSocketBindMode::Bind);
    assert_eq!(session.shell.uri(), "tcp://127.0.0.1:57503");
    assert_eq!(session.iopub.uri(), "tcp://127.0.0.1:40885");
    assert_eq!(session.heartbeat.uri(), "tcp://127.0.0.1:42540");
}

#[test]
fn socket_driver_pumps_heartbeat_and_execute_over_local_session() {
    let bootstrap = KernelBootstrapConfig::from_connection_file(test_connection_file())
        .with_username("rum-kernel")
        .with_session_validation("frontend-session");
    let codec = bootstrap.wire_codec();
    let mut backend = LocalSocketTransportBackend::new(bootstrap.clone());
    let frontend = backend.frontend();
    let mut driver = bootstrap.socket_driver(Session::new());

    frontend.send_heartbeat(b"hb".to_vec());
    frontend.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "execute-driver-1",
                    "frontend-session",
                    "frontend",
                    "execute_request",
                ),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1 + 1")),
            })
            .expect("request should encode"),
    );

    assert_eq!(
        driver
            .pump_once(backend.transport_mut())
            .expect("heartbeat should pump"),
        Some(KernelSocketPollEvent::Heartbeat)
    );
    assert_eq!(frontend.pop_heartbeat_reply(), Some(b"hb".to_vec()));

    assert_eq!(
        driver
            .pump_once(backend.transport_mut())
            .expect("execute should pump"),
        Some(KernelSocketPollEvent::Channel(KernelChannel::Shell))
    );
    let completion = driver
        .pump_until_idle(backend.transport_mut())
        .expect("execute completion should pump");
    assert_eq!(completion.execute_completions, 1);

    let reply_wire = frontend
        .pop_outgoing(KernelChannel::Shell)
        .expect("frontend should receive a shell reply");
    let reply = decode_response_message(&reply_wire).expect("reply should decode");
    match reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }
}

#[test]
fn socket_driver_observes_shutdown_promptly_during_running_execute() {
    let bootstrap = KernelBootstrapConfig::from_connection_file(test_connection_file())
        .with_session_validation("frontend-session");
    let codec = bootstrap.wire_codec();
    let mut backend = LocalSocketTransportBackend::new(bootstrap.clone());
    let frontend = backend.frontend();
    let mut driver = bootstrap.socket_driver(long_running_cancel_session());
    let worker_gate = driver
        .event_pump_mut()
        .adapter_mut()
        .gate_next_execute_worker();

    frontend.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "execute-driver-shutdown-1",
                    "frontend-session",
                    "frontend",
                    "execute_request",
                ),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("Do[x = 1, 5000000000]")),
            })
            .expect("execute request should encode"),
    );
    assert_eq!(
        driver
            .pump_once(backend.transport_mut())
            .expect("execute should start"),
        Some(KernelSocketPollEvent::Channel(KernelChannel::Shell))
    );
    driver
        .event_pump_mut()
        .adapter_mut()
        .wait_for_execute_worker_started();
    assert!(driver.event_pump().adapter().execute_running());

    let shutdown = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Control,
            header: KernelMessageHeader::new(
                "shutdown-driver-running-1",
                "frontend-session",
                "frontend",
                "shutdown_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Shutdown(KernelShutdownRequest { restart: false }),
        })
        .expect("shutdown request should encode");
    frontend.queue_incoming(shutdown);

    let report = driver
        .pump_until_idle(backend.transport_mut())
        .expect("shutdown should be observed while execute is running");

    assert_eq!(report.control_messages, 1);
    assert!(driver.event_pump().adapter().shutdown_requested());
    let reply = frontend
        .pop_outgoing(KernelChannel::Control)
        .expect("frontend should receive shutdown reply");
    match decode_response_message(&reply)
        .expect("shutdown reply should decode")
        .response
    {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(!response.restart);
        }
        other => panic!("expected shutdown reply, got {other:?}"),
    }

    // Shutdown intentionally does not alter worker cancellation semantics.
    // Cancel directly so this test never leaves its long-running worker detached.
    driver
        .event_pump_mut()
        .adapter_mut()
        .interrupt(KernelInterruptRequest);
    worker_gate.release();
    driver
        .event_pump_mut()
        .adapter_mut()
        .wait_for_execute_worker_finished();
    assert_eq!(
        driver
            .pump_once(backend.transport_mut())
            .expect("cancelled execute completion should pump"),
        Some(KernelSocketPollEvent::ExecuteCompletion)
    );
}

#[test]
fn socket_driver_resumes_stdin_round_trip_over_local_session() {
    let bootstrap = KernelBootstrapConfig::from_connection_file(test_connection_file())
        .with_session_validation("frontend-session");
    let codec = bootstrap.wire_codec();
    let mut backend = LocalSocketTransportBackend::new(bootstrap.clone());
    let frontend = backend.frontend();
    let mut driver = bootstrap.socket_driver(Session::new());

    frontend.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "execute-driver-stdin-1",
                    "frontend-session",
                    "frontend",
                    "execute_request",
                ),
                parent_header: None,
                metadata: KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new(
                    "InputString[\"Name: \"]",
                )),
            })
            .expect("request should encode"),
    );

    let first = driver
        .pump_until_idle(backend.transport_mut())
        .expect("initial stdin-producing execute should pump");
    assert_eq!(first.shell_messages, 1);
    assert_eq!(first.heartbeat_messages, 0);

    let stdin_wire = frontend
        .pop_outgoing(KernelChannel::Stdin)
        .expect("frontend should receive stdin request");
    let stdin_request =
        decode_input_request_message(&stdin_wire).expect("stdin request should decode");

    frontend.send_heartbeat(b"mid".to_vec());
    frontend.queue_incoming(
        codec
            .encode_input_reply_envelope(&KernelInputReplyEnvelope {
                channel: KernelChannel::Stdin,
                header: KernelMessageHeader::new(
                    "input-driver-reply-1",
                    "frontend-session",
                    "frontend",
                    "input_reply",
                ),
                parent_header: Some(stdin_request.header.clone()),
                metadata: KernelMetadata::new(),
                reply: KernelInputReply::new("Ada"),
            })
            .expect("stdin reply should encode"),
    );

    let second = driver
        .pump_until_idle(backend.transport_mut())
        .expect("stdin reply should resume execute");
    assert_eq!(second.stdin_messages, 1);
    assert_eq!(second.heartbeat_messages, 1);
    assert_eq!(frontend.pop_heartbeat_reply(), Some(b"mid".to_vec()));

    let reply_wire = frontend
        .pop_outgoing(KernelChannel::Shell)
        .expect("frontend should receive resumed shell reply");
    let reply = decode_response_message(&reply_wire).expect("reply should decode");
    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }
}
