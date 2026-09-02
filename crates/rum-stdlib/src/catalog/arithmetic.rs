use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::arithmetic;

pub(crate) const ARITHMETIC_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Identity",
        "Returns its argument unchanged.",
        arithmetic::eval_identity,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Plus",
        "Adds exact numbers and canonically combines additive expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        arithmetic::eval_plus,
        MemoryGrowthContract::Preflight,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Times",
        "Multiplies exact numbers and canonically combines multiplicative expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        arithmetic::eval_times,
        MemoryGrowthContract::Preflight,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Power",
        "Raises an exact number to an integer power when possible.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_power,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Subtract",
        "Subtracts one exact number from another.",
        arithmetic::eval_subtract,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Divide",
        "Divides exact numbers, producing exact rationals when possible.",
        arithmetic::eval_divide,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Abs",
        "Returns the absolute value of exact integers and rationals.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_abs,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Sign",
        "Returns the sign of exact integers and rationals.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_sign,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Floor",
        "Returns the greatest integer not greater than an exact number.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_floor,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Ceiling",
        "Returns the least integer not less than an exact number.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_ceiling,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerPart",
        "Returns the integer part of an exact number by truncating toward zero.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_integer_part,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FractionalPart",
        "Returns the exact fractional part after truncating toward zero.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_fractional_part,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Minus",
        "Negates one exact or symbolic argument.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_minus,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Round",
        "Rounds exact numbers to the nearest integer or exact multiple in the current snapshot-helper subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_round,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Factorial",
        "Computes factorials for non-negative exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_factorial,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Factorial2",
        "Computes double factorials for non-negative exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_factorial2,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Total",
        "Totals list elements through the current exact Plus semantics.",
        arithmetic::eval_total,
        MemoryGrowthContract::Preflight,
    ).protected(),
    BuiltinSpec::ordinary(
        "Mean",
        "Aggregates a non-empty list as Total[list] divided by Length[list] through exact arithmetic.",
        arithmetic::eval_mean,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "empty",
            template: "Mean cannot aggregate an empty list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Median",
        "Returns the middle element of a canonically sorted non-empty list, averaging the two middle values exactly for even lengths.",
        arithmetic::eval_median,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "empty",
            template: "Median cannot aggregate an empty list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Commonest",
        "Returns the most frequent elements of a list in first-occurrence order, including all ties.",
        arithmetic::eval_commonest,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Quantile",
        "Returns the nearest-rank quantile of a non-empty list using Ceiling[q * n] clamped to 1..n, with q exact in [0, 1].",
        arithmetic::eval_quantile,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "empty",
            template: "Quantile cannot aggregate an empty list.",
        },
        BuiltinMessageSpec {
            tag: "range",
            template: "Quantile rank parameter must be an exact number between 0 and 1.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Denominator",
        "Returns the denominator of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_denominator,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Numerator",
        "Returns the numerator of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_numerator,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Mod",
        "Returns the modulus of one integer with respect to another.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_mod,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Quotient",
        "Returns the floor-division quotient of two integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_quotient,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "PowerMod",
        "Computes modular exponentiation for exact integers.",
        AttributeSet::from_attribute(Attribute::Listable),
        arithmetic::eval_power_mod,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "ninv",
            template: "`1` is not invertible modulo `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "GCD",
        "Computes the greatest common divisor of exact integers.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        arithmetic::eval_gcd,
        MemoryGrowthContract::Preflight,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "LCM",
        "Computes the least common multiple of exact integers.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        arithmetic::eval_lcm,
        MemoryGrowthContract::Preflight,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Max",
        "Returns the maximum exact integer or rational operand.",
        AttributeSet::empty()
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_max,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Min",
        "Returns the minimum exact integer or rational operand.",
        AttributeSet::empty()
            .with(Attribute::NumericFunction)
            .with(Attribute::Protected),
        arithmetic::eval_min,
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "MinMax",
        "Returns the exact minimum and maximum for one scalar or list argument.",
        AttributeSet::from_attribute(Attribute::Protected),
        arithmetic::eval_min_max,
        MemoryGrowthContract::NonAmplifying,
    ),
];
