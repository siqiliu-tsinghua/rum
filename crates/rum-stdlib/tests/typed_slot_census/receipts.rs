//! Earned receipt identities and source-stage acceptance.
use super::{
    classify::{self, Report},
    scan::{self, Facts, Population},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const RECEIPTS: &str =
    include_str!("../suspension_oracle/inventory/typed-slot-stages.tsv");
pub(super) const SHAPES: &str =
    include_str!("../suspension_oracle/inventory/typed-slot-shapes.tsv");
pub(super) fn rows(input: &str) -> Vec<Vec<&str>> {
    input
        .lines()
        .skip(1)
        .map(|line| line.split('\t').collect())
        .collect()
}
pub(super) fn receipt_id(kind: &str, identity: &str) -> String {
    static INDEX: std::sync::OnceLock<
        BTreeMap<&'static str, BTreeMap<&'static str, &'static str>>,
    > = std::sync::OnceLock::new();
    INDEX
        .get_or_init(|| {
            let mut index: BTreeMap<_, BTreeMap<_, _>> = BTreeMap::new();
            for row in rows(RECEIPTS) {
                index.entry(row[0]).or_default().insert(row[4], row[1]);
            }
            index
        })
        .get(kind)
        .and_then(|rows| rows.get(identity))
        .map(|id| (*id).to_owned())
        .unwrap_or_else(|| classify::stable(kind, identity))
}
pub(super) fn use_identity(site: &scan::Use) -> String {
    format!(
        "{}#{}{}:{}:{}",
        site.caller, site.kind, site.ordinal, site.name, site.receiver
    )
}

fn exact(kind: &str, actual: BTreeSet<String>) -> Result<(), String> {
    let expected: BTreeSet<_> = rows(RECEIPTS)
        .into_iter()
        .filter(|r| r[0] == kind && r[3] != "removed")
        .map(|r| r[4].to_owned())
        .collect();
    if expected == actual {
        return Ok(());
    }
    Err(format!(
        "{kind} receipt mismatch: expected {}, discovered {}\nmissing: {}\nunrecorded: {}",
        expected.len(),
        actual.len(),
        expected
            .difference(&actual)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n"),
        actual
            .difference(&expected)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    ))
}
pub(super) fn baseline(facts: &Facts, report: &Report) -> Result<(), String> {
    let mut counts = Vec::new();
    for population in [
        Population::Production,
        Population::TestOnly,
        Population::TestSupport,
    ] {
        let cohort: Vec<_> = report
            .frames
            .iter()
            .filter(|f| f.population == population)
            .collect();
        let frames = cohort
            .iter()
            .filter(|f| f.kind == "SuspensionFrame")
            .count();
        let finalizers = cohort
            .iter()
            .filter(|f| f.kind == "PureFrameFinalizer")
            .count();
        let macros: Vec<_> = facts
            .uses
            .iter()
            .filter(|s| s.population == population && s.kind == "macro")
            .collect();
        let result = macros.iter().filter(|s| s.receiver == "result").count();
        let insert = macros.iter().filter(|s| s.receiver == "insert").count();
        println!(
            "S4 CENSUS {} frames={frames} finalizers={finalizers} implementations={} typed={} macros={} result={result} insert={insert}",
            population.label(),
            cohort.len(),
            cohort.len() - frames - finalizers,
            macros.len()
        );
        counts.push((
            frames,
            finalizers,
            cohort.len(),
            macros.len(),
            result,
            insert,
        ));
    }
    println!(
        "S4 PARTITION implementations={} macros={}",
        counts[0].2 + counts[1].2,
        counts[0].3 + counts[1].3
    );
    if counts[0] != (532, 52, 606, 588, 356, 232) || counts[1] != (11, 5, 16, 11, 3, 8) {
        return Err(
            "S4 baseline requires production 606/588 and test-only 16/11; combined 622/599".into(),
        );
    }
    let macros: Vec<_> = facts
        .uses
        .iter()
        .filter(|s| s.population != Population::TestSupport && s.kind == "macro")
        .collect();
    if report.unresolved.iter().any(|row| {
        row.starts_with("implementation-")
            || row.starts_with("frame-type")
            || row.starts_with("macro-parse")
    }) {
        return Err(format!(
            "unresolved frame/macro discovery: {:?}",
            report.unresolved
        ));
    }
    exact(
        "F",
        report
            .frames
            .iter()
            .filter(|f| f.population != Population::TestSupport)
            .map(|f| f.identity.clone())
            .collect(),
    )?;
    exact("M", macros.iter().map(|s| use_identity(s)).collect())?;
    let receipts = rows(RECEIPTS);
    for (kind, count) in [
        ("F", 624), // F0273 merged; F0275P splits the prepared consumer; F0248 has a retained removal receipt.
        ("M", 602), // M0280 and M0288 are retained removals; M0250 has a retained removal receipt.
        // M0282S/M0283S complete the two looping stage consumers.
        ("Chain", 62),
        ("Obligation", 14),
        ("Route", 10),
        ("Transform", 90),
        ("Decoder", 134),
    ] {
        if receipts.iter().filter(|r| r[0] == kind).count() != count {
            return Err(format!("{kind}: expected {count} exact baseline receipts"));
        }
    }
    let by_id: BTreeMap<_, _> = receipts.iter().map(|r| (r[1], r)).collect();
    if by_id.len() != receipts.len() {
        return Err("duplicate receipt identity".into());
    }
    let uses: BTreeMap<_, _> = facts.uses.iter().map(|s| (use_identity(s), s)).collect();
    super::s4::transitions(facts, report)?;
    for row in &receipts {
        // The S4 gate checks the exact closed set of removals and proves each
        // old identity absent. All other source anchors remain mandatory.
        if row[3] == "removed" {
            continue;
        }
        let population = match row[0] {
            "F" => report
                .frames
                .iter()
                .find(|f| f.identity == row[4])
                .map(|f| f.population),
            "M" | "Transform" | "Whole" | "Nested" => uses.get(row[4]).map(|s| s.population),
            _ => continue,
        }
        .ok_or_else(|| format!("{}: source anchor vanished: {}", row[1], row[4]))?;
        if population.label() != row[7] {
            return Err(format!(
                "{}: population changed from {} to {}",
                row[1],
                row[7],
                population.label()
            ));
        }
    }
    for row in receipts
        .iter()
        .filter(|r| matches!(r[0], "Chain" | "Obligation"))
    {
        for anchor in row[8].split(';') {
            let target = by_id
                .get(anchor)
                .ok_or_else(|| format!("{}: missing owned anchor {anchor}", row[1]))?;
            if target[2] != row[2] {
                return Err(format!("{}: anchor {anchor} changed owner", row[1]));
            }
        }
    }
    for (kind, count, prefix_len) in [("Whole", 209, 9), ("Nested", 434, 10)] {
        let parents: BTreeSet<_> = receipts
            .iter()
            .filter(|r| r[0] == kind)
            .map(|r| &r[1][..prefix_len])
            .collect();
        if parents.len() != count {
            return Err(format!(
                "{kind}: expected {count} source-use receipts, found {}",
                parents.len()
            ));
        }
        for row in receipts.iter().filter(|r| r[0] == kind) {
            for decoder in row[8].split(';') {
                if !by_id.get(decoder).is_some_and(|r| r[0] == "Decoder") {
                    return Err(format!("{}: decoder {decoder} is not recorded", row[1]));
                }
            }
        }
    }
    exact(
        "X",
        facts
            .uses
            .iter()
            .filter(|s| {
                s.population == Population::TestSupport
                    && (classify::legacy_name(&s.name)
                        || s.name == "resume_or_suspend"
                        || s.name == "insert_suspension_frame")
            })
            .map(use_identity)
            .collect(),
    )?;
    let registry: BTreeMap<_, _> = report
        .registry
        .iter()
        .filter(|r| r.population == Population::Production)
        .map(|r| (r.slot.as_str(), r.mode.as_str()))
        .collect();
    let expected: BTreeMap<_, _> = rows(SHAPES)
        .into_iter()
        .filter(|r| r[3] == "S1")
        .map(|r| {
            (
                r[1],
                match r[2] {
                    "Legacy" => "LegacyMode",
                    "Value" => "ValueMode",
                    _ => panic!("unexpected S1 mode"),
                },
            )
        })
        .collect();
    if registry != expected || registry.len() != 22 {
        return Err(format!("S1 registry mismatch: {registry:?}"));
    }
    for row in rows(RECEIPTS).into_iter().filter(|r| r[0] == "Decoder") {
        if facts
            .functions
            .iter()
            .filter(|f| f.population != Population::TestSupport && f.identity == row[4])
            .count()
            != 1
        {
            return Err(format!(
                "decoder definition identity {} missing/ambiguous: {}",
                row[1], row[4]
            ));
        }
    }
    let mut legacy = 0;
    for shape in rows(SHAPES).into_iter().filter(|r| r[2] == "Legacy") {
        let decoder = rows(RECEIPTS)
            .into_iter()
            .find(|r| r[1] == shape[5])
            .ok_or_else(|| format!("missing decoder {}", shape[5]))?;
        let resolved = report
            .calls
            .iter()
            .filter(|c| c.targets.iter().any(|t| t == decoder[4]))
            .count();
        let function = facts
            .functions
            .iter()
            .find(|f| f.identity == decoder[4])
            .unwrap();
        let unresolved = report
            .calls
            .iter()
            .filter(|c| c.targets.len() != 1 && c.site.name == function.name)
            .count();
        if resolved + unresolved == 0 {
            return Err(format!(
                "legacy slot {} has no discovered live decoder uses",
                shape[1]
            ));
        }
        println!(
            "LEGACY {} decoder={} resolved_uses={resolved} unresolved_same_name_candidates={unresolved}",
            shape[1], shape[5]
        );
        legacy += 1;
    }
    if legacy != 23 {
        return Err(format!(
            "expected 23 legacy whole-slot types, discovered {legacy}"
        ));
    }
    println!(
        "S4 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes"
    );
    Ok(())
}

pub(super) fn preflip(facts: &Facts, report: &Report, slot: &str) -> Result<(), String> {
    if !rows(RECEIPTS)
        .iter()
        .any(|r| r[0] == "Stage" && r[1] == "S54" && r[3] == "complete")
    {
        return Err("preflip refused: S54 completed receipt is required first".into());
    }
    if report
        .frames
        .iter()
        .filter(|f| f.population != Population::TestSupport)
        .any(|f| matches!(f.kind.as_str(), "SuspensionFrame" | "PureFrameFinalizer"))
        || facts.uses.iter().any(|s| {
            !s.test && (s.name == "resume_or_suspend" || s.name == "insert_suspension_frame")
        })
    {
        return Err(
            "preflip refused: production legacy implementation or insertion remains".into(),
        );
    }
    let production_registry: Vec<_> = report
        .registry
        .iter()
        .filter(|r| r.population == Population::Production)
        .collect();
    let slots: BTreeSet<_> = production_registry
        .iter()
        .map(|r| r.slot.as_str())
        .collect();
    let expected: BTreeSet<_> = rows(SHAPES).into_iter().map(|r| r[1]).collect();
    if slots != expected || production_registry.len() != 67 || !slots.contains(slot) {
        return Err("preflip refused: exact 67-type sealed registry required".into());
    }
    if !report.unresolved.is_empty()
        || report
            .calls
            .iter()
            .any(|c| classify::boundary_name(&c.site.name) && c.targets.len() != 1)
    {
        return Err("preflip incomplete: unresolved slot boundary".into());
    }
    if report
        .closure
        .iter()
        .any(|r| r.contains("\tforbidden\t") || r.contains("\tunresolved-"))
    {
        return Err("preflip refused: wire-provider closure is not proved pure".into());
    }
    let allowed: BTreeSet<_> = report
        .closure
        .iter()
        .filter_map(|r| r.split('\t').nth(1))
        .collect();
    if report
        .calls
        .iter()
        .any(|c| classify::legacy_name(&c.site.name) && !allowed.contains(c.site.caller.as_str()))
    {
        return Err("preflip refused: raw codec call outside pure provider closure".into());
    }
    Ok(())
}
