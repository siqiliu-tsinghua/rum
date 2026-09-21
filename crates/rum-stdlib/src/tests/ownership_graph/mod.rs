//! Discovery-only ownership inventory (brief-479).
//!
//! Counts are source sites, not distinct trait names or concrete runtime values.
//! UnknownHostPayload includes owned opaque storage and explicit public by-value
//! opaque inputs; an input and its storage slot are distinct, labelled rows.
//! UnknownConstruct includes unresolved captures, expansions, cfg alternatives
//! and foreign representations. Exact equality checks that none disappear or
//! arrive silently; matching an unknown is not a waiver or proof of safety.
//!
//! Function pointer signatures and references do not store their mentioned
//! types. Strong shared ownership is conditional on LastStrongHolder. Generic
//! parameters have finite owned-port summaries, not guessed concrete values.
//!
//! The TSV files are exact data, grouped by construct class because expansion
//! and capture failures dominate their size. Discovery never reads them and
//! there is no auto-update mode, wildcard, disposition, or exemption. A match
//! certifies the inventory only: unresolved rows remain analysis failures.

mod accumulator_coverage;
mod effects;
mod evidence;
mod fixtures;
mod graph;
mod model;
mod resolve;
mod source;

#[test]
fn discovery_report_without_inventory_input() {
    evidence::report(&graph::discover(source::production()));
}

#[test]
fn production_inventory_is_exact() {
    use model::UnknownKind::{UnknownConstruct, UnknownHostPayload};

    let discovered = graph::discover(source::production());
    let cycles = evidence::lines(include_str!("inventory/self_reaching.tsv"));
    let hosts = evidence::lines(include_str!("inventory/host_payloads.tsv"));
    let mut constructs = Vec::new();
    for text in [
        include_str!("inventory/unknown_attributes.tsv"),
        include_str!("inventory/unknown_captures.tsv"),
        include_str!("inventory/unknown_macros.tsv"),
        include_str!("inventory/unknown_storage.tsv"),
    ] {
        let rows = evidence::lines(text);
        evidence::compare(&rows, &rows)
            .expect("each inventory partition must be sorted and unique");
        constructs.extend(rows);
    }
    evidence::sort_rows(&mut constructs);
    for (kind, actual, expected) in [
        (
            "self-reaching types",
            evidence::cycle_rows(&discovered),
            cycles,
        ),
        (
            "UnknownHostPayload",
            evidence::unknown_rows(&discovered, UnknownHostPayload),
            hosts,
        ),
        (
            "UnknownConstruct",
            evidence::unknown_rows(&discovered, UnknownConstruct),
            constructs,
        ),
    ] {
        evidence::compare(&actual, &expected)
            .unwrap_or_else(|error| panic!("{kind} inventory mismatch:\n{error}"));
        println!("EXACT INVENTORY {kind}: {} rows", actual.len());
    }
}
