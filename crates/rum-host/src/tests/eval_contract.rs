use super::super::*;
use super::common::*;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const CANCELLABLE_LONG_RUNNING_EVAL: &str = "Print[\"evaluation-started\"]; Do[x = 1, 50000000]";

#[test]
fn host_eval_script_uses_last_form_and_classifies_syntax_failures() {
    let mut session = HostSession::new();

    let success = successful_eval(
        session.eval_script("scriptValue = 40\nPrint[scriptValue]\nscriptValue + 2"),
    );
    assert_eq!(success.value.text, "42");
    assert_eq!(success.prints, vec!["40"]);
    assert_eq!(
        successful_eval(session.eval("scriptValue")).value.text,
        "40"
    );

    match session.eval_script("validForm = 1\nf[1,]") {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Syntax);
        }
        other => panic!("expected script syntax failure, got {other:?}"),
    }
}

#[test]
fn host_eval_file_uses_provider_and_missing_provider_is_other() {
    let dir = TempDirGuard::new("eval-file");
    let path = dir.write("batch.rum", "fileValue = 40\nfileValue + 2");
    let mut session = HostSession::new();

    let success = successful_eval(session.eval_file(&path));
    assert_eq!(success.value.text, "42");
    assert_eq!(successful_eval(session.eval("fileValue")).value.text, "40");

    let mut sandboxed = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();
    match sandboxed.eval_file(&path) {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Other);
            assert_eq!(failure.message, "no file provider installed");
        }
        other => panic!("expected provider-less file failure, got {other:?}"),
    }
}

#[test]
fn host_eval_script_suspends_but_resume_fails_without_running_remaining_forms() {
    let mut session = HostSession::new();

    let suspended =
        session.eval_script("scriptBefore = 1\nInputString[\"value: \"]\nscriptAfter = 3");
    match suspended {
        HostEvalOutcome::InputRequired(suspension) => {
            assert_eq!(suspension.request.kind, HostInputKind::InputString);
            assert_eq!(suspension.request.prompt, "value: ");
        }
        other => panic!("expected script input suspension, got {other:?}"),
    }

    match session.resume_with_input("hello") {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Other);
            assert_eq!(
                failure.message,
                "no suspended evaluation is waiting for host input"
            );
        }
        other => panic!("expected non-resumable script failure, got {other:?}"),
    }

    let definitions = session.definitions();
    assert!(
        definitions
            .iter()
            .any(|definition| definition.name == "Global`scriptBefore")
    );
    assert!(
        definitions
            .iter()
            .all(|definition| definition.name != "Global`scriptAfter")
    );
}

#[test]
fn host_iteration_accumulator_completes_after_multiple_inputs_without_poisoning() {
    let mut session = HostSession::new();

    assert!(matches!(
        session.eval("Table[InputString[], 2]"),
        HostEvalOutcome::InputRequired(_)
    ));
    assert!(matches!(
        session.resume_with_input("0"),
        HostEvalOutcome::InputRequired(_)
    ));

    let success = successful_eval(session.resume_with_input("1"));
    assert_eq!(success.value.text, "List[\"0\", \"1\"]");
    assert!(success.messages.is_empty());
    assert_eq!(success.termination, None);
    assert!(!session.poisoned_for_test());

    let next = successful_eval(session.eval("1 + 1"));
    assert_eq!(next.value.text, "2");
    assert_eq!(next.termination, None);
}

#[test]
fn host_failures_use_stable_failure_codes() {
    let mut session = HostSession::new();

    let expected_syntax_rendered = match rum_runtime::Session::new()
        .eval_request("f[1,]")
        .expect_err("malformed source should produce a syntax request error")
        .error
    {
        rum_runtime::SessionRequestError::Syntax(diagnostic) => diagnostic.rendered,
        other => panic!("expected runtime syntax failure, got {other:?}"),
    };

    // `1 2` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let outcome = session.eval("f[1,]");

    match outcome {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Syntax);
            assert_eq!(failure.exit_status, None);
            assert!(!failure.message.is_empty());
            assert_eq!(
                failure.syntax_rendered.as_deref(),
                Some(expected_syntax_rendered.as_str())
            );
        }
        other => panic!("expected syntax failure, got {other:?}"),
    }

    match session.eval("Exit[7]") {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Exit);
            assert_eq!(failure.syntax_rendered, None);
        }
        other => panic!("expected non-syntax failure, got {other:?}"),
    }
}

#[test]
fn host_failure_codes_report_session_reset_requirement() {
    for code in [HostFailureCode::Panic, HostFailureCode::Poisoned] {
        assert!(code.requires_reset(), "{code:?} should require reset");
    }

    for code in [
        HostFailureCode::Cancelled,
        HostFailureCode::TimeConstraintExceeded,
        HostFailureCode::Syntax,
        HostFailureCode::Exit,
    ] {
        assert!(!code.requires_reset(), "{code:?} should not require reset");
    }
}

#[test]
fn host_eval_ordinary_value_has_no_termination_reason() {
    let mut session = HostSession::new();

    let success = successful_eval(session.eval("1 + 1"));

    assert_eq!(success.value.text, "2");
    assert_eq!(success.termination, None);
}

#[test]
fn host_eval_literal_terminated_evaluation_has_no_termination_reason() {
    let mut session = HostSession::new();

    let success = successful_eval(session.eval("TerminatedEvaluation[\"Cancelled\"]"));

    assert_eq!(success.value.text, "TerminatedEvaluation[\"Cancelled\"]");
    assert_eq!(success.termination, None);
}

#[test]
fn host_eval_recursion_limit_reports_structured_termination_reason() {
    let mut limits = HostResourceLimits::default();
    limits.recursion_limit = 32;
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build();

    let success = successful_eval(session.eval("f[x_] := f[x + 1]; f[0]"));

    assert_eq!(
        success.value.text,
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert_eq!(
        success.termination,
        Some(HostTerminationReason::RecursionLimit)
    );
}

#[test]
fn host_eval_memory_ceiling_reports_structured_termination_and_session_recovers() {
    let mut limits = HostResourceLimits::default();
    limits.eval_memory_limit = Some(100_000);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build();

    let success = successful_eval(session.eval("Table[k, {k, 10000}]"));

    assert_eq!(success.value.text, "TerminatedEvaluation[\"MemoryLimit\"]");
    assert_eq!(
        success.termination,
        Some(HostTerminationReason::MemoryLimit)
    );
    assert!(
        success
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "memlim")
    );

    let recovered = successful_eval(session.eval("1 + 1"));
    assert_eq!(recovered.value.text, "2");
    assert_eq!(recovered.termination, None);
}

#[test]
fn host_cancel_handle_interrupts_running_eval_from_another_thread() {
    let mut limits = HostResourceLimits::default();
    limits.iteration_work_limit = 50_000_000;
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build();
    let cancel = session.cancel_handle();
    let (cancelled_tx, cancelled_rx) = mpsc::channel();
    let (evaluation_started_tx, evaluation_started_rx) = mpsc::sync_channel(0);
    session.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
        assert_eq!(line, "evaluation-started");
        evaluation_started_tx
            .send(())
            .expect("canceller should receive evaluation-start signal");
    }));

    let canceller = {
        let cancel = cancel.clone();
        thread::spawn(move || {
            evaluation_started_rx
                .recv()
                .expect("evaluation should report that it started");
            let cancelled_at = Instant::now();
            cancel.cancel();
            cancelled_tx
                .send(cancelled_at)
                .expect("test should receive cancel timestamp");
        })
    };

    let started = Instant::now();
    let outcome = session.eval(CANCELLABLE_LONG_RUNNING_EVAL);
    let completed_at = Instant::now();
    let elapsed = completed_at.duration_since(started);
    let cancelled_at = cancelled_rx
        .recv()
        .expect("canceller should report cancel timestamp");
    canceller.join().expect("canceller should not panic");
    let cancel_latency = completed_at.duration_since(cancelled_at);

    eprintln!(
        "host_cancel_handle_interrupts_running_eval_from_another_thread elapsed_ms={} cancel_latency_ms={}",
        elapsed.as_millis(),
        cancel_latency.as_millis()
    );

    assert!(
        elapsed < Duration::from_secs(2),
        "cancelled eval took {elapsed:?}"
    );
    assert!(
        cancel_latency < Duration::from_secs(1),
        "cancel observation took {cancel_latency:?}"
    );

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "TerminatedEvaluation[\"Cancelled\"]");
            assert_eq!(success.termination, Some(HostTerminationReason::Cancelled));
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "cancel")
            );
        }
        other => panic!("expected graceful cancelled value, got {other:?}"),
    }
}

#[test]
fn host_eval_with_timeout_interrupts_running_eval() {
    let mut session = host_session_with_large_iteration_budget();

    let started = Instant::now();
    let outcome = session.eval_with_timeout(LONG_RUNNING_EVAL, Duration::from_millis(200));
    let elapsed = started.elapsed();

    eprintln!(
        "host_eval_with_timeout_interrupts_running_eval elapsed_ms={}",
        elapsed.as_millis()
    );

    assert!(
        elapsed < Duration::from_secs(2),
        "timed eval took {elapsed:?}"
    );

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(
                success.value.text,
                "TerminatedEvaluation[\"TimeConstraint\"]"
            );
            assert_eq!(
                success.termination,
                Some(HostTerminationReason::TimeConstraint)
            );
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "timelim")
            );
        }
        other => panic!("expected graceful timeout value, got {other:?}"),
    }
}

#[test]
fn host_eval_with_unused_handle_and_generous_timeout_still_succeeds() {
    let mut session = HostSession::new();
    let _cancel = session.cancel_handle();

    let started = Instant::now();
    let outcome = session.eval_with_timeout("1 + 2", Duration::from_secs(5));
    let elapsed = started.elapsed();

    eprintln!(
        "host_eval_with_unused_handle_and_generous_timeout_still_succeeds elapsed_ms={}",
        elapsed.as_millis()
    );

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "3");
            assert_eq!(success.termination, None);
        }
        other => panic!("expected normal success, got {other:?}"),
    }
}
