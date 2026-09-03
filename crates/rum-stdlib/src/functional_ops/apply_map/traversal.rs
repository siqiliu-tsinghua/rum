use super::*;

mod recursive;
use recursive::{apply_heads_recursive_bounded, map_parts_recursive_bounded};

const RECURSIVE_TRANSFORM_CALL_DEPTH: usize = 16;

pub(super) fn apply_heads_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
) -> (Expr, usize) {
    apply_heads_recursive_bounded(
        expr,
        operator,
        config,
        flavor,
        0,
        RECURSIVE_TRANSFORM_CALL_DEPTH,
    )
    .unwrap_or_else(|| {
        transform_parts_iterative(
            expr,
            operator,
            config,
            IterativeTransformFlavor::Apply(flavor),
        )
    })
}

pub(super) fn map_parts_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
) -> (Expr, usize) {
    map_parts_with_mode(expr, operator, config, MapTraversalFlavor::Map)
}

pub(super) fn map_parts_with_mode(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
) -> (Expr, usize) {
    map_parts_recursive_bounded(
        expr,
        operator,
        config,
        flavor,
        0,
        RECURSIVE_TRANSFORM_CALL_DEPTH,
    )
    .unwrap_or_else(|| {
        transform_parts_iterative(
            expr,
            operator,
            config,
            IterativeTransformFlavor::Map(flavor),
        )
    })
}

#[derive(Clone, Copy)]
enum IterativeTransformFlavor {
    Apply(ApplyFlavor),
    Map(MapTraversalFlavor),
}

enum TransformWork {
    Visit {
        expr: Expr,
        level: usize,
    },
    FinishCall {
        original_expr: Expr,
        original_head: Expr,
        arg_count: usize,
        level: usize,
    },
    FinishAssociation {
        original_expr: Expr,
        keys: Vec<Expr>,
        level: usize,
    },
}

struct TransformResult {
    expr: Expr,
    depth: usize,
    changed: bool,
}

fn transform_parts_iterative(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: IterativeTransformFlavor,
) -> (Expr, usize) {
    let mut work = vec![TransformWork::Visit {
        expr: expr.clone(),
        level: 0,
    }];
    let mut completed = Vec::<TransformResult>::new();

    while let Some(task) = work.pop() {
        match task {
            TransformWork::Visit { expr, level } => match &expr {
                Expr::Atom(_) => {
                    let transformed = match flavor {
                        IterativeTransformFlavor::Map(_) if config.levels.matches(level, 1) => {
                            Expr::call(operator.clone(), vec![expr])
                        }
                        _ => expr,
                    };
                    completed.push(TransformResult {
                        expr: transformed,
                        depth: 1,
                        changed: matches!(flavor, IterativeTransformFlavor::Map(_))
                            && config.levels.matches(level, 1),
                    });
                }
                Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
                    if let Some(entries) = association_entries_from_expr(&expr) {
                        let mut keys = Vec::with_capacity(entries.len());
                        let mut values = Vec::with_capacity(entries.len());
                        for (key, value) in entries {
                            keys.push(key);
                            values.push(value);
                        }
                        work.push(TransformWork::FinishAssociation {
                            original_expr: expr,
                            keys,
                            level,
                        });
                        for value in values.into_iter().rev() {
                            work.push(TransformWork::Visit {
                                expr: value,
                                level: level + 1,
                            });
                        }
                    } else {
                        schedule_transform_call(&mut work, expr, level, config.include_heads);
                    }
                }
                Expr::Call { .. } => {
                    schedule_transform_call(&mut work, expr, level, config.include_heads);
                }
            },
            TransformWork::FinishCall {
                original_expr,
                original_head,
                arg_count,
                level,
            } => {
                let child_count = arg_count + usize::from(config.include_heads);
                let first_child = completed
                    .len()
                    .checked_sub(child_count)
                    .expect("transform child results remain balanced");
                let children = completed.split_off(first_child);
                let mut children = children.into_iter();
                let (transformed_head, head_depth, mut children_changed) = if config.include_heads {
                    let result = children
                        .next()
                        .expect("head-inclusive traversal completes its head");
                    (result.expr, result.depth, result.changed)
                } else {
                    (original_head, 0, false)
                };
                let mut transformed_args = Vec::with_capacity(arg_count);
                let mut max_arg_depth = 1usize;
                for result in children {
                    transformed_args.push(result.expr);
                    max_arg_depth = max_arg_depth.max(result.depth);
                    children_changed |= result.changed;
                }
                let depth = head_depth.max(max_arg_depth).saturating_add(1);
                let selected = config.levels.matches(level, depth);
                let rebuilt = if children_changed {
                    Expr::call(transformed_head, transformed_args)
                } else {
                    original_expr
                };
                let transformed = match flavor {
                    IterativeTransformFlavor::Apply(_) if selected => {
                        let args = rebuilt
                            .into_call_parts()
                            .expect("a completed call remains a call")
                            .1;
                        Expr::call(operator.clone(), args)
                    }
                    IterativeTransformFlavor::Apply(_) => rebuilt,
                    IterativeTransformFlavor::Map(_) if selected => {
                        Expr::call(operator.clone(), vec![rebuilt])
                    }
                    IterativeTransformFlavor::Map(_) => rebuilt,
                };
                completed.push(TransformResult {
                    expr: transformed,
                    depth,
                    changed: children_changed || selected,
                });
            }
            TransformWork::FinishAssociation {
                original_expr,
                keys,
                level,
            } => {
                let first_value = completed
                    .len()
                    .checked_sub(keys.len())
                    .expect("association transform results remain balanced");
                let values = completed.split_off(first_value);
                let mut max_value_depth = 1usize;
                let mut transformed_values = Vec::with_capacity(values.len());
                let mut children_changed = false;
                for result in values {
                    max_value_depth = max_value_depth.max(result.depth);
                    transformed_values.push(result.expr);
                    children_changed |= result.changed;
                }
                let depth = max_value_depth.saturating_add(1);
                let selected = config.levels.matches(level, depth);
                let transformed = match flavor {
                    IterativeTransformFlavor::Apply(ApplyFlavor::Apply) if selected => {
                        Expr::call(operator.clone(), transformed_values)
                    }
                    IterativeTransformFlavor::Apply(_) if children_changed => {
                        render_association(keys.into_iter().zip(transformed_values).collect())
                    }
                    IterativeTransformFlavor::Apply(_) => original_expr,
                    IterativeTransformFlavor::Map(map_flavor) => {
                        let rebuilt = if children_changed {
                            render_association(keys.into_iter().zip(transformed_values).collect())
                        } else {
                            original_expr
                        };
                        if selected && map_flavor != MapTraversalFlavor::MapAll {
                            Expr::call(operator.clone(), vec![rebuilt])
                        } else {
                            rebuilt
                        }
                    }
                };
                let changed = match flavor {
                    IterativeTransformFlavor::Apply(ApplyFlavor::Apply) => {
                        children_changed || selected
                    }
                    IterativeTransformFlavor::Apply(_) => children_changed,
                    IterativeTransformFlavor::Map(map_flavor) => {
                        children_changed || (selected && map_flavor != MapTraversalFlavor::MapAll)
                    }
                };
                completed.push(TransformResult {
                    expr: transformed,
                    depth,
                    changed,
                });
            }
        }
    }

    let result = completed
        .pop()
        .expect("a transform traversal always completes its root");
    (result.expr, result.depth)
}

fn schedule_transform_call(
    work: &mut Vec<TransformWork>,
    expr: Expr,
    level: usize,
    include_heads: bool,
) {
    let original_expr = expr.clone();
    let Expr::Call { head, args } = expr else {
        unreachable!("only calls reach the transform call scheduler");
    };
    let original_head = head.as_ref().clone();
    work.push(TransformWork::FinishCall {
        original_expr,
        original_head,
        arg_count: args.len(),
        level,
    });
    for arg in args.iter().rev() {
        work.push(TransformWork::Visit {
            expr: arg.clone(),
            level: level + 1,
        });
    }
    if include_heads {
        work.push(TransformWork::Visit {
            expr: head.into_owned(),
            level: level + 1,
        });
    }
}

pub(super) fn map_indexed_parts_with_config(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
) -> (Expr, usize) {
    map_indexed_parts_recursive(expr, operator, config, 0, &[])
}

pub(super) fn map_indexed_parts_recursive(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    level: usize,
    path: &[isize],
) -> (Expr, usize) {
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
            (rendered, depth)
        }
        Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 1usize;
                let mut rebuilt_entries = Vec::with_capacity(entries.len());

                for (index, (key, value)) in entries.into_iter().enumerate() {
                    let child_path = extend_isize_path(path, index as isize + 1);
                    let (mapped_value, child_depth) = map_indexed_parts_recursive(
                        &value,
                        operator,
                        config,
                        level + 1,
                        &child_path,
                    );
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

                (rendered, depth)
            } else {
                map_indexed_parts_on_standard_call(expr, operator, config, level, path)
            }
        }
        Expr::Call { .. } => {
            map_indexed_parts_on_standard_call(expr, operator, config, level, path)
        }
    }
}

pub(super) fn map_indexed_parts_on_standard_call(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    level: usize,
    path: &[isize],
) -> (Expr, usize) {
    let Expr::Call { head, args } = expr else {
        return (expr.clone(), 1);
    };

    let (transformed_head, head_depth) = if config.include_heads {
        let child_path = extend_isize_path(path, 0);
        map_indexed_parts_recursive(head, operator, config, level + 1, &child_path)
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 1usize;
    for (index, arg) in args.iter().enumerate() {
        let child_path = extend_isize_path(path, index as isize + 1);
        let (transformed_arg, arg_depth) =
            map_indexed_parts_recursive(arg, operator, config, level + 1, &child_path);
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

    (rendered, depth)
}

pub(super) fn collect_scan_parts_with_config(expr: &Expr, config: TraversalConfig) -> Vec<Expr> {
    collect_scan_parts_recursive(expr, config, 0).parts
}

pub(super) fn collect_scan_parts_recursive(
    expr: &Expr,
    config: TraversalConfig,
    level: usize,
) -> ScanCollection {
    match expr {
        Expr::Atom(_) => {
            let mut parts = Vec::new();
            if config.levels.matches(level, 1) {
                parts.push(expr.clone());
            }

            ScanCollection { parts, depth: 1 }
        }
        Expr::Call { head, args: _ } if is_system_symbol_name(head, "Association") => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut parts = Vec::new();
                let mut max_depth = 1usize;

                for (_, value) in entries {
                    let child = collect_scan_parts_recursive(&value, config, level + 1);
                    max_depth = max_depth.max(child.depth);
                    parts.extend(child.parts);
                }

                let depth = 1 + max_depth;
                if config.levels.matches(level, depth) {
                    parts.push(expr.clone());
                }

                ScanCollection { parts, depth }
            } else {
                collect_scan_parts_on_standard_call(expr, config, level)
            }
        }
        Expr::Call { .. } => collect_scan_parts_on_standard_call(expr, config, level),
    }
}

pub(super) fn collect_scan_parts_on_standard_call(
    expr: &Expr,
    config: TraversalConfig,
    level: usize,
) -> ScanCollection {
    let Expr::Call { head, args } = expr else {
        return ScanCollection {
            parts: vec![expr.clone()],
            depth: 1,
        };
    };

    let mut parts = Vec::new();
    let mut max_child_depth = 1usize;

    if config.include_heads {
        let head_result = collect_scan_parts_recursive(head, config, level + 1);
        max_child_depth = max_child_depth.max(head_result.depth);
        parts.extend(head_result.parts);
    }

    for arg in args {
        let child = collect_scan_parts_recursive(arg, config, level + 1);
        max_child_depth = max_child_depth.max(child.depth);
        parts.extend(child.parts);
    }

    let depth = 1 + max_child_depth;
    if config.levels.matches(level, depth) {
        parts.push(expr.clone());
    }

    ScanCollection { parts, depth }
}
