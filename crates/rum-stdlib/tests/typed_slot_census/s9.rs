//! Closed S9 control and cleanup conversion; modes stay fixed.
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
        ("T0029", "F0341"),
        ("Whole0034", "D122"),
        ("Whole0045", "D049"),
        ("Whole0046", "D049"),
        ("Whole0047", "D049"),
        ("Whole0048", "D049"),
    ]);
    let actual: BTreeMap<_, _> = receipts
        .iter()
        .filter(|r| r[2] == "S9" && r[3] == "removed")
        .map(|r| (r[1], r[8]))
        .collect();
    if actual != removed {
        return Err(format!("S9: unapproved removal set: {actual:?}"));
    }
    for (id, anchor) in removed {
        if !by_id.contains_key(anchor) || facts.uses.iter().any(|s| use_identity(s) == by_id[id][4])
        {
            return Err(format!("{id}: owned transport removal not proved"));
        }
        println!("S9 TRANSITION {id} removed; successor/provider={anchor}");
    }
    let expected = [
        ("F0283", "Expr", "Expr"),
        ("F0284", "Expr", "Expr"),
        ("F0285", "Expr", "Expr"),
        ("F0286", "Expr", "Expr"),
        ("F0287", "Expr", "Expr"),
        ("F0288", "Expr", "Expr"),
        ("F0289", "Expr", "Expr"),
        ("F0290", "Expr", "Expr"),
        ("F0313", "Expr", "Expr"),
        ("F0314", "Expr", "Expr"),
        ("F0315", "Expr", "Expr"),
        ("F0316", "Expr", "Expr"),
        ("F0317", "Expr", "Expr"),
        ("F0333", "Expr", "Expr"),
        ("F0334", "CompiledMatchOutcome", "Expr"),
        ("F0335", "CompiledMatchOutcome", "Expr"),
        ("F0336", "CompiledMatchOutcome", "Expr"),
        ("F0337", "CompiledMatchOutcome", "Expr"),
        ("F0338", "Expr", "Expr"),
        ("F0339", "Expr", "Expr"),
        ("F0340", "Expr", "Expr"),
        ("F0341", "CompiledMatchOutcome", "Expr"),
        ("F0342", "Expr", "Expr"),
        ("F0343", "CompiledMatchOutcome", "Expr"),
        ("F0572", "", "Expr"),
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "F" && r[2] == "S9" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.iter().map(|(id, _, _)| *id).collect() {
        return Err("S9: typed frame identities differ from owned set".into());
    }
    for (id, input, output) in expected {
        let frame = report
            .frames
            .iter()
            .find(|f| f.identity == by_id[id][4])
            .ok_or_else(|| format!("{id}: typed frame absent"))?;
        if frame.population != Population::Production
            || frame.kind
                != if id == "F0572" {
                    "SlotFinalizer"
                } else {
                    "TypedFrame"
                }
            || frame.input.replace(' ', "") != input
            || frame.output.replace(' ', "") != output
        {
            return Err(format!("{id}: typed input/output receipt mismatch"));
        }
        println!("S9 FRAME {id} {} {input} -> {output}", frame.name);
    }
    let expected = [
        "M0293", "M0294", "M0295", "M0296", "M0297", "M0298", "M0299", "M0300", "M0323", "M0324",
        "M0325", "M0326", "M0327", "M0344", "M0345", "M0346", "M0347", "M0348", "M0349", "M0350",
        "M0351", "M0352", "M0353", "M0354",
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "M" && r[2] == "S9" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.into_iter().collect() {
        return Err("S9: insertion identities differ from owned set".into());
    }
    for id in expected {
        if !facts.uses.iter().any(|s| {
            s.population == Population::Production
                && use_identity(s) == by_id[id][4]
                && (s.name == "resume_typed_or_suspend" || super::witnessed::insertion(facts, s))
        }) {
            return Err(format!("{id}: witnessed typed insertion absent"));
        }
        println!("S9 INSERTION {id} {}", by_id[id][4]);
    }
    for (scope, caller) in [
        ("::exceptions::", "suspend_suspended_exception_handlers"),
        (
            "::exceptions::",
            "suspend_suspended_exception_match_outcome",
        ),
    ] {
        if !facts.uses.iter().any(|s| {
            s.population == Population::Production
                && s.caller.contains(scope)
                && s.caller.ends_with(&format!("::{caller}"))
                && s.name == "suspend"
                && s.receiver == "route"
        }) {
            return Err(format!("S9: fresh-state token route absent: {caller}"));
        }
    }
    if by_id["S9"][3] != "implemented" {
        return Err("S9: stage receipt missing".into());
    }
    println!(
        "S9 TOTAL typed=95 typed_macros=55 witnessed_sites=46; one transport map and five identity decodes removed; modes unchanged; preflip remains refused"
    );
    Ok(())
}
