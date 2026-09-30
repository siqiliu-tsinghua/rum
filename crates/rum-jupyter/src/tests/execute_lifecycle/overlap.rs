use super::*;

#[test]
fn red_overlap_rejection_preserves_busy_state() {
    overlap(false, false);
}

#[test]
fn overlap_while_running_is_rejected_before_session_move() {
    overlap(true, false);
}

#[test]
fn immediate_overlap_while_running_is_rejected() {
    overlap(true, true);
}

#[test]
fn immediate_overlap_while_waiting_preserves_count_and_context() {
    overlap(false, true);
}

fn overlap(running: bool, immediate: bool) {
    let mut adapter = KernelAdapter::new(Session::new());
    let gate = running.then(|| adapter.gate_next_execute_worker());
    let original = request("original", "kept=42; InputString[\"owner: \"]");
    let started = adapter.dispatch_request(original.clone());
    let mut paused = if running {
        adapter.wait_for_execute_worker_started();
        None
    } else {
        Some(finish_adapter_execute(&mut adapter, started))
    };
    let input_before = adapter.pending_input_request().cloned();
    if immediate {
        let rejected = adapter.execute(KernelExecuteRequest::new("kept=99"));
        assert!(
            matches!(rejected.outcome, KernelExecuteOutcome::Failure(ref f) if f.code == rum_host::HostFailureCode::Other)
        );
    } else {
        let next = request("overlap", "kept=99");
        let rejected = adapter.dispatch_request(next.clone());
        let rejected = finish_dispatch(&mut adapter, rejected);
        assert_eq!(response(&rejected).status, KernelReplyStatus::Error);
        assert_eq!(rejected.iopub.len(), 1);
        assert!(matches!(
            &rejected.iopub[0].payload,
            KernelIoPubPayload::Error(_)
        ));
        assert_eq!(
            rejected.reply.as_ref().unwrap().parent_header,
            Some(next.header)
        );
        assert_eq!(
            idle_count(&rejected.iopub),
            0,
            "rejection must not publish idle"
        );
        assert!(rejected.stdin.is_empty());
    }
    assert_eq!(adapter.execution_state(), KernelExecutionState::Busy);
    assert_eq!(adapter.lifecycle_state(), KernelLifecycleState::Running);
    assert_eq!(adapter.execution_count(), 1);
    assert_eq!(adapter.pending_input_request(), input_before.as_ref());
    if let Some(gate) = gate {
        gate.release();
        paused = Some(wait_for_execute_completion(&mut adapter));
    }
    let paused = paused.unwrap();
    let done = resume(&mut adapter, &paused.stdin[0], "original answer");
    assert_eq!(value(response(&done)), "\"original answer\"");
    assert_eq!(response(&done).execution_count, Some(1));
    assert_eq!(done.reply.unwrap().parent_header, Some(original.header));
    assert_reusable(&mut adapter);
    assert_eq!(
        value(&adapter.execute(KernelExecuteRequest::new("kept"))),
        "42"
    );
}

#[test]
fn red_overlap_reply_keeps_original_routing() {
    wire_overlap(false, false);
}

#[test]
fn overlap_with_identical_parent_keeps_original_routing() {
    wire_overlap(false, true);
}

#[test]
fn wire_overlap_while_running_keeps_original_routing() {
    wire_overlap(true, true);
}

fn wire_overlap(running: bool, same_parent: bool) {
    let codec = KernelWireCodec::new(KernelWireConfig::default());
    let mut pump = KernelEventPump::new(KernelAdapter::new(Session::new()), codec.clone());
    let mut transport = MockSocketTransport::default();
    let gate = running.then(|| pump.adapter_mut().gate_next_execute_worker());
    let original = request("original", "InputString[\"owner: \"]");
    wire_request(
        &mut pump,
        &mut transport,
        &original,
        &[b"original", b"shell"],
    );
    if running {
        pump.adapter_mut().wait_for_execute_worker_started();
    } else {
        pump_completion(&mut pump, &mut transport);
    }
    let mut second = request("overlap", "99");
    if same_parent {
        second.header = original.header.clone();
    }
    wire_request(&mut pump, &mut transport, &second, &[b"overlap"]);
    if !running && transport.shell.is_empty() {
        pump_completion(&mut pump, &mut transport);
    }
    assert_eq!(transport.shell.len(), 1);
    assert_eq!(transport.shell[0].routing_ids, [b"overlap".to_vec()]);
    let rejected = codec.decode_response_message(&transport.shell[0]).unwrap();
    assert_eq!(rejected.parent_header, Some(second.header));
    assert!(
        matches!(rejected.response, KernelResponse::Execute(r) if r.status == KernelReplyStatus::Error)
    );
    if let Some(gate) = gate {
        gate.release();
        pump_completion(&mut pump, &mut transport);
    }
    let input = codec
        .decode_input_request_message(&transport.stdin[0])
        .unwrap();
    wire_input(&mut pump, &mut transport, &input, "answer");
    pump_completion(&mut pump, &mut transport);
    assert_eq!(transport.shell.len(), 2);
    assert_eq!(
        transport.shell[1].routing_ids,
        [b"original".to_vec(), b"shell".to_vec()]
    );
    let reply = codec.decode_response_message(&transport.shell[1]).unwrap();
    assert_eq!(reply.parent_header, Some(original.header));
    assert!(
        matches!(reply.response, KernelResponse::Execute(r) if r.status == KernelReplyStatus::Ok && r.execution_count == Some(1))
    );
    assert_reusable(pump.adapter_mut());
}
