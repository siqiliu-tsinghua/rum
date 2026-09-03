use super::*;
use rum_core::SharedArgs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlattenDepth {
    Finite(usize),
    Infinity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FlattenSpec {
    Depth(FlattenDepth),
    LevelGroups(Vec<Vec<usize>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FlattenSpecError {
    InvalidDepth,
    InvalidPositiveIntegerList,
    RepeatedLevel(usize),
}

pub(crate) fn eval_flatten(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let expr = match args {
        [expr] | [expr, _] | [expr, _, _] => expr,
        _ => return Ok(session.invalid_call_with_arg_range("Flatten", args, 1, 3)),
    };

    if matches!(expr, Expr::Atom(_)) || association_entries_from_expr(expr).is_some() {
        return Ok(invalid_flatten_normal_expression_call(session, args));
    }

    let (spec, explicit_head) = match args {
        [_] => (FlattenSpec::Depth(FlattenDepth::Infinity), None),
        [_, depth_expr] => {
            let spec = match parse_flatten_spec_with_messages(session, args, depth_expr) {
                Ok(spec) => spec,
                Err(value) => return Ok(value),
            };
            (spec, None)
        }
        [_, depth_expr, head_spec] => {
            let spec = match parse_flatten_spec_with_messages(session, args, depth_expr) {
                Ok(spec) => spec,
                Err(value) => return Ok(value),
            };
            (spec, Some(head_spec))
        }
        _ => unreachable!("arity checked above"),
    };

    Ok(match spec {
        FlattenSpec::Depth(depth) => flatten_same_head(expr, depth, explicit_head),
        FlattenSpec::LevelGroups(groups) => {
            flatten_level_groups(session, args, expr, groups, explicit_head)
        }
    })
}

fn parse_flatten_spec_with_messages(
    session: &mut Session,
    args: &[Expr],
    spec_expr: &Expr,
) -> Result<FlattenSpec, Expr> {
    parse_flatten_spec(spec_expr).map_err(|error| match error {
        FlattenSpecError::InvalidDepth => invalid_flatten_level_call(session, args, spec_expr),
        FlattenSpecError::InvalidPositiveIntegerList => {
            invalid_flatten_positive_integer_list_call(session, args, spec_expr)
        }
        FlattenSpecError::RepeatedLevel(level) => {
            invalid_flatten_repeated_level_call(session, args, level)
        }
    })
}

fn flatten_level_groups(
    session: &mut Session,
    call_args: &[Expr],
    expr: &Expr,
    groups: Vec<Vec<usize>>,
    explicit_head: Option<&Expr>,
) -> Expr {
    let Expr::Call { head, .. } = expr else {
        return expr.clone();
    };
    let target_head = explicit_head.unwrap_or(head.as_ref());
    let depth = flattenable_depth(expr, target_head);

    if let Some(level) = first_excess_level(&groups, depth) {
        return invalid_flatten_depth_call(session, call_args, level, depth);
    }

    flatten_by_level_groups(expr, target_head, complete_level_groups(groups, depth))
}

fn invalid_flatten_level_call(session: &mut Session, args: &[Expr], depth_expr: &Expr) -> Expr {
    let call = Expr::call(system_expr("Flatten"), args.to_vec());
    session.invalid_call_with_message(
            "Flatten",
            args,
            "flev",
            format!(
                "The level argument {depth_expr} in position 2 of {call} should be a non-negative integer or Infinity."
            ),
        )
}

fn invalid_flatten_positive_integer_list_call(
    session: &mut Session,
    args: &[Expr],
    spec_expr: &Expr,
) -> Expr {
    session.invalid_call_with_message(
        "Flatten",
        args,
        "flpi",
        format!(
            "Levels to be flattened together in {spec_expr} should be lists of positive integers."
        ),
    )
}

fn invalid_flatten_repeated_level_call(session: &mut Session, args: &[Expr], level: usize) -> Expr {
    session.invalid_call_with_message(
        "Flatten",
        args,
        "flrep",
        format!("Level {level} should not be repeated."),
    )
}

fn invalid_flatten_depth_call(
    session: &mut Session,
    args: &[Expr],
    level: usize,
    depth: usize,
) -> Expr {
    let call = Expr::call(system_expr("Flatten"), args.to_vec());
    session.invalid_call_with_message(
        "Flatten",
        args,
        "fldep",
        format!(
            "Level {level} exceeds the levels, {depth}, which can be flattened together in {call}."
        ),
    )
}

fn invalid_flatten_normal_expression_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Flatten",
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr("Flatten"), args.to_vec())
        ),
    )
}

fn parse_flatten_spec(expr: &Expr) -> Result<FlattenSpec, FlattenSpecError> {
    if is_system_symbol_name(expr, "Infinity") {
        return Ok(FlattenSpec::Depth(FlattenDepth::Infinity));
    }
    if let Some(depth) = expr_as_non_negative_usize(expr) {
        return Ok(FlattenSpec::Depth(FlattenDepth::Finite(depth)));
    }

    let Some(items) = list_items(expr) else {
        return Err(FlattenSpecError::InvalidDepth);
    };

    let groups = parse_flatten_level_groups(items)?;
    validate_flatten_level_repetitions(&groups)?;
    Ok(FlattenSpec::LevelGroups(groups))
}

fn parse_flatten_level_groups(items: &[Expr]) -> Result<Vec<Vec<usize>>, FlattenSpecError> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    if let Some(group) = items.iter().map(expr_as_positive_usize).collect() {
        return Ok(vec![group]);
    }

    items
        .iter()
        .map(|item| {
            let Some(group_items) = list_items(item) else {
                return Err(FlattenSpecError::InvalidPositiveIntegerList);
            };
            if group_items.is_empty() {
                return Err(FlattenSpecError::InvalidPositiveIntegerList);
            }
            group_items
                .iter()
                .map(expr_as_positive_usize)
                .collect::<Option<Vec<_>>>()
                .ok_or(FlattenSpecError::InvalidPositiveIntegerList)
        })
        .collect()
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    expr_as_non_negative_usize(expr).filter(|value| *value > 0)
}

fn validate_flatten_level_repetitions(groups: &[Vec<usize>]) -> Result<(), FlattenSpecError> {
    let mut seen = BTreeSet::new();
    for level in groups.iter().flatten() {
        if !seen.insert(*level) {
            return Err(FlattenSpecError::RepeatedLevel(*level));
        }
    }
    Ok(())
}

fn flatten_same_head(expr: &Expr, depth: FlattenDepth, explicit_head: Option<&Expr>) -> Expr {
    let Expr::Call { head, args } = expr else {
        return expr.clone();
    };

    if depth == FlattenDepth::Finite(0) {
        return expr.clone();
    }

    let target_head = explicit_head.unwrap_or(head.as_ref());
    let mut flattened = Vec::new();
    for arg in args {
        flatten_matching_arg(arg, target_head, depth, &mut flattened);
    }

    Expr::call(head.as_ref().clone(), flattened)
}

fn flatten_matching_arg(
    expr: &Expr,
    target_head: &Expr,
    depth: FlattenDepth,
    flattened: &mut Vec<Expr>,
) {
    match depth {
        FlattenDepth::Finite(0) => flattened.push(expr.clone()),
        FlattenDepth::Finite(level) => match expr {
            Expr::Call { head, args } if head.as_ref() == target_head => {
                for arg in args {
                    flatten_matching_arg(
                        arg,
                        target_head,
                        FlattenDepth::Finite(level.saturating_sub(1)),
                        flattened,
                    );
                }
            }
            _ => flattened.push(expr.clone()),
        },
        FlattenDepth::Infinity => match expr {
            Expr::Call { head, args } if head.as_ref() == target_head => {
                for arg in args {
                    flatten_matching_arg(arg, target_head, FlattenDepth::Infinity, flattened);
                }
            }
            _ => flattened.push(expr.clone()),
        },
    }
}

fn flattenable_depth(expr: &Expr, target_head: &Expr) -> usize {
    let Expr::Call { head, args } = expr else {
        return 0;
    };
    if head.as_ref() != target_head {
        return 0;
    }
    if args.is_empty() {
        return 1;
    }

    1 + args
        .iter()
        .map(|arg| flattenable_depth(arg, target_head))
        .min()
        .unwrap_or(0)
}

fn first_excess_level(groups: &[Vec<usize>], depth: usize) -> Option<usize> {
    groups
        .iter()
        .flatten()
        .copied()
        .find(|level| *level > depth)
}

fn complete_level_groups(mut groups: Vec<Vec<usize>>, depth: usize) -> Vec<Vec<usize>> {
    let used = groups.iter().flatten().copied().collect::<BTreeSet<_>>();
    for level in 1..=depth {
        if !used.contains(&level) {
            groups.push(vec![level]);
        }
    }
    groups
}

fn flatten_by_level_groups(expr: &Expr, target_head: &Expr, groups: Vec<Vec<usize>>) -> Expr {
    if groups.is_empty() {
        return expr.clone();
    }

    let Expr::Call { head, .. } = expr else {
        return expr.clone();
    };
    let depth = groups.iter().flatten().copied().max().unwrap_or(0);
    let mut records = Vec::new();
    collect_flatten_level_records(expr, target_head, depth, &mut Vec::new(), &mut records);

    if records.is_empty() {
        return if is_identity_level_groups(&groups) {
            expr.clone()
        } else {
            Expr::call(head.as_ref().clone(), Vec::new())
        };
    }

    let keyed_records = records
        .into_iter()
        .map(|(path, value)| {
            let keys = groups
                .iter()
                .map(|group| group.iter().map(|level| path[*level - 1]).collect())
                .collect();
            (keys, value)
        })
        .collect::<Vec<_>>();

    build_flatten_group(head.as_ref(), &keyed_records, 0)
}

fn collect_flatten_level_records(
    expr: &Expr,
    target_head: &Expr,
    remaining_depth: usize,
    path: &mut Vec<usize>,
    records: &mut Vec<(Vec<usize>, Expr)>,
) {
    if remaining_depth == 0 {
        records.push((path.clone(), expr.clone()));
        return;
    }

    let Expr::Call { head, args } = expr else {
        return;
    };
    if head.as_ref() != target_head {
        return;
    }

    for (index, arg) in args.iter().enumerate() {
        path.push(index + 1);
        collect_flatten_level_records(arg, target_head, remaining_depth - 1, path, records);
        path.pop();
    }
}

fn build_flatten_group(
    head: &Expr,
    records: &[(Vec<Vec<usize>>, Expr)],
    group_index: usize,
) -> Expr {
    if group_index >= records.first().map(|(keys, _)| keys.len()).unwrap_or(0) {
        return records
            .first()
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| Expr::call(head.clone(), Vec::new()));
    }

    let mut groups: BTreeMap<Vec<usize>, Vec<(Vec<Vec<usize>>, Expr)>> = BTreeMap::new();
    for (keys, value) in records {
        let key = keys[group_index].clone();
        groups
            .entry(key)
            .or_default()
            .push((keys.clone(), value.clone()));
    }

    Expr::call(
        head.clone(),
        groups
            .values()
            .map(|values| build_flatten_group(head, values, group_index + 1))
            .collect(),
    )
}

fn is_identity_level_groups(groups: &[Vec<usize>]) -> bool {
    groups
        .iter()
        .enumerate()
        .all(|(index, group)| group.as_slice() == [index + 1])
}
