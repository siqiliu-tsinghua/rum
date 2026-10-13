mod aggregate;
mod evaluator;
mod falsifiers;
mod grouping;
mod held;
mod lifecycle;
mod lookup;
mod mapping;
mod schema;
mod support;
use super::super::descriptors::{Fixture, Variant};
use super::primary::Descriptor;
pub fn descriptors() -> Vec<Descriptor> {
    let entries: &[(&str, fn(&str, Variant) -> Fixture)] = &[
        ("ConformSchema.default", schema::fixture),
        ("ConformSchema.predicate", schema::fixture),
        ("SchemaErrors.default", schema::fixture),
        ("SchemaErrors.predicate", schema::fixture),
        ("Evaluator.defaultFallback", evaluator::guard),
        ("AssociateTo.target", held::prepared),
        ("KeyDropFrom.target", held::prepared),
        ("GroupBy.callback", grouping::key),
        ("GroupBy.reducer", grouping::reducer),
        ("KeyMap.callback", mapping::fixture),
        ("Association.keyValue", mapping::fixture),
        ("KeyValueMap.callback", mapping::fixture),
        ("KeyGroupBy.callback", mapping::fixture),
        ("KeySelect.predicate", mapping::fixture),
        ("KeySortBy.keyFunction", mapping::fixture),
        ("KeySort.comparator", mapping::comparator),
        ("AssociationMap.callback", mapping::fixture),
        ("CountsBy.keyFunction", mapping::fixture),
        ("KeyUnion.missingValue", mapping::fixture),
        ("Normal.lazyConsumer", mapping::fixture),
        ("Counts.delayedValue", aggregate::fixture),
        ("Catenate.delayedValue", aggregate::fixture),
        ("Values.delayedValue", aggregate::fixture),
        ("Lookup.delayedValue", aggregate::fixture),
        ("Merge.delayedValue", aggregate::fixture),
        ("Merge.reducer", aggregate::fixture),
        ("MergeNested.delayedValue", aggregate::nested),
        ("MergeNested.reducer", aggregate::nested),
        ("Lookup.heldTarget", lookup::fixture),
        ("Lookup.key", lookup::fixture),
        ("Lookup.default", lookup::fixture),
        ("Lookup.wrapper", lookup::fixture),
    ];
    let mut rows: Vec<_> = entries
        .iter()
        .map(|&(entry, factory)| Descriptor {
            id: entry,
            entry,
            factory,
            variants: if entry.ends_with("predicate") || entry == "Evaluator.defaultFallback" {
                support::GUARD_VARIANTS
            } else {
                support::VARIANTS
            },
            boundary: false,
            lifecycle: lifecycle::exercise,
        })
        .collect();
    for (id, entry, factory) in [
        (
            "Lookup.wrapper.afterDefault",
            "Lookup.wrapper",
            lookup::wrapper_after_default as fn(&str, Variant) -> Fixture,
        ),
        (
            "GroupBy.callback.value",
            "GroupBy.callback",
            grouping::value as fn(&str, Variant) -> Fixture,
        ),
        (
            "GroupBy.callback.cross",
            "GroupBy.callback",
            grouping::cross,
        ),
        (
            "GroupBy.callback.nested",
            "GroupBy.callback",
            grouping::nested,
        ),
        (
            "GroupBy.callback.association",
            "GroupBy.callback",
            grouping::assoc_key,
        ),
        (
            "GroupBy.callback.associationValue",
            "GroupBy.callback",
            grouping::assoc_value,
        ),
        (
            "GroupBy.callback.associationCross",
            "GroupBy.callback",
            grouping::assoc_cross,
        ),
        (
            "GroupBy.callback.associationNested",
            "GroupBy.callback",
            grouping::assoc_nested,
        ),
        (
            "GroupBy.reducer.nested",
            "GroupBy.reducer",
            grouping::nested_reducer,
        ),
        (
            "GroupBy.reducer.leaf",
            "GroupBy.reducer",
            grouping::leaf_reducer,
        ),
        (
            "GroupBy.reducer.association",
            "GroupBy.reducer",
            grouping::assoc_reducer,
        ),
        (
            "GroupBy.reducer.associationNested",
            "GroupBy.reducer",
            grouping::assoc_nested_reducer,
        ),
        (
            "GroupBy.reducer.associationLeaf",
            "GroupBy.reducer",
            grouping::assoc_leaf_reducer,
        ),
        (
            "KeyGroupBy.callback.reducer",
            "KeyGroupBy.callback",
            grouping::key_group_reducer,
        ),
        (
            "KeySortBy.keyFunction.ordering",
            "KeySortBy.keyFunction",
            mapping::sort_by_ordering,
        ),
        (
            "AssociateTo.target.read",
            "AssociateTo.target",
            held::read as fn(&str, Variant) -> Fixture,
        ),
        ("AssociateTo.target.part", "AssociateTo.target", held::part),
        ("KeyDropFrom.target.read", "KeyDropFrom.target", held::read),
        ("KeyDropFrom.target.part", "KeyDropFrom.target", held::part),
        (
            "Evaluator.defaultFallback.cross",
            "Evaluator.defaultFallback",
            evaluator::cross,
        ),
        (
            "Evaluator.defaultFallback.body",
            "Evaluator.defaultFallback",
            evaluator::body,
        ),
        (
            "AssociationMap.callback.association",
            "AssociationMap.callback",
            mapping::map_association as fn(&str, Variant) -> Fixture,
        ),
        (
            "Catenate.delayedValue.list",
            "Catenate.delayedValue",
            aggregate::catenate_list,
        ),
        (
            "MergeNested.delayedValue.deep",
            "MergeNested.delayedValue",
            aggregate::nested_deep,
        ),
        (
            "MergeNested.reducer.deep",
            "MergeNested.reducer",
            aggregate::nested_deep,
        ),
    ] {
        rows.push(Descriptor {
            id,
            entry,
            factory,
            variants: if entry == "Evaluator.defaultFallback" {
                support::GUARD_VARIANTS
            } else {
                support::VARIANTS
            },
            boundary: false,
            lifecycle: lifecycle::exercise,
        });
    }
    rows
}
