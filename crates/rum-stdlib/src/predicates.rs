use rum_core::SharedArgs;
use rum_core::{Atom, BigInt, Expr, Rational, SYSTEM_CONTEXT, heads};
use rum_runtime::kernel_api::{
    CasesTraversalConfig, EvalError, Session, collect_level_parts_owned, compare_expr_canonical,
    expr_as_exact_integer, expr_as_exact_rational, expr_as_non_negative_usize, expr_as_string,
    expr_as_symbol, is_system_symbol_name, parse_standard_level_spec, system_expr,
    truthiness_symbol,
};

mod arrays;
mod basic;
mod collections;
mod numeric;
mod ordering;
mod suspended_state;
mod truth;

use self::suspended_state::{FrozenPredicateSequence, PersistentPredicateMatches};

pub(crate) use self::arrays::{eval_array_q, eval_matrix_q, eval_vector_q};
pub(crate) use self::basic::{
    eval_atom_q, eval_date_object_q, eval_list_q, eval_string_q, eval_time_object_q,
};
pub(crate) use self::collections::{
    eval_contains_all, eval_contains_any, eval_contains_exactly, eval_contains_none,
    eval_contains_only, eval_disjoint_q, eval_duplicate_free_q, eval_intersecting_q, eval_subset_q,
};
pub(crate) use self::numeric::{
    eval_algebraic_integer_q, eval_coprime_q, eval_even_q, eval_integer_q, eval_machine_number_q,
    eval_negative, eval_non_negative, eval_non_positive, eval_number_q, eval_numeric_q, eval_odd_q,
    eval_positive, eval_possible_zero_q, eval_prime_q, eval_real_valued_number_q,
    eval_real_valued_numeric_q,
};
pub(crate) use self::ordering::{
    eval_alphabetic_order, eval_between, eval_lexicographic_order, eval_numerical_order,
    eval_order, eval_ordered_q, eval_same_q, eval_unsame_q,
};
pub(crate) use self::truth::{
    eval_all_true, eval_any_true, eval_boole, eval_boolean_q, eval_none_true, eval_true_q,
};

fn exact_numeric_value(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.to_bigint())),
        Expr::Atom(Atom::Rational(value)) => Some(value.as_ref().clone()),
        _ => None,
    }
}

fn string_atom_value(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Atom(Atom::String(value)) => Some(value),
        _ => None,
    }
}

fn exact_integer_value(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.to_bigint()),
        _ => None,
    }
}

fn expr_has_system_head(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Call { head, .. } => is_system_symbol_name(head, name),
        Expr::Atom(_) => false,
    }
}

fn list_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::structure::LIST) => {
            Some(args)
        }
        _ => None,
    }
}

fn two_list_args(args: &[Expr]) -> Option<(&[Expr], &[Expr])> {
    let [lhs, rhs] = args else {
        return None;
    };

    Some((list_items(lhs)?, list_items(rhs)?))
}

fn expr_set(items: &[Expr]) -> std::collections::HashSet<Expr> {
    items.iter().cloned().collect()
}

fn expr_counts(items: &[Expr]) -> std::collections::HashMap<Expr, usize> {
    let mut counts = std::collections::HashMap::new();

    for item in items {
        *counts.entry(item.clone()).or_insert(0) += 1;
    }

    counts
}

fn array_shape(expr: &Expr) -> Option<Vec<usize>> {
    let items = list_items(expr)?;
    list_shape(items)
}

fn list_shape(items: &[Expr]) -> Option<Vec<usize>> {
    if items.is_empty() {
        return Some(vec![0]);
    }

    let child_shape = element_shape(&items[0])?;
    for item in &items[1..] {
        if element_shape(item)? != child_shape {
            return None;
        }
    }

    let mut shape = vec![items.len()];
    shape.extend(child_shape);
    Some(shape)
}

fn element_shape(expr: &Expr) -> Option<Vec<usize>> {
    match list_items(expr) {
        Some(items) => list_shape(items),
        None => Some(Vec::new()),
    }
}

fn compare_exact_numeric(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

fn compare_exact_order_subset(lhs: &Expr, rhs: &Expr) -> std::cmp::Ordering {
    match (exact_numeric_value(lhs), exact_numeric_value(rhs)) {
        (Some(lhs), Some(rhs)) => compare_exact_numeric(&lhs, &rhs),
        _ => compare_expr_canonical(lhs, rhs),
    }
}

fn compare_lexicographic_order_subset(lhs: &Expr, rhs: &Expr) -> std::cmp::Ordering {
    match (list_items(lhs), list_items(rhs)) {
        (Some(lhs), Some(rhs)) => compare_lexicographic_slices(lhs, rhs),
        _ => match (string_atom_value(lhs), string_atom_value(rhs)) {
            (Some(lhs), Some(rhs)) => lhs.cmp(rhs),
            _ => compare_exact_order_subset(lhs, rhs),
        },
    }
}

fn compare_lexicographic_slices(lhs: &[Expr], rhs: &[Expr]) -> std::cmp::Ordering {
    for (lhs_item, rhs_item) in lhs.iter().zip(rhs) {
        let ordering = compare_lexicographic_order_subset(lhs_item, rhs_item);
        if ordering != std::cmp::Ordering::Equal {
            return ordering;
        }
    }

    lhs.len().cmp(&rhs.len())
}

fn order_result_expr(ordering: std::cmp::Ordering) -> Expr {
    let value = match ordering {
        std::cmp::Ordering::Less => 1_i64,
        std::cmp::Ordering::Equal => 0_i64,
        std::cmp::Ordering::Greater => -1_i64,
    };

    Expr::integer(BigInt::from(value))
}

fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

pub(crate) fn eval_polynomial_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr] => Ok(polynomial_predicate_bool_expr(
            is_polynomial_expression_unconstrained(expr),
        )),
        [expr, vars_expr] => {
            let Some(variables) = parse_polynomial_variable_spec(vars_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "PolynomialQ",
                    args,
                    "expected a variable or variable list",
                ));
            };
            Ok(polynomial_predicate_bool_expr(
                is_polynomial_expression_in_variables(expr, &variables),
            ))
        }
        _ => Ok(session.invalid_call_with_arg_range("PolynomialQ", args, 1, 2)),
    }
}

pub(crate) fn eval_polynomial_expression_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_polynomial_like_expression_q(
        session,
        head,
        args,
        "PolynomialExpressionQ",
        is_polynomial_expression_in_variables,
    )
}

pub(crate) fn eval_rational_expression_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_polynomial_like_expression_q(
        session,
        head,
        args,
        "RationalExpressionQ",
        is_rational_expression_in_variables,
    )
}

fn eval_polynomial_like_expression_q<F>(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    predicate: F,
) -> Result<Expr, EvalError>
where
    F: Fn(&Expr, &[Expr]) -> bool,
{
    match args {
        [expr, vars_expr] => {
            let Some(variables) = parse_polynomial_variable_spec(vars_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    symbol,
                    args,
                    "expected a variable or variable list",
                ));
            };
            Ok(polynomial_predicate_bool_expr(predicate(expr, &variables)))
        }
        [_, _, _] => Ok(Expr::call(head.clone(), args.to_vec())),
        _ => Ok(session.invalid_call_with_arg_range(symbol, args, 2, 3)),
    }
}

fn parse_polynomial_variable_spec(expr: &Expr) -> Option<Vec<Expr>> {
    if let Expr::Call { head, args } = expr {
        if is_system_symbol_name(head, heads::structure::LIST) {
            return args.iter().map(parse_indeterminate_expr).collect();
        }
    }

    parse_indeterminate_expr(expr).map(|variable| vec![variable])
}

fn is_polynomial_expression_unconstrained(expr: &Expr) -> bool {
    if expr_as_exact_rational(expr).is_some() {
        return true;
    }
    if expr_contains_pattern_language(expr) {
        return false;
    }

    match expr {
        Expr::Atom(Atom::String(_)) => false,
        Expr::Atom(_) => true,
        Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => {
            args.iter().all(is_polynomial_expression_unconstrained)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") => {
            args.iter().all(is_polynomial_expression_unconstrained)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Power") && args.len() == 2 => {
            if let Some(exponent) = expr_as_non_negative_usize(&args[1]) {
                return exponent == 0 || is_polynomial_expression_unconstrained(&args[0]);
            }

            if expr_as_exact_integer(&args[1]).is_some_and(|exponent| exponent < BigInt::from(0_u8))
            {
                return is_opaque_polynomial_indeterminate(&args[0]);
            }

            true
        }
        Expr::Call { .. } => true,
    }
}

fn is_polynomial_expression_in_variables(expr: &Expr, variables: &[Expr]) -> bool {
    if expr_as_exact_rational(expr).is_some() {
        return true;
    }
    if variables.iter().any(|variable| variable == expr) {
        return true;
    }

    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => args
            .iter()
            .all(|arg| is_polynomial_expression_in_variables(arg, variables)),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") => args
            .iter()
            .all(|arg| is_polynomial_expression_in_variables(arg, variables)),
        Expr::Call { head, args } if is_system_symbol_name(head, "Power") && args.len() == 2 => {
            expr_as_non_negative_usize(&args[1]).is_some()
                && is_polynomial_expression_in_variables(&args[0], variables)
        }
        _ => is_valid_polynomial_coefficient(expr, variables),
    }
}

fn is_rational_expression_in_variables(expr: &Expr, variables: &[Expr]) -> bool {
    if is_polynomial_expression_in_variables(expr, variables) {
        return true;
    }

    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => args
            .iter()
            .all(|arg| is_rational_expression_in_variables(arg, variables)),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") => args
            .iter()
            .all(|arg| is_rational_expression_in_variables(arg, variables)),
        Expr::Call { head, args } if is_system_symbol_name(head, "Power") && args.len() == 2 => {
            expr_as_exact_integer(&args[1]).is_some()
                && is_rational_expression_in_variables(&args[0], variables)
        }
        _ => is_valid_polynomial_coefficient(expr, variables),
    }
}

fn is_valid_polynomial_coefficient(expr: &Expr, variables: &[Expr]) -> bool {
    !expr_contains_pattern_language(expr)
        && variables
            .iter()
            .all(|variable| !expr_contains_exact_subexpr(expr, variable))
}

fn expr_contains_exact_subexpr(expr: &Expr, needle: &Expr) -> bool {
    if expr == needle {
        return true;
    }

    match expr {
        Expr::Call { head, args } => {
            expr_contains_exact_subexpr(head, needle)
                || args
                    .iter()
                    .any(|arg| expr_contains_exact_subexpr(arg, needle))
        }
        Expr::Atom(_) => false,
    }
}

fn is_opaque_polynomial_indeterminate(expr: &Expr) -> bool {
    if expr_contains_pattern_language(expr) {
        return false;
    }

    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, .. }
            if is_system_symbol_name(head, "Plus")
                || is_system_symbol_name(head, "Times")
                || is_system_symbol_name(head, "Power") =>
        {
            false
        }
        Expr::Call { .. } => true,
    }
}

fn polynomial_predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

pub(super) fn expr_contains_pattern_language(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => {
            is_pattern_language_head(head)
                || expr_contains_pattern_language(head)
                || args.iter().any(expr_contains_pattern_language)
        }
        _ => false,
    }
}

fn is_pattern_language_head(expr: &Expr) -> bool {
    matches!(
        expr_as_symbol(expr),
        Some(symbol)
            if symbol.context() == SYSTEM_CONTEXT
                && matches!(
                    symbol.name(),
                    "Alternatives"
                        | "Blank"
                        | "BlankNullSequence"
                        | "BlankSequence"
                        | "Condition"
                        | "Except"
                        | "HoldPattern"
                        | "Optional"
                        | "OrderlessPatternSequence"
                        | "Pattern"
                        | "PatternSequence"
                        | "PatternTest"
                        | "Repeated"
                        | "RepeatedNull"
                        | "Verbatim"
                )
    )
}

pub fn parse_indeterminate_expr(expr: &Expr) -> Option<Expr> {
    if expr_contains_pattern_language(expr) {
        return None;
    }

    match expr {
        Expr::Atom(Atom::Integer(_))
        | Expr::Atom(Atom::Rational(_))
        | Expr::Atom(Atom::String(_)) => None,
        Expr::Call { head, .. }
            if is_system_symbol_name(head, "Plus")
                || is_system_symbol_name(head, "Times")
                || is_system_symbol_name(head, "Power") =>
        {
            None
        }
        _ => Some(expr.clone()),
    }
}
