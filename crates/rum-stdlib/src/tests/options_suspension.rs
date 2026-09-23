use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn assert_prompt(error: &SessionRequestError, prompt: &str) {
    match error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, prompt),
        other => panic!("expected actual input suspension, got {other:?}"),
    }
}

fn assert_queued_and_resumed(source: &str, inputs: &[(&str, &str)], expected: &str) {
    assert!(!inputs.is_empty(), "every certificate must suspend");
    let mut queued = core_session();
    for (_, input) in inputs {
        queued.queue_input_response(*input);
    }
    let expected_value = queued.eval_request(source).unwrap().value;
    assert_eq!(expected_value.to_string(), expected, "queued: {source}");

    let mut suspended = core_session();
    let mut result = suspended.eval_request(source);
    for (prompt, input) in inputs {
        let failure = result.expect_err("each option input must suspend");
        assert_prompt(&failure.error, prompt);
        result = suspended.resume_eval_request_with_input(*input);
    }
    assert_eq!(result.unwrap().value, expected_value, "resumed: {source}");
}

#[test]
fn delayed_string_replace_option_resumes_into_consumer() {
    // IgnoreCase makes the documented replacement observable: "A" becomes "b".
    let source =
        r#"StringReplace["A", "a" -> "b", IgnoreCase :> SameQ[InputString["case: "], "yes"]]"#;
    let mut queued = core_session();
    queued.queue_input_response("yes");
    let expected = queued.eval_request(source).unwrap().value;
    assert_eq!(expected.to_string(), r#""b""#);

    let mut suspended = core_session();
    let failure = suspended
        .eval_request(source)
        .expect_err("option must suspend");
    match failure.error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, "case: "),
        other => panic!("expected actual input suspension, got {other:?}"),
    }
    let actual = suspended
        .resume_eval_request_with_input("yes")
        .unwrap()
        .value;
    println!("queued={expected}; resumed={actual}");
    assert_eq!(actual, expected);
}

#[test]
fn string_option_consumers_finish_after_actual_suspension() {
    for (symbol, arguments, expected) in [
        (
            "StringPosition",
            r#""aA", "a""#,
            "List[List[1, 1], List[2, 2]]",
        ),
        ("StringSplit", r#""aXbxc", "x""#, r#"List["a", "b", "c"]"#),
        ("StringReplace", r#""A", "a" -> "b""#, r#""b""#),
        ("StringCases", r#""AbA", "a""#, r#"List["A", "A"]"#),
        ("StringCount", r#""Aa", "a""#, "2"),
        ("StringDelete", r#""AbA", "a""#, r#""b""#),
        ("StringContainsQ", r#"{"A", "b"}, "a""#, "List[True, False]"),
        ("StringStartsQ", r#"{"Ab", "cb"}, "a""#, "List[True, False]"),
        ("StringEndsQ", r#"{"bA", "bc"}, "a""#, "List[True, False]"),
        ("StringMatchQ", r#"{"A", "b"}, "a""#, "List[True, False]"),
        ("StringFreeQ", r#"{"A", "b"}, "a""#, "List[False, True]"),
    ] {
        for default in [false, true] {
            let option = r#"IgnoreCase :> SameQ[InputString["case: "], "yes"]"#;
            let source = if default {
                format!(
                    "Unprotect[{symbol}]; SetOptions[{symbol}, {option}]; {symbol}[{arguments}]"
                )
            } else {
                format!("{symbol}[{arguments}, {option}]")
            };
            assert_queued_and_resumed(&source, &[("case: ", "yes")], expected);
        }
        println!("consumer={symbol}; queued=resumed={expected}; default+explicit suspended");
    }
}

#[test]
fn option_loop_preserves_prefix_and_side_effect_order_across_repeated_suspensions() {
    let source = r#"
        seen = {};
        Unprotect[StringReplace];
        SetOptions[StringReplace, IgnoreCase :> (
            AppendTo[seen, defaultBefore]; InputString["default: "];
            AppendTo[seen, defaultAfter]; False
        )];
        result = StringReplace["AA", "a" -> "b", 1, IgnoreCase :> (
            AppendTo[seen, explicitBefore]; InputString["explicit: "];
            AppendTo[seen, explicitBetween]; InputString["again: "];
            AppendTo[seen, explicitAfter]; True
        )];
        {result, seen}
    "#;
    assert_queued_and_resumed(
        source,
        &[
            ("default: ", "one"),
            ("explicit: ", "two"),
            ("again: ", "three"),
        ],
        r#"List["bA", List[defaultBefore, defaultAfter, explicitBefore, explicitBetween, explicitAfter]]"#,
    );
}

#[test]
fn option_consumer_can_suspend_again_in_delayed_replacement() {
    let source = r#"StringReplace["A", "a" :> InputString["replacement: "],
        IgnoreCase :> SameQ[InputString["case: "], "yes"]]"#;
    assert_queued_and_resumed(
        source,
        &[("case: ", "yes"), ("replacement: ", "b")],
        r#""b""#,
    );
}

#[test]
fn nested_option_consumers_resume_into_their_own_calls() {
    let source = r#"StringReplace["A", "a" -> "b", IgnoreCase :>
        SameQ[StringReplace["X", "x" -> "yes", IgnoreCase :>
            SameQ[InputString["nested: "], "yes"]], "yes"]]"#;
    assert_queued_and_resumed(source, &[("nested: ", "yes")], r#""b""#);
}

#[test]
fn option_suspension_cancellation_abandons_remaining_guest_work() {
    let mut session = core_session();
    let accepted = session.accept_cancellable_request().unwrap();
    let cancel = accepted.cancel_handle();
    let failure = session
        .eval_accepted_request(
            accepted,
            r#"
        seen = {};
        StringReplace["A", "a" :> (AppendTo[seen, replacement]; "b"),
            IgnoreCase :> (AppendTo[seen, before]; InputString["cancel: "];
                AppendTo[seen, after]; True)]
    "#,
        )
        .expect_err("option must suspend before cancellation");
    assert_prompt(&failure.error, "cancel: ");
    cancel.cancel();
    let response = session.resume_eval_request_with_input("yes").unwrap();
    assert_eq!(
        response.termination,
        Some(rum_runtime::EvalTerminationReason::Cancelled)
    );
    assert!(session.pending_input_request().is_none());
    assert_eq!(
        session
            .eval_request("{seen, 1 + 1}")
            .unwrap()
            .value
            .to_string(),
        "List[List[before], 2]"
    );
}

#[test]
fn option_error_after_resume_propagates_without_running_the_consumer() {
    let source = r#"seen = {}; result = Catch[
        StringReplace["A", "a" :> (AppendTo[seen, replacement]; "b"),
            IgnoreCase :> (InputString["throw: "]; Throw[stopped])]];
        {result, seen}"#;
    assert_queued_and_resumed(source, &[("throw: ", "yes")], "List[stopped, List[]]");
}

#[path = "options_suspension/remaining.rs"]
mod remaining;

#[path = "options_suspension/inventory.rs"]
mod inventory;
