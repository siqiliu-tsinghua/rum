use rum_host::{
    ClosurePrintSink, HostCancelHandle, HostConfig, HostDefinitionSummary, HostEvalOutcome,
    HostEvalSuccess, HostFailure, HostFailureCode, HostInputKind, HostMessage, HostResourceLimits,
    HostSession, HostStructuredOutputKind, HostTerminationReason, HostTyped,
};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const LONG_RUNNING_EVAL: &str = "Do[x = 1, 50000000]";
const CANCELLABLE_LONG_RUNNING_EVAL: &str = "Print[\"evaluation-started\"]; Do[x = 1, 50000000]";

fn host_session_with_large_iteration_budget() -> HostSession {
    let mut limits = HostResourceLimits::default();
    limits.iteration_work_limit = 50_000_000;
    HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build()
}

fn host_session_with_iteration_work_limit(limit: usize) -> HostSession {
    let mut limits = HostResourceLimits::default();
    limits.iteration_work_limit = limit;
    HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build()
}

fn successful_eval(outcome: HostEvalOutcome) -> HostEvalSuccess {
    match outcome {
        HostEvalOutcome::Success(success) => success,
        other => panic!("expected success, got {other:?}"),
    }
}

fn assert_success_text(outcome: HostEvalOutcome, expected: &str) -> HostEvalSuccess {
    let success = successful_eval(outcome);
    assert_eq!(success.value.text, expected);
    success
}

fn assert_terminated_value(outcome: HostEvalOutcome, expected_text: &str, expected_tag: &str) {
    let success = successful_eval(outcome);
    assert_eq!(success.value.text, expected_text);
    assert!(
        success
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == expected_tag),
        "missing General::{expected_tag} in {:?}",
        success.messages
    );
    assert_eq!(
        success.outputs.last().map(|output| output.kind),
        Some(HostStructuredOutputKind::Result)
    );
}

fn typed_contains_other(value: &HostTyped) -> bool {
    match value {
        HostTyped::Other { .. } => true,
        HostTyped::List(items) => items.iter().any(typed_contains_other),
        HostTyped::Association(entries) => entries
            .iter()
            .any(|(key, value)| typed_contains_other(key) || typed_contains_other(value)),
        HostTyped::Rule { lhs, rhs } => {
            typed_contains_other(lhs.as_ref()) || typed_contains_other(rhs.as_ref())
        }
        _ => false,
    }
}

#[test]
fn eval_with_timeout_returns_normal_value_before_deadline() {
    let mut session = HostSession::new();

    let success = assert_success_text(
        session.eval_with_timeout("Do[x = 1, 3]; x", Duration::from_secs(1)),
        "1",
    );
    assert!(success.messages.is_empty());

    assert_success_text(session.eval("2 + 2"), "4");
}

#[test]
fn eval_with_timeout_terminates_long_eval_and_session_remains_usable() {
    let mut session = host_session_with_large_iteration_budget();

    let started = Instant::now();
    let outcome = session.eval_with_timeout(LONG_RUNNING_EVAL, Duration::from_millis(100));
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_secs(2),
        "timed eval took {elapsed:?}"
    );
    assert_terminated_value(
        outcome,
        "TerminatedEvaluation[\"TimeConstraint\"]",
        "timelim",
    );

    assert_success_text(session.eval("2 + 3"), "5");
}

#[test]
fn cancel_handle_targets_one_request_and_is_inert_after_completion() {
    let mut session = HostSession::new();
    let cancel = session.cancel_handle();

    cancel.cancel();
    assert_terminated_value(
        session.eval("1 + 2"),
        "TerminatedEvaluation[\"Cancelled\"]",
        "cancel",
    );

    cancel.cancel();
    assert_success_text(session.eval("3 + 4"), "7");
}

#[test]
fn stale_cancel_cannot_erase_a_current_request_cancel() {
    let mut session = HostSession::new();
    let stale = session.cancel_handle();
    assert_success_text(session.eval("1"), "1");

    let current = session.cancel_handle();
    current.cancel();
    stale.cancel();
    assert_terminated_value(
        session.eval("2"),
        "TerminatedEvaluation[\"Cancelled\"]",
        "cancel",
    );
    assert_success_text(session.eval("3"), "3");
}

#[test]
fn cancel_accepted_before_eval_request_survives_request_prologue() {
    let session = HostSession::new();
    let cancel = session.cancel_handle();
    let (ready_tx, ready_rx) = mpsc::sync_channel(0);
    let (cancelled_tx, cancelled_rx) = mpsc::sync_channel(0);

    let worker = thread::spawn(move || {
        let mut session = session;
        ready_tx
            .send(())
            .expect("canceller should observe the pre-evaluation boundary");
        cancelled_rx
            .recv()
            .expect("worker should wait until cancellation is accepted");
        session.eval("1 + 1")
    });

    ready_rx
        .recv()
        .expect("worker should reach the pre-evaluation boundary");
    cancel.cancel();
    cancelled_tx
        .send(())
        .expect("worker should be released after cancellation");

    let outcome = worker.join().expect("evaluation worker should not panic");
    assert_terminated_value(outcome, "TerminatedEvaluation[\"Cancelled\"]", "cancel");
}

#[test]
fn cancel_handle_interrupts_running_eval_and_session_remains_usable() {
    let mut session = host_session_with_large_iteration_budget();
    let cancel = session.cancel_handle();
    let (cancelled_tx, cancelled_rx) = mpsc::channel();
    let (evaluation_started_tx, evaluation_started_rx) = mpsc::sync_channel(0);
    session.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
        assert_eq!(line, "evaluation-started");
        evaluation_started_tx
            .send(())
            .expect("canceller should receive evaluation-start signal");
    }));

    let canceller = thread::spawn(move || {
        evaluation_started_rx
            .recv()
            .expect("evaluation should report that it started");
        let cancelled_at = Instant::now();
        cancel.cancel();
        cancelled_tx
            .send(cancelled_at)
            .expect("test should receive cancel timestamp");
    });

    let started = Instant::now();
    let outcome = session.eval(CANCELLABLE_LONG_RUNNING_EVAL);
    let completed_at = Instant::now();
    let elapsed = completed_at.duration_since(started);
    let cancelled_at = cancelled_rx
        .recv()
        .expect("canceller should report cancel timestamp");
    canceller.join().expect("canceller should not panic");
    let cancel_latency = completed_at.duration_since(cancelled_at);

    assert!(
        elapsed < Duration::from_secs(2),
        "cancelled eval took {elapsed:?}"
    );
    assert!(
        cancel_latency < Duration::from_secs(1),
        "cancel observation took {cancel_latency:?}"
    );
    assert_terminated_value(outcome, "TerminatedEvaluation[\"Cancelled\"]", "cancel");

    assert_success_text(session.eval("10 + 1"), "11");
}

#[test]
fn reset_is_idempotent_and_old_cancel_handles_do_not_affect_rebuilt_session() {
    let mut session = HostSession::new();
    let old_cancel = session.cancel_handle();

    assert_success_text(session.eval("y = 9; y"), "9");
    let pending_across_reset = session.cancel_handle();
    session.reset();
    session.reset();
    old_cancel.cancel();
    pending_across_reset.cancel();

    let symbol = assert_success_text(session.eval("y"), "y");
    assert_eq!(symbol.value.typed, HostTyped::Symbol("y".to_string()));
    assert_success_text(session.eval("2 + 2"), "4");
}

#[test]
fn snapshot_restore_preserves_package_contexts_and_replaces_prior_state() {
    let mut source = HostSession::new();
    assert_success_text(source.eval("BeginPackage[\"Snap`\"]"), "\"Snap`\"");
    successful_eval(source.eval("public::usage = \"public\""));
    assert_success_text(source.eval("Begin[\"`Private`\"]"), "\"Snap`Private`\"");
    assert_success_text(source.eval("helper[x_] := x + 1"), "Null");
    assert_success_text(source.eval("End[]"), "\"Snap`Private`\"");
    assert_success_text(source.eval("public[x_] := Snap`Private`helper[x]"), "Null");
    assert_success_text(source.eval("EndPackage[]"), "Null");
    assert_success_text(source.eval("Snap`public[4]"), "5");

    let bytes = source.snapshot_to_bytes().expect("snapshot should render");
    assert!(!bytes.is_empty());

    let mut restored = HostSession::new();
    assert_success_text(restored.eval("stale = 99"), "99");
    restored
        .restore_from_bytes(&bytes)
        .expect("snapshot bytes should restore");

    let definitions = restored.definitions();
    assert!(
        definitions
            .iter()
            .any(|summary| summary.name == "Snap`public"),
        "missing Snap`public in {definitions:?}"
    );
    assert!(
        definitions
            .iter()
            .any(|summary| summary.name == "Snap`Private`helper"),
        "missing private helper in {definitions:?}"
    );
    let contexts = restored.contexts();
    assert!(contexts.iter().any(|context| context == "Snap`"));
    assert!(contexts.iter().any(|context| context == "Snap`Private`"));

    assert_success_text(restored.eval("public[9]"), "10");
    assert_success_text(restored.eval("stale"), "stale");
}

#[test]
fn terminated_snapshot_restore_fails_while_unlimited_restore_succeeds() {
    // Contract: HostSession::restore_from_bytes and the embedding guide's
    // Snapshot And Restore section. Zero allowed evaluator depth rejects Set
    // at its first evaluation, without timing, memory sampling, or threads.
    let mut source = HostSession::new();
    let success = assert_success_text(source.eval("a = 1"), "1");
    assert_eq!(success.termination, None);
    let bytes = source.snapshot_to_bytes().expect("snapshot should render");

    // The exact same image must restore completely without the low depth limit.
    let mut control = HostSession::new();
    assert_eq!(control.restore_from_bytes(&bytes), Ok(()));
    assert_eq!(control.definitions(), source.definitions());
    let success = assert_success_text(control.eval("a"), "1");
    assert_eq!(success.termination, None);

    let limits = HostResourceLimits {
        recursion_limit: 0,
        ..HostResourceLimits::default()
    };
    let mut restored = HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build();

    // Prove this is a synthesized termination in an Ok runtime response, not
    // an unrelated syntax or request failure that restore already rejected.
    let terminated = successful_eval(restored.eval_script(std::str::from_utf8(&bytes).unwrap()));
    assert_eq!(
        terminated.termination,
        Some(HostTerminationReason::RecursionLimit),
        "snapshot: {}; outcome: {terminated:?}",
        std::str::from_utf8(&bytes).unwrap()
    );
    assert_eq!(
        terminated.messages,
        vec![HostMessage {
            symbol: "General".to_string(),
            tag: "reclim".to_string(),
            text: "evaluation exceeded recursion limit (0)".to_string(),
        }]
    );
    assert!(terminated.prints.is_empty());

    assert_eq!(
        restored.restore_from_bytes(&bytes),
        Err(HostFailure {
            code: HostFailureCode::Other,
            exit_status: None,
            message: "snapshot restore terminated: RecursionLimit".to_string(),
            syntax_rendered: None,
            messages: Vec::new(),
            prints: Vec::new(),
            trace: Vec::new(),
            outputs: Vec::new(),
        })
    );

    assert!(restored.definitions().is_empty());
}

#[test]
fn restore_from_invalid_bytes_returns_failure_without_panic_or_state_loss() {
    let mut session = HostSession::new();
    assert_success_text(session.eval("keep = 11"), "11");

    let failure = session
        .restore_from_bytes(&[0xff, 0xfe, 0xfd])
        .expect_err("invalid UTF-8 snapshot should fail");

    assert_eq!(failure.code, HostFailureCode::Other);
    assert!(failure.message.contains("not valid UTF-8"));
    assert_success_text(session.eval("keep"), "11");
}

#[test]
fn definitions_and_contexts_start_clean_and_track_user_state() {
    let mut session = HostSession::new();

    assert!(session.definitions().is_empty());
    assert_eq!(
        session.contexts(),
        vec!["Global`".to_string(), "System`".to_string()]
    );

    assert_success_text(session.eval("a = 1; f[x_] := x"), "Null");
    assert_success_text(session.eval("BeginPackage[\"Life`\"]"), "\"Life`\"");
    assert_success_text(session.eval("token = 3"), "3");
    assert_success_text(session.eval("EndPackage[]"), "Null");

    assert_eq!(
        session.definitions(),
        vec![
            HostDefinitionSummary {
                name: "Global`a".to_string(),
                has_own_value: true,
                down_value_count: 0,
                up_value_count: 0,
                sub_value_count: 0,
            },
            HostDefinitionSummary {
                name: "Global`f".to_string(),
                has_own_value: false,
                down_value_count: 1,
                up_value_count: 0,
                sub_value_count: 0,
            },
            HostDefinitionSummary {
                name: "Life`token".to_string(),
                has_own_value: true,
                down_value_count: 0,
                up_value_count: 0,
                sub_value_count: 0,
            },
        ]
    );
    assert!(session.contexts().iter().any(|context| context == "Life`"));
}

#[test]
fn structured_host_value_depth_cap_preserves_text_and_uses_other_for_deep_tail() {
    let mut session = HostSession::new();
    let mut expr = "1".to_string();
    for _ in 0..70 {
        expr = format!("{{{expr}}}");
    }

    let success = successful_eval(session.eval(expr));

    assert!(success.value.text.starts_with("List["));
    assert!(
        typed_contains_other(&success.value.typed),
        "deep typed value should contain capped Other tail: {:?}",
        success.value.typed
    );

    let mut association_at_cap = r#"<|"a" -> 1|>"#.to_string();
    for _ in 0..64 {
        association_at_cap = format!("{{{association_at_cap}}}");
    }

    let success = successful_eval(session.eval(association_at_cap));
    let mut typed_tail = &success.value.typed;
    for depth in 0..64 {
        match typed_tail {
            HostTyped::List(items) if items.len() == 1 => typed_tail = &items[0],
            other => panic!("expected singleton list at typed depth {depth}, got {other:?}"),
        }
    }
    assert!(
        matches!(
            typed_tail,
            HostTyped::Other { head, .. } if head == "Association"
        ),
        "association at the typed depth cap should remain Other: {typed_tail:?}"
    );
}

#[test]
fn host_session_and_cancel_handle_are_send_across_worker_thread() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<HostSession>();
    assert_send::<HostCancelHandle>();
    assert_sync::<HostCancelHandle>();

    let session = HostSession::new();
    let worker = thread::spawn(move || {
        let mut session = session;
        assert_success_text(session.eval("workerValue = 21; workerValue * 2"), "42");
        session
    });

    let mut session = worker.join().expect("worker should return session");
    assert_success_text(session.eval("workerValue"), "21");
}

#[test]
fn pending_input_request_tracks_suspension_and_clears_after_resume() {
    let mut session = HostSession::new();

    match session.eval("Input[\"n: \"] + 2") {
        HostEvalOutcome::InputRequired(suspension) => {
            assert_eq!(suspension.request.kind, HostInputKind::Input);
            assert_eq!(suspension.request.prompt, "n: ");
            assert_eq!(
                session
                    .pending_input_request()
                    .map(|request| request.prompt),
                Some("n: ".to_string())
            );
        }
        other => panic!("expected input suspension, got {other:?}"),
    }

    assert_success_text(session.resume_with_input("5"), "7");
    assert!(session.pending_input_request().is_none());
    assert_success_text(session.eval("1 + 1"), "2");
}

#[test]
fn resource_limit_returns_terminated_evaluation_and_session_remains_usable() {
    let mut session = host_session_with_iteration_work_limit(2);

    assert_terminated_value(
        session.eval("Select[{2, 4, 6}, OddQ]"),
        "TerminatedEvaluation[\"IterationWorkLimit\"]",
        "reslim",
    );

    assert_success_text(session.eval("1 + 1"), "2");
}

#[test]
fn replace_repeated_limit_returns_terminated_success() {
    let mut session = HostSession::new();

    assert_terminated_value(
        session.eval("1 //. x_ :> x + 1"),
        "TerminatedEvaluation[\"IterationLimit\"]",
        "iterlim",
    );
}
