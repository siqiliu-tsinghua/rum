//! Private Rule boundary. A clone is owned but unleased until its actual
//! occurrence is charged. No caller can extract children or a live result.

use super::lease::Mode;
use super::retained::{expr_bytes, scoped, values_bytes};
use super::*;
use std::marker::PhantomData;

#[cfg(test)]
#[path = "tests/rule.rs"]
mod tests;

/// B04 handoff for C02's non-delayed branches. There is no vector, frame or
/// weight walk in the unarmed specialization. Publication ends the scope.
pub(in crate::runtime) fn clone_result(
    session: &mut Session,
    source: &Expr,
) -> Result<Expr, EvalError> {
    if session.current_memory_limit.is_some() {
        scoped::<AccountingShadow, _>(session, |cx, empty| clone_owned(cx, empty, source))
    } else {
        scoped::<Unarmed, _>(session, |cx, empty| clone_owned(cx, empty, source))
    }
}

fn clone_owned<'id, M: Mode>(
    cx: RetentionContext<'_, 'id, M>,
    empty: RetainedValues<'id, M>,
    source: &Expr,
) -> Result<RetainedResult<'id, M>, EvalError> {
    let mut receipt = empty.receipt;
    let value = source.clone();
    M::resize(&mut receipt, || Some(expr_bytes(&value)));
    if M::ARMED {
        cx.session
            .enforce_active_memory_limit_for_projected_bytes(0)?;
    }
    Ok(RetainedResult {
        outcome: Ok(value),
        receipt: Some(receipt),
        brand: PhantomData,
    })
}

/// Clone, adopt, then grow. The source receipt covers the actual clone even
/// while reserve is checking the destination's proposed capacity.
pub(super) fn clone_child<'id, M: Mode>(
    cx: &mut RetentionContext<'_, 'id, M>,
    children: &mut RetainedValues<'id, M>,
    source: &Expr,
) -> Result<(), EvalError> {
    let mut receipt = M::receipt(cx.session);
    let value = source.clone();
    M::resize(&mut receipt, || Some(expr_bytes(&value)));
    cx.reserve(children, 1)?;
    children.values.push(value);
    M::resize(&mut children.receipt, || values_bytes(&children.values));
    drop(receipt);
    if M::ARMED {
        cx.session
            .enforce_active_memory_limit_for_projected_bytes(0)?;
    }
    Ok(())
}

/// B10 is limited to this adapter: one adopted RHS, a borrowed name, the exact
/// System`Rule head (B11), checked shell/capacity, and an outgoing receipt.
pub(super) fn finish_rule<'id, M: Mode>(
    mut cx: RetentionContext<'_, 'id, M>,
    mut children: RetainedValues<'id, M>,
    name: &Expr,
) -> Result<RetainedResult<'id, M>, EvalError> {
    assert_eq!(children.values.len(), 1, "one adopted rule value");
    clone_child(&mut cx, &mut children, name)?;
    children.values.swap(0, 1);
    let mut head_receipt = M::receipt(cx.session);
    let head = system_expr("Rule");
    M::resize(&mut head_receipt, || Some(expr_bytes(&head)));
    M::resize(&mut children.receipt, || {
        values_bytes(&children.values)?
            .checked_add(expr_bytes(&head))?
            .checked_add(size_of::<Expr>())
    });
    drop(head_receipt);
    if M::ARMED {
        cx.session
            .enforce_active_memory_limit_for_projected_bytes(0)?;
    }
    let value = Expr::call(head, children.values);
    Ok(RetainedResult {
        outcome: Ok(value),
        receipt: Some(children.receipt),
        brand: PhantomData,
    })
}

#[inline]
pub(in crate::runtime) fn evaluate_rule(
    session: &mut Session,
    name: &Expr,
    rhs: &Expr,
) -> Result<Expr, EvalError> {
    if session.current_memory_limit.is_some() {
        evaluate_in::<AccountingShadow>(session, name, rhs)
    } else {
        evaluate_unarmed(session, name, rhs, None)
    }
}

fn evaluate_in<M: Mode>(session: &mut Session, name: &Expr, rhs: &Expr) -> Result<Expr, EvalError> {
    scoped::<M, _>(session, |cx, empty| match cx.evaluate(empty, rhs.clone()) {
        Ok(value) => value.into_rule(name),
        Err(failure) => Ok(failure.suspend_rule(name)),
    })
}

/// The callback starts with an empty owner. Materialize receipts/builders only
/// if it leaves a limit armed or suspends; a terminal unarmed success has no
/// intervening guest entry and can construct its two children directly.
#[inline(always)]
pub(super) fn evaluate_unarmed(
    session: &mut Session,
    name: &Expr,
    rhs: &Expr,
    previous: Option<&Expr>,
) -> Result<Expr, EvalError> {
    match session.evaluate(rhs.clone()) {
        Ok(value) if session.current_memory_limit.is_none() => {
            let rule = system_expr("Rule");
            let children = vec![name.clone(), value];
            // Borrowing the first completed option adds no owner over the
            // callback. Only after the unarmed check may its exact canonical
            // head be shared by this new occurrence; arbitrary heads decline.
            if let Some(Expr::Call { head, .. }) = previous {
                if **head == rule {
                    return Ok(Expr::call_shared(head.clone(), SharedArgs::new(children)));
                }
            }
            Ok(Expr::call(rule, children))
        }
        Ok(value) => finish_armed(session, name, value),
        Err(error) => fail_unarmed(session, name, error),
    }
}

#[cold]
fn fail_unarmed(session: &mut Session, name: &Expr, error: EvalError) -> Result<Expr, EvalError> {
    if session.current_memory_limit.is_some() {
        return Err(suspend_armed(session, error, name));
    }
    scoped::<Unarmed, _>(session, |cx, empty| {
        match cx.adopt_outcome(empty, Err(error)) {
            Err(failure) => Ok(failure.suspend_rule(name)),
            Ok(_) => unreachable!("exceptional outcome stays exceptional"),
        }
    })
}

pub(in crate::runtime) fn resume_rule(
    session: &mut Session,
    frozen: &FrozenValues,
    outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    if session.current_memory_limit.is_some() {
        resume_in::<AccountingShadow>(session, frozen, outcome)
    } else {
        resume_in::<Unarmed>(session, frozen, outcome)
    }
}

fn resume_in<M: Mode>(
    session: &mut Session,
    frozen: &FrozenValues,
    outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    scoped::<M, _>(session, |cx, empty| {
        match cx.adopt_outcome(empty, outcome) {
            Ok(value) => value.into_rule(frozen.rule_name()),
            Err(failure) => Ok(failure.finish()),
        }
    })
}

/// Terminal nested scope for a callback that left a new limit active. Its
/// outgoing W is handed directly to the enclosing scope's terminal result.
pub(super) fn finish_armed(
    session: &mut Session,
    name: &Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    scoped::<AccountingShadow, _>(session, |cx, empty| {
        match cx.adopt_outcome(empty, Ok(value)) {
            Ok(value) => value.into_rule(name),
            Err(failure) => Ok(failure.finish()),
        }
    })
}

/// InputRequired can leave MemoryConstrained's limit active across suspension.
/// Re-select before cloning the name, not just when freezing the finished Vec.
pub(super) fn suspend_armed(session: &mut Session, error: EvalError, name: &Expr) -> EvalError {
    scoped::<AccountingShadow, _>(session, |cx, empty| {
        match cx.adopt_outcome(empty, Err(error)) {
            Err(failure) => Ok(failure.suspend_rule(name)),
            Ok(_) => unreachable!("exceptional outcome stays exceptional"),
        }
    })
    .expect_err("suspension boundary cannot produce a successful expression")
}
