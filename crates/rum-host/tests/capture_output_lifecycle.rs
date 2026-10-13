//! The embedding guide's provisional-output and exclusive PrintSink contract.
use rum_host::{
    ClosurePrintSink, HostEvalOutcome, HostEvalSuccess, HostInputSuspension, HostSession,
    HostTerminationReason,
};
use std::sync::{Arc, Mutex};

fn success(out: HostEvalOutcome) -> HostEvalSuccess {
    let HostEvalOutcome::Success(s) = out else {
        panic!("expected success: {out:?}")
    };
    s
}
fn pause(out: HostEvalOutcome, prompt: &str) -> HostInputSuspension {
    let HostEvalOutcome::InputRequired(s) = out else {
        panic!("expected pause: {out:?}")
    };
    assert_eq!(s.request.prompt, prompt);
    s
}
fn new() -> HostSession {
    let mut s = HostSession::new();
    success(s.eval(r#"probe::tag="capture";later=0"#));
    s
}
fn cases() -> Vec<(&'static str, String, bool, bool)> {
    let body = r#"(Print["one"];Message[probe::tag];1+2;InputString["first"];Print["two"];Message[probe::tag];4+5;InputString["second"];Print["three"];Message[probe::tag];later=1;7)"#;
    vec![
        (
            "EvaluationData",
            format!("EvaluationData[{body},\"Result\"]"),
            true,
            true,
        ),
        (
            "TestReport",
            format!(
                "TestReport[{{TestCreate[{body},7,{{ExpectedMessages->{{probe::tag,probe::tag,probe::tag}}}}]}},\"TestsSucceededCount\"]"
            ),
            true,
            true,
        ),
        ("Check", format!("Check[{body},11]"), false, true),
        ("Quiet", format!("Quiet[{body}]"), false, true),
        (
            "ConfirmQuiet",
            format!("Enclose[ConfirmQuiet[{body}],Function[11]]"),
            false,
            true,
        ),
        ("Trace", format!("Trace[{body}];7"), false, false),
        (
            "TraceDetail",
            format!("TraceDetail[{body}];7"),
            false,
            false,
        ),
        ("TracePrint", format!("TracePrint[{body}];7"), false, false),
    ]
}
fn source(capture: &str) -> String {
    format!("Print[\"outside\"];Message[probe::tag];result=({capture});Print[\"after\"];result")
}
fn assert_pause(
    p: &HostInputSuspension,
    n: usize,
    owns_prints: bool,
    owns_messages: bool,
    streaming: bool,
) {
    let expected = if streaming {
        vec![]
    } else if owns_prints {
        vec!["outside"]
    } else if n == 0 {
        vec!["outside", "one"]
    } else {
        vec!["outside", "one", "two"]
    };
    assert_eq!(p.prints, expected);
    assert_eq!(p.messages.len(), if owns_messages { 1 } else { n + 2 });
    assert!(p.trace.is_empty());
}
#[test]
fn queued_and_resumed_output_and_streaming_transcripts_match() {
    for (name, capture, owns_prints, owns_messages) in cases() {
        for streaming in [false, true] {
            let mut completed = Vec::new();
            let mut transcripts = Vec::new();
            for queued in [false, true] {
                let mut s = new();
                let delivered = Arc::new(Mutex::new(Vec::<String>::new()));
                if streaming {
                    let sink = delivered.clone();
                    s.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
                        sink.lock().unwrap().push(line.into())
                    }));
                }
                let out = if queued {
                    s.queue_input_response("first");
                    s.queue_input_response("second");
                    s.eval(source(&capture).as_str())
                } else {
                    let first = pause(s.eval(source(&capture).as_str()), "first");
                    assert_pause(&first, 0, owns_prints, owns_messages, streaming);
                    assert!(matches!(s.eval("later=99"), HostEvalOutcome::Failure(_)));
                    let second = pause(s.resume_with_input("first"), "second");
                    assert_pause(&second, 1, owns_prints, owns_messages, streaming);
                    assert!(second.messages.starts_with(&first.messages));
                    assert!(second.prints.starts_with(&first.prints));
                    s.resume_with_input("second")
                };
                let out = success(out);
                if streaming {
                    assert!(out.prints.is_empty());
                }
                assert!(out.trace.is_empty());
                assert!(s.pending_input_request().is_none());
                assert_eq!(success(s.eval("later")).value.text, "1");
                assert_eq!(success(s.eval("1+1")).value.text, "2");
                completed.push(out);
                transcripts.push(delivered.lock().unwrap().clone());
            }
            assert_eq!(completed[0], completed[1], "{name} streaming={streaming}");
            assert_eq!(transcripts[0], transcripts[1], "{name} streamed lines");
            if streaming {
                assert!(transcripts[0].starts_with(&[
                    "outside".into(),
                    "one".into(),
                    "two".into(),
                    "three".into()
                ]));
            }
        }
    }
}
#[test]
fn cancel_and_reset_at_both_pauses_keep_only_public_buffered_output() {
    for (name, capture, owns_prints, owns_messages) in cases() {
        for stop in [0, 1] {
            for reset in [false, true] {
                let mut s = new();
                let cancel = s.cancel_handle();
                let first = pause(s.eval(source(&capture).as_str()), "first");
                assert_pause(&first, 0, owns_prints, owns_messages, false);
                let last = if stop == 1 {
                    let second = pause(s.resume_with_input("first"), "second");
                    assert_pause(&second, 1, owns_prints, owns_messages, false);
                    second
                } else {
                    first
                };
                if reset {
                    s.queue_input_response("discard with reset");
                    s.reset();
                    cancel.cancel();
                    assert!(s.pending_input_request().is_none());
                } else {
                    cancel.cancel();
                    let cancelled = success(s.resume_with_input("not delivered"));
                    assert_eq!(
                        cancelled.termination,
                        Some(HostTerminationReason::Cancelled)
                    );
                    assert_eq!(cancelled.prints, last.prints, "{name} {stop}");
                    assert_eq!(cancelled.messages.len(), last.messages.len() + 1);
                    assert!(cancelled.messages.starts_with(&last.messages));
                    assert!(cancelled.trace.is_empty());
                    assert_eq!(success(s.eval("later")).value.text, "0");
                }
                let fresh = success(s.eval("1+1"));
                assert_eq!(fresh.value.text, "2");
                assert!(fresh.trace.is_empty());
                assert!(s.pending_input_request().is_none());
            }
        }
    }
}
#[test]
fn streamed_prints_never_enter_evaluation_data_output_log() {
    let capture = r#"EvaluationData[(Print["one"];InputString["first"];Print["two"];InputString["second"];Print["three"]),"OutputLog"]"#;
    for queued in [false, true] {
        let mut s = new();
        let delivered = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = delivered.clone();
        s.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
            sink.lock().unwrap().push(line.into())
        }));
        let out = if queued {
            s.queue_input_response("a");
            s.queue_input_response("b");
            s.eval(capture)
        } else {
            let first = pause(s.eval(capture), "first");
            assert!(first.prints.is_empty());
            assert_eq!(*delivered.lock().unwrap(), vec!["one"]);
            let second = pause(s.resume_with_input("a"), "second");
            assert!(second.prints.is_empty());
            assert_eq!(*delivered.lock().unwrap(), vec!["one", "two"]);
            s.resume_with_input("b")
        };
        let out = success(out);
        assert_eq!(out.value.text, "List[]");
        assert!(out.prints.is_empty());
        assert_eq!(*delivered.lock().unwrap(), vec!["one", "two", "three"]);
    }
}
