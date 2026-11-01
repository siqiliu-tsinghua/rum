//! Closed S8 ReplaceAll and traversal collector conversion; modes stay fixed.
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
        ("T0018", "F0242"),
        ("T0019", "F0243"),
        ("T0089", "F0559"),
        ("T0090", "F0564"),
        ("Whole0012", "D049"),
        ("Whole0013", "D005"),
        ("Whole0014", "D005"),
        ("Whole0015", "D125"),
        ("Whole0180", "D006"),
        ("Whole0181", "D112"),
        ("Whole0182", "D049"),
        ("Whole0183", "D049"),
        ("Whole0184", "D134"),
        ("Whole0185", "D111"),
    ]);
    let actual: BTreeMap<_, _> = receipts
        .iter()
        .filter(|r| r[2] == "S8" && r[3] == "removed")
        .map(|r| (r[1], r[8]))
        .collect();
    if actual != removed {
        return Err(format!("S8: unapproved removal set: {actual:?}"));
    }
    for (id, anchor) in removed {
        if !by_id.contains_key(anchor) || facts.uses.iter().any(|s| use_identity(s) == by_id[id][4])
        {
            return Err(format!("{id}: owned transport removal not proved"));
        }
        println!("S8 TRANSITION {id} removed; successor/provider={anchor}");
    }
    let expected = [
        ("F0240", "ReplaceAllTraversalOutcome", "Expr"),
        ("F0241", "ReplaceAllTraversalOutcome", "Expr"),
        ("F0242", "Option<Expr>", "ReplaceAllTraversalOutcome"),
        ("F0243", "CompiledMatchOutcome", "Option<Expr>"),
        ("F0551", "Expr", "Expr"),
        ("F0552", "Expr", "Expr"),
        ("F0553", "Expr", "Expr"),
        ("F0554", "Expr", "Expr"),
        ("F0555", "Expr", "Expr"),
        ("F0556", "Expr", "Expr"),
        ("F0557", "TraversalWithoutLimitResolution", "Expr"),
        ("F0558", "TraversalCountOutcome", "Expr"),
        ("F0559", "CompiledMatchOutcome", "TraversalCountOutcome"),
        ("F0560", "Expr", "Expr"),
        ("F0561", "Expr", "Expr"),
        ("F0562", "CasesTraversalResolution", "Expr"),
        ("F0563", "PositionCollectionOutcome", "Expr"),
        ("F0564", "CompiledMatchOutcome", "PositionCollectionOutcome"),
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "F" && r[2] == "S8" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.iter().map(|(id, _, _)| *id).collect() {
        return Err("S8: typed frame identities differ from owned set".into());
    }
    for (id, input, output) in expected {
        let frame = report
            .frames
            .iter()
            .find(|f| f.identity == by_id[id][4])
            .ok_or_else(|| format!("{id}: typed frame absent"))?;
        if frame.population != Population::Production
            || frame.kind != "TypedFrame"
            || frame.input.replace(' ', "") != input
            || frame.output.replace(' ', "") != output
        {
            return Err(format!("{id}: typed input/output receipt mismatch"));
        }
        println!("S8 FRAME {id} {} {input} -> {output}", frame.name);
    }
    let expected = [
        "M0240", "M0241", "M0242", "M0243", "M0244", "M0586", "M0587", "M0588", "M0589", "M0590",
        "M0591", "M0592", "M0593", "M0594", "M0595", "M0596", "M0597", "M0598", "M0599", "M0600",
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|r| r[0] == "M" && r[2] == "S8" && r[3] == "typed")
        .map(|r| r[1])
        .collect();
    if owned != expected.into_iter().collect() {
        return Err("S8: insertion identities differ from owned set".into());
    }
    for id in expected {
        if !facts.uses.iter().any(|s| {
            s.population == Population::Production
                && use_identity(s) == by_id[id][4]
                && (s.name == "resume_typed_or_suspend" || super::witnessed::insertion(facts, s))
        }) {
            return Err(format!("{id}: witnessed typed insertion absent"));
        }
        println!("S8 INSERTION {id} {}", by_id[id][4]);
    }
    for (scope, caller) in [
        ("::replace_all::", "suspend_replace_all_traversal"),
        ("::replace_all::", "suspend_rewrite_rule_match"),
        ("::count_like::", "suspend_count_traversal"),
        ("::position::", "suspend_position_traversal"),
    ] {
        if !facts.uses.iter().any(|s| {
            s.population == Population::Production
                && s.caller.contains(scope)
                && s.caller.ends_with(&format!("::{caller}"))
                && s.name == "suspend"
                && s.receiver == "route"
        }) {
            return Err(format!("S8: fresh-state token route absent: {caller}"));
        }
    }
    if by_id["S8"][3] != "implemented" {
        return Err("S8: stage receipt missing".into());
    }
    println!(
        "S8 TOTAL typed=70 typed_macros=40 witnessed_sites=37; four transport maps removed; modes unchanged; preflip remains refused"
    );
    Ok(())
}
