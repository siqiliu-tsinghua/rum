use rum_host::{
    HostConfig, HostDefinitionSummary, HostEvalOutcome, HostEvalSuccess, HostResourceLimits,
    HostSession, HostStructuredOutputKind, native_stack_limit_for_declared_stack,
};
use rum_runtime::Session;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const REWRITE_LOOP_UNDER_LOAD: &str =
    "expr = 0; Do[expr = expr /. n_Integer :> n + 1, 50000000]; expr";
const CANCELLABLE_REWRITE_LOOP_UNDER_LOAD: &str = "Print[\"evaluation-started\"]; expr = 0; \
     Do[expr = expr /. n_Integer :> n + 1, 50000000]; expr";
const BATTERED_SESSION_STACK_BYTES: usize = 8 * 1024 * 1024;

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

fn rewrite_load_session() -> HostSession {
    let mut limits = HostResourceLimits::default();
    limits.iteration_work_limit = 50_000_000;
    limits.evaluation_pass_limit = 50_000_000;
    HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build()
}

#[test]
fn cancel_interrupts_heavy_rewrite_loop_and_session_recovers() {
    let mut session = rewrite_load_session();
    let cancel = session.cancel_handle();
    let (cancelled_tx, cancelled_rx) = mpsc::channel();
    let (evaluation_started_tx, evaluation_started_rx) = mpsc::sync_channel(0);
    session.set_print_sink(move |line| {
        assert_eq!(line, "evaluation-started");
        evaluation_started_tx
            .send(())
            .expect("canceller should receive evaluation-start signal");
    });

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
    let outcome = session.eval(CANCELLABLE_REWRITE_LOOP_UNDER_LOAD);
    let completed_at = Instant::now();
    let elapsed = completed_at.duration_since(started);
    let cancelled_at = cancelled_rx
        .recv()
        .expect("canceller should report cancel timestamp");
    canceller.join().expect("canceller should not panic");
    let cancel_latency = completed_at.duration_since(cancelled_at);

    assert!(
        elapsed < Duration::from_secs(5),
        "cancelled rewrite eval took {elapsed:?}"
    );
    assert!(
        cancel_latency < Duration::from_secs(2),
        "cancel observation took {cancel_latency:?}"
    );
    assert_terminated_value(outcome, "TerminatedEvaluation[\"Cancelled\"]", "cancel");
    assert_success_text(session.eval("10 + 1"), "11");
}

#[test]
fn eval_with_timeout_interrupts_heavy_rewrite_loop_and_session_recovers() {
    let mut session = rewrite_load_session();

    let started = Instant::now();
    let outcome = session.eval_with_timeout(REWRITE_LOOP_UNDER_LOAD, Duration::from_millis(100));
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_secs(5),
        "timed rewrite eval took {elapsed:?}"
    );
    assert_terminated_value(
        outcome,
        "TerminatedEvaluation[\"TimeConstraint\"]",
        "timelim",
    );
    assert_success_text(session.eval("12 + 1"), "13");
}

/// Run with:
/// `cargo test -p rum-host --test stress cancel_rewrite_loop_stability_10_cycles -- --ignored --nocapture`
#[test]
#[ignore]
fn cancel_rewrite_loop_stability_10_cycles() {
    let mut session = rewrite_load_session();

    for cycle in 0..10 {
        let cancel = session.cancel_handle();
        let canceller = thread::spawn(move || {
            thread::sleep(Duration::from_millis(25));
            cancel.cancel();
        });

        assert_terminated_value(
            session.eval(REWRITE_LOOP_UNDER_LOAD),
            "TerminatedEvaluation[\"Cancelled\"]",
            "cancel",
        );
        canceller.join().expect("canceller should not panic");
        assert_success_text(
            session.eval(format!("{cycle} + 100")),
            &(cycle + 100).to_string(),
        );
    }
}

fn install_snapshot_definitions(session: &mut HostSession, count: usize) {
    for index in 0..count {
        assert_success_text(
            session.eval(format!(
                "SetAttributes[snap{index}, Orderless]; snap{index}[x_] := x + {index}"
            )),
            "Null",
        );
    }
}

fn definition_summary(definitions: &[HostDefinitionSummary], name: &str) -> HostDefinitionSummary {
    definitions
        .iter()
        .find(|summary| summary.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("missing {name} in {definitions:?}"))
}

#[test]
fn snapshot_restore_preserves_20_generated_definitions_and_attributes() {
    let mut source = HostSession::new();
    install_snapshot_definitions(&mut source, 20);

    let source_definitions = source.definitions();
    assert_eq!(source_definitions.len(), 20);
    assert_eq!(
        definition_summary(&source_definitions, "Global`snap17"),
        HostDefinitionSummary {
            name: "Global`snap17".to_string(),
            has_own_value: false,
            down_value_count: 1,
            up_value_count: 0,
            sub_value_count: 0,
        }
    );
    assert_success_text(source.eval("snap17[25]"), "42");
    assert_success_text(source.eval("Attributes[snap17]"), "List[Orderless]");

    let bytes = source.snapshot_to_bytes().expect("snapshot should render");
    assert!(!bytes.is_empty());

    let mut restored = HostSession::new();
    assert_success_text(restored.eval("stale = 99"), "99");
    restored
        .restore_from_bytes(&bytes)
        .expect("snapshot bytes should restore");

    let restored_definitions = restored.definitions();
    assert_eq!(restored_definitions.len(), source_definitions.len());
    assert_eq!(
        definition_summary(&restored_definitions, "Global`snap19").down_value_count,
        1
    );
    for (name, expected) in [
        ("snap0[10]", "10"),
        ("snap17[25]", "42"),
        ("snap18[49]", "67"),
        ("snap19[1]", "20"),
    ] {
        assert_success_text(restored.eval(name), expected);
    }
    assert_success_text(restored.eval("Attributes[snap19]"), "List[Orderless]");
    assert_success_text(restored.eval("stale"), "stale");
}

/// Run with:
/// `cargo test -p rum-host --test stress bug_snapshot_restore_500_generated_definitions_overflows_native_stack -- --nocapture`
#[test]
fn bug_snapshot_restore_500_generated_definitions_overflows_native_stack() {
    let mut source = HostSession::new();
    install_snapshot_definitions(&mut source, 500);

    let source_definitions = source.definitions();
    assert_eq!(source_definitions.len(), 500);
    assert_eq!(
        definition_summary(&source_definitions, "Global`snap499"),
        HostDefinitionSummary {
            name: "Global`snap499".to_string(),
            has_own_value: false,
            down_value_count: 1,
            up_value_count: 0,
            sub_value_count: 0,
        }
    );

    let bytes = source.snapshot_to_bytes().expect("snapshot should render");
    assert!(!bytes.is_empty());

    let mut restored = HostSession::new();
    assert_success_text(restored.eval("stale = 99"), "99");
    restored
        .restore_from_bytes(&bytes)
        .expect("500-definition snapshot bytes should restore");

    let restored_definitions = restored.definitions();
    assert_eq!(restored_definitions.len(), source_definitions.len());
    for index in [0usize, 1, 17, 250, 499] {
        assert_eq!(
            definition_summary(&restored_definitions, &format!("Global`snap{index}"))
                .down_value_count,
            1
        );
        assert_success_text(
            restored.eval(format!("snap{index}[10]")),
            &(10 + index).to_string(),
        );
    }
    assert_success_text(restored.eval("Attributes[snap499]"), "List[Orderless]");
    assert_success_text(restored.eval("stale"), "stale");
}

fn runtime_core_session() -> Session {
    let mut session = Session::new();
    rum_stdlib::install_core(&mut session);
    session
}

fn runtime_eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn assert_runtime_termination(
    session: &mut Session,
    source: &str,
    expected_text: &str,
    expected_tag: &str,
) {
    let outcome = session.eval_text(source).unwrap();
    assert_eq!(outcome.value.to_string(), expected_text);
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == expected_tag),
        "missing General::{expected_tag} in {:?}",
        outcome.messages
    );
}

/// Run with:
/// `cargo test -p rum-host --test stress battered_session_recovers_after_repeated_recoverable_limit_trips -- --ignored --nocapture`
#[test]
#[ignore]
fn battered_session_recovers_after_repeated_recoverable_limit_trips() {
    thread::Builder::new()
        .stack_size(BATTERED_SESSION_STACK_BYTES)
        .spawn(run_battered_session_recoverable_limit_trips)
        .expect("battered-session stress thread should start")
        .join()
        .expect("battered-session stress should finish");
}

fn run_battered_session_recoverable_limit_trips() {
    let mut session = runtime_core_session();
    let declared_stack_limit = native_stack_limit_for_declared_stack(BATTERED_SESSION_STACK_BYTES)
        .expect("battered-session stack should satisfy the supported minimum");

    for cycle in 0..3 {
        session.set_recursion_limit(32);
        session.set_native_stack_limit(declared_stack_limit);
        assert_runtime_termination(
            &mut session,
            &format!("rec{cycle}[x_] := rec{cycle}[f[x]]; rec{cycle}[1]"),
            "TerminatedEvaluation[\"RecursionLimit\"]",
            "reclim",
        );
        assert_eq!(runtime_eval_to_string(&mut session, "1 + 1"), "2");

        session.set_recursion_limit(4096);
        session.set_native_stack_limit(512 * 1024);
        assert_runtime_termination(
            &mut session,
            &format!("stk{cycle}[n_] := stk{cycle}[n + 1]; stk{cycle}[0]"),
            "TerminatedEvaluation[\"StackLimit\"]",
            "reslim",
        );
        assert_eq!(runtime_eval_to_string(&mut session, "2 + 2"), "4");

        session.set_native_stack_limit(declared_stack_limit);
        session.set_iteration_work_limit(2);
        assert_runtime_termination(
            &mut session,
            "Select[{2, 4, 6}, OddQ]",
            "TerminatedEvaluation[\"IterationWorkLimit\"]",
            "reslim",
        );
        assert_eq!(runtime_eval_to_string(&mut session, "3 + 3"), "6");
        session.set_iteration_work_limit(100_000);

        session.set_generated_element_limit(3);
        assert_runtime_termination(
            &mut session,
            "Range[4]",
            "TerminatedEvaluation[\"GeneratedElementLimit\"]",
            "reslim",
        );
        assert_eq!(runtime_eval_to_string(&mut session, "4 + 4"), "8");
        session.set_generated_element_limit(100_000);

        session.set_replace_repeated_limit(3);
        assert_runtime_termination(
            &mut session,
            "1 //. x_ :> x + 1",
            "TerminatedEvaluation[\"IterationLimit\"]",
            "iterlim",
        );
        assert_eq!(runtime_eval_to_string(&mut session, "5 + 5"), "10");
        session.set_replace_repeated_limit(256);

        assert_runtime_termination(
            &mut session,
            &format!(
                "SetAttributes[ord{cycle}, Orderless]; \
                 MatchQ[ord{cycle} @@ Range[14], ord{cycle}[a___, b___, c___] /; False]"
            ),
            "TerminatedEvaluation[\"PatternMatchRecursionLimit\"]",
            "matchreclim",
        );
        assert_eq!(runtime_eval_to_string(&mut session, "6 + 6"), "12");
    }
}
