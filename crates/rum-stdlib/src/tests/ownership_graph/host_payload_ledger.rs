//! Host payload dispositions, independent of the discovery inventories.
//!
//! An Unbounded witness proves an existential ownership path through this
//! particular opaque slot and a concrete implementation to expression debt.
//! It does not certify every implementation, its other fields, or external code.
//! PublicInput identifies a signature site, not a retained storage edge; it is
//! not an interprocedural proof that the callee never forwards/stores its input.

use std::collections::{BTreeMap, BTreeSet};

use super::model::{Discovery, Unknown, UnknownKind};
use super::{graph, source};

mod adapters;
mod ownership;
mod tests;

const LEDGER: &str = include_str!("inventory/host_payload_ledger.tsv");
const HEADER: &str = "kind\tfile\tline\tcolumn\tconstruct\tdisposition\twitness\towed_by\texternal_implementation_coverage";
const OWED_BY: &str = "TODO.md#standing-engineering-debt";
const PUBLIC_INPUT: &str = "public by-value input; owning storage sites are separate rows";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Disposition {
    Declared,
    Unbounded,
    PublicInput,
}

#[derive(Clone)]
struct Entry {
    row: String,
    disposition: Disposition,
    witness: String,
    owed_by: String,
    external_implementation_coverage: String,
}

fn entries(text: &str) -> Result<Vec<Entry>, String> {
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Err("invalid host payload ledger header".into());
    }
    lines
        .enumerate()
        .map(|(index, line)| {
            let columns: Vec<_> = line.split('\t').collect();
            if columns.len() != 9
                || columns[0] != "UnknownHostPayload"
                || columns[1].is_empty()
                || columns[2].parse::<usize>().ok().is_none_or(|n| n == 0)
                || columns[3].parse::<usize>().ok().is_none_or(|n| n == 0)
                || columns[4].is_empty()
                || columns[6].is_empty()
            {
                return Err(format!(
                    "invalid host payload ledger line {}: {line}",
                    index + 2
                ));
            }
            let disposition = match columns[5] {
                "Declared" => Disposition::Declared,
                "Unbounded" => Disposition::Unbounded,
                "PublicInput" => Disposition::PublicInput,
                other => return Err(format!("unknown disposition {other}: {line}")),
            };
            Ok(Entry {
                row: columns[..5].join("\t"),
                disposition,
                witness: columns[6].into(),
                owed_by: columns[7].into(),
                external_implementation_coverage: columns[8].into(),
            })
        })
        .collect()
}

fn host_rows(discovered: &Discovery) -> BTreeMap<String, &Unknown> {
    discovered
        .unknowns
        .iter()
        .filter(|row| row.kind == UnknownKind::UnknownHostPayload)
        .map(|row| (row.row(), row))
        .collect()
}

fn check(discovered: &Discovery, entries: &[Entry]) -> Result<(), String> {
    let rows = host_rows(discovered);
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for entry in entries {
        if !seen.insert(entry.row.as_str()) {
            errors.push(format!("duplicate ledger entry: {}", entry.row));
        }
        let Some(row) = rows.get(&entry.row) else {
            errors.push(format!("check 2: stale ledger entry: {}", entry.row));
            continue;
        };
        if let Err(error) = check_entry(discovered, row, entry) {
            errors.push(format!("{error}\nrow: {}", entry.row));
        }
    }
    for row in rows.keys() {
        if !seen.contains(row.as_str()) {
            errors.push(format!("check 1: missing ledger entry: {row}"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn check_entry(discovered: &Discovery, row: &Unknown, entry: &Entry) -> Result<(), String> {
    let public_input = row.construct.starts_with("public input ");
    if public_input != (entry.disposition == Disposition::PublicInput) {
        return Err("disposition disagrees with discovered storage/input site".into());
    }
    if entry.disposition == Disposition::Unbounded {
        if entry.owed_by != OWED_BY || entry.external_implementation_coverage != "unresolved" {
            return Err("Unbounded requires the expression debt citation and external_implementation_coverage: unresolved".into());
        }
    } else if entry.owed_by != "-" || entry.external_implementation_coverage != "-" {
        return Err("only Unbounded carries debt and external implementation coverage".into());
    }
    let witness = match entry.disposition {
        Disposition::PublicInput => PUBLIC_INPUT.into(),
        Disposition::Declared => ownership::declared_witness(discovered, row)?,
        Disposition::Unbounded => ownership::expression_witness(discovered, row)
            .map_err(|error| format!("check 3: {error}"))?,
    };
    if entry.witness != witness {
        return Err(format!(
            "witness mismatch: recorded {}\nverified witness: {witness}",
            entry.witness
        ));
    }
    Ok(())
}

#[test]
fn production_host_payload_ledger_is_complete_and_attributed() {
    let discovered = graph::discover(source::production());
    let entries = entries(LEDGER).unwrap();
    check(&discovered, &entries).unwrap_or_else(|error| panic!("{error}"));

    // Check the citation still identifies the recorded drop debt. Its truth is
    // not re-measured here: no crash/overflow probes belong in this test.
    let debt = include_str!("../../../../../TODO.md")
        .split_once("## Standing engineering debt\n")
        .expect("owed_by section must exist")
        .1
        .split("\n## ")
        .next()
        .unwrap();
    assert!(debt.contains("Deep `Expr` trees") && debt.contains("native stack on drop"));

    let inputs = entries
        .iter()
        .filter(|entry| entry.disposition == Disposition::PublicInput)
        .count();
    let declared = entries
        .iter()
        .filter(|entry| entry.disposition == Disposition::Declared)
        .count();
    let unbounded = entries.len() - inputs - declared;
    println!(
        "HOST PAYLOAD LEDGER: {} rows; {} storage edges ({declared} Declared, {unbounded} Unbounded); {inputs} PublicInput",
        entries.len(),
        declared + unbounded
    );
    println!(
        "checks 1/2/3: complete, no stale entries, field-specific expression attribution verified"
    );
    println!("external_implementation_coverage: unresolved for every Unbounded entry");
}
