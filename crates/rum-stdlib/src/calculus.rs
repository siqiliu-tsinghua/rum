use rum_core::SharedArgs;
use rum_core::{Atom, BigInt, Expr, Rational};
use rum_runtime::kernel_api::{
    EvalError, Session, expr_as_exact_integer, expr_as_exact_rational, expr_as_non_negative_usize,
    expr_as_symbol, is_system_symbol_name, system_expr,
};

use crate::polynomials::parse_indeterminate_expr;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DerivativeSpec {
    variable: Expr,
    order: usize,
}

pub(crate) fn eval_d(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() < 2 {
        return Ok(session.invalid_call_with_message(
            "D",
            args,
            "argx",
            format!("expected at least 2 argument(s), got {}", args.len()),
        ));
    }

    let mut differentiated = args[0].clone();

    for spec_expr in &args[1..] {
        let Some(spec) = parse_derivative_spec(spec_expr) else {
            if let Some(invalid) = invalid_derivative_spec_call(session, args, spec_expr) {
                return Ok(invalid);
            }
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        for _ in 0..spec.order {
            differentiated = differentiate_expr(session, &differentiated, &spec.variable)?;
        }
    }

    Ok(differentiated)
}

fn invalid_derivative_spec_call(
    session: &mut Session,
    args: &[Expr],
    spec_expr: &Expr,
) -> Option<Expr> {
    if expr_as_exact_rational(spec_expr).is_some() {
        return Some(session.invalid_call_with_message(
            "D",
            args,
            "dvar",
            format!("{spec_expr} is not a valid variable."),
        ));
    }

    let Expr::Call {
        head,
        args: spec_args,
    } = spec_expr
    else {
        return None;
    };
    if !is_system_symbol_name(head, "List") || spec_args.len() != 2 {
        return None;
    }
    parse_derivative_variable(&spec_args[0])?;

    let order = expr_as_exact_integer(&spec_args[1])?;
    if order >= BigInt::from(0_u8) {
        return None;
    }

    Some(session.invalid_call_with_message(
        "D",
        args,
        "dvord",
        format!("Derivative order {order} is not a non-negative integer."),
    ))
}

fn differentiate_expr(session: &Session, expr: &Expr, variable: &Expr) -> Result<Expr, EvalError> {
    if expr == variable {
        return Ok(Expr::integer(BigInt::from(1_u8)));
    }

    if !expr_contains_exact_subexpr(expr, variable) {
        return Ok(Expr::integer(BigInt::from(0_u8)));
    }

    match expr {
        Expr::Atom(_) => Ok(Expr::integer(BigInt::from(0_u8))),
        Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => {
            differentiate_plus(session, args, variable)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") => {
            differentiate_times(session, args, variable)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Power") && args.len() == 2 => {
            differentiate_power(session, expr, &args[0], &args[1], variable)
        }
        Expr::Call { head, args } => {
            differentiate_abstract_call(session, expr, head, args, variable)
        }
    }
}

fn differentiate_plus(
    session: &Session,
    args: &[Expr],
    variable: &Expr,
) -> Result<Expr, EvalError> {
    let mut derivatives = Vec::new();

    for arg in args {
        let derivative = differentiate_expr(session, arg, variable)?;
        if !expr_is_exact_zero(&derivative) {
            derivatives.push(derivative);
        }
    }

    session.combine_additive_args(&system_expr("Plus"), &derivatives)
}

fn differentiate_times(
    session: &Session,
    args: &[Expr],
    variable: &Expr,
) -> Result<Expr, EvalError> {
    let mut terms = Vec::new();

    for (index, arg) in args.iter().enumerate() {
        let derivative = differentiate_expr(session, arg, variable)?;
        if expr_is_exact_zero(&derivative) {
            continue;
        }

        let mut factors = Vec::with_capacity(args.len());
        for (factor_index, factor) in args.iter().enumerate() {
            if factor_index == index {
                factors.push(derivative.clone());
            } else {
                factors.push(factor.clone());
            }
        }

        terms.push(session.combine_multiplicative_args(&system_expr("Times"), &factors)?);
    }

    session.combine_additive_args(&system_expr("Plus"), &terms)
}

fn differentiate_power(
    session: &Session,
    original: &Expr,
    base: &Expr,
    exponent: &Expr,
    variable: &Expr,
) -> Result<Expr, EvalError> {
    if let Some(exponent_value) = expr_as_exact_rational(exponent) {
        if exponent_value.is_zero() {
            return Ok(Expr::integer(BigInt::from(0_u8)));
        }

        let base_derivative = differentiate_expr(session, base, variable)?;
        if expr_is_exact_zero(&base_derivative) {
            return Ok(Expr::integer(BigInt::from(0_u8)));
        }

        if exponent_value.is_one() {
            return Ok(base_derivative);
        }

        let decreased = exponent_value.sub(&Rational::one());
        let power = power_expr(base.clone(), decreased);

        return session.combine_multiplicative_args(
            &system_expr("Times"),
            &[Expr::rational(exponent_value), power, base_derivative],
        );
    }

    differentiate_abstract_call(
        session,
        original,
        &system_expr("Power"),
        &[base.clone(), exponent.clone()],
        variable,
    )
}

fn differentiate_abstract_call(
    session: &Session,
    original: &Expr,
    head: &Expr,
    args: &[Expr],
    variable: &Expr,
) -> Result<Expr, EvalError> {
    let mut terms = Vec::new();

    for (index, arg) in args.iter().enumerate() {
        let derivative = differentiate_expr(session, arg, variable)?;
        if expr_is_exact_zero(&derivative) {
            continue;
        }

        let Some(partial) = partial_derivative_call(head, args, index) else {
            return Ok(Expr::call(
                system_expr("D"),
                vec![original.clone(), variable.clone()],
            ));
        };

        terms.push(
            session.combine_multiplicative_args(&system_expr("Times"), &[partial, derivative])?,
        );
    }

    session.combine_additive_args(&system_expr("Plus"), &terms)
}

fn parse_derivative_spec(expr: &Expr) -> Option<DerivativeSpec> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") && args.len() == 2 => {
            Some(DerivativeSpec {
                variable: parse_derivative_variable(&args[0])?,
                order: expr_as_non_negative_usize(&args[1])?,
            })
        }
        _ => Some(DerivativeSpec {
            variable: parse_derivative_variable(expr)?,
            order: 1,
        }),
    }
}

fn parse_derivative_variable(expr: &Expr) -> Option<Expr> {
    let parsed = parse_indeterminate_expr(expr)?;

    match &parsed {
        Expr::Atom(Atom::Symbol(_)) => Some(parsed),
        Expr::Call { head, .. } if expr_as_symbol(head).is_some() => Some(parsed),
        _ => None,
    }
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

fn expr_is_exact_zero(expr: &Expr) -> bool {
    expr_as_exact_rational(expr).is_some_and(|value| value.is_zero())
}

fn power_expr(base: Expr, exponent: Rational) -> Expr {
    if exponent.is_zero() {
        Expr::integer(BigInt::from(1_u8))
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
        Expr::Atom(Atom::Symbol(_)) => Some((head.clone(), vec![0; arity])),
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
