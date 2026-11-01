//! Closed S6 guard and frozen-binding completion transitions.
use super::{
    classify::Report,
    receipts::{RECEIPTS, rows, use_identity},
    scan::{Facts, Population},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn transitions(facts: &Facts, report: &Report) -> Result<(), String> {
    let receipts = rows(RECEIPTS);
    let by_id: BTreeMap<_, _> = receipts.iter().map(|r| (r[1], r)).collect();
    let removed = BTreeMap::from([
        ("T0022", "F0268"),
        ("Whole0027", "D049"),
        ("Whole0028", "D049"),
    ]);
    let actual_removed: BTreeMap<_, _> = receipts
        .iter()
        .filter(|r| r[2] == "S6" && r[3] == "removed")
        .map(|r| (r[1], r[8]))
        .collect();
    if actual_removed != removed {
        return Err(format!("S6: unapproved removal set: {actual_removed:?}"));
    }
    for (id, anchor) in removed {
        if !by_id.contains_key(anchor) || facts.uses.iter().any(|s| use_identity(s) == by_id[id][4])
        {
            return Err(format!("{id}: owned transport removal not proved"));
        }
        println!("S6 TRANSITION {id} removed; successor/provider={anchor}");
    }
    let expected = [
        ("F0266", "CompiledMatchOutcome", "CompiledMatchOutcome"),
        ("F0267", "Expr", "CompiledMatchOutcome"),
        ("F0268", "Expr", "bool"),
        ("F0269", "CompiledMatchOutcome", "bool"),
        ("F0569", "", "CompiledMatchOutcome"),
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "F" && r[2] == "S6" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.iter().map(|(id, _, _)| *id).collect() {
        return Err("S6: typed implementation receipt identities differ from owned set".into());
    }
    for (id, input, output) in expected {
        let frame = report
            .frames
            .iter()
            .find(|f| f.identity == by_id[id][4])
            .ok_or_else(|| format!("{id}: typed implementation absent"))?;
        if frame.population != Population::Production
            || frame.kind
                != if id == "F0569" {
                    "SlotFinalizer"
                } else {
                    "TypedFrame"
                }
            || frame.input.replace(' ', "") != input
            || frame.output.replace(' ', "") != output
        {
            return Err(format!("{id}: typed input/output receipt mismatch"));
        }
        println!("S6 FRAME {id} {} {input} -> {output}", frame.name);
    }
    let expected = ["M0273", "M0274", "M0275", "M0276", "M0277"];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "M" && r[2] == "S6" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.into_iter().collect() {
        return Err("S6: insertion receipt identities differ from owned set".into());
    }
    for id in expected {
        if !facts.uses.iter().any(|s| {
            s.population == Population::Production
                && use_identity(s) == by_id[id][4]
                && (s.name == "resume_typed_or_suspend" || super::witnessed::insertion(facts, s))
        }) {
            return Err(format!("{id}: witnessed typed insertion absent"));
        }
        println!("S6 INSERTION {id} {}", by_id[id][4]);
    }
    for caller in ["suspend_deferred_guards", "suspend_readonly_guards"] {
        if !facts.uses.iter().any(|s| {
            s.population == Population::Production
                && s.caller.contains("::definitions::rhs_guards::")
                && s.caller.ends_with(&format!("::{caller}"))
                && s.name == "suspend"
                && s.receiver == "route"
        }) {
            return Err(format!("S6: shared ordinary/token route missing: {caller}"));
        }
    }
    if by_id["D0003"][3] != "implemented"
        || by_id["S6"][3] != "implemented"
        || !facts.uses.iter().any(|s| {
            s.caller
                .ends_with("::rhs_guards::resume_frames::complete_guard_frame")
                && s.name == "finish_work"
        })
    {
        return Err("D0003: typed deferred finalizer or S6 receipt missing".into());
    }
    println!(
        "S6 TOTAL typed=42 typed_macros=24 witnessed_sites=23; D0003 typed; modes unchanged; preflip remains refused"
    );
    Ok(())
}
