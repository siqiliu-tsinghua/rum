use super::*;

pub(in super::super) fn recursive_delayed_wrapper_count(
    expr: &Expr,
    depth: DeleteMissingDepth,
) -> usize {
    recursive_delayed_wrapper_count_with_front_depth(
        expr,
        depth,
        RECURSIVE_DELETE_MISSING_CALL_DEPTH,
    )
}

pub(super) fn recursive_delayed_wrapper_count_with_front_depth(
    expr: &Expr,
    depth: DeleteMissingDepth,
    front_depth: usize,
) -> usize {
    match recursive_delayed_wrapper_count_bounded(expr, depth, front_depth) {
        BoundedTraversal::Complete(count) => count,
        BoundedTraversal::Exhausted => recursive_delayed_wrapper_count_iterative(expr, depth),
    }
}

fn recursive_delayed_wrapper_count_bounded(
    expr: &Expr,
    depth: DeleteMissingDepth,
    remaining_call_depth: usize,
) -> BoundedTraversal<usize> {
    let Some(child_depth) = depth.descend() else {
        return BoundedTraversal::Complete(0);
    };

    if let Some(items) = list_items(expr) {
        let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
            return BoundedTraversal::Exhausted;
        };
        let mut total = 0usize;
        for item in items {
            let BoundedTraversal::Complete(count) =
                recursive_delayed_wrapper_count_bounded(item, child_depth, child_call_depth)
            else {
                return BoundedTraversal::Exhausted;
            };
            total = total.saturating_add(count);
        }
        return BoundedTraversal::Complete(total);
    }

    let Expr::Call { head, args } = expr else {
        return BoundedTraversal::Complete(0);
    };
    if !is_system_symbol_name(head, "Association") {
        return BoundedTraversal::Complete(0);
    }
    let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
        return BoundedTraversal::Exhausted;
    };
    recursive_association_item_wrapper_count(args, child_depth, child_call_depth)
}

fn recursive_association_item_wrapper_count(
    items: &[Expr],
    child_depth: DeleteMissingDepth,
    remaining_call_depth: usize,
) -> BoundedTraversal<usize> {
    let mut total = 0usize;
    for item in items {
        let count = match item {
            Expr::Call { head, args }
                if (is_system_symbol_name(head, "Rule")
                    || is_system_symbol_name(head, "RuleDelayed"))
                    && args.len() == 2 =>
            {
                if is_system_symbol_name(head, "RuleDelayed") {
                    1
                } else {
                    let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
                        return BoundedTraversal::Exhausted;
                    };
                    let BoundedTraversal::Complete(count) = recursive_delayed_wrapper_count_bounded(
                        &args[1],
                        child_depth,
                        child_call_depth,
                    ) else {
                        return BoundedTraversal::Exhausted;
                    };
                    count
                }
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Association")
                    || is_system_symbol_name(head, "List") =>
            {
                let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
                    return BoundedTraversal::Exhausted;
                };
                let BoundedTraversal::Complete(count) =
                    recursive_association_item_wrapper_count(args, child_depth, child_call_depth)
                else {
                    return BoundedTraversal::Exhausted;
                };
                count
            }
            _ => 0,
        };
        total = total.saturating_add(count);
    }
    BoundedTraversal::Complete(total)
}

enum RecursiveWrapperCountWork<'a> {
    Expr {
        expr: &'a Expr,
        depth: DeleteMissingDepth,
    },
    AssociationItems {
        items: &'a [Expr],
        child_depth: DeleteMissingDepth,
    },
}

#[cold]
#[inline(never)]
fn recursive_delayed_wrapper_count_iterative(expr: &Expr, depth: DeleteMissingDepth) -> usize {
    let mut work = vec![RecursiveWrapperCountWork::Expr { expr, depth }];
    let mut total = 0usize;

    while let Some(task) = work.pop() {
        match task {
            RecursiveWrapperCountWork::Expr { expr, depth } => {
                let Some(child_depth) = depth.descend() else {
                    continue;
                };
                if let Some(items) = list_items(expr) {
                    work.extend(
                        items
                            .iter()
                            .rev()
                            .map(|item| RecursiveWrapperCountWork::Expr {
                                expr: item,
                                depth: child_depth,
                            }),
                    );
                    continue;
                }
                let Expr::Call { head, args } = expr else {
                    continue;
                };
                if is_system_symbol_name(head, "Association") {
                    work.push(RecursiveWrapperCountWork::AssociationItems {
                        items: args,
                        child_depth,
                    });
                }
            }
            RecursiveWrapperCountWork::AssociationItems { items, child_depth } => {
                for item in items.iter().rev() {
                    match item {
                        Expr::Call { head, args }
                            if (is_system_symbol_name(head, "Rule")
                                || is_system_symbol_name(head, "RuleDelayed"))
                                && args.len() == 2 =>
                        {
                            if is_system_symbol_name(head, "RuleDelayed") {
                                total = total.saturating_add(1);
                            } else {
                                work.push(RecursiveWrapperCountWork::Expr {
                                    expr: &args[1],
                                    depth: child_depth,
                                });
                            }
                        }
                        Expr::Call { head, args }
                            if is_system_symbol_name(head, "Association")
                                || is_system_symbol_name(head, "List") =>
                        {
                            work.push(RecursiveWrapperCountWork::AssociationItems {
                                items: args,
                                child_depth,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    total
}

pub(in super::super) fn target_delayed_wrapper_count(expr: &Expr, level: usize) -> usize {
    target_delayed_wrapper_count_with_front_depth(expr, level, RECURSIVE_DELETE_MISSING_CALL_DEPTH)
}

pub(super) fn target_delayed_wrapper_count_with_front_depth(
    expr: &Expr,
    level: usize,
    front_depth: usize,
) -> usize {
    match target_delayed_wrapper_count_bounded(expr, level, front_depth) {
        BoundedTraversal::Complete(count) => count,
        BoundedTraversal::Exhausted => target_delayed_wrapper_count_iterative(expr, level),
    }
}

fn target_delayed_wrapper_count_bounded(
    expr: &Expr,
    level: usize,
    remaining_call_depth: usize,
) -> BoundedTraversal<usize> {
    if level <= 1 {
        return BoundedTraversal::Complete(0);
    }

    if let Some(items) = list_items(expr) {
        let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
            return BoundedTraversal::Exhausted;
        };
        let mut total = 0usize;
        for item in items {
            let BoundedTraversal::Complete(count) =
                target_delayed_wrapper_count_bounded(item, level - 1, child_call_depth)
            else {
                return BoundedTraversal::Exhausted;
            };
            total = total.saturating_add(count);
        }
        return BoundedTraversal::Complete(total);
    }

    let Expr::Call { head, args } = expr else {
        return BoundedTraversal::Complete(0);
    };
    if !is_system_symbol_name(head, "Association") {
        return BoundedTraversal::Complete(0);
    }
    let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
        return BoundedTraversal::Exhausted;
    };
    target_association_item_wrapper_count(args, level - 1, child_call_depth)
}

fn target_association_item_wrapper_count(
    items: &[Expr],
    child_level: usize,
    remaining_call_depth: usize,
) -> BoundedTraversal<usize> {
    let mut total = 0usize;
    for item in items {
        let count = match item {
            Expr::Call { head, args }
                if (is_system_symbol_name(head, "Rule")
                    || is_system_symbol_name(head, "RuleDelayed"))
                    && args.len() == 2 =>
            {
                if is_system_symbol_name(head, "RuleDelayed") {
                    1
                } else {
                    let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
                        return BoundedTraversal::Exhausted;
                    };
                    let BoundedTraversal::Complete(count) = target_delayed_wrapper_count_bounded(
                        &args[1],
                        child_level,
                        child_call_depth,
                    ) else {
                        return BoundedTraversal::Exhausted;
                    };
                    count
                }
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Association")
                    || is_system_symbol_name(head, "List") =>
            {
                let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
                    return BoundedTraversal::Exhausted;
                };
                let BoundedTraversal::Complete(count) =
                    target_association_item_wrapper_count(args, child_level, child_call_depth)
                else {
                    return BoundedTraversal::Exhausted;
                };
                count
            }
            _ => 0,
        };
        total = total.saturating_add(count);
    }
    BoundedTraversal::Complete(total)
}

enum TargetWrapperCountWork<'a> {
    Expr {
        expr: &'a Expr,
        level: usize,
    },
    AssociationItems {
        items: &'a [Expr],
        child_level: usize,
    },
}

#[cold]
#[inline(never)]
fn target_delayed_wrapper_count_iterative(expr: &Expr, level: usize) -> usize {
    let mut work = vec![TargetWrapperCountWork::Expr { expr, level }];
    let mut total = 0usize;

    while let Some(task) = work.pop() {
        match task {
            TargetWrapperCountWork::Expr { expr, level } => {
                if level <= 1 {
                    continue;
                }
                if let Some(items) = list_items(expr) {
                    work.extend(items.iter().rev().map(|item| TargetWrapperCountWork::Expr {
                        expr: item,
                        level: level - 1,
                    }));
                    continue;
                }
                let Expr::Call { head, args } = expr else {
                    continue;
                };
                if is_system_symbol_name(head, "Association") {
                    work.push(TargetWrapperCountWork::AssociationItems {
                        items: args,
                        child_level: level - 1,
                    });
                }
            }
            TargetWrapperCountWork::AssociationItems { items, child_level } => {
                for item in items.iter().rev() {
                    match item {
                        Expr::Call { head, args }
                            if (is_system_symbol_name(head, "Rule")
                                || is_system_symbol_name(head, "RuleDelayed"))
                                && args.len() == 2 =>
                        {
                            if is_system_symbol_name(head, "RuleDelayed") {
                                total = total.saturating_add(1);
                            } else {
                                work.push(TargetWrapperCountWork::Expr {
                                    expr: &args[1],
                                    level: child_level,
                                });
                            }
                        }
                        Expr::Call { head, args }
                            if is_system_symbol_name(head, "Association")
                                || is_system_symbol_name(head, "List") =>
                        {
                            work.push(TargetWrapperCountWork::AssociationItems {
                                items: args,
                                child_level,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system_call(name: &str, args: Vec<Expr>) -> Expr {
        Expr::call(system_expr(name), args)
    }

    fn rule(name: &str, value: Expr) -> Expr {
        system_call("Rule", vec![Expr::string(name), value])
    }

    fn rule_delayed(name: &str, value: Expr) -> Expr {
        system_call("RuleDelayed", vec![Expr::string(name), value])
    }

    fn list(items: Vec<Expr>) -> Expr {
        system_call("List", items)
    }

    fn association(items: Vec<Expr>) -> Expr {
        system_call("Association", items)
    }

    fn missing() -> Expr {
        system_call("Missing", Vec::new())
    }

    #[test]
    fn zero_front_uses_heap_tails_for_delayed_wrapper_counts() {
        let fixture = association(vec![
            rule_delayed("left", list(vec![missing()])),
            rule(
                "middle",
                association(vec![rule_delayed("right", Expr::small_integer(7))]),
            ),
        ]);

        assert_eq!(
            recursive_delayed_wrapper_count_with_front_depth(
                &fixture,
                DeleteMissingDepth::Infinity,
                0,
            ),
            2
        );
        assert_eq!(
            target_delayed_wrapper_count_with_front_depth(&fixture, 3, 0),
            2
        );
    }
}
