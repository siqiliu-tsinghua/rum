use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{
    Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec, Capability, HoldAttribute,
};

use super::message_templates::*;

use crate::random;

const ATTR_PROTECTED: AttributeSet = AttributeSet::from_attribute(Attribute::Protected);

pub(crate) const RANDOM_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "SeedRandom",
        "Reseeds the deterministic session PRNG from an exact integer.",
        ATTR_PROTECTED,
        random::eval_seed_random,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "RandomInteger",
        "Draws exact integers uniformly from the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_integer,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "RandomChoice",
        "Chooses elements from a list using the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_choice,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "empty",
            template: "expected a non-empty list",
        },
        BuiltinMessageSpec {
            tag: "weights",
            template: "Reports invalid RandomChoice weights.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "RandomSample",
        "Samples distinct positions from a list using the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_sample,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "ksamp",
            template: "sample count cannot exceed list length",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "RandomString",
        "Draws a string of characters from an alphabet using the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_string,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "BlockRandom",
        "Evaluates an expression while restoring the deterministic session PRNG state afterward.",
        AttributeSet::from_hold(HoldAttribute::HoldAll).with(Attribute::Protected),
        random::eval_block_random,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "RandomUuid",
        "Returns a reproducible version-4-format UUID string from the deterministic session PRNG.",
        ATTR_PROTECTED,
        random::eval_random_uuid,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "UuidQ",
        "Tests whether a string has version-4 UUID format and variant bits.",
        ATTR_PROTECTED,
        random::eval_uuid_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "RandomBytes",
        "Returns entropy bytes as a ByteArray using the host entropy provider.",
        ATTR_PROTECTED,
        random::eval_random_bytes,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
        ENTROPY,
    ])
    .with_capability(Capability::Random),
    BuiltinSpec::ordinary_with_attributes(
        "ReseedRandom",
        "Reseeds the deterministic session PRNG from host entropy. Requires Capability::Random; denied under sandboxed().",
        ATTR_PROTECTED,
        random::eval_reseed_random,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        ENTROPY,
    ])
    .with_capability(Capability::Random),
    BuiltinSpec::ordinary_with_attributes(
        "RandomPassword",
        "Draws an entropy-backed password string from an alphabet, optionally enforcing policy constraints.",
        ATTR_PROTECTED,
        random::eval_random_password,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected RandomPassword[n, opts...] or RandomPassword[alphabet, n, opts...]",
        },
        ENTROPY,
        BuiltinMessageSpec {
            tag: "password",
            template: "Reports an invalid or unsatisfiable random-password policy.",
        },
    ])
    .with_capability(Capability::Random),
];
