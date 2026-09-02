use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::number_theory;

pub(crate) const NUMBER_THEORY_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "Divisors",
        "Returns the sorted positive divisors of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_divisors,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "EulerPhi",
        "Computes Euler's totient function for an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_euler_phi,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "MoebiusMu",
        "Computes the Moebius mu function for an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_moebius_mu,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "DivisorSigma",
        "Computes exact divisor power sums for exact integer arguments.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NHoldAll)
            .with(Attribute::Protected),
        number_theory::eval_divisor_sigma,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "Prime",
        "Returns the n-th positive prime for a positive integer n.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_prime,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "intpp",
            template: "Positive integer argument expected in `1`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "PrimePi",
        "Counts primes not greater than an exact integer or rational argument.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        number_theory::eval_prime_pi,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "PrimeNu",
        "Counts distinct prime factors of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        number_theory::eval_prime_nu,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "PrimeOmega",
        "Counts prime factors with multiplicity for an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        number_theory::eval_prime_omega,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGX_EXACT]),
    BuiltinSpec::ordinary_with_attributes(
        "NextPrime",
        "Returns the next or previous prime in rum's signed-prime order.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        number_theory::eval_next_prime,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGTU]),
];
