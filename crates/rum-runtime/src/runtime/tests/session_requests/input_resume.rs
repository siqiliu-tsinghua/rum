use super::*;

#[test]
fn queued_input_responses_feed_inputstring_without_prompting_again() {
    let mut session = session_with_test_control_builtins();
    session.queue_input_response("Alice");

    let response = session
        .eval_request("InputString[\"name: \"]")
        .expect("queued host input should satisfy InputString");

    assert_eq!(response.value.to_string(), "\"Alice\"");
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_preserves_already_evaluated_prefix_arguments() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("x = seed").unwrap();

    let failure = session
        .eval_request("List[x = List[x], InputString[\"name: \"], x = List[x]]")
        .expect_err("InputString should suspend the request");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue from the suspended argument list");

    assert_eq!(
        response.value.to_string(),
        "List[List[seed], \"Alice\", List[List[seed]]]"
    );
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[List[seed]]"
    );
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_can_suspend_multiple_times_without_restarting() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("x = seed").unwrap();

    let first = session
        .eval_request(
            "List[x = List[x], InputString[\"first: \"], x = List[x], InputString[\"second: \"], x = List[x]]",
        )
        .expect_err("first InputString should suspend the request");

    match &first.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "first: ");
        }
        other => panic!("expected first input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("Alice")
        .expect_err("second InputString should suspend the resumed request");

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "second: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Bob")
        .expect("second resume should finish the original request");

    assert_eq!(
        response.value.to_string(),
        "List[List[seed], \"Alice\", List[List[seed]], \"Bob\", List[List[List[seed]]]]"
    );
    assert_eq!(
        session.eval_text("x").unwrap().value.to_string(),
        "List[List[List[seed]]]"
    );
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_continues_symbol_evaluation_after_fixed_point_suspend() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("name := InputString[\"name: \"]")
        .unwrap();
    session.eval_text("g[\"Alice\"] := done").unwrap();

    let failure = session
        .eval_request("g[name]")
        .expect_err("symbol evaluation should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue the suspended fixed-point evaluation");

    assert_eq!(response.value.to_string(), "done");
}

#[test]
fn resume_eval_request_with_input_finishes_with_bindings_without_reprompting() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("With[{x = InputString[\"name: \"]}, List[x, x]]")
        .expect_err("With binding should suspend when its initializer requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should reuse the suspended With binding");

    assert_eq!(response.value.to_string(), "List[\"Alice\", \"Alice\"]");
}

#[test]
fn resume_eval_request_with_input_preserves_block_scope_until_completion() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("x = global").unwrap();

    let failure = session
        .eval_request("Block[{x = local}, List[x, InputString[\"name: \"], x = List[x], x]]")
        .expect_err("Block body should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue inside the active Block scope");

    assert_eq!(
        response.value.to_string(),
        "List[local, \"Alice\", List[local], List[local]]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "global");
}

#[test]
fn resume_eval_request_with_input_skips_uninitialized_module_bindings() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "Module[{before, value = InputString[\"name: \"], after}, List[before, value, after]]",
        )
        .expect_err("Module initializer should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should skip the remaining uninitialized binding");

    assert_eq!(
        response.value.to_string(),
        "List[before$1, \"Alice\", after$3]"
    );
}

#[test]
fn resume_eval_request_with_input_reports_input_parse_errors_as_syntax_diagnostics() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("Input[\"expr: \"]")
        .expect_err("Input should suspend until host text is available");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::Input);
            assert_eq!(request.prompt, "expr: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let failure = session
        .resume_eval_request_with_input("1 ! 2")
        .expect_err("malformed host text should surface a syntax diagnostic");

    match &failure.error {
        SessionRequestError::Syntax(diagnostic) => {
            assert!(!diagnostic.message.is_empty());
            assert!(!diagnostic.is_incomplete);
            assert!(diagnostic.rendered.contains("--> 1:3"));
            assert_eq!(diagnostic.spans.len(), 1);
            assert_eq!(diagnostic.spans[0].start.column, 3);
            assert_eq!(diagnostic.spans[0].text, "!");
        }
        other => panic!("expected syntax diagnostic, got {other:?}"),
    }
    assert!(failure.messages.is_empty());
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_keeps_monitor_display_and_finishes_the_body() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("Monitor[InputString[\"name: \"], progress]")
        .expect_err("Monitor body should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should finish the suspended Monitor body");

    assert_eq!(response.value.to_string(), "\"Alice\"");
    assert_eq!(response.prints, vec!["progress"]);
}

#[test]
fn resume_eval_request_with_input_finishes_dialog_subsets() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("Dialog[InputString[\"name: \"]]")
        .expect_err("Dialog body should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should finish the suspended Dialog body");

    assert_eq!(response.value.to_string(), "\"Alice\"");
}

#[test]
fn resume_eval_request_with_input_can_finish_delayed_message_templates() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("f::argx := InputString[\"msg: \"]")
        .unwrap();

    let failure = session
        .eval_request("Message[f::argx]")
        .expect_err("delayed message template should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should finish delayed message rendering");

    assert_eq!(response.value.to_string(), "Null");
    assert!(response.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "argx" && message.text == "Alice"
    }));
}

#[test]
fn resume_eval_request_with_input_formats_delayed_message_templates_after_resume() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text(
            "f::argx := If[MatchQ[InputString[\"msg: \"], \"Alice\"], \"bad arg `1` from Alice\", \"bad arg `1` from stranger\"]",
        )
        .unwrap();

    let failure = session
        .eval_request("Message[f::argx, 42]")
        .expect_err("delayed formatted message template should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "msg: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should finish delayed message formatting after the template resolves");

    assert_eq!(response.value.to_string(), "Null");
    assert!(response.messages.iter().any(|message| {
        message.symbol == "f" && message.tag == "argx" && message.text == "bad arg 42 from Alice"
    }));
}

#[test]
fn resume_eval_request_with_input_continues_replace_all_without_restarting_target() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("ReplaceAll[InputString[\"name: \"], x_String :> List[x, x]]")
        .expect_err("ReplaceAll target should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should continue ReplaceAll from the suspended target");

    assert_eq!(response.value.to_string(), "List[\"Alice\", \"Alice\"]");
}

#[test]
fn nested_input_suspension_precedes_the_old_suffix_without_reordering_effects() {
    let mut session = session_with_test_control_builtins();

    let first = session
        .eval_request(
            r#"order = {};
               AppendTo[order, "A"];
               List[
                 List[
                   Input["A: "],
                   AppendTo[order, "C"]; "C-value"
                 ],
                 order
               ]"#,
        )
        .expect_err("A should suspend before the old C suffix");
    match &first.error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, "A: "),
        other => panic!("expected the A input request, got {other:?}"),
    }

    let nested = session
        .resume_eval_request_with_input(r#"AppendTo[order, "B"]; InputString["B: "]"#)
        .expect_err("A's resumed expression should create nested B");
    match &nested.error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, "B: "),
        other => panic!("expected the nested B input request, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("B-value")
        .expect("nested B should finish before the old C suffix");
    assert_eq!(
        response.value.to_string(),
        "List[List[\"B-value\", \"C-value\"], List[\"A\", \"B\", \"C\"]]"
    );
}

#[test]
fn queued_and_resumed_malformed_input_report_the_same_syntax_diagnostic() {
    // The depth-limit case needs 3 MiB in debug builds; allow more than twice that.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let deep = format!("{}1{}", "(".repeat(129), ")".repeat(129));
            for answer in ["(", "1 ! 2", "", "\"é\" + )", "f[", deep.as_str()] {
                let expected = rum_syntax::SourceDiagnostic::new(
                    rum_syntax::SourceText::new(answer),
                    rum_syntax::parse_expr(answer).unwrap_err(),
                )
                .structured();
                let mut queued = session_with_test_control_builtins();
                queued.queue_input_response(answer);
                let queued_failure = queued
                    .eval_request("Input[\"answer\"]")
                    .expect_err("malformed queued input should fail");

                let mut resumed = session_with_test_control_builtins();
                let pause = resumed
                    .eval_request("Input[\"answer\"]")
                    .expect_err("Input should suspend without a queued answer");
                assert!(matches!(
                    pause.error,
                    SessionRequestError::InputRequired { .. }
                ));
                let resumed_failure = resumed
                    .resume_eval_request_with_input(answer)
                    .expect_err("malformed resumed input should fail");

                for failure in [&queued_failure, &resumed_failure] {
                    let SessionRequestError::Syntax(diagnostic) = &failure.error else {
                        panic!(
                            "expected original Input syntax diagnostic for {answer:?}: {failure:?}"
                        );
                    };
                    assert_eq!(diagnostic, &expected, "answer: {answer:?}");
                }
                assert_eq!(queued_failure, resumed_failure, "answer: {answer:?}");
                for session in [&mut queued, &mut resumed] {
                    assert!(session.pending_input_request().is_none());
                    assert_eq!(session.eval_request("83").unwrap().value.to_string(), "83");
                }
            }
        })
        .expect("spawn Input syntax contract test")
        .join()
        .expect("Input syntax contract test panicked");
}
