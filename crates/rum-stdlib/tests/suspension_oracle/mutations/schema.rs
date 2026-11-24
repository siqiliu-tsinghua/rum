//! Committed requirements are declarations, never executed mutation receipts.
use crate::oracle::obligations::{bindings, content_digest};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct MutationSpec {
    pub row: &'static str,
    pub source_key: &'static str,
    pub tail_digest: &'static str,
    pub carrier: &'static str,
    pub phases: Vec<&'static str>,
    pub selector: &'static str,
    pub observer: &'static str,
    pub source_schema: &'static str,
}

pub(super) fn specs() -> BTreeMap<&'static str, MutationSpec> {
    let mut lines = include_str!("../inventory/obligations/mutants.tsv").lines();
    assert_eq!(
        lines.next(),
        Some("row\tsource_key\ttail_digest\tcarrier\tphases\tselector\tobserver\tsource_schema")
    );
    let schema = content_digest(include_bytes!("source_auth/overlays.tsv"));
    let mut rows = BTreeMap::new();
    for line in lines {
        let c: Vec<_> = line.split('\t').collect();
        assert_eq!(c.len(), 8);
        assert_eq!(c[0], c[5]);
        assert_eq!(c[7], schema);
        for digest in [c[1], c[2], c[7]] {
            assert!(digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()));
        }
        assert!(matches!(
            c[6],
            "Strict" | "NativeEqual" | "StrictNativeEqual"
        ));
        let spec = MutationSpec {
            row: c[0],
            source_key: c[1],
            tail_digest: c[2],
            carrier: c[3],
            phases: c[4].split(';').collect(),
            selector: c[5],
            observer: c[6],
            source_schema: c[7],
        };
        assert!(!spec.carrier.is_empty() && spec.phases.iter().all(|p| !p.is_empty()));
        assert!(rows.insert(c[0], spec).is_none());
    }
    assert_eq!(rows.len(), 151);
    rows
}

pub(super) fn unhex(text: &str) -> String {
    assert!(text.len() % 2 == 0 && text.is_ascii());
    String::from_utf8(
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect(),
    )
    .unwrap()
}

#[derive(Debug)]
pub(super) struct Case {
    pub kind: &'static str,
    pub row: &'static str,
    pub tuple: String,
    pub program: &'static str,
    pub bar: &'static str,
    pub configurations: &'static str,
}

pub(super) fn cases() -> Vec<Case> {
    let mut lines = include_str!("../inventory/obligations/mutant-coverage.tsv").lines();
    assert_eq!(
        lines.next(),
        Some("kind\trow\ttuple\tprogram\tbar\tconfigurations")
    );
    let mut seen = BTreeSet::new();
    let rows: Vec<_> = lines
        .map(|line| {
            let c: Vec<_> = line.split('\t').collect();
            assert_eq!(c.len(), 6);
            assert!(seen.insert((c[0], c[1], c[2])));
            let tuple = if c[2] == "-" {
                String::new()
            } else {
                unhex(c[2])
            };
            assert!(!tuple.contains("guest_heads."));
            Case {
                kind: c[0],
                row: c[1],
                tuple,
                program: c[3],
                bar: c[4],
                configurations: c[5],
            }
        })
        .collect();
    assert_eq!(rows.len(), 255);
    rows
}

#[test]
fn cm_mutation_requirements_exact() {
    let specs = specs();
    let aliases: BTreeMap<_, _> = include_str!("../inventory/obligations/aliases.tsv")
        .lines()
        .skip(1)
        .map(|line| {
            let c: Vec<_> = line.split('\t').collect();
            (c[0], c[2])
        })
        .collect();
    let sources: BTreeMap<_, _> = include_str!("../inventory/obligations/population.tsv")
        .lines()
        .skip(1)
        .map(|line| {
            let c: Vec<_> = line.split('\t').collect();
            (c[0], c[9])
        })
        .collect();
    for spec in specs.values() {
        assert_eq!(aliases[spec.row], spec.source_key);
        assert_eq!(sources[spec.source_key], spec.tail_digest);
    }
    let required = bindings::ActiveRequiredBag::committed().unwrap();
    let active: BTreeSet<_> = required
        .keys
        .iter()
        .filter(|key| !required.port_blocked.contains_key(*key))
        .collect();
    assert_eq!(active.len(), 9060);
    assert_eq!(
        active.iter().map(|k| k.row()).collect::<BTreeSet<_>>(),
        specs.keys().copied().collect()
    );
    let bindings = bindings::FrozenBindings::committed().unwrap();
    let rows = cases();
    let own: Vec<_> = rows.iter().filter(|c| c.kind == "own").collect();
    assert_eq!(own.len(), 151);
    assert_eq!(
        own.iter().map(|c| c.row).collect::<BTreeSet<_>>(),
        specs.keys().copied().collect()
    );
    for case in own {
        assert_eq!(case.bar, "one-own-catch");
        assert!(active.iter().any(|k| k.line() == case.tuple));
        assert!(
            bindings
                .entries
                .iter()
                .any(|b| b.key.line() == case.tuple && b.program.as_deref() == Some(case.program))
        );
        assert_eq!(
            case.configurations,
            if case.row == "P0543" {
                "p0543-unchecked"
            } else {
                "default;no-default"
            }
        );
    }
    for row in ["P0325", "P0326"] {
        let pair: Vec<_> = rows
            .iter()
            .filter(|c| c.kind == "pair" && c.row == row)
            .collect();
        assert_eq!(pair.len(), 50);
        assert!(pair.iter().all(|c| c.bar == "every-cell-own-catch"));
        assert_eq!(
            pair.iter()
                .filter(|c| c.tuple.split('\t').nth(1) == Some("Error"))
                .count(),
            10
        );
    }
    assert_eq!(
        rows.iter()
            .filter(|c| c.kind == "structural")
            .map(|c| c.row)
            .collect::<BTreeSet<_>>(),
        ["Some", "G15A", "G15B", "C1"].into()
    );
    println!(
        "CM4 COMMITTED REQUIREMENTS PASS own_rows=151 real_keys=9060 pair_cells=100 structural_controls=4 cases_per_feature=255 logical_cases=510 declarations_are_receipts=false"
    );
}
