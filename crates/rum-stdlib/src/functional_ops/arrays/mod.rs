use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{
    AbandonmentContext, FrameAbandon, FrameAbandonAction, FrameRootTrace, FrameRootVisitor,
    OptionReadContinuation,
};

mod filter;
mod generators;
mod options;
mod spec;
mod tensor;

use self::filter::*;
pub(crate) use self::generators::{eval_array, eval_outer};
use self::spec::*;
use self::tensor::*;

pub(crate) fn eval_inner(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (f, left, right, g) = match args {
        [f, left, right] => (f, left, right, system_expr("Plus")),
        [f, left, right, g] => (f, left, right, g.clone()),
        _ => return Ok(session.invalid_call_with_arg_range("Inner", args, 3, 4)),
    };

    let Some(result) = compute_inner(session, "Inner", f, left, right, &g)? else {
        return Ok(Expr::call(system_expr("Inner"), args.to_vec()));
    };

    Ok(result)
}

pub(crate) fn eval_dot(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let Some((first, rest)) = args.split_first() else {
        return Ok(Expr::call(head.clone(), Vec::new()));
    };
    if rest.is_empty() {
        return Ok(first.clone());
    }

    let times = system_expr("Times");
    let plus = system_expr("Plus");
    let mut current = first.clone();
    let mut remaining = rest.iter();
    while let Some(next) = remaining.next() {
        let result = match compute_inner(session, "Dot", &times, &current, next, &plus) {
            Ok(result) => result,
            Err(error) => {
                return suspend_dot(session, head, args, args.len() - remaining.len(), error);
            }
        };
        let Some(result) = result else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        current = result;
    }

    Ok(current)
}

#[cold]
#[inline(never)]
fn suspend_dot(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
    next_index: usize,
    error: EvalError,
) -> Result<Expr, EvalError> {
    rum_runtime::resume_or_suspend!(
        session,
        Err(error),
        ok(value) => Ok(value),
        suspend => DotSuspensionFrame {
            head: head.clone(),
            args: args.clone(),
            next_index,
        },
    )
}

fn dot_from_operand(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
    mut current: Expr,
    next_index: usize,
) -> Result<Expr, EvalError> {
    let times = system_expr("Times");
    let plus = system_expr("Plus");
    for (index, next) in args.iter().enumerate().skip(next_index) {
        let result = compute_inner(session, "Dot", &times, &current, next, &plus);
        let result = rum_runtime::resume_or_suspend!(
            session,
            result,
            ok(value) => Ok(value),
            suspend => DotSuspensionFrame {
                head: head.clone(),
                args: args.clone(),
                next_index: index + 1,
            },
        )?;
        let Some(result) = result else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        current = result;
    }

    Ok(current)
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct DotSuspensionFrame {
    head: Expr,
    args: SharedArgs,
    next_index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame for DotSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        // compute_inner's tensor frames return the completed contraction itself.
        let resumed = outcome.and_then(|current| {
            dot_from_operand(session, &self.head, &self.args, current, self.next_index)
        });
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

pub(crate) fn eval_array_filter(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    let (operator, array, radius_spec, padding_expr) = match args_slice {
        [operator, array, radius_spec] => (operator, array, radius_spec, None),
        [operator, array, radius_spec, padding_expr] => {
            (operator, array, radius_spec, Some(padding_expr))
        }
        _ => return Ok(session.invalid_call_with_arg_range("ArrayFilter", args_slice, 3, 4)),
    };

    let Some((shape, flat)) = parse_rectangular_list_array(array) else {
        return Ok(session.invalid_call_with_arg_form(
            "ArrayFilter",
            args_slice,
            "expected a rectangular List array as the second argument",
        ));
    };
    let Some(spec) = parse_array_filter_spec(radius_spec, shape.len()) else {
        return Ok(session.invalid_call_with_arg_form(
            "ArrayFilter",
            args_slice,
            "expected a non-negative integer radius, a matching list of non-negative integer radii, or a matching rectangular 0/1 template array as the third argument",
        ));
    };
    let next = options::ArrayFilterOptionWorking {
        args,
        operator,
        shape,
        flat,
        spec,
    };
    match padding_expr.cloned() {
        Some(expr) => next.finish(session, Some(expr)),
        None => session.option_rule_expr_for_symbol(
            &Symbol::system("ArrayFilter"),
            &Symbol::system("Padding"),
            next,
        ),
    }
}

pub(crate) fn eval_array_reduce(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [operator, array, dims_spec] = args else {
        return Ok(session.invalid_call_with_arg_range("ArrayReduce", args, 3, 3));
    };

    let Some((shape, flat)) = parse_rectangular_list_array(array) else {
        return Ok(session.invalid_call_with_arg_form(
            "ArrayReduce",
            args,
            "expected a rectangular List array as the second argument",
        ));
    };
    let Some(spec) = parse_array_reduce_spec(dims_spec, shape.len()) else {
        return Ok(session.invalid_call_with_arg_form(
            "ArrayReduce",
            args,
            "expected a positive integer dimension, a dimension Span, a list of dimensions, or a grouped dimension list as the third argument",
        ));
    };

    apply_array_reduce(session, operator, &shape, &flat, &spec)
}
