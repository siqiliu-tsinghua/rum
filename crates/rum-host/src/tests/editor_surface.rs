use super::super::*;
use super::common::*;
use std::sync::{Arc, Mutex};

#[test]
fn host_eval_reports_value_messages_prints_and_structured_outputs() {
    let mut session = HostSession::new();

    let outcome = session.eval("Print[1]; Message[General::stop]; 2");

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "2");
            assert_eq!(success.prints, vec!["1"]);
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "stop")
            );
            assert!(
                success
                    .outputs
                    .iter()
                    .any(|output| output.kind == HostStructuredOutputKind::Result)
            );
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn host_eval_renders_top_level_hold_form_transparently_in_canonical_style() {
    let mut session = HostSession::new();

    let success = successful_eval(session.eval("HoldForm[1 + 1]"));

    assert_eq!(success.value.text, "Plus[1, 1]");
}

#[test]
fn host_print_sink_streams_exclusively_and_clear_restores_buffering() {
    let mut session = HostSession::new();
    let streamed = Arc::new(Mutex::new(Vec::new()));
    let sink_streamed = Arc::clone(&streamed);

    session.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
        sink_streamed.lock().unwrap().push(line.to_string());
    }));

    let streamed_outcome = successful_eval(session.eval("Print[1]; Print[2]; 3"));
    assert_eq!(
        streamed.lock().unwrap().as_slice(),
        ["1".to_string(), "2".to_string()]
    );
    assert!(streamed_outcome.prints.is_empty());

    session.clear_print_sink();

    let buffered_outcome = successful_eval(session.eval("Print[4]; 5"));
    assert_eq!(buffered_outcome.prints, vec!["4"]);
    assert_eq!(
        streamed.lock().unwrap().as_slice(),
        ["1".to_string(), "2".to_string()]
    );
}

#[test]
fn host_suspend_resume_flow_wraps_input_requests() {
    let mut session = HostSession::new();

    let suspended = session.eval("StringJoin[InputString[\"name: \"], \"!\"]");
    match suspended {
        HostEvalOutcome::InputRequired(suspension) => {
            assert_eq!(suspension.request.kind, HostInputKind::InputString);
            assert_eq!(suspension.request.prompt, "name: ");
            assert!(!suspension.request.password);
        }
        other => panic!("expected input suspension, got {other:?}"),
    }

    let resumed = session.resume_with_input("rum");
    match resumed {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"rum!\""),
        other => panic!("expected resumed success, got {other:?}"),
    }
}

#[test]
fn host_queued_input_completes_without_suspension() {
    let mut session = HostSession::new();

    session.queue_input_response("hello");

    let success = successful_eval(session.eval("InputString[]"));
    assert_eq!(success.value.text, "\"hello\"");
    assert_eq!(success.value.typed, HostTyped::Str("hello".to_string()));
}

#[test]
fn host_parse_inspect_complete_and_is_complete_cover_editor_flows() {
    let mut session = HostSession::new();

    let parsed = session.parse("1 + 2").expect("parse should succeed");
    assert_eq!(parsed.syntax, "1 + 2");

    assert!(matches!(
        session.eval("localName = 1"),
        HostEvalOutcome::Success(_)
    ));

    let inspect = session.inspect(HostInspectRequest::new("localName", 3));
    assert!(inspect.found);
    assert_eq!(
        inspect
            .info
            .as_ref()
            .map(|info| info.qualified_name.as_str()),
        Some("Global`localName")
    );

    let completion = session.complete(HostCompletionRequest::new("localName", 3));
    assert_eq!(completion.replacement_start, 0);
    assert_eq!(completion.replacement_end, "localName".len());
    assert_eq!(completion.prefix, "loc");
    assert_eq!(completion.matches, vec!["localName"]);

    let incomplete = session.is_complete("f[\n  1");
    assert_eq!(incomplete.status, HostCompletenessStatus::Incomplete);
    assert_eq!(incomplete.indent.as_deref(), Some("    "));

    let open_multiline_string = session.is_complete("x = \"\"\"\nabc");
    assert_eq!(
        open_multiline_string.status,
        HostCompletenessStatus::Incomplete
    );

    let open_raw_multiline_string = session.is_complete("x = '''\nabc");
    assert_eq!(
        open_raw_multiline_string.status,
        HostCompletenessStatus::Incomplete
    );
}

#[test]
fn host_lower_dump_returns_canonical_text_and_rendered_syntax_failures() {
    let mut session = HostSession::new();

    let lowered = session
        .lower_dump("x_Integer := f[x]")
        .expect("valid source should lower");
    assert!(!lowered.is_empty());
    assert_eq!(lowered, "SetDelayed[Pattern[x, Blank[Integer]], f[x]]");

    let failure = session
        .lower_dump("f[1,]")
        .expect_err("malformed source should fail lowering");
    assert_eq!(failure.code, HostFailureCode::Syntax);
    assert!(
        failure
            .syntax_rendered
            .as_deref()
            .is_some_and(|rendered| rendered.contains("error: expected expression"))
    );
}

#[test]
fn host_message_catalog_supports_global_symbol_and_invalid_queries() {
    let mut session = HostSession::new();
    successful_eval(session.eval("f::argx = \"bad arg\""));

    let global = session
        .message_catalog(None)
        .expect("the global catalog query should be valid");
    assert!(global.iter().any(|entry| {
        entry.message_name == "f::argx"
            && entry.qualified_symbol_name == "Global`f"
            && entry.tag == "argx"
            && entry.template == "\"bad arg\""
            && !entry.delayed
    }));

    let symbol = session
        .message_catalog(Some("f"))
        .expect("a symbol catalog query should be valid");
    assert_eq!(
        symbol,
        vec![HostMessageTemplate {
            message_name: "f::argx".to_string(),
            qualified_symbol_name: "Global`f".to_string(),
            tag: "argx".to_string(),
            template: "\"bad arg\"".to_string(),
            delayed: false,
        }]
    );

    let builtin = session
        .message_catalog(Some("Set"))
        .expect("a builtin symbol catalog query should be valid");
    assert_eq!(
        builtin.first(),
        Some(&HostMessageTemplate {
            message_name: "Set::argform".to_string(),
            qualified_symbol_name: "System`Set".to_string(),
            tag: "argform".to_string(),
            template: "\"Reports an invalid assignment target or assigned metadata value.\""
                .to_string(),
            delayed: false,
        })
    );

    assert_eq!(session.message_catalog(Some("f::")), None);
}

#[test]
fn host_evaluates_multiline_string_literals_as_ordinary_strings() {
    let mut session = HostSession::new();

    let escaped = successful_eval(session.eval("\"\"\"\nab\"\"\" === \"ab\""));
    assert_eq!(escaped.value.text, "True");

    let raw =
        successful_eval(session.eval("'''\nC:\\path \\t raw\n''' === \"C:\\\\path \\\\t raw\\n\""));
    assert_eq!(raw.value.text, "True");
}

#[test]
fn host_processes_inline_multiline_csv_data() {
    let mut session = HostSession::new();
    let source = "data = \"\"\"\na,1\nb,2\"\"\"; {StringLines[data], FromCsv[data]}";

    let success = successful_eval(session.eval(source));

    assert_eq!(
        success.value.text,
        "List[List[\"a,1\", \"b,2\"], List[List[\"a\", \"1\"], List[\"b\", \"2\"]]]"
    );
}
