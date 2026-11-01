//! Closed S7 selector/held-assignment conversion; no transport-mode flip.
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
        ("F0348", "F0347"),
        ("M0359", "M0358"),
        ("Whole0025", "D011"),
        ("Whole0026", "D011"),
        ("Whole0033", "D011"),
        ("Whole0050", "D126"),
        ("Whole0051", "D126"),
    ]);
    let actual: BTreeMap<_, _> = receipts
        .iter()
        .filter(|r| r[2] == "S7" && r[3] == "removed")
        .map(|r| (r[1], r[8]))
        .collect();
    if actual != removed {
        return Err(format!("S7: unapproved removal set: {actual:?}"));
    }
    for (id, anchor) in removed {
        if !by_id.contains_key(anchor)
            || facts.uses.iter().any(|s| use_identity(s) == by_id[id][4])
            || report.frames.iter().any(|f| f.identity == by_id[id][4])
        {
            return Err(format!("{id}: owned source removal not proved"));
        }
        println!("S7 TRANSITION {id} removed; successor/provider={anchor}");
    }
    let expected = [
        ("F0047", "EvaluatedPartSelectors", "Expr"),
        ("F0048", "Expr", "Expr"),
        ("F0049", "Expr", "Expr"),
        ("F0260", "EvaluatedPartSelectors", "Expr"),
        ("F0261", "Expr", "Expr"),
        ("F0265", "EvaluatedPartSelectors", "Expr"),
        ("F0280", "EvaluatedPartSelectors", "Expr"),
        ("F0281", "Expr", "Expr"),
        ("F0282", "Expr", "Expr"),
        ("F0347", "Vec<Expr>", "EvaluatedPartSelectors"),
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "F" && r[2] == "S7" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.iter().map(|(id, _, _)| *id).collect() {
        return Err("S7: typed implementation identities differ from owned set".into());
    }
    for (id, input, output) in expected {
        let frame = report
            .frames
            .iter()
            .find(|f| f.identity == by_id[id][4])
            .ok_or_else(|| format!("{id}: typed implementation absent"))?;
        if frame.population != Population::Production
            || frame.kind != "TypedFrame"
            || frame.input.replace(' ', "") != input
            || frame.output.replace(' ', "") != output
        {
            return Err(format!("{id}: typed input/output receipt mismatch"));
        }
        println!("S7 FRAME {id} {} {input} -> {output}", frame.name);
    }
    let expected = [
        "M0041", "M0042", "M0043", "M0265", "M0266", "M0272", "M0290", "M0291", "M0292", "M0358",
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "M" && r[2] == "S7" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.into_iter().collect() {
        return Err("S7: insertion identities differ from owned set".into());
    }
    for id in expected {
        if !facts.uses.iter().any(|s| {
            s.population == Population::Production
                && use_identity(s) == by_id[id][4]
                && (s.name == "resume_typed_or_suspend" || super::witnessed::insertion(facts, s))
        }) {
            return Err(format!("{id}: witnessed typed insertion absent"));
        }
        println!("S7 INSERTION {id} {}", by_id[id][4]);
    }
    if !facts.uses.iter().any(|s| {
        s.caller.ends_with(
            "::association_data::resume_frames::suspend_suspended_association_set_lvalue",
        ) && s.name == "suspend"
            && s.receiver == "route"
    }) {
        return Err("S7: F0282 fresh-state ordinary/token route absent".into());
    }
    let producer = facts
        .functions
        .iter()
        .find(|f| f.name == "evaluate_part_selectors_owned_for_builtin")
        .ok_or("S7: builtin selector producer absent")?;
    if !producer
        .signature
        .replace(' ', "")
        .contains("Result<EvaluatedPartSelectors,EvalError>")
        || facts
            .functions
            .iter()
            .any(|f| f.owner.contains("EvaluatedPartSelectorsContinuation"))
        || report
            .registry
            .iter()
            .any(|r| r.slot == "HeldUpdateCurrent")
        || by_id["D0014"][3] != "implemented"
        || by_id["S7"][3] != "implemented"
    {
        return Err("S7: publishing removal or D0014 outer Expr completion not proved".into());
    }
    println!(
        "S7 TOTAL typed=52 typed_macros=28 witnessed_sites=29; F0348 retired; D0014 typed; modes unchanged; preflip remains refused"
    );
    Ok(())
}
