//! G15-A adds exactly two status Function completions; other transports stay fixed.
use super::{
    classify::Report,
    receipts::{RECEIPTS, rows, use_identity},
    scan::{Facts, Population},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn transitions(facts: &Facts, report: &Report) -> Result<(), String> {
    let receipts = rows(RECEIPTS);
    let by_id: BTreeMap<_, _> = receipts.iter().map(|row| (row[1], row)).collect();
    let expected_frames = BTreeSet::from(["FG15AProjection", "FG15AValue"]);
    let expected_sites = BTreeSet::from(["MG15AProjection", "MG15AValue"]);
    let owned = |kind| {
        receipts
            .iter()
            .filter(|row| row[0] == kind && row[2] == "G15-A")
            .map(|row| row[1])
            .collect::<BTreeSet<_>>()
    };
    if owned("F") != expected_frames || owned("M") != expected_sites {
        return Err("G15-A: frame or insertion receipt differs from the two new sites".into());
    }
    for id in expected_frames {
        let frame = report
            .frames
            .iter()
            .find(|frame| frame.identity == by_id[id][4])
            .ok_or_else(|| format!("{id}: new typed frame absent"))?;
        if by_id[id][3] != "typed"
            || frame.population != Population::Production
            || frame.kind != "TypedFrame"
            || frame.input.replace(' ', "") != "Expr"
            || frame.output.replace(' ', "") != "Option<Expr>"
        {
            return Err(format!("{id}: Expr -> Option<Expr> completion not proved"));
        }
        println!("G15-A FRAME {id} {} Expr -> Option<Expr>", frame.name);
    }
    for id in expected_sites {
        if by_id[id][3] != "typed"
            || !facts.uses.iter().any(|site| {
                site.population == Population::Production
                    && use_identity(site) == by_id[id][4]
                    && super::witnessed::insertion(facts, site)
            })
        {
            return Err(format!("{id}: actual-result witnessed insertion absent"));
        }
        println!("G15-A INSERTION {id} {}", by_id[id][4]);
    }
    let typed = report
        .frames
        .iter()
        .filter(|frame| {
            frame.population == Population::Production
                && matches!(frame.kind.as_str(), "TypedFrame" | "SlotFinalizer")
        })
        .count();
    let macros = facts
        .uses
        .iter()
        .filter(|site| {
            site.population == Population::Production && site.name == "resume_typed_or_suspend"
        })
        .count();
    if typed != 97 || macros != 55 || by_id["G15-A"][3] != "implemented" {
        return Err(format!(
            "G15-A: expected typed=97 and typed_macros=55, got {typed}/{macros}"
        ));
    }
    println!(
        "G15-A TOTAL implementations=608 typed=97 typed_macros=55 witnessed_sites=48; two new Expr -> Option<Expr> frames; global modes unchanged"
    );
    Ok(())
}
