//! Measure the host collections without runtime access or buffer mutation.
use super::*;
use rum_host::{HostFailureCode, HostMessage, HostStructuredOutput, HostTraceEvent};

fn buffers(
    outcome: &HostEvalOutcome,
) -> (
    &[String],
    &[HostMessage],
    &[HostTraceEvent],
    &[HostStructuredOutput],
) {
    match outcome {
        HostEvalOutcome::Success(s) => (&s.prints, &s.messages, &s.trace, &s.outputs),
        HostEvalOutcome::InputRequired(s) => (&s.prints, &s.messages, &s.trace, &s.outputs),
        HostEvalOutcome::Failure(s) => (&s.prints, &s.messages, &s.trace, &s.outputs),
    }
}

#[test]
fn host_families_append_across_success_exit_and_cancellation() {
    for ending in ["42", "Exit[7]", "cancel"] {
        let source = format!(
            r#"notice::same="same"; Print["prefix"]; Message[notice::same]; InputString["first: "]; Print["middle"]; Message[notice::same]; WithCleanup[InputString["second: "]; Print["tail"]; Message[notice::same]; {}, Print["cleanup"]]"#,
            if ending == "cancel" { "42" } else { ending }
        );
        let mut session = tracing_session();
        let cancel = session.cancel_handle();
        let first = session.eval(source.clone());
        assert!(matches!(first, HostEvalOutcome::InputRequired(_)));
        let second = session.resume_with_input("one");
        assert!(matches!(second, HostEvalOutcome::InputRequired(_)));
        let last = if ending == "cancel" {
            cancel.cancel();
            session.eval("")
        } else {
            session.resume_with_input("two")
        };
        assert!(!matches!(last, HostEvalOutcome::InputRequired(_)));
        for (label, before, after) in [("suspend", &first, &second), ("terminal", &second, &last)] {
            let (ap, am, at, ao) = buffers(before);
            let (bp, bm, bt, bo) = buffers(after);
            println!(
                "HOST {ending}/{label} prints={}->{} messages={}->{} trace={}->{} grouped_append={}",
                ap.len(),
                bp.len(),
                am.len(),
                bm.len(),
                at.len(),
                bt.len(),
                bo.starts_with(ao)
            );
            assert!(bp.starts_with(ap));
            assert!(bm.starts_with(am));
            assert!(bt.starts_with(at));
            assert!(
                !bo.starts_with(ao),
                "grouped outputs are rebuilt, not a chronology"
            );
        }
        let mut queued = tracing_session();
        let cancel = queued.cancel_handle();
        queued.queue_input_response("one");
        if ending != "cancel" {
            queued.queue_input_response("two");
        }
        let mut expected = queued.eval(source);
        if ending == "cancel" {
            assert!(matches!(expected, HostEvalOutcome::InputRequired(_)));
            cancel.cancel();
            expected = queued.eval("");
        }
        assert_eq!(buffers(&last), buffers(&expected));
    }
}

fn panicking_session() -> Session {
    let mut session = tracing_session();
    session.set_print_sink(rum_host::ClosurePrintSink::assert_bounded_destruction(
        |_| {
            panic!("output contract panic");
        },
    ));
    session
}

#[test]
fn synthesized_panic_has_no_buffers_and_preserves_published_prefix() {
    let source = r#"notice::before="before"; Message[notice::before]; InputString["first: "]; Print["panic"]"#;
    // Synthesized host failures return no checkpoint, unlike runtime failures.
    let mut host = panicking_session();
    let first = host.eval(source);
    let (_, messages, trace, _) = buffers(&first);
    assert_eq!(messages.len(), 1);
    assert!(!trace.is_empty());
    let last = host.resume_with_input("answer");
    let HostEvalOutcome::Failure(failure) = &last else {
        panic!("host failure")
    };
    assert_eq!(failure.code, HostFailureCode::Panic);
    assert_eq!(buffers(&last), (&[][..], &[][..], &[][..], &[][..]));
    println!("HOST panic: published message/trace prefix -> empty synthesized terminal buffers");

    let mut adapter = KernelAdapter::new(panicking_session());
    let initial = adapter.dispatch_request(request(source));
    let first = finish_adapter_execute(&mut adapter, initial);
    assert_eq!(first.stdin.len(), 1);
    assert_eq!(
        first
            .iopub
            .iter()
            .filter(|e| matches!(e.payload, KernelIoPubPayload::Stream(_)))
            .count(),
        1
    );
    assert_eq!(
        first
            .iopub
            .iter()
            .filter(|e| matches!(e.payload, KernelIoPubPayload::DisplayData(_)))
            .count(),
        1
    );
    adapter
        .handle_input_reply(input_reply(&first.stdin[0], "answer"))
        .unwrap()
        .unwrap();
    let last = wait_for_execute_completion(&mut adapter);
    assert_eq!(response(&last).outputs.len(), 1);
    assert_eq!(response(&last).error.as_ref().unwrap().error_name, "Panic");
    assert!(last.iopub.iter().all(|e| matches!(
        e.payload,
        KernelIoPubPayload::Error(_) | KernelIoPubPayload::Status(_)
    )));
    let recovered = adapter.execute(KernelExecuteRequest::new("2"));
    assert!(matches!(
        recovered.outcome,
        KernelExecuteOutcome::Success(_)
    ));
}
