//! Exact inventory comparison and verbatim, reproducible discovery output.

use std::collections::BTreeSet;

use super::model::*;

pub(super) fn compare(actual: &[String], inventory: &[String]) -> Result<(), String> {
    let actual_set: BTreeSet<_> = actual.iter().collect();
    let inventory_set: BTreeSet<_> = inventory.iter().collect();
    let mut errors = Vec::new();
    if actual_set.len() != actual.len() {
        errors.push("duplicate discovered row".into());
    }
    if inventory_set.len() != inventory.len() {
        errors.push("duplicate inventory row".into());
    }
    if !inventory
        .windows(2)
        .all(|rows| row_key(&rows[0]) < row_key(&rows[1]))
    {
        errors.push("inventory is not strictly sorted".into());
    }
    for row in actual_set.difference(&inventory_set) {
        errors.push(format!("missing inventory row: {row}"));
    }
    for row in inventory_set.difference(&actual_set) {
        errors.push(format!("stale inventory row: {row}"));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

pub(super) fn cycle_rows(discovery: &Discovery) -> Vec<String> {
    let mut rows: Vec<_> = discovery.cycles.iter().map(Cycle::row).collect();
    rows.sort();
    rows
}

pub(super) fn unknown_rows(discovery: &Discovery, kind: UnknownKind) -> Vec<String> {
    discovery
        .unknowns
        .iter()
        .filter(|u| u.kind == kind)
        .map(Unknown::row)
        .collect()
}

fn row_key(row: &str) -> (&str, &str, usize, usize, &str) {
    let columns: Vec<_> = row.splitn(5, '\t').collect();
    if row.starts_with("Unknown") {
        (
            columns[0],
            columns[1],
            columns[2].parse().unwrap(),
            columns[3].parse().unwrap(),
            columns[4],
        )
    } else {
        (
            columns[0],
            columns[1],
            columns[2].parse().unwrap(),
            0,
            columns[3],
        )
    }
}

pub(super) fn sort_rows(rows: &mut [String]) {
    rows.sort_by(|a, b| row_key(a).cmp(&row_key(b)));
}

pub(super) fn report(discovery: &Discovery) {
    for kind in [
        UnknownKind::UnknownHostPayload,
        UnknownKind::UnknownConstruct,
    ] {
        let rows = unknown_rows(discovery, kind);
        println!("{kind:?} count: {}", rows.len());
        for row in rows {
            println!("{row}");
        }
    }
    let host_inputs = discovery
        .unknowns
        .iter()
        .filter(|u| {
            u.kind == UnknownKind::UnknownHostPayload && u.construct.starts_with("public input ")
        })
        .count();
    let hosts = discovery
        .unknowns
        .iter()
        .filter(|u| u.kind == UnknownKind::UnknownHostPayload)
        .count();
    println!(
        "UnknownHostPayload breakdown: {} storage edges; {host_inputs} public input sites",
        hosts - host_inputs
    );
    println!("Self-reaching type count: {}", discovery.cycles.len());
    for row in cycle_rows(discovery) {
        println!("{row}");
    }
    let nominal = discovery
        .source
        .definitions
        .values()
        .filter(|d| d.alias.is_none())
        .count();
    println!(
        "Production files: {}; test-only files: {}; nominal definitions: {}; aliases: {}; owning edges: {}; resolved Drop owners: {}",
        discovery.source.files.len(),
        discovery.source.test_files.len(),
        nominal,
        discovery.source.definitions.len() - nominal,
        discovery.edges.values().map(|e| e.len()).sum::<usize>(),
        discovery.drops.len()
    );
}

pub(super) fn lines(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}
