use super::*;
use crate::runtime::suspension::AbandonReason;
use std::panic::{AssertUnwindSafe, catch_unwind};

struct AccountingEntropy523([u8; 32]);

// Fixed inline bytes have bounded destruction.
impl crate::runtime::host_payload::BoundedDestruction for AccountingEntropy523 {}

impl EntropySource for AccountingEntropy523 {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        buf.fill(self.0[0]);
        Ok(())
    }
}

#[test]
fn maintained_scopes_registry_sealed_units_and_abandonment() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for source in [
        "Block[{dynamic523 = {1, 2}}, Block[{dynamic523 = {3}}, dynamic523]]",
        "Module[{fresh523 = {1, 2, 3}}, savedFresh523 = fresh523]; Clear[savedFresh523]",
        "Options[scoped523] = {option523 -> {1, 2}}; scoped523[OptionsPattern[]] := OptionValue[option523]; scoped523[option523 -> {4, 5, 6}]",
        "RegisterExceptionType[\"Ledger523\"]; ValueQ[RegisterExceptionType[\"Rollback523\"]; unknown523]",
        "DefinePackage[\"LedgerUnit523`\", Public[{f}], f[x_] := helper[x]; helper[x_] := {x}]",
        "DefinePackage[\"LedgerUnit523`\", Public[{f}], f[x_] := {x, x}]",
        "DefinePackage[\"LedgerBroken523`\", Public[{f}], f[] := 4; Throw[1]]",
    ] {
        session.eval_text(source).unwrap();
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .unwrap();
    }
    session
        .eval_text("scoped523[OptionsPattern[]] := InputString[\"options\"]")
        .unwrap();
    assert!(
        session
            .eval_text("scoped523[option523 -> {9, 8, 7}]")
            .is_err()
    );
    session.abandon_pending_suspension(AbandonReason::CheckpointRollbackDiscard);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.collect_and_sweep_dead_gensyms(&[]);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    assert!(
        session
            .eval_text("Module[{local523}, Block[{dynamic523 = 5}, InputString[\"scope\"]]]")
            .is_err()
    );
    session.abandon_pending_suspension(AbandonReason::CheckpointRollbackDiscard);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

#[test]
fn maintained_providers_and_print_sink_take_reinstall() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    session.eval_text("Null").unwrap();
    let retained = [7u8; 64];
    session.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |_line: &str| {
            std::hint::black_box(retained);
        },
    )));
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.eval_text("ValueQ[Print[1]; missing523]").unwrap();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.clear_print_sink();
    session.set_entropy_source(Some(EntropyProvider::new(AccountingEntropy523([3; 32]))));
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.set_entropy_source(None);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.set_file_io(None);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.set_file_io(Some(FileIoProvider::new(TestFileIo)));
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

#[test]
fn maintained_contexts_paths_and_loader_rollback() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for source in [
        "Begin[\"Ledger523`\"]; End[]",
        "BeginPackage[\"LedgerPackage523`\"]; Begin[\"`Private`\"]; EndPackage[]; End[]; EndPackage[]",
        "$ContextPath = {\"System`\", \"Global`\"}; $Path = {\".\", \"target/brief-523\"}; $Packages = {\"LedgerLoaded523`\"}",
        "ValueQ[Begin[\"Rollback523`\"]; unknown523]",
        "BeginPackage[\"FailedLedger523`\", {\"DoesNotExist523`\"}]",
    ] {
        session.eval_text(source).unwrap();
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .unwrap();
    }
    session.set_current_context("HostLedger523`");
    session.set_context_path(["System`", "Global`"]);
    session.set_package_search_paths(["."]);
    session.add_package_search_path("target/brief-523");
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

#[test]
fn maintained_stubs_generations_and_once_restore_and_resume() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for source in [
        "DeclarePackage[\"Ledger523`\", {\"stub\"}]",
        "Remove[Ledger523`stub]; generation523 = 1; Remove[generation523]; generation523 = 2",
        "Once[{1, 2}]; Once[{3, 4, 5}]",
        "ValueQ[Once[{6, 7, 8, 9}]; unassigned523]",
    ] {
        session.eval_text(source).unwrap();
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .unwrap();
    }
    assert!(session.eval_text("Once[InputString[\"once\"]]").is_err());
    session
        .resume_suspended_evaluation("cached response")
        .unwrap();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

#[test]
fn maintained_options_and_syntax_replace_remove_and_rollback() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for source in [
        "Options[option523] = {name523 -> {1, 2, 3}}; SyntaxInformation[option523] = {\"ArgumentsPattern\" -> {_}}",
        "Options[option523] = {name523 -> 8}; SyntaxInformation[option523] = {\"ArgumentsPattern\" -> {_, _}}",
        "Block[{option523}, Options[option523] = {name523 -> 7}]; Options[option523]",
        "ValueQ[Options[option523] = {name523 -> 9}; option523]; Options[option523] =.; SyntaxInformation[option523] =.",
        "Options[option523] = {name523 -> 4}; ClearAll[option523]",
    ] {
        session.eval_text(source).unwrap();
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .unwrap();
    }
}

#[test]
fn maintained_evaluation_caches_charge_evict_and_keep_capacity() {
    let mut session = session_with_test_control_builtins();
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    let shared = Expr::call(system_expr("List"), vec![Expr::string("cached occurrence")]);
    assert!(session.cached_expr_is_evaluation_stable(&shared));
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    let Expr::Call { head, args } = &shared else {
        unreachable!()
    };
    let proof = session.take_call_stability_proof(head, args).unwrap();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.reinsert_call_stability_proof(head, args, proof);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.clear_evaluation_caches();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    for base in 0..5 {
        let association = Expr::call(
            system_expr("Association"),
            (0..64)
                .map(|n| {
                    Expr::call(
                        system_expr("Rule"),
                        vec![Expr::small_integer(n), Expr::small_integer(base + n)],
                    )
                })
                .collect::<Vec<_>>(),
        );
        for _ in 0..4 {
            assert_eq!(
                session.lookup_association_key_value(&association, &Expr::small_integer(8)),
                Some(Some(Expr::small_integer(base + 8)))
            );
            session
                .enforce_active_memory_limit_for_projected_bytes(0)
                .unwrap();
        }
    }
    session
        .eval_text("cacheSymbol523 = 4; cacheSymbol523; Clear[cacheSymbol523]")
        .unwrap();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

#[test]
fn maintained_definitions_store_remove_and_lazy_dispatch() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for i in 0..40 {
        session
            .eval_text(&format!("indexed523[{i}] = {i}"))
            .unwrap();
    }
    assert_eq!(
        session.eval_text("indexed523[17]").unwrap().value,
        Expr::small_integer(17)
    );
    for source in [
        "indexed523[17] = 81; indexed523[18] =.; indexed523[17]",
        "root523 = {1, 2, 3}; root523[[2]] = 9; root523",
        "Block[{root523 = 8}, root523]; Module[{fresh523 = 7}, fresh523]",
        "ValueQ[root523 = {7}; root523]; Clear[root523]; Remove[indexed523]",
    ] {
        session.eval_text(source).unwrap();
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .unwrap();
    }
    session.install_system_own_value("Installed523", Expr::string("one"));
    session.install_system_own_value("Installed523", Expr::string("two"));
    session.install_system_down_value(
        crate::runtime::TrustedSystemDownValueLhs::default_for_plus(),
        Expr::small_integer(0),
    );
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

#[test]
fn maintained_attributes_and_caches_mutate_clear_and_restore() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for source in [
        "SetAttributes[attr523, HoldAll]; Attributes[attr523]",
        "attr523[1 + 2]; ClearAttributes[attr523, HoldAll]; attr523[1 + 2]",
        "SetAttributes[attr523, Listable]; Block[{attr523}, attr523[{1, 2}]]; Attributes[attr523]",
        "ValueQ[SetAttributes[attr523, Flat]; attr523[1]]; ClearAll[attr523]",
        "SetAttributes[attr523, HoldAll]; Remove[attr523]",
    ] {
        session.eval_text(source).unwrap();
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .unwrap();
    }
    session.register_builtin(BuiltinSpec::ordinary(
        "AttrRegister523",
        "Cache invalidation.",
        panic_after_stack_charge,
        MemoryGrowthContract::Delegated,
    ));
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

#[test]
fn maintained_builtins_and_limits_charge_replace_and_rearm() {
    let mut session = Session::empty();
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    let spec = BuiltinSpec::ordinary(
        "LedgerBuiltin523",
        "Accounting replacement probe.",
        panic_after_stack_charge,
        MemoryGrowthContract::Delegated,
    );
    session.register_builtin(spec);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.register_builtin(spec);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.set_recursion_limit(64);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    // Replacing the fixed-size interrupt object has the same modeled size.
    session.next_request_generation.set(0);
    let accepted = session.accept_cancellable_request().unwrap();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    drop(accepted);
    session.current_memory_limit = None;
    session.register_builtin(BuiltinSpec::ordinary(
        "UnarmedBuiltin523",
        "Rearm probe.",
        panic_after_stack_charge,
        MemoryGrowthContract::Delegated,
    ));
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}

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
