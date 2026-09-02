use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec, HoldAttribute};

use super::message_templates::*;

use crate::logic;

pub(crate) const LOGIC_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "And",
        "Evaluates a conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_and,
        MemoryGrowthContract::Delegated,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Or",
        "Evaluates a disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_or,
        MemoryGrowthContract::Delegated,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Nand",
        "Evaluates a negated conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_nand,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Nor",
        "Evaluates a negated disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        logic::eval_nor,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Xor",
        "Evaluates exclusive-or over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty()
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        logic::eval_xor,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Equivalent",
        "Evaluates logical equivalence over exact boolean arguments and reduces duplicate symbolic terms in the current subset.",
        AttributeSet::empty().with(Attribute::Orderless),
        logic::eval_equivalent,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "Implies",
        "Evaluates binary logical implication over exact boolean arguments in the current subset.",
        logic::eval_implies,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Not",
        "Negates a boolean value in the current exact boolean subset.",
        logic::eval_not,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Equal",
        "Tests equality in the current exact subset for numbers, strings, and obviously identical expressions.",
        logic::eval_equal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "Unequal",
        "Tests inequality in the current exact subset for numbers, strings, and obviously identical expressions.",
        logic::eval_unequal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "Less",
        "Tests strict increasing order in the current exact comparable subset.",
        logic::eval_less,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "Greater",
        "Tests strict decreasing order in the current exact comparable subset.",
        logic::eval_greater,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "LessEqual",
        "Tests nondecreasing order in the current exact comparable subset.",
        logic::eval_less_equal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "GreaterEqual",
        "Tests nonincreasing order in the current exact comparable subset.",
        logic::eval_greater_equal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "Inequality",
        "Evaluates alternating operand/operator comparison chains without short-circuiting; each operand evaluates once, binary chains degenerate to their comparison head, and undetermined multi-segment chains remain symbolic.",
        logic::eval_inequality,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "LessThan",
        "Represents a unary comparison operator equivalent to Less[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "GreaterThan",
        "Represents a unary comparison operator equivalent to Greater[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "LessEqualThan",
        "Represents a unary comparison operator equivalent to LessEqual[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "GreaterEqualThan",
        "Represents a unary comparison operator equivalent to GreaterEqual[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "EqualTo",
        "Represents a unary comparison operator equivalent to Equal[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "UnequalTo",
        "Represents a unary comparison operator equivalent to Unequal[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "SameAs",
        "Represents a unary comparison operator equivalent to SameQ[#, bound] when applied.",
        logic::eval_comparison_operator_wrapper,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
];
