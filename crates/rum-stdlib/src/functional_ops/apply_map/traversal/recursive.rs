use super::*;

pub(super) fn apply_heads_recursive_bounded(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
    level: usize,
    remaining_call_depth: usize,
) -> Option<(Expr, usize)> {
    match expr {
        Expr::Atom(_) => Some((expr.clone(), 1)),
        Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
            let child_call_depth = remaining_call_depth.checked_sub(1)?;
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 1usize;
                let mut transformed_entries = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    let (transformed_value, value_depth) = apply_heads_recursive_bounded(
                        &value,
                        operator,
                        config,
                        flavor,
                        level + 1,
                        child_call_depth,
                    )?;
                    max_depth = max_depth.max(value_depth);
                    transformed_entries.push((key, transformed_value));
                }

                let depth = max_depth.saturating_add(1);
                let transformed =
                    if config.levels.matches(level, depth) && flavor == ApplyFlavor::Apply {
                        Expr::call(
                            operator.clone(),
                            transformed_entries
                                .iter()
                                .map(|(_, value)| value.clone())
                                .collect(),
                        )
                    } else {
                        render_association(transformed_entries)
                    };
                Some((transformed, depth))
            } else {
                apply_heads_standard_call_recursive_bounded(
                    expr,
                    operator,
                    config,
                    flavor,
                    level,
                    child_call_depth,
                )
            }
        }
        Expr::Call { .. } => {
            let child_call_depth = remaining_call_depth.checked_sub(1)?;
            apply_heads_standard_call_recursive_bounded(
                expr,
                operator,
                config,
                flavor,
                level,
                child_call_depth,
            )
        }
    }
}

fn apply_heads_standard_call_recursive_bounded(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: ApplyFlavor,
    level: usize,
    child_call_depth: usize,
) -> Option<(Expr, usize)> {
    let Expr::Call { head, args, .. } = expr else {
        unreachable!("only calls reach the bounded recursive Apply transform");
    };

    let (transformed_head, head_depth) = if config.include_heads {
        apply_heads_recursive_bounded(head, operator, config, flavor, level + 1, child_call_depth)?
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 1usize;
    for arg in args {
        let (transformed_arg, arg_depth) = apply_heads_recursive_bounded(
            arg,
            operator,
            config,
            flavor,
            level + 1,
            child_call_depth,
        )?;
        max_arg_depth = max_arg_depth.max(arg_depth);
        transformed_args.push(transformed_arg);
    }

    let depth = head_depth.max(max_arg_depth).saturating_add(1);
    let rebuilt = Expr::call(transformed_head, transformed_args.clone());
    let transformed = if config.levels.matches(level, depth) {
        Expr::call(operator.clone(), transformed_args)
    } else {
        rebuilt
    };
    Some((transformed, depth))
}

pub(super) fn map_parts_recursive_bounded(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
    level: usize,
    remaining_call_depth: usize,
) -> Option<(Expr, usize)> {
    match expr {
        Expr::Atom(_) => {
            let transformed = if config.levels.matches(level, 1) {
                Expr::call(operator.clone(), vec![expr.clone()])
            } else {
                expr.clone()
            };
            Some((transformed, 1))
        }
        Expr::Call { head, .. } if is_system_symbol_name(head, "Association") => {
            let child_call_depth = remaining_call_depth.checked_sub(1)?;
            if let Some(entries) = association_entries_from_expr(expr) {
                let mut max_depth = 1usize;
                let mut transformed_entries = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    let (transformed_value, value_depth) = map_parts_recursive_bounded(
                        &value,
                        operator,
                        config,
                        flavor,
                        level + 1,
                        child_call_depth,
                    )?;
                    max_depth = max_depth.max(value_depth);
                    transformed_entries.push((key, transformed_value));
                }

                let depth = max_depth.saturating_add(1);
                let rebuilt = render_association(transformed_entries);
                let transformed = if config.levels.matches(level, depth)
                    && flavor != MapTraversalFlavor::MapAll
                {
                    Expr::call(operator.clone(), vec![rebuilt])
                } else {
                    rebuilt
                };
                Some((transformed, depth))
            } else {
                map_parts_standard_call_recursive_bounded(
                    expr,
                    operator,
                    config,
                    flavor,
                    level,
                    child_call_depth,
                )
            }
        }
        Expr::Call { .. } => {
            let child_call_depth = remaining_call_depth.checked_sub(1)?;
            map_parts_standard_call_recursive_bounded(
                expr,
                operator,
                config,
                flavor,
                level,
                child_call_depth,
            )
        }
    }
}

fn map_parts_standard_call_recursive_bounded(
    expr: &Expr,
    operator: &Expr,
    config: TraversalConfig,
    flavor: MapTraversalFlavor,
    level: usize,
    child_call_depth: usize,
) -> Option<(Expr, usize)> {
    let Expr::Call { head, args, .. } = expr else {
        unreachable!("only calls reach the bounded recursive Map transform");
    };

    let (transformed_head, head_depth) = if config.include_heads {
        map_parts_recursive_bounded(head, operator, config, flavor, level + 1, child_call_depth)?
    } else {
        (head.as_ref().clone(), 0)
    };

    let mut transformed_args = Vec::with_capacity(args.len());
    let mut max_arg_depth = 1usize;
    for arg in args {
        let (transformed_arg, arg_depth) = map_parts_recursive_bounded(
            arg,
            operator,
            config,
            flavor,
            level + 1,
            child_call_depth,
        )?;
        max_arg_depth = max_arg_depth.max(arg_depth);
        transformed_args.push(transformed_arg);
    }

    let depth = head_depth.max(max_arg_depth).saturating_add(1);
    let rebuilt = Expr::call(transformed_head, transformed_args);
    let transformed = if config.levels.matches(level, depth) {
        Expr::call(operator.clone(), vec![rebuilt])
    } else {
        rebuilt
    };
    Some((transformed, depth))
}
