use super::{bindings::*, receipts::*, schedule};
use crate::oracle::fixtures::obligations::{
    boundaries,
    guest::{self, RunFailure},
};
use crate::{HostEvalOutcome, HostInputKind, HostMessage, HostTraceEvent};

#[test]
fn cm_unchecked_build_configuration() {
    let control = std::env::var_os("RUM_CM739_GROWTH_CONTROL_CHILD").is_some();
    let lane = match std::env::var("RUM_CM739_FEATURE_LANE").as_deref() {
        Err(std::env::VarError::NotPresent) => false,
        Ok("p0543-unchecked") => true,
        other => panic!("unknown configuration lane: {other:?}"),
    };
    let expected = !lane || control;
    let observed = crate::oracle::fixtures::obligations::native::checked_dispatch_enabled();
    assert_eq!(observed, expected, "BuildConfiguration");
    assert_ne!(observed, !expected, "configuration falsifier must reject");
    println!(
        "CM BUILD CONFIGURATION PASS lane={} control={control} checked_only_enforcement={observed}",
        if lane { "p0543-unchecked" } else { "mandated" }
    );
    if !control {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "oracle::obligations::execution_tests::cm_unchecked_build_configuration",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("RUM_CM739_GROWTH_CONTROL_CHILD", "1")
            .env("RUM_TEST_BUILTIN_MEMORY_GROWTH", "1")
            .output()
            .unwrap();
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        assert!(output.status.success(), "checked-only control must fire");
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("control=true checked_only_enforcement=true")
        );
        println!("CM BUILD CONFIGURATION FALSIFIER checked-only-enforcement REJECT NonAmplifying");
    }
}

fn cohort() -> (ActiveRequiredBag, FrozenBindings, BoundBag) {
    let required = ActiveRequiredBag::committed().unwrap().cohort(&["P0001"]);
    let frozen = FrozenBindings::committed().unwrap().cohort(&["P0001"]);
    let bound = bind(&required, &frozen).unwrap();
    (required, frozen, bound)
}

#[test]
fn cm_bindings_exact() {
    let required = ActiveRequiredBag::committed().unwrap();
    let frozen = FrozenBindings::committed().unwrap();
    let bound = bind(&required, &frozen).unwrap();
    assert_eq!(
        (required.keys.len(), required.port_blocked.len()),
        (9060, 1445)
    );
    assert_eq!(bound.plans.len(), guest::programs().unwrap().len());
    assert_eq!(bound.plans.len() + bound.blocked.len(), 10505);
    let (cohort, _, complete) = cohort();
    assert_eq!((cohort.keys.len(), complete.plans.len()), (60, 60));
    assert!(complete.blocked.is_empty());
    println!(
        "CM BINDINGS PASS required=10505 active=9060 declared_bound={} PortBlocked={} absent_packets=600 unresolved_targets=785 repl_placement=60 unported_programs={} full_CM_green=false",
        bound.plans.len(),
        bound.blocked.len(),
        required.keys.len() - bound.plans.len()
    );
}

#[test]
fn cm_receipts_exact() {
    if !schedule::profile_child("oracle::obligations::execution_tests::cm_receipts_exact") {
        return;
    }
    let required = ActiveRequiredBag::committed().unwrap();
    let bound = bind(&required, &FrozenBindings::committed().unwrap()).unwrap();
    let programs = guest::programs().unwrap();
    let mut receipts = Vec::new();
    for plan in bound
        .plans
        .iter()
        .filter(|p| !profile_off(p) && !feature_lane(p))
    {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        receipts.push(receipt);
    }
    let off: Vec<_> = bound
        .plans
        .iter()
        .filter(|p| profile_off(p))
        .cloned()
        .collect();
    receipts.extend(execute_off_group(&off).unwrap());
    let lane: Vec<_> = bound
        .plans
        .iter()
        .filter(|p| feature_lane(p))
        .cloned()
        .collect();
    receipts.extend(execute_p0543_group(&lane).unwrap());
    for plan in off.iter().chain(&lane) {
        receipts
            .iter()
            .find(|r| r.key() == &plan.key)
            .unwrap()
            .assert_identity(plan, &RunIdentity::planned(plan));
    }
    let rows: std::collections::BTreeSet<_> = bound.plans.iter().map(|p| p.key.row()).collect();
    for row in &rows {
        let row_receipts: Vec<_> = receipts
            .iter()
            .filter(|r| r.key().row() == *row)
            .cloned()
            .collect();
        let row_required = required.cohort(&[row]);
        let row_bound = bind(
            &row_required,
            &FrozenBindings::committed().unwrap().cohort(&[row]),
        )
        .unwrap();
        reconcile_executions(&row_required, &row_bound, &row_receipts).unwrap();
        assert_eq!(row_receipts.len(), row_required.keys.len());
        let pauses = row_receipts
            .iter()
            .map(ExecutionReceipt::actual_requests)
            .sum::<usize>();
        let planned = programs
            .values()
            .filter(|p| p.key.row() == *row)
            .map(|p| p.total_requests())
            .sum::<usize>();
        assert_eq!(pauses, planned);
        let lifecycle = row_receipts
            .iter()
            .map(ExecutionReceipt::lifecycle_count)
            .sum::<usize>();
        println!(
            "CM RECEIPT FAMILY PASS row={row} required={} bound={} executed={} multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle}",
            row_required.keys.len(),
            row_bound.plans.len(),
            row_receipts.len()
        );
    }
    let remaining = required.keys.len() - bound.plans.len();
    let exact = reconcile_executions(&required, &bound, &receipts);
    if remaining == 0 {
        exact.unwrap();
    } else {
        assert_eq!(exact, Err(ExactError::MissingBound));
    }
    let pauses = receipts
        .iter()
        .map(ExecutionReceipt::actual_requests)
        .sum::<usize>();
    let lifecycle = receipts
        .iter()
        .map(ExecutionReceipt::lifecycle_count)
        .sum::<usize>();
    let families: std::collections::BTreeSet<_> =
        programs.values().map(|p| p.family.as_str()).collect();
    println!(
        "CM RECEIPTS PASS calibration=P0001 rows={} families={} bound={} executed={} multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle} campaign_remaining={remaining} PortBlocked=1445 full_CM_green=false",
        rows.len(),
        families.len(),
        bound.plans.len(),
        receipts.len()
    );
}

#[test]
fn cm_live_off_worker() {
    let Ok(nonce) = std::env::var("RUM_CM739_OFF_WORKER") else {
        return;
    };
    let required = ActiveRequiredBag::committed().unwrap();
    let bound = bind(&required, &FrozenBindings::committed().unwrap()).unwrap();
    let mut count = 0;
    for plan in bound.plans.iter().filter(|p| profile_off(p)) {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        export_live(&receipt, &nonce);
        count += 1;
    }
    println!("CM739_WIRE_END\t{nonce}\t{count}");
}

/// Campaign driver: every selected row runs its complete frozen cohort. The
/// standing cm_receipts_exact gate always runs all bound rows independently.
#[test]
fn cm_selected_families_worker() {
    let Ok(selection) = std::env::var("RUM_CM739_SELECTED_FAMILIES") else {
        return;
    };
    if !schedule::profile_child("oracle::obligations::execution_tests::cm_selected_families_worker")
    {
        return;
    }
    let selected: Vec<_> = selection.split(',').collect();
    let required = ActiveRequiredBag::committed().unwrap().cohort(&selected);
    let bound = bind(
        &required,
        &FrozenBindings::committed().unwrap().cohort(&selected),
    )
    .unwrap();
    assert!(!required.keys.is_empty() && bound.blocked.is_empty());
    assert_eq!(required.keys.len(), bound.plans.len());
    let mut receipts = Vec::new();
    for plan in &bound.plans {
        assert!(!feature_lane(plan) && !profile_off(plan));
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        receipts.push(receipt);
    }
    reconcile_executions(&required, &bound, &receipts).unwrap();
    for row in selected {
        let count = required.keys.iter().filter(|k| k.row() == row).count();
        assert!(count > 0);
        let actual: Vec<_> = receipts.iter().filter(|r| r.key().row() == row).collect();
        assert_eq!(actual.len(), count);
        let pauses: usize = actual.iter().map(|r| r.actual_requests()).sum();
        let lifecycle: usize = actual.iter().map(|r| r.lifecycle_count()).sum();
        println!(
            "CM RECEIPT FAMILY PASS row={row} required={count} bound={count} executed={count} multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle}"
        );
    }
    println!(
        "CM SELECTED FAMILIES PASS bound={} executed={} multiplicity=1 full_CM_green=false",
        bound.plans.len(),
        receipts.len()
    );
}

#[test]
fn cm_p0543_receipts_exact() {
    let required = ActiveRequiredBag::committed().unwrap().cohort(&["P0543"]);
    let frozen = FrozenBindings::committed().unwrap().cohort(&["P0543"]);
    let bound = bind(&required, &frozen).unwrap();
    assert_eq!((required.keys.len(), bound.plans.len()), (55, 55));
    let receipts = execute_p0543_group(&bound.plans).unwrap();
    reconcile_executions(&required, &bound, &receipts).unwrap();
    for plan in &bound.plans {
        receipts
            .iter()
            .find(|r| r.key() == &plan.key)
            .unwrap()
            .assert_identity(plan, &RunIdentity::planned(plan));
    }
    println!(
        "CM P0543 RECEIPTS PASS required=55 bound=55 executed=55 multiplicity=1 lane=p0543-unchecked native_fields=10 full_CM_green=false"
    );
}

#[test]
fn cm_live_p0543_worker() {
    let Ok(nonce) = std::env::var("RUM_CM739_P0543_WORKER") else {
        return;
    };
    assert_eq!(RunIdentity::current().feature, "p0543-unchecked");
    let required = ActiveRequiredBag::committed().unwrap().cohort(&["P0543"]);
    let bound = bind(
        &required,
        &FrozenBindings::committed().unwrap().cohort(&["P0543"]),
    )
    .unwrap();
    assert_eq!((required.keys.len(), bound.plans.len()), (55, 55));
    let programs = guest::programs().unwrap();
    let mut receipts = Vec::new();
    for plan in &bound.plans {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        if plan.key.variant() == "Middle" && plan.key.context() == "phase_observation.trace" {
            let p = &programs[&plan.program];
            let (resumed, queued, _) = receipt.observations();
            let mut changed = resumed.clone();
            let input = changed
                .counters
                .iter_mut()
                .find(|(name, _)| name.contains("name: \"Input\""))
                .expect("authenticated Input builtin native row");
            assert_eq!(input.1, [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
            input.1 = [1, 0, 0, 0, 0, 0, 0, 0, 1, 0];
            rejected(
                "P0543-unchecked-boundary-own-omission-equal-scalar",
                RunFailure::Native,
                guest::validate(p, &changed, queued),
            );
            let mut changed = resumed.clone();
            changed
                .counters
                .iter_mut()
                .find(|(name, _)| name == "CM739UncheckedDispatchBuildConfiguration")
                .unwrap()
                .1[0] = 1;
            rejected(
                "P0543-checked-route-substitution",
                RunFailure::Native,
                guest::validate(p, &changed, queued),
            );
            let mut both = queued.clone();
            both.counters = changed.counters.clone();
            rejected(
                "P0543-both-routes-checked",
                RunFailure::Native,
                guest::validate(p, &changed, &both),
            );
        }
        export_live(&receipt, &nonce);
        receipts.push(receipt);
    }
    reconcile_executions(&required, &bound, &receipts).unwrap();
    println!("CM739_WIRE_END\t{nonce}\t{}", receipts.len());
}

fn rejected(name: &str, expected: RunFailure, actual: Result<(), RunFailure>) {
    assert_eq!(actual, Err(expected), "{name}: exact predicate");
    println!("CM EXECUTION FALSIFIER {name} REJECT {expected:?}");
}

#[test]
fn cm_fixedpoint_receipts_exact() {
    if !schedule::profile_child(
        "oracle::obligations::execution_tests::cm_fixedpoint_receipts_exact",
    ) {
        return;
    }
    let selected = ["P1470", "P1471"];
    let required = ActiveRequiredBag::committed().unwrap().cohort(&selected);
    let bound = bind(
        &required,
        &FrozenBindings::committed().unwrap().cohort(&selected),
    )
    .unwrap();
    assert_eq!((required.keys.len(), bound.plans.len()), (90, 90));
    let programs = guest::programs().unwrap();
    let mut receipts = Vec::new();
    for plan in &bound.plans {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        if plan.key.variant() == "Error" && plan.key.context() == "phase_observation.trace" {
            let (resumed, queued, _) = receipt.observations();
            let mut omitted = resumed.clone();
            let counter = omitted
                .counters
                .iter_mut()
                .find(|(name, _)| {
                    name == "Runtime { phase: \"evaluate.once_unstable\", source: None }"
                })
                .unwrap();
            assert_eq!(counter.1, [7, 4, 3, 0, 0, 0, 3, 0, 0, 0]);
            counter.1 = [7, 3, 4, 0, 0, 0, 4, 0, 0, 0];
            assert_eq!(omitted.output, queued.output);
            rejected(
                &format!("{}-own-pass-publication-omission", plan.key.row()),
                RunFailure::Native,
                guest::validate(&programs[&plan.program], &omitted, queued),
            );
        }
        receipts.push(receipt);
    }
    reconcile_executions(&required, &bound, &receipts).unwrap();
    for row in selected {
        let actual: Vec<_> = receipts.iter().filter(|r| r.key().row() == row).collect();
        assert_eq!(actual.len(), 45);
        let pauses: usize = actual.iter().map(|r| r.actual_requests()).sum();
        let lifecycle: usize = actual.iter().map(|r| r.lifecycle_count()).sum();
        println!(
            "CM FIXEDPOINT RECEIPTS PASS row={row} required=45 bound=45 executed=45 multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle} native_fields=10 full_CM_green=false"
        );
    }
}

#[test]
fn cm_adapter_receipts_exact() {
    if !schedule::profile_child("oracle::obligations::execution_tests::cm_adapter_receipts_exact") {
        return;
    }
    let selected = ["P2611", "P2612"];
    let required = ActiveRequiredBag::committed().unwrap().cohort(&selected);
    let bound = bind(
        &required,
        &FrozenBindings::committed().unwrap().cohort(&selected),
    )
    .unwrap();
    assert_eq!((required.keys.len(), bound.plans.len()), (60, 60));
    let programs = guest::programs().unwrap();
    let seals = provenance().unwrap();
    let mut receipts = Vec::new();
    for plan in &bound.plans {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        let program = &programs[&plan.program];
        let (resumed, queued, _) = receipt.observations();
        let own: Vec<_> = seals
            .iter()
            .filter(|r| r[0] == "c52-own-omission-output" && r[1] == program.id)
            .collect();
        assert_eq!(own.len(), 1);
        assert_eq!((own[0][4], own[0][5]), ("1", LEDGER_PIN));
        // Every sealed-valid C52 omission forwards the actual Cancelled
        // success before the adapter's normalization. Native rows are equal.
        let mut omitted = queued.clone();
        let HostEvalOutcome::Failure(failure) = &omitted.output else {
            panic!("C52 must normalize the cancelled success");
        };
        assert_eq!(failure.code, crate::HostFailureCode::Cancelled);
        let success = crate::HostEvalSuccess {
            value: crate::HostValue {
                text: "TerminatedEvaluation[\"Cancelled\"]".into(),
                typed: crate::HostTyped::Other {
                    head: "TerminatedEvaluation".into(),
                    text: "TerminatedEvaluation[\"Cancelled\"]".into(),
                },
            },
            termination: Some(crate::HostTerminationReason::Cancelled),
            messages: failure.messages.clone(),
            prints: failure.prints.clone(),
            trace: failure.trace.clone(),
            outputs: failure.outputs.clone(),
        };
        omitted.output = HostEvalOutcome::Success(success.clone());
        omitted.adapter.as_mut().unwrap().terminal = crate::KernelExecuteOutcome::Success(success);
        assert_eq!(resumed.counters, omitted.counters);
        rejected(
            &format!("{}-sealed-own-omission", program.id),
            RunFailure::Output,
            guest::validate(program, resumed, &omitted),
        );
        for control in ["constant-native", "missing-native", "duplicate-native"] {
            let mut r = resumed.clone();
            let mut q = queued.clone();
            for side in [&mut r, &mut q] {
                assert!(!side.counters.is_empty());
                match control {
                    "constant-native" => {
                        assert_eq!(side.counters[0].1[9], 0);
                        side.counters[0].1[9] = 1;
                    }
                    "missing-native" => {
                        side.counters.remove(0);
                    }
                    "duplicate-native" => side.counters.push(side.counters[0].clone()),
                    _ => unreachable!(),
                }
            }
            rejected(
                &format!("{}-{control}", program.id),
                RunFailure::Native,
                guest::validate(program, &r, &q),
            );
        }
        receipts.push(receipt);
    }
    reconcile_executions(&required, &bound, &receipts).unwrap();
    for row in selected {
        let actual: Vec<_> = receipts.iter().filter(|r| r.key().row() == row).collect();
        assert_eq!(actual.len(), 30);
        let pauses: usize = actual.iter().map(|r| r.actual_requests()).sum();
        let lifecycle: usize = actual.iter().map(|r| r.lifecycle_count()).sum();
        println!(
            "CM ADAPTER RECEIPTS PASS row={row} required=30 bound=30 executed=30 multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle} own_Output=30 native_controls=90 sealed_invalid=0 native_fields=10 full_CM_green=false"
        );
    }
}

#[test]
fn cm_timeout_receipts_exact() {
    use crate::oracle::fixtures::obligations::native::timeout_event;
    if !schedule::profile_child("oracle::obligations::execution_tests::cm_timeout_receipts_exact") {
        return;
    }
    let selected = ["P0813", "P2600"];
    let required = ActiveRequiredBag::committed().unwrap().cohort(&selected);
    let bound = bind(
        &required,
        &FrozenBindings::committed().unwrap().cohort(&selected),
    )
    .unwrap();
    assert_eq!((required.keys.len(), bound.plans.len()), (60, 60));
    let programs = guest::programs().unwrap();
    let mut receipts = Vec::new();
    for plan in &bound.plans {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        let program = &programs[&plan.program];
        let (resumed, queued, _) = receipt.observations();
        let event = timeout_event(plan.key.row()).unwrap();
        if plan.key.variant() == "Middle"
            && plan.key.context() == "error_before_tail.last-pause-throw"
        {
            let mut omitted = queued.clone();
            let predicate = if plan.key.row() == "P0813" {
                // This exact vector difference is in the sealed-valid C49
                // own omission. Isolate its Native predicate with all public
                // effects equal; never use the five sealed-invalid cells.
                let total = omitted
                    .counters
                    .iter_mut()
                    .find(|(name, _)| {
                        name == "Runtime { phase: \"catch.eval.total\", source: None }"
                    })
                    .unwrap();
                assert_eq!(total.1, [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
                total.1 = [1, 0, 1, 0, 0, 0, 1, 0, 0, 0];
                assert_eq!(resumed.output, omitted.output);
                RunFailure::Native
            } else {
                // C47's own omission drops this structured result push;
                // its native vectors and scalar value remain unchanged.
                let HostEvalOutcome::Success(out) = &mut omitted.output else {
                    panic!("sealed C47 representative must succeed");
                };
                assert_eq!(out.value.text, "43");
                let results: Vec<_> = out
                    .outputs
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| item.kind == crate::HostStructuredOutputKind::Result)
                    .map(|(index, item)| (index, item.text.clone()))
                    .collect();
                assert_eq!(results.len(), 1);
                assert_eq!(results[0].1, "43");
                out.outputs.remove(results[0].0);
                assert_eq!(resumed.counters, omitted.counters);
                RunFailure::Output
            };
            rejected(
                &format!("{}-sealed-valid-own-omission", plan.key.row()),
                predicate,
                guest::validate(program, resumed, &omitted),
            );
            for control in ["constant-timeout", "missing-timeout", "duplicate-timeout"] {
                let mut r = resumed.clone();
                let mut q = queued.clone();
                for side in [&mut r, &mut q] {
                    let index = side
                        .counters
                        .iter()
                        .position(|(name, _)| name == event)
                        .unwrap();
                    match control {
                        "constant-timeout" => side.counters[index].1[0] = 1,
                        "missing-timeout" => {
                            side.counters.remove(index);
                        }
                        "duplicate-timeout" => side.counters.push(side.counters[index].clone()),
                        _ => unreachable!(),
                    }
                }
                rejected(
                    &format!("{}-{control}", plan.key.row()),
                    RunFailure::Native,
                    guest::validate(program, &r, &q),
                );
            }
        }
        receipts.push(receipt);
    }
    reconcile_executions(&required, &bound, &receipts).unwrap();
    let invalid: Vec<_> = provenance()
        .unwrap()
        .into_iter()
        .filter(|row| {
            row[0] == "sealed-invalid"
                && row[1].starts_with("C49.catch-body-result-dispatch.P0813.")
        })
        .collect();
    assert_eq!(invalid.len(), 5);
    for row in invalid {
        assert!(row[1].ends_with(".return_boundary.loop-store-reset"));
        assert_eq!(row[4], "0");
        assert_eq!(row[5], LEDGER_PIN);
        println!(
            "CM SEALED INVALID probe={} valid=false caught=false falsifier_credit=0 raw={} sha256={}",
            row[1], row[2], row[3]
        );
    }
    for row in selected {
        let actual: Vec<_> = receipts.iter().filter(|r| r.key().row() == row).collect();
        assert_eq!(actual.len(), 30);
        let pauses: usize = actual.iter().map(|r| r.actual_requests()).sum();
        let lifecycle: usize = actual.iter().map(|r| r.lifecycle_count()).sum();
        println!(
            "CM TIMEOUT RECEIPTS PASS row={row} required=30 bound=30 executed=30 multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle} timeout_zero=30 native_fields=10 full_CM_green=false"
        );
    }
}

#[test]
fn cm_checkpoint_receipts_exact() {
    use crate::oracle::fixtures::obligations::native::CHECKPOINT_EVENT;
    if !schedule::profile_child(
        "oracle::obligations::execution_tests::cm_checkpoint_receipts_exact",
    ) {
        return;
    }
    let selected = ["P1566", "P1568"];
    let required = ActiveRequiredBag::committed().unwrap().cohort(&selected);
    let bound = bind(
        &required,
        &FrozenBindings::committed().unwrap().cohort(&selected),
    )
    .unwrap();
    assert_eq!((required.keys.len(), bound.plans.len()), (60, 60));
    let programs = guest::programs().unwrap();
    let mut receipts = Vec::new();
    let mut counts = std::collections::BTreeMap::new();
    for plan in &bound.plans {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        let program = &programs[&plan.program];
        let (resumed, queued, _) = receipt.observations();
        let event = resumed
            .counters
            .iter()
            .find(|(name, _)| name == CHECKPOINT_EVENT)
            .unwrap();
        *counts
            .entry((plan.key.row().to_string(), event.1))
            .or_insert(0) += 1;
        if plan.key.variant() == "Middle" && plan.key.context() == "phase_observation.trace" {
            let mut r = resumed.clone();
            let mut q = queued.clone();
            let omitted = if plan.key.row() == "P1566" {
                &mut q
            } else {
                &mut r
            };
            omitted
                .counters
                .iter_mut()
                .find(|(name, _)| name == CHECKPOINT_EVENT)
                .unwrap()
                .1 = [1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
            assert_eq!(r.output, q.output);
            assert_eq!(r.projections, q.projections);
            assert_eq!(r.checkpoint_snapshot, q.checkpoint_snapshot);
            assert_eq!(r.pending, q.pending);
            assert_eq!(r.fresh, q.fresh);
            assert_eq!(r.queue_probe, q.queue_probe);
            assert_eq!(r.queue_finish, q.queue_finish);
            rejected(
                &format!(
                    "{}-own-checkpoint-omission-public-effects-equal",
                    plan.key.row()
                ),
                RunFailure::Native,
                guest::validate(program, &r, &q),
            );
            for (name, duplicate) in [
                ("missing-checkpoint", false),
                ("duplicate-checkpoint", true),
            ] {
                let mut r = resumed.clone();
                let mut q = queued.clone();
                for side in [&mut r, &mut q] {
                    if duplicate {
                        side.counters.push(event.clone());
                    } else {
                        side.counters.retain(|(name, _)| name != CHECKPOINT_EVENT);
                    }
                }
                rejected(
                    &format!("{}-{name}", plan.key.row()),
                    RunFailure::Native,
                    guest::validate(program, &r, &q),
                );
            }
        }
        if plan.key.context() == "value.base" && matches!(plan.key.variant(), "Middle" | "Error") {
            let mut r = resumed.clone();
            let mut q = queued.clone();
            let constant_dirty = u64::from(plan.key.variant() == "Middle");
            for side in [&mut r, &mut q] {
                side.counters
                    .iter_mut()
                    .find(|(name, _)| name == CHECKPOINT_EVENT)
                    .unwrap()
                    .1 = [0, constant_dirty, 0, 0, 0, 0, 0, 0, 0, 0];
            }
            rejected(
                &format!("{}-constant-checkpoint-0,{constant_dirty}", plan.key.row()),
                RunFailure::Native,
                guest::validate(program, &r, &q),
            );
        }
        receipts.push(receipt);
    }
    reconcile_executions(&required, &bound, &receipts).unwrap();
    for row in selected {
        assert_eq!(counts[&(row.to_string(), [0; 10])], 20);
        assert_eq!(
            counts[&(row.to_string(), [0, 1, 0, 0, 0, 0, 0, 0, 0, 0])],
            10
        );
        let actual: Vec<_> = receipts.iter().filter(|r| r.key().row() == row).collect();
        let pauses: usize = actual.iter().map(|r| r.actual_requests()).sum();
        let lifecycle: usize = actual.iter().map(|r| r.lifecycle_count()).sum();
        println!(
            "CM CHECKPOINT RECEIPTS PASS row={row} required=30 bound=30 executed=30 multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle} pair_false_false=20 pair_false_true=10 native_fields=10 full_CM_green=false"
        );
    }
}

#[test]
fn cm_p1476_receipts_exact() {
    if !schedule::profile_child("oracle::obligations::execution_tests::cm_p1476_receipts_exact") {
        return;
    }
    let required = ActiveRequiredBag::committed().unwrap().cohort(&["P1476"]);
    let bound = bind(
        &required,
        &FrozenBindings::committed().unwrap().cohort(&["P1476"]),
    )
    .unwrap();
    assert_eq!((required.keys.len(), bound.plans.len()), (30, 30));
    let programs = guest::programs().unwrap();
    let mut receipts = Vec::new();
    for plan in &bound.plans {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        if plan.key.variant() == "Middle" && plan.key.context() == "phase_observation.trace" {
            use crate::oracle::fixtures::obligations::native::C31_EVENT;
            let program = &programs[&plan.program];
            let (resumed, queued, _) = receipt.observations();
            let mut omitted = resumed.clone();
            let row = omitted
                .counters
                .iter_mut()
                .find(|(name, _)| name == C31_EVENT)
                .unwrap();
            assert_eq!(row.1, [6, 6, 0, 0, 0, 0, 0, 0, 0, 0]);
            row.1 = [4, 4, 0, 0, 0, 0, 0, 0, 0, 0];
            assert_eq!(omitted.output, queued.output);
            rejected(
                "P1476-own-omission-4,4-vs-6,6-equal-scalar",
                RunFailure::Native,
                guest::validate(program, &omitted, queued),
            );
            let mut both = queued.clone();
            both.counters = omitted.counters.clone();
            rejected(
                "P1476-both-events-stale",
                RunFailure::Native,
                guest::validate(program, &omitted, &both),
            );
            let mut missing = resumed.clone();
            missing.counters.retain(|(name, _)| name != C31_EVENT);
            let mut absent = queued.clone();
            absent.counters = missing.counters.clone();
            rejected(
                "P1476-missing-synchronous-event",
                RunFailure::Native,
                guest::validate(program, &missing, &absent),
            );
            let mut duplicate = resumed.clone();
            duplicate.counters.push(
                resumed
                    .counters
                    .iter()
                    .find(|(name, _)| name == C31_EVENT)
                    .unwrap()
                    .clone(),
            );
            let mut twice = queued.clone();
            twice.counters = duplicate.counters.clone();
            rejected(
                "P1476-duplicate-synchronous-event",
                RunFailure::Native,
                guest::validate(program, &duplicate, &twice),
            );
        }
        receipts.push(receipt);
    }
    reconcile_executions(&required, &bound, &receipts).unwrap();
    let pauses: usize = receipts.iter().map(ExecutionReceipt::actual_requests).sum();
    let lifecycle: usize = receipts.iter().map(ExecutionReceipt::lifecycle_count).sum();
    println!(
        "CM P1476 RECEIPTS PASS required=30 bound=30 executed=30 multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle} native_fields=10 full_CM_green=false"
    );
}

#[test]
fn cm_p0508_receipts_exact() {
    if !schedule::profile_child("oracle::obligations::execution_tests::cm_p0508_receipts_exact") {
        return;
    }
    let required = ActiveRequiredBag::committed().unwrap().cohort(&["P0508"]);
    let bound = bind(
        &required,
        &FrozenBindings::committed().unwrap().cohort(&["P0508"]),
    )
    .unwrap();
    assert_eq!((required.keys.len(), bound.plans.len()), (30, 30));
    let programs = guest::programs().unwrap();
    let mut receipts = Vec::new();
    let mut states = 0;
    for plan in &bound.plans {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        let (resumed, queued, _) = receipt.observations();
        states += resumed.evaluation_states.len() + queued.evaluation_states.len();
        if plan.key.variant() == "Middle" && plan.key.context() == "phase_observation.trace" {
            let p = &programs[&plan.program];
            let mut changed = resumed.clone();
            changed
                .counters
                .iter_mut()
                .find(|(name, _)| name == "C28BetweenRequestsEvaluationStateMax")
                .unwrap()
                .1 = [2, 2, 0, 0, 0, 0, 0, 0, 0, 0];
            rejected(
                "P0508-between-request-state-2,2-vs-0,0-equal-scalar",
                RunFailure::Native,
                guest::validate(p, &changed, queued),
            );
            let mut both = queued.clone();
            both.counters = changed.counters.clone();
            rejected(
                "P0508-between-request-state-both-stale",
                RunFailure::Native,
                guest::validate(p, &changed, &both),
            );
            let mut changed = resumed.clone();
            changed.evaluation_states.pop();
            rejected(
                "P0508-missing-request-return-state-sample",
                RunFailure::Native,
                guest::validate(p, &changed, queued),
            );
            let mut changed = resumed.clone();
            changed.evaluation_states[0] = (2, 2);
            rejected(
                "P0508-state-sample-disagrees-with-saved-maximum",
                RunFailure::Native,
                guest::validate(p, &changed, queued),
            );
        }
        receipts.push(receipt);
    }
    reconcile_executions(&required, &bound, &receipts).unwrap();
    let pauses: usize = receipts.iter().map(ExecutionReceipt::actual_requests).sum();
    let lifecycle: usize = receipts.iter().map(ExecutionReceipt::lifecycle_count).sum();
    println!(
        "CM P0508 RECEIPTS PASS required=30 bound=30 executed=30 multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle} direct_state_samples={states} native_fields=10 full_CM_green=false"
    );
}

#[test]
fn cm_execution_negative_controls() {
    if !schedule::profile_child(
        "oracle::obligations::execution_tests::cm_execution_negative_controls",
    ) {
        return;
    }
    let (required, frozen, bound) = cohort();
    let mut deleted = frozen.clone();
    deleted.entries.pop();
    assert_eq!(
        bind(&required, &deleted).unwrap_err(),
        BindingError::MissingBinding
    );
    println!("CM EXECUTION FALSIFIER delete-binding REJECT MissingBinding");
    let mut duplicate = frozen.clone();
    duplicate.entries.push(duplicate.entries[0].clone());
    assert_eq!(
        bind(&required, &duplicate).unwrap_err(),
        BindingError::DuplicateBinding
    );
    println!("CM EXECUTION FALSIFIER duplicate-binding REJECT DuplicateBinding");
    assert_eq!(
        reconcile_executions(&required, &bound, &Vec::new()),
        Err(ExactError::MissingReceipt)
    );
    println!("CM EXECUTION FALSIFIER declare-without-execute REJECT MissingReceipt");
    let plan = bound
        .plans
        .iter()
        .find(|p| p.key.variant() == "Twice" && p.key.context() == "prefix_and_tail.log")
        .unwrap();
    let receipt = execute(plan, &RunIdentity::current()).unwrap();
    assert_eq!(
        reconcile_executions(&required, &bound, &vec![receipt.clone(), receipt.clone()]),
        Err(ExactError::DuplicateReceipt)
    );
    println!("CM EXECUTION FALSIFIER duplicate-receipt REJECT DuplicateReceipt");
    let programs = guest::programs().unwrap();
    let p = &programs[&plan.program];
    let (resumed, queued, lifecycle) = receipt.observations();
    let mut changed = resumed.clone();
    changed.setup_native.entries = 1;
    rejected(
        "pause-in-setup",
        RunFailure::Setup,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    changed.setup = guest::SetupObservation::Absent;
    rejected(
        "omit-required-setup",
        RunFailure::Setup,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    changed.pauses.pop();
    rejected(
        "collapse-two-pauses",
        RunFailure::PauseCount,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    changed.pauses[1].ordinal = 0;
    rejected(
        "change-ordinal",
        RunFailure::Ordinal,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    changed.pauses[1].phase = "first-guard".to_string();
    rejected(
        "borrow-first-pause-guard",
        RunFailure::Phase,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    if let HostEvalOutcome::InputRequired(out) = &mut changed.pauses[1].output {
        out.request.prompt.push('!');
    }
    rejected(
        "change-prompt",
        RunFailure::Prompt,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    if let HostEvalOutcome::InputRequired(out) = &mut changed.pauses[1].output {
        out.request.kind = HostInputKind::InputString;
    }
    rejected(
        "change-kind",
        RunFailure::Kind,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    changed
        .counters
        .iter_mut()
        .find(|(key, _)| key == "C34ActualCallbackReceiptBytes.0")
        .unwrap()
        .1[0] = 0;
    rejected(
        "actual-receipt-32-to-0-equal-scalar",
        RunFailure::Native,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    changed.projections.pop();
    rejected(
        "drop-projection-equal-scalar",
        RunFailure::Projection,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    changed.queue_probe = resumed.output.clone();
    rejected(
        "extra-queued-answer",
        RunFailure::Queue,
        guest::validate(p, &changed, queued),
    );
    let mut changed = resumed.clone();
    changed.fresh = resumed.output.clone();
    rejected(
        "fresh-request-failure",
        RunFailure::Fresh,
        guest::validate(p, &changed, queued),
    );
    let mut changed = lifecycle.to_vec();
    changed[0].delivered.retain(|text| text != "cm-cleanup");
    rejected(
        "lost-cancellation-cleanup",
        RunFailure::Lifecycle,
        boundaries::validate(p, &changed),
    );
    // Non-crediting channel validator controls: preserve the scalar while
    // deleting an earlier event from a later cumulative response.
    let mut before = resumed.pauses[0].output.clone();
    let mut after = resumed.output.clone();
    if let (HostEvalOutcome::InputRequired(a), HostEvalOutcome::Success(b)) =
        (&mut before, &mut after)
    {
        a.prints = vec!["prefix".into()];
        b.prints = vec!["prefix".into(), "tail".into()];
        guest::prefix(&before, &after).unwrap();
    }
    if let HostEvalOutcome::Success(b) = &mut after {
        b.prints.remove(0);
    }
    rejected(
        "drop-nonfinal-print",
        RunFailure::PrintPrefix,
        guest::prefix(&before, &after),
    );
    if let (HostEvalOutcome::InputRequired(a), HostEvalOutcome::Success(b)) =
        (&mut before, &mut after)
    {
        a.prints.clear();
        b.prints.clear();
        a.messages = vec![HostMessage {
            symbol: "Global`cm".into(),
            tag: "prefix".into(),
            text: "prefix".into(),
        }];
        b.messages = a.messages.clone();
    }
    guest::prefix(&before, &after).unwrap();
    if let HostEvalOutcome::Success(b) = &mut after {
        b.messages.clear();
    }
    rejected(
        "drop-nonfinal-message",
        RunFailure::MessagePrefix,
        guest::prefix(&before, &after),
    );
    if let (HostEvalOutcome::InputRequired(a), HostEvalOutcome::Success(b)) =
        (&mut before, &mut after)
    {
        a.messages.clear();
        b.messages.clear();
        a.trace = vec![HostTraceEvent::EvaluationPass {
            before: "prefix".into(),
            after: "observed".into(),
        }];
        b.trace = a.trace.clone();
    }
    guest::prefix(&before, &after).unwrap();
    if let HostEvalOutcome::Success(b) = &mut after {
        b.trace.clear();
    }
    rejected(
        "drop-nonfinal-trace",
        RunFailure::TracePrefix,
        guest::prefix(&before, &after),
    );
    for row in ["P1562", "P1563"] {
        let p = programs
            .values()
            .find(|p| p.key.row() == row && p.key.variant() == "Twice")
            .unwrap();
        let resumed = guest::run(p, false).unwrap();
        let queued = guest::run(p, true).unwrap();
        guest::validate(p, &resumed, &queued).unwrap();
        let mut changed = queued.clone();
        changed
            .counters
            .iter_mut()
            .find(|(name, _)| name == "C29TerminalAcceptedRequestState")
            .unwrap()
            .1 = [1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(resumed.output, changed.output);
        rejected(
            &format!("{row}-terminal-accepted-state-0,0-vs-1,1-equal-scalar"),
            RunFailure::Native,
            guest::validate(p, &resumed, &changed),
        );
        // Equal stale bits on both sides must also reject, even though the
        // scalar and all other channels still agree.
        let mut both = resumed.clone();
        both.counters = changed.counters.clone();
        rejected(
            &format!("{row}-terminal-accepted-state-both-stale"),
            RunFailure::Native,
            guest::validate(p, &both, &changed),
        );
    }
    let p = programs
        .values()
        .find(|p| {
            p.key.row() == "P1577"
                && p.key.variant() == "Middle"
                && p.key.context() == "phase_observation.trace"
        })
        .unwrap();
    let resumed = guest::run(p, false).unwrap();
    let queued = guest::run(p, true).unwrap();
    guest::validate(p, &resumed, &queued).unwrap();
    let mut changed = resumed.clone();
    changed.evaluation_states[1] = (1, 0);
    changed
        .counters
        .iter_mut()
        .find(|(name, _)| name == "C29ResumingDriverEvaluationStateMax")
        .unwrap()
        .1 = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(changed.output, queued.output);
    rejected(
        "P1577-resuming-driver-state-1,0-vs-0,0-equal-scalar",
        RunFailure::Native,
        guest::validate(p, &changed, &queued),
    );
    let mut both = queued.clone();
    both.counters = changed.counters.clone();
    both.evaluation_states[0] = (1, 0);
    rejected(
        "P1577-resuming-driver-state-both-stale",
        RunFailure::Native,
        guest::validate(p, &changed, &both),
    );
    let mut missing = resumed.clone();
    missing.evaluation_states.remove(1);
    rejected(
        "P1577-missing-second-request-state-sample",
        RunFailure::Native,
        guest::validate(p, &missing, &queued),
    );
    let mut hidden = resumed.clone();
    hidden.evaluation_states[1] = (1, 0);
    rejected(
        "P1577-state-sample-disagrees-with-saved-maximum",
        RunFailure::Native,
        guest::validate(p, &hidden, &queued),
    );
    println!(
        "CM EXECUTION CONTROLS PASS actual_run=true validator_controls_noncrediting=true full_CM_green=false"
    );
}

#[test]
fn cm_resuming_driver_receipts_exact() {
    if !schedule::profile_child(
        "oracle::obligations::execution_tests::cm_resuming_driver_receipts_exact",
    ) {
        return;
    }
    let required = ActiveRequiredBag::committed().unwrap().cohort(&["P1577"]);
    let frozen = FrozenBindings::committed().unwrap().cohort(&["P1577"]);
    let bound = bind(&required, &frozen).unwrap();
    assert_eq!(bound.plans.len(), 30);
    let mut receipts = Vec::new();
    let mut states = 0;
    for plan in &bound.plans {
        let identity = RunIdentity::planned(plan);
        let receipt = execute(plan, &identity)
            .unwrap_or_else(|error| panic!("{} {error:?}", plan.key.line()));
        receipt.assert_identity(plan, &identity);
        let (resumed, queued, _) = receipt.observations();
        states += resumed.evaluation_states.len() + queued.evaluation_states.len();
        receipts.push(receipt);
    }
    reconcile_executions(&required, &bound, &receipts).unwrap();
    let pauses: usize = receipts.iter().map(ExecutionReceipt::actual_requests).sum();
    let lifecycle: usize = receipts.iter().map(ExecutionReceipt::lifecycle_count).sum();
    assert_eq!((pauses, lifecycle, states), (72, 72, 132));
    println!(
        "CM P1577 RECEIPTS PASS required=30 bound=30 executed=30 multiplicity=1 actual_pauses={pauses} lifecycle={lifecycle} direct_state_samples={states} native_fields=10 full_CM_green=false"
    );
}
