//! Prepared arguments complete before the later call can Return or Throw.
use crate::runtime::kernel_api::{BuiltinSpec, MemoryGrowthContract};
use crate::runtime::profiling::logical::test_support;
use crate::runtime::{EvalError, Expr, Session, SharedArgs};

const PREPARE_KEY: &str = "Runtime { phase: \"call.stage.prepare_arguments\", source: None }";

fn prepared_value(_: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    Ok(args[0].clone())
}

fn run(queued: bool, pauses: usize, terminal: &str) -> ([u64; 10], String) {
    let _enabled = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    session.register_builtin(BuiltinSpec::ordinary(
        "PreparedStageValue",
        "Return the first genuinely prepared argument.",
        prepared_value,
        MemoryGrowthContract::Delegated,
    ));
    session.enable_tracing(true);
    let arguments = (1..=pauses)
        .map(|index| format!("Input[\"prepared{index}\"]"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!("{terminal}[PreparedStageValue[{arguments}]]");
    if queued {
        for _ in 0..pauses {
            session.queue_input_response("83");
        }
    }
    let mut outcome = session.eval_request(&source);
    if !queued {
        for index in 1..=pauses {
            assert!(outcome.is_err(), "pause {index}: {outcome:?}");
            assert_eq!(
                session
                    .pending_input_request()
                    .expect("real paused argument preparation")
                    .prompt,
                format!("prepared{index}"),
            );
            assert!(
                Session::logical_profile_test_rows()
                    .iter()
                    .any(|(key, counts)| key == PREPARE_KEY && counts[9] > 0)
            );
            outcome = session.resume_eval_request_with_input("83");
        }
    }
    assert!(session.pending_input_request().is_none());
    let language = format!("{outcome:?} trace={:?}", session.take_trace());
    let rows = Session::logical_profile_test_rows();
    assert!(
        rows.iter()
            .all(|(_, counts)| counts[8] == 0 && counts[9] == 0)
    );
    let counts = rows
        .iter()
        .find(|(key, _)| key == PREPARE_KEY)
        .expect("preparation row")
        .1;
    assert!(counts[0] > 0);
    assert_eq!(counts[0], counts[1] + counts[2]);
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(Session::logical_profile_test_leases(), 0);
    (counts, language)
}

fn compare(pauses: usize, terminal: &str) {
    let queued = run(true, pauses, terminal);
    let resumed = run(false, pauses, terminal);
    assert_eq!(resumed.1, queued.1, "language: {pauses} pauses, {terminal}");
    assert_eq!(
        resumed.0, queued.0,
        "whole preparation row, completion once: {pauses} pauses, {terminal}"
    );
}

#[test]
fn prepared_stage_completes_before_return() {
    compare(1, "Return");
}

#[test]
fn prepared_stage_completes_before_throw() {
    compare(1, "Throw");
}

#[test]
fn prepared_stage_two_pauses_complete_once_before_return() {
    compare(2, "Return");
}

#[test]
fn prepared_stage_two_pauses_complete_once_before_throw() {
    compare(2, "Throw");
}
