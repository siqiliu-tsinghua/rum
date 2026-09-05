use super::*;

const RECURSIVE_EXACT_EXPR_AT_PATH_DEPTH: usize = 16;
const RECURSIVE_COLLECT_EXACT_PATHS_DEPTH: usize = 16;
const RECURSIVE_REPLACE_EXACT_PATH_DEPTH: usize = 16;

enum BoundedTraversal<T> {
    Complete(T),
    Exhausted(T),
}

impl<T> BoundedTraversal<T> {
    fn into_value(self) -> T {
        match self {
            Self::Complete(value) | Self::Exhausted(value) => value,
        }
    }
}

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
    exact_expr_at_path_with_front_depth(expr, path, RECURSIVE_EXACT_EXPR_AT_PATH_DEPTH).into_value()
}

fn exact_expr_at_path_with_front_depth(
    expr: &Expr,
    path: &[isize],
    remaining_path_depth: usize,
) -> BoundedTraversal<Option<Expr>> {
    let Some((first, rest)) = path.split_first() else {
        return BoundedTraversal::Complete(Some(expr.clone()));
    };

    if remaining_path_depth == 0 {
        return BoundedTraversal::Exhausted(exact_expr_at_path_iterative(expr, path));
    }

    let Some(selected) = select_exact_part(expr, *first) else {
        return BoundedTraversal::Complete(None);
    };
    exact_expr_at_path_with_front_depth(&selected, rest, remaining_path_depth - 1)
}

#[cold]
#[inline(never)]
fn exact_expr_at_path_iterative(expr: &Expr, path: &[isize]) -> Option<Expr> {
    let mut selected = expr.clone();
    for component in path {
        selected = select_exact_part(&selected, *component)?;
    }
    Some(selected)
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
    collect_exact_paths_for_position_spec_with_front_depth(
        expr,
        selectors,
        heads,
        RECURSIVE_COLLECT_EXACT_PATHS_DEPTH,
    )
    .into_value()
}

fn collect_exact_paths_for_position_spec_with_front_depth(
    expr: &Expr,
    selectors: &[PartSelector],
    heads: ReplacePartHeadsMode,
    remaining_path_depth: usize,
) -> BoundedTraversal<Vec<Vec<isize>>> {
    let Some((first, rest)) = selectors.split_first() else {
        return BoundedTraversal::Complete(vec![Vec::new()]);
    };

    if remaining_path_depth == 0 {
        return BoundedTraversal::Exhausted(collect_exact_paths_for_position_spec_iterative(
            expr, selectors, heads,
        ));
    }

    let Some(selected) = select_positioned_parts_for_selector(expr, first, heads) else {
        return BoundedTraversal::Complete(Vec::new());
    };

    let mut paths = Vec::new();
    let mut exhausted = false;
    for (prefix, value) in selected {
        let tails = match collect_exact_paths_for_position_spec_with_front_depth(
            &value,
            rest,
            heads,
            remaining_path_depth - 1,
        ) {
            BoundedTraversal::Complete(tails) => tails,
            BoundedTraversal::Exhausted(tails) => {
                exhausted = true;
                tails
            }
        };
        for mut tail in tails {
            let mut path = prefix.clone();
            path.append(&mut tail);
            paths.push(path);
        }
    }

    if exhausted {
        BoundedTraversal::Exhausted(paths)
    } else {
        BoundedTraversal::Complete(paths)
    }
}

enum CollectExactPathWork {
    Visit {
        expr: Expr,
        selector_index: usize,
        prefix: Vec<isize>,
    },
    Exit {
        component_count: usize,
    },
}

#[cold]
#[inline(never)]
fn collect_exact_paths_for_position_spec_iterative(
    expr: &Expr,
    selectors: &[PartSelector],
    heads: ReplacePartHeadsMode,
) -> Vec<Vec<isize>> {
    let mut work = vec![CollectExactPathWork::Visit {
        expr: expr.clone(),
        selector_index: 0,
        prefix: Vec::new(),
    }];
    let mut current_path = Vec::new();
    let mut paths = Vec::new();

    while let Some(task) = work.pop() {
        match task {
            CollectExactPathWork::Visit {
                expr,
                selector_index,
                prefix,
            } => {
                let component_count = prefix.len();
                current_path.extend(prefix);
                work.push(CollectExactPathWork::Exit { component_count });

                let Some(selector) = selectors.get(selector_index) else {
                    paths.push(current_path.clone());
                    continue;
                };
                let Some(selected) = select_positioned_parts_for_selector(&expr, selector, heads)
                else {
                    continue;
                };

                work.extend(selected.into_iter().rev().map(|(prefix, expr)| {
                    CollectExactPathWork::Visit {
                        expr,
                        selector_index: selector_index + 1,
                        prefix,
                    }
                }));
            }
            CollectExactPathWork::Exit { component_count } => {
                current_path.truncate(current_path.len() - component_count);
            }
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
        PartSelector::Multi(_) => select_positioned_parts_for_selector_with_front_depth(
            expr,
            selector,
            heads,
            RECURSIVE_COLLECT_EXACT_PATHS_DEPTH,
        )
        .into_value(),
        PartSelector::Single(_) | PartSelector::Key(_) => {
            select_positioned_parts_for_non_multi_selector(expr, selector, heads)
        }
    }
}

fn select_positioned_parts_for_selector_with_front_depth(
    expr: &Expr,
    selector: &PartSelector,
    heads: ReplacePartHeadsMode,
    remaining_selector_depth: usize,
) -> BoundedTraversal<Option<Vec<(Vec<isize>, Expr)>>> {
    match selector {
        PartSelector::Single(_) | PartSelector::Key(_) => BoundedTraversal::Complete(
            select_positioned_parts_for_non_multi_selector(expr, selector, heads),
        ),
        PartSelector::Multi(specs) => {
            if remaining_selector_depth == 0 {
                return BoundedTraversal::Exhausted(
                    select_positioned_parts_for_selector_iterative(expr, selector, heads),
                );
            }

            let mut selected = Vec::new();
            let mut exhausted = false;
            for spec in specs {
                let parts = match select_positioned_parts_for_selector_with_front_depth(
                    expr,
                    spec,
                    heads,
                    remaining_selector_depth - 1,
                ) {
                    BoundedTraversal::Complete(parts) => parts,
                    BoundedTraversal::Exhausted(parts) => {
                        exhausted = true;
                        parts
                    }
                };
                let Some(parts) = parts else {
                    return if exhausted {
                        BoundedTraversal::Exhausted(None)
                    } else {
                        BoundedTraversal::Complete(None)
                    };
                };
                selected.extend(parts);
            }
            if exhausted {
                BoundedTraversal::Exhausted(Some(selected))
            } else {
                BoundedTraversal::Complete(Some(selected))
            }
        }
    }
}

fn select_positioned_parts_for_non_multi_selector(
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
        PartSelector::Multi(_) => {
            unreachable!("the non-multi selector helper receives only selector leaves")
        }
    }
}

#[cold]
#[inline(never)]
fn select_positioned_parts_for_selector_iterative(
    expr: &Expr,
    selector: &PartSelector,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<(Vec<isize>, Expr)>> {
    let mut work = vec![selector];
    let mut selected = Vec::new();

    while let Some(selector) = work.pop() {
        match selector {
            PartSelector::Multi(specs) => work.extend(specs.iter().rev()),
            PartSelector::Single(_) | PartSelector::Key(_) => selected.extend(
                select_positioned_parts_for_non_multi_selector(expr, selector, heads)?,
            ),
        }
    }

    Some(selected)
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
    replace_exact_path_with_front_depth(expr, path, rhs, RECURSIVE_REPLACE_EXACT_PATH_DEPTH)
        .into_value()
}

fn replace_exact_path_with_front_depth(
    expr: Expr,
    path: &[isize],
    rhs: Expr,
    remaining_path_depth: usize,
) -> BoundedTraversal<Expr> {
    let Some((first, rest)) = path.split_first() else {
        return BoundedTraversal::Complete(rhs);
    };

    if remaining_path_depth == 0 {
        return BoundedTraversal::Exhausted(replace_exact_path_iterative(expr, path, rhs));
    }

    if *first == 0 {
        let replaced_head = replace_exact_path_with_front_depth(
            structural_head_expr(&expr),
            rest,
            rhs,
            remaining_path_depth - 1,
        );
        return match replaced_head {
            BoundedTraversal::Complete(replaced_head) => {
                BoundedTraversal::Complete(rebuild_exact_path_head(expr, replaced_head))
            }
            BoundedTraversal::Exhausted(replaced_head) => {
                BoundedTraversal::Exhausted(rebuild_exact_path_head(expr, replaced_head))
            }
        };
    }

    match expr {
        expr @ Expr::Atom(_) => BoundedTraversal::Complete(expr),
        Expr::Call { head, args } => {
            let Some(index) = normalize_structural_arg_position(*first, args.len()) else {
                return BoundedTraversal::Complete(Expr::Call { head, args });
            };

            let args = Expr::unwrap_shared_args(args);
            let replaced_child = replace_exact_path_with_front_depth(
                args[index].clone(),
                rest,
                rhs,
                remaining_path_depth - 1,
            );
            match replaced_child {
                BoundedTraversal::Complete(replaced_child) => BoundedTraversal::Complete(
                    rebuild_exact_path_child(head, args, index, replaced_child),
                ),
                BoundedTraversal::Exhausted(replaced_child) => BoundedTraversal::Exhausted(
                    rebuild_exact_path_child(head, args, index, replaced_child),
                ),
            }
        }
    }
}

fn rebuild_exact_path_head(expr: Expr, replaced_head: Expr) -> Expr {
    match expr {
        Expr::Atom(_) => Expr::call(replaced_head, Vec::new()),
        Expr::Call { args, .. } => Expr::call_with_shared_args(replaced_head, args),
    }
}

fn rebuild_exact_path_child(
    head: SharedExpr,
    mut args: Vec<Expr>,
    index: usize,
    replaced_child: Expr,
) -> Expr {
    args[index] = replaced_child;
    Expr::Call {
        head,
        args: SharedArgs::new(args),
    }
}

enum ReplaceExactPathAncestor {
    Head(Expr),
    Child {
        head: SharedExpr,
        args: Vec<Expr>,
        index: usize,
    },
}

#[cold]
#[inline(never)]
fn replace_exact_path_iterative(mut expr: Expr, path: &[isize], rhs: Expr) -> Expr {
    let mut ancestors = Vec::new();
    let mut path_index = 0;
    let mut replacement = Some(rhs);

    while let Some(component) = path.get(path_index).copied() {
        if component == 0 {
            let selected = structural_head_expr(&expr);
            ancestors.push(ReplaceExactPathAncestor::Head(expr));
            expr = selected;
            path_index += 1;
            continue;
        }

        if matches!(&expr, Expr::Atom(_)) {
            break;
        }
        let Expr::Call { head, args } = expr else {
            unreachable!("an exact-path child descent has a call after the atom check")
        };
        let Some(index) = normalize_structural_arg_position(component, args.len()) else {
            expr = Expr::Call { head, args };
            break;
        };

        let args = Expr::unwrap_shared_args(args);
        let selected = args[index].clone();
        ancestors.push(ReplaceExactPathAncestor::Child { head, args, index });
        expr = selected;
        path_index += 1;
    }

    if path_index == path.len() {
        expr = replacement
            .take()
            .expect("an exact-path replacement is consumed only at its target");
    }

    while let Some(ancestor) = ancestors.pop() {
        expr = match ancestor {
            ReplaceExactPathAncestor::Head(parent) => rebuild_exact_path_head(parent, expr),
            ReplaceExactPathAncestor::Child { head, args, index } => {
                rebuild_exact_path_child(head, args, index, expr)
            }
        };
    }

    expr
}

#[cfg(test)]
#[path = "exact_paths_tests.rs"]
mod tests;
