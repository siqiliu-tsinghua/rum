use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_join(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.is_empty() {
        return Ok(Expr::call(system_expr("List"), Vec::new()));
    }

    if let Some((join_args, level)) = split_join_level_arg(args) {
        return Ok(eval_join_with_level(session, head, args, join_args, level));
    }

    Ok(eval_join_top_level(session, head, args, args))
}

fn eval_join_with_level(
    session: &mut Session,
    head: &Expr,
    call_args: &[Expr],
    join_args: &[Expr],
    level: usize,
) -> Expr {
    if join_args.len() == 1 {
        return if matches!(join_args[0], Expr::Call { .. }) {
            join_args[0].clone()
        } else {
            Expr::call(head.clone(), call_args.to_vec())
        };
    }

    if level == 1 {
        return eval_join_top_level(session, head, call_args, join_args);
    }

    let Some(expected_heads) = collect_first_join_level_heads(&join_args[0], level) else {
        return invalid_join_normal1_call(session, call_args, level);
    };

    for (index, arg) in join_args.iter().enumerate().skip(1) {
        if let Err(expected_head) = validate_join_level_heads(arg, &expected_heads, 0) {
            return invalid_join_headsd_call(session, call_args, index + 1, &expected_head, level);
        }
    }

    join_level_unchecked(&join_args.iter().collect::<Vec<_>>(), &expected_heads, 0)
}

fn eval_join_top_level(
    session: &mut Session,
    head: &Expr,
    call_args: &[Expr],
    args: &[Expr],
) -> Expr {
    if args.len() == 1 && matches!(args[0], Expr::Atom(_)) {
        return invalid_join_normal_call(session, call_args);
    }

    if matches!(args[0], Expr::Atom(_)) {
        return Expr::call(head.clone(), call_args.to_vec());
    }

    if args.iter().any(|arg| matches!(arg, Expr::Atom(_)))
        && association_entry_forms_from_expr(&args[0]).is_none()
    {
        return Expr::call(head.clone(), call_args.to_vec());
    }

    let association_args = args
        .iter()
        .filter(|arg| association_entry_forms_from_expr(arg).is_some())
        .count();
    if association_args > 0 && association_args != args.len() {
        return invalid_join_incompatible_call(session, call_args);
    }

    if args
        .iter()
        .all(|arg| association_entry_forms_from_expr(arg).is_some())
    {
        let entries = args
            .iter()
            .flat_map(|arg| {
                association_entry_forms_from_expr(arg)
                    .expect("checked all arguments are associations")
            })
            .collect();
        return render_association_entry_forms_preserving_heads(entries);
    }

    let Some((first_head, joined_args)) = join_same_head_args(args) else {
        return Expr::call(head.clone(), call_args.to_vec());
    };

    if let Some((position, actual_head)) = first_mismatched_join_head(args, &first_head) {
        session.push_message(
                "Join",
                "heads",
                format!(
                    "Heads {first_head} and {actual_head} at positions 1 and {position} are expected to be the same."
                ),
            );
        return Expr::call(head.clone(), call_args.to_vec());
    }

    Expr::call(first_head, joined_args)
}

fn invalid_join_normal_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Join",
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr("Join"), args.to_vec())
        ),
    )
}

fn invalid_join_normal1_call(session: &mut Session, args: &[Expr], level: usize) -> Expr {
    session.invalid_call_with_message(
            "Join",
            args,
            "normal1",
            format!(
                "Expression {} at position 1 is expected to have nonatomic subexpression at level {level}.",
                args[0]
            ),
        )
}

fn invalid_join_headsd_call(
    session: &mut Session,
    args: &[Expr],
    position: usize,
    expected_head: &Expr,
    level: usize,
) -> Expr {
    session.invalid_call_with_message(
            "Join",
            args,
            "headsd",
            format!(
                "Expression {} at position {position} is expected to have head {expected_head} for all expressions at level {level}.",
                args[position - 1]
            ),
        )
}

fn invalid_join_incompatible_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Join",
        args,
        "incpt",
        format!(
            "Incompatible elements in {} cannot be joined.",
            Expr::call(system_expr("Join"), args.to_vec())
        ),
    )
}

fn split_join_level_arg(args: &[Expr]) -> Option<(&[Expr], usize)> {
    if args.len() < 2 {
        return None;
    }
    let (level_arg, join_args) = args.split_last()?;
    let level = expr_as_non_negative_usize(level_arg).filter(|level| *level > 0)?;
    Some((join_args, level))
}

fn join_same_head_args(args: &[Expr]) -> Option<(Expr, Vec<Expr>)> {
    let Expr::Call {
        head: first_head, ..
    } = &args[0]
    else {
        return None;
    };

    let mut joined = Vec::new();
    for arg in args {
        let Expr::Call { head, args } = arg else {
            return None;
        };
        if head.as_ref() != first_head.as_ref() {
            continue;
        }
        joined.extend(args.clone());
    }

    Some((first_head.as_ref().clone(), joined))
}

fn first_mismatched_join_head(args: &[Expr], first_head: &Expr) -> Option<(usize, Expr)> {
    args.iter().enumerate().skip(1).find_map(|(index, arg)| {
        let Expr::Call { head, .. } = arg else {
            return None;
        };
        (head.as_ref() != first_head).then(|| (index + 1, head.as_ref().clone()))
    })
}

fn collect_first_join_level_heads(expr: &Expr, level: usize) -> Option<Vec<Expr>> {
    let mut expected_heads = Vec::new();
    collect_first_join_level_heads_at(expr, level, 0, &mut expected_heads).then_some(expected_heads)
}

fn collect_first_join_level_heads_at(
    expr: &Expr,
    level: usize,
    depth: usize,
    expected_heads: &mut Vec<Expr>,
) -> bool {
    let Expr::Call { head, args } = expr else {
        return false;
    };

    if association_entry_forms_from_expr(expr).is_some() && depth + 1 < level {
        return false;
    }

    match expected_heads.get(depth) {
        Some(expected_head) if expected_head != head.as_ref() => return false,
        Some(_) => {}
        None => expected_heads.push(head.as_ref().clone()),
    }

    if depth + 1 == level {
        return true;
    }
    if args.is_empty() {
        return false;
    }
    args.iter()
        .all(|arg| collect_first_join_level_heads_at(arg, level, depth + 1, expected_heads))
}

fn validate_join_level_heads(
    expr: &Expr,
    expected_heads: &[Expr],
    depth: usize,
) -> Result<(), Expr> {
    let expected_head = &expected_heads[depth];
    let Expr::Call { head, args } = expr else {
        return Err(expected_head.clone());
    };
    if head.as_ref() != expected_head {
        return Err(expected_head.clone());
    }

    if depth + 1 == expected_heads.len() || args.is_empty() {
        return Ok(());
    }

    args.iter()
        .try_for_each(|arg| validate_join_level_heads(arg, expected_heads, depth + 1))
}

fn join_level_unchecked(exprs: &[&Expr], expected_heads: &[Expr], depth: usize) -> Expr {
    if depth + 1 == expected_heads.len() {
        return join_level_targets_unchecked(exprs);
    }

    let max_len = exprs
        .iter()
        .filter_map(|expr| match expr {
            Expr::Call { args, .. } => Some(args.len()),
            Expr::Atom(_) => None,
        })
        .max()
        .unwrap_or(0);

    Expr::call(
        expected_heads[depth].clone(),
        (0..max_len)
            .filter_map(|index| {
                let children = exprs
                    .iter()
                    .filter_map(|expr| match expr {
                        Expr::Call { args, .. } => args.get(index),
                        Expr::Atom(_) => None,
                    })
                    .collect::<Vec<_>>();
                (!children.is_empty())
                    .then(|| join_level_unchecked(&children, expected_heads, depth + 1))
            })
            .collect(),
    )
}

fn join_level_targets_unchecked(exprs: &[&Expr]) -> Expr {
    if exprs.len() == 1 {
        return (*exprs[0]).clone();
    }

    if exprs
        .iter()
        .all(|expr| association_entry_forms_from_expr(expr).is_some())
    {
        let entries = exprs
            .iter()
            .flat_map(|expr| {
                association_entry_forms_from_expr(expr)
                    .expect("validated all join targets are associations")
            })
            .collect();
        return render_association_entry_forms_preserving_heads(entries);
    }

    let Expr::Call {
        head: first_head, ..
    } = exprs[0]
    else {
        return (*exprs[0]).clone();
    };
    Expr::call(
        first_head.as_ref().clone(),
        exprs
            .iter()
            .flat_map(|expr| match expr {
                Expr::Call { args, .. } => args.to_vec(),
                Expr::Atom(_) => Vec::new(),
            })
            .collect(),
    )
}
