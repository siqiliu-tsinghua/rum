use rum_runtime::kernel_api::BuiltinSpec;
use rum_runtime::kernel_api::MemoryGrowthContract;

use super::message_templates::*;

use crate::polynomials;

pub(crate) const POLYNOMIAL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Expand",
        "Expands rational-coefficient polynomial expressions over pattern-free indeterminates.",
        polynomials::eval_expand,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Coefficient",
        "Extracts coefficients of exact monomial powers from rational-coefficient polynomial expressions.",
        polynomials::eval_coefficient,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "CoefficientList",
        "Builds coefficient arrays for rational-coefficient polynomial expressions in specified indeterminates.",
        polynomials::eval_coefficient_list,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Together",
        "Combines rational-function terms over a common polynomial denominator.",
        polynomials::eval_together,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Factor",
        "Factors rational-coefficient polynomials over the rationals using the current polynomial kernel.",
        polynomials::eval_factor,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
];
