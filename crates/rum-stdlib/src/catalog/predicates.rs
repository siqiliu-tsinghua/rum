use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::predicates;

pub(crate) const PREDICATE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "TrueQ",
        "Tests whether an expression evaluates exactly to True.",
        predicates::eval_true_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "BooleanQ",
        "Tests whether an expression evaluates exactly to True or False.",
        predicates::eval_boolean_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Boole",
        "Converts exact True and False to 1 and 0, leaving non-boolean conditions unevaluated.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_boole,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "AtomQ",
        "Tests whether an expression is atomic rather than a normal expression.",
        predicates::eval_atom_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ListQ",
        "Tests whether an expression has head List.",
        predicates::eval_list_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringQ",
        "Tests whether an expression is a string atom.",
        predicates::eval_string_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "IntegerQ",
        "Tests whether an expression is an exact integer atom.",
        predicates::eval_integer_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "NumberQ",
        "Tests whether an expression is an exact numeric atom in the current number subset.",
        predicates::eval_number_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "NumericQ",
        "Tests whether an expression is currently numeric in the exact-number subset.",
        predicates::eval_numeric_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "EvenQ",
        "Tests whether an expression is currently an even exact integer.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_even_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "OddQ",
        "Tests whether an expression is currently an odd exact integer.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_odd_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Positive",
        "Tests positivity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_positive,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Negative",
        "Tests negativity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_negative,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "NonPositive",
        "Tests non-positivity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_non_positive,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "NonNegative",
        "Tests non-negativity in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_non_negative,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "PossibleZeroQ",
        "Tests exact zero in the current exact integer/rational subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_possible_zero_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "MachineNumberQ",
        "Tests whether an expression is a machine number; currently always False because only exact numbers exist.",
        predicates::eval_machine_number_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "RealValuedNumberQ",
        "Tests whether an expression is a real-valued number in the current exact-number subset.",
        predicates::eval_real_valued_number_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "RealValuedNumericQ",
        "Tests whether an expression is real-valued and numeric in the current exact-number subset.",
        predicates::eval_real_valued_numeric_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "PrimeQ",
        "Tests primality in the current exact-integer subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_prime_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "AlgebraicIntegerQ",
        "Tests whether an expression is an exact integer in the current algebraic-integer subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_algebraic_integer_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "PolynomialQ",
        "Tests whether an expression is polynomial in the current exact symbolic subset.",
        predicates::eval_polynomial_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a variable or variable list",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "PolynomialExpressionQ",
        "Tests whether an expression is polynomial in the requested variable subset.",
        predicates::eval_polynomial_expression_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a variable or variable list",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "RationalExpressionQ",
        "Tests whether an expression is a rational expression in the requested variable subset.",
        predicates::eval_rational_expression_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a variable or variable list",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "DateObjectQ",
        "Tests whether an expression has a DateObject head in the current structural subset.",
        predicates::eval_date_object_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "TimeObjectQ",
        "Tests whether an expression has a TimeObject head in the current structural subset.",
        predicates::eval_time_object_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "DuplicateFreeQ",
        "Tests whether a List has no repeated elements, optionally using SameTest.",
        predicates::eval_duplicate_free_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "IntersectingQ",
        "Tests whether two Lists share any element, optionally using SameTest.",
        predicates::eval_intersecting_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "DisjointQ",
        "Tests whether two Lists share no element, optionally using SameTest.",
        predicates::eval_disjoint_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "SubsetQ",
        "Tests subset inclusion for Lists, optionally using SameTest.",
        predicates::eval_subset_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Order",
        "Orders two expressions using exact numeric order when possible, then the current canonical textual ordering subset.",
        predicates::eval_order,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "OrderedQ",
        "Tests whether an expression's arguments are ordered under the default order or a supplied binary predicate.",
        predicates::eval_ordered_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "LexicographicOrder",
        "Compares two expressions lexicographically under the default order or a supplied binary predicate.",
        predicates::eval_lexicographic_order,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "NumericalOrder",
        "Compares two expressions using exact numeric order when possible, then canonical order.",
        predicates::eval_numerical_order,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "AlphabeticOrder",
        "Compares two strings alphabetically in the current string-order subset.",
        predicates::eval_alphabetic_order,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "nstr",
            template: "argument `1` is not a string",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ContainsAll",
        "Tests whether a List contains every requested element, optionally using SameTest.",
        predicates::eval_contains_all,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ContainsAny",
        "Tests whether a List contains any requested element, optionally using SameTest.",
        predicates::eval_contains_any,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ContainsExactly",
        "Tests List multiset equality, optionally using SameTest.",
        predicates::eval_contains_exactly,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ContainsNone",
        "Tests whether a List contains none of the requested elements, optionally using SameTest.",
        predicates::eval_contains_none,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ContainsOnly",
        "Tests whether every element of a List belongs to the allowed-element List, optionally using SameTest.",
        predicates::eval_contains_only,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Between",
        "Tests interval inclusion in the current exact numeric subset.",
        predicates::eval_between,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "VectorQ",
        "Tests whether an expression is a one-dimensional List array, optionally requiring every leaf to satisfy a test.",
        predicates::eval_vector_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "MatrixQ",
        "Tests whether an expression is a rectangular two-dimensional List array, optionally requiring every leaf to satisfy a test.",
        predicates::eval_matrix_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ArrayQ",
        "Tests whether an expression is a rectangular List array, optionally matching its depth against a pattern and requiring every leaf to satisfy a test.",
        predicates::eval_array_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "AllTrue",
        "Tests whether all selected structural parts satisfy a criterion in the current boolean subset.",
        predicates::eval_all_true,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a standard levelspec as the third argument",
        },
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "AnyTrue",
        "Tests whether any selected structural part satisfies a criterion in the current boolean subset.",
        predicates::eval_any_true,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a standard levelspec as the third argument",
        },
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "NoneTrue",
        "Tests whether no selected structural parts satisfy a criterion in the current boolean subset.",
        predicates::eval_none_true,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a standard levelspec as the third argument",
        },
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "CoprimeQ",
        "Tests whether exact integers are pairwise relatively prime in the current integer subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        predicates::eval_coprime_q,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "SameQ",
        "Tests whether all arguments are exactly identical after evaluation.",
        predicates::eval_same_q,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "UnsameQ",
        "Tests whether no two arguments are exactly identical after evaluation.",
        predicates::eval_unsame_q,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
];
