use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
    time::Instant,
};

use super::{
    applicability,
    imports::AcceptedInputs,
    owners::{self, OwnerState},
    partition::{self, ApprovedLedger, EnforcementMode, Plan, Scope},
    policy::{self, Checked, Predicate},
    requirements::{self, Count},
    routes,
};

struct Model {
    inputs: AcceptedInputs,
    policies: policy::PolicyTable,
    applicability: applicability::Applicability,
    routes: routes::Routes,
    plan: Plan,
}

fn model() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        let inputs = AcceptedInputs::committed().unwrap();
        let policies = policy::committed().unwrap();
        let applicability = applicability::committed().unwrap();
        let routes = routes::committed().unwrap();
        let required =
            requirements::requirements(&inputs.population, &policies, &applicability, &routes)
                .unwrap();
        let ledger = ApprovedLedger::committed().unwrap();
        let states = owners::committed().unwrap();
        let plan = partition::partition(&required, &ledger, &states).unwrap();
        Model {
            inputs,
            policies,
            applicability,
            routes,
            plan,
        }
    })
}

fn rejected<T>(name: &str, predicate: Predicate, result: Checked<T>) {
    let Err(diagnostics) = result else {
        panic!("{name}: negative accepted")
    };
    assert_eq!(diagnostics.len(), 1, "{name}: wrong guard {diagnostics:?}");
    assert_eq!(
        diagnostics[0].predicate, predicate,
        "{name}: wrong predicate"
    );
    println!(
        "CM FALSIFIER {name} REJECT {:?} subject={} datum={}",
        predicate, diagnostics[0].subject, diagnostics[0].datum
    );
}

#[test]
fn cm_policy_exhaustive() {
    let m = model();
    policy::validate(&m.inputs.population, &m.policies).unwrap();
    routes::validate(&m.routes).unwrap();
    for row in m.inputs.population.aliases.values() {
        applicability::for_row(row, &m.applicability, &m.policies).unwrap();
    }
    assert_eq!(m.policies.len(), 25);
    assert_eq!(m.applicability.len(), 154);
    assert_eq!(m.routes.bindings.len(), 1085);
    assert_eq!(m.routes.leaves.len(), 22);
    println!(
        "CM POLICY PASS policies=25 applicability=154 concrete_leaves=22 frozen_routes=1085 Twice_phases=2 generic_route_blockers=5"
    );
}

#[test]
fn cm_required_bag_exact() {
    let m = model();
    requirements::validate_bag(&m.plan.required).unwrap();
    for row in ["P0325", "P0326"] {
        let req = &m.plan.required.rows[row];
        assert!(matches!(
            req.count,
            Count::Frozen {
                keys: 70,
                encoding: 20,
                ..
            }
        ));
    }
    assert_eq!(m.plan.required.entries.len(), 3582);
    let mut prior = BTreeMap::<&str, usize>::new();
    for req in m
        .plan
        .required
        .rows
        .values()
        .filter(|r| matches!(r.count, Count::PriorAccepted(_)))
    {
        *prior.entry(&req.alias.verdict).or_default() += 1;
    }
    assert_eq!(
        prior,
        BTreeMap::from([
            ("Boundary", 10),
            ("Equal", 43),
            ("Lost", 5),
            ("Unreachable", 140)
        ])
    );
    println!(
        "CM REQUIRED PASS aliases=2204 policy_obligations=3582 c2_rows=1042 canonical_keys=78330 encoding=18840 g13=964 recorded_rows=44 recorded_keys=3025 PortBlocked_counts=920 prior=198 pair=140/40"
    );
}

#[test]
fn cm_partition_exact() {
    let p = &model().plan;
    partition::validate_partition(p).unwrap();
    assert!(partition::manifest_integrity(p));
    assert_eq!(p.bags.iter().map(|r| r.keys).sum::<usize>(), 78330);
    let proof_keys: usize = p
        .bags
        .iter()
        .filter(|r| matches!(r.mode, EnforcementMode::Proof(_)))
        .map(|r| r.keys)
        .sum();
    assert_eq!(proof_keys, 43390);
    assert!(
        p.bags
            .iter()
            .filter(|r| r.scope == Scope::Encoding)
            .all(|r| r.mode
                == EnforcementMode::Deferred(
                    owners::TRANSPORT.into(),
                    owners::TRANSPORT_BAR.into()
                ))
    );
    println!(
        "CM PARTITION PASS Execute=10505 Deferred=24435 Proof=43390 residual_deferred=5595 encoding_deferred=18840 proof_modes=5 multiplicity=1 guests=0 receipts=0"
    );
}

#[test]
fn cm_owner_integrity() {
    let m = model();
    owners::owner_integrity(&m.inputs, &m.plan.owners).unwrap();
    partition::validate_ledger(&m.plan.required, &m.plan.ledger, &m.plan.owners).unwrap();
    let g13: BTreeSet<_> = m
        .plan
        .required
        .rows
        .values()
        .filter(|r| r.alias.cohort == "g13")
        .map(|r| &r.alias.owner)
        .collect();
    assert_eq!(g13.len(), 44);
    assert_eq!(&m.inputs.owners["S10"][2..5], &["28", "10", "1"]);
    assert_eq!(m.plan.ledger.owned.len(), 1082);
    assert!(
        m.plan
            .owners
            .registry
            .values()
            .all(|r| r.state == OwnerState::Open)
    );
    println!(
        "CM OWNERS PASS states=51 Open=51 Done=0 campaign=1 g13_steps=44 g13_transport=1 source_duties=5 g13_rows=964 S10=28+10+1"
    );
    println!("CM TRANSPORT BAR {}", owners::TRANSPORT_BAR_TEXT);
}

#[test]
fn cm_c2_closure_is_blocked() {
    let p = &model().plan;
    assert!(partition::manifest_integrity(p));
    assert!(!partition::owner_closure(p, "G15-C2"));
    assert_eq!(
        p.required
            .rows
            .values()
            .filter(|r| r.alias.verdict == "Residual")
            .count(),
        118
    );
    assert_eq!(
        p.required
            .rows
            .values()
            .filter(|r| matches!(r.count, Count::Frozen { .. }) && r.alias.verdict == "Lost")
            .count(),
        0
    );
    println!(
        "CM CLOSURE BLOCKED owner=G15-C2 Residual=118 Lost=0 counted_integrity=true full_CM_green=false promotions=0"
    );
}

#[test]
fn cm_missing_leaf_is_named() {
    let mut routes = model().routes.clone();
    routes.leaves.remove("held_and_splice.held-evaluate");
    rejected(
        "missing-leaf",
        Predicate::MissingLeaf,
        routes::validate(&routes),
    );
}

#[test]
fn cm_missing_applicability_is_named() {
    let m = model();
    let row = &m.inputs.population.aliases["P0325"];
    let mut rules = m.applicability.clone();
    rules.remove(&row.policies.join(";"));
    rejected(
        "missing-applicability",
        Predicate::MissingApplicability,
        applicability::for_row(row, &rules, &m.policies),
    );
}

#[test]
fn cm_missing_concrete_route_and_twice_phase_are_named() {
    let m = model();
    let mut routes = m.routes.clone();
    routes
        .gaps
        .remove(&("P0508".into(), "AccountingMode".into()));
    rejected(
        "missing-concrete-mode",
        Predicate::MissingRoute,
        routes::validate(&routes),
    );
    let mut routes = m.routes.clone();
    routes.phases.remove("projection_then_value");
    rejected(
        "missing-distinct-Twice-phase",
        Predicate::MissingRoute,
        routes::validate(&routes),
    );
}

#[test]
fn cm_duplicate_requirement_is_counted_first() {
    let entry = model().plan.required.entries.first_key_value().unwrap();
    rejected(
        "duplicate-requirement",
        Predicate::DuplicateRequirement,
        requirements::counted(vec![
            (entry.0.clone(), entry.1.clone()),
            (entry.0.clone(), entry.1.clone()),
        ]),
    );
}

#[test]
fn cm_duplicate_route_requirement_is_named() {
    let mut routes = model().routes.clone();
    routes.bindings.push(routes.bindings[0].clone());
    rejected(
        "duplicate-route-requirement",
        Predicate::DuplicateRequirement,
        routes::validate(&routes),
    );
}

#[test]
fn cm_policy_unknown_duplicate_and_default_are_named() {
    let m = model();
    let mut population = m.inputs.population.clone();
    population
        .aliases
        .values_mut()
        .next()
        .unwrap()
        .policies
        .push("UnclassifiedAddedPolicy".into());
    rejected(
        "unclassified-policy",
        Predicate::UnclassifiedPolicy,
        policy::validate(&population, &m.policies),
    );
    let entry = m.policies.values().next().unwrap().clone();
    rejected(
        "duplicate-policy",
        Predicate::DuplicatePolicy,
        policy::declarations(vec![entry.clone(), entry]),
    );
    for token in ["Other", "*", "_", "default"] {
        rejected(
            token,
            Predicate::ForbiddenPolicy,
            policy::PolicyId::parse(token),
        );
    }
}

#[test]
fn cm_lost_frozen_key_is_named() {
    let mut plan = model().plan.clone();
    plan.bags[0].keys -= 1;
    rejected(
        "lost-frozen-key",
        Predicate::LostFrozenKey,
        partition::validate_partition(&plan),
    );
    assert!(!partition::manifest_integrity(&plan));
}

#[test]
fn cm_duplicate_partition_is_named() {
    let mut plan = model().plan.clone();
    plan.bags.push(plan.bags[0].clone());
    rejected(
        "duplicate-partition-key",
        Predicate::PartitionMultiplicity,
        partition::validate_partition(&plan),
    );
}

#[test]
fn cm_encoding_execution_and_owner_are_named() {
    let mut plan = model().plan.clone();
    plan.bags
        .iter_mut()
        .find(|r| r.scope == Scope::Encoding)
        .unwrap()
        .mode = EnforcementMode::Execute;
    rejected(
        "encoding-execution",
        Predicate::EncodingExecution,
        partition::validate_partition(&plan),
    );
    let mut plan = model().plan.clone();
    plan.bags
        .iter_mut()
        .find(|r| r.scope == Scope::Encoding)
        .unwrap()
        .mode = EnforcementMode::Deferred("G15-C2".into(), owners::TRANSPORT_BAR.into());
    rejected(
        "encoding-wrong-owner",
        Predicate::EncodingOwner,
        partition::validate_partition(&plan),
    );
}

#[test]
fn cm_unknown_and_completed_owners_are_named() {
    let m = model();
    let mut ledger = m.plan.ledger.clone();
    ledger.owned[0].owner = "UnknownOwner".into();
    rejected(
        "unknown-owner",
        Predicate::UnknownOwner,
        partition::validate_ledger(&m.plan.required, &ledger, &m.plan.owners),
    );
    let mut ledger = m.plan.ledger.clone();
    ledger.owned[0].owner = "S1".into();
    rejected(
        "completed-S1-cannot-own-deferral",
        Predicate::UnknownOwner,
        partition::validate_ledger(&m.plan.required, &ledger, &m.plan.owners),
    );
    let mut states = m.plan.owners.clone();
    states.registry.get_mut("G15-C2").unwrap().state = OwnerState::Done("negative-only".into());
    rejected(
        "Done-owner-for-residual",
        Predicate::OwnerDone,
        partition::partition(&m.plan.required, &m.plan.ledger, &states),
    );
}

#[test]
fn cm_wrong_row_owner_is_named() {
    let m = model();
    let mut ledger = m.plan.ledger.clone();
    ledger.row_owners.insert("P0325".into(), "S10".into());
    rejected(
        "reassigned-C2-row",
        Predicate::OwnerReassignment,
        partition::validate_ledger(&m.plan.required, &ledger, &m.plan.owners),
    );
}

#[test]
fn cm_source_duty_reassignment_omission_and_orphan_are_named() {
    let m = model();
    let mut states = m.plan.owners.clone();
    states.duties.get_mut("ToExpression").unwrap().owner = "G15-C2".into();
    rejected(
        "reassigned-source-duty",
        Predicate::DutyReassignment,
        owners::owner_integrity(&m.inputs, &states),
    );
    let mut states = m.plan.owners.clone();
    states.duties.remove("Session::lower_text");
    rejected(
        "missing-source-duty",
        Predicate::MissingDuty,
        owners::owner_integrity(&m.inputs, &states),
    );
    let mut states = m.plan.owners.clone();
    states
        .duties
        .get_mut("ToExpression")
        .unwrap()
        .dependencies
        .clear();
    rejected(
        "orphaned-source-duty",
        Predicate::DutySubjectMismatch,
        owners::owner_integrity(&m.inputs, &states),
    );
}

#[test]
fn cm_premature_closure_with_one_residual_is_named() {
    let mut plan = model().plan.clone();
    plan.owners.registry.get_mut("G15-C2").unwrap().state =
        OwnerState::Done("negative-only".into());
    plan.ledger
        .owned
        .retain(|r| r.owner != "G15-C2" || r.row == "P0325");
    plan.bags.retain(|r| {
        !matches!(&r.mode, EnforcementMode::Deferred(id, _) if id == "G15-C2") || r.row == "P0325"
    });
    assert_eq!(
        plan.ledger
            .owned
            .iter()
            .filter(|r| r.owner == "G15-C2")
            .count(),
        1
    );
    rejected(
        "C2-Done-one-Residual",
        Predicate::PrematureClosure,
        partition::closure_check(&plan, "G15-C2"),
    );
    assert!(!partition::owner_closure(&plan, "G15-C2"));
}

#[test]
fn cm_proof_certificate_identity_is_exact() {
    let m = model();
    let mut ledger = m.plan.ledger.clone();
    ledger.proofs.remove("Transferred");
    rejected(
        "missing-proof-mode",
        Predicate::MissingProof,
        partition::validate_ledger(&m.plan.required, &ledger, &m.plan.owners),
    );
    let mut plan = m.plan.clone();
    let part = plan
        .bags
        .iter_mut()
        .find(|r| matches!(r.mode, EnforcementMode::Proof(_)))
        .unwrap();
    part.mode = EnforcementMode::Proof("wrong-certificate".into());
    rejected(
        "wrong-proof-certificate",
        Predicate::ProofMismatch,
        partition::validate_partition(&plan),
    );
}

#[test]
fn cm_bag_duplicate_policy_metadata_is_named() {
    let mut plan = model().plan.clone();
    let policies = &mut plan.required.rows.get_mut("P0001").unwrap().policies;
    policies[1] = policies[0];
    rejected(
        "bag-duplicate-policy-metadata",
        Predicate::DuplicateRequirement,
        requirements::validate_bag(&plan.required),
    );
    assert!(!partition::manifest_integrity(&plan));
}

#[test]
fn cm_bag_extra_policy_requirement_is_named() {
    let m = model();
    let mut plan = m.plan.clone();
    let (key, entry) = plan
        .required
        .entries
        .iter()
        .find(|(_, r)| r.row == "P0325")
        .unwrap();
    let mut key = key.clone();
    let mut entry = entry.clone();
    let extra = policy::PolicyId::ErrorTransform;
    match &mut key {
        requirements::ObligationKey::Source(k) => k.policy = format!("{extra:?}"),
        requirements::ObligationKey::PortBlocked { policy, .. } => *policy = extra,
    }
    entry.policy = extra;
    entry.contexts = m.policies[&extra].contexts.clone();
    entry.variants = m.policies[&extra].variants.clone();
    assert!(plan.required.entries.insert(key, entry).is_none());
    rejected(
        "bag-extra-policy-obligation",
        Predicate::RequirementSealMismatch,
        requirements::validate_bag(&plan.required),
    );
    assert!(!partition::manifest_integrity(&plan));
}

#[test]
fn cm_bag_recorded_g13_counts_are_row_bound() {
    let mut plan = model().plan.clone();
    for (row, replacement) in [("P0716", 81), ("P0717", 79)] {
        assert_eq!(plan.required.rows[row].count, Count::Recorded(80));
        plan.required.rows.get_mut(row).unwrap().count = Count::Recorded(replacement);
    }
    rejected(
        "bag-balanced-G13-count-corruption",
        Predicate::RequirementSealMismatch,
        requirements::validate_bag(&plan.required),
    );
    assert!(!partition::manifest_integrity(&plan));
}

#[test]
fn cm_bag_consumer_route_is_bound() {
    let mut plan = model().plan.clone();
    plan.required.rows.get_mut("P0001").unwrap().routes[0]
        .consumer
        .push_str("::unapproved-route");
    rejected(
        "bag-changed-consumer-projection",
        Predicate::RouteSealMismatch,
        requirements::validate_bag(&plan.required),
    );
    assert!(!partition::manifest_integrity(&plan));
}

#[test]
#[ignore = "full counted campaign: run explicitly once per feature mode after shape"]
fn cm_requirements_campaign() {
    let start = Instant::now();
    let m = model();
    assert!(partition::manifest_integrity(&m.plan));
    let mut counts = BTreeMap::<(String, String, String), usize>::new();
    for (row, req) in &m.plan.required.rows {
        if let Count::Frozen {
            keys: _,
            encoding: _,
            digest: _,
        } = &req.count
        {
            for route in &req.routes {
                for (scope, n) in [
                    (Scope::Nonencoding, route.keys() - route.encoding()),
                    (Scope::Encoding, route.encoding()),
                ] {
                    let mode = if scope == Scope::Encoding {
                        EnforcementMode::Deferred(
                            owners::TRANSPORT.into(),
                            owners::TRANSPORT_BAR.into(),
                        )
                    } else {
                        m.plan.ledger.rows[row].mode.clone()
                    };
                    let owner = match &mode {
                        EnforcementMode::Deferred(id, _) => id,
                        EnforcementMode::Execute | EnforcementMode::Proof(_) => &req.alias.owner,
                    };
                    *counts
                        .entry(("owner".into(), owner.clone(), mode.name().into()))
                        .or_default() += n;
                    *counts
                        .entry(("route".into(), route.consumer.clone(), mode.name().into()))
                        .or_default() += n;
                    for policy in &req.policies {
                        *counts
                            .entry(("policy".into(), format!("{policy:?}"), mode.name().into()))
                            .or_default() += n;
                    }
                }
            }
        }
    }
    for ((kind, id, mode), count) in counts {
        println!("CM COUNT {kind}\t{id}\t{mode}\t{count}");
    }
    for owner in m.plan.owners.registry.values() {
        println!(
            "CM OWNER {} kind={} state={:?} subject={} bar={}",
            owner.id, owner.kind, owner.state, owner.subject, owner.bar
        );
        if owner.kind == "g13" {
            let rows: Vec<_> = m
                .plan
                .required
                .rows
                .values()
                .filter(|r| r.alias.cohort == "g13" && r.alias.owner == owner.id)
                .collect();
            let recorded: usize = rows
                .iter()
                .filter_map(|r| {
                    if let Count::Recorded(n) = r.count {
                        Some(n)
                    } else {
                        None
                    }
                })
                .sum();
            println!(
                "CM G13 {} Deferred_rows={} recorded_keys={} NotYetBound={}",
                owner.id,
                rows.len(),
                recorded,
                rows.iter()
                    .filter(|r| r.count == Count::PortBlocked)
                    .count()
            );
        }
    }
    for duty in m.plan.owners.duties.values() {
        println!(
            "CM DUTY {} owner={} subjects={} dependencies={} bar={}",
            duty.id,
            duty.owner,
            duty.subjects.join(";"),
            duty.dependencies.join(";"),
            duty.bar
        );
    }
    for d in &m.plan.required.diagnostics {
        println!("{}", d.line());
    }
    for r in &m.inputs.crosswalks {
        if r[0] == "737-source-v1" && r[2] == "-" {
            println!("CM1 DIAGNOSTIC {}\t{}\t{}", r[3], r[1], r[4]);
        }
    }
    println!(
        "CM CAMPAIGN PASS feature_hash={} aliases=2204 C2_rows=1042 keys=78330 encoding=18840 G13_rows=964 G13_owners=44 G13_recorded_rows=44 G13_recorded_keys=3025 G13_NotYetBound=920 prior=198 Residual=118 Lost=0 Execute=10505 Deferred=24435 Proof=43390 policies=25 applicability=154 route_bindings=1085 leaves=22 proof_modes=5 duties=5 states=51 counted_integrity=true owner_closure_C2=false full_CM_green=false guests=0 receipts=0 promotions=0 elapsed_ms={}",
        cfg!(feature = "hash"),
        start.elapsed().as_millis()
    );
}
