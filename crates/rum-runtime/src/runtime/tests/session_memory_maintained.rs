use super::*;
use crate::runtime::suspension::AbandonReason;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn maintained_stack_checkpoint_workloads() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for source in [
        "x522 = {1, 2, 3}; f522[x_Integer] := x + 1; f522[2]",
        "f522[2]; f522[2]; Clear[f522]; x522 = {x522, x522}",
        "Begin[\"Ledger522`\"]; local = 1; End[]",
        "Block[{x522 = 7}, x522 + 1]",
        "Module[{local = 4}, local + 1]",
        "Print[\"ledger\"]; Trace[x522]",
        "Reap[Sow[1]; Sow[{2, 3}]]",
        "Check[Message[General::argx, f, 2], fallback]",
        "ValueQ[Print[\"rolled back\"]; x522]",
    ] {
        session.eval_text(source).unwrap();
        session.assert_evaluation_stack_accounting();
    }

    let thrown = session.lower_text("Throw[7]").unwrap();
    assert!(session.evaluate(thrown).is_err());
    session.assert_evaluation_stack_accounting();

    let suspended = session
        .lower_text("Block[{x522 = 9}, Reap[Sow[InputString[\"resume\"]]]]")
        .unwrap();
    assert!(matches!(
        session.evaluate(suspended),
        Err(EvalError::InputRequired { .. })
    ));
    session.assert_evaluation_stack_accounting();
    session.resume_suspended_evaluation("answer").unwrap();
    session.assert_evaluation_stack_accounting();

    let suspended = session
        .lower_text("Block[{x522 = 10}, Reap[Sow[InputString[\"abandon\"]]]]")
        .unwrap();
    assert!(matches!(
        session.evaluate(suspended),
        Err(EvalError::InputRequired { .. })
    ));
    session.abandon_pending_suspension(AbandonReason::CancelledRequest);
    session.assert_evaluation_stack_accounting();
    assert!(session.evaluation_stack.is_empty());
}

#[test]
fn maintained_stack_counts_shared_occurrences_and_retained_capacity() {
    let mut session = Session::empty();
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    let shared = Expr::call(system_expr("List"), vec![Expr::string("payload")]);
    let entry = EvaluationStackEntry {
        expr: shared.clone(),
        epoch: 0,
    };
    let first = session.push_accounted_evaluation_entry(entry.clone());
    let second = session.push_accounted_evaluation_entry(entry);
    assert_eq!(first, second);
    session.assert_evaluation_stack_accounting();
    session.pop_accounted_evaluation_entry(second);
    session.assert_evaluation_stack_accounting();
    session.pop_accounted_evaluation_entry(first);
    session.assert_evaluation_stack_accounting();
    assert!(
        session
            .accounting_route
            .as_ref()
            .unwrap()
            .evaluation_stack_bytes
            .unwrap()
            > 0
    );
    session.replace_accounted_evaluation_stack(Vec::new());
    assert_eq!(
        session
            .accounting_route
            .as_ref()
            .unwrap()
            .evaluation_stack_bytes,
        Some(0)
    );
    session.assert_evaluation_stack_accounting();
}

fn panic_after_stack_charge(_: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    panic!("intentional retained-stack panic");
}

#[test]
fn maintained_stack_survives_panic_and_rollback() {
    let mut session = Session::new();
    session.register_builtin(BuiltinSpec::ordinary(
        "PanicAfterStackCharge522",
        "Tests retained stack accounting after unwinding.",
        panic_after_stack_charge,
        MemoryGrowthContract::Delegated,
    ));
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    let snapshot = session.capture_evaluation_rollback_snapshot();
    let expr = session.lower_text("PanicAfterStackCharge522[]").unwrap();
    assert!(catch_unwind(AssertUnwindSafe(|| session.evaluate(expr))).is_err());
    // Existing evaluator panic behavior retains the root. Keep charging it
    // until rollback actually replaces the stack; do not invent a release.
    assert!(!session.evaluation_stack.is_empty());
    session.assert_evaluation_stack_accounting();
    session.restore_evaluation_rollback_snapshot(snapshot);
    session.assert_evaluation_stack_accounting();
    assert!(session.evaluation_stack.is_empty());
}

#[test]
fn maintained_stack_rearms_from_scratch_and_observation_keeps_walking() {
    let mut session = session_with_test_control_builtins();
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    session.current_memory_limit = None;
    let before = session
        .accounting_route
        .as_ref()
        .unwrap()
        .evaluation_stack_bytes;
    session
        .eval_text("f522[x_] := x; f522[f522[{1, 2, 3}]]")
        .unwrap();
    assert_eq!(
        session
            .accounting_route
            .as_ref()
            .unwrap()
            .evaluation_stack_bytes,
        before
    );

    // Cold owner transfers do not maintain anything while unarmed either.
    session.replace_accounted_evaluation_stack(vec![EvaluationStackEntry {
        expr: Expr::string("new root retained while unarmed"),
        epoch: 0,
    }]);
    assert_eq!(
        session
            .accounting_route
            .as_ref()
            .unwrap()
            .evaluation_stack_bytes,
        before
    );
    session
        .accounting_route
        .as_mut()
        .unwrap()
        .evaluation_stack_bytes = None;
    assert_model(&session); // The observation walker ignores an invalid ledger.
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    session.assert_evaluation_stack_accounting();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();

    let snapshot = session.capture_evaluation_rollback_snapshot();
    session.current_memory_limit = None;
    session.replace_accounted_evaluation_stack(Vec::new());
    session.restore_evaluation_rollback_snapshot(snapshot);
    session.assert_evaluation_stack_accounting();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

#[test]
fn maintained_stack_local_arm_survives_suspend_resume_abandon_and_errors() {
    let mut session = session_with_test_control_builtins();
    for abandon in [false, true] {
        let source = "MemoryConstrained[Block[{x522 = 4}, (InputString[\"first\"]; InputString[\"second\"]; x522)], 1000000000]";
        assert!(matches!(session.eval_text(source), Err(_)));
        assert!(session.current_memory_limit.is_some());
        session.assert_evaluation_stack_accounting();
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .unwrap();
        assert!(matches!(
            session.resume_suspended_evaluation("one"),
            Err(EvalError::InputRequired { .. })
        ));
        session.assert_evaluation_stack_accounting();
        if abandon {
            session.abandon_pending_suspension(AbandonReason::CancelledRequest);
        } else {
            assert_eq!(
                session.resume_suspended_evaluation("two").unwrap(),
                Expr::small_integer(4)
            );
        }
        assert_eq!(session.current_memory_limit, None);
        assert!(session.evaluation_stack.is_empty());
    }
    assert_eq!(
        session
            .eval_text("MemoryConstrained[(1; 2), 0, 123]")
            .unwrap()
            .value,
        Expr::small_integer(123)
    );
    assert_eq!(
        session
            .eval_text("MemoryConstrained[(1; 2), 1000000000]")
            .unwrap()
            .value,
        Expr::small_integer(2)
    );
    assert_eq!(session.current_memory_limit, None);
}
