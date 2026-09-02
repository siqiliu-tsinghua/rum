use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use super::message_templates::*;

use crate::combinatorics;

pub(crate) const COMBINATORICS_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "Binomial",
        "Computes exact binomial coefficients for integer arguments.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        combinatorics::eval_binomial,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "Fibonacci",
        "Computes exact Fibonacci numbers for integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        combinatorics::eval_fibonacci,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "PartitionsP",
        "Computes the number of integer partitions by Euler's pentagonal recurrence.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_partitions_p,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "PartitionsQ",
        "Computes the number of partitions into distinct parts.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_partitions_q,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "BernoulliB",
        "Computes exact Bernoulli numbers for non-negative integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_bernoulli_b,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "Subfactorial",
        "Computes derangement numbers for non-negative integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        combinatorics::eval_subfactorial,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "StirlingS1",
        "Computes signed Stirling numbers of the first kind.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_stirling_s1,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "StirlingS2",
        "Computes Stirling numbers of the second kind.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        combinatorics::eval_stirling_s2,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "CatalanNumber",
        "Computes exact Catalan numbers for non-negative integer indices.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        combinatorics::eval_catalan_number,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "Multinomial",
        "Computes exact multinomial coefficients for non-negative integer arguments.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Orderless)
            .with(Attribute::Protected),
        combinatorics::eval_multinomial,
        MemoryGrowthContract::Preflight,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Pochhammer",
        "Computes finite rising factorial products for non-negative integer counts.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        combinatorics::eval_pochhammer,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
];
