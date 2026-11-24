//! Synthetic transactions exercise the validator; no live owner is promoted.
use super::promotions::*;
use crate::oracle::obligations::{
    bindings::TupleKey,
    owners::{self, OwnerState},
};

const OWNER: &str = "synthetic-owner";
const OTHER: &str = "synthetic-other-owner";
const PROMOTION: &str = "synthetic-CM740";

fn ids(names: &[&str]) -> Ids {
    names.iter().map(|s| (*s).into()).collect()
}

fn key(row: &str, variant: &str) -> TupleKey {
    TupleKey([
        row.into(),
        variant.into(),
        "value.base".into(),
        "synthetic-route".into(),
        "synthetic-descriptor".into(),
    ])
}

fn requirement(row: &str, owner: &str, variants: &[&str]) -> Requirement {
    Requirement {
        owner: owner.into(),
        tuples: variants.iter().map(|v| key(row, v)).collect(),
        source: format!("synthetic-source:{row}"),
        tail: format!("synthetic-tail:{row}"),
        per_cell_mutants: row.starts_with("synthetic-pair"),
        alternative: None,
        dependencies: Ids::new(),
        crosswalks: ids(&["synthetic-crosswalk"]),
        certificates: ids(&["synthetic-row-certificate"]),
    }
}

fn fixture() -> (Snapshot, Promotion, Evidence) {
    let mut before = Snapshot::default();
    before.owners.insert(OWNER.into(), OwnerState::Open);
    before.owners.insert(OTHER.into(), OwnerState::Open);
    for row in ["synthetic-pair-1", "synthetic-pair-2", "synthetic-value"] {
        before
            .requirements
            .insert(row.into(), requirement(row, OWNER, &["Middle", "Error"]));
        before.owned_rows.insert(row.into(), OWNER.into());
    }
    for row in ["synthetic-pair-1", "synthetic-pair-2"] {
        before.requirements.get_mut(row).unwrap().tuples =
            ["Middle", "Twice", "Error", "AlreadyResumed", "Nested"]
                .into_iter()
                .flat_map(|variant| {
                    (0..10).map(move |context| {
                        let mut k = key(row, variant);
                        k.0[2] = format!("synthetic-context-{context}");
                        k
                    })
                })
                .collect();
    }
    let mut proof = requirement("synthetic-proof", OWNER, &["Middle"]);
    proof.alternative = Some("synthetic-row-certificate".into());
    proof.dependencies = ids(&["synthetic-value"]);
    before.requirements.insert("synthetic-proof".into(), proof);
    before
        .owned_rows
        .insert("synthetic-proof".into(), OWNER.into());
    before.requirements.insert(
        "synthetic-other".into(),
        requirement("synthetic-other", OTHER, &["Middle"]),
    );
    before
        .owned_rows
        .insert("synthetic-other".into(), OTHER.into());
    before.requirements.insert(
        "synthetic-regression".into(),
        requirement("synthetic-regression", OTHER, &["Middle"]),
    );
    before.active_rows = ids(&["synthetic-regression"]);
    let duty = Duty {
        owner: OWNER.into(),
        subject: "synthetic-source-subject".into(),
        bar: "synthetic-reviewed-bar".into(),
        dependencies: ids(&["synthetic-value"]),
        certificate: "synthetic-duty-certificate".into(),
    };
    before
        .tuple_duties
        .insert("synthetic-tuple-duty".into(), duty.clone());
    before
        .source_duties
        .insert("synthetic-source-duty".into(), duty);
    before.open_tuple_duties = ids(&["synthetic-tuple-duty"]);
    before.open_source_duties = ids(&["synthetic-source-duty"]);
    for (id, row, owner, promotion) in [
        ("synthetic-loss", "synthetic-value", OWNER, PROMOTION),
        (
            "synthetic-unrelated-loss",
            "synthetic-other",
            OTHER,
            "synthetic-other-promotion",
        ),
    ] {
        before.losses.insert(
            id.into(),
            Loss {
                owner: owner.into(),
                row: row.into(),
                tuple: key(row, "Middle"),
                program: format!("synthetic-program:{row}"),
                difference: "synthetic-ordered-trace-DIFF".into(),
                promotion: promotion.into(),
            },
        );
    }
    before.crosswalks.insert(
        "synthetic-crosswalk".into(),
        "synthetic-approved-exact-crosswalk".into(),
    );
    before.certificates.insert(
        "synthetic-row-certificate".into(),
        "synthetic-approved-row-certificate".into(),
    );
    before.certificates.insert(
        "synthetic-duty-certificate".into(),
        "synthetic-approved-duty-certificate".into(),
    );
    let mut encoding = key("synthetic-other", "Encoding");
    encoding.0[2] = "guest_heads.synthetic".into();
    before.encoding.insert(encoding);
    let rows: Vec<_> = before
        .owned_rows
        .iter()
        .filter(|(_, owner)| *owner == OWNER)
        .map(|(id, _)| id.clone())
        .collect();
    let mut evidence = Evidence::default();
    for row in &rows {
        let r = &before.requirements[row];
        let tuples: Vec<_> = r.tuples.iter().cloned().collect();
        evidence.rows.insert(
            row.clone(),
            RowEvidence {
                row: row.clone(),
                source: r.source.clone(),
                tail: r.tail.clone(),
                real_tuples: if r.alternative.is_none() {
                    tuples.clone()
                } else {
                    vec![]
                },
                caught_tuples: if r.alternative.is_none() {
                    tuples
                } else {
                    vec![]
                },
                alternative: r.alternative.clone(),
            },
        );
        if r.alternative.is_none() {
            evidence
                .tuples
                .extend(r.tuples.iter().cloned().map(|k| (k, Observation::Same)));
        }
    }
    evidence
        .tuples
        .insert(key("synthetic-regression", "Middle"), Observation::Same);
    evidence.tuple_duties = before.tuple_duties.clone();
    evidence.source_duties = before.source_duties.clone();
    evidence.crosswalks = before.crosswalks.clone();
    evidence.certificates = before.certificates.clone();
    evidence
        .losses
        .insert("synthetic-loss".into(), Observation::Same);
    evidence.losses.insert(
        "synthetic-unrelated-loss".into(),
        Observation::Diff("synthetic-ordered-trace-DIFF".into()),
    );
    let mut after = before.clone();
    for row in &rows {
        after.owned_rows.remove(row);
        if before.requirements[row].alternative.is_none() {
            after.active_rows.insert(row.clone());
        }
    }
    after.open_source_duties.clear();
    after.open_tuple_duties.clear();
    after.losses.remove("synthetic-loss");
    after
        .owners
        .insert(OWNER.into(), OwnerState::Done(PROMOTION.into()));
    let mut proposal = Promotion {
        id: PROMOTION.into(),
        owner: OWNER.into(),
        before_hash: before.fingerprint(),
        exact_rows: rows,
        tuple_duties: before.open_tuple_duties.clone(),
        source_duties: before.open_source_duties.clone(),
        losses_removed: ids(&["synthetic-loss"]),
        crosswalks: before.crosswalks.clone(),
        certificates: before.certificates.clone(),
        after,
        after_hash: String::new(),
    };
    proposal
        .after
        .receipts
        .insert(PROMOTION.into(), receipt_hash(&proposal));
    proposal.after_hash = proposal.after.fingerprint();
    (before, proposal, evidence)
}

#[test]
fn cm_promotion_atomic() {
    let (before, proposal, evidence) = fixture();
    let table = include_str!("../inventory/obligations/promotions.tsv");
    let mut lines = table.lines();
    assert_eq!(
        lines.next(),
        Some(
            "scope\tpromotion\towner\tbefore_hash\tafter_hash\trows\ttuple_duties\tsource_duties\tlosses_removed\tcrosswalks\tcertificates\treceipt"
        )
    );
    assert_eq!(lines.next(), Some(declaration(&before, &proposal).as_str()));
    assert!(lines.next().is_none());
    let original = before.clone();
    let delta = validate_promotion(&before, &proposal, &evidence).unwrap();
    assert_eq!(before, original, "validation is pure");
    let mut result = before.clone();
    delta.apply(&mut result).unwrap();
    assert_eq!(result, proposal.after);
    assert_eq!(result.requirements, before.requirements);
    assert_eq!(result.tuple_duties, before.tuple_duties);
    assert_eq!(result.source_duties, before.source_duties);
    assert_eq!(result.crosswalks, before.crosswalks);
    assert_eq!(result.certificates, before.certificates);
    assert_eq!(result.encoding, before.encoding);
    assert_eq!(result.receipts.len(), before.receipts.len() + 1);
    assert_eq!(result.owners[OTHER], OwnerState::Open);
    let states = owners::committed().unwrap();
    assert_eq!(states.registry.len(), 51);
    assert!(
        states
            .registry
            .values()
            .all(|owner| owner.state == OwnerState::Open)
    );
    assert_eq!(states.duties.len(), 5);
    println!(
        "CM4 PROMOTION ATOMIC PASS synthetic_rows=4 tuple_duties=1 source_duties=1 removed_losses=1 receipts=1 real_owners=51 real_Open=51 real_promotions=0"
    );
}

fn reject(
    name: &str,
    expected: PromotionError,
    change: impl FnOnce(&mut Snapshot, &mut Promotion, &mut Evidence),
) {
    let (mut before, mut proposal, mut evidence) = fixture();
    change(&mut before, &mut proposal, &mut evidence);
    if expected != PromotionError::BeforeHash {
        proposal.before_hash = before.fingerprint();
    }
    if expected != PromotionError::AfterHash {
        proposal.after_hash = proposal.after.fingerprint();
    }
    let original = before.clone();
    let actual = validate_promotion(&before, &proposal, &evidence).err();
    assert_eq!(actual, Some(expected), "{name}");
    assert_eq!(before, original, "{name} mutated before-state");
    println!(
        "CM4 PROMOTION NEGATIVE control={name} predicate={expected:?} before_unchanged=true synthetic=true"
    );
}

#[test]
fn cm_promotion_negative_controls() {
    use PromotionError as E;
    reject("partial-owner", E::RowSet, |_, p, _| {
        p.exact_rows.pop();
    });
    reject("duplicate-row", E::RowSet, |_, p, _| {
        p.exact_rows.push(p.exact_rows[0].clone());
    });
    reject("delete-a-row", E::WrongDelta, |_, p, _| {
        p.after.requirements.remove("synthetic-value");
    });
    reject("C2-done-with-residuals", E::DoneWithDuty, |_, p, _| {
        p.after
            .owned_rows
            .insert("synthetic-value".into(), OWNER.into());
    });
    reject("done-before-removal", E::DoneWithDuty, |b, _, _| {
        b.owners
            .insert(OWNER.into(), OwnerState::Done(PROMOTION.into()));
    });
    reject("one-lingering-source-duty", E::DoneWithDuty, |_, p, _| {
        p.after
            .open_source_duties
            .insert("synthetic-source-duty".into());
    });
    reject("one-lingering-tuple-duty", E::DoneWithDuty, |_, p, _| {
        p.after
            .open_tuple_duties
            .insert("synthetic-tuple-duty".into());
    });
    reject("wrong-row-mutant", E::WrongRow, |_, _, e| {
        e.rows.get_mut("synthetic-value").unwrap().row = "synthetic-other".into();
    });
    reject("wrong-source-mutant", E::Source, |_, _, e| {
        e.rows.get_mut("synthetic-value").unwrap().source = "another-source".into();
    });
    reject("duplicate-tuple", E::DuplicateTuple, |_, _, e| {
        let row = e.rows.get_mut("synthetic-value").unwrap();
        row.real_tuples.push(row.real_tuples[0].clone());
    });
    reject("unregistered-owner", E::Owner, |_, p, _| {
        p.owner = "unregistered".into();
    });
    reject("reused-receipt", E::Receipt, |b, _, _| {
        b.receipts
            .insert(PROMOTION.into(), "earlier-receipt".into());
    });
    reject("surviving-identity-promoted", E::OwnMutation, |_, _, e| {
        e.rows
            .get_mut("synthetic-value")
            .unwrap()
            .caught_tuples
            .clear();
    });
    reject(
        "incomplete-matrix-promoted",
        E::MissingRealTuple,
        |_, _, e| {
            e.rows.get_mut("synthetic-value").unwrap().real_tuples.pop();
        },
    );
    for row in ["synthetic-pair-1", "synthetic-pair-2"] {
        reject(
            &format!("partial-held-pair-{row}-45-of-50"),
            E::PartialHeldPair,
            |_, _, e| {
                let caught = &mut e.rows.get_mut(row).unwrap().caught_tuples;
                assert_eq!(caught.len(), 50);
                let missing: std::collections::BTreeSet<_> = caught
                    .iter()
                    .filter(|k| k.variant() == "Error")
                    .take(5)
                    .cloned()
                    .collect();
                assert_eq!(missing.len(), 5);
                caught.retain(|key| !missing.contains(key));
                assert_eq!(caught.len(), 45);
            },
        );
    }
    reject("missing-tuple-duty", E::TupleDuty, |_, _, e| {
        e.tuple_duties.clear();
    });
    reject("missing-source-duty", E::SourceDuty, |_, _, e| {
        e.source_duties.clear();
    });
    reject("wrong-duty-bar", E::SourceDuty, |_, _, e| {
        e.source_duties
            .get_mut("synthetic-source-duty")
            .unwrap()
            .bar = "weaker".into();
    });
    reject("unresolved-dependency", E::Dependency, |b, _, _| {
        b.requirements
            .get_mut("synthetic-value")
            .unwrap()
            .dependencies
            .insert("synthetic-other".into());
    });
    reject("missing-duty-dependency", E::Dependency, |b, _, e| {
        b.source_duties
            .get_mut("synthetic-source-duty")
            .unwrap()
            .dependencies
            .insert("absent".into());
        e.source_duties = b.source_duties.clone();
    });
    reject("unregistered-open-duty", E::SourceDuty, |b, _, _| {
        b.open_source_duties.insert("absent".into());
    });
    reject("deleted-loss", E::LossSet, |_, p, _| {
        p.losses_removed.clear();
    });
    reject("old-loss-still-DIFF", E::LossStillDiff, |_, _, e| {
        e.losses.insert(
            "synthetic-loss".into(),
            Observation::Diff("synthetic-ordered-trace-DIFF".into()),
        );
    });
    reject("stale-now-SAME-loss", E::StaleNowSame, |_, _, e| {
        e.losses
            .insert("synthetic-unrelated-loss".into(), Observation::Same);
    });
    reject("new-DIFF", E::NewDiff, |_, _, e| {
        e.tuples.insert(
            key("synthetic-value", "Middle"),
            Observation::Diff("new".into()),
        );
    });
    reject("unrelated-regression", E::NewDiff, |_, _, e| {
        e.tuples.insert(
            key("synthetic-regression", "Middle"),
            Observation::Diff("new".into()),
        );
    });
    reject("changed-unrelated-loss", E::ChangedLoss, |_, _, e| {
        e.losses.insert(
            "synthetic-unrelated-loss".into(),
            Observation::Diff("changed".into()),
        );
    });
    reject("changed-unrelated-owner", E::WrongDelta, |_, p, _| {
        p.after
            .owners
            .insert("unapproved-owner".into(), OwnerState::Open);
    });
    reject("missing-crosswalk", E::Crosswalk, |_, p, e| {
        p.crosswalks.clear();
        e.crosswalks.clear();
    });
    reject("missing-certificate", E::Certificate, |_, p, e| {
        p.certificates.clear();
        e.certificates.clear();
    });
    reject("proof-as-SAME", E::Alternative, |_, _, e| {
        e.rows
            .get_mut("synthetic-proof")
            .unwrap()
            .real_tuples
            .push(key("synthetic-proof", "Middle"));
    });
    reject("encoding-key-deletion", E::Encoding, |_, p, _| {
        p.after.encoding.clear();
    });
    reject("encoding-key-activation", E::Encoding, |b, p, _| {
        p.after.executed_encoding = b.encoding.clone();
    });
    reject(
        "blocked-encoding-matrix-promoted",
        E::Encoding,
        |b, _, e| {
            e.tuples
                .insert(b.encoding.first().unwrap().clone(), Observation::Same);
        },
    );
    reject("wrong-before-hash", E::BeforeHash, |_, p, _| {
        p.before_hash = "stale".into();
    });
    reject("wrong-after-hash", E::AfterHash, |_, p, _| {
        p.after_hash = "stale".into();
    });
    reject("wrong-delta", E::WrongDelta, |_, p, _| {
        p.after
            .requirements
            .get_mut("synthetic-proof")
            .unwrap()
            .dependencies
            .clear();
    });
    let (before, proposal, evidence) = fixture();
    let delta = validate_promotion(&before, &proposal, &evidence).unwrap();
    let mut changed = before;
    changed.crosswalks.clear();
    let unchanged = changed.clone();
    assert_eq!(delta.apply(&mut changed), Err(E::BeforeHash));
    assert_eq!(changed, unchanged);
    println!(
        "CM4 PROMOTION NEGATIVE control=stale-at-apply predicate=BeforeHash before_unchanged=true synthetic=true"
    );
    println!("CM4 PROMOTION CONTROLS PASS synthetic=true real_promotions=0");
}

#[test]
#[ignore = "exports only a synthetic transaction declaration, never a live promotion"]
fn cm_export_synthetic_promotion() {
    let (before, proposal, evidence) = fixture();
    validate_promotion(&before, &proposal, &evidence).unwrap();
    println!(
        "CM740_SYNTHETIC_PROMOTION\t{}",
        declaration(&before, &proposal)
    );
}

fn declaration(before: &Snapshot, proposal: &Promotion) -> String {
    format!(
        "synthetic\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        proposal.id,
        proposal.owner,
        before.fingerprint(),
        proposal.after_hash,
        proposal.exact_rows.join(";"),
        proposal
            .tuple_duties
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(";"),
        proposal
            .source_duties
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(";"),
        proposal
            .losses_removed
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(";"),
        proposal
            .crosswalks
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(";"),
        proposal
            .certificates
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(";"),
        receipt_hash(&proposal)
    )
}
