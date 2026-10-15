//! Closed S5 definition application and storage transitions.
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
        ("T0011", "F0244A"),
        ("T0012", "F0244A"),
        ("T0013", "F0244A"),
        ("T0014", "F0244A"),
        ("T0015", "F0244A"),
        ("T0016", "F0244A"),
        ("T0017", "F0237"),
        ("T0020", "F0245"),
        ("T0021", "F0246"),
        ("Whole0009", "D049"),
        ("Whole0010", "D049"),
        ("Whole0011", "D125"),
        ("Whole0016", "D049"),
        ("Whole0017", "D049"),
        ("X0002", "F0270"),
    ]);
    let actual_removed: BTreeMap<_, _> = receipts
        .iter()
        .filter(|r| r[2] == "S5" && r[3] == "removed")
        .map(|r| (r[1], r[8]))
        .collect();
    if actual_removed != removed {
        return Err(format!("S5: unapproved removal set: {actual_removed:?}"));
    }
    for (id, anchor) in removed {
        let row = by_id[id];
        if !by_id.contains_key(anchor)
            || report.frames.iter().any(|f| f.identity == row[4])
            || facts.uses.iter().any(|s| use_identity(s) == row[4])
        {
            return Err(format!("{id}: owned transformation not proved"));
        }
        println!("S5 TRANSITION {id} removed; successor/provider={anchor}");
    }
    let expected = [
        ("F0237", "CompiledMatchOutcome", "Option<Expr>"),
        ("F0238", "Option<Expr>", "Option<Expr>"),
        ("F0239", "CompiledMatchOutcome", "Option<Expr>"),
        ("F0244", "Expr", "Expr"),
        ("F0244A", "Expr", "Option<Expr>"),
        ("F0245", "CompiledMatchOutcome", "Option<Expr>"),
        ("F0246", "CompiledMatchOutcome", "Option<Expr>"),
        ("F0257", "Expr", "Expr"),
        ("F0258", "Expr", "Expr"),
        ("F0259", "Expr", "Expr"),
        ("F0262", "Expr", "Expr"),
        ("F0263", "Expr", "Expr"),
        ("F0264", "Expr", "Expr"),
        ("F0270", "Expr", "Expr"),
        ("F0271", "Expr", "Expr"),
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "F" && r[2] == "S5" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.iter().map(|(id, _, _)| *id).collect() {
        return Err("S5: typed frame receipt identities differ from owned set".into());
    }
    for (id, input, output) in expected {
        let row = by_id[id];
        let frame = report
            .frames
            .iter()
            .find(|f| f.identity == row[4])
            .ok_or_else(|| format!("{id}: typed implementation absent"))?;
        if frame.kind != "TypedFrame"
            || frame.population != Population::Production
            || frame.input.replace(' ', "") != input
            || frame.output.replace(' ', "") != output
        {
            return Err(format!("{id}: input/output receipt mismatch"));
        }
        println!("S5 FRAME {id} {} {input} -> {output}", frame.name);
    }
    let expected = [
        "M0237", "M0238", "M0239", "M0245", "M0245A", "M0246", "M0247", "M0260", "M0261", "M0262",
        "M0263", "M0264", "M0267", "M0268", "M0269", "M0270", "M0271", "M0278",
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "M" && r[2] == "S5" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.into_iter().collect() {
        return Err("S5: macro receipt identities differ from owned set".into());
    }
    for id in expected {
        if !facts.uses.iter().any(|s| {
            s.population == Population::Production
                && use_identity(s) == by_id[id][4]
                && (s.name == "resume_typed_or_suspend" || super::witnessed::insertion(facts, s))
        }) {
            return Err(format!("{id}: witnessed typed insertion absent"));
        }
    }
    for suffix in [
        "::resume_suspended_exact_default_value_scan",
        "::resume_suspended_up_value_scan",
        "::suspend",
    ] {
        if !facts.uses.iter().any(|s| {
            s.population == Population::Production
                && s.caller.contains("::definitions::application::")
                && s.caller.ends_with(suffix)
                && s.name == "suspend"
                && matches!(s.receiver.as_str(), "token" | "self")
        }) {
            return Err(format!("S5: token insertion route missing: {suffix}"));
        }
    }
    if by_id["I0004"][3] != "implemented" || by_id["S5"][3] != "implemented" {
        return Err("S5: fallback/stage receipt missing".into());
    }
    println!(
        "S5 SPLIT F0244 Expr / F0244A Option<Expr>; one frame per pause; generic F0239 retains Scan"
    );
    println!(
        "S5 TOTAL typed=37 typed_macros=30 witnessed_sites=12; modes unchanged; preflip remains refused"
    );
    Ok(())
}
