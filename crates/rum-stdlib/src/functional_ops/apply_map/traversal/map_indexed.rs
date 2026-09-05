use super::*;

const RECURSIVE_MAP_INDEXED_CALL_DEPTH: usize = 16;

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

pub(in crate::functional_ops::apply_map) fn map_indexed_parts_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
) -> (Expr, usize) {
    map_indexed_parts_with_front_depth(expr, operator, config, RECURSIVE_MAP_INDEXED_CALL_DEPTH)
        .into_value()
}

fn map_indexed_parts_with_front_depth(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    remaining_call_depth: usize,
) -> BoundedTraversal<(Expr, usize)> {
    match map_indexed_parts_recursive_bounded(expr, operator, config, 0, &[], remaining_call_depth)
    {
        Some(result) => BoundedTraversal::Complete(result),
        None => BoundedTraversal::Exhausted(map_indexed_parts_iterative(expr, operator, config)),
    }
}

fn map_indexed_parts_recursive_bounded(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    level: usize,
    path: &[isize],
    remaining_call_depth: usize,
) -> Option<(Expr, usize)> {
    match expr {
        Expr::Atom(_) => {
            let depth = 1;
            let rendered = if config.levels.matches(level, depth) {
                Expr::call(
                    operator.clone(),
                    vec![expr.clone(), position_expr_from_path(path)],
                )
            } else {
                expr.clone()
            };
            Some((rendered, depth))
        }
        Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
            let child_call_depth = remaining_call_depth.checked_sub(1)?;
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 1usize;
                let mut rebuilt_entries = Vec::with_capacity(entries.len());

                for (index, (key, value)) in entries.into_iter().enumerate() {
                    let child_path = extend_isize_path(path, index as isize + 1);
                    let (mapped_value, child_depth) = map_indexed_parts_recursive_bounded(
                        &value,
                        operator,
                        config,
                        level + 1,
                        &child_path,
                        child_call_depth,
                    )?;
                    max_depth = max_depth.max(child_depth);
                    rebuilt_entries.push((key, mapped_value));
                }

                let depth = 1 + max_depth;
                let rebuilt = render_association(rebuilt_entries);
                let rendered = if config.levels.matches(level, depth) {
                    Expr::call(
                        operator.clone(),
                        vec![rebuilt, position_expr_from_path(path)],
                    )
                } else {
                    rebuilt
                };

                Some((rendered, depth))
            } else {
                map_indexed_parts_on_standard_call_recursive_bounded(
                    expr,
                    operator,
                    config,
                    level,
                    path,
                    child_call_depth,
                )
            }
        }
        Expr::Call { .. } => {
            let child_call_depth = remaining_call_depth.checked_sub(1)?;
            map_indexed_parts_on_standard_call_recursive_bounded(
                expr,
                operator,
                config,
                level,
                path,
                child_call_depth,
            )
        }
    }
}

fn map_indexed_parts_on_standard_call_recursive_bounded(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    level: usize,
    path: &[isize],
    child_call_depth: usize,
) -> Option<(Expr, usize)> {
    let Expr::Call { head, args } = expr else {
        return Some((expr.clone(), 1));
    };

    let (transformed_head, head_depth) = if config.include_heads {
        let child_path = extend_isize_path(path, 0);
        map_indexed_parts_recursive_bounded(
            head,
            operator,
            config,
            level + 1,
            &child_path,
            child_call_depth,
        )?
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 1usize;
    for (index, arg) in args.iter().enumerate() {
        let child_path = extend_isize_path(path, index as isize + 1);
        let (transformed_arg, arg_depth) = map_indexed_parts_recursive_bounded(
            arg,
            operator,
            config,
            level + 1,
            &child_path,
            child_call_depth,
        )?;
        max_arg_depth = max_arg_depth.max(arg_depth);
        transformed_args.push(transformed_arg);
    }

    let depth = 1 + head_depth.max(max_arg_depth);
    let rebuilt = Expr::call(transformed_head, transformed_args);
    let rendered = if config.levels.matches(level, depth) {
        Expr::call(
            operator.clone(),
            vec![rebuilt, position_expr_from_path(path)],
        )
    } else {
        rebuilt
    };

    Some((rendered, depth))
}

enum MapIndexedWork {
    Visit {
        expr: Expr,
        level: usize,
        path: Vec<isize>,
    },
    FinishCall {
        original_head: Expr,
        arg_count: usize,
        level: usize,
        path: Vec<isize>,
    },
    FinishAssociation {
        keys: Vec<Expr>,
        level: usize,
        path: Vec<isize>,
    },
}

struct MapIndexedResult {
    expr: Expr,
    depth: usize,
}

#[cold]
#[inline(never)]
fn map_indexed_parts_iterative(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
) -> (Expr, usize) {
    let mut work = vec![MapIndexedWork::Visit {
        expr: expr.clone(),
        level: 0,
        path: Vec::new(),
    }];
    let mut completed = Vec::<MapIndexedResult>::new();

    while let Some(task) = work.pop() {
        match task {
            MapIndexedWork::Visit { expr, level, path } => match &expr {
                Expr::Atom(_) => {
                    let depth = 1;
                    let rendered = if config.levels.matches(level, depth) {
                        Expr::call(operator.clone(), vec![expr, position_expr_from_path(&path)])
                    } else {
                        expr
                    };
                    completed.push(MapIndexedResult {
                        expr: rendered,
                        depth,
                    });
                }
                Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
                    if let Some(entries) = association_entries_from_expr(&expr) {
                        schedule_map_indexed_association(&mut work, entries, level, path);
                    } else {
                        schedule_map_indexed_call(
                            &mut work,
                            expr,
                            level,
                            path,
                            config.include_heads,
                        );
                    }
                }
                Expr::Call { .. } => {
                    schedule_map_indexed_call(&mut work, expr, level, path, config.include_heads);
                }
            },
            MapIndexedWork::FinishCall {
                original_head,
                arg_count,
                level,
                path,
            } => {
                let child_count = arg_count + usize::from(config.include_heads);
                let first_child = completed
                    .len()
                    .checked_sub(child_count)
                    .expect("MapIndexed call child results remain balanced");
                let children = completed.split_off(first_child);
                let mut children = children.into_iter();
                let (transformed_head, head_depth) = if config.include_heads {
                    let result = children
                        .next()
                        .expect("head-inclusive MapIndexed completes its head");
                    (result.expr, result.depth)
                } else {
                    (original_head, 0)
                };

                let mut transformed_args = Vec::with_capacity(arg_count);
                let mut max_arg_depth = 1usize;
                for result in children {
                    transformed_args.push(result.expr);
                    max_arg_depth = max_arg_depth.max(result.depth);
                }

                let depth = 1 + head_depth.max(max_arg_depth);
                let rebuilt = Expr::call(transformed_head, transformed_args);
                let rendered = if config.levels.matches(level, depth) {
                    Expr::call(
                        operator.clone(),
                        vec![rebuilt, position_expr_from_path(&path)],
                    )
                } else {
                    rebuilt
                };
                completed.push(MapIndexedResult {
                    expr: rendered,
                    depth,
                });
            }
            MapIndexedWork::FinishAssociation { keys, level, path } => {
                let first_value = completed
                    .len()
                    .checked_sub(keys.len())
                    .expect("MapIndexed association value results remain balanced");
                let values = completed.split_off(first_value);
                let mut max_depth = 1usize;
                let mut rebuilt_entries = Vec::with_capacity(keys.len());
                for (key, result) in keys.into_iter().zip(values) {
                    max_depth = max_depth.max(result.depth);
                    rebuilt_entries.push((key, result.expr));
                }

                let depth = 1 + max_depth;
                let rebuilt = render_association(rebuilt_entries);
                let rendered = if config.levels.matches(level, depth) {
                    Expr::call(
                        operator.clone(),
                        vec![rebuilt, position_expr_from_path(&path)],
                    )
                } else {
                    rebuilt
                };
                completed.push(MapIndexedResult {
                    expr: rendered,
                    depth,
                });
            }
        }
    }

    let result = completed
        .pop()
        .expect("a MapIndexed traversal always completes its root");
    debug_assert!(completed.is_empty());
    (result.expr, result.depth)
}

fn schedule_map_indexed_call(
    work: &mut Vec<MapIndexedWork>,
    expr: Expr,
    level: usize,
    path: Vec<isize>,
    include_heads: bool,
) {
    let Expr::Call { head, args } = expr else {
        unreachable!("only calls reach the MapIndexed call scheduler");
    };
    let original_head = head.as_ref().clone();
    work.push(MapIndexedWork::FinishCall {
        original_head,
        arg_count: args.len(),
        level,
        path: path.clone(),
    });
    for (index, arg) in args.iter().enumerate().rev() {
        work.push(MapIndexedWork::Visit {
            expr: arg.clone(),
            level: level + 1,
            path: extend_isize_path(&path, index as isize + 1),
        });
    }
    if include_heads {
        work.push(MapIndexedWork::Visit {
            expr: head.into_owned(),
            level: level + 1,
            path: extend_isize_path(&path, 0),
        });
    }
}

fn schedule_map_indexed_association(
    work: &mut Vec<MapIndexedWork>,
    entries: Vec<(Expr, Expr)>,
    level: usize,
    path: Vec<isize>,
) {
    let mut keys = Vec::with_capacity(entries.len());
    let mut values = Vec::with_capacity(entries.len());
    for (key, value) in entries {
        keys.push(key);
        values.push(value);
    }

    work.push(MapIndexedWork::FinishAssociation {
        keys,
        level,
        path: path.clone(),
    });
    for (index, value) in values.into_iter().enumerate().rev() {
        work.push(MapIndexedWork::Visit {
            expr: value,
            level: level + 1,
            path: extend_isize_path(&path, index as isize + 1),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(key: &str, value: Expr) -> Expr {
        Expr::call(system_expr("Rule"), vec![Expr::string(key), value])
    }

    fn association(entries: Vec<(&str, Expr)>) -> Expr {
        Expr::call(
            system_expr("Association"),
            entries
                .into_iter()
                .map(|(key, value)| rule(key, value))
                .collect(),
        )
    }

    fn position(path: &[isize]) -> Expr {
        Expr::call(
            system_expr("List"),
            path.iter()
                .map(|component| Expr::small_integer(*component as i64))
                .collect(),
        )
    }

    fn mapped(operator: &Expr, value: Expr, path: &[isize]) -> Expr {
        Expr::call(operator.clone(), vec![value, position(path)])
    }

    fn fixture() -> Expr {
        association(vec![
            (
                "root-left",
                Expr::call(
                    system_expr("leftHead"),
                    vec![
                        association(vec![
                            ("left-deep-first", Expr::small_integer(11)),
                            ("left-deep-second", Expr::small_integer(12)),
                        ]),
                        Expr::small_integer(13),
                    ],
                ),
            ),
            (
                "root-right",
                Expr::call(
                    system_expr("rightHead"),
                    vec![Expr::small_integer(21), Expr::small_integer(22)],
                ),
            ),
        ])
    }

    fn expected(operator: &Expr) -> Expr {
        association(vec![
            (
                "root-left",
                Expr::call(
                    mapped(operator, system_expr("leftHead"), &[1, 0]),
                    vec![
                        mapped(
                            operator,
                            association(vec![
                                ("left-deep-first", Expr::small_integer(11)),
                                ("left-deep-second", Expr::small_integer(12)),
                            ]),
                            &[1, 1],
                        ),
                        mapped(operator, Expr::small_integer(13), &[1, 2]),
                    ],
                ),
            ),
            (
                "root-right",
                Expr::call(
                    mapped(operator, system_expr("rightHead"), &[2, 0]),
                    vec![
                        mapped(operator, Expr::small_integer(21), &[2, 1]),
                        mapped(operator, Expr::small_integer(22), &[2, 2]),
                    ],
                ),
            ),
        ])
    }

    #[test]
    fn map_indexed_forced_front_tail_matches_complete_front_on_branched_fixture() {
        let operator = system_expr("mapIndexedWitness");
        let config = TraversalConfig {
            levels: CasesLevelSpec::exact(2),
            include_heads: true,
        };

        let tail = map_indexed_parts_with_front_depth(&fixture(), &operator, config, 0);
        assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
        let tail = tail.into_value();

        let front = map_indexed_parts_with_front_depth(&fixture(), &operator, config, 16);
        assert!(matches!(&front, BoundedTraversal::Complete(_)));
        let front = front.into_value();

        let exact_expected = (expected(&operator), 4);
        assert_eq!(tail, front);
        assert_eq!(tail, exact_expected);
    }
}
