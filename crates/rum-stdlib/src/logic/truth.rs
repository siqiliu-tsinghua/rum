mod resume_frames;
use resume_frames::*;

use rum_core::SharedArgs;
use std::collections::HashSet;

use super::shared::{negated_relation_head_name, predicate_bool_expr};
use super::suspended_state::PersistentLogicSequence;
use rum_core::{Expr, heads};
use rum_runtime::kernel_api::{
    EvalError, Session, is_system_symbol_name, system_expr, truthiness_symbol,
};

pub(crate) fn eval_and(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    resume_and_from_index(session, args.clone(), 0, Vec::new())
}

pub(crate) fn eval_or(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    resume_or_from_index(session, args.clone(), 0, Vec::new())
}

pub(crate) fn eval_nand(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    resume_nand_from_index(session, args.clone(), 0, Vec::new())
}

pub(crate) fn eval_nor(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    resume_nor_from_index(session, args.clone(), 0, Vec::new())
}

pub(crate) fn eval_xor(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    const INDEX_THRESHOLD: usize = 16;

    let mut invert = false;
    if args.len() <= INDEX_THRESHOLD {
        let mut residual = Vec::new();

        for arg in args {
            match truthiness_symbol(arg) {
                Some(true) => invert = !invert,
                Some(false) => {}
                None => toggle_residual_expr(&mut residual, arg.clone()),
            }
        }

        return Ok(finalize_xor(head, residual, invert));
    }

    let mut odd = HashSet::with_capacity(args.len());
    for arg in args {
        match truthiness_symbol(arg) {
            Some(true) => invert = !invert,
            Some(false) => {}
            None => {
                if !odd.insert(arg) {
                    odd.remove(arg);
                }
            }
        }
    }

    let mut residual = Vec::with_capacity(odd.len());
    for arg in args.iter().rev() {
        if odd.remove(arg) {
            residual.push(arg.clone());
        }
    }
    residual.reverse();

    Ok(finalize_xor(head, residual, invert))
}

pub(crate) fn eval_equivalent(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    const INDEX_THRESHOLD: usize = 16;

    let mut saw_true = false;
    let mut saw_false = false;
    let mut residual = Vec::new();

    if args.len() <= INDEX_THRESHOLD {
        for arg in args {
            match truthiness_symbol(arg) {
                Some(true) => saw_true = true,
                Some(false) => saw_false = true,
                None => push_unique_residual_expr(&mut residual, arg.clone()),
            }
        }

        return Ok(finalize_equivalent(head, residual, saw_true, saw_false));
    }

    let mut seen = HashSet::with_capacity(args.len());
    for arg in args {
        match truthiness_symbol(arg) {
            Some(true) => saw_true = true,
            Some(false) => saw_false = true,
            None if seen.insert(arg) => residual.push(arg.clone()),
            None => {}
        }
    }

    Ok(finalize_equivalent(head, residual, saw_true, saw_false))
}

pub(crate) fn eval_implies(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [lhs, rhs] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Implies", args, 2));
    };

    if lhs == rhs {
        return Ok(system_expr(heads::structure::TRUE));
    }

    Ok(match (truthiness_symbol(lhs), truthiness_symbol(rhs)) {
        (Some(false), _) | (_, Some(true)) => system_expr(heads::structure::TRUE),
        (Some(true), _) => rhs.clone(),
        (_, Some(false)) => negate_logical_expr(lhs.clone()),
        _ => Expr::call(head.clone(), args.to_vec()),
    })
}

pub(crate) fn eval_not(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Not", args, 1));
    };

    Ok(negate_logical_expr(arg.clone()))
}

fn resume_and_from_index(
    session: &mut Session,
    args: SharedArgs,
    mut index: usize,
    mut residual: Vec<Expr>,
) -> Result<Expr, EvalError> {
    while index < args.len() {
        let arg = args[index].clone();
        let value = if let Some(truth) = truthiness_symbol(&arg) {
            predicate_bool_expr(truth)
        } else {
            match session.evaluate(arg) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let residual = PersistentLogicSequence::freeze(residual);
                    ResumeAndFromIndexSuspensionFrame { args, index, residual }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        };

        match truthiness_symbol(&value) {
            Some(true) => {}
            Some(false) => return Ok(system_expr(heads::structure::FALSE)),
            None => extend_flat_logical_residual(&mut residual, value, "And"),
        }
        index += 1;
    }

    Ok(finalize_logical_sequence("And", residual, true))
}

fn resume_suspended_and_from_index(
    session: &mut Session,
    args: SharedArgs,
    mut index: usize,
    mut residual: PersistentLogicSequence<Expr>,
) -> Result<Expr, EvalError> {
    while index < args.len() {
        let arg = args[index].clone();
        let value = if let Some(truth) = truthiness_symbol(&arg) {
            predicate_bool_expr(truth)
        } else {
            match session.evaluate(arg) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeSuspendedAndFromIndexSuspensionFrame { args, index, residual },
                    );
                }
                Err(error) => return Err(error),
            }
        };
        match truthiness_symbol(&value) {
            Some(true) => {}
            Some(false) => return Ok(system_expr(heads::structure::FALSE)),
            None => residual = append_flat_logical_residual(residual, value, "And"),
        }
        index += 1;
    }
    Ok(finalize_logical_sequence("And", residual.to_vec(), true))
}

fn resume_suspended_and_with_value(
    session: &mut Session,
    args: SharedArgs,
    index: usize,
    residual: PersistentLogicSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    match truthiness_symbol(&value) {
        Some(true) => resume_suspended_and_from_index(session, args, index + 1, residual),
        Some(false) => Ok(system_expr(heads::structure::FALSE)),
        None => resume_suspended_and_from_index(
            session,
            args,
            index + 1,
            append_flat_logical_residual(residual, value, "And"),
        ),
    }
}

fn resume_or_from_index(
    session: &mut Session,
    args: SharedArgs,
    mut index: usize,
    mut residual: Vec<Expr>,
) -> Result<Expr, EvalError> {
    while index < args.len() {
        let arg = args[index].clone();
        let value = if let Some(truth) = truthiness_symbol(&arg) {
            predicate_bool_expr(truth)
        } else {
            match session.evaluate(arg) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let residual = PersistentLogicSequence::freeze(residual);
                    ResumeOrFromIndexSuspensionFrame { args, index, residual }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        };

        match truthiness_symbol(&value) {
            Some(true) => return Ok(system_expr(heads::structure::TRUE)),
            Some(false) => {}
            None => extend_flat_logical_residual(&mut residual, value, "Or"),
        }
        index += 1;
    }

    Ok(finalize_logical_sequence("Or", residual, false))
}

fn resume_suspended_or_from_index(
    session: &mut Session,
    args: SharedArgs,
    mut index: usize,
    mut residual: PersistentLogicSequence<Expr>,
) -> Result<Expr, EvalError> {
    while index < args.len() {
        let arg = args[index].clone();
        let value = if let Some(truth) = truthiness_symbol(&arg) {
            predicate_bool_expr(truth)
        } else {
            match session.evaluate(arg) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeSuspendedOrFromIndexSuspensionFrame { args, index, residual },
                    );
                }
                Err(error) => return Err(error),
            }
        };
        match truthiness_symbol(&value) {
            Some(true) => return Ok(system_expr(heads::structure::TRUE)),
            Some(false) => {}
            None => residual = append_flat_logical_residual(residual, value, "Or"),
        }
        index += 1;
    }
    Ok(finalize_logical_sequence("Or", residual.to_vec(), false))
}

fn resume_suspended_or_with_value(
    session: &mut Session,
    args: SharedArgs,
    index: usize,
    residual: PersistentLogicSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    match truthiness_symbol(&value) {
        Some(true) => Ok(system_expr(heads::structure::TRUE)),
        Some(false) => resume_suspended_or_from_index(session, args, index + 1, residual),
        None => resume_suspended_or_from_index(
            session,
            args,
            index + 1,
            append_flat_logical_residual(residual, value, "Or"),
        ),
    }
}

fn resume_nand_from_index(
    session: &mut Session,
    args: SharedArgs,
    mut index: usize,
    mut residual: Vec<Expr>,
) -> Result<Expr, EvalError> {
    while index < args.len() {
        let arg = args[index].clone();
        let value = if let Some(truth) = truthiness_symbol(&arg) {
            predicate_bool_expr(truth)
        } else {
            match session.evaluate(arg) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let residual = PersistentLogicSequence::freeze(residual);
                    ResumeNandFromIndexSuspensionFrame { args, index, residual }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        };

        match truthiness_symbol(&value) {
            Some(false) => return Ok(system_expr(heads::structure::TRUE)),
            Some(true) => {}
            None => residual.push(value),
        }
        index += 1;
    }

    Ok(finalize_negated_logical_sequence("Nand", residual, false))
}

fn resume_suspended_nand_from_index(
    session: &mut Session,
    args: SharedArgs,
    mut index: usize,
    mut residual: PersistentLogicSequence<Expr>,
) -> Result<Expr, EvalError> {
    while index < args.len() {
        let arg = args[index].clone();
        let value = if let Some(truth) = truthiness_symbol(&arg) {
            predicate_bool_expr(truth)
        } else {
            match session.evaluate(arg) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeSuspendedNandFromIndexSuspensionFrame { args, index, residual },
                    );
                }
                Err(error) => return Err(error),
            }
        };
        match truthiness_symbol(&value) {
            Some(false) => return Ok(system_expr(heads::structure::TRUE)),
            Some(true) => {}
            None => residual = residual.appended(value),
        }
        index += 1;
    }
    Ok(finalize_negated_logical_sequence(
        "Nand",
        residual.to_vec(),
        false,
    ))
}

fn resume_suspended_nand_with_value(
    session: &mut Session,
    args: SharedArgs,
    index: usize,
    residual: PersistentLogicSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    match truthiness_symbol(&value) {
        Some(false) => Ok(system_expr(heads::structure::TRUE)),
        Some(true) => resume_suspended_nand_from_index(session, args, index + 1, residual),
        None => {
            resume_suspended_nand_from_index(session, args, index + 1, residual.appended(value))
        }
    }
}

fn resume_nor_from_index(
    session: &mut Session,
    args: SharedArgs,
    mut index: usize,
    mut residual: Vec<Expr>,
) -> Result<Expr, EvalError> {
    while index < args.len() {
        let arg = args[index].clone();
        let value = if let Some(truth) = truthiness_symbol(&arg) {
            predicate_bool_expr(truth)
        } else {
            match session.evaluate(arg) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let residual = PersistentLogicSequence::freeze(residual);
                    ResumeNorFromIndexSuspensionFrame { args, index, residual }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        };

        match truthiness_symbol(&value) {
            Some(true) => return Ok(system_expr(heads::structure::FALSE)),
            Some(false) => {}
            None => residual.push(value),
        }
        index += 1;
    }

    Ok(finalize_negated_logical_sequence("Nor", residual, true))
}

fn resume_suspended_nor_from_index(
    session: &mut Session,
    args: SharedArgs,
    mut index: usize,
    mut residual: PersistentLogicSequence<Expr>,
) -> Result<Expr, EvalError> {
    while index < args.len() {
        let arg = args[index].clone();
        let value = if let Some(truth) = truthiness_symbol(&arg) {
            predicate_bool_expr(truth)
        } else {
            match session.evaluate(arg) {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeSuspendedNorFromIndexSuspensionFrame { args, index, residual },
                    );
                }
                Err(error) => return Err(error),
            }
        };
        match truthiness_symbol(&value) {
            Some(true) => return Ok(system_expr(heads::structure::FALSE)),
            Some(false) => {}
            None => residual = residual.appended(value),
        }
        index += 1;
    }
    Ok(finalize_negated_logical_sequence(
        "Nor",
        residual.to_vec(),
        true,
    ))
}

fn resume_suspended_nor_with_value(
    session: &mut Session,
    args: SharedArgs,
    index: usize,
    residual: PersistentLogicSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    match truthiness_symbol(&value) {
        Some(true) => Ok(system_expr(heads::structure::FALSE)),
        Some(false) => resume_suspended_nor_from_index(session, args, index + 1, residual),
        None => resume_suspended_nor_from_index(session, args, index + 1, residual.appended(value)),
    }
}

fn append_flat_logical_residual(
    mut residual: PersistentLogicSequence<Expr>,
    value: Expr,
    head_name: &str,
) -> PersistentLogicSequence<Expr> {
    match value {
        Expr::Call { head, args, .. } if is_system_symbol_name(&head, head_name) => {
            for value in args {
                residual = residual.appended(value);
            }
            residual
        }
        other => residual.appended(other),
    }
}

fn extend_flat_logical_residual(residual: &mut Vec<Expr>, value: Expr, head_name: &str) {
    match value {
        Expr::Call { head, args, .. } if is_system_symbol_name(&head, head_name) => {
            residual.extend(args);
        }
        other => residual.push(other),
    }
}

fn toggle_residual_expr(residual: &mut Vec<Expr>, value: Expr) {
    if let Some(index) = residual.iter().position(|existing| existing == &value) {
        residual.remove(index);
    } else {
        residual.push(value);
    }
}

fn push_unique_residual_expr(residual: &mut Vec<Expr>, value: Expr) {
    if !residual.iter().any(|existing| existing == &value) {
        residual.push(value);
    }
}

fn finalize_xor(head: &Expr, mut residual: Vec<Expr>, invert: bool) -> Expr {
    let expr = match residual.len() {
        0 => predicate_bool_expr(invert),
        1 => residual.pop().expect("single residual value"),
        _ => Expr::call(head.clone(), residual),
    };

    if invert && truthiness_symbol(&expr).is_none() {
        negate_logical_expr(expr)
    } else {
        expr
    }
}

fn finalize_equivalent(head: &Expr, residual: Vec<Expr>, saw_true: bool, saw_false: bool) -> Expr {
    if saw_true && saw_false {
        return predicate_bool_expr(false);
    }

    if saw_true {
        return finalize_equivalent_constant_side(residual, false);
    }

    if saw_false {
        return finalize_equivalent_constant_side(residual, true);
    }

    match residual.len() {
        0 | 1 => predicate_bool_expr(true),
        _ => Expr::call(head.clone(), residual),
    }
}

fn finalize_equivalent_constant_side(mut residual: Vec<Expr>, negate_residuals: bool) -> Expr {
    match residual.len() {
        0 => predicate_bool_expr(true),
        1 => {
            let expr = residual.pop().expect("single residual value");
            if negate_residuals {
                negate_logical_expr(expr)
            } else {
                expr
            }
        }
        _ => {
            let args = if negate_residuals {
                residual.into_iter().map(negate_logical_expr).collect()
            } else {
                residual
            };
            Expr::call(system_expr("And"), args)
        }
    }
}

fn negate_logical_expr(expr: Expr) -> Expr {
    match truthiness_symbol(&expr) {
        Some(truth) => return predicate_bool_expr(!truth),
        None => {}
    }

    if let Expr::Call { head, args, .. } = expr {
        if is_system_symbol_name(&head, "Not") && args.len() == 1 {
            return args.into_iter().next().expect("single Not argument");
        }

        if args.len() == 2 {
            if let Some(opposite) = negated_relation_head_name(&head) {
                return Expr::call_with_shared_args(system_expr(opposite), args);
            }
        }

        return Expr::call(system_expr("Not"), vec![Expr::call_shared(head, args)]);
    }

    Expr::call(system_expr("Not"), vec![expr])
}

fn finalize_logical_sequence(head_name: &str, mut residual: Vec<Expr>, identity: bool) -> Expr {
    match residual.len() {
        0 => predicate_bool_expr(identity),
        1 => residual.pop().expect("single residual value"),
        _ => Expr::call(system_expr(head_name), residual),
    }
}

fn finalize_negated_logical_sequence(
    head_name: &str,
    mut residual: Vec<Expr>,
    empty_result: bool,
) -> Expr {
    match residual.len() {
        0 => predicate_bool_expr(empty_result),
        1 => negate_logical_expr(residual.pop().expect("single residual value")),
        _ => Expr::call(system_expr(head_name), residual),
    }
}
