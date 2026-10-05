use rum_core::SharedArgs;

use rum_core::{BigInt, Expr, Rational};
use rum_runtime::kernel_api::{
    EvalError, FrameRootTrace, FrameRootVisitor, Session, expr_as_exact_rational,
    expr_as_non_negative_usize, expr_as_symbol, is_system_symbol_name, list_items, system_expr,
};

mod continuation;
mod descriptor;
mod machine;
mod realization;
mod source;
mod state;
mod table_continuation;

#[derive(Clone, Copy, FrameRootTrace)]
enum PullBudget {
    AtMost(usize),
    UntilExhausted,
}

impl PullBudget {
    fn reached(self, pulled: usize) -> bool {
        match self {
            Self::AtMost(limit) => pulled >= limit,
            Self::UntilExhausted => false,
        }
    }
}

pub(crate) fn eval_lazy_drop(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [lazy_list, count_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyDrop", args, 2));
    };
    let (Some(descriptor), Some(count)) = (
        lazy_list_descriptor(lazy_list),
        expr_as_non_negative_usize(count_expr),
    ) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyDrop",
            args,
            "expected a LazyList and a non-negative integer",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyDrop"),
        vec![descriptor.clone(), Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn eval_lazy_fold_list(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [function, init, lazy_list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyFoldList", args, 3));
    };
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyFoldList",
            args,
            "expected a LazyList as the third argument",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyFoldList"),
        vec![function.clone(), init.clone(), descriptor.clone()],
    )))
}

pub(crate) fn eval_lazy_map(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [function, lazy_list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyMap", args, 2));
    };
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyMap",
            args,
            "expected a LazyList as the second argument",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyMap"),
        vec![function.clone(), descriptor.clone()],
    )))
}

pub(crate) fn eval_lazy_nest_list(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [function, init] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyNestList", args, 2));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyNestList"),
        vec![function.clone(), init.clone()],
    )))
}

pub(crate) fn eval_lazy_permutations(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyPermutations", args, 1));
    };
    if list_items(list).is_none() {
        return Ok(session.invalid_call_with_arg_form("LazyPermutations", args, "expected a List"));
    }

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyPermutations"),
        vec![list.clone()],
    )))
}

pub(crate) fn eval_lazy_range(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() > 2 {
        return Ok(session.invalid_call_with_arg_range("LazyRange", args, 0, 2));
    }

    let start = args
        .first()
        .cloned()
        .unwrap_or_else(|| Expr::small_integer(1));
    let step = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| Expr::small_integer(1));

    if expr_as_exact_rational(&start).is_none() || expr_as_exact_rational(&step).is_none() {
        return Ok(session.invalid_call_with_arg_form(
            "LazyRange",
            args,
            "start and step must be Integer or Rational values",
        ));
    }

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyRange"),
        vec![start, step],
    )))
}

pub(crate) fn eval_lazy_select(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [predicate, lazy_list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazySelect", args, 2));
    };
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazySelect",
            args,
            "expected a LazyList as the second argument",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazySelect"),
        vec![predicate.clone(), descriptor.clone()],
    )))
}

pub(crate) fn eval_lazy_subsets(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazySubsets", args, 1));
    };
    if list_items(list).is_none() {
        return Ok(session.invalid_call_with_arg_form("LazySubsets", args, "expected a List"));
    }

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazySubsets"),
        vec![list.clone()],
    )))
}

pub(crate) fn eval_lazy_table(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    use table_continuation::{BoundSlot, suspend_bounds};

    let held_args = args;
    let args = args.as_ref();
    let [body, spec] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyTable", args, 2));
    };
    let Expr::Call {
        head: spec_head,
        args: spec_args,
    } = spec
    else {
        return Ok(invalid_lazy_table_spec(session, args));
    };
    if !is_system_symbol_name(spec_head, "List") {
        return Ok(invalid_lazy_table_spec(session, args));
    }

    let one = || Expr::small_integer(1);
    let (iterator, [imin, imax, step]) = match spec_args.as_slice() {
        [iterator, imax] => (iterator, [None, Some(imax), None]),
        [iterator, imin, imax] => (iterator, [Some(imin), Some(imax), None]),
        [iterator, imin, imax, step] => (iterator, [Some(imin), Some(imax), Some(step)]),
        _ => return Ok(invalid_lazy_table_spec(session, args)),
    };
    if expr_as_symbol(iterator).is_none() {
        return Ok(invalid_lazy_table_spec(session, args));
    }

    let imin = match session.evaluate(imin.cloned().unwrap_or_else(one)) {
        Ok(value) => value,
        Err(error) => {
            return Err(suspend_bounds(
                session,
                held_args,
                iterator,
                [
                    imin.cloned().unwrap_or_else(one),
                    imax.cloned().unwrap_or_else(one),
                    step.cloned().unwrap_or_else(one),
                ],
                BoundSlot::Lower,
                error,
            ));
        }
    };
    let imax = match session.evaluate(imax.cloned().unwrap_or_else(one)) {
        Ok(value) => value,
        Err(error) => {
            return Err(suspend_bounds(
                session,
                held_args,
                iterator,
                [
                    imin,
                    imax.cloned().unwrap_or_else(one),
                    step.cloned().unwrap_or_else(one),
                ],
                BoundSlot::Upper,
                error,
            ));
        }
    };
    let step = match session.evaluate(step.cloned().unwrap_or_else(one)) {
        Ok(value) => value,
        Err(error) => {
            return Err(suspend_bounds(
                session,
                held_args,
                iterator,
                [imin, imax, step.cloned().unwrap_or_else(one)],
                BoundSlot::Step,
                error,
            ));
        }
    };
    finish_lazy_table(session, args, iterator, body, imin, imax, step)
}

fn finish_lazy_table(
    session: &mut Session,
    args: &[Expr],
    iterator: &Expr,
    body: &Expr,
    imin: Expr,
    imax: Expr,
    step: Expr,
) -> Result<Expr, EvalError> {
    let (Some(imin_value), Some(imax_value), Some(step_value)) = (
        expr_as_exact_rational(&imin),
        expr_as_exact_rational(&imax),
        expr_as_exact_rational(&step),
    ) else {
        return Ok(invalid_lazy_table_spec(session, args));
    };
    let Some(count) = lazy_table_count(&imin_value, &imax_value, &step_value) else {
        return Ok(invalid_lazy_table_spec(session, args));
    };

    let function = Expr::call(
        system_expr("Function"),
        vec![iterator.clone(), body.clone()],
    );
    let range = Expr::call(system_expr("LazyRange"), vec![imin, step]);
    let map = Expr::call(system_expr("LazyMap"), vec![function, range]);
    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyTake"),
        vec![map, Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn eval_lazy_take(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [lazy_list, count_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LazyTake", args, 2));
    };
    let (Some(descriptor), Some(count)) = (
        lazy_list_descriptor(lazy_list),
        parse_lazy_take_count(count_expr),
    ) else {
        return Ok(session.invalid_call_with_arg_form(
            "LazyTake",
            args,
            "expected a LazyList and a non-negative integer or UpTo[n]",
        ));
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyTake"),
        vec![descriptor.clone(), Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn eval_lazy_tuples(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let list_of_lists = match args {
        [list_of_lists]
            if list_items(list_of_lists)
                .is_some_and(|items| items.iter().all(|item| list_items(item).is_some())) =>
        {
            list_of_lists.clone()
        }
        [list, count] => {
            let (Some(_), Some(count)) = (list_items(list), expr_as_non_negative_usize(count))
            else {
                return Ok(session.invalid_call_with_arg_form(
                    "LazyTuples",
                    args,
                    "expected a list of lists, or a list and a non-negative integer",
                ));
            };
            Expr::call(system_expr("List"), vec![list.clone(); count])
        }
        [_] => {
            return Ok(session.invalid_call_with_arg_form(
                "LazyTuples",
                args,
                "expected a list of lists, or a list and a non-negative integer",
            ));
        }
        _ => {
            return Ok(session.invalid_call_with_message(
                "LazyTuples",
                args,
                "argx",
                format!("expected 1 or 2 argument(s), got {}", args.len()),
            ));
        }
    };

    Ok(wrap_descriptor(Expr::call(
        system_expr("LazyTuples"),
        vec![list_of_lists],
    )))
}

pub(crate) fn is_lazy_list(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, "LazyList")
    )
}

pub(crate) fn lazy_drop(lazy_list: &Expr, count: usize) -> Option<Expr> {
    let descriptor = lazy_list_descriptor(lazy_list)?;
    Some(wrap_descriptor(Expr::call(
        system_expr("LazyDrop"),
        vec![descriptor.clone(), Expr::integer(BigInt::from(count))],
    )))
}

pub(crate) fn realize_all(
    session: &mut Session,
    lazy_list: &Expr,
) -> Result<Option<Expr>, EvalError> {
    realize(session, lazy_list, PullBudget::UntilExhausted)
}

pub(crate) fn realize_at_most(
    session: &mut Session,
    lazy_list: &Expr,
    limit: usize,
) -> Result<Option<Expr>, EvalError> {
    realize(session, lazy_list, PullBudget::AtMost(limit))
}

fn realize(
    session: &mut Session,
    lazy_list: &Expr,
    budget: PullBudget,
) -> Result<Option<Expr>, EvalError> {
    let Some(descriptor) = lazy_list_descriptor(lazy_list) else {
        return Ok(None);
    };
    let Some(mut machine) = descriptor::machine_from_descriptor(descriptor) else {
        return Ok(None);
    };
    let mut ctx = state::PullContext::new(session);
    let mut items = Vec::new();
    match machine::drive_machine(&mut machine, session, &mut ctx, &mut items, budget, 0, None) {
        Ok(()) => Ok(Some(Expr::call(system_expr("List"), items))),
        Err(error) => Err(continuation::suspend_realization(
            session, machine, items, budget, ctx, error,
        )),
    }
}

fn lazy_list_descriptor(expr: &Expr) -> Option<&Expr> {
    if !is_lazy_list(expr) {
        return None;
    }
    let Expr::Call { args, .. } = expr else {
        return None;
    };
    let [descriptor] = args.as_slice() else {
        return None;
    };
    Some(descriptor)
}

fn parse_lazy_take_count(expr: &Expr) -> Option<usize> {
    if let Some(count) = expr_as_non_negative_usize(expr) {
        return Some(count);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "UpTo") {
        return None;
    }
    let [count] = args.as_slice() else {
        return None;
    };
    expr_as_non_negative_usize(count)
}

fn invalid_lazy_table_spec(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_arg_form(
        "LazyTable",
        args,
        "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step",
    )
}

fn lazy_table_count(imin: &Rational, imax: &Rational, step: &Rational) -> Option<usize> {
    if step.is_zero() {
        return None;
    }

    let zero = BigInt::from(0_u8);
    let (span, step_magnitude) = if step.numerator() > &zero {
        (imax.sub(imin), step.clone())
    } else {
        (imin.sub(imax), step.negated())
    };
    if span.numerator() < &zero {
        return Some(0);
    }

    let quotient = span.div(&step_magnitude)?;
    let count = quotient.numerator() / quotient.denominator() + BigInt::from(1_u8);
    expr_as_non_negative_usize(&Expr::integer(count))
}

fn wrap_descriptor(descriptor: Expr) -> Expr {
    Expr::call(system_expr("LazyList"), vec![descriptor])
}

#[cfg(test)]
mod tests;
