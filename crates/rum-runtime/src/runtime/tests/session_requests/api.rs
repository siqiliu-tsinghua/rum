use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;
use std::time::Duration;

fn mark_file_request_evaluation_started(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    assert!(args.is_empty(), "evaluation-start probe takes no arguments");
    session.push_print("evaluation-started");
    Ok(system_expr(heads::structure::NULL))
}

struct InputConversionProbe {
    converted: Arc<AtomicBool>,
}

impl From<InputConversionProbe> for String {
    fn from(probe: InputConversionProbe) -> Self {
        probe.converted.store(true, Ordering::Relaxed);
        "input-that-must-not-run".to_string()
    }
}

fn cancel_after_evaluation_start(
    session: &mut Session,
    cancel: RequestCancelHandle,
) -> thread::JoinHandle<()> {
    let (evaluation_started_tx, evaluation_started_rx) = mpsc::sync_channel(0);
    let (cancelled_tx, cancelled_rx) = mpsc::sync_channel(0);
    session.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |line| {
            assert_eq!(line, "evaluation-started");
            evaluation_started_tx
                .send(())
                .expect("canceller should receive evaluation-start signal");
            cancelled_rx
                .recv()
                .expect("canceller should acknowledge the qualified cancel");
        },
    )));

    thread::spawn(move || {
        evaluation_started_rx
            .recv()
            .expect("evaluation should report that it started");
        cancel.cancel();
        cancelled_tx
            .send(())
            .expect("evaluation should wait for cancellation acknowledgement");
    })
}

fn file_request_error(source: &str, configure: impl FnOnce(&mut Session)) -> SessionRequestError {
    let dir = TempDirGuard::new("request-error");
    let file = dir.write("request-error.rum", source);
    let mut session = session_with_test_control_builtins();
    install_test_file_io(&mut session);
    configure(&mut session);

    session
        .eval_file_request(&file)
        .expect_err("file request should fail")
        .error
}

fn assert_file_request_error(
    source: &str,
    configure: impl FnOnce(&mut Session),
    matches_error: impl FnOnce(SessionRequestError),
) {
    matches_error(file_request_error(source, configure));
}

#[test]
fn parse_request_returns_structured_syntax_diagnostics() {
    let session = Session::new();
    let failure = session
        .parse_request("f[1 ! 2]")
        .expect_err("malformed input should fail with a syntax diagnostic");

    match failure.error {
        SessionRequestError::Syntax(diagnostic) => {
            assert!(diagnostic.message.starts_with("expected ,"));
            assert!(!diagnostic.is_incomplete);
            assert!(diagnostic.rendered.contains("--> 1:5"));
            assert_eq!(diagnostic.spans.len(), 1);
            assert_eq!(diagnostic.spans[0].start.line, 1);
            assert_eq!(diagnostic.spans[0].start.column, 5);
            assert_eq!(diagnostic.spans[0].text, "!");
        }
        other => panic!("expected syntax diagnostic, got {other:?}"),
    }

    assert!(failure.messages.is_empty());
    assert!(failure.trace.is_empty());
}

#[test]
fn script_requests_report_file_coordinates_for_every_form_shape() {
    let cases = [
        ("x = 1\nw = 2 !", 12..13, 2, 7, "!"),
        ("a = 1\nb = 2\nc = 3\nd = 1 ! 2", 24..25, 4, 7, "!"),
        ("a = 1\nf[\n  1 ! 2\n]", 13..14, 3, 5, "!"),
        ("a = 1\r\nb = 2\r\nc = 3\r\nd = 1 ! 2", 27..28, 4, 7, "!"),
        ("f[1,]\nb = 2", 4..5, 1, 5, "]"),
        ("x = 1\ny +", 9..9, 2, 4, ""),
        ("x = 1\n\\", 6..7, 2, 1, "\\"),
        ("x = 1\ns = \"", 10..11, 2, 5, "\""),
    ];

    for (source, span, line, column, text) in cases {
        let mut session = Session::new();
        let failure = session
            .eval_script_request(source)
            .expect_err("malformed script should return a syntax diagnostic");
        let SessionRequestError::Syntax(diagnostic) = failure.error else {
            panic!("expected syntax diagnostic for {source:?}")
        };

        assert_eq!(diagnostic.spans.len(), 1, "source: {source:?}");
        assert_eq!(diagnostic.spans[0].span.start, span.start, "{source:?}");
        assert_eq!(diagnostic.spans[0].span.end, span.end, "{source:?}");
        assert_eq!(diagnostic.spans[0].start.line, line, "{source:?}");
        assert_eq!(diagnostic.spans[0].start.column, column, "{source:?}");
        assert_eq!(diagnostic.spans[0].text, text, "{source:?}");
        assert!(
            diagnostic
                .rendered
                .contains(&format!("--> {line}:{column}")),
            "{}",
            diagnostic.rendered
        );
    }

    let mut session = Session::new();
    let failure = session
        .eval_script_request("a = 1\nb = 2\nc = 3\nd = 1 ! 2")
        .expect_err("the fourth form should fail");
    let SessionRequestError::Syntax(diagnostic) = failure.error else {
        panic!("expected a syntax diagnostic")
    };
    assert!(
        diagnostic.rendered.contains("4 | d = 1 ! 2\n  |       ^"),
        "{}",
        diagnostic.rendered
    );

    let mut session = Session::new();
    let failure = session
        .eval_script_request("a = 1\nf[\n  1 ! 2\n]")
        .expect_err("the multiline second form should fail");
    let SessionRequestError::Syntax(diagnostic) = failure.error else {
        panic!("expected a syntax diagnostic")
    };
    assert!(
        diagnostic.rendered.contains("3 |   1 ! 2\n  |     ^"),
        "{}",
        diagnostic.rendered
    );
}

#[test]
fn complete_request_returns_transport_friendly_matches() {
    let mut session = Session::new();
    session.eval_text("localName = 1").unwrap();

    let response = session.complete_request("loc");

    assert_eq!(response.prefix, "loc");
    assert_eq!(response.matches, vec!["localName"]);
}

#[test]
fn is_complete_request_distinguishes_complete_incomplete_and_invalid_input() {
    let session = Session::new();

    let complete = session.is_complete_request("f[1]");
    assert_eq!(complete.status, SessionCompletenessStatus::Complete);
    assert!(complete.indent.is_none());
    assert!(complete.diagnostic.is_none());

    let incomplete = session.is_complete_request("f[\n  1");
    assert_eq!(incomplete.status, SessionCompletenessStatus::Incomplete);
    assert_eq!(incomplete.indent.as_deref(), Some("    "));
    assert!(
        incomplete
            .diagnostic
            .as_ref()
            .is_some_and(|diagnostic| diagnostic.is_incomplete)
    );

    let invalid = session.is_complete_request("f[1 ! 2]");
    assert_eq!(invalid.status, SessionCompletenessStatus::Invalid);
    assert!(invalid.indent.is_none());
    assert!(
        invalid
            .diagnostic
            .as_ref()
            .is_some_and(|diagnostic| !diagnostic.is_incomplete)
    );
}

#[test]
fn eval_request_returns_transport_friendly_success_data() {
    let mut session = Session::new();
    session.enable_tracing(true);
    session.eval_text("f[x_Integer] := x").unwrap();

    let response = session.eval_request("f[1]").unwrap();

    assert_eq!(response.value.to_string(), "1");
    assert!(response.messages.is_empty());
    assert!(response.trace.iter().any(|event| matches!(
        event,
        TraceEvent::RuleApplied {
            source: RuleApplicationSource::DownValue,
            input,
            ..
        } if input.to_string() == "f[1]"
    )));
}

#[test]
fn eval_request_returns_transport_friendly_cycle_termination_data() {
    let mut session = Session::new();
    session.enable_tracing(true);
    session.eval_text("f[x_Integer] := g[x]").unwrap();
    session.eval_text("g[x_Integer] := f[x]").unwrap();

    let response = session
        .eval_request("f[1]")
        .expect("recursive evaluation should surface a termination value");

    assert_eq!(
        response.value.to_string(),
        "TerminatedEvaluation[\"IterationLimit\"]"
    );
    assert!(response.trace.iter().any(|event| matches!(
        event,
        TraceEvent::RuleApplied {
            source: RuleApplicationSource::DownValue,
            input,
            ..
        } if input.to_string() == "f[1]" || input.to_string() == "g[1]"
    )));
    assert!(
        response
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "passlim")
    );
}

#[test]
fn eval_expr_request_aggregates_messages_and_classifies_budget_termination() {
    let mut session = session_with_test_control_builtins();
    session.set_recursion_limit(32);
    session
        .eval_request("grow[x_] := grow[f[x]]")
        .expect("recursive definition should be stored");
    let expr = session
        .lower_request("Message[General::stop]; grow[1]")
        .expect("request expression should lower")
        .expr;

    let response = session
        .eval_expr_request(expr)
        .expect("recursion budget should surface a termination value");

    assert_eq!(
        response.value.to_string(),
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert_eq!(
        response.termination,
        Some(EvalTerminationReason::RecursionLimit)
    );
    assert!(
        response
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "stop")
    );
    assert!(
        response
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reclim")
    );
}

#[test]
fn eval_request_memory_ceiling_returns_classified_termination_and_session_recovers() {
    let mut session = session_with_test_control_builtins();
    let mut limits = session.resource_limits();
    limits.eval_memory_limit = Some(100_000);
    session.set_resource_limits(limits);

    let response = session
        .eval_request("Table[k, {k, 10000}]")
        .expect("memory ceiling should surface a termination value");

    assert_eq!(
        response.value.to_string(),
        "TerminatedEvaluation[\"MemoryLimit\"]"
    );
    assert_eq!(
        response.termination,
        Some(EvalTerminationReason::MemoryLimit)
    );
    assert!(
        response
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "memlim")
    );

    let recovered = session
        .eval_request("1 + 1")
        .expect("the next request should re-arm and complete normally");
    assert_eq!(recovered.value.to_string(), "2");
    assert_eq!(recovered.termination, None);
}

#[test]
fn eval_request_print_flood_trips_memory_ceiling_with_classified_termination() {
    let mut session = session_with_test_control_builtins();
    let mut limits = session.resource_limits();
    limits.eval_memory_limit = Some(100_000);
    session.set_resource_limits(limits);

    let response = session
        .eval_request("Do[Print[\"x\"], {100000}]")
        .expect("print flood should surface a memory-limit termination value");

    assert_eq!(
        response.value.to_string(),
        "TerminatedEvaluation[\"MemoryLimit\"]"
    );
    assert_eq!(
        response.termination,
        Some(EvalTerminationReason::MemoryLimit)
    );
    assert!(
        response
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "memlim")
    );
}

#[test]
fn eval_request_nested_memory_constraint_catches_before_higher_request_ceiling() {
    let mut session = session_with_test_control_builtins();
    let mut limits = session.resource_limits();
    limits.eval_memory_limit = Some(1_000_000);
    session.set_resource_limits(limits);

    let response = session
        .eval_request("List[MemoryConstrained[Table[k, {k, 10000}], 100000], 1 + 1]")
        .expect("the nested memory frame should catch its own breach");

    assert_eq!(response.value.to_string(), "List[$Aborted, 2]");
    assert_eq!(response.termination, None);
}

#[test]
fn inspect_request_aggregates_symbol_definitions_and_messages() {
    let mut session = Session::new();
    session.eval_text("f[x_Integer] := x").unwrap();
    session.eval_text("f::usage = \"desc\"").unwrap();

    let response = session
        .inspect_request("f")
        .expect("known symbol should produce an inspect response");

    assert_eq!(response.info.qualified_name, "Global`f");
    assert!(response.info.has_down_values);
    assert_eq!(response.definitions.len(), 1);
    assert_eq!(response.message_catalog.len(), 1);
    assert_eq!(response.message_catalog[0].message_name, "f::usage");
}

#[test]
fn eval_request_surfaces_structured_input_requests() {
    let mut session = Session::new();

    let failure = session
        .eval_request("InputString[\"name: \"]")
        .expect_err("missing host input should become a structured request failure");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.kind, RuntimeInputKind::InputString);
            assert_eq!(request.prompt, "name: ");
            assert!(!request.password);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    assert_eq!(
        session.pending_input_request(),
        Some(&RuntimeInputRequest::new("name: "))
    );
}

#[test]
fn eval_request_take_pending_input_request_moves_and_clears_the_request() {
    let mut session = Session::new();

    let failure = session
        .eval_request("Input[\"expr: \"]")
        .expect_err("Input should suspend the request");
    assert!(matches!(
        failure.error,
        SessionRequestError::InputRequired { .. }
    ));

    let request = session
        .take_pending_input_request()
        .expect("pending input request should be movable");
    assert_eq!(request.kind, RuntimeInputKind::Input);
    assert_eq!(request.prompt, "expr: ");
    assert!(session.pending_input_request().is_none());
}

#[test]
fn file_request_maps_completed_syntax_io_and_missing_provider_errors() {
    assert_file_request_error(
        "f[1,]",
        |_| {},
        |error| match error {
            SessionRequestError::Syntax(diagnostic) => {
                assert!(!diagnostic.message.is_empty());
                assert!(!diagnostic.is_incomplete);
                assert!(diagnostic.rendered.contains("f[1,]"));
            }
            other => panic!("expected syntax request error, got {other:?}"),
        },
    );

    let missing_error = {
        let mut session = Session::new();
        install_test_file_io(&mut session);
        session
            .eval_file_request("missing-file.rum")
            .expect_err("missing file should fail")
            .error
    };
    match missing_error {
        SessionRequestError::Io { path, message } => {
            assert!(path.ends_with("missing-file.rum"));
            assert!(!message.is_empty());
        }
        other => panic!("expected IO request error, got {other:?}"),
    }

    let mut no_provider_session = Session::new();
    no_provider_session.set_file_io(None);
    let no_provider = no_provider_session
        .eval_file_request("missing-file.rum")
        .expect_err("file request without provider should fail")
        .error;
    match no_provider {
        SessionRequestError::Other { message } => {
            assert_eq!(message, "no file provider installed");
        }
        other => panic!("expected no-provider request error, got {other:?}"),
    }
}

#[test]
fn accepted_cancel_before_request_entry_is_observed() {
    let mut session = session_with_test_control_builtins();
    let accepted = session
        .accept_cancellable_request()
        .expect("the idle session should accept a cancellable request");
    let cancel = accepted.cancel_handle();
    cancel.cancel();
    cancel.cancel();

    let response = session
        .eval_accepted_request(accepted, "1 + 1")
        .expect("cancellation should surface as a terminated evaluation");

    assert_eq!(response.termination, Some(EvalTerminationReason::Cancelled));
}

#[test]
fn ordinary_request_and_lazy_observer_leave_interrupt_poll_unarmed() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<CancellationObserver>();
    let mut session = session_with_test_control_builtins();
    assert!(session.eval_timeout.is_none());

    let ordinary = session
        .eval_request("1 + 1")
        .expect("an ordinary request should complete");
    assert_eq!(ordinary.value.to_string(), "2");
    let ordinary_request_armed = session.interrupt_armed;

    let observer = session.cancellation_observer();
    let observer_held = session
        .eval_request("2 + 2")
        .expect("holding an observer must not arm the request");
    assert_eq!(observer_held.value.to_string(), "4");
    assert!(!observer.is_cancelled());
    let observer_held_request_armed = session.interrupt_armed;

    println!(
        "ordinary_request_armed={ordinary_request_armed} observer_held_request_armed={observer_held_request_armed}"
    );
    assert!(!ordinary_request_armed);
    assert!(!observer_held_request_armed);
}

#[test]
fn cancellation_observer_held_across_session_rebuild_is_inert() {
    let mut session = session_with_test_control_builtins();
    let observer = session.cancellation_observer();
    let old_cancel = session.request_cancel_handle();
    old_cancel.cancel();
    assert!(observer.is_cancelled());

    session = session_with_test_control_builtins();
    let current_cancel = session.request_cancel_handle();
    current_cancel.cancel();
    assert!(
        !observer.is_cancelled(),
        "an observer retained across a session rebuild must be inert"
    );
    let current = session
        .eval_request("3 + 3")
        .expect("the rebuilt session's current cancellation should complete");
    assert_eq!(current.termination, Some(EvalTerminationReason::Cancelled));
}

#[test]
fn completed_request_handle_is_inert_for_later_requests() {
    let mut session = session_with_test_control_builtins();
    let first = session
        .accept_cancellable_request()
        .expect("the first request should be accepted");
    let stale = first.cancel_handle();
    let first_response = session
        .eval_accepted_request(first, "1 + 1")
        .expect("the first request should complete");
    assert_eq!(first_response.value.to_string(), "2");

    stale.cancel();
    let ordinary = session
        .eval_request("2 + 2")
        .expect("a stale handle must not affect an ordinary request");
    assert_eq!(ordinary.value.to_string(), "4");

    let second = session
        .accept_cancellable_request()
        .expect("the second request should be accepted");
    stale.cancel();
    let second_response = session
        .eval_accepted_request(second, "3 + 3")
        .expect("a stale handle must not affect a later accepted request");
    assert_eq!(second_response.value.to_string(), "6");
}

#[test]
fn stale_cancel_cannot_overwrite_current_request_cancel() {
    let mut session = session_with_test_control_builtins();
    let completed = session
        .accept_cancellable_request()
        .expect("the first request should be accepted");
    let stale = completed.cancel_handle();
    session
        .eval_accepted_request(completed, "1")
        .expect("the first request should complete");

    let current = session
        .accept_cancellable_request()
        .expect("the current request should be accepted");
    let current_cancel = current.cancel_handle();
    current_cancel.cancel();
    stale.cancel();

    let response = session
        .eval_accepted_request(current, "2")
        .expect("current cancellation should surface as a value");
    assert_eq!(response.termination, Some(EvalTerminationReason::Cancelled));
}

#[test]
fn accepted_cancel_abandons_input_before_conversion_or_continuation() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_request("x = outside; cancelSentinel404 = 0")
        .expect("the outer state should be installed");
    let accepted = session
        .accept_cancellable_request()
        .expect("the request should be accepted");
    let cancel = accepted.cancel_handle();

    let suspended = session
        .eval_accepted_request(
            accepted,
            "Block[{x = inside}, InputString[\"name: \"]; cancelSentinel404 = 1]",
        )
        .expect_err("the request should suspend for host input");
    assert!(matches!(
        suspended.error,
        SessionRequestError::InputRequired { .. }
    ));

    cancel.cancel();
    let converted = Arc::new(AtomicBool::new(false));
    let response = session
        .resume_eval_request_with_input(InputConversionProbe {
            converted: Arc::clone(&converted),
        })
        .expect("latched cancellation should terminate the suspended request");

    assert!(!converted.load(Ordering::Relaxed));
    assert_eq!(response.termination, Some(EvalTerminationReason::Cancelled));
    let follow_up = session
        .eval_request("List[4 + 4, x, cancelSentinel404]")
        .expect("the session should remain usable after suspended cancellation");
    assert_eq!(follow_up.value.to_string(), "List[8, outside, 0]");
}

#[test]
fn dropped_accepted_ticket_and_generation_rotation_retire_handles() {
    let mut session = session_with_test_control_builtins();
    let abandoned = session
        .accept_cancellable_request()
        .expect("the request should be accepted");
    let stale = abandoned.cancel_handle();
    drop(abandoned);
    stale.cancel();

    session.next_request_generation.set(MAX_REQUEST_GENERATION);
    let final_generation = session
        .accept_cancellable_request()
        .expect("the final generation should be accepted");
    let retired_control = final_generation.cancel_handle();
    drop(final_generation);

    let rotated = session
        .accept_cancellable_request()
        .expect("generation exhaustion should rotate the control block");
    let current = rotated.cancel_handle();
    current.cancel();
    stale.cancel();
    retired_control.cancel();
    let response = session
        .eval_accepted_request(rotated, "8 + 8")
        .expect("retired handles must not erase the current cancellation");
    assert_eq!(response.termination, Some(EvalTerminationReason::Cancelled));
}

#[test]
fn accepted_script_and_file_stop_before_later_forms_after_cancellation() {
    let mut script_session = session_with_test_control_builtins();
    script_session.register_builtin(BuiltinSpec::ordinary(
        "TestEvaluationStarted",
        "Marks the accepted script's first form as started.",
        mark_file_request_evaluation_started,
        MemoryGrowthContract::Incremental,
    ));
    let script = script_session
        .accept_cancellable_request()
        .expect("the script request should be accepted");
    let script_canceller =
        cancel_after_evaluation_start(&mut script_session, script.cancel_handle());
    let script_response = script_session
        .eval_accepted_script_request(script, "TestEvaluationStarted[]\nlaterScriptForm404 = 1")
        .expect("script cancellation should surface as a terminated value");
    script_canceller
        .join()
        .expect("script canceller should not panic");
    script_session.clear_print_sink();
    assert_eq!(
        script_response.termination,
        Some(EvalTerminationReason::Cancelled)
    );
    assert_eq!(
        script_session
            .eval_request("laterScriptForm404")
            .expect("the script session should remain usable")
            .value
            .to_string(),
        "laterScriptForm404"
    );

    let dir = TempDirGuard::new("accepted-request-cancel");
    let file = dir.write(
        "accepted-cancel.rum",
        "TestEvaluationStarted[]\nlaterFileForm404 = 1",
    );
    let mut file_session = session_with_test_control_builtins();
    file_session.register_builtin(BuiltinSpec::ordinary(
        "TestEvaluationStarted",
        "Marks the accepted file's first form as started.",
        mark_file_request_evaluation_started,
        MemoryGrowthContract::Incremental,
    ));
    install_test_file_io(&mut file_session);
    let file_request = file_session
        .accept_cancellable_request()
        .expect("the file request should be accepted");
    let file_canceller =
        cancel_after_evaluation_start(&mut file_session, file_request.cancel_handle());
    let file_failure = file_session
        .eval_accepted_file_request(file_request, &file)
        .expect_err("loader-shaped cancellation should remain a request failure");
    file_canceller
        .join()
        .expect("file canceller should not panic");
    file_session.clear_print_sink();
    assert!(matches!(file_failure.error, SessionRequestError::Cancelled));
    assert_eq!(
        file_session
            .eval_request("laterFileForm404")
            .expect("the file session should remain usable")
            .value
            .to_string(),
        "laterFileForm404"
    );
}

#[test]
fn file_request_maps_resource_limit_exit_and_cancel_errors() {
    assert_file_request_error(
        "grow[x_] := grow[f[x]]\ngrow[1]",
        |session| session.set_recursion_limit(32),
        |error| match error {
            SessionRequestError::RecursionLimitExceeded { limit } => assert_eq!(limit, 32),
            other => panic!("expected recursion-limit request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "f[n_] := f[n + 1]\nf[0]",
        |session| session.set_native_stack_limit(512 * 1024),
        |error| match error {
            SessionRequestError::StackLimitExceeded { limit } => assert_eq!(limit, 512 * 1024),
            other => panic!("expected stack-limit request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Table[x, 5]",
        |session| session.set_generated_element_limit(3),
        |error| match error {
            SessionRequestError::GeneratedElementLimitExceeded { limit } => assert_eq!(limit, 3),
            other => panic!("expected generated-element request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Factorial[5]",
        |session| session.set_integer_work_limit(3),
        |error| match error {
            SessionRequestError::IntegerWorkLimitExceeded { limit } => assert_eq!(limit, 3),
            other => panic!("expected integer-work request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Do[Null, 5]",
        |session| session.set_iteration_work_limit(3),
        |error| match error {
            SessionRequestError::IterationWorkLimitExceeded { limit } => assert_eq!(limit, 3),
            other => panic!("expected iteration-work request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "1 //. {1 -> 2, 2 -> 1}",
        |session| session.set_replace_repeated_limit(3),
        |error| match error {
            SessionRequestError::IterationLimitExceeded { limit } => assert_eq!(limit, 3),
            other => panic!("expected replace-repeated request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Exit[5]",
        |_| {},
        |error| match error {
            SessionRequestError::Exit { code } => assert_eq!(code, 5),
            other => panic!("expected exit request error, got {other:?}"),
        },
    );

    assert_file_request_error(
        "Do[x = 1, 50000000]",
        |session| {
            session.set_iteration_work_limit(50_000_000);
            session.set_eval_timeout(Some(Duration::from_millis(0)));
        },
        |error| match error {
            SessionRequestError::TimeConstraintExceeded => {}
            other => panic!("expected timeout request error, got {other:?}"),
        },
    );

    let dir = TempDirGuard::new("request-cancel");
    let file = dir.write("cancel.rum", "TestEvaluationStarted[]; Do[x = 1, 50000000]");
    let mut session = session_with_test_control_builtins();
    session.register_builtin(BuiltinSpec::ordinary(
        "TestEvaluationStarted",
        "Marks that a cancellation-test file evaluation reached builtin dispatch.",
        mark_file_request_evaluation_started,
        MemoryGrowthContract::Incremental,
    ));
    let (evaluation_started_tx, evaluation_started_rx) = mpsc::sync_channel(0);
    session.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |line| {
            assert_eq!(line, "evaluation-started");
            evaluation_started_tx
                .send(())
                .expect("canceller should receive evaluation-start signal");
        },
    )));
    install_test_file_io(&mut session);
    session.set_iteration_work_limit(50_000_000);
    let cancel = session.request_cancel_handle();
    let canceller = {
        let cancel = cancel.clone();
        thread::spawn(move || {
            evaluation_started_rx
                .recv()
                .expect("file evaluation should report that it started");
            cancel.cancel();
        })
    };
    let cancelled = session
        .eval_file_request(&file)
        .expect_err("cancelled file request should fail")
        .error;
    canceller.join().expect("canceller should not panic");
    assert!(matches!(cancelled, SessionRequestError::Cancelled));
}
