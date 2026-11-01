//! Closed S4 transitions: a same-type argument-resolution repeat and removed wires.
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
        ("F0248", "F0247"),
        ("M0250", "F0247"),
        ("Whole0018", "D122"),
        ("Whole0019", "D122"),
        ("Whole0020", "D128"),
        ("Whole0021", "D128"),
        ("Whole0022", "D126"),
        ("Whole0023", "D126"),
        ("Whole0024", "D126"),
        ("Nested0005", "D122"),
        ("Nested0006", "D122"),
    ]);
    let actual_removed: BTreeMap<_, _> = receipts
        .iter()
        .filter(|r| r[2] == "S4" && r[3] == "removed")
        .map(|r| (r[1], r[8]))
        .collect();
    if actual_removed != removed {
        return Err(format!("S4: unapproved removal set: {actual_removed:?}"));
    }
    for (id, anchor) in removed {
        let row = by_id[id];
        if row[2] != "S4"
            || !by_id.contains_key(anchor)
            || report.frames.iter().any(|f| f.identity == row[4])
            || facts.uses.iter().any(|s| use_identity(s) == row[4])
        {
            return Err(format!("{id}: removal/merge not proved"));
        }
        println!("S4 TRANSITION {id} removed; successor/provider={anchor}");
    }

    let later_frames: BTreeSet<_> = receipts
        .iter()
        .filter(|r| {
            r[0] == "F"
                && matches!(r[2], "S5" | "S6" | "S7" | "S8" | "S9" | "G15-A" | "G15-C1")
                && r[3] == "typed"
        })
        .map(|r| r[4])
        .collect();
    let later_macros: BTreeSet<_> = receipts
        .iter()
        .filter(|r| {
            r[0] == "M"
                && matches!(r[2], "S5" | "S6" | "S7" | "S8" | "S9" | "G15-A" | "G15-C1")
                && r[3] == "typed"
        })
        .map(|r| r[4])
        .collect();
    let expected = [
        ("F0204", "Expr", "Expr"),
        ("F0247", "Expr", "Vec<Expr>"),
        ("F0249", "Expr", "Expr"),
        ("F0250", "Expr", "Option<Vec<Expr>>"),
        ("F0251", "Option<Vec<Expr>>", "Expr"),
        ("F0252", "Option<Vec<Expr>>", "Option<Vec<RewriteRuleInfo>>"),
        ("F0253", "Expr", "Expr"),
        ("F0254", "Vec<Expr>", "CasesTraversalResolution"),
        ("F0255", "Vec<Expr>", "CasesTraversalResolution"),
        ("F0256", "Vec<Expr>", "TraversalWithoutLimitResolution"),
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
        ("F0568", "", "Vec<Expr>"),
        ("F0570", "", "PreparedCallArguments"),
    ];
    let actual: BTreeSet<_> = report
        .frames
        .iter()
        .filter(|f| {
            f.population == Population::Production
                && !later_frames.contains(f.identity.as_str())
                && matches!(f.kind.as_str(), "TypedFrame" | "SlotFinalizer")
        })
        .map(|f| f.identity.as_str())
        .collect();
    let identities: BTreeSet<_> = expected.iter().map(|(id, _, _)| by_id[id][4]).collect();
    if actual != identities {
        return Err("S4: typed implementation identities differ from owned set".into());
    }
    for (id, input, output) in expected {
        let row = by_id[id];
        let f = report.frames.iter().find(|f| f.identity == row[4]).unwrap();
        if row[3] != "typed"
            || f.input.replace(' ', "") != input
            || f.output.replace(' ', "") != output
            || f.kind
                != if matches!(id, "F0568" | "F0570") {
                    "SlotFinalizer"
                } else {
                    "TypedFrame"
                }
        {
            return Err(format!("{id}: typed input/output receipt mismatch"));
        }
        println!("S4 FRAME {id} {} {input} -> {output}", f.name);
    }
    for id in [
        "F0247", "F0249", "F0250", "F0251", "F0252", "F0253", "F0254", "F0255", "F0256", "F0568",
    ] {
        if by_id[id][2] != "S4" {
            return Err(format!("{id}: wrong typing stage"));
        }
    }

    let expected = [
        "M0205", "M0248", "M0249", "M0251", "M0252", "M0253", "M0254", "M0255", "M0256", "M0257",
        "M0258", "M0259", "M0279", "M0285", "M0286", "M0287", "M0289",
    ];
    let identities: BTreeSet<_> = expected.iter().map(|id| by_id[id][4].to_owned()).collect();
    let actual: BTreeSet<_> = facts
        .uses
        .iter()
        .filter(|s| {
            s.population == Population::Production
                && s.name == "resume_typed_or_suspend"
                && !later_macros.contains(use_identity(s).as_str())
        })
        .map(use_identity)
        .collect();
    if actual != identities || expected.iter().any(|id| by_id[id][3] != "typed") {
        return Err("S4: witnessed macro identities differ from owned set".into());
    }
    for id in [
        "M0248", "M0249", "M0251", "M0252", "M0253", "M0254", "M0255", "M0256", "M0257", "M0258",
        "M0259",
    ] {
        if by_id[id][2] != "S4" {
            return Err(format!("{id}: wrong typing stage"));
        }
    }
    if !facts.uses.iter().any(|s| {
        s.population == Population::Production
            && s.caller.ends_with("::resume_suspended_argument_resolution")
            && s.name == "suspend"
            && s.receiver == "token"
    }) || by_id["D0002"][3] != "implemented"
        || by_id["S4"][3] != "implemented"
    {
        return Err("D0002: same-type token repeat or S4 receipt missing".into());
    }
    println!("S4 D0002 witnessed token live; typed=22 macros=24");
    Ok(())
}
