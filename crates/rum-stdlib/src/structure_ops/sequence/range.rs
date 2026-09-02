use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_range(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let Some((start, end, step)) = range_spec(args) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let Some(count) = integer_range_count(&start, &end, &step) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    enforce_range_memory_constraint(session, &count)?;
    enforce_generated_element_limit(session, &count)?;

    let count = bigint_count_to_usize(&count)
        .expect("generated element limit precheck keeps Range count representable");
    Ok(Expr::call(
        system_expr(heads::structure::LIST),
        integer_range_values(start, step, count),
    ))
}

fn enforce_range_memory_constraint(session: &Session, count: &BigInt) -> Result<(), EvalError> {
    let Some(limit) = session.active_memory_limit() else {
        // Keep the BigInt conversion off the unarmed Range path while still
        // passing through the projection choke point for the structural
        // Preflight falsifier.
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    };
    if count > &BigInt::from(usize::MAX) {
        return Err(EvalError::MemoryConstraintExceeded { limit });
    }

    let count = bigint_count_to_usize(count)
        .expect("Range count was checked against usize::MAX before conversion");
    session.enforce_active_memory_limit_for_projected_bytes(range_memory_proxy_bytes(count))
}

fn enforce_generated_element_limit(session: &Session, count: &BigInt) -> Result<(), EvalError> {
    let limit = session.resource_limits().generated_element_limit;
    if count > &BigInt::from(limit) {
        return Err(EvalError::GeneratedElementLimitExceeded { limit });
    }

    Ok(())
}

fn range_spec(args: &[Expr]) -> Option<(BigInt, BigInt, BigInt)> {
    match args {
        [end] => Some((
            BigInt::from(1_u8),
            expr_as_exact_integer(end)?,
            BigInt::from(1_u8),
        )),
        [start, end] => Some((
            expr_as_exact_integer(start)?,
            expr_as_exact_integer(end)?,
            BigInt::from(1_u8),
        )),
        [start, end, step] => Some((
            expr_as_exact_integer(start)?,
            expr_as_exact_integer(end)?,
            expr_as_exact_integer(step)?,
        )),
        _ => None,
    }
}

fn integer_range_count(start: &BigInt, end: &BigInt, step: &BigInt) -> Option<BigInt> {
    let zero = BigInt::from(0_u8);
    if step == &zero {
        return None;
    }

    let one = BigInt::from(1_u8);
    if step > &zero {
        if start > end {
            return Some(zero);
        }
        return Some(((end - start) / step) + one);
    }

    if start < end {
        return Some(zero);
    }

    Some(((start - end) / -step) + one)
}

fn integer_range_values(start: BigInt, step: BigInt, count: usize) -> Vec<Expr> {
    if let Some(values) = small_integer_range_values(&start, &step, count) {
        return values;
    }

    let mut values = Vec::with_capacity(count);
    let mut current = start;

    for _ in 0..count {
        values.push(Expr::integer(current.clone()));
        current += step.clone();
    }

    values
}

fn small_integer_range_values(start: &BigInt, step: &BigInt, count: usize) -> Option<Vec<Expr>> {
    let mut current = i64::try_from(start).ok()?;
    let step = i64::try_from(step).ok()?;
    let mut values = Vec::with_capacity(count);

    for index in 0..count {
        values.push(Expr::small_integer(current));
        if index + 1 < count {
            current = current.checked_add(step)?;
        }
    }

    Some(values)
}

fn bigint_count_to_usize(count: &BigInt) -> Option<usize> {
    count.to_string().parse().ok()
}

fn range_memory_proxy_bytes(count: usize) -> usize {
    std::mem::size_of::<Expr>().saturating_add(count.saturating_mul(std::mem::size_of::<Expr>()))
}
