use super::*;

pub(in super::super) fn delete_missing_from_keyed_data_recursive(
    expr: &Expr,
    depth: DeleteMissingDepth,
) -> Option<Expr> {
    delete_missing_from_keyed_data_with_front_depth(
        expr,
        depth,
        RECURSIVE_DELETE_MISSING_CALL_DEPTH,
    )
}

pub(super) fn delete_missing_from_keyed_data_with_front_depth(
    expr: &Expr,
    depth: DeleteMissingDepth,
    front_depth: usize,
) -> Option<Expr> {
    match delete_missing_from_keyed_data_recursive_bounded(expr, depth, front_depth) {
        BoundedTraversal::Complete(value) => value,
        BoundedTraversal::Exhausted => delete_missing_from_keyed_data_iterative(expr, depth),
    }
}

fn delete_missing_from_keyed_data_recursive_bounded(
    expr: &Expr,
    depth: DeleteMissingDepth,
    remaining_call_depth: usize,
) -> BoundedTraversal<Option<Expr>> {
    let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
        return BoundedTraversal::Exhausted;
    };

    if let Some(items) = list_items(expr) {
        return match delete_missing_from_list_recursive(items, depth, child_call_depth) {
            BoundedTraversal::Complete(value) => BoundedTraversal::Complete(Some(value)),
            BoundedTraversal::Exhausted => BoundedTraversal::Exhausted,
        };
    }

    let Some(entries) = association_entry_forms_from_expr(expr) else {
        return BoundedTraversal::Complete(None);
    };
    match delete_missing_from_association_entries_recursive(entries, depth, child_call_depth) {
        BoundedTraversal::Complete(entries) => BoundedTraversal::Complete(Some(
            render_association_entry_forms_preserving_heads(entries),
        )),
        BoundedTraversal::Exhausted => BoundedTraversal::Exhausted,
    }
}

fn delete_missing_from_list_recursive(
    items: &[Expr],
    depth: DeleteMissingDepth,
    remaining_call_depth: usize,
) -> BoundedTraversal<Expr> {
    let mut kept = Vec::with_capacity(items.len());
    for item in items {
        let BoundedTraversal::Complete(item) =
            delete_missing_list_item_recursive(item, depth, remaining_call_depth)
        else {
            return BoundedTraversal::Exhausted;
        };
        if let Some(item) = item {
            kept.push(item);
        }
    }
    BoundedTraversal::Complete(Expr::call(system_expr("List"), kept))
}

fn delete_missing_list_item_recursive(
    item: &Expr,
    depth: DeleteMissingDepth,
    remaining_call_depth: usize,
) -> BoundedTraversal<Option<Expr>> {
    if is_missing_expr(item) || rule_like_expr_has_missing_value(item) {
        return BoundedTraversal::Complete(None);
    }

    if let Some(child_depth) = depth.descend() {
        if list_items(item).is_some() || association_entry_forms_from_expr(item).is_some() {
            return delete_missing_from_keyed_data_recursive_bounded(
                item,
                child_depth,
                remaining_call_depth,
            );
        }
    }

    BoundedTraversal::Complete(Some(item.clone()))
}

fn delete_missing_from_association_entries_recursive(
    mut entries: Vec<AssociationEntryForm>,
    depth: DeleteMissingDepth,
    remaining_call_depth: usize,
) -> BoundedTraversal<Vec<AssociationEntryForm>> {
    let mut exhausted = false;
    entries.retain_mut(|entry| {
        if exhausted {
            return true;
        }
        if is_missing_expr(&entry.value) {
            return false;
        }

        if !entry.delayed {
            if let Some(child_depth) = depth.descend() {
                let BoundedTraversal::Complete(value) =
                    delete_missing_from_keyed_data_recursive_bounded(
                        &entry.value,
                        child_depth,
                        remaining_call_depth,
                    )
                else {
                    exhausted = true;
                    return true;
                };
                if let Some(value) = value {
                    entry.value = value;
                }
            }
        } else if let Some(child_depth) = depth.descend() {
            let value = std::mem::replace(&mut entry.value, Expr::small_integer(0));
            entry.value =
                delete_missing_delayed_wrapper(value, DeleteMissingMode::Recursive(child_depth));
        }

        true
    });

    if exhausted {
        BoundedTraversal::Exhausted
    } else {
        BoundedTraversal::Complete(entries)
    }
}

enum DeleteMissingWork {
    VisitKeyed {
        expr: Expr,
        depth: DeleteMissingDepth,
    },
    VisitListItem {
        item: Expr,
        depth: DeleteMissingDepth,
    },
    VisitAssociationEntry {
        entry: AssociationEntryForm,
        depth: DeleteMissingDepth,
    },
    FinishList {
        item_count: usize,
    },
    FinishListItemFromChild {
        original: Expr,
    },
    FinishAssociation {
        entry_count: usize,
    },
    FinishAssociationEntryFromChild {
        entry: AssociationEntryForm,
    },
}

enum DeleteMissingResult {
    Keyed(Option<Expr>),
    ListItem(Option<Expr>),
    AssociationEntry(Option<AssociationEntryForm>),
}

#[cold]
#[inline(never)]
fn delete_missing_from_keyed_data_iterative(
    expr: &Expr,
    depth: DeleteMissingDepth,
) -> Option<Expr> {
    let mut work = vec![DeleteMissingWork::VisitKeyed {
        expr: expr.clone(),
        depth,
    }];
    let mut completed = Vec::<DeleteMissingResult>::new();

    while let Some(task) = work.pop() {
        match task {
            DeleteMissingWork::VisitKeyed { expr, depth } => {
                if let Some(items) = list_items(&expr) {
                    work.push(DeleteMissingWork::FinishList {
                        item_count: items.len(),
                    });
                    work.extend(
                        items
                            .iter()
                            .rev()
                            .map(|item| DeleteMissingWork::VisitListItem {
                                item: item.clone(),
                                depth,
                            }),
                    );
                    continue;
                }

                let Some(entries) = association_entry_forms_from_expr(&expr) else {
                    completed.push(DeleteMissingResult::Keyed(None));
                    continue;
                };
                work.push(DeleteMissingWork::FinishAssociation {
                    entry_count: entries.len(),
                });
                work.extend(
                    entries
                        .into_iter()
                        .rev()
                        .map(|entry| DeleteMissingWork::VisitAssociationEntry { entry, depth }),
                );
            }
            DeleteMissingWork::VisitListItem { item, depth } => {
                if is_missing_expr(&item) || rule_like_expr_has_missing_value(&item) {
                    completed.push(DeleteMissingResult::ListItem(None));
                    continue;
                }

                let Some(child_depth) = depth.descend() else {
                    completed.push(DeleteMissingResult::ListItem(Some(item)));
                    continue;
                };
                work.push(DeleteMissingWork::FinishListItemFromChild {
                    original: item.clone(),
                });
                work.push(DeleteMissingWork::VisitKeyed {
                    expr: item,
                    depth: child_depth,
                });
            }
            DeleteMissingWork::VisitAssociationEntry { mut entry, depth } => {
                if is_missing_expr(&entry.value) {
                    completed.push(DeleteMissingResult::AssociationEntry(None));
                    continue;
                }

                let Some(child_depth) = depth.descend() else {
                    completed.push(DeleteMissingResult::AssociationEntry(Some(entry)));
                    continue;
                };
                if entry.delayed {
                    entry.value = delete_missing_delayed_wrapper(
                        entry.value,
                        DeleteMissingMode::Recursive(child_depth),
                    );
                    completed.push(DeleteMissingResult::AssociationEntry(Some(entry)));
                    continue;
                }

                work.push(DeleteMissingWork::FinishAssociationEntryFromChild {
                    entry: entry.clone(),
                });
                work.push(DeleteMissingWork::VisitKeyed {
                    expr: entry.value,
                    depth: child_depth,
                });
            }
            DeleteMissingWork::FinishList { item_count } => {
                let first_item = completed
                    .len()
                    .checked_sub(item_count)
                    .expect("DeleteMissing list results remain balanced");
                let items = completed
                    .split_off(first_item)
                    .into_iter()
                    .filter_map(|result| match result {
                        DeleteMissingResult::ListItem(item) => item,
                        _ => panic!("DeleteMissing list finishes only list items"),
                    })
                    .collect();
                completed.push(DeleteMissingResult::Keyed(Some(Expr::call(
                    system_expr("List"),
                    items,
                ))));
            }
            DeleteMissingWork::FinishListItemFromChild { original } => {
                let child = match completed.pop() {
                    Some(DeleteMissingResult::Keyed(child)) => child,
                    _ => panic!("DeleteMissing list child result remains balanced"),
                };
                completed.push(DeleteMissingResult::ListItem(Some(
                    child.unwrap_or(original),
                )));
            }
            DeleteMissingWork::FinishAssociation { entry_count } => {
                let first_entry = completed
                    .len()
                    .checked_sub(entry_count)
                    .expect("DeleteMissing association results remain balanced");
                let entries = completed
                    .split_off(first_entry)
                    .into_iter()
                    .filter_map(|result| match result {
                        DeleteMissingResult::AssociationEntry(entry) => entry,
                        _ => panic!("DeleteMissing association finishes only entries"),
                    })
                    .collect();
                completed.push(DeleteMissingResult::Keyed(Some(
                    render_association_entry_forms_preserving_heads(entries),
                )));
            }
            DeleteMissingWork::FinishAssociationEntryFromChild { mut entry } => {
                let child = match completed.pop() {
                    Some(DeleteMissingResult::Keyed(child)) => child,
                    _ => panic!("DeleteMissing association child result remains balanced"),
                };
                if let Some(child) = child {
                    entry.value = child;
                }
                completed.push(DeleteMissingResult::AssociationEntry(Some(entry)));
            }
        }
    }

    match completed.pop() {
        Some(DeleteMissingResult::Keyed(value)) if completed.is_empty() => value,
        _ => panic!("DeleteMissing traversal completes with one keyed result"),
    }
}

pub(in super::super) fn contains_missing_within(expr: &Expr, depth: DeleteMissingDepth) -> bool {
    contains_missing_with_front_depth(expr, depth, RECURSIVE_DELETE_MISSING_CALL_DEPTH)
}

pub(super) fn contains_missing_with_front_depth(
    expr: &Expr,
    depth: DeleteMissingDepth,
    front_depth: usize,
) -> bool {
    match contains_missing_within_bounded(expr, depth, front_depth) {
        BoundedTraversal::Complete(found) => found,
        BoundedTraversal::Exhausted => contains_missing_within_iterative(expr, depth),
    }
}

fn contains_missing_within_bounded(
    expr: &Expr,
    depth: DeleteMissingDepth,
    remaining_call_depth: usize,
) -> BoundedTraversal<bool> {
    if is_missing_expr(expr) {
        return BoundedTraversal::Complete(true);
    }

    let Some(child_depth) = depth.descend_scan() else {
        return BoundedTraversal::Complete(false);
    };
    let Expr::Call { args, .. } = expr else {
        return BoundedTraversal::Complete(false);
    };
    let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
        return BoundedTraversal::Exhausted;
    };

    for arg in args {
        match contains_missing_within_bounded(arg, child_depth, child_call_depth) {
            BoundedTraversal::Complete(true) => return BoundedTraversal::Complete(true),
            BoundedTraversal::Complete(false) => {}
            BoundedTraversal::Exhausted => return BoundedTraversal::Exhausted,
        }
    }
    BoundedTraversal::Complete(false)
}

#[cold]
#[inline(never)]
fn contains_missing_within_iterative(expr: &Expr, depth: DeleteMissingDepth) -> bool {
    let mut work = vec![(expr, depth)];

    while let Some((expr, depth)) = work.pop() {
        if is_missing_expr(expr) {
            return true;
        }
        let Some(child_depth) = depth.descend_scan() else {
            continue;
        };
        let Expr::Call { args, .. } = expr else {
            continue;
        };
        work.extend(args.iter().rev().map(|arg| (arg, child_depth)));
    }

    false
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

    fn association(items: Vec<Expr>) -> Expr {
        system_call("Association", items)
    }

    fn missing() -> Expr {
        system_call("Missing", Vec::new())
    }

    #[test]
    fn zero_front_heap_tail_matches_front_for_branched_associations() {
        const OUTER_LEVELS: i64 = 3;

        let mut fixture = association(vec![
            rule("x", missing()),
            rule("y", Expr::string("deep-survivor")),
        ]);
        let mut expected = association(vec![rule("y", Expr::string("deep-survivor"))]);
        for level in (1..=OUTER_LEVELS).rev() {
            fixture = association(vec![
                rule("drop", missing()),
                rule("k", fixture),
                rule("keep", Expr::small_integer(level)),
            ]);
            expected = association(vec![
                rule("k", expected),
                rule("keep", Expr::small_integer(level)),
            ]);
        }

        assert!(matches!(
            delete_missing_from_keyed_data_recursive_bounded(
                &fixture,
                DeleteMissingDepth::Infinity,
                0,
            ),
            BoundedTraversal::Exhausted
        ));
        assert!(matches!(
            delete_missing_from_keyed_data_recursive_bounded(
                &fixture,
                DeleteMissingDepth::Infinity,
                RECURSIVE_DELETE_MISSING_CALL_DEPTH,
            ),
            BoundedTraversal::Complete(_)
        ));

        let through_tail = delete_missing_from_keyed_data_with_front_depth(
            &fixture,
            DeleteMissingDepth::Infinity,
            0,
        )
        .expect("forced heap tail should return keyed data");
        let through_front = delete_missing_from_keyed_data_with_front_depth(
            &fixture,
            DeleteMissingDepth::Infinity,
            RECURSIVE_DELETE_MISSING_CALL_DEPTH,
        )
        .expect("bounded front should return keyed data");
        assert_eq!(through_tail, through_front);
        assert_eq!(through_tail, expected);

        let mut current = through_tail;
        for level in 1..=OUTER_LEVELS {
            let entries = association_entry_forms_from_expr(&current)
                .expect("each outer result level should remain an association");
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].key, Expr::string("k"));
            assert_eq!(entries[1].key, Expr::string("keep"));
            assert_eq!(entries[1].value, Expr::small_integer(level));
            current = entries[0].value.clone();
        }
        let deepest = association_entry_forms_from_expr(&current)
            .expect("deepest result should remain an association");
        assert_eq!(deepest.len(), 1);
        assert_eq!(deepest[0].key, Expr::string("y"));
        assert_eq!(deepest[0].value, Expr::string("deep-survivor"));

        assert!(contains_missing_with_front_depth(
            &fixture,
            DeleteMissingDepth::Infinity,
            0,
        ));
        assert!(!contains_missing_with_front_depth(
            &current,
            DeleteMissingDepth::Infinity,
            0,
        ));
    }
}
