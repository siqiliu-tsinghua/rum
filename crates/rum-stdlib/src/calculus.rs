use rum_core::SharedArgs;
use rum_core::{Atom, BigInt, Expr};
use rum_runtime::kernel_api::{
    EvalError, Session, expr_as_exact_integer, expr_as_exact_rational, expr_as_non_negative_usize,
    expr_as_symbol, is_system_symbol_name,
};

use crate::polynomials::parse_indeterminate_expr;

mod containment;
mod differentiation;

use differentiation::differentiate_expr;

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
