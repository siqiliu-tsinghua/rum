use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::integer_digits;

pub(crate) const INTEGER_DIGITS_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "IntegerLength",
        "Returns the number of digits in an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        integer_digits::eval_integer_length,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGTU, IBASE]),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerDigits",
        "Returns the digits of an exact integer in a given base.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        integer_digits::eval_integer_digits,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        ARGTU,
        IBASE,
        BuiltinMessageSpec {
            tag: "intnn",
            template: "Length `1` is not a non-negative integer in `2`.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "IntegerReverse",
        "Reverses the digits of an exact integer in a given base.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        integer_digits::eval_integer_reverse,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGTU, IBASE]),
    BuiltinSpec::ordinary_with_attributes(
        "FromDigits",
        "Constructs an exact integer from a list or string of digits.",
        AttributeSet::from_attribute(Attribute::Protected),
        integer_digits::eval_from_digits,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGTU]),
    BuiltinSpec::ordinary_with_attributes(
        "DigitCount",
        "Counts digits in the absolute value of an exact integer.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        integer_digits::eval_digit_count,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[ARGTU, IBASE]),
];
