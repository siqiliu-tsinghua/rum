use rum_core::{BigInt, Expr, Rational, SharedArgs};
use rum_runtime::kernel_api::{
    EvalError, Session, expr_as_exact_rational, expr_as_non_negative_usize, is_system_symbol_name,
    system_expr,
};

use super::containment::expr_contains_exact_subexpr;

mod iterative;

use iterative::differentiate_expr_iterative_after_prepass;

const RECURSIVE_DIFFERENTIATION_CALL_DEPTH: usize = 16;

pub(super) fn differentiate_expr(
    session: &Session,
    expr: &Expr,
    variable: &Expr,
) -> Result<Expr, EvalError> {
    differentiate_expr_recursive_bounded(
        session,
        expr,
        variable,
        RECURSIVE_DIFFERENTIATION_CALL_DEPTH,
    )
}

fn differentiate_expr_recursive_bounded(
    session: &Session,
    expr: &Expr,
    variable: &Expr,
    remaining_call_depth: usize,
) -> Result<Expr, EvalError> {
    if expr == variable {
        return Ok(one_expr());
    }

    if !expr_contains_exact_subexpr(expr, variable) {
        return Ok(zero_expr());
    }

    if remaining_call_depth == 0 && matches!(expr, Expr::Call { .. }) {
        return differentiate_expr_iterative_after_prepass(session, expr, variable);
    }

    match expr {
        Expr::Atom(_) => Ok(zero_expr()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => {
            differentiate_plus_recursive_bounded(
                session,
                args,
                variable,
                next_call_depth(remaining_call_depth),
            )
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") => {
            differentiate_times_recursive_bounded(
                session,
                args,
                variable,
                next_call_depth(remaining_call_depth),
            )
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Power") && args.len() == 2 => {
            differentiate_power_recursive_bounded(
                session,
                expr,
                &args[0],
                &args[1],
                variable,
                next_call_depth(remaining_call_depth),
            )
        }
        Expr::Call { head, args } => differentiate_abstract_call_recursive_bounded(
            session,
            expr,
            head,
            args,
            variable,
            next_call_depth(remaining_call_depth),
        ),
    }
}

fn differentiate_plus_recursive_bounded(
    session: &Session,
    args: &[Expr],
    variable: &Expr,
    remaining_call_depth: usize,
) -> Result<Expr, EvalError> {
    let mut derivatives = Vec::new();

    for arg in args {
        let derivative =
            differentiate_expr_recursive_bounded(session, arg, variable, remaining_call_depth)?;
        if !expr_is_exact_zero(&derivative) {
            derivatives.push(derivative);
        }
    }

    session.combine_additive_args(&system_expr("Plus"), &derivatives)
}

fn differentiate_times_recursive_bounded(
    session: &Session,
    args: &[Expr],
    variable: &Expr,
    remaining_call_depth: usize,
) -> Result<Expr, EvalError> {
    let mut terms = Vec::new();

    for (index, arg) in args.iter().enumerate() {
        let derivative =
            differentiate_expr_recursive_bounded(session, arg, variable, remaining_call_depth)?;
        if expr_is_exact_zero(&derivative) {
            continue;
        }

        terms.push(combine_product_rule_term(session, args, index, derivative)?);
    }

    session.combine_additive_args(&system_expr("Plus"), &terms)
}

fn differentiate_power_recursive_bounded(
    session: &Session,
    original: &Expr,
    base: &Expr,
    exponent: &Expr,
    variable: &Expr,
    remaining_call_depth: usize,
) -> Result<Expr, EvalError> {
    if let Some(exponent_value) = expr_as_exact_rational(exponent) {
        if exponent_value.is_zero() {
            return Ok(zero_expr());
        }

        let base_derivative =
            differentiate_expr_recursive_bounded(session, base, variable, remaining_call_depth)?;
        return finish_rational_power_derivative(session, base, exponent_value, base_derivative);
    }

    differentiate_abstract_call_recursive_bounded(
        session,
        original,
        &system_expr("Power"),
        &[base.clone(), exponent.clone()],
        variable,
        remaining_call_depth,
    )
}

fn differentiate_abstract_call_recursive_bounded(
    session: &Session,
    original: &Expr,
    head: &Expr,
    args: &[Expr],
    variable: &Expr,
    remaining_call_depth: usize,
) -> Result<Expr, EvalError> {
    let mut terms = Vec::new();

    for (index, arg) in args.iter().enumerate() {
        let derivative =
            differentiate_expr_recursive_bounded(session, arg, variable, remaining_call_depth)?;
        if expr_is_exact_zero(&derivative) {
            continue;
        }

        let Some(partial) = partial_derivative_call(head, args, index) else {
            return Ok(unevaluated_derivative(original, variable));
        };
        terms.push(combine_chain_rule_term(session, partial, derivative)?);
    }

    session.combine_additive_args(&system_expr("Plus"), &terms)
}

fn next_call_depth(remaining_call_depth: usize) -> usize {
    remaining_call_depth
        .checked_sub(1)
        .expect("iterative fallback handles the bounded differentiation tail")
}

fn combine_product_rule_term(
    session: &Session,
    args: &[Expr],
    index: usize,
    derivative: Expr,
) -> Result<Expr, EvalError> {
    let mut factors = Vec::with_capacity(args.len());
    for (factor_index, factor) in args.iter().enumerate() {
        if factor_index == index {
            factors.push(derivative.clone());
        } else {
            factors.push(factor.clone());
        }
    }

    session.combine_multiplicative_args(&system_expr("Times"), &factors)
}

fn finish_rational_power_derivative(
    session: &Session,
    base: &Expr,
    exponent_value: Rational,
    base_derivative: Expr,
) -> Result<Expr, EvalError> {
    if expr_is_exact_zero(&base_derivative) {
        return Ok(zero_expr());
    }

    if exponent_value.is_one() {
        return Ok(base_derivative);
    }

    let decreased = exponent_value.sub(&Rational::one());
    let power = power_expr(base.clone(), decreased);
    session.combine_multiplicative_args(
        &system_expr("Times"),
        &[Expr::rational(exponent_value), power, base_derivative],
    )
}

fn combine_chain_rule_term(
    session: &Session,
    partial: Expr,
    derivative: Expr,
) -> Result<Expr, EvalError> {
    session.combine_multiplicative_args(&system_expr("Times"), &[partial, derivative])
}

fn unevaluated_derivative(original: &Expr, variable: &Expr) -> Expr {
    Expr::call(system_expr("D"), vec![original.clone(), variable.clone()])
}

fn zero_expr() -> Expr {
    Expr::integer(BigInt::from(0_u8))
}

fn one_expr() -> Expr {
    Expr::integer(BigInt::from(1_u8))
}

fn expr_is_exact_zero(expr: &Expr) -> bool {
    expr_as_exact_rational(expr).is_some_and(|value| value.is_zero())
}

fn power_expr(base: Expr, exponent: Rational) -> Expr {
    if exponent.is_zero() {
        one_expr()
    } else if exponent.is_one() {
        base
    } else {
        Expr::call(system_expr("Power"), vec![base, Expr::rational(exponent)])
    }
}

fn partial_derivative_call(head: &Expr, args: &[Expr], index: usize) -> Option<Expr> {
    let operator = increment_derivative_operator(head, args.len(), index)?;
    Some(Expr::call(operator, args.to_vec()))
}

fn increment_derivative_operator(head: &Expr, arity: usize, index: usize) -> Option<Expr> {
    let (base, mut orders) = parse_derivative_operator(head, arity)?;
    *orders.get_mut(index)? += 1;
    Some(build_derivative_operator(base, &orders))
}

fn parse_derivative_operator(head: &Expr, arity: usize) -> Option<(Expr, Vec<usize>)> {
    match head {
        Expr::Atom(rum_core::Atom::Symbol(_)) => Some((head.clone(), vec![0; arity])),
        Expr::Call {
            head: derivative_head,
            args,
        } if args.len() == 1 => {
            let Expr::Call {
                head: derivative_symbol,
                args: order_args,
            } = derivative_head.as_ref()
            else {
                return None;
            };

            if !is_system_symbol_name(derivative_symbol, "Derivative") {
                return None;
            }

            let orders = order_args
                .iter()
                .map(expr_as_non_negative_usize)
                .collect::<Option<Vec<_>>>()?;

            (orders.len() == arity).then(|| (args[0].clone(), orders))
        }
        _ => None,
    }
}

fn build_derivative_operator(base: Expr, orders: &[usize]) -> Expr {
    Expr::call(
        Expr::call(
            system_expr("Derivative"),
            orders
                .iter()
                .map(|order| Expr::integer(BigInt::from(*order as u64)))
                .collect(),
        ),
        vec![base],
    )
}
