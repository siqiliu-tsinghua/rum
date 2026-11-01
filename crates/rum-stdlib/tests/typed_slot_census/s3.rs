//! Closed S3 transitions: split consumers, a same-type repeat, and removed wire maps.
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
        ("F0273", "F0272"),
        ("M0280", "F0272"),
        ("M0281", "M0601"),
        ("M0282", "M0602"),
        ("M0282S", "M0603"),
        ("M0283", "M0604"),
        ("M0283S", "M0605"),
        ("M0288", "M0287"),
        ("M0561", "M0606"),
        ("M0562", "M0607"),
        ("T0023", "F0272"),
        ("T0027", "F0277"),
        ("Whole0029", "D009"),
        ("Whole0030", "D125"),
        ("Whole0031", "D125"),
        ("Whole0032", "D049"),
    ]);
    let actual_removed: BTreeMap<_, _> = receipts
        .iter()
        .filter(|r| r[2] == "S3" && r[3] == "removed")
        .map(|r| (r[1], r[8]))
        .collect();
    if actual_removed != removed {
        return Err(format!("S3: unapproved removal set: {actual_removed:?}"));
    }
    for (id, anchor) in removed {
        let row = by_id[id];
        if row[2] != "S3"
            || row[7] != "production"
            || !by_id.contains_key(anchor)
            || report.frames.iter().any(|f| f.identity == row[4])
            || facts.uses.iter().any(|s| use_identity(s) == row[4])
        {
            return Err(format!("{id}: removal/merge not proved"));
        }
        println!("S3 TRANSITION {id} removed; successor/provider={anchor}");
    }

    let expected = [
        ("F0204", "Expr", "Expr"),
        ("F0272", "Expr", "PreparedCallArguments"),
        ("F0274", "Expr", "Expr"),
        ("F0275", "Option<Expr>", "Expr"),
        ("F0276", "Option<Expr>", "Expr"),
        ("F0275P", "PreparedCallArguments", "Expr"),
        ("F0277", "CompiledMatchOutcome", "Expr"),
        ("F0278", "Expr", "Option<Expr>"),
        ("F0279", "Expr", "Expr"),
        ("F0526", "Expr", "Expr"),
        ("F0527", "Expr", "Expr"),
        ("F0570", "", "PreparedCallArguments"),
    ];
    let identities: BTreeSet<_> = expected.iter().map(|(id, _, _)| by_id[id][4]).collect();
    let actual: BTreeSet<_> = report
        .frames
        .iter()
        .filter(|f| {
            f.population == Population::Production
                && matches!(f.kind.as_str(), "TypedFrame" | "SlotFinalizer")
                && identities.contains(f.identity.as_str())
        })
        .map(|f| f.identity.as_str())
        .collect();
    if actual != identities {
        return Err("S3: typed implementation identities differ from owned set".into());
    }
    for (id, input, output) in expected {
        let row = by_id[id];
        let f = report.frames.iter().find(|f| f.identity == row[4]).unwrap();
        if row[2] != "S3"
            || row[3] != "typed"
            || f.input.replace(' ', "") != input
            || f.output.replace(' ', "") != output
            || f.kind
                != if id == "F0570" {
                    "SlotFinalizer"
                } else {
                    "TypedFrame"
                }
        {
            return Err(format!("{id}: typed input/output receipt mismatch"));
        }
        println!("S3 FRAME {id} {} {input} -> {output}", f.name);
    }
    let expected = ["M0205", "M0279", "M0285", "M0286", "M0287", "M0289"];
    let identities: BTreeSet<_> = expected.iter().map(|id| by_id[id][4].to_owned()).collect();
    let actual: BTreeSet<_> = facts
        .uses
        .iter()
        .filter(|s| {
            s.population == Population::Production
                && s.name == "resume_typed_or_suspend"
                && identities.contains(&use_identity(s))
        })
        .map(use_identity)
        .collect();
    if actual != identities
        || expected
            .iter()
            .any(|id| by_id[id][2] != "S3" || by_id[id][3] != "typed")
    {
        return Err("S3: witnessed macro identities differ from owned set".into());
    }
    let expected = [
        "M0601", "M0602", "M0603", "M0604", "M0605", "M0606", "M0607",
    ];
    let owned: BTreeSet<_> = receipts
        .iter()
        .filter(|row| row[0] == "M" && row[2] == "S3" && row[3] == "typed" && row[6] == "witnessed")
        .map(|row| row[1])
        .collect();
    if owned != expected.into_iter().collect() {
        return Err("S3: witnessed cold-helper identities differ from owned set".into());
    }
    for id in expected {
        if !facts.uses.iter().any(|site| {
            site.population == Population::Production
                && use_identity(site) == by_id[id][4]
                && super::witnessed::insertion(facts, site)
        }) {
            return Err(format!("{id}: witnessed typed insertion absent"));
        }
        println!("S3 INSERTION {id} {}", by_id[id][4]);
    }
    // T0023 removes a wrapper, including its unreachable generic dispatch use.
    // Check names in the parsed definition and use inventories, not text counts.
    if facts
        .functions
        .iter()
        .any(|f| f.name == "prepare_arguments_for_call")
        || facts
            .uses
            .iter()
            .any(|s| s.name == "prepare_arguments_for_call")
    {
        return Err("T0023: removed wrapper still has a definition or caller".into());
    }
    for id in ["T0024", "T0025", "T0026"] {
        let row = by_id[id];
        if row[3] != "applied" || !facts.uses.iter().any(|s| use_identity(s) == row[4]) {
            for site in facts.uses.iter().filter(|site| site.name == row[6]) {
                println!("S3 MAP_CANDIDATE {id} {}", use_identity(site));
            }
            return Err(format!("{id}: relocated semantic map missing"));
        }
        println!("S3 TRANSITION {id} applied at {}", row[4]);
    }
    if !facts.uses.iter().any(|s| {
        s.population == Population::Production
            && s.caller
                .ends_with("::resume_suspended_call_argument_preparation")
            && s.name == "suspend"
            && s.receiver == "token"
    }) || by_id["D0001"][3] != "implemented"
        || by_id["S3"][3] != "implemented"
    {
        return Err("D0001: same-type token repeat or S3 receipt missing".into());
    }
    println!(
        "S3 T0023 wrapper/callers absent; D0001 witnessed token live; typed=12 macros=6 witnessed=7"
    );
    Ok(())
}
