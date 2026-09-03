use rum_core::SharedArgs;
use rum_core::{Atom, BigInt, Expr, IntValue};
use rum_runtime::kernel_api::{
    ControlDataBuiltin, EvalError, Session, ToPrimitive, expr_as_symbol, is_system_symbol_name,
};

fn dispatch(
    session: &mut Session,
    builtin: ControlDataBuiltin,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_control_data_builtin(builtin, head, args)
}

pub(crate) fn eval_compound_expression(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::CompoundExpression, head, args)
}

pub(crate) fn eval_if(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::If, head, args)
}

pub(crate) fn eval_which(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Which, head, args)
}

pub(crate) fn eval_switch(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Switch, head, args)
}

pub(crate) fn eval_do(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    enforce_static_multi_iterator_product(session, args, true, false)?;
    dispatch(session, ControlDataBuiltin::Do, head, args)
}

pub(crate) fn eval_product(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    enforce_static_multi_iterator_product(session, args, false, false)?;
    dispatch(session, ControlDataBuiltin::Product, head, args)
}

pub(crate) fn eval_sum(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    enforce_static_multi_iterator_product(session, args, false, false)?;
    dispatch(session, ControlDataBuiltin::Sum, head, args)
}

pub(crate) fn eval_while(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::While, head, args)
}

pub(crate) fn eval_until(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Until, head, args)
}

pub(crate) fn eval_for(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::For, head, args)
}

pub(crate) fn eval_table(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    enforce_static_multi_iterator_product(session, args, true, true)?;
    dispatch(session, ControlDataBuiltin::Table, head, args)
}

fn enforce_static_multi_iterator_product(
    session: &Session,
    args: &[Expr],
    accepts_bare_count: bool,
    generates_elements: bool,
) -> Result<(), EvalError> {
    let Some(specs) = args.get(1..).filter(|specs| specs.len() > 1) else {
        return Ok(());
    };
    let Some(product) = static_iterator_product(specs, accepts_bare_count) else {
        return Ok(());
    };

    match product {
        StaticIteratorProduct::Count(product) => {
            if generates_elements {
                session.enforce_generated_element_limit_for_count(product)?;
            }
            let limit = session.resource_limits().iteration_work_limit;
            if product > limit {
                return Err(EvalError::IterationWorkLimitExceeded { limit });
            }
            Ok(())
        }
        StaticIteratorProduct::Big(product) => {
            if generates_elements {
                session.enforce_generated_element_limit_for_big_count(&product)?;
            }
            session.enforce_iteration_work_limit_for_big_count(&product)
        }
    }
}

enum StaticIteratorProduct {
    Count(usize),
    Big(BigInt),
}

fn static_iterator_product(
    specs: &[Expr],
    accepts_bare_count: bool,
) -> Option<StaticIteratorProduct> {
    let mut small_product = Some(1_usize);
    let mut big_product = None;
    for spec in specs {
        let count = static_iterator_count(spec, accepts_bare_count)?;
        if let Some(product) = small_product {
            if let Some(product) = product.checked_mul(count) {
                small_product = Some(product);
                continue;
            }
            big_product = Some(BigInt::from(product) * BigInt::from(count));
            small_product = None;
        } else if let Some(product) = &mut big_product {
            *product *= count;
        }
    }
    Some(match big_product {
        Some(product) => StaticIteratorProduct::Big(product),
        None => StaticIteratorProduct::Count(small_product.unwrap_or_default()),
    })
}

fn static_iterator_count(spec: &Expr, accepts_bare_count: bool) -> Option<usize> {
    if let Some(count) = static_non_negative_integer(spec) {
        return accepts_bare_count.then_some(count);
    }

    let Expr::Call { head, args, .. } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    match args.as_slice() {
        [count] => static_non_negative_integer(count),
        [symbol, values_or_end] => {
            expr_as_symbol(symbol)?;
            if let Expr::Call {
                head: values_head,
                args: values,
                ..
            } = values_or_end
            {
                if is_system_symbol_name(values_head, "List")
                    && values.iter().all(static_iterator_value_is_stable)
                {
                    return Some(values.len());
                }
            }
            static_integer_range_count(1, static_exact_machine_integer(values_or_end)?, 1)
        }
        [symbol, start, end] => {
            expr_as_symbol(symbol)?;
            static_integer_range_count(
                static_exact_machine_integer(start)?,
                static_exact_machine_integer(end)?,
                1,
            )
        }
        [symbol, start, end, step] => {
            expr_as_symbol(symbol)?;
            static_integer_range_count(
                static_exact_machine_integer(start)?,
                static_exact_machine_integer(end)?,
                static_exact_machine_integer(step)?,
            )
        }
        _ => None,
    }
}

fn static_non_negative_integer(expr: &Expr) -> Option<usize> {
    let value = static_exact_machine_integer(expr)?;
    usize::try_from(value).ok()
}

fn static_exact_machine_integer(expr: &Expr) -> Option<i128> {
    if let Expr::Atom(Atom::Integer(value)) = expr {
        return match value {
            IntValue::Small(value) => Some(i128::from(*value)),
            IntValue::Big(value) => value.to_i128(),
        };
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if is_system_symbol_name(head, "Plus") {
        let value = args.iter().try_fold(0_i128, |total, arg| {
            total.checked_add(static_exact_machine_integer(arg)?)
        })?;
        return Some(value);
    }
    if is_system_symbol_name(head, "Times") {
        let value = args.iter().try_fold(1_i128, |product, arg| {
            product.checked_mul(static_exact_machine_integer(arg)?)
        })?;
        return Some(value);
    }
    if is_system_symbol_name(head, "Power") {
        let [base, exponent] = args.as_slice() else {
            return None;
        };
        let base = static_exact_machine_integer(base)?;
        let exponent = u32::try_from(static_exact_machine_integer(exponent)?).ok()?;
        return base.checked_pow(exponent);
    }

    None
}

fn static_integer_range_count(start: i128, end: i128, step: i128) -> Option<usize> {
    if step == 0 {
        return None;
    }

    if step > 0 {
        if start > end {
            return Some(0);
        }
    } else if start < end {
        return Some(0);
    }

    let steps = start.abs_diff(end) / step.unsigned_abs();
    usize::try_from(steps).ok()?.checked_add(1)
}

fn static_iterator_value_is_stable(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Atom(Atom::Integer(_) | Atom::Rational(_) | Atom::String(_))
    ) || static_exact_machine_integer(expr).is_some()
}

pub(crate) fn eval_abort(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Abort, head, args)
}

pub(crate) fn eval_exit(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Exit, head, args)
}

pub(crate) fn eval_check_abort(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::CheckAbort, head, args)
}

pub(crate) fn eval_abort_protect(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::AbortProtect, head, args)
}

pub(crate) fn eval_with_cleanup(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::WithCleanup, head, args)
}

pub(crate) fn eval_failsafe(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Failsafe, head, args)
}

pub(crate) fn eval_assert(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Assert, head, args)
}

pub(crate) fn eval_enclose(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Enclose, head, args)
}

pub(crate) fn eval_confirm(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Confirm, head, args)
}

pub(crate) fn eval_confirm_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ConfirmBy, head, args)
}

pub(crate) fn eval_confirm_match(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ConfirmMatch, head, args)
}

pub(crate) fn eval_confirm_quiet(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ConfirmQuiet, head, args)
}

pub(crate) fn eval_confirm_assert(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ConfirmAssert, head, args)
}

pub(crate) fn eval_return(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Return, head, args)
}

pub(crate) fn eval_break(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Break, head, args)
}

pub(crate) fn eval_continue(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Continue, head, args)
}

pub(crate) fn eval_throw(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Throw, head, args)
}

pub(crate) fn eval_catch(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Catch, head, args)
}

pub(crate) fn eval_catch_exceptions(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::CatchExceptions, head, args)
}

pub(crate) fn eval_throw_exception(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ThrowException, head, args)
}

pub(crate) fn eval_register_exception_type(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(
        session,
        ControlDataBuiltin::RegisterExceptionType,
        head,
        args,
    )
}

pub(crate) fn eval_exception_types(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ExceptionTypes, head, args)
}

pub(crate) fn eval_failure(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Failure, head, args)
}

pub(crate) fn eval_failure_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::FailureQ, head, args)
}

pub(crate) fn eval_success(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Success, head, args)
}

pub(crate) fn eval_terminated_evaluation(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(
        session,
        ControlDataBuiltin::TerminatedEvaluation,
        head,
        args,
    )
}
