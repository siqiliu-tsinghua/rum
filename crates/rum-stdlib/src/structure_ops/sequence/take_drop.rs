mod lazy_completion;

use super::*;
use rum_core::SharedArgs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StructuralSequenceSpec {
    All,
    None,
    UpTo(Option<isize>),
    Count(isize),
    Single(isize),
    Range {
        start: isize,
        end: isize,
        step: isize,
    },
}

const RECURSIVE_TAKE_DROP_SPEC_DEPTH: usize = 16;

pub(crate) fn eval_take(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.first().is_some_and(crate::lazy::is_lazy_list) {
        return eval_take_lazy_list(session, head, args);
    }

    if args.len() < 2 {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_message(
            "Take",
            args,
            "argx",
            "expected at least 2 arguments",
        ));
    }

    let Some(specs) = parse_structural_sequence_specs(session, &args[1..]) else {
        return Ok(Expr::call(system_expr("Take"), args.to_vec()));
    };

    match apply_take_specs(&args[0], &specs) {
        Some(result) => Ok(result),
        None if matches!(args[0], Expr::Atom(_)) => Ok(invalid_take_drop_atom_call(
            session, "Take", "take", args, specs[0],
        )),
        None => Ok(invalid_take_drop_sequence_call(
            session, "Take", "take", args,
        )),
    }
}

fn eval_take_lazy_list(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lazy_list, spec_expr] = args else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let Some(spec) = parse_structural_sequence_spec(session, spec_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let limit = match spec {
        StructuralSequenceSpec::Count(count) if count >= 0 => usize::try_from(count).ok(),
        StructuralSequenceSpec::UpTo(Some(limit)) => usize::try_from(limit).ok(),
        _ => None,
    };
    let Some(limit) = limit else {
        return Ok(invalid_take_drop_sequence_call(
            session, "Take", "take", args,
        ));
    };

    let realized = match crate::lazy::realize_at_most(session, lazy_list, limit) {
        Ok(value) => value,
        Err(error) => lazy_completion::capture(session, error, head, args)?,
    };
    Ok(realized.unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
}

pub(crate) fn eval_drop(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() < 2 {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_message(
            "Drop",
            args,
            "argx",
            "expected at least 2 arguments",
        ));
    }

    let Some(specs) = parse_structural_sequence_specs(session, &args[1..]) else {
        return Ok(Expr::call(system_expr("Drop"), args.to_vec()));
    };

    match apply_drop_specs(&args[0], &specs) {
        Some(result) => Ok(result),
        None if matches!(args[0], Expr::Atom(_)) => Ok(invalid_take_drop_atom_call(
            session, "Drop", "drop", args, specs[0],
        )),
        None => Ok(invalid_take_drop_sequence_call(
            session, "Drop", "drop", args,
        )),
    }
}

pub(crate) fn eval_take_drop(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() < 2 {
        return Ok(session.invalid_call_with_message(
            "TakeDrop",
            args,
            "argx",
            "expected at least 2 arguments",
        ));
    }

    let Some(specs) = parse_structural_sequence_specs(session, &args[1..]) else {
        return Ok(Expr::call(system_expr("TakeDrop"), args.to_vec()));
    };

    match (
        apply_take_specs(&args[0], &specs),
        apply_drop_specs(&args[0], &specs),
    ) {
        (Some(taken), Some(dropped)) => Ok(Expr::call(system_expr("List"), vec![taken, dropped])),
        _ if matches!(args[0], Expr::Atom(_)) => {
            Ok(invalid_take_drop_atom_sequence_call(session, args))
        }
        _ => Ok(session.invalid_call_with_message(
            "TakeDrop",
            args,
            "take",
            format!(
                "Cannot apply the requested sequence specification in {}.",
                Expr::call(system_expr("TakeDrop"), args.to_vec())
            ),
        )),
    }
}

fn invalid_take_drop_sequence_call(
    session: &mut Session,
    symbol: &str,
    tag: &str,
    args: &[Expr],
) -> Expr {
    session.push_message(
        symbol,
        tag,
        format!(
            "Cannot apply the requested sequence specification in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    );
    Expr::call(system_expr(symbol), args.to_vec())
}

fn invalid_take_drop_atom_call(
    session: &mut Session,
    symbol: &str,
    tag: &str,
    args: &[Expr],
    spec: StructuralSequenceSpec,
) -> Expr {
    let (start, end) = take_drop_message_range(spec);
    session.push_message(
        symbol,
        tag,
        format!(
            "Cannot {} positions {} through {} in {}.",
            tag, start, end, args[0]
        ),
    );
    Expr::call(system_expr(symbol), args.to_vec())
}

fn invalid_take_drop_atom_sequence_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.push_message(
        "TakeDrop",
        "iseq",
        format!(
            "Invalid sequence specification {} for an expression of length {}.",
            args[1],
            structural_length(&args[0])
        ),
    );
    Expr::call(system_expr("TakeDrop"), args.to_vec())
}

fn take_drop_message_range(spec: StructuralSequenceSpec) -> (isize, isize) {
    match spec {
        StructuralSequenceSpec::All => (1, -1),
        StructuralSequenceSpec::None => (1, 0),
        StructuralSequenceSpec::UpTo(Some(limit)) => (1, limit),
        StructuralSequenceSpec::UpTo(None) => (1, -1),
        StructuralSequenceSpec::Count(count) if count >= 0 => (1, count),
        StructuralSequenceSpec::Count(count) => (count, -1),
        StructuralSequenceSpec::Single(index) => (index, index),
        StructuralSequenceSpec::Range { start, end, .. } => (start, end),
    }
}

fn parse_structural_sequence_specs(
    session: &mut Session,
    args: &[Expr],
) -> Option<Vec<StructuralSequenceSpec>> {
    args.iter()
        .map(|expr| parse_structural_sequence_spec(session, expr))
        .collect()
}

fn parse_structural_sequence_spec(
    session: &mut Session,
    expr: &Expr,
) -> Option<StructuralSequenceSpec> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(StructuralSequenceSpec::All);
    }

    if is_system_symbol_name(expr, heads::structure::NONE) {
        return Some(StructuralSequenceSpec::None);
    }

    if let Some(count) = expr_as_signed_isize(expr) {
        return Some(StructuralSequenceSpec::Count(count));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "UpTo") {
        let [limit] = args.as_slice() else {
            session.push_message(
                "UpTo",
                "argx",
                format!(
                    "UpTo called with {} arguments; 1 argument is expected.",
                    args.len()
                ),
            );
            return None;
        };

        if is_system_symbol_name(limit, heads::structure::INFINITY) {
            return Some(StructuralSequenceSpec::UpTo(None));
        }

        if let Some(limit) = expr_as_signed_isize(limit).filter(|limit| *limit >= 0) {
            return Some(StructuralSequenceSpec::UpTo(Some(limit)));
        }

        // Message only on definitely-invalid contents (negative integer or
        // rational), matching the kernel Part-span rule; anything else stays
        // on the silent inert path like any other out-of-subset spec.
        let definitely_invalid = match limit {
            Expr::Atom(Atom::Integer(value)) => value.to_bigint() < BigInt::from(0),
            Expr::Atom(Atom::Rational(_)) => true,
            _ => false,
        };
        if definitely_invalid {
            session.push_message(
                "UpTo",
                "innf",
                format!("Non-negative integer or Infinity expected at position 1 in {expr}."),
            );
        }
        return None;
    }

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    match args.as_slice() {
        [index] => Some(StructuralSequenceSpec::Single(expr_as_signed_isize(index)?)),
        [start, end] => Some(StructuralSequenceSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: 1,
        }),
        [start, end, step] => Some(StructuralSequenceSpec::Range {
            start: expr_as_signed_isize(start)?,
            end: expr_as_signed_isize(end)?,
            step: expr_as_signed_isize(step)?,
        }),
        _ => None,
    }
}

fn apply_take_specs(expr: &Expr, specs: &[StructuralSequenceSpec]) -> Option<Expr> {
    apply_take_specs_recursive_bounded(expr, specs, RECURSIVE_TAKE_DROP_SPEC_DEPTH)
}

fn apply_take_specs_recursive_bounded(
    expr: &Expr,
    specs: &[StructuralSequenceSpec],
    remaining_spec_depth: usize,
) -> Option<Expr> {
    if remaining_spec_depth == 0 {
        return apply_take_drop_specs_iterative(expr, specs, apply_take_spec);
    }

    let Some((first, rest)) = specs.split_first() else {
        return Some(expr.clone());
    };

    let selected = apply_take_spec(expr, *first)?;

    if rest.is_empty() {
        Some(selected)
    } else {
        apply_take_drop_rest_to_expr(
            selected,
            rest,
            remaining_spec_depth - 1,
            apply_take_specs_recursive_bounded,
        )
    }
}

fn apply_take_spec(expr: &Expr, spec: StructuralSequenceSpec) -> Option<Expr> {
    let Expr::Call { .. } = expr else {
        return None;
    };

    let components = resolve_take_components(spec, structural_length(expr))?;
    let selectors = vec![PartSelector::Multi(
        components
            .into_iter()
            .map(|component| PartSelector::Single(AtomicPartSpec::Index(component)))
            .collect(),
    )];
    apply_part_selectors(expr, &selectors)
}

fn apply_drop_specs(expr: &Expr, specs: &[StructuralSequenceSpec]) -> Option<Expr> {
    apply_drop_specs_recursive_bounded(expr, specs, RECURSIVE_TAKE_DROP_SPEC_DEPTH)
}

fn apply_drop_specs_recursive_bounded(
    expr: &Expr,
    specs: &[StructuralSequenceSpec],
    remaining_spec_depth: usize,
) -> Option<Expr> {
    if remaining_spec_depth == 0 {
        return apply_take_drop_specs_iterative(expr, specs, apply_drop_spec);
    }

    let Some((first, rest)) = specs.split_first() else {
        return Some(expr.clone());
    };

    let kept = apply_drop_spec(expr, *first)?;

    if rest.is_empty() {
        Some(kept)
    } else {
        apply_take_drop_rest_to_expr(
            kept,
            rest,
            remaining_spec_depth - 1,
            apply_drop_specs_recursive_bounded,
        )
    }
}

fn apply_drop_spec(expr: &Expr, spec: StructuralSequenceSpec) -> Option<Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    let dropped = resolve_take_components(spec, args.len())?;
    let kept_args = retain_args_except_positions(args, dropped);
    Some(Expr::call(head.as_ref().clone(), kept_args))
}

// Keep the wide scan out of the recursive spec walker. Release A/B found no
// small-call cost and materially less same-tier microbenchmark variance.
#[inline(never)]
fn retain_args_except_positions(args: &[Expr], mut dropped: Vec<isize>) -> Vec<Expr> {
    if dropped.first() > dropped.last() {
        dropped.reverse();
    }
    debug_assert!(dropped.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(dropped.len() <= args.len());

    let mut kept_args = Vec::with_capacity(args.len() - dropped.len());
    let mut dropped = dropped.into_iter();
    let mut next_dropped = dropped.next();
    for (index, arg) in args.iter().enumerate() {
        let position = index as isize + 1;
        if next_dropped == Some(position) {
            next_dropped = dropped.next();
        } else {
            kept_args.push(arg.clone());
        }
    }
    debug_assert!(next_dropped.is_none());
    kept_args
}

fn apply_take_drop_rest_to_expr(
    expr: Expr,
    rest: &[StructuralSequenceSpec],
    remaining_spec_depth: usize,
    recurse: fn(&Expr, &[StructuralSequenceSpec], usize) -> Option<Expr>,
) -> Option<Expr> {
    match expr {
        Expr::Atom(_) => None,
        Expr::Call { head, args } => Some(Expr::call(
            head.as_ref().clone(),
            args.iter()
                .map(|arg| recurse(arg, rest, remaining_spec_depth))
                .collect::<Option<Vec<_>>>()?,
        )),
    }
}

enum TakeDropSpecWork<'a> {
    Visit {
        expr: Expr,
        specs: &'a [StructuralSequenceSpec],
    },
    FinishCall {
        head: Expr,
        arg_count: usize,
    },
}

#[cold]
#[inline(never)]
fn apply_take_drop_specs_iterative(
    expr: &Expr,
    specs: &[StructuralSequenceSpec],
    apply_spec: fn(&Expr, StructuralSequenceSpec) -> Option<Expr>,
) -> Option<Expr> {
    let mut work = vec![TakeDropSpecWork::Visit {
        expr: expr.clone(),
        specs,
    }];
    let mut completed = Vec::new();

    while let Some(task) = work.pop() {
        match task {
            TakeDropSpecWork::Visit { expr, specs } => {
                let Some((first, rest)) = specs.split_first() else {
                    completed.push(expr);
                    continue;
                };

                let selected = apply_spec(&expr, *first)?;
                if rest.is_empty() {
                    completed.push(selected);
                    continue;
                }

                let Expr::Call { head, args } = selected else {
                    return None;
                };
                let arg_count = args.len();
                let head = head.as_ref().clone();
                if arg_count == 0 {
                    completed.push(Expr::call(head, Vec::new()));
                    continue;
                }

                work.push(TakeDropSpecWork::FinishCall { head, arg_count });
                for arg in args.iter().rev() {
                    work.push(TakeDropSpecWork::Visit {
                        expr: arg.clone(),
                        specs: rest,
                    });
                }
            }
            TakeDropSpecWork::FinishCall { head, arg_count } => {
                let first_arg = completed
                    .len()
                    .checked_sub(arg_count)
                    .expect("Take/Drop iterative results remain balanced");
                let args = completed.split_off(first_arg);
                completed.push(Expr::call(head, args));
            }
        }
    }

    debug_assert_eq!(completed.len(), 1);
    completed.pop()
}

fn resolve_take_components(spec: StructuralSequenceSpec, len: usize) -> Option<Vec<isize>> {
    match spec {
        StructuralSequenceSpec::All => Some((1..=len).map(|index| index as isize).collect()),
        StructuralSequenceSpec::None => Some(Vec::new()),
        StructuralSequenceSpec::UpTo(limit) => {
            let end = match limit {
                Some(limit) => usize::try_from(limit).ok()?.min(len),
                None => len,
            };
            Some((1..=end).map(|index| index as isize).collect())
        }
        StructuralSequenceSpec::Count(0) => Some(Vec::new()),
        StructuralSequenceSpec::Count(count) if count > 0 => {
            let count = usize::try_from(count).ok()?;
            (count <= len).then(|| (1..=count).map(|index| index as isize).collect())
        }
        StructuralSequenceSpec::Count(count) => {
            let count = count.unsigned_abs();
            if count > len {
                None
            } else {
                Some(
                    ((len - count + 1)..=len)
                        .map(|index| index as isize)
                        .collect(),
                )
            }
        }
        StructuralSequenceSpec::Single(index) => {
            Some(vec![normalize_exact_structural_component(index, len)?])
        }
        StructuralSequenceSpec::Range { start, end, step } => {
            if step == 0 {
                return None;
            }

            let start = normalize_exact_structural_component(start, len)?;
            let end = normalize_exact_structural_component(end, len)?;
            let mut current = start;
            let mut components = Vec::new();

            if step > 0 {
                while current <= end {
                    components.push(current);
                    current += step;
                }
            } else {
                while current >= end {
                    components.push(current);
                    current += step;
                }
            }

            Some(components)
        }
    }
}

fn normalize_exact_structural_component(position: isize, len: usize) -> Option<isize> {
    Some(normalize_structural_arg_position(position, len)? as isize + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap_unary_lists(mut expr: Expr, count: usize) -> Expr {
        for _ in 0..count {
            expr = Expr::call(system_expr("List"), vec![expr]);
        }
        expr
    }

    fn branching_take_drop_fixture() -> Expr {
        Expr::call(
            system_expr("f"),
            vec![
                Expr::call(system_expr("g"), vec![system_expr("a"), system_expr("b")]),
                Expr::call(system_expr("h"), vec![system_expr("c"), system_expr("d")]),
            ],
        )
    }

    fn branching_take_drop_expected() -> Expr {
        Expr::call(
            system_expr("f"),
            vec![
                Expr::call(system_expr("g"), vec![system_expr("b")]),
                Expr::call(system_expr("h"), vec![system_expr("d")]),
            ],
        )
    }

    #[test]
    fn iterative_take_tail_preserves_fanout_order() {
        let wrapper_count = RECURSIVE_TAKE_DROP_SPEC_DEPTH;
        let expr = wrap_unary_lists(branching_take_drop_fixture(), wrapper_count);
        let mut specs = vec![StructuralSequenceSpec::Single(1); wrapper_count];
        specs.extend([
            StructuralSequenceSpec::All,
            StructuralSequenceSpec::Single(2),
        ]);

        assert_eq!(
            apply_take_specs(&expr, &specs),
            Some(wrap_unary_lists(
                branching_take_drop_expected(),
                wrapper_count
            ))
        );
    }

    #[test]
    fn iterative_drop_tail_preserves_fanout_order() {
        let wrapper_count = RECURSIVE_TAKE_DROP_SPEC_DEPTH;
        let expr = wrap_unary_lists(branching_take_drop_fixture(), wrapper_count);
        let mut specs = vec![StructuralSequenceSpec::None; wrapper_count + 1];
        specs.push(StructuralSequenceSpec::Count(1));

        assert_eq!(
            apply_drop_specs(&expr, &specs),
            Some(wrap_unary_lists(
                branching_take_drop_expected(),
                wrapper_count
            ))
        );
    }

    #[test]
    fn linear_drop_filter_matches_set_membership_for_small_ranges() {
        for len in 1..=8 {
            let positions = (-(len as isize)..=-1)
                .chain(1..=len as isize)
                .collect::<Vec<_>>();
            let args = (1..=len)
                .map(|value| Expr::integer(BigInt::from(value)))
                .collect::<Vec<_>>();

            for &start in &positions {
                for &end in &positions {
                    for step in [-3, -2, -1, 1, 2, 3] {
                        let dropped = resolve_take_components(
                            StructuralSequenceSpec::Range { start, end, step },
                            len,
                        )
                        .expect("in-range endpoints and a nonzero step are valid");
                        assert!(
                            dropped.windows(2).all(|pair| pair[0] < pair[1])
                                || dropped.windows(2).all(|pair| pair[0] > pair[1])
                        );

                        let dropped_set = dropped.iter().copied().collect::<BTreeSet<_>>();
                        let expected = args
                            .iter()
                            .enumerate()
                            .filter(|(index, _)| !dropped_set.contains(&(*index as isize + 1)))
                            .map(|(_, arg)| arg.clone())
                            .collect::<Vec<_>>();
                        assert_eq!(retain_args_except_positions(&args, dropped), expected);
                    }
                }
            }
        }
    }
}
