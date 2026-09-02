use super::*;

#[test]
fn event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-13")
            .with_signature_key("secret"),
    );
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    let mut request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-66",
                "session-13",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
        })
        .expect("request should encode");
    request.routing_ids = vec![b"frontend".to_vec(), b"shell".to_vec()];
    transport.queue_incoming(request);

    assert!(
        pump.pump_once(&mut transport, KernelChannel::Shell)
            .expect("pump should succeed")
    );
    assert_eq!(transport.shell.len(), 0);
    assert_eq!(transport.iopub.len(), 2);
    pump.adapter_mut().wait_for_execute_worker_finished();
    assert!(
        pump.pump_execute_completion(&mut transport)
            .expect("completion poll should succeed")
    );
    assert_eq!(transport.shell.len(), 1);
    assert_eq!(transport.iopub.len(), 4);
    assert_eq!(
        transport.shell[0].routing_ids,
        vec![b"frontend".to_vec(), b"shell".to_vec()]
    );
    assert!(
        transport
            .iopub
            .iter()
            .all(|message| message.routing_ids.is_empty())
    );
}

#[test]
fn pump_next_prioritizes_control_messages_before_shell_messages() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-15"));
    let adapter = KernelAdapter::new(long_running_cancel_session());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-68",
                    "session-15",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
            })
            .expect("shell request should encode"),
    );
    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: KernelMessageHeader::new(
                    "client-msg-69",
                    "session-15",
                    "tester",
                    "interrupt_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Interrupt(KernelInterruptRequest),
            })
            .expect("control request should encode"),
    );

    let channel = pump
        .pump_next(&mut transport)
        .expect("pump should succeed")
        .expect("one request should be processed");

    assert_eq!(channel, KernelPumpEvent::Channel(KernelChannel::Control));
    assert_eq!(transport.control.len(), 1);
    assert!(transport.shell.is_empty());
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);
}

#[test]
fn pump_next_prioritizes_stdin_replies_while_kernel_is_waiting_for_input() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-15b"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    let parent_header = KernelMessageHeader::new(
        "client-msg-stdin-parent",
        "session-15b",
        "tester",
        "execute_request",
    );
    let pending = pump
        .adapter_mut()
        .request_stdin_for_execute(
            &parent_header,
            &KernelExecuteRequest::new("futureInput[]"),
            KernelInputRequest::new("name: "),
        )
        .expect("stdin request should stage");

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-shell",
                    "session-15b",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
            })
            .expect("shell request should encode"),
    );
    transport.queue_incoming(encode_input_reply_envelope(&KernelInputReplyEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new(
            "client-msg-stdin-reply",
            "session-15b",
            "tester",
            "input_reply",
        ),
        parent_header: Some(pending.header.clone()),
        metadata: super::KernelMetadata::new(),
        reply: KernelInputReply::new("Alice"),
    }));

    let channel = pump
        .pump_next(&mut transport)
        .expect("pump should succeed")
        .expect("one request should be processed");

    assert_eq!(channel, KernelPumpEvent::Channel(KernelChannel::Stdin));
    assert!(!pump.adapter().awaiting_stdin());
    assert_eq!(
        pump.adapter_mut()
            .take_last_input_reply()
            .map(|reply| reply.value),
        Some("Alice".to_string())
    );
    assert!(transport.shell.is_empty());
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);
}

#[test]
fn control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-int"));
    let adapter = KernelAdapter::new(long_running_cancel_session());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();
    let worker_gate = pump.adapter_mut().gate_next_execute_worker();

    let mut infinite = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-interrupt-run",
                "session-int",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("Do[x = 1, 5000000000]")),
        })
        .expect("execute request should encode");
    infinite.routing_ids = vec![b"frontend".to_vec(), b"shell".to_vec()];
    transport.queue_incoming(infinite);

    let start_event = pump
        .pump_next(&mut transport)
        .expect("execute start should pump")
        .expect("execute request should be processed");
    assert_eq!(start_event, KernelPumpEvent::Channel(KernelChannel::Shell));
    pump.adapter_mut().wait_for_execute_worker_started();
    assert!(pump.adapter().execute_running());
    assert_eq!(pump.adapter().execution_state(), KernelExecutionState::Busy);
    assert!(transport.shell.is_empty());
    assert_eq!(transport.iopub.len(), 2);

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-after-interrupt",
                    "session-int",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("42")),
            })
            .expect("deferred shell request should encode"),
    );
    assert!(
        pump.pump_next(&mut transport)
            .expect("busy shell deferral should not error")
            .is_none()
    );
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: KernelMessageHeader::new(
                    "client-msg-interrupt",
                    "session-int",
                    "tester",
                    "interrupt_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Interrupt(KernelInterruptRequest),
            })
            .expect("interrupt request should encode"),
    );

    assert_eq!(
        pump.pump_next(&mut transport)
            .expect("interrupt should pump"),
        Some(KernelPumpEvent::Channel(KernelChannel::Control))
    );
    worker_gate.release();
    pump.adapter_mut().wait_for_execute_worker_finished();
    assert_eq!(
        pump.pump_next(&mut transport)
            .expect("cancelled execute completion should pump"),
        Some(KernelPumpEvent::ExecuteCompletion)
    );

    assert_eq!(transport.control.len(), 1);
    let interrupt_reply =
        decode_response_message(&transport.control[0]).expect("interrupt reply should decode");
    match interrupt_reply.response {
        KernelResponse::Interrupt(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(response.interrupt_requested);
        }
        other => panic!("expected interrupt response, got {other:?}"),
    }

    let cancelled_reply =
        decode_response_message(&transport.shell[0]).expect("cancelled reply should decode");
    assert_eq!(
        cancelled_reply
            .parent_header
            .as_ref()
            .map(|header| header.msg_id.as_str()),
        Some("client-msg-interrupt-run")
    );
    match cancelled_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(
                response.status,
                KernelReplyStatus::Error,
                "unexpected interrupted response: {response:?}; content={}",
                transport.shell[0].content_json
            );
            assert_eq!(
                response
                    .error
                    .as_ref()
                    .map(|error| error.error_name.as_str()),
                Some("Cancelled")
            );
        }
        other => panic!("expected cancelled execute reply, got {other:?}"),
    }
    assert_eq!(
        transport.shell[0].routing_ids,
        vec![b"frontend".to_vec(), b"shell".to_vec()]
    );
    assert!(!pump.adapter().execute_running());
    assert_eq!(pump.adapter().execution_state(), KernelExecutionState::Idle);
    assert_eq!(
        pump.adapter().lifecycle_state(),
        KernelLifecycleState::Running
    );
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);

    let iopub_before_followup = transport.iopub.len();
    let followup = pump
        .pump_until_idle(&mut transport)
        .expect("deferred shell execute should run after interrupt");
    assert_eq!(followup.shell_messages, 1);
    assert_eq!(followup.execute_completions, 1);
    assert_eq!(transport.shell.len(), 2);

    let followup_reply =
        decode_response_message(&transport.shell[1]).expect("follow-up reply should decode");
    match followup_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
        }
        other => panic!("expected follow-up execute reply, got {other:?}"),
    }

    assert!(
        transport.iopub[iopub_before_followup..]
            .iter()
            .filter_map(|message| decode_iopub_message(message).ok())
            .any(|message| matches!(
                message.payload,
                KernelIoPubPayload::ExecuteResult(result)
                    if result.data.get("text/plain") == Some("42")
            ))
    );
}

#[test]
fn pump_until_idle_stops_after_shutdown_and_leaves_later_shell_work_queued() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-16"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-70",
                    "session-16",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
            })
            .expect("shell request should encode"),
    );
    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: KernelMessageHeader::new(
                    "client-msg-71",
                    "session-16",
                    "tester",
                    "shutdown_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Shutdown(KernelShutdownRequest { restart: true }),
            })
            .expect("shutdown request should encode"),
    );

    let report = pump
        .pump_until_idle(&mut transport)
        .expect("loop should succeed");

    assert_eq!(report.control_messages, 1);
    assert_eq!(report.shell_messages, 0);
    assert_eq!(report.total_messages(), 1);
    assert!(report.stopped_for_shutdown);
    assert_eq!(transport.control.len(), 1);
    assert!(transport.shell.is_empty());
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);
    assert!(pump.adapter().shutdown_requested());

    let reply = decode_response_message(&transport.control[0]).expect("reply should decode");
    match reply.response {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(response.restart);
            assert_eq!(
                response.lifecycle_state,
                KernelLifecycleState::ShutdownRequested
            );
        }
        other => panic!("expected shutdown reply, got {other:?}"),
    }
}
