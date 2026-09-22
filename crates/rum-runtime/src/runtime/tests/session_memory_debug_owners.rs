use super::*;
use crate::runtime::suspension::AbandonReason;

#[test]
fn maintained_reap_monitor_append_replace_resume_and_abandon() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for source in [
        "Reap[Do[Sow[{i523, i523}, i523], {i523, 40}]]",
        "Reap[Reap[Do[Sow[{i523, i523}, repeated523], {i523, 40}]], {_, repeated523}]",
        "Monitor[display523 = 1; display523 = {1, 2, 3}; display523, display523]",
        "ValueQ[Reap[Sow[1]; unknown523]]",
    ] {
        session.eval_text(source).unwrap();
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .unwrap();
    }
    assert!(
        session
            .eval_text("Reap[Sow[{1, 2}]; InputString[\"reap\"]]")
            .is_err()
    );
    session.resume_suspended_evaluation("done").unwrap();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    assert!(
        session
            .eval_text("Monitor[Reap[Sow[3]; InputString[\"monitor\"]], display523]")
            .is_err()
    );
    session.abandon_pending_suspension(AbandonReason::CheckpointRollbackDiscard);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

#[test]
fn maintained_message_print_trace_capture_and_rollback() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for source in [
        "message523::tag = \"first `1`\"; Message[message523::tag, {1, 2}]; Print[{1, 2}]",
        "Off[message523::tag]; Message[message523::tag, 3]; On[message523::tag]; Message[message523::tag, 4]",
        "message523::tag := \"second\"; Block[{message523}, message523::tag = \"inner\"; Message[message523::tag]]; message523::tag =.",
        "Quiet[1/0]; Check[1/0, 7]; Trace[trace523 = 1; trace523 + 2]",
        "EvaluationData[Print[1]; 1/0]; ValueQ[Print[2]; Message[message523::tag]; unknown523]",
        "message523::tag = \"last\"; ClearAll[message523]",
    ] {
        session.eval_text(source).unwrap();
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .unwrap();
    }
    session.enable_tracing(true);
    session.eval_text("trace523 = {1, 2, 3}; trace523").unwrap();
    session.take_trace();
    session.take_messages();
    session.take_prints();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    assert!(
        session
            .eval_text("EvaluationData[Print[3]; InputString[\"capture\"]]")
            .is_err()
    );
    session.abandon_pending_suspension(AbandonReason::CheckpointRollbackDiscard);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}
