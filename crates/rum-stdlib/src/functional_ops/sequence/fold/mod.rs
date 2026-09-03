use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod pair;
mod plain;
mod sequence_fold;
mod while_loop;

use pair::apply_fold_pair;
use plain::apply_fold;
use sequence_fold::apply_sequence_fold;
use while_loop::apply_fold_while;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) struct FoldWhileConfig {
    pub(super) test_arity: NestWhileTestArity,
    pub(super) tail_adjustment: isize,
}

pub(crate) fn eval_fold(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator] => Ok(variadic_operator_form("Fold", vec![operator.clone()])),
        [operator, list_expr] => apply_fold(session, operator, None, list_expr, false),
        [operator, initial, list_expr] => {
            apply_fold(session, operator, Some(initial.clone()), list_expr, false)
        }
        _ => Ok(session.invalid_call_with_arg_range("Fold", args, 1, 3)),
    }
}

pub(crate) fn eval_fold_list(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator] => Ok(variadic_operator_form("FoldList", vec![operator.clone()])),
        [operator, list_expr] => apply_fold(session, operator, None, list_expr, true),
        [operator, initial, list_expr] => {
            apply_fold(session, operator, Some(initial.clone()), list_expr, true)
        }
        _ => Ok(session.invalid_call_with_arg_range("FoldList", args, 1, 3)),
    }
}

pub(crate) fn eval_fold_pair(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    match args_slice {
        [operator, list_expr] => {
            apply_fold_pair(session, head, operator, None, list_expr, None, args)
        }
        [operator, initial, list_expr] => apply_fold_pair(
            session,
            head,
            operator,
            Some(initial.clone()),
            list_expr,
            None,
            args,
        ),
        [operator, initial, list_expr, projector] => apply_fold_pair(
            session,
            head,
            operator,
            Some(initial.clone()),
            list_expr,
            Some(projector.clone()),
            args,
        ),
        _ => Ok(session.invalid_call_with_arg_range("FoldPair", args_slice, 2, 4)),
    }
}

pub(crate) fn eval_fold_while(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    match args_slice {
        [operator, test] => Ok(variadic_operator_form_with_suffix(
            "FoldWhile",
            vec![operator.clone()],
            vec![test.clone()],
        )),
        [operator, list_expr, test] => apply_fold_while(
            session, head, operator, None, list_expr, test, None, None, args,
        ),
        [operator, initial, list_expr, test] => apply_fold_while(
            session,
            head,
            operator,
            Some(initial.clone()),
            list_expr,
            test,
            None,
            None,
            args,
        ),
        [operator, initial, list_expr, test, m_expr] => apply_fold_while(
            session,
            head,
            operator,
            Some(initial.clone()),
            list_expr,
            test,
            Some(m_expr),
            None,
            args,
        ),
        [operator, initial, list_expr, test, m_expr, n_expr] => apply_fold_while(
            session,
            head,
            operator,
            Some(initial.clone()),
            list_expr,
            test,
            Some(m_expr),
            Some(n_expr),
            args,
        ),
        _ => Ok(session.invalid_call_with_arg_range("FoldWhile", args_slice, 2, 6)),
    }
}

pub(crate) fn eval_sequence_fold(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    match args_slice {
        [operator, seeds, items] => {
            apply_sequence_fold(session, head, operator, seeds, items, None, false, args)
        }
        [operator, seeds, items, arity_expr] => apply_sequence_fold(
            session,
            head,
            operator,
            seeds,
            items,
            Some(arity_expr),
            false,
            args,
        ),
        _ => Ok(session.invalid_call_with_arg_range("SequenceFold", args_slice, 3, 4)),
    }
}

pub(crate) fn eval_sequence_fold_list(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    match args_slice {
        [operator, seeds, items] => {
            apply_sequence_fold(session, head, operator, seeds, items, None, true, args)
        }
        [operator, seeds, items, arity_expr] => apply_sequence_fold(
            session,
            head,
            operator,
            seeds,
            items,
            Some(arity_expr),
            true,
            args,
        ),
        _ => Ok(session.invalid_call_with_arg_range("SequenceFoldList", args_slice, 3, 4)),
    }
}

pub(super) fn iterable_items(expr: &Expr) -> Option<Vec<Expr>> {
    match expr {
        Expr::Call { head, args: _, .. } if is_system_symbol_name(head, "Association") => Some(
            association_entries_from_expr(expr)?
                .into_iter()
                .map(|(_, value)| value)
                .collect(),
        ),
        Expr::Call { args, .. } => Some(args.to_vec()),
        Expr::Atom(_) => None,
    }
}

pub(super) fn parse_fold_while_config(
    m_expr: Option<&Expr>,
    n_expr: Option<&Expr>,
) -> Option<FoldWhileConfig> {
    let test_arity = match m_expr {
        Some(expr) if is_system_symbol_name(expr, "All") => NestWhileTestArity::All,
        Some(expr) => {
            let count = expr_as_non_negative_usize(expr)?;
            if count == 0 {
                return None;
            }
            NestWhileTestArity::Count(count)
        }
        None => NestWhileTestArity::Count(1),
    };

    let tail_adjustment = match n_expr {
        Some(expr) => expr_as_signed_isize(expr)?,
        None => 0,
    };

    Some(FoldWhileConfig {
        test_arity,
        tail_adjustment,
    })
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FoldWhileConfig>();
};
