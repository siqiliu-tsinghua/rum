use super::*;

#[test]
fn pause_keeps_raw_capture_buffers_and_handled_errors_for_nested_observers() {
    let mut s = session_with_test_control_builtins();
    s.eval_request(r#"probe::tag="captured""#).unwrap();
    let first=s.eval_request(r#"Print["outside"];EvaluationData[(Message[probe::tag];Print["inside"];1+2;InputString["one"];InputString["two"];7),"OutputLog"]"#).unwrap_err();
    assert_eq!(first.prints, vec!["outside"]);
    assert!(first.messages.is_empty());
    assert!(first.trace.is_empty());
    assert_eq!(s.prints, vec!["outside", "inside"]);
    assert_eq!(s.messages.len(), 1);
    assert!(!s.trace_events.is_empty());
    let raw_trace = s.trace_events.clone();
    // The public boundary query is metadata-only, including repeated reads.
    let first_boundary = s.pending_output_boundary();
    let second_boundary = s.pending_output_boundary();
    assert_eq!(first_boundary.prints, second_boundary.prints);
    assert_eq!(s.trace_events, raw_trace);
    let error = EvalError::IterationLimitExceeded { limit: 17 };
    s.emit_non_loader_eval_message(0, &error);
    let handled = s.handled_non_loader_eval_errors.len();
    assert!(handled > 0);
    let second = s.resume_eval_request_with_input("one").unwrap_err();
    assert_eq!(second.prints, first.prints);
    assert_eq!(second.messages, first.messages);
    assert_eq!(second.trace, first.trace);
    assert_eq!(s.handled_non_loader_eval_errors.len(), handled);
    let done = s.resume_eval_request_with_input("two").unwrap();
    assert_eq!(done.value.to_string(), r#"List["inside"]"#);
    assert_eq!(done.prints, vec!["outside"]);
    assert!(done.messages.is_empty());
    assert!(done.trace.is_empty());
    assert!(s.handled_non_loader_eval_errors.is_empty());
}

#[test]
fn nested_capture_observes_raw_messages_but_cannot_publish_outer_output() {
    let mut s = session_with_test_control_builtins();
    s.eval_request(r#"probe::tag="captured""#).unwrap();
    let first=s.eval_request(r#"EvaluationData[(Print["outer"];Check[(Message[probe::tag];InputString["inner"];7),11];Print["between"];InputString["outer"];13),{"Result","OutputLog","MessagesText"}]"#).unwrap_err();
    assert!(first.prints.is_empty());
    assert!(first.messages.is_empty());
    assert!(first.trace.is_empty());
    let second = s.resume_eval_request_with_input("inner").unwrap_err();
    assert!(second.prints.is_empty());
    assert!(second.messages.is_empty());
    assert!(second.trace.is_empty());
    assert_eq!(s.prints, vec!["outer", "between"]);
    assert_eq!(s.messages.len(), 1);
    let done = s.resume_eval_request_with_input("outer").unwrap();
    assert_eq!(
        done.value.to_string(),
        r#"Association[Rule["Result", 13], Rule["OutputLog", List["outer", "between"]], Rule["MessagesText", List["captured"]]]"#
    );
    assert!(done.prints.is_empty());
    assert!(done.messages.is_empty());
    assert!(done.trace.is_empty());
}

#[test]
fn abandonment_rolls_back_all_declared_channels_with_existing_tracing() {
    for capture in [
        r#"EvaluationData[(Print["inside"];Message[probe::tag];1+2;InputString["one"];4+5;InputString["two"];later=1)]"#,
        r#"TestReport[{TestCreate[(Print["inside"];Message[probe::tag];1+2;InputString["one"];4+5;InputString["two"];later=1),7]}]"#,
        r#"Trace[(1+2;InputString["one"];4+5;InputString["two"];later=1)]"#,
        r#"TraceDetail[(1+2;InputString["one"];4+5;InputString["two"];later=1)]"#,
        r#"TracePrint[(1+2;InputString["one"];4+5;InputString["two"];later=1)]"#,
    ] {
        for second in [false, true] {
            let mut s = session_with_test_control_builtins();
            s.eval_request(r#"probe::tag="captured";later=0"#).unwrap();
            s.tracing_enabled = true;
            let out = s
                .eval_request(&format!(
                    "Print[\"outside\"];Message[probe::tag];2+3;{capture}"
                ))
                .unwrap_err();
            assert_eq!(out.prints, vec!["outside"]);
            assert_eq!(out.messages.len(), 1);
            assert!(!out.trace.is_empty());
            let boundary = s.pending_output_boundary();
            let public_trace = out.trace.clone();
            if second {
                let again = s.resume_eval_request_with_input("one").unwrap_err();
                assert_eq!(again.trace, public_trace);
            }
            assert!(s.trace_events.len() > boundary.trace.unwrap());
            abandon(&mut s, AbandonReason::CancelledRequest);
            assert!(s.tracing_enabled);
            assert_eq!(s.prints, vec!["outside"]);
            assert_eq!(s.messages.len(), 1);
            assert!(s.trace_events.ends_with(&public_trace));
            assert_eq!(s.trace_events.len(), boundary.trace.unwrap());
            assert_eq!(s.trace_event_depths.len(), s.trace_events.len());
            assert_eq!(value(&mut s, "later"), "0");
        }
    }
}

#[test]
fn abandonment_discards_messages_without_running_fallback_or_confirmation_info() {
    for capture in [
        r#"Check[(Message[probe::tag];Print["inside"];InputString["one"];Message[probe::tag];InputString["two"];7),later=1]"#,
        r#"Quiet[(Message[probe::tag];Print["inside"];InputString["one"];Message[probe::tag];InputString["two"];7)]"#,
        r#"ConfirmQuiet[(Message[probe::tag];Print["inside"];InputString["one"];Message[probe::tag];InputString["two"];7),All,later=1]"#,
    ] {
        for second in [false, true] {
            let mut s = session_with_test_control_builtins();
            s.eval_request(r#"probe::tag="capture";later=0"#).unwrap();
            let first = s
                .eval_request(&format!("Message[probe::tag];Print[\"outside\"];{capture}"))
                .unwrap_err();
            assert_eq!(first.prints, vec!["outside", "inside"]);
            assert_eq!(first.messages.len(), 1);
            if second {
                let next = s.resume_eval_request_with_input("one").unwrap_err();
                assert_eq!(next.messages, first.messages);
            }
            abandon(&mut s, AbandonReason::CancelledRequest);
            assert_eq!(s.messages.len(), 1);
            assert_eq!(s.prints, vec!["outside", "inside"]);
            assert_eq!(value(&mut s, "later"), "0");
        }
    }
}

#[test]
fn completion_releases_trace_prefix_when_tracing_was_already_enabled() {
    for source in [
        r#"2+3;EvaluationData[(1+2;InputString["one"];4+5;InputString["two"];6+7),"Result"]"#,
        r#"2+3;Trace[(1+2;InputString["one"];4+5;InputString["two"];6+7)]"#,
        r#"2+3;TestReport[{TestCreate[(1+2;InputString["one"];4+5;InputString["two"];6+7),13]}]"#,
    ] {
        let mut s = session_with_test_control_builtins();
        s.enable_tracing(true);
        let first = s.eval_request(source).unwrap_err();
        assert!(!first.trace.is_empty());
        let second = s.resume_eval_request_with_input("one").unwrap_err();
        assert_eq!(second.trace, first.trace);
        let raw = s.trace_events.clone();
        assert!(raw.len() > second.trace.len());
        let done = s.resume_eval_request_with_input("two").unwrap();
        assert!(done.trace.starts_with(&raw));
        assert!(s.tracing_enabled);
    }
}

#[test]
fn a_handled_guest_error_remains_handled_across_two_pauses() {
    let mut s = session_with_test_control_builtins();
    s.set_replace_repeated_limit(8);
    let first=s.eval_request(r#"Quiet[(Check[1 //. {1->2,2->1},7,"General::iterlim"];InputString["one"];InputString["two"];7)]"#).unwrap_err();
    assert!(first.messages.is_empty());
    let handled = s.handled_non_loader_eval_errors.len();
    assert!(handled > 0);
    let second = s.resume_eval_request_with_input("one").unwrap_err();
    assert!(second.messages.is_empty());
    assert_eq!(s.handled_non_loader_eval_errors.len(), handled);
    let done = s.resume_eval_request_with_input("two").unwrap();
    assert_eq!(done.value.to_string(), "7");
    assert!(done.messages.is_empty());
    assert!(s.handled_non_loader_eval_errors.is_empty());
}
