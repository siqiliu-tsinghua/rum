use rum_host::{HostEvalOutcome, HostFailureCode, HostInputKind, HostSession};

// Pins malformed Input answer provenance in docs/host-embedding-and-hardening.md.
#[test]
fn input_syntax_failure_has_the_same_rendering_when_queued_or_resumed() {
    // The depth-limit case needs 3 MiB in debug builds; allow more than twice that.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let deep = format!("{}1{}", "(".repeat(129), ")".repeat(129));
            for answer in ["(", "1 ! 2", "", "\"é\" + )", "f[", deep.as_str()] {
                let mut queued = HostSession::new();
                queued.queue_input_response(answer);
                let HostEvalOutcome::Failure(queued_failure) = queued.eval("Input[\"answer\"]")
                else {
                    panic!("malformed queued Input should fail for {answer:?}");
                };

                let mut resumed = HostSession::new();
                let HostEvalOutcome::InputRequired(pause) = resumed.eval("Input[\"answer\"]")
                else {
                    panic!("Input should suspend without an answer");
                };
                assert_eq!(pause.request.kind, HostInputKind::Input);
                assert_eq!(pause.request.prompt, "answer");
                let HostEvalOutcome::Failure(resumed_failure) = resumed.resume_with_input(answer)
                else {
                    panic!("malformed resumed Input should fail for {answer:?}");
                };

                assert_eq!(queued_failure.code, HostFailureCode::Syntax);
                assert_eq!(resumed_failure.code, HostFailureCode::Syntax);
                assert!(
                    queued_failure
                        .syntax_rendered
                        .as_deref()
                        .is_some_and(|text| !text.is_empty())
                );
                assert_eq!(
                    queued_failure.syntax_rendered,
                    resumed_failure.syntax_rendered
                );
                assert_eq!(queued_failure, resumed_failure, "answer: {answer:?}");
                for session in [&mut queued, &mut resumed] {
                    assert!(session.pending_input_request().is_none());
                    let HostEvalOutcome::Success(fresh) = session.eval("83") else {
                        panic!("a fresh request should succeed after a malformed Input answer");
                    };
                    assert_eq!(fresh.value.text, "83");
                }
            }
        })
        .expect("spawn Input syntax contract test")
        .join()
        .expect("Input syntax contract test panicked");
}
