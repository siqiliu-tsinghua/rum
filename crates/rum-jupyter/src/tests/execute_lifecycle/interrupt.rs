use super::*;

// Promoted from brief-552; the original reply and route prove retirement.
#[test]
fn red_interrupt_while_waiting_finishes_request() {
    for streaming in [false, true] {
        interrupt_at_prompt(false, streaming);
    }
}

#[test]
fn interrupt_at_second_prompt_finishes_original_request() {
    for streaming in [false, true] {
        interrupt_at_prompt(true, streaming);
    }
}

fn interrupt_at_prompt(second: bool, streaming: bool) {
    let (send, receive) = std::sync::mpsc::channel();
    let mut session = Session::new();
    if streaming {
        // HostSession's streaming sink replaces per-request print buffering.
        session.set_print_sink(rum_host::ClosurePrintSink::assert_bounded_destruction(
            move |line| {
                send.send((line.to_string(), std::thread::current().id()))
                    .unwrap();
            },
        ));
    }
    let codec = KernelWireCodec::new(KernelWireConfig::default());
    let mut pump = KernelEventPump::new(KernelAdapter::new(session), codec.clone());
    let mut transport = MockSocketTransport::default();
    let source = "kept=42; cleanupCount=0; tail=0; WithCleanup[InputString[\"first: \"]; InputString[\"second: \"], cleanupCount=cleanupCount+1; Print[\"cleanup\"]]; tail=1; Print[\"tail\"]";
    let original = request("original", source);
    wire_request(
        &mut pump,
        &mut transport,
        &original,
        &[b"original", b"shell"],
    );
    pump_completion(&mut pump, &mut transport);
    let first = codec
        .decode_input_request_message(&transport.stdin[0])
        .unwrap();
    if second {
        wire_input(&mut pump, &mut transport, &first, "first answer");
        pump_completion(&mut pump, &mut transport);
    }
    let current = codec
        .decode_input_request_message(transport.stdin.last().unwrap())
        .unwrap();
    assert_eq!(
        current.request.prompt,
        if second { "second: " } else { "first: " }
    );
    assert_eq!(pump.adapter().execution_count(), 1);
    assert!(transport.shell.is_empty());
    let gate = pump.adapter_mut().gate_next_execute_worker();
    let mut interrupt = request("interrupt", "");
    interrupt.channel = KernelChannel::Control;
    interrupt.header.msg_type = "interrupt_request".into();
    interrupt.request = KernelRequest::Interrupt(KernelInterruptRequest);
    wire_request(&mut pump, &mut transport, &interrupt, &[b"control"]);
    assert!(
        !pump.adapter().awaiting_stdin(),
        "interrupt must withdraw the prompt"
    );
    assert!(pump.adapter().pending_input_request().is_none());
    assert!(
        pump.adapter().execute_running(),
        "cleanup must run on a worker"
    );
    pump.adapter_mut().wait_for_execute_worker_started();
    assert_eq!(pump.adapter().execution_state(), KernelExecutionState::Busy);
    assert_eq!(
        pump.adapter().lifecycle_state(),
        KernelLifecycleState::InterruptRequested
    );
    assert_eq!(transport.control.len(), 1);
    assert_eq!(transport.control[0].routing_ids, [b"control".to_vec()]);
    assert!(
        matches!(codec.decode_response_message(&transport.control[0]).unwrap().response,
        KernelResponse::Interrupt(reply) if reply.status == KernelReplyStatus::Ok)
    );
    assert!(
        receive.try_recv().is_err(),
        "cleanup is gated off the transport thread"
    );
    assert_eq!(
        pump.adapter_mut()
            .handle_input_reply(input_reply(&current, "late")),
        Err(KernelStdinError::NoPendingInputRequest)
    );
    // Repeated interrupts and overlap during retirement must leave one owner.
    wire_request(&mut pump, &mut transport, &interrupt, &[b"control"]);
    wire_request(
        &mut pump,
        &mut transport,
        &request("overlap", "tail=99"),
        &[b"overlap"],
    );
    assert_eq!(transport.shell.len(), 1);
    assert_eq!(transport.shell[0].routing_ids, [b"overlap".to_vec()]);
    let before = transport
        .iopub
        .iter()
        .map(|w| codec.decode_iopub_message(w).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(idle_count(&before), 0);
    gate.release();
    pump_completion(&mut pump, &mut transport);
    assert!(!pump.pump_execute_completion(&mut transport).unwrap());
    assert_eq!(
        transport.shell.len(),
        2,
        "exactly one original terminal reply plus the rejected overlap"
    );
    let terminal = codec.decode_response_message(&transport.shell[1]).unwrap();
    assert_eq!(terminal.parent_header, Some(original.header.clone()));
    assert_eq!(
        transport.shell[1].routing_ids,
        [b"original".to_vec(), b"shell".to_vec()]
    );
    let KernelResponse::Execute(result) = terminal.response else {
        panic!("execute reply")
    };
    assert_eq!(result.status, KernelReplyStatus::Error);
    assert_eq!(result.execution_count, Some(1));
    assert_eq!(result.error.unwrap().error_name, "Cancelled");
    let outputs = transport
        .iopub
        .iter()
        .map(|w| codec.decode_iopub_message(w).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(idle_count(&outputs), 1);
    assert_eq!(transport.stdin.len(), if second { 2 } else { 1 });
    let errors = outputs
        .iter()
        .filter(|o| {
            matches!(&o.payload,
        KernelIoPubPayload::Error(error) if error.error_name == "Cancelled")
        })
        .collect::<Vec<_>>();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].parent_header, Some(original.header.clone()));
    assert_eq!(outputs.last().unwrap().parent_header, Some(original.header));
    assert!(
        matches!(&outputs.last().unwrap().payload, KernelIoPubPayload::Status(status) if status.execution_state == KernelExecutionState::Idle)
    );
    if !streaming {
        assert_eq!(
            outputs
                .iter()
                .filter(
                    |o| matches!(&o.payload, KernelIoPubPayload::Stream(s) if s.text == "cleanup\n")
                )
                .count(),
            1
        );
    }
    assert!(
        !outputs.iter().any(
            |o| matches!(&o.payload, KernelIoPubPayload::Stream(s) if s.text.contains("tail"))
        )
    );
    if streaming {
        let (line, thread) = receive.try_recv().unwrap();
        assert_eq!(line, "cleanup");
        assert_ne!(thread, std::thread::current().id());
        assert!(receive.try_recv().is_err());
    }
    assert!(!pump.adapter().awaiting_stdin());
    assert!(!pump.adapter().execute_running());
    assert_eq!(
        pump.adapter().lifecycle_state(),
        KernelLifecycleState::Running
    );
    assert_eq!(
        pump.adapter_mut()
            .handle_input_reply(input_reply(&current, "late")),
        Err(KernelStdinError::NoPendingInputRequest)
    );
    assert_reusable(pump.adapter_mut());
    assert_eq!(
        pump.adapter().execution_count(),
        2,
        "rejections do not consume execution counts"
    );
    assert_eq!(
        value(
            &pump
                .adapter_mut()
                .execute(KernelExecuteRequest::new("{cleanupCount,tail,kept}"))
        ),
        "List[1, 0, 42]"
    );
    // An old answer also cannot answer a new execute's live prompt.
    let paused = start(pump.adapter_mut(), "InputString[\"new: \"]");
    assert!(matches!(
        pump.adapter_mut()
            .handle_input_reply(input_reply(&first, "stale")),
        Err(KernelStdinError::ReplyParentMismatch { .. })
    ));
    let done = resume(pump.adapter_mut(), &paused.stdin[0], "new answer");
    assert_eq!(value(response(&done)), "\"new answer\"");
}

#[test]
fn interrupt_after_suspension_before_completion_poll_retires_on_worker() {
    let mut adapter = KernelAdapter::new(Session::new());
    let initial = adapter.dispatch_request(request("original", "cleanupCount=0; tail=0; WithCleanup[InputString[\"p: \"], cleanupCount=cleanupCount+1]; tail=1"));
    adapter.wait_for_execute_worker_finished();
    adapter.interrupt(KernelInterruptRequest);
    let mut done = adapter.poll_execute_completion().unwrap();
    assert!(
        done.stdin.is_empty(),
        "cancelled input must never be advertised"
    );
    if done.reply.is_none() {
        done = wait_for_execute_completion(&mut adapter);
    }
    assert_eq!(
        response(&done).error.as_ref().unwrap().error_name,
        "Cancelled"
    );
    assert_eq!(
        done.reply.as_ref().unwrap().parent_header,
        Some(request("original", "").header)
    );
    assert_eq!(idle_count(&initial.iopub) + idle_count(&done.iopub), 1);
    assert_eq!(
        value(&adapter.execute(KernelExecuteRequest::new("{cleanupCount,tail}"))),
        "List[1, 0]"
    );
    assert_reusable(&mut adapter);
}

#[test]
fn interrupt_after_denied_input_retirement_does_not_retire_again() {
    let mut adapter = KernelAdapter::new(Session::new());
    let mut execute = request(
        "denied",
        "kept=42; cleanupCount=0; tail=0; WithCleanup[InputString[\"p: \"], cleanupCount=cleanupCount+1; Print[\"cleanup\"]]; tail=1",
    );
    let KernelRequest::Execute(body) = &mut execute.request else {
        unreachable!()
    };
    body.allow_stdin = false;
    adapter.dispatch_request(execute);
    adapter.wait_for_execute_worker_finished();
    adapter.interrupt(KernelInterruptRequest);
    let completion = adapter.poll_execute_completion().unwrap();
    let done = finish_dispatch(&mut adapter, completion);
    assert_eq!(
        response(&done).error.as_ref().unwrap().error_name,
        "InputRequired"
    );
    assert!(done.stdin.is_empty());
    assert!(!adapter.execute_running());
    assert!(!adapter.awaiting_stdin());
    assert_eq!(adapter.lifecycle_state(), KernelLifecycleState::Running);
    assert_eq!(idle_count(&done.iopub), 1);
    assert_eq!(
        done.iopub
            .iter()
            .filter(|o| matches!(&o.payload,
        KernelIoPubPayload::Stream(s) if s.text == "cleanup\n"))
            .count(),
        1
    );
    assert_eq!(
        value(&adapter.execute(KernelExecuteRequest::new("{cleanupCount,tail,kept}"))),
        "List[1, 0, 42]"
    );
    assert_reusable(&mut adapter);
}
