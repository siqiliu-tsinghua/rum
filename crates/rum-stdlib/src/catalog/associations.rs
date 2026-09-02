use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{BuiltinMessageSpec, BuiltinSpec, HoldAttribute};

use super::message_templates::*;

use crate::associations;

pub(crate) const ASSOCIATION_CORE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Association",
        "Builds an association from Rule/RuleDelayed entries in the current first-pass subset.",
        associations::eval_association,
        MemoryGrowthContract::Incremental,
    ).protected(),
    BuiltinSpec::ordinary(
        "AssociationQ",
        "Tests whether an expression is an Association with rule entries.",
        associations::eval_association_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "AssociationQ called with `1` arguments; 1 argument is expected.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "AssociationKeyDeflatten",
        "Builds nested associations from flat associations whose keys are path lists.",
        associations::eval_association_key_deflatten,
        MemoryGrowthContract::Incremental,
    ).protected(),
    BuiltinSpec::ordinary(
        "AssociationKeyFlatten",
        "Flattens nested associations into associations whose keys are path lists.",
        associations::eval_association_key_flatten,
        MemoryGrowthContract::Incremental,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Lookup",
        "Looks up a key in an association in the current first-pass subset.",
        HoldAttribute::HoldAll,
        associations::eval_lookup,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argb",
            template: "Lookup called with `1` arguments; between 2 and 4 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Key",
        "Represents a keyed selector in the current first-pass association subset.",
        associations::eval_key,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "Key called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Keys",
        "Returns association keys in structural order.",
        associations::eval_keys,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Keys called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Values",
        "Returns association values in structural order.",
        associations::eval_values,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Values called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyExistsQ",
        "Tests whether an association contains a given key in the current first-pass subset.",
        associations::eval_key_exists_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyExistsQ called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyTake",
        "Keeps selected keys from an association in the current first-pass subset.",
        associations::eval_key_take,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyTake called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
        BuiltinMessageSpec {
            tag: "invru",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyDrop",
        "Drops selected keys from an association in the current first-pass subset.",
        associations::eval_key_drop,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyDrop called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
        BuiltinMessageSpec {
            tag: "invru",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyMap",
        "Maps a function over association keys in the current first-pass subset.",
        associations::eval_key_map,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyMap called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyValueMap",
        "Maps a binary function over association key-value pairs in the current first-pass subset.",
        associations::eval_key_value_map,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyValueMap called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeySelect",
        "Selects association entries whose keys satisfy a criterion in the current first-pass subset.",
        associations::eval_key_select,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeySelect called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invru",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyMemberQ",
        "Tests whether an association contains a key in the current first-pass subset.",
        associations::eval_key_member_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argrx",
            template: "KeyMemberQ called with `1` arguments; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invas",
            template: "The argument `1` is not a valid Association or rule.",
        },
        BuiltinMessageSpec {
            tag: "invrk",
            template: "The argument `1` is not a valid rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyFreeQ",
        "Tests whether an association does not contain a key in the current first-pass subset.",
        associations::eval_key_free_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argrx",
            template: "KeyFreeQ called with `1` arguments; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invas",
            template: "The argument `1` is not a valid Association or rule.",
        },
        BuiltinMessageSpec {
            tag: "invrk",
            template: "The argument `1` is not a valid rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyGroupBy",
        "Groups association entries by transformed keys, including the current reducer form.",
        associations::eval_key_group_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeySort",
        "Sorts association entries by key in the current canonical ordering subset.",
        associations::eval_key_sort,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeySort called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeySortBy",
        "Sorts association entries by transformed keys, optionally using a supplied binary ordering predicate.",
        associations::eval_key_sort_by,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeySortBy called with `1` arguments; 1, 2, or 3 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
];
