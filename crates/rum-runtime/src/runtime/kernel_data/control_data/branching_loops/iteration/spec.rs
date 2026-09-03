use super::*;

pub(in crate::runtime::kernel_data::control_data::branching_loops) fn evaluate_do_spec_expr(
    session: &mut Session,
    raw_spec: Expr,
) -> Result<Expr, EvalError> {
    match raw_spec {
        Expr::Call { head, args, .. } if is_system_symbol_name(&head, heads::structure::LIST) => {
            let evaluated = if let Some((first, rest)) = args.split_first() {
                if expr_as_symbol(first).is_some() {
                    let mut normalized = Vec::with_capacity(args.len());
                    normalized.push(Expr::call(system_expr("Hold"), vec![first.clone()]));
                    normalized.extend(rest.iter().cloned());
                    Expr::call(system_expr("List"), normalized)
                } else {
                    Expr::call_with_shared_args(system_expr("List"), args)
                }
            } else {
                Expr::call_with_shared_args(system_expr("List"), args)
            };

            session.evaluate(evaluated)
        }
        other => session.evaluate(other),
    }
}

pub(in crate::runtime::kernel_data::control_data::branching_loops) fn parse_do_spec(
    session: &Session,
    expr: &Expr,
    consumer: IterationConsumer,
) -> Result<Option<DoSpec>, EvalError> {
    if let Some(count) = expr_as_non_negative_bigint(expr) {
        if !consumer.accepts_bare_count() {
            return Ok(None);
        }
        return Ok(Some(DoSpec::Count(checked_iteration_count(
            session, &count, consumer,
        )?)));
    }

    let Expr::Call { head, args, .. } = expr else {
        return Ok(None);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Ok(None);
    }

    Ok(match args.as_slice() {
        [count_expr] => {
            let Some(count) = expr_as_non_negative_bigint(count_expr) else {
                return Ok(None);
            };
            Some(DoSpec::Count(checked_iteration_count(
                session, &count, consumer,
            )?))
        }
        [symbol_expr, values_expr] => {
            let Some(symbol) = parse_held_loop_symbol(symbol_expr) else {
                return Ok(None);
            };

            if let Expr::Call { head, args, .. } = values_expr {
                if is_system_symbol_name(head, heads::structure::LIST) {
                    checked_iteration_count(session, &BigInt::from(args.len()), consumer)?;
                    return Ok(Some(DoSpec::Iterator {
                        symbol,
                        values: args.to_vec(),
                    }));
                }
            }

            let Some(end) = expr_as_signed_bigint(values_expr) else {
                return Ok(None);
            };
            let values = checked_integer_range_values(
                session,
                BigInt::from(1_u8),
                end,
                BigInt::from(1_u8),
                consumer,
            )?;
            Some(DoSpec::Iterator { symbol, values })
        }
        [symbol_expr, start_expr, end_expr] => {
            let (Some(symbol), Some(start), Some(end)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
            ) else {
                return Ok(None);
            };
            let values =
                checked_integer_range_values(session, start, end, BigInt::from(1_u8), consumer)?;
            Some(DoSpec::Iterator { symbol, values })
        }
        [symbol_expr, start_expr, end_expr, step_expr] => {
            let (Some(symbol), Some(start), Some(end), Some(step)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
                expr_as_signed_bigint(step_expr),
            ) else {
                return Ok(None);
            };
            if step == BigInt::from(0_u8) {
                return Ok(None);
            }

            let values = checked_integer_range_values(session, start, end, step, consumer)?;
            Some(DoSpec::Iterator { symbol, values })
        }
        _ => None,
    })
}

pub(in crate::runtime::kernel_data::control_data::branching_loops) fn build_iteration_body_or_nested(
    args: &[Expr],
    spec_index: usize,
    nested_head: &str,
) -> Expr {
    if spec_index + 2 >= args.len() {
        args[0].clone()
    } else {
        let mut nested_args = Vec::with_capacity(args.len() - spec_index - 1);
        nested_args.push(args[0].clone());
        nested_args.extend(args[(spec_index + 2)..].iter().cloned());
        Expr::call(system_expr(nested_head), nested_args)
    }
}

pub(super) fn parse_do_spec_iteration_count(
    session: &Session,
    expr: &Expr,
    consumer: IterationConsumer,
) -> Result<Option<usize>, EvalError> {
    if let Some(count) = expr_as_non_negative_bigint(expr) {
        return Ok(Some(checked_iteration_count(session, &count, consumer)?));
    }

    let Expr::Call { head, args, .. } = expr else {
        return Ok(None);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Ok(None);
    }

    Ok(match args.as_slice() {
        [count_expr] => {
            let Some(count) = expr_as_non_negative_bigint(count_expr) else {
                return Ok(None);
            };
            Some(checked_iteration_count(session, &count, consumer)?)
        }
        [symbol_expr, values_expr] => {
            let Some(_symbol) = parse_held_loop_symbol(symbol_expr) else {
                return Ok(None);
            };

            if let Expr::Call { head, args, .. } = values_expr {
                if is_system_symbol_name(head, heads::structure::LIST) {
                    let count = BigInt::from(args.len());
                    return Ok(Some(checked_iteration_count(session, &count, consumer)?));
                }
            }

            let Some(end) = expr_as_signed_bigint(values_expr) else {
                return Ok(None);
            };
            checked_integer_range_iteration_count(
                session,
                &BigInt::from(1_u8),
                &end,
                &BigInt::from(1_u8),
                consumer,
            )?
        }
        [symbol_expr, start_expr, end_expr] => {
            let (Some(_symbol), Some(start), Some(end)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
            ) else {
                return Ok(None);
            };
            checked_integer_range_iteration_count(
                session,
                &start,
                &end,
                &BigInt::from(1_u8),
                consumer,
            )?
        }
        [symbol_expr, start_expr, end_expr, step_expr] => {
            let (Some(_symbol), Some(start), Some(end), Some(step)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
                expr_as_signed_bigint(step_expr),
            ) else {
                return Ok(None);
            };
            if step == BigInt::from(0_u8) {
                return Ok(None);
            }
            checked_integer_range_iteration_count(session, &start, &end, &step, consumer)?
        }
        _ => None,
    })
}

fn parse_held_loop_symbol(expr: &Expr) -> Option<Symbol> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "Hold") && args.len() == 1 => {
            expr_as_symbol(&args[0]).cloned()
        }
        _ => expr_as_symbol(expr).cloned(),
    }
}

fn expr_as_signed_bigint(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.to_bigint()),
        Expr::Call { head, args, .. }
            if is_system_symbol_name(head, "Times") && args.len() == 2 =>
        {
            if is_negative_one_expr(&args[0]) {
                expr_as_exact_integer(&args[1]).map(|value| -value)
            } else if is_negative_one_expr(&args[1]) {
                expr_as_exact_integer(&args[0]).map(|value| -value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn expr_as_non_negative_bigint(expr: &Expr) -> Option<BigInt> {
    let value = expr_as_exact_integer(expr)?;
    (value >= BigInt::from(0_u8)).then_some(value)
}

fn checked_integer_range_values(
    session: &Session,
    start: BigInt,
    end: BigInt,
    step: BigInt,
    consumer: IterationConsumer,
) -> Result<Vec<Expr>, EvalError> {
    let Some(count) = integer_range_count(&start, &end, &step) else {
        return Ok(Vec::new());
    };
    let count = checked_iteration_count(session, &count, consumer)?;
    Ok(integer_range_values(start, step, count))
}

fn checked_integer_range_iteration_count(
    session: &Session,
    start: &BigInt,
    end: &BigInt,
    step: &BigInt,
    consumer: IterationConsumer,
) -> Result<Option<usize>, EvalError> {
    let Some(count) = integer_range_count(start, end, step) else {
        return Ok(None);
    };
    Ok(Some(checked_iteration_count(session, &count, consumer)?))
}

fn checked_iteration_count(
    session: &Session,
    count: &BigInt,
    consumer: IterationConsumer,
) -> Result<usize, EvalError> {
    let (scoped_product, enclosing_consumer_tag) = session.iteration_product_budget.get();
    let consumer_tag = consumer.product_scope_tag();
    let parent_product = if consumer.inherits_product_from(enclosing_consumer_tag) {
        scoped_product
    } else {
        1
    };
    if parent_product == 1 {
        enforce_iteration_count_limits(session, count, consumer, true)?;
        let count = bigint_count_to_usize(count)
            .expect("resource limit prechecks keep iteration counts representable");
        session.iteration_product_budget.set((count, consumer_tag));
        return Ok(count);
    }

    if matches!(consumer, IterationConsumer::Table) {
        enforce_table_memory_projection(session, count)?;
    }
    let Some(count) = bigint_count_to_usize(count) else {
        let accumulated_count = BigInt::from(parent_product) * count;
        enforce_iteration_count_limits(session, &accumulated_count, consumer, false)?;
        unreachable!("an unrepresentable count must exceed a usize resource limit");
    };
    let Some(accumulated_count) = parent_product.checked_mul(count) else {
        let accumulated_count = BigInt::from(parent_product) * BigInt::from(count);
        enforce_iteration_count_limits(session, &accumulated_count, consumer, false)?;
        unreachable!("an overflowing running product must exceed a usize resource limit");
    };
    enforce_iteration_count_limits_for_count(session, accumulated_count, consumer)?;
    session
        .iteration_product_budget
        .set((accumulated_count, consumer_tag));

    Ok(count)
}

fn enforce_iteration_count_limits_for_count(
    session: &Session,
    count: usize,
    consumer: IterationConsumer,
) -> Result<(), EvalError> {
    if matches!(consumer, IterationConsumer::Table) {
        session.enforce_generated_element_limit_for_count(count)?;
    }
    let limit = session.resource_limits().iteration_work_limit;
    if count > limit {
        return Err(EvalError::IterationWorkLimitExceeded { limit });
    }

    Ok(())
}

fn enforce_table_memory_projection(session: &Session, count: &BigInt) -> Result<(), EvalError> {
    let Some(limit) = session.current_memory_limit else {
        return Ok(());
    };
    if count > &BigInt::from(usize::MAX) {
        return Err(EvalError::MemoryConstraintExceeded { limit });
    }
    let count = bigint_count_to_usize(count)
        .expect("Table count was checked against usize::MAX before conversion");
    session
        .enforce_active_memory_limit_for_projected_bytes(generated_list_memory_proxy_bytes(count))
}

fn enforce_iteration_count_limits(
    session: &Session,
    count: &BigInt,
    consumer: IterationConsumer,
    project_table_memory: bool,
) -> Result<(), EvalError> {
    match consumer {
        IterationConsumer::Do | IterationConsumer::Product | IterationConsumer::Sum => {
            session.enforce_iteration_work_limit_for_big_count(count)?;
        }
        IterationConsumer::Table => {
            if project_table_memory {
                enforce_table_memory_projection(session, count)?;
            }
            session.enforce_generated_element_limit_for_big_count(count)?;
            session.enforce_iteration_work_limit_for_big_count(count)?;
        }
    }

    Ok(())
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

fn generated_list_memory_proxy_bytes(count: usize) -> usize {
    std::mem::size_of::<Expr>().saturating_add(count.saturating_mul(std::mem::size_of::<Expr>()))
}
