//! Once-only delivery by output position, as documented on KernelAdapter.
use super::*;
mod host_buffers;
mod support;
use support::*;

// Promoted from brief-552: keep the exact aggregate stream comparison with
// queued host prints, including its final-host-output control.
#[test]
fn red_worker_streams_match_queued_input() {
    let nested = NestedGet::new();
    for (source, answers) in [
        (nested.source.as_str(), vec!["raw"]),
        (
            "Print[\"prefix\"]; a=Input[\"first: \"]; Print[\"middle\"]; b=InputString[\"second: \"]; {a,b}",
            vec!["6*7", "literal"],
        ),
        (
            "InputString[\"secret: \",FieldMasked->True]",
            vec!["secret"],
        ),
    ] {
        let mut queued = Session::new();
        eval_text(&mut queued, "before=0; after=0");
        for answer in &answers {
            queued.queue_input_response(*answer);
        }
        let HostEvalOutcome::Success(expected) = queued.eval(source) else {
            panic!("queued success")
        };
        let mut session = Session::new();
        eval_text(&mut session, "before=0; after=0");
        let mut adapter = KernelAdapter::new(session);
        let initial = adapter.dispatch_request(request(source));
        let mut done = finish_adapter_execute(&mut adapter, initial);
        let mut streams = String::new();
        for answer in answers {
            assert!(done.reply.is_none());
            assert_eq!(done.stdin.len(), 1, "must actually suspend");
            assert!(adapter.awaiting_stdin());
            assert_eq!(adapter.execution_state(), KernelExecutionState::Busy);
            for out in &done.iopub {
                if let KernelIoPubPayload::Stream(s) = &out.payload {
                    streams.push_str(&s.text);
                }
            }
            adapter
                .handle_input_reply(input_reply(&done.stdin[0], answer))
                .unwrap()
                .unwrap();
            done = wait_for_execute_completion(&mut adapter);
        }
        for out in &done.iopub {
            if let KernelIoPubPayload::Stream(s) = &out.payload {
                streams.push_str(&s.text);
            }
        }
        let result = response(&done);
        let KernelExecuteOutcome::Success(actual) = &result.outcome else {
            panic!("success")
        };
        assert_eq!(actual.value.text, expected.value.text);
        assert_eq!(
            actual.prints, expected.prints,
            "final host outputs match the queued oracle"
        );
        assert_eq!(result.status, KernelReplyStatus::Ok);
        let expected_streams = expected
            .prints
            .iter()
            .map(|p| format!("{p}\n"))
            .collect::<String>();
        assert_eq!(streams, expected_streams);
    }
}

#[test]
fn one_prompt_outputs_match_queued_input() {
    let source = r#"Print["prefix"]; InputString["first: "]; Print["tail"]; 42"#;
    let expected = queued(source, &["answer"], false);
    let mut run = Run::start(source, Session::new());
    run.at_prompt("first: ", &["prefix\n"], &[], false);
    run.answer("answer");
    run.assert_done(&expected);
}

#[test]
fn two_prompts_outputs_match_queued_input() {
    let source = r#"Print["prefix"]; InputString["first: "]; Print["middle"]; InputString["second: "]; Print["tail"]; 42"#;
    let expected = queued(source, &["one", "two"], true);
    let mut run = Run::start(source, tracing_session());
    run.at_prompt("first: ", &["prefix\n"], &[], true);
    run.answer("one");
    run.at_prompt("second: ", &["middle\n"], &[], true);
    run.answer("two");
    run.assert_done(&expected);
}

#[test]
fn nested_get_outputs_match_queued_input() {
    let nested = NestedGet::new();
    let expected = queued(&nested.source, &["answer"], true);
    let mut run = Run::start(&nested.source, tracing_session());
    run.at_prompt("nested: ", &["outer-before\n", "inner-before\n"], &[], true);
    run.answer("answer");
    run.assert_done(&expected);
}

#[test]
fn equal_prints_keep_every_occurrence() {
    let source =
        r#"Print["same"]; Print["same"]; InputString["first: "]; Print["same"]; Print["same"]; 42"#;
    let expected = queued(source, &["answer"], false);
    assert_eq!(
        Effects::from_outputs(expected.outputs.clone()).stdout,
        ["same\n"; 4]
    );
    let mut run = Run::start(source, Session::new());
    run.at_prompt("first: ", &["same\n", "same\n"], &[], false);
    run.answer("answer");
    run.assert_done(&expected);
}

#[test]
fn messages_before_and_after_prompt_match_queued_input() {
    let source = r#"notice::same="same"; Message[notice::same]; InputString["first: "]; Message[notice::same]; 42"#;
    let expected = queued(source, &["answer"], false);
    let mut run = Run::start(source, Session::new());
    run.at_prompt("first: ", &[], &["notice::same: same\n"], false);
    run.answer("answer");
    run.assert_done(&expected);
}

#[test]
fn trace_events_match_queued_input() {
    let source = r#"a=1; InputString["first: "]; a=2; InputString["second: "]; a+1"#;
    let expected = queued(source, &["one", "two"], true);
    let mut run = Run::start(source, tracing_session());
    run.at_prompt("first: ", &[], &[], true);
    run.answer("one");
    run.at_prompt("second: ", &[], &[], true);
    run.answer("two");
    run.assert_done(&expected);
}

#[test]
fn failure_after_resume_matches_queued_input() {
    let source = r#"notice::same="same"; Print["prefix"]; Message[notice::same]; InputString["first: "]; Print["tail"]; Message[notice::same]; Exit[7]"#;
    let expected = queued(source, &["answer"], true);
    assert_eq!(expected.error.as_ref().unwrap().error_name, "Exit");
    let mut run = Run::start(source, tracing_session());
    run.at_prompt("first: ", &["prefix\n"], &["notice::same: same\n"], true);
    run.answer("answer");
    run.assert_done(&expected);
}

#[test]
fn interrupt_after_resume_matches_queued_input() {
    let source = r#"notice::same="same"; Print["prefix"]; Message[notice::same]; InputString["first: "]; Print["middle"]; Message[notice::same]; WithCleanup[InputString["second: "]; Print["unreachable"], Print["cleanup"]; Message[notice::same]]"#;
    // Queue the supplied answer; both paths cancel at the same second prompt.
    // Render the queued path's cumulative terminal host outcome once.
    let mut session = tracing_session();
    session.queue_input_response("one");
    let cancel = session.cancel_handle();
    assert!(matches!(
        session.eval(source),
        HostEvalOutcome::InputRequired(_)
    ));
    cancel.cancel();
    let expected = KernelAdapter::new(session).execute(KernelExecuteRequest::new(""));
    assert_eq!(expected.error.as_ref().unwrap().error_name, "Cancelled");
    let mut run = Run::start(source, tracing_session());
    run.at_prompt("first: ", &["prefix\n"], &["notice::same: same\n"], true);
    run.answer("one");
    run.at_prompt("second: ", &["middle\n"], &["notice::same: same\n"], true);
    run.interrupt();
    run.assert_done(&expected);
}

#[test]
fn cross_family_order_is_grouped_per_segment() {
    let source = r#"notice::before="before"; Message[notice::before]; InputString["first: "]; Print["tail"]; 42"#;
    let expected = queued(source, &["answer"], true);
    let mut run = Run::start(source, tracing_session());
    run.at_prompt("first: ", &[], &["notice::before: before\n"], true);
    run.answer("answer");
    run.assert_done(&expected);
}
