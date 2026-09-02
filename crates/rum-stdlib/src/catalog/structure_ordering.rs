use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::structure_ops;

const SET_OPERATION_ATTRIBUTES: AttributeSet = AttributeSet::empty()
    .with(Attribute::Flat)
    .with(Attribute::OneIdentity)
    .with(Attribute::Protected)
    .with(Attribute::ReadProtected);

pub(crate) const STRUCTURE_ORDERING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "DeleteDuplicates",
        "Keeps the first occurrence of structurally equal top-level elements, or of elements equivalent under a custom binary test.",
        structure_ops::eval_delete_duplicates,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "DeleteDuplicatesBy",
        "Keeps the first top-level element for each evaluated key in the current unary-function subset.",
        structure_ops::eval_delete_duplicates_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Duplicates",
        "Returns structurally equal top-level List elements that occur two or more times, once each in first-appearance order.",
        structure_ops::eval_duplicates,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Tally",
        "Tallies structurally same top-level elements, or elements equivalent under a custom binary test, with the first element as representative.",
        structure_ops::eval_tally,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "GatherBy",
        "Groups top-level elements by evaluated keys in first-occurrence key order, including hierarchical grouping for a list of functions.",
        structure_ops::eval_gather_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Sort",
        "Sorts top-level elements using the current structural ordering subset.",
        structure_ops::eval_sort,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "SortBy",
        "Sorts top-level elements by an evaluated key or a lexicographic tuple from a list of key functions, optionally ordering keys with a binary predicate and preserving input order for ties.",
        structure_ops::eval_sort_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "BinarySearch",
        "Binary-searches an ascending List under rum's canonical order without validating sortedness, returning a 1-based position or Missing[\"NotFound\"].",
        structure_ops::eval_binary_search,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ReverseSort",
        "Sorts top-level elements using structural ordering or a binary predicate, then reverses the result.",
        structure_ops::eval_reverse_sort,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ReverseSortBy",
        "Sorts top-level elements by an evaluated key, then reverses the result.",
        structure_ops::eval_reverse_sort_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "MaxBy",
        "Returns the first top-level element attaining the maximum evaluated key, or the top n in decreasing key order with input-order ties.",
        structure_ops::eval_max_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "empty",
            template: "MaxBy cannot select from an empty list.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "MaximalBy",
        "Returns all top-level elements attaining the maximum evaluated key, preserving input order, with an operator form.",
        structure_ops::eval_maximal_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "MinBy",
        "Returns the first top-level element attaining the minimum evaluated key, or the bottom n in increasing key order with input-order ties.",
        structure_ops::eval_min_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "empty",
            template: "MinBy cannot select from an empty list.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "MinimalBy",
        "Returns all top-level elements attaining the minimum evaluated key, preserving input order, with an operator form.",
        structure_ops::eval_minimal_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TakeLargestBy",
        "Takes top-level elements with largest evaluated keys in descending key order, preserving tie order.",
        structure_ops::eval_take_largest_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TakeSmallestBy",
        "Takes top-level elements with smallest evaluated keys in ascending key order, preserving tie order.",
        structure_ops::eval_take_smallest_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Riffle",
        "Inserts one separator between top-level elements of a normal expression, with optional interval or position specs.",
        structure_ops::eval_riffle,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
        BuiltinMessageSpec {
            tag: "seqs",
            template: "Position 3 of `1` must be a positive machine-sized integer or {imin, imax, di} with nonzero integer step.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "SelectPositions",
        "Returns one-element 1-based position paths for top-level List elements whose criterion evaluates to True, matching Position's path convention, with an optional first-n limit.",
        structure_ops::eval_select_positions,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer selection limit",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Union",
        "Returns sorted unique elements across same-head normal expressions; SameTest keeps the first representative in structural scan order.",
        SET_OPERATION_ATTRIBUTES,
        structure_ops::eval_union,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "heads",
            template: "Heads `1` and `2` at positions `3` and 1 are expected to be the same.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Intersection",
        "Returns sorted unique elements common to all same-head normal expressions; SameTest keeps the first representative in structural scan order.",
        SET_OPERATION_ATTRIBUTES,
        structure_ops::eval_intersection,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argm",
            template: "Intersection called with 0 arguments; 1 or more arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "heads",
            template: "Heads `1` and `2` at positions `3` and 1 are expected to be the same.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary(
        "Complement",
        "Returns sorted unique elements from the first expression that are absent from later same-head expressions; SameTest keeps the first representative in structural scan order.",
        structure_ops::eval_complement,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argm",
            template: "Complement called with 0 arguments; 1 or more arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "heads",
            template: "Heads `1` and `2` at positions `3` and 1 are expected to be the same.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Append",
        "Appends an element to the end of a normal expression, with current operator-form support.",
        structure_ops::eval_append,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Prepend",
        "Prepends an element to the front of a normal expression, with current operator-form support.",
        structure_ops::eval_prepend,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Symbol",
        "Resolves a symbol from a string name using the current context path.",
        structure_ops::eval_symbol,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "string",
            template: "String expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Construct",
        "Constructs an expression from a head and zero or more arguments.",
        structure_ops::eval_construct,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
];
