use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{
    Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec, HoldAttribute,
};

use super::message_templates::*;

use crate::associations;

pub(crate) const ASSOCIATION_TRANSFORM_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "AssociationMap",
        "Builds an association by mapping a function over keys in the current first-pass subset.",
        associations::eval_association_map,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "AssociationMap called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrlf",
            template: "Applying `1` to `2` yields `3`, which is not a valid rule, list of rules or association.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "AssociationThread",
        "Threads rule or key/value inputs into an association in the current first-pass subset.",
        associations::eval_association_thread,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "AssociationThread called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "idim",
            template: "`1` and `2` must have the same length.",
        },
        BuiltinMessageSpec {
            tag: "invrk",
            template: "The argument `1` is not a valid rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyUnion",
        "Builds keyed unions, including the current missing-key fill-function form.",
        associations::eval_key_union,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyUnion called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invar",
            template: "The argument `1` is not a valid list of Associations or rules.",
        },
        BuiltinMessageSpec {
            tag: "invas",
            template: "The argument `1` is not a valid Association or rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyIntersection",
        "Builds a key-intersection association in the current first-pass subset.",
        associations::eval_key_intersection,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "KeyIntersection called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "invar",
            template: "The argument `1` is not a valid list of Associations or rules.",
        },
        BuiltinMessageSpec {
            tag: "invas",
            template: "The argument `1` is not a valid Association or rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyComplement",
        "Builds a key-complement association in the current first-pass subset.",
        associations::eval_key_complement,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "KeyComplement called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "empt",
            template: "Argument `1` should be a nonempty list.",
        },
        BuiltinMessageSpec {
            tag: "invar",
            template: "The argument `1` is not a valid list of Associations or rules.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Normal",
        "Converts an association into a list of rules in the current first-pass subset.",
        associations::eval_normal,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argb",
            template: "Normal called with `1` arguments; between 1 and 3 arguments are expected.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "AssociateTo",
        "Adds or replaces association entries, including held symbol mutation in the current first-pass subset.",
        AttributeSet::from_hold(HoldAttribute::HoldFirst).with(Attribute::SubValuesHoldAll),
        associations::eval_associate_to,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argr",
            template: "AssociateTo called with 1 argument; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "argrx",
            template: "AssociateTo called with `1` arguments; 2 arguments are expected.",
        },
        ARGX_OPERATOR,
        BuiltinMessageSpec {
            tag: "blnoval",
            template: "The symbol `1` at position `2` should have an immediate value defined.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
        BuiltinMessageSpec {
            tag: "invlb",
            template: "The argument `1` is not a list, Rule or Association.",
        },
        BuiltinMessageSpec {
            tag: "keynx",
            template: "Key `1` does not exist in `2`.",
        },
        PARTW,
        RVALUE,
        WRSYM,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "KeyDropFrom",
        "Drops keys from an association, including held symbol mutation in the current first-pass subset.",
        AttributeSet::from_hold(HoldAttribute::HoldFirst).with(Attribute::SubValuesHoldAll),
        associations::eval_key_drop_from,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argr",
            template: "KeyDropFrom called with 1 argument; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "argrx",
            template: "KeyDropFrom called with `1` arguments; 2 arguments are expected.",
        },
        ARGX_OPERATOR,
        BuiltinMessageSpec {
            tag: "blnoval",
            template: "The symbol `1` at position `2` should have an immediate value defined.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
        RVALUE,
        WRSYM,
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyValuePattern",
        "Represents an association key/value pattern specification in the current first-pass subset.",
        associations::eval_key_value_pattern,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "Merge",
        "Merges associations by key using a combiner in the current first-pass subset.",
        associations::eval_merge,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Merge called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "list1",
            template: "The argument `1` is not a valid list of Associations or rules or lists of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "JoinAcross",
        "Joins two lists of associations by shared key values.",
        associations::eval_join_across,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argb",
            template: "JoinAcross called with `1` arguments; 3 or 4 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "lists",
            template: "The argument at position `1` in `2` is not a valid list of Associations.",
        },
        BuiltinMessageSpec {
            tag: "keys",
            template: "The argument `1` is not a valid key specification: a key, a rule of keys, or a non-empty list of these.",
        },
        BuiltinMessageSpec {
            tag: "jtype",
            template: "The argument `1` is not a valid join type; \"Inner\", \"Left\", \"Right\", or \"Outer\" is expected.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "MergeNested",
        "Recursively merges nested associations by key using a combiner at leaves.",
        associations::eval_merge_nested,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Catenate",
        "Concatenates lists and association values into a list in the current first-pass subset.",
        associations::eval_catenate,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "Catenate called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Counts",
        "Counts repeated elements and returns an association in the current first-pass subset.",
        associations::eval_counts,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Counts called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invl",
            template: "The argument `1` is not a list.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "CountsBy",
        "Counts transformed elements and returns an association in the current first-pass subset.",
        associations::eval_counts_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "CountsBy called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "list",
            template: "List expected at position `1` in `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "GroupBy",
        "Groups list elements by transformed keys, including the current reducer form.",
        associations::eval_group_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argb",
            template: "GroupBy called with `1` arguments; between 1 and 3 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "list1",
            template: "The argument `1` is not a valid list of Associations or rules or lists of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "PositionIndex",
        "Builds an association from elements to 1-based position lists in the current first-pass subset.",
        associations::eval_position_index,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "PositionIndex called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Missing",
        "Represents missing-data markers in the current first-pass subset.",
        associations::eval_missing,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "DeleteMissing",
        "Deletes Missing entries from lists, associations, and rule-list data in the current first-pass subset.",
        associations::eval_delete_missing,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "arg2",
            template: "The second argument `1` is expected to be a positive integer or Infinity.",
        },
        BuiltinMessageSpec {
            tag: "arg3",
            template: "The third argument `1` is expected to be a non-negative integer or Infinity.",
        },
        BuiltinMessageSpec {
            tag: "argb",
            template: "DeleteMissing called with `1` arguments; between 1 and 3 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
];
