//! Source exhaustion, charging and cursor advancement.
use super::state::{AwaitCursor, PullContext, PullSource};
use crate::structure_ops::next_permutation;
use rum_core::Expr;
use rum_runtime::kernel_api::{EvalError, Session, list_items, system_expr};

pub(super) fn pull_source(
    source: &mut PullSource,
    session: &mut Session,
    ctx: &mut PullContext,
) -> Result<Option<Expr>, EvalError> {
    match source {
        PullSource::Range { current, step } => {
            ctx.charge_source_pull()?;
            let value = Expr::rational(current.clone());
            *current = current.add(step);
            Ok(Some(value))
        }
        PullSource::Permutations { current, exhausted } => {
            if *exhausted {
                return Ok(None);
            }
            ctx.charge_source_pull()?;
            let permutation = Expr::call(system_expr("List"), current.clone());
            *exhausted = !next_permutation(current);
            Ok(Some(permutation))
        }
        PullSource::Subsets { elements, k, combo } => {
            if *k > elements.len() {
                return Ok(None);
            }
            ctx.charge_source_pull()?;
            let subset = Expr::call(
                system_expr("List"),
                combo.iter().map(|index| elements[*index].clone()).collect(),
            );
            advance_subset_cursor(elements.len(), k, combo);
            Ok(Some(subset))
        }
        PullSource::Tuples {
            inner_lists,
            digits,
            exhausted,
        } => {
            if *exhausted {
                return Ok(None);
            }
            ctx.charge_source_pull()?;
            let tuple = Expr::call(
                system_expr("List"),
                inner_lists
                    .iter()
                    .zip(digits.iter())
                    .map(|(list, digit)| {
                        list_items(list).expect("tuple factors were validated")[*digit].clone()
                    })
                    .collect(),
            );
            *exhausted = advance_tuple_cursor(inner_lists, digits);
            Ok(Some(tuple))
        }
        PullSource::NestList { function, current } => {
            ctx.charge_source_pull()?;
            let value = current.clone();
            let answer = match session.evaluate(Expr::call(function.clone(), vec![value.clone()])) {
                Ok(answer) => answer,
                Err(error) => {
                    return Err(ctx.await_callback(error, AwaitCursor::Nest { yield_value: value }));
                }
            };
            ctx.observe_cancellation()?;
            *current = answer;
            Ok(Some(value))
        }
    }
}

fn advance_subset_cursor(item_count: usize, k: &mut usize, combo: &mut Vec<usize>) {
    if *k == item_count {
        *k += 1;
        combo.clear();
        return;
    }

    if let Some(index) = (0..combo.len())
        .rev()
        .find(|index| combo[*index] < item_count - *k + *index)
    {
        combo[index] += 1;
        for next in index + 1..combo.len() {
            combo[next] = combo[next - 1] + 1;
        }
    } else {
        *k += 1;
        *combo = (0..*k).collect();
    }
}

fn advance_tuple_cursor(inner_lists: &[Expr], digits: &mut [usize]) -> bool {
    for index in (0..digits.len()).rev() {
        digits[index] += 1;
        if digits[index]
            < list_items(&inner_lists[index])
                .expect("tuple factors were validated")
                .len()
        {
            return false;
        }
        digits[index] = 0;
    }
    true
}
