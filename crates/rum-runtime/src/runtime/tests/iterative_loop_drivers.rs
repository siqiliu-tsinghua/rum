use super::*;
use std::time::Duration;

fn loop_driver_session() -> Session {
    let mut session = session_with_test_control_builtins();
    super::super::test_builtins::register_test_sameq_builtin(&mut session);
    session
}

fn assert_input_prompt(failure: &SessionRequestFailure, prompt: &str) {
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, prompt);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert!(failure.messages.is_empty());
}

#[test]
fn while_drives_one_hundred_thousand_iterations_without_native_stack_growth() {
    let mut session = loop_driver_session();

    let outcome = session
        .eval_text("i = 0; While[SameQ[Sign[100000 - i], 1], i++]; i")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "100000");
    assert!(outcome.messages.is_empty());
}

#[test]
fn for_drives_one_hundred_thousand_iterations_without_native_stack_growth() {
    let mut session = loop_driver_session();

    let outcome = session
        .eval_text("For[i = 0, SameQ[Sign[100000 - i], 1], i++, Null]; i")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "100000");
    assert!(outcome.messages.is_empty());
}

#[test]
fn until_drives_one_hundred_thousand_iterations_without_native_stack_growth() {
    let mut session = loop_driver_session();

    let outcome = session
        .eval_text("i = 0; Until[SameQ[i, 100000], i++]; i")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "100000");
    assert!(outcome.messages.is_empty());
}

#[test]
fn module_drives_thirty_thousand_initialized_bindings_without_native_stack_growth() {
    let binding_count = 30_000;
    let mut source = String::with_capacity(binding_count * 16 + 16);
    source.push_str("Module[{");
    for index in 0..binding_count {
        if index > 0 {
            source.push(',');
        }
        source.push_str("local");
        source.push_str(&index.to_string());
        source.push_str("=1");
    }
    source.push_str("}, 0]");

    let mut session = loop_driver_session();
    let outcome = session.eval_text(&source).unwrap();

    assert_eq!(outcome.value.to_string(), "0");
    assert!(outcome.messages.is_empty());
}

#[test]
fn while_resumes_mid_loop_without_replaying_the_body() {
    let mut session = loop_driver_session();

    let failure = session
        .eval_request(
            "i = 0; hits = 0; While[SameQ[Sign[4 - i], 1], If[SameQ[i, 2], CompoundExpression[hits++, InputString[\"while: \"]], Null]; i++]; {i, hits}",
        )
        .expect_err("While body should suspend midway through the loop");
    assert_input_prompt(&failure, "while: ");

    let response = session
        .resume_eval_request_with_input("ready")
        .expect("While should continue from the suspended body");

    assert_eq!(response.value.to_string(), "List[4, 1]");
    assert!(response.messages.is_empty());
}

#[test]
fn for_resumes_mid_loop_without_replaying_the_body() {
    let mut session = loop_driver_session();

    let failure = session
        .eval_request(
            "i = 0; hits = 0; For[i = 0, SameQ[Sign[4 - i], 1], i++, If[SameQ[i, 2], CompoundExpression[hits++, InputString[\"for: \"]], Null]]; {i, hits}",
        )
        .expect_err("For body should suspend midway through the loop");
    assert_input_prompt(&failure, "for: ");

    let response = session
        .resume_eval_request_with_input("ready")
        .expect("For should continue from the suspended body");

    assert_eq!(response.value.to_string(), "List[4, 1]");
    assert!(response.messages.is_empty());
}

#[test]
fn until_resumes_mid_loop_without_replaying_the_body() {
    let mut session = loop_driver_session();

    let failure = session
        .eval_request(
            "i = 0; hits = 0; Until[SameQ[i, 4], If[SameQ[i, 2], CompoundExpression[hits++, InputString[\"until: \"]], Null]; i++]; {i, hits}",
        )
        .expect_err("Until body should suspend midway through the loop");
    assert_input_prompt(&failure, "until: ");

    let response = session
        .resume_eval_request_with_input("ready")
        .expect("Until should continue from the suspended body");

    assert_eq!(response.value.to_string(), "List[4, 1]");
    assert!(response.messages.is_empty());
}

#[test]
fn module_resumes_mid_initializer_without_replaying_bindings() {
    let mut session = loop_driver_session();

    let failure = session
        .eval_request(
            "hits = 0; Module[{a = hits++, b = CompoundExpression[hits++, InputString[\"module: \"]], c = hits++}, {a, b, c, hits}]",
        )
        .expect_err("Module initializer should suspend midway through the binding list");
    assert_input_prompt(&failure, "module: ");

    let response = session
        .resume_eval_request_with_input("ready")
        .expect("Module should continue from the suspended initializer");

    assert_eq!(response.value.to_string(), "List[0, \"ready\", 2, 3]");
    assert!(response.messages.is_empty());
}

#[test]
fn table_iteration_accumulator_completes_after_multiple_input_suspensions() {
    let mut session = session_with_test_control_builtins();

    let failure = session.eval_request("Table[InputString[], 2]").unwrap_err();
    assert_input_prompt(&failure, "");
    let failure = session.resume_eval_request_with_input("0").unwrap_err();
    assert_input_prompt(&failure, "");

    let response = session.resume_eval_request_with_input("1").unwrap();
    assert_eq!(response.value.to_string(), "List[\"0\", \"1\"]");
    assert!(response.messages.is_empty());
}

#[test]
fn sum_iteration_accumulator_completes_after_multiple_input_suspensions() {
    let mut session = session_with_test_control_builtins();

    let failure = session.eval_request("Sum[Input[], {i, 2}]").unwrap_err();
    assert_input_prompt(&failure, "?");
    let failure = session.resume_eval_request_with_input("2").unwrap_err();
    assert_input_prompt(&failure, "?");

    let response = session.resume_eval_request_with_input("3").unwrap();
    assert_eq!(response.value.to_string(), "5");
    assert!(response.messages.is_empty());
}

#[test]
fn product_iteration_accumulator_completes_after_multiple_input_suspensions() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("Product[Input[], {i, 2}]")
        .unwrap_err();
    assert_input_prompt(&failure, "?");
    let failure = session.resume_eval_request_with_input("2").unwrap_err();
    assert_input_prompt(&failure, "?");

    let response = session.resume_eval_request_with_input("3").unwrap();
    assert_eq!(response.value.to_string(), "6");
    assert!(response.messages.is_empty());
}

#[test]
fn iteration_accumulator_preserves_unsuspended_values() {
    for (source, expected) in [
        ("Table[InputString[], 2]", "List[\"2\", \"3\"]"),
        ("Sum[Input[], {i, 2}]", "5"),
        ("Product[Input[], {i, 2}]", "6"),
    ] {
        let mut session = session_with_test_control_builtins();
        session.queue_input_response("2");
        session.queue_input_response("3");

        let response = session.eval_request(source).unwrap();
        assert_eq!(response.value.to_string(), expected, "{source}");
        assert!(response.messages.is_empty());
    }
}

#[test]
fn iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values() {
    for (source, expected) in [
        (
            "Table[If[SameQ[i, 2], Input[], i], {i, 3}]",
            "List[1, 5, 3]",
        ),
        ("Sum[If[SameQ[i, 2], Input[], i], {i, 3}]", "9"),
        ("Product[If[SameQ[i, 2], Input[], i], {i, 3}]", "15"),
    ] {
        let mut session = loop_driver_session();
        let failure = session.eval_request(source).unwrap_err();
        assert_input_prompt(&failure, "?");

        let response = session.resume_eval_request_with_input("5").unwrap();
        assert_eq!(response.value.to_string(), expected, "{source}");
        assert!(response.messages.is_empty());
    }
}

#[test]
fn unbounded_while_reaches_the_existing_eval_deadline() {
    let mut session = loop_driver_session();
    session.set_eval_timeout(Some(Duration::from_millis(0)));

    let response = session
        .eval_request("While[True, 1]")
        .expect("the armed evaluation deadline should stop an unbounded While");

    assert_eq!(
        response.value.to_string(),
        "TerminatedEvaluation[\"TimeConstraint\"]"
    );
    assert_eq!(
        response.termination,
        Some(EvalTerminationReason::TimeConstraint)
    );
    assert!(
        response
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "timelim")
    );
}
