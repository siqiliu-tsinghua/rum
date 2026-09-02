use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{
    Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec, HoldAttribute,
};

use super::message_templates::*;

use crate::structure_ops;

pub(crate) const STRUCTURE_CORE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "FullForm",
        "Renders the held argument as a canonical structural-form string in the current core expression subset.",
        structure_ops::eval_full_form,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Head",
        "Returns the structural head of an expression, optionally wrapped in a second head.",
        structure_ops::eval_head,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Length",
        "Returns the number of structural elements in an expression.",
        structure_ops::eval_length,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Depth",
        "Returns the structural depth of an expression, optionally including heads.",
        structure_ops::eval_depth,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid level specification or Heads option for Depth.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        OPTTF,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "LeafCount",
        "Counts atomic leaves in an expression's full structural tree, including heads.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_leaf_count,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_hold(
        "First",
        "Returns the first structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        structure_ops::eval_first,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "nofirst",
            template: "`1` has zero length and no `2` element.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Last",
        "Returns the last structural element, with an optional held default.",
        HoldAttribute::HoldRest,
        structure_ops::eval_last,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "nolast",
            template: "`1` has zero length and no `2` element.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Rest",
        "Returns an expression with its first structural element removed.",
        structure_ops::eval_rest,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "norest",
            template: "Cannot take `1` of expression `2` with length zero.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Most",
        "Returns an expression with its last structural element removed.",
        structure_ops::eval_most,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "nomost",
            template: "Cannot take `1` of expression `2` with length zero.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Take",
        "Takes structural elements using the current standard sequence-spec subset, including UpTo bounds.",
        structure_ops::eval_take,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "take",
            template: "Cannot apply the requested sequence specification in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Drop",
        "Drops structural elements using the current standard sequence-spec subset, including UpTo bounds.",
        structure_ops::eval_drop,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "drop",
            template: "Cannot apply the requested sequence specification in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TakeDrop",
        "Returns both the taken and dropped structural elements using the current standard sequence-spec subset, including UpTo bounds.",
        structure_ops::eval_take_drop,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "iseq",
            template: "Invalid sequence specification `1` for an expression of length `2`.",
        },
        BuiltinMessageSpec {
            tag: "take",
            template: "Cannot apply the requested sequence specification in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Insert",
        "Inserts an element at a top-level structural position in the current first-pass subset.",
        structure_ops::eval_insert,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "ins",
            template: "Cannot insert at position `1` in `2`",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Delete",
        "Deletes one or more top-level structural positions in the current first-pass subset.",
        structure_ops::eval_delete,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        PARTW,
    ]).protected(),
    BuiltinSpec::ordinary(
        "TakeList",
        "Splits a normal expression into sequential chunks by non-negative counts.",
        structure_ops::eval_take_list,
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
        "Range",
        "Constructs a plain list of exact integers from the current one-, two-, and three-argument subset.",
        structure_ops::eval_range,
        MemoryGrowthContract::Preflight,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Differences",
        "Returns successive differences for plain lists, including the current non-negative order subset.",
        AttributeSet::empty()
            .with(Attribute::NonThreadable)
            .with(Attribute::Protected),
        structure_ops::eval_differences,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsmn",
            template: "Single or list of non-negative machine-sized integers expected at position 2 of `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ConstantArray",
        "Constructs a constant plain list for non-negative integer scalar and list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_constant_array,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "PadLeft",
        "Pads or truncates a normal expression on the left using scalar lengths or rectangular list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_pad_left,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsm",
            template: "Non-negative machine-sized integer or non-empty list of non-negative machine-sized integers expected at position 2 in `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "PadRight",
        "Pads or truncates a normal expression on the right using scalar lengths or rectangular list dimensions.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_pad_right,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsm",
            template: "Non-negative machine-sized integer or non-empty list of non-negative machine-sized integers expected at position 2 in `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Permutations",
        "Returns distinct permutations of a list in rum's canonical lexicographic (sorted-first, not input-first) order; fixed-length forms are deferred.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_permutations,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "List expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Subsets",
        "Returns subsets of a normal expression in rum's size-major order for scalar, All, Infinity, and range size specs.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_subsets,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "nninfseq",
            template: "Position 2 of `1` must be All, Infinity, nmax, {nmin}, {nmin, nmax} or {nmin, nmax, dn}, where nmin is a non-negative integer, nmax is non-negative integer or Infinity and dn is a nonzero integer.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Tuples",
        "Returns repeated tuples or cartesian products for normal expressions in the current subset.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_tuples,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsmn",
            template: "Single or list of non-negative machine-sized integers expected at position 2 of `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position `1` in `2`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Partition",
        "Partitions a flat normal expression into fixed-size blocks; aligned forms anchor the first element at kL and stop once the last element appears at or beyond kR.",
        AttributeSet::from_attribute(Attribute::Protected),
        structure_ops::eval_partition,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "align",
            template: "Position 4 of `1` must be {kL, kR}, where kL and kR are positive machine-sized integers not greater than the block size `2`.",
        },
        ARGTU,
        ARGX_OPERATOR,
        BuiltinMessageSpec {
            tag: "ilsmp",
            template: "Positive machine-sized integer expected at position `1` of `2`.",
        },
        BuiltinMessageSpec {
            tag: "npart",
            template: "The expression `1` cannot be partitioned.",
        },
    ]),
    BuiltinSpec::ordinary(
        "Flatten",
        "Flattens nested expressions through the current non-negative level, single-head-spec, and level-list subset.",
        structure_ops::eval_flatten,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "fldep",
            template: "Level `1` exceeds the levels, `2`, which can be flattened together in `3`.",
        },
        BuiltinMessageSpec {
            tag: "flev",
            template: "The level argument `1` in position 2 of `2` should be a non-negative integer or Infinity.",
        },
        BuiltinMessageSpec {
            tag: "flpi",
            template: "Levels to be flattened together in `1` should be lists of positive integers.",
        },
        BuiltinMessageSpec {
            tag: "flrep",
            template: "Level `1` should not be repeated.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Join",
        "Concatenates same-head normal expressions and associations in the current top-level subset.",
        structure_ops::eval_join,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "heads",
            template: "Heads `1` and `2` at positions 1 and `3` are expected to be the same.",
        },
        BuiltinMessageSpec {
            tag: "headsd",
            template: "Expression `1` at position `2` is expected to have head `3` for all expressions at level `4`.",
        },
        BuiltinMessageSpec {
            tag: "incpt",
            template: "Incompatible elements in `1` cannot be joined.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal1",
            template: "Expression `1` at position 1 is expected to have nonatomic subexpression at level `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Reverse",
        "Reverses normal-expression arguments at positive levels or at every level with All.",
        structure_ops::eval_reverse,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ilsmp",
            template: "All, a positive machine-sized integer, or a list of positive machine-sized integers expected at position 2 of `1`.",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Transpose",
        "Transposes rectangular plain-list matrices and tensors using identity, first-two-swap, or rank-matching positive permutations.",
        structure_ops::eval_transpose,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "nmtx",
            template: "The first two levels of `1` cannot be transposed.",
        },
        BuiltinMessageSpec {
            tag: "permspec",
            template: "Invalid permutation specification found at position 2 in `1`.",
        },
    ]).protected(),
];
