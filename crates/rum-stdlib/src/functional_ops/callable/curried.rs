use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_operator_applied(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [function] => Ok(Expr::call(head.clone(), vec![function.clone()])),
        [function, spec] if parse_curried_operator_spec(spec).is_some() => {
            Ok(Expr::call(head.clone(), vec![function.clone(), spec.clone()]))
        }
        [function, spec, accumulated]
            if parse_curried_operator_spec(spec).is_some()
                && parse_callable_wrapper_accumulated(accumulated).is_some() =>
        {
            Ok(Expr::call(
                head.clone(),
                vec![function.clone(), spec.clone(), accumulated.clone()],
            ))
        }
        [_, _] | [_, _, _] => Ok(session.invalid_call_with_arg_form(
            "OperatorApplied",
            args,
            "expected a non-negative integer, an index list, or k -> index-list as the specification argument",
        )),
        _ => Ok(session.invalid_call_with_arg_range("OperatorApplied", args, 1, 3)),
    }
}

pub(crate) fn eval_curry_applied(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [spec] => {
            if parse_curried_operator_spec(spec).is_some() {
                Ok(unary_operator_form_slot_first("CurryApplied", spec))
            } else {
                Ok(session.invalid_call_with_arg_form(
                    "CurryApplied",
                    args,
                    "expected a non-negative integer, an index list, or k -> index-list as the specification argument",
                ))
            }
        }
        [function, spec] if parse_curried_operator_spec(spec).is_some() => Ok(Expr::call(
            system_expr("CurryApplied"),
            vec![function.clone(), spec.clone()],
        )),
        [function, spec, accumulated]
            if parse_curried_operator_spec(spec).is_some()
                && parse_callable_wrapper_accumulated(accumulated).is_some() =>
        {
            Ok(Expr::call(
                system_expr("CurryApplied"),
                vec![function.clone(), spec.clone(), accumulated.clone()],
            ))
        }
        [_, _] | [_, _, _] => Ok(session.invalid_call_with_arg_form(
            "CurryApplied",
            args,
            "expected a non-negative integer, an index list, or k -> index-list as the specification argument",
        )),
        _ => Ok(session.invalid_call_with_arg_range("CurryApplied", args, 1, 3)),
    }
}

fn parse_curried_operator_spec(spec: &Expr) -> Option<(usize, Vec<usize>)> {
    if let Some(arity) = expr_as_non_negative_usize(spec) {
        return Some((arity, (0..arity).collect()));
    }

    if let Some(order) = expr_as_index_list(spec) {
        let arity = order.iter().copied().max().map_or(0usize, |max| max + 1);
        return Some((arity, order));
    }

    match spec {
        Expr::Call { head, args } if is_system_symbol_name(head, "Rule") && args.len() == 2 => {
            let arity = expr_as_non_negative_usize(&args[0])?;
            let order = expr_as_index_list(&args[1])?;
            if order.iter().any(|index| *index >= arity) {
                return None;
            }
            Some((arity, order))
        }
        _ => None,
    }
}

fn expr_as_index_list(expr: &Expr) -> Option<Vec<usize>> {
    let items = call_items_with_head(expr, &system_expr("List"))?;
    items
        .iter()
        .map(|item| {
            let one_based = expr_as_non_negative_usize(item)?;
            if one_based == 0 {
                None
            } else {
                Some(one_based - 1)
            }
        })
        .collect()
}

fn parse_callable_wrapper_accumulated(expr: &Expr) -> Option<Vec<Expr>> {
    call_items_with_head(expr, &system_expr("List")).map(|items| items.to_vec())
}
