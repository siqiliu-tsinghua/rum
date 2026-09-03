use super::*;

#[test]
fn input_print_and_pause_cover_current_first_pass_io_subset() {
    let mut session = Session::new();

    let printed = session.eval_text("Print[\"x=\", 2]").unwrap();
    assert_eq!(printed.value.to_string(), "Null");
    assert_eq!(printed.prints, vec!["x=2"]);

    let printed_held = session.eval_text("Print[HoldComplete[1 + 2]]").unwrap();
    assert_eq!(printed_held.value.to_string(), "Null");
    assert_eq!(printed_held.prints, vec!["HoldComplete[1 + 2]"]);

    let paused = session.eval_text("Pause[0]").unwrap();
    assert_eq!(paused.value.to_string(), "Null");
    assert!(paused.prints.is_empty());

    let paused = session.eval_text("Pause[1/1000]").unwrap();
    assert_eq!(paused.value.to_string(), "Null");
    assert!(paused.prints.is_empty());

    let invalid_pause = session.eval_text("Pause[-1]").unwrap();
    assert_eq!(invalid_pause.value.to_string(), "Pause[-1]");
    assert!(
        invalid_pause
            .messages
            .iter()
            .any(|message| message.symbol == "Pause" && message.tag == "numnm")
    );

    let failure = session
        .eval_request("InputString[1, FieldMasked -> True]")
        .expect_err("InputString should accept non-string prompts and surface masked input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::InputString);
            assert_eq!(request.prompt, "1");
            assert!(request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("secret")
        .expect("resuming masked InputString should return the supplied text");
    assert_eq!(response.value.to_string(), "\"secret\"");

    let failure = session
        .eval_request("Input[\"expr: \"]")
        .expect_err("Input should suspend until the host provides an expression");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::Input);
            assert_eq!(request.prompt, "expr: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("1 + 4")
        .expect("resuming Input should parse and evaluate the supplied expression");
    assert_eq!(response.value.to_string(), "5");
    assert!(response.prints.is_empty());

    let failure = session
        .eval_request("Input[\"expr: \", FieldMasked -> True]")
        .expect_err("Input should pass FieldMasked through to the host request");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::Input);
            assert_eq!(request.prompt, "expr: ");
            assert!(request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("2 + 3")
        .expect("resuming masked Input should parse and evaluate the supplied expression");
    assert_eq!(response.value.to_string(), "5");

    session.eval_text("Unprotect[Input, InputString]").unwrap();
    session
        .eval_text("SetOptions[Input, FieldMasked -> True]")
        .unwrap();
    let failure = session
        .eval_request("Input[]")
        .expect_err("Input should use its default prompt and default options");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::Input);
            assert_eq!(request.prompt, "?");
            assert!(request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("3 + 4")
        .expect("resuming default Input should parse and evaluate the supplied expression");
    assert_eq!(response.value.to_string(), "7");
    session
        .eval_text("SetOptions[Input, FieldMasked -> False]")
        .unwrap();

    session
        .eval_text("SetOptions[InputString, FieldMasked -> True]")
        .unwrap();
    let failure = session
        .eval_request("InputString[\"default masked: \"]")
        .expect_err("InputString should honor FieldMasked from SetOptions");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::InputString);
            assert_eq!(request.prompt, "default masked: ");
            assert!(request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("secret")
        .expect("resuming default-masked InputString should return the supplied text");
    assert_eq!(response.value.to_string(), "\"secret\"");
    session
        .eval_text("SetOptions[InputString, FieldMasked -> False]")
        .unwrap();

    let failure = session
        .eval_request("InputString[FieldMasked -> True]")
        .expect_err("InputString should treat a first-position option as an option, not a prompt");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::InputString);
            assert_eq!(request.prompt, "");
            assert!(request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("secret")
        .expect("resuming first-position option InputString should return the supplied text");
    assert_eq!(response.value.to_string(), "\"secret\"");

    let failure = session
        .eval_request("Input[{FieldMasked -> True}]")
        .expect_err("Input should treat a first-position option list as options");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::Input);
            assert_eq!(request.prompt, "?");
            assert!(request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("4 + 5")
        .expect("resuming first-position option-list Input should parse and evaluate text");
    assert_eq!(response.value.to_string(), "9");

    let failure = session
        .eval_request("InputString[HoldComplete[1 + 2]]")
        .expect_err("InputString should render held prompts in input-like form");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::InputString);
            assert_eq!(request.prompt, "HoldComplete[1 + 2]");
            assert!(!request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    let response = session
        .resume_eval_request_with_input("held")
        .expect("resuming held-prompt InputString should return the supplied text");
    assert_eq!(response.value.to_string(), "\"held\"");

    let unknown = session.eval_text("InputString[BadOption -> 1]").unwrap();
    assert_eq!(unknown.value.to_string(), "InputString[Rule[BadOption, 1]]");
    assert!(
        unknown
            .messages
            .iter()
            .any(|message| message.symbol == "InputString" && message.tag == "optx")
    );

    let non_option = session.eval_text("InputString[\"p\", 1]").unwrap();
    assert_eq!(non_option.value.to_string(), "InputString[\"p\", 1]");
    assert!(
        non_option
            .messages
            .iter()
            .any(|message| message.symbol == "InputString" && message.tag == "nonopt")
    );
    let non_option_list = session
        .eval_text("InputString[\"p\", {FieldMasked -> False, 1}]")
        .unwrap();
    assert_eq!(
        non_option_list.value.to_string(),
        "InputString[\"p\", List[Rule[FieldMasked, False], 1]]"
    );
    assert!(non_option_list.messages.iter().any(|message| {
        message.symbol == "InputString"
            && message.tag == "nonopt"
            && message.text.contains("List[Rule[FieldMasked, False], 1]")
    }));

    let invalid_option_name = session.eval_text("InputString[\"p\", 1 -> 2]").unwrap();
    assert_eq!(
        invalid_option_name.value.to_string(),
        "InputString[\"p\", Rule[1, 2]]"
    );
    assert!(
        invalid_option_name
            .messages
            .iter()
            .any(|message| message.symbol == "InputString" && message.tag == "optrs")
    );

    let input_unknown = session.eval_text("Input[BadOption -> 1]").unwrap();
    assert_eq!(input_unknown.value.to_string(), "Input[Rule[BadOption, 1]]");
    assert!(
        input_unknown
            .messages
            .iter()
            .any(|message| message.symbol == "Input" && message.tag == "optx")
    );

    let input_non_option = session.eval_text("Input[1, 2]").unwrap();
    assert_eq!(input_non_option.value.to_string(), "Input[1, 2]");
    assert!(
        input_non_option
            .messages
            .iter()
            .any(|message| message.symbol == "Input" && message.tag == "nonopt")
    );
    let input_non_option_list = session
        .eval_text("Input[\"p\", {FieldMasked -> False, 1}]")
        .unwrap();
    assert_eq!(
        input_non_option_list.value.to_string(),
        "Input[\"p\", List[Rule[FieldMasked, False], 1]]"
    );
    assert!(input_non_option_list.messages.iter().any(|message| {
        message.symbol == "Input"
            && message.tag == "nonopt"
            && message.text.contains("List[Rule[FieldMasked, False], 1]")
    }));
}

#[test]
fn pause_rejects_unrepresentable_integer_and_rational_durations() {
    let mut session = Session::new();

    for source in ["Pause[10^100]", "Pause[10^100/3]"] {
        let invalid_pause = session.eval_text(source).unwrap();
        let Expr::Call { head, args, .. } = &invalid_pause.value else {
            panic!("{source} should remain inert, got {}", invalid_pause.value);
        };
        assert_eq!(head.to_string(), "Pause");
        assert_eq!(args.len(), 1);
        assert!(
            invalid_pause
                .messages
                .iter()
                .any(|message| message.symbol == "Pause" && message.tag == "numnm"),
            "{source} should emit Pause::numnm"
        );
    }
}
