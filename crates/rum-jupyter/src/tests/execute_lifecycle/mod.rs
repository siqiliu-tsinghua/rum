//! Admitted asynchronous executes own cancellation, input and reply routing.
use super::*;
use crate::{KernelExecuteResponse, KernelInputRequestEnvelope, KernelStdinError};

mod interrupt;
mod overlap;

fn request(id: &str, source: &str) -> KernelRequestEnvelope {
    KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(id, "execute-lifecycle", "tester", "execute_request"),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new(source)),
    }
}

fn input_reply(input: &KernelInputRequestEnvelope, answer: &str) -> KernelInputReplyEnvelope {
    KernelInputReplyEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new("answer", "execute-lifecycle", "tester", "input_reply"),
        parent_header: Some(input.header.clone()),
        metadata: KernelMetadata::new(),
        reply: KernelInputReply::new(answer),
    }
}

fn response(dispatch: &KernelDispatchResult) -> &KernelExecuteResponse {
    match &dispatch.reply.as_ref().expect("terminal reply").response {
        KernelResponse::Execute(response) => response,
        other => panic!("expected execute reply: {other:?}"),
    }
}

fn value(response: &KernelExecuteResponse) -> &str {
    match &response.outcome {
        KernelExecuteOutcome::Success(success) => &success.value.text,
        other => panic!("expected success: {other:?}"),
    }
}

fn finish_dispatch(
    adapter: &mut KernelAdapter,
    initial: KernelDispatchResult,
) -> KernelDispatchResult {
    if initial.reply.is_some() {
        initial
    } else {
        finish_adapter_execute(adapter, initial)
    }
}

fn start(adapter: &mut KernelAdapter, source: &str) -> KernelDispatchResult {
    let initial = adapter.dispatch_request(request("original", source));
    finish_dispatch(adapter, initial)
}

fn resume(
    adapter: &mut KernelAdapter,
    input: &KernelInputRequestEnvelope,
    answer: &str,
) -> KernelDispatchResult {
    let initial = adapter
        .handle_input_reply(input_reply(input, answer))
        .unwrap()
        .unwrap();
    finish_dispatch(adapter, initial)
}

fn idle_count(outputs: &[crate::KernelIoPubEnvelope]) -> usize {
    outputs
        .iter()
        .filter(|out| {
            matches!(&out.payload,
        KernelIoPubPayload::Status(status) if status.execution_state == KernelExecutionState::Idle)
        })
        .count()
}

fn wire_request(
    pump: &mut KernelEventPump,
    transport: &mut MockSocketTransport,
    envelope: &KernelRequestEnvelope,
    route: &[&[u8]],
) {
    let mut wire = pump.codec().encode_request_envelope(envelope).unwrap();
    wire.routing_ids = route.iter().map(|id| id.to_vec()).collect();
    pump.handle_wire_request(transport, wire).unwrap();
}

fn pump_completion(pump: &mut KernelEventPump, transport: &mut MockSocketTransport) {
    pump.adapter_mut().wait_for_execute_worker_finished();
    assert!(pump.pump_execute_completion(transport).unwrap());
}

fn wire_input(
    pump: &mut KernelEventPump,
    transport: &mut MockSocketTransport,
    input: &KernelInputRequestEnvelope,
    answer: &str,
) {
    let wire = pump
        .codec()
        .encode_input_reply_envelope(&input_reply(input, answer))
        .unwrap();
    pump.handle_wire_input_reply(transport, wire).unwrap();
}

fn assert_reusable(adapter: &mut KernelAdapter) {
    let done = start(adapter, "1+1");
    assert_eq!(value(response(&done)), "2");
    assert!(!adapter.awaiting_stdin());
    assert!(!adapter.execute_running());
    assert!(adapter.poll_execute_completion().is_none());
}

#[test]
fn mismatched_parent_preserves_waiting_execute() {
    let mut adapter = KernelAdapter::new(Session::new());
    let paused = start(&mut adapter, "InputString[\"p: \"]");
    let mut wrong = input_reply(&paused.stdin[0], "wrong");
    wrong.parent_header.as_mut().unwrap().msg_id = "wrong-parent".into();
    assert!(matches!(
        adapter.handle_input_reply(wrong),
        Err(KernelStdinError::ReplyParentMismatch { .. })
    ));
    assert_eq!(adapter.pending_input_request(), Some(&paused.stdin[0]));
    assert_eq!(adapter.execution_state(), KernelExecutionState::Busy);
    let done = resume(&mut adapter, &paused.stdin[0], "right");
    assert_eq!(value(response(&done)), "\"right\"");
    assert_reusable(&mut adapter);
}

#[test]
fn non_execute_reply_during_input_wait_preserves_busy() {
    let mut adapter = KernelAdapter::new(Session::new());
    let paused = start(&mut adapter, "InputString[\"p: \"]");
    let mut info = request("info", "");
    info.request = KernelRequest::KernelInfo(KernelInfoRequest);
    let dispatch = adapter.dispatch_request(info);
    assert!(dispatch.reply.is_some());
    assert_eq!(idle_count(&dispatch.iopub), 0);
    assert_eq!(adapter.execution_state(), KernelExecutionState::Busy);
    let done = resume(&mut adapter, &paused.stdin[0], "right");
    assert_eq!(value(response(&done)), "\"right\"");
}
