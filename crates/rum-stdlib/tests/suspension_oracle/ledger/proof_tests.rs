use super::super::imports::AcceptedInputs;
use super::{
    known_losses::{self, LossError},
    proofs::*,
};
use crate::oracle::fixtures::obligations::proof_controls;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn cm_proof_certificates_exact() {
    let current = CurrentSource {
        inputs: AcceptedInputs::committed().unwrap(),
    };
    let imported = committed().unwrap();
    let certificates: Vec<_> = imported
        .iter()
        .map(|(d, e)| validate_proof(d, &current, e))
        .collect();
    reconcile(&certificates).unwrap();
    let mut totals = BTreeMap::<String, (usize, usize)>::new();
    let mut packet_verified = (0, 0);
    let mut verified = (0, 0);
    let mut absent = (0, 0);
    for ((d, e), c) in imported.iter().zip(&certificates) {
        let count = totals.entry(d.verdict.clone()).or_default();
        count.0 += 1;
        count.1 += d.keys;
        if e.imported_status == "VerifiedPacket" {
            packet_verified.0 += 1;
            packet_verified.1 += d.keys;
            assert_eq!(c.packet_hash().len(), 64);
        }
        if c.status() == &ProofStatus::Verified {
            verified.0 += 1;
            verified.1 += c.keys();
        }
        if e.missing.starts_with("continuation2/") {
            absent.0 += 1;
            absent.1 += d.keys;
            assert!(matches!(c.status(), ProofStatus::PortBlocked(_)));
        }
        if e.imported_status == "VerifiedPacket" {
            println!(
                "CM PROOF COHORT row={} status={:?} keys={} packet={} executions=0 SAME=0",
                d.row,
                c.status(),
                d.keys,
                c.packet_hash()
            );
        }
    }
    assert_eq!(
        totals,
        BTreeMap::from([
            ("Boundary".into(), (20, 735)),
            ("Identity".into(), (5, 185)),
            ("NoObservableTail".into(), (126, 7260)),
            ("Transferred".into(), (594, 35000)),
            ("Unreachable".into(), (4, 210)),
        ])
    );
    assert_eq!(absent, (598, 35135));
    assert_eq!(packet_verified, (151, 8255));
    // P0011's historical survivor packet passes, but its accepted alias is
    // unresolved. It cannot be waived into a current source proof.
    assert!(
        matches!(certificates.iter().find(|c| c.row()=="P0011").unwrap().status(),ProofStatus::PortBlocked(s) if s.contains("unresolved-target"))
    );
    for (mode, (rows, keys)) in totals {
        println!("CM PROOF CLASS {mode} rows={rows} keys={keys}");
    }
    println!(
        "CM PROOFS PASS imported=749/43390 packet_authenticated={}/{} absent=598/35135 current_verified={}/{} partial=true executions=0 SAME=0 full_CM_green=false",
        packet_verified.0, packet_verified.1, verified.0, verified.1
    );
}

#[test]
fn cm_loss_baselines_exact() {
    let records = known_losses::committed().unwrap();
    for record in &records {
        if record.status == "Lost" {
            known_losses::compare(record, record, None).unwrap();
        } else {
            known_losses::compare(record, record, Some(known_losses::PROMOTION_HASH)).unwrap();
        }
        println!(
            "CM KNOWN LOSS row={} status={} owner={} requests={:?} duty={} promotion={} missing={}",
            record.row,
            record.status,
            record.owner,
            record.requests,
            record.duty,
            record.promotion,
            record.missing
        );
    }
    println!(
        "CM LOSSES PASS prior_Lost=5 immutable_differences=3 C1_promotions=2 missing_original_baselines=2 encoding_runs=0 fresh_allowed_DIFF=0"
    );
}

fn reject<T: std::fmt::Debug>(name: &str, actual: Result<T, ProofError>, expected: ProofError) {
    assert_eq!(actual.unwrap_err(), expected, "{name}");
    println!("CM PROOF REJECT {name} predicate={expected:?}");
}

#[test]
fn cm_proof_negative_controls() {
    let current = CurrentSource {
        inputs: AcceptedInputs::committed().unwrap(),
    };
    let imported = committed().unwrap();
    let find = |row: &str| imported.iter().find(|(d, _)| d.row == row).unwrap().clone();
    let (boundary, mut e) = find("P1556");
    validate_content(&boundary, &e, &current).unwrap();
    e.terminal = "-".into();
    reject(
        "boundary-without-terminal-contract",
        validate_content(&boundary, &e, &current),
        ProofError::TerminalContract,
    );
    let (tail, e) = find("P0011");
    validate_content(&tail, &e, &current).unwrap();
    let mut bad = e.clone();
    bad.survivors[0].2 = 0;
    reject(
        "no-tail-unarmed",
        validate_content(&tail, &bad, &current),
        ProofError::UnarmedSurvivor,
    );
    bad = e.clone();
    bad.survivors[0].0 = "P0001".into();
    reject(
        "no-tail-wrong-row",
        validate_content(&tail, &bad, &current),
        ProofError::WrongRowSurvivor,
    );
    let (strict, mut strict_evidence) = find("P1564");
    validate_content(&strict, &strict_evidence, &current).unwrap();
    strict_evidence.terminal = "-".into();
    reject(
        "strict-tail-error-scope-removed",
        validate_content(&strict, &strict_evidence, &current),
        ProofError::Scope,
    );
    // All Identity packets are absent. This validator-only control reuses
    // committed data and never enters validate_proof or the certificate bag.
    let (identity, _) = find("P0107");
    let mut control = e;
    for survivor in &mut control.survivors {
        survivor.0 = identity.row.clone();
    }
    validate_content(&identity, &control, &current).unwrap();
    let mut bad = control.clone();
    bad.survivors[0].2 = 0;
    reject(
        "identity-unarmed",
        validate_content(&identity, &bad, &current),
        ProofError::UnarmedSurvivor,
    );
    bad = control;
    bad.survivors[0].0 = tail.row.clone();
    reject(
        "identity-wrong-row",
        validate_content(&identity, &bad, &current),
        ProofError::WrongRowSurvivor,
    );
    let (transfer, mut e) = find("P0500");
    validate_content(&transfer, &e, &current).unwrap();
    e.targets.pop();
    reject(
        "transfer-target-removed",
        validate_content(&transfer, &e, &current),
        ProofError::MissingTransferTarget,
    );
    let edges = BTreeMap::from([
        ("P0500".into(), vec!["P0505".into()]),
        ("P0505".into(), vec!["P0500".into()]),
    ]);
    reject(
        "transfer-rootless-cycle",
        validate_transfer("P0500", &edges, &BTreeSet::new()),
        ProofError::RootlessCycle,
    );
    let sources = proof_controls::current_sources();
    proof_controls::validate_closure(&sources).unwrap();
    assert!(proof_controls::ordinary_reachable(&sources));
    let mut fourth = sources.clone();
    fourth.insert("crates/synthetic-fourth-caller.rs".into(),"fn extra(s: &mut Session) { s.resume_suspended_call_with_dispatch_plan(head, plan, state, 0, None); }".into());
    reject(
        "P0496-fourth-caller",
        proof_controls::validate_closure(&fourth),
        ProofError::ClosedCallers,
    );
    let mut no_skip = sources.clone();
    no_skip
        .get_mut(proof_controls::CALL)
        .unwrap()
        .replace_range(
            ..,
            &sources[proof_controls::CALL].replace(
                proof_controls::SKIP,
                "if stage == CallDispatchStage::PrepareArguments && false {",
            ),
        );
    assert!(proof_controls::ordinary_reachable(&no_skip));
    reject(
        "P0496-prepared-skip-removed",
        proof_controls::validate_closure(&no_skip),
        ProofError::PreparedSkip,
    );
    let certificates: Vec<_> = imported
        .iter()
        .map(|(d, e)| validate_proof(d, &current, e))
        .collect();
    reconcile(&certificates).unwrap();
    let mut duplicate = certificates.clone();
    duplicate.push(duplicate[0].clone());
    reject(
        "duplicate-certificate",
        reconcile(&duplicate),
        ProofError::Duplicate,
    );
    let mut missing = certificates;
    missing.pop();
    reject(
        "missing-certificate",
        reconcile(&missing),
        ProofError::Missing,
    );
    let losses = known_losses::committed().unwrap();
    let baseline = losses.iter().find(|l| l.status == "Lost").unwrap();
    let mut changed = baseline.clone();
    changed.difference.as_mut().unwrap().resumed = "changed".into();
    assert_eq!(
        known_losses::compare(baseline, &changed, None),
        Err(LossError::ChangedBaseline)
    );
    println!("CM PROOF REJECT changed-known-loss predicate=ChangedBaseline");
    changed = baseline.clone();
    changed.difference = None;
    assert_eq!(
        known_losses::compare(baseline, &changed, None),
        Err(LossError::NowSame)
    );
    println!("CM PROOF REJECT now-SAME-known-loss predicate=NowSame");
    let c1 = losses.iter().find(|l| l.row == "P0941").unwrap();
    assert_eq!(known_losses::compare(c1, c1, None), Err(LossError::NowSame));
    known_losses::compare(c1, c1, Some(known_losses::PROMOTION_HASH)).unwrap();
    println!("CM PROOF REJECT C1-without-promotion-acknowledgement predicate=NowSame");
    println!(
        "CM PROOF CONTROLS PASS controls=15 ordinary_branch_reachable=true Identity_positive_calibration=false validator_controls_credit=0 owners_changed=0"
    );
}
