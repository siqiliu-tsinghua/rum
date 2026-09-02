use rum_host::{
    HostCompletenessStatus, HostCompletionRequest, HostConfig, HostEvalOutcome, HostFailureCode,
    HostInputKind, HostInspectRequest, HostIsCompleteRequest, HostSession,
    HostStructuredOutputKind, HostTyped,
};
use std::time::Duration;

fn assert_waiting_for_input_failure(outcome: HostEvalOutcome) {
    match outcome {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Other);
            assert!(failure.message.contains("waiting for host input"));
            assert!(!failure.code.requires_reset());
        }
        other => panic!("expected waiting-for-input failure, got {other:?}"),
    }
}

#[test]
fn public_contract_covers_eval_outputs_without_runtime_types() {
    let mut session = HostSession::builder().with_tracing(true).build();

    let outcome = session.eval("f[x_Integer] := x; Print[\"seen\"]; Message[General::stop]; f[3]");

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "3");
            assert_eq!(success.prints, vec!["seen"]);
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "stop")
            );
            assert!(success.trace.iter().any(|event| {
                matches!(event, rum_host::HostTraceEvent::RuleApplied { source, .. } if source == "DownValue")
            }));
            assert_eq!(
                success.outputs.last().map(|output| output.kind),
                Some(HostStructuredOutputKind::Result)
            );
            assert!(
                success
                    .outputs
                    .iter()
                    .any(|output| output.kind == HostStructuredOutputKind::Trace)
            );
        }
        other => panic!("expected host success, got {other:?}"),
    }
}

#[test]
fn public_contract_preserves_input_suspension_and_resume_boundaries() {
    let mut session = HostSession::new();

    match session.eval("Input[\"expr: \"] + 2") {
        HostEvalOutcome::InputRequired(suspension) => {
            assert_eq!(suspension.request.kind, HostInputKind::Input);
            assert_eq!(suspension.request.prompt, "expr: ");
            assert_eq!(
                session
                    .pending_input_request()
                    .map(|request| request.prompt),
                Some("expr: ".to_string())
            );
        }
        other => panic!("expected input suspension, got {other:?}"),
    }

    match session.resume_with_input("5") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "7"),
        other => panic!("expected resumed success, got {other:?}"),
    }

    match session.resume_with_input("unused") {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Other);
            assert!(failure.message.contains("no suspended evaluation"));
        }
        other => panic!("expected no-pending-input failure, got {other:?}"),
    }
}

#[test]
fn public_contract_rejects_overlapping_eval_and_preserves_block_suspension() {
    let mut session = HostSession::new();

    match session.eval("overlapBlock403 = \"outside\"") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"outside\""),
        other => panic!("expected setup success, got {other:?}"),
    }

    match session.eval("Block[{overlapBlock403 = \"inside\"}, InputString[\"old\"]]") {
        HostEvalOutcome::InputRequired(suspension) => {
            assert_eq!(suspension.request.kind, HostInputKind::InputString);
            assert_eq!(suspension.request.prompt, "old");
        }
        other => panic!("expected block suspension, got {other:?}"),
    }

    match session.eval("\"replacement\"") {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Other);
            assert!(failure.message.contains("waiting for host input"));
            assert!(!failure.code.requires_reset());
        }
        other => panic!("expected overlapping-evaluation rejection, got {other:?}"),
    }

    assert_eq!(
        session
            .pending_input_request()
            .map(|request| request.prompt),
        Some("old".to_string())
    );

    match session.resume_with_input("answer") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"answer\""),
        other => panic!("expected original continuation to resume, got {other:?}"),
    }

    match session.eval("overlapBlock403") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"outside\""),
        other => panic!("expected restored outer binding, got {other:?}"),
    }
}

#[test]
fn every_public_evaluation_entrypoint_rejects_a_pending_continuation() {
    let mut session = HostSession::new();
    assert!(matches!(
        session.eval("InputString[\"entrypoint\"]"),
        HostEvalOutcome::InputRequired(_)
    ));

    assert_waiting_for_input_failure(session.call("", &[]));
    assert_waiting_for_input_failure(session.call("Identity", &[HostTyped::Integer(1)]));
    assert_waiting_for_input_failure(session.eval_script("1\n2"));
    assert_waiting_for_input_failure(session.eval_file("missing-overlap-guard-403.rum"));
    assert_waiting_for_input_failure(session.eval_with_timeout("1", Duration::from_secs(1)));

    assert_eq!(
        session
            .pending_input_request()
            .map(|request| request.prompt),
        Some("entrypoint".to_string())
    );
    session.queue_input_response("queued");
    assert_waiting_for_input_failure(session.eval("1"));

    match session.resume_with_input("direct") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"direct\""),
        other => panic!("expected pending continuation to resume, got {other:?}"),
    }
    match session.eval("InputString[\"prequeued\"]") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"queued\""),
        other => panic!("expected queued response to remain available, got {other:?}"),
    }
}

#[test]
fn script_suspension_requires_reset_or_restore_while_inspection_remains_available() {
    let mut session = HostSession::new();
    assert!(matches!(
        session.eval("saved403 = 42"),
        HostEvalOutcome::Success(_)
    ));
    let snapshot = session.snapshot_to_bytes().unwrap();

    assert!(matches!(
        session.eval_script("InputString[\"script\"]\nscriptTail403 = 1"),
        HostEvalOutcome::InputRequired(_)
    ));
    assert!(session.inspect_symbol("saved403").found);

    match session.resume_with_input("unavailable") {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Other);
            assert!(failure.message.contains("no suspended evaluation"));
        }
        other => panic!("expected script resumption failure, got {other:?}"),
    }
    assert_waiting_for_input_failure(session.eval("1"));

    session.restore_from_bytes(&snapshot).unwrap();
    match session.eval("saved403") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "42"),
        other => panic!("expected restored session to evaluate, got {other:?}"),
    }
    match session.eval("scriptTail403") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "scriptTail403"),
        other => panic!("expected suspended script tail not to run, got {other:?}"),
    }
}

#[test]
fn public_contract_covers_editor_requests_and_failures() {
    let mut session = HostSession::new();

    assert!(matches!(
        session.eval("localName = 42"),
        HostEvalOutcome::Success(_)
    ));

    let inspect = session.inspect(HostInspectRequest::new("localName", 4));
    assert!(inspect.found);
    assert_eq!(
        inspect
            .info
            .as_ref()
            .map(|info| info.qualified_name.as_str()),
        Some("Global`localName")
    );
    assert!(
        inspect
            .definitions
            .iter()
            .any(|definition| definition.rhs == "42")
    );

    let completion = session.complete(HostCompletionRequest::new("localName", 4));
    assert_eq!(completion.prefix, "loca");
    assert_eq!(completion.replacement_start, 0);
    assert_eq!(completion.replacement_end, "localName".len());
    assert_eq!(completion.matches, vec!["localName"]);

    let incomplete = session.is_complete(HostIsCompleteRequest::new("f[\n  1"));
    assert_eq!(incomplete.status, HostCompletenessStatus::Incomplete);
    assert_eq!(incomplete.indent.as_deref(), Some("    "));

    // `1 2` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let invalid = session.is_complete("f[1,]");
    assert_eq!(invalid.status, HostCompletenessStatus::Invalid);
    assert!(invalid.diagnostic.is_some());

    match session.eval("f[1,]") {
        HostEvalOutcome::Failure(failure) => assert_eq!(failure.code, HostFailureCode::Syntax),
        other => panic!("expected syntax failure, got {other:?}"),
    }
}

#[test]
fn public_config_facade_controls_core_builtin_installation() {
    let disabled_config = HostConfig::default().with_core_builtins(false);
    let mut disabled = HostSession::builder().with_config(disabled_config).build();
    assert!(!disabled.configuration().install_core_builtins);

    assert!(
        disabled
            .complete(HostCompletionRequest::new("Ide", 3))
            .matches
            .is_empty()
    );
    let disabled_identity = disabled.inspect_symbol("Identity");
    assert!(disabled_identity.found);
    assert_eq!(
        disabled_identity
            .info
            .and_then(|info| info.summary)
            .as_deref(),
        None
    );

    let mut enabled = HostSession::new();
    assert_eq!(
        enabled
            .complete(HostCompletionRequest::new("Ide", 3))
            .matches,
        vec!["Identity"]
    );
    assert!(
        enabled
            .inspect_symbol("Identity")
            .info
            .and_then(|info| info.summary)
            .is_some()
    );
    match enabled.eval("1 + 2") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "3"),
        other => panic!("expected evaluated Plus with installed builtins, got {other:?}"),
    }
}
