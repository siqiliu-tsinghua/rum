use super::*;

pub(super) fn exact_position_paths_from_expr(
    expr: &Expr,
    spec_expr: &Expr,
) -> Option<Vec<Vec<isize>>> {
    if let Some(position_spec) = parse_single_position_spec(spec_expr) {
        return Some(collect_exact_paths_for_position_spec(
            expr,
            &position_spec,
            ReplacePartHeadsMode::Automatic,
        ));
    }

    let Expr::Call { head, args } = spec_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let mut paths = Vec::new();
    for item in args {
        let position_spec = parse_single_position_spec(item)?;
        paths.extend(collect_exact_paths_for_position_spec(
            expr,
            &position_spec,
            ReplacePartHeadsMode::Automatic,
        ));
    }
    Some(paths)
}

pub(super) fn exact_expr_at_path(expr: &Expr, path: &[isize]) -> Option<Expr> {
    let Some((first, rest)) = path.split_first() else {
        return Some(expr.clone());
    };

    let selected = select_exact_part(expr, *first)?;
    exact_expr_at_path(&selected, rest)
}

pub(super) fn select_exact_part(expr: &Expr, index: isize) -> Option<Expr> {
    if index == 0 {
        return Some(structural_head_expr(expr));
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            let normalized = normalize_structural_arg_position(index, args.len())?;
            Some(args[normalized].clone())
        }
    }
}

pub(super) fn exact_part_component(expr: &Expr, index: isize) -> Option<isize> {
    if index == 0 {
        return Some(0);
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            Some(normalize_structural_arg_position(index, args.len())? as isize + 1)
        }
    }
}

pub(super) fn normalize_structural_arg_position(position: isize, len: usize) -> Option<usize> {
    if position == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position - 1
    } else {
        len + position
    };

    (0..len)
        .contains(&normalized)
        .then_some(normalized as usize)
}

pub(super) fn collect_exact_paths_for_position_spec(
    expr: &Expr,
    selectors: &[PartSelector],
    heads: ReplacePartHeadsMode,
) -> Vec<Vec<isize>> {
    let Some((first, rest)) = selectors.split_first() else {
        return vec![Vec::new()];
    };

    let Some(selected) = select_positioned_parts_for_selector(expr, first, heads) else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    for (prefix, value) in selected {
        for mut tail in collect_exact_paths_for_position_spec(&value, rest, heads) {
            let mut path = prefix.clone();
            path.append(&mut tail);
            paths.push(path);
        }
    }
    paths
}

fn select_positioned_parts_for_selector(
    expr: &Expr,
    selector: &PartSelector,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<(Vec<isize>, Expr)>> {
    match selector {
        PartSelector::Single(spec) => Some(
            select_positioned_parts_for_atomic_spec_with_heads(expr, spec, heads)?
                .into_iter()
                .map(|(component, value)| (vec![component], value))
                .collect(),
        ),
        PartSelector::Key(key) => association_position_and_value_by_key(expr, key)
            .map(|(component, value)| vec![(vec![component, 2], value)]),
        PartSelector::Multi(specs) => {
            let mut selected = Vec::new();
            for spec in specs {
                selected.extend(select_positioned_parts_for_selector(expr, spec, heads)?);
            }
            Some(selected)
        }
    }
}

fn select_positioned_parts_for_atomic_spec_with_heads(
    expr: &Expr,
    spec: &AtomicPartSpec,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<(isize, Expr)>> {
    match spec {
        AtomicPartSpec::Index(0) => match heads {
            ReplacePartHeadsMode::Exclude => None,
            ReplacePartHeadsMode::Automatic | ReplacePartHeadsMode::Include => match expr {
                Expr::Atom(_) => Some(Vec::new()),
                Expr::Call { .. } => Some(vec![(0, structural_head_expr(expr))]),
            },
        },
        AtomicPartSpec::Index(index) => {
            let value = select_exact_part(expr, *index)?;
            let component = exact_part_component(expr, *index)?;
            Some(vec![(component, value)])
        }
        AtomicPartSpec::All => match expr {
            Expr::Atom(_) => Some(Vec::new()),
            Expr::Call { args, .. } => {
                let mut parts = Vec::new();
                if matches!(heads, ReplacePartHeadsMode::Include) {
                    parts.push((0, structural_head_expr(expr)));
                }
                parts.extend(
                    args.iter()
                        .enumerate()
                        .map(|(index, value)| (index as isize + 1, value.clone())),
                );
                Some(parts)
            }
        },
        AtomicPartSpec::Span(span) => select_positioned_parts_for_span(expr, *span),
    }
}

pub(super) fn replace_exact_path(expr: Expr, path: &[isize], rhs: Expr) -> Expr {
    let Some((first, rest)) = path.split_first() else {
        return rhs;
    };

    if *first == 0 {
        let replaced_head = replace_exact_path(structural_head_expr(&expr), rest, rhs);
        return match expr {
            Expr::Atom(_) => Expr::call(replaced_head, Vec::new()),
            Expr::Call { args, .. } => Expr::call_with_shared_args(replaced_head, args),
        };
    }

    match expr {
        Expr::Atom(_) => expr,
        Expr::Call { head, args } => {
            let Some(index) = normalize_structural_arg_position(*first, args.len()) else {
                return Expr::Call { head, args };
            };

            let mut args = Expr::unwrap_shared_args(args);
            let replaced_child = replace_exact_path(args[index].clone(), rest, rhs);
            args[index] = replaced_child;
            Expr::Call {
                head,
                args: SharedArgs::new(args),
            }
        }
    }
}
