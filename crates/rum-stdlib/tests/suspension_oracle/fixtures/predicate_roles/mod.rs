//! Role contracts: generated builtins reference and guide/evaluation.md,
//! including continuation preservation and the ordering callback contract.
mod carried;
mod collections;
mod falsifiers;
mod keys;
mod lifecycle;
mod options;
mod ordering;
mod phases;
mod predicates;
mod structure;
mod support;
use super::super::descriptors::{Fixture, Variant};
use super::primary::Descriptor;

pub const ENTRIES: &[&str] = &[
    "AllTrue.predicate",
    "AnyTrue.predicate",
    "ArrayQ.predicate",
    "ArrayQ.rankPattern",
    "Catch.handler",
    "Complement.defaultOption",
    "Complement.explicitOption",
    "Complement.sameTest",
    "ContainsAll.defaultOption",
    "ContainsAll.explicitOption",
    "ContainsAll.sameTest",
    "ContainsAny.defaultOption",
    "ContainsAny.explicitOption",
    "ContainsAny.sameTest",
    "ContainsExactly.defaultOption",
    "ContainsExactly.explicitOption",
    "ContainsExactly.sameTest",
    "ContainsNone.defaultOption",
    "ContainsNone.explicitOption",
    "ContainsNone.sameTest",
    "ContainsOnly.defaultOption",
    "ContainsOnly.explicitOption",
    "ContainsOnly.sameTest",
    "DeleteDuplicates.sameTest",
    "DeleteDuplicatesBy.keyFunction",
    "Depth.defaultOption",
    "Differences.arithmetic",
    "DisjointQ.defaultOption",
    "DisjointQ.explicitOption",
    "DisjointQ.sameTest",
    "DuplicateFreeQ.defaultOption",
    "DuplicateFreeQ.explicitOption",
    "DuplicateFreeQ.sameTest",
    "Enclose.handler",
    "First.lazyConsumer",
    "GatherBy.keyFunction",
    "If.branch",
    "IntersectingQ.defaultOption",
    "IntersectingQ.explicitOption",
    "IntersectingQ.sameTest",
    "Intersection.sameTest",
    "LexicographicOrder.comparator",
    "MatrixQ.predicate",
    "MaxBy.keyFunction",
    "MaximalBy.keyFunction",
    "MinBy.keyFunction",
    "MinimalBy.keyFunction",
    "NoneTrue.predicate",
    "OrderedQ.comparator",
    "Part.lazyConsumer",
    "ReverseSort.comparator",
    "ReverseSortBy.keyFunction",
    "SelectPositions.predicate",
    "Sort.comparator",
    "SortBy.keyFunction",
    "SubsetQ.defaultOption",
    "SubsetQ.explicitOption",
    "SubsetQ.sameTest",
    "Switch.branch",
    "Take.lazyConsumer",
    "TakeLargestBy.keyFunction",
    "TakeSmallestBy.keyFunction",
    "Tally.sameTest",
    "Union.sameTest",
    "VectorQ.predicate",
    "Which.branch",
];

pub fn descriptors() -> Vec<Descriptor> {
    let mut rows: Vec<_> = ENTRIES
        .iter()
        .map(|&entry| Descriptor {
            id: entry,
            entry,
            variants: variants(entry),
            factory: fixture,
            boundary: false,
            lifecycle: lifecycle::exercise,
        })
        .collect();
    rows.extend(phases::descriptors());
    rows
}
fn variants(entry: &str) -> &'static [Variant] {
    if entry.ends_with(".predicate") || entry.ends_with(".sameTest") {
        support::DECLINE_VARIANTS
    } else {
        support::VARIANTS
    }
}
fn fixture(entry: &str, v: Variant) -> Fixture {
    let (name, role) = entry.split_once('.').unwrap();
    match role {
        "defaultOption" | "explicitOption" => options::fixture(entry, v),
        "predicate" | "rankPattern" => predicates::fixture(entry, v),
        "sameTest" if matches!(name, "Union" | "Intersection" | "Complement") => {
            collections::set_fixture(entry, v, false)
        }
        "sameTest" => collections::fixture(entry, v),
        "keyFunction" => keys::fixture(entry, v),
        "comparator" => ordering::fixture(entry, v),
        "lazyConsumer" | "arithmetic" => structure::fixture(entry, v),
        "branch" | "handler" => carried::fixture(entry, v),
        _ => unreachable!("{entry}"),
    }
}
