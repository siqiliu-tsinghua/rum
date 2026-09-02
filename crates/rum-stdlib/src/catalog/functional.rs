use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::functional_ops;

pub(crate) const FUNCTIONAL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "BlockMap",
        "Maps a function over contiguous blocks in the current first-pass subset.",
        functional_ops::eval_block_map,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a positive integer block size as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "SubsetMap",
        "Replaces selected exact parts using a mapped subset result in the current first-pass subset.",
        functional_ops::eval_subset_map,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact part specification or list of exact part specifications as the third argument",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "path",
            template: "Duplicate subset path `1` is not allowed.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Comap",
        "Applies selected parts of a function collection to a common expression in the current first-pass subset.",
        functional_ops::eval_comap,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a standard levelspec or Heads -> True/False as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ComapApply",
        "Applies selected parts of a function collection as heads of a common expression in the current first-pass subset.",
        functional_ops::eval_comap_apply,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a Heads -> True/False option as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Accumulate",
        "Returns running totals in the current first-pass subset.",
        functional_ops::eval_accumulate,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "MapIndexed",
        "Applies a function to parts together with their positions in the current first-pass subset.",
        functional_ops::eval_map_indexed,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a standard levelspec or Heads -> True/False as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Map",
        "Applies a function over selected levels of an expression in the current first-pass subset.",
        functional_ops::eval_map,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a standard levelspec or Heads -> True/False as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "MapAll",
        "Applies a function to every subexpression in the current first-pass subset.",
        functional_ops::eval_map_all,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a Heads -> True/False option as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "MapAt",
        "Applies a function at exact part specifications in the current first-pass subset.",
        functional_ops::eval_map_at,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact part specification or list of exact part specifications as the third argument",
        },
        ARGTU,
        PARTW,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Scan",
        "Evaluates a function across selected levels for side effects in the current first-pass subset.",
        functional_ops::eval_scan,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a standard levelspec or Heads -> True/False as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "MapThread",
        "Threads corresponding elements from multiple expressions in the current first-pass subset.",
        functional_ops::eval_map_thread,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a positive integer level as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "TakeWhile",
        "Takes the longest prefix satisfying a criterion in the current first-pass subset.",
        functional_ops::eval_take_while,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "LengthWhile",
        "Counts the longest prefix satisfying a criterion in the current first-pass subset.",
        functional_ops::eval_length_while,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Inner",
        "Combines matching tensor dimensions in the current first-pass subset.",
        functional_ops::eval_inner,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "dim",
            template: "Tensor contraction dimensions `1` and `2` do not match for `3` and `4`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Dot",
        "Contracts list tensors over the last axis of each left operand and the first axis of each right operand.",
        functional_ops::eval_dot,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "dot",
            template: "Tensor contraction dimensions `1` and `2` do not match for `3` and `4`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Fold",
        "Accumulates over elements in the current first-pass subset.",
        functional_ops::eval_fold,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FoldList",
        "Returns successive accumulator states in the current first-pass subset.",
        functional_ops::eval_fold_list,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FoldPair",
        "Accumulates pair-valued state transitions in the current first-pass subset.",
        functional_ops::eval_fold_pair,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FoldWhile",
        "Accumulates while a test stays True in the current first-pass subset.",
        functional_ops::eval_fold_while,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a positive integer or All test arity, with an optional signed integer tail adjustment",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "SequenceFold",
        "Folds a sequence with a sliding history in the current first-pass subset.",
        functional_ops::eval_sequence_fold,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer arity as the fourth argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "SequenceFoldList",
        "Returns the running history of a sequence fold in the current first-pass subset.",
        functional_ops::eval_sequence_fold_list,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer arity as the fourth argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Nest",
        "Applies a function repeatedly a fixed number of times.",
        functional_ops::eval_nest,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer iteration count",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "NestWhile",
        "Repeatedly applies a function while a test stays True in the current first-pass subset.",
        functional_ops::eval_nest_while,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected All or a positive integer test arity as the fourth argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "NestList",
        "Returns repeated applications of a function in the current first-pass subset.",
        functional_ops::eval_nest_list,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer iteration count",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "NestWhileList",
        "Returns repeated applications while a test stays True in the current first-pass subset.",
        functional_ops::eval_nest_while_list,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected All or a positive integer test arity as the fourth argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FixedPoint",
        "Repeatedly applies a function until the result stops changing in the current first-pass subset.",
        functional_ops::eval_fixed_point,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer iteration limit",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FixedPointList",
        "Returns successive repeated applications until a fixed point in the current first-pass subset.",
        functional_ops::eval_fixed_point_list,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer iteration limit",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Apply",
        "Replaces the head of an expression in the current first-pass subset.",
        functional_ops::eval_apply,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a standard levelspec or Heads -> True/False as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "MapApply",
        "Applies head replacement one level down in the current first-pass subset.",
        functional_ops::eval_map_apply,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a standard levelspec or Heads -> True/False as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "OperatorApplied",
        "Represents a curried operator wrapper in the current first-pass subset.",
        functional_ops::eval_operator_applied,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer, an index list, or k -> index-list as the specification argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "CurryApplied",
        "Represents a left-to-right curried operator wrapper in the current first-pass subset.",
        functional_ops::eval_curry_applied,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer, an index list, or k -> index-list as the specification argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ReverseApplied",
        "Represents a wrapper that reverses current call arguments before application in the current first-pass subset.",
        functional_ops::eval_reverse_applied,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer as the second argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Composition",
        "Represents a left composition of functions in the current first-pass subset.",
        AttributeSet::empty()
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        functional_ops::eval_composition,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "RightComposition",
        "Represents a right composition of functions in the current first-pass subset.",
        AttributeSet::empty()
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        functional_ops::eval_right_composition,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "Through",
        "Threads arguments through a functional head in the current first-pass subset.",
        functional_ops::eval_through,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Operate",
        "Applies an operator to heads in the head chain of an expression in the current first-pass subset.",
        functional_ops::eval_operate,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer as the third argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Array",
        "Constructs a dense list-like array by applying a function to generated indices in the current first-pass subset.",
        functional_ops::eval_array,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-negative integer dimension or a list of non-negative integer dimensions",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ArrayFilter",
        "Applies a function to local list-array neighborhoods in the current first-pass subset.",
        functional_ops::eval_array_filter,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a rectangular List array as the second argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ArrayReduce",
        "Reduces rectangular list arrays across selected dimensions in the current first-pass subset.",
        functional_ops::eval_array_reduce,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a rectangular List array as the second argument",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Outer",
        "Forms outer combinations of same-head expressions in the current dense-array subset.",
        functional_ops::eval_outer,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "heads",
            template: "Heads `1` and `2` at positions `3` and 2 are expected to be the same.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Select",
        "Selects elements satisfying a criterion in the current sequence subset.",
        functional_ops::eval_select,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a sequence expression or a non-negative integer or Infinity selection limit",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "SelectFirst",
        "Returns the first element satisfying a criterion in the current sequence subset.",
        functional_ops::eval_select_first,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
];
