use super::*;
use rum_core::SharedArgs;

// Deletion compaction crossed over at eight paths in the measured wide-parent
// workload; preserve the allocation-light direct path below that point.
const BATCHED_DELETE_THRESHOLD: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeletePathFailure {
    part: String,
    target: Expr,
}

pub(crate) fn eval_delete(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr, positions_expr] = args else {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_exact_arg_count("Delete", args, 2));
    };

    if matches!(expr, Expr::Atom(_)) {
        if let Some(part) = delete_atom_part_message_part(positions_expr) {
            return Ok(invalid_delete_atom_part_call(session, args, &part));
        }
        return Ok(invalid_normal_expression_call(session, "Delete", args));
    }

    let Expr::Call {
        head: expr_head,
        args: expr_args,
        ..
    } = expr
    else {
        return Ok(invalid_normal_expression_call(session, "Delete", args));
    };

    let Some(paths) = parse_delete_position_paths(positions_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let mut canonical_paths = Vec::new();
    for path in paths {
        match canonical_delete_path(expr, &path) {
            Ok(path) => canonical_paths.push(path),
            Err(failure) => return Ok(invalid_delete_path_call(session, args, failure)),
        }
    }

    Ok(delete_canonical_paths(
        Expr::call_with_shared_args(expr_head.as_ref().clone(), expr_args.clone()),
        canonical_paths,
    ))
}

fn invalid_delete_atom_part_call(session: &mut Session, args: &[Expr], part: &Expr) -> Expr {
    session.push_message(
        "Delete",
        "partw",
        format!("Part {} of {} does not exist.", part, args[0]),
    );
    Expr::call(system_expr("Delete"), args.to_vec())
}

fn invalid_delete_path_call(
    session: &mut Session,
    args: &[Expr],
    failure: DeletePathFailure,
) -> Expr {
    session.push_message(
        "Delete",
        "partw",
        format!(
            "Part {} of {} does not exist.",
            failure.part, failure.target
        ),
    );
    Expr::call(system_expr("Delete"), args.to_vec())
}

fn delete_atom_part_message_part(positions_expr: &Expr) -> Option<Expr> {
    if expr_as_signed_isize(positions_expr).is_some() {
        return Some(positions_expr.clone());
    }

    let Expr::Call { head, args, .. } = positions_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let part = args.first()?;
    expr_as_signed_isize(part).is_some().then(|| part.clone())
}

fn parse_delete_position_paths(positions_expr: &Expr) -> Option<Vec<Vec<isize>>> {
    if let Some(position) = expr_as_signed_isize(positions_expr) {
        return Some(vec![vec![position]]);
    }

    let Expr::Call { head, args, .. } = positions_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    if args.is_empty() {
        return Some(Vec::new());
    }

    if args.iter().all(|expr| expr_as_signed_isize(expr).is_some()) {
        return Some(vec![
            args.iter()
                .map(expr_as_signed_isize)
                .collect::<Option<Vec<_>>>()?,
        ]);
    }

    args.iter().map(parse_delete_position_path_list).collect()
}

fn parse_delete_position_path_list(expr: &Expr) -> Option<Vec<isize>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) || args.is_empty() {
        return None;
    }

    args.iter().map(expr_as_signed_isize).collect()
}

fn canonical_delete_path(expr: &Expr, path: &[isize]) -> Result<Vec<isize>, DeletePathFailure> {
    let mut target = expr.clone();
    let mut canonical = Vec::with_capacity(path.len());

    for (depth, position) in path.iter().copied().enumerate() {
        if position == 0 {
            match &target {
                Expr::Atom(_) => {
                    return Err(DeletePathFailure {
                        part: delete_failure_part(path, depth, position),
                        target,
                    });
                }
                Expr::Call { .. } => {
                    canonical.push(0);
                    target = structural_head_expr(&target);
                }
            }
            continue;
        }

        let Expr::Call { args, .. } = &target else {
            return Err(DeletePathFailure {
                part: delete_failure_part(path, depth, position),
                target,
            });
        };

        let Some(index) = normalize_structural_arg_position(position, args.len()) else {
            return Err(DeletePathFailure {
                part: delete_failure_part(path, depth, position),
                target,
            });
        };
        canonical.push(index as isize + 1);
        target = args[index].clone();
    }

    Ok(canonical)
}

fn delete_failure_part(path: &[isize], depth: usize, position: isize) -> String {
    if depth == 0 {
        return format!(
            "{{{}}}",
            path.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    position.to_string()
}

fn delete_canonical_paths(expr: Expr, mut paths: Vec<Vec<isize>>) -> Expr {
    paths.sort_by(|left, right| right.cmp(left));
    paths.dedup();
    if paths.len() >= BATCHED_DELETE_THRESHOLD {
        return super::edit_plan::batch_delete_canonical_paths(expr, &paths);
    }
    paths
        .into_iter()
        .fold(expr, |expr, path| delete_canonical_path(expr, &path))
}

fn delete_canonical_path(expr: Expr, path: &[isize]) -> Expr {
    let Some((first, rest)) = path.split_first() else {
        return expr;
    };

    match expr {
        Expr::Atom(_) => expr,
        Expr::Call { head, args, .. } if *first == 0 => {
            if rest.is_empty() {
                return expression_without_head(args.into_vec());
            }

            let replaced_head = delete_canonical_path(head.as_ref().clone(), rest);
            Expr::call_with_shared_args(replaced_head, args)
        }
        Expr::Call { head, args, .. } => {
            let index = (*first as usize).saturating_sub(1);
            if index >= args.len() {
                return Expr::call_shared(head, args);
            }

            let mut args = Expr::unwrap_shared_args(args);
            if rest.is_empty() {
                args.remove(index);
            } else {
                args[index] = delete_canonical_path(args[index].clone(), rest);
            }
            Expr::call_shared(head, SharedArgs::new(args))
        }
    }
}

fn expression_without_head(args: Vec<Expr>) -> Expr {
    match args.as_slice() {
        [single] => single.clone(),
        _ => Expr::call(system_expr("Sequence"), args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_planner_starts_at_the_measured_crossover() {
        let expr = list_of_integers(64);
        super::super::edit_plan::reset_delete_parent_scan_count();
        delete_canonical_paths(expr.clone(), (1..=7).map(|index| vec![index]).collect());
        assert_eq!(super::super::edit_plan::delete_parent_scan_count(), 0);

        super::super::edit_plan::reset_delete_parent_scan_count();
        delete_canonical_paths(expr, (1..=8).map(|index| vec![index]).collect());
        assert_eq!(super::super::edit_plan::delete_parent_scan_count(), 64);
    }

    fn list_of_integers(len: usize) -> Expr {
        Expr::call(
            system_expr(heads::structure::LIST),
            (0..len)
                .map(|value| Expr::integer(BigInt::from(value)))
                .collect(),
        )
    }
}
