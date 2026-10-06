//! Contracts: generated builtin reference and guide/evaluation.md lazy sequences.
mod arithmetic;
mod falsifiers;
mod folds;
mod lazy;
mod lifecycle;
mod mapping;
mod phases;
mod support;
use super::super::descriptors::{Fixture, Variant};
use super::primary::Descriptor;

pub const ENTRIES: &[&str] = &[
    "Accumulate.arithmetic",
    "Array.callback",
    "ArrayFilter.callback",
    "ArrayFilter.defaultOption",
    "ArrayFilter.explicitOption",
    "ArrayReduce.callback",
    "BlockMap.callback",
    "Comap.callback",
    "ComapApply.callback",
    "Dot.arithmetic",
    "FixedPoint.callback",
    "FixedPointList.callback",
    "Fold.callback",
    "FoldList.callback",
    "FoldPair.callback",
    "FoldPair.combiner",
    "FoldWhile.callback",
    "FoldWhile.predicate",
    "Inner.arithmetic",
    "LengthWhile.predicate",
    "Map.callback",
    "MapAll.callback",
    "MapAt.callback",
    "MapIndexed.callback",
    "MapThread.callback",
    "Nest.callback",
    "NestList.callback",
    "NestWhile.callback",
    "NestWhile.predicate",
    "NestWhileList.callback",
    "NestWhileList.predicate",
    "Operate.callback",
    "Outer.callback",
    "Select.predicate",
    "SelectFirst.predicate",
    "SequenceFold.callback",
    "SequenceFoldList.callback",
    "SubsetMap.callback",
    "TakeWhile.predicate",
    "Through.callback",
    "LazyDrop.source",
    "LazyFoldList.callback",
    "LazyFoldList.source",
    "LazyMap.callback",
    "LazyMap.source",
    "LazyNestList.callback",
    "LazySelect.predicate",
    "LazySelect.source",
    "LazyTable.body",
    "LazyTable.bounds",
    "LazyTake.source",
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
    for (id, entry, factory) in [
        (
            "Inner.arithmetic/reduction",
            "Inner.arithmetic",
            (|e, v| arithmetic::tensor(e, v, true)) as fn(&str, Variant) -> Fixture,
        ),
        (
            "Dot.arithmetic/reduction",
            "Dot.arithmetic",
            (|e, v| arithmetic::tensor(e, v, true)) as fn(&str, Variant) -> Fixture,
        ),
        (
            "LazyTable.bounds/upper",
            "LazyTable.bounds",
            (|e, v| lazy::bounds(e, v, 1)) as fn(&str, Variant) -> Fixture,
        ),
        (
            "LazyTable.bounds/step",
            "LazyTable.bounds",
            (|e, v| lazy::bounds(e, v, 2)) as fn(&str, Variant) -> Fixture,
        ),
    ] {
        rows.push(Descriptor {
            id,
            entry,
            variants: variants(entry),
            factory,
            boundary: false,
            lifecycle: lifecycle::exercise,
        });
    }
    rows.extend(phases::descriptors());
    rows
}
fn variants(entry: &str) -> &'static [Variant] {
    if entry.ends_with(".predicate") {
        support::DECLINE_VARIANTS
    } else {
        support::VARIANTS
    }
}
fn fixture(entry: &str, v: Variant) -> Fixture {
    let (name, role) = entry.split_once('.').unwrap();
    if name.starts_with("Lazy") {
        return if role == "bounds" {
            lazy::bounds(entry, v, 0)
        } else {
            lazy::fixture(entry, v)
        };
    }
    match role {
        "defaultOption" | "explicitOption" => arithmetic::option(entry, v),
        "arithmetic" => arithmetic::fixture(entry, v),
        _ if name.starts_with("Fold") || name.starts_with("SequenceFold") => {
            folds::fixture(entry, v)
        }
        _ if name.starts_with("Nest") || name.starts_with("FixedPoint") => {
            folds::iteration(entry, v)
        }
        "predicate" => folds::predicate(entry, v),
        _ if matches!(name, "Operate" | "SubsetMap") => mapping::single(entry, v),
        _ => mapping::fixture(entry, v),
    }
}
