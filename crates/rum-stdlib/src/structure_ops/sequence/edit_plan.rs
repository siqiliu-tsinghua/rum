use super::*;

#[cfg(test)]
std::thread_local! {
    static INSERT_EDITED_ELEMENTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DELETE_EDITED_ELEMENTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn reset_insert_parent_scan_count() {
    INSERT_EDITED_ELEMENTS.set(0);
}

#[cfg(test)]
pub(super) fn insert_parent_scan_count() -> usize {
    INSERT_EDITED_ELEMENTS.get()
}

#[cfg(test)]
pub(super) fn reset_delete_parent_scan_count() {
    DELETE_EDITED_ELEMENTS.set(0);
}

#[cfg(test)]
pub(super) fn delete_parent_scan_count() -> usize {
    DELETE_EDITED_ELEMENTS.get()
}

#[cold]
#[inline(never)]
pub(super) fn batch_insert_canonical_paths(expr: Expr, item: &Expr, paths: &[Vec<usize>]) -> Expr {
    insert_paths_at_depth(expr, item, paths, 0)
}

#[cold]
#[inline(never)]
fn insert_paths_at_depth(expr: Expr, item: &Expr, paths: &[Vec<usize>], depth: usize) -> Expr {
    let Expr::Call { head, args, .. } = expr else {
        return expr;
    };
    let original_len = args.len();
    let mut child_edits = Vec::new();
    let mut insertion_positions = Vec::new();
    let mut group_start = 0usize;

    while group_start < paths.len() {
        let component = paths[group_start][depth];
        let mut group_end = group_start + 1;
        while group_end < paths.len() && paths[group_end][depth] == component {
            group_end += 1;
        }
        let group = &paths[group_start..group_end];
        let deeper_end = group
            .iter()
            .position(|path| path.len() == depth + 1)
            .unwrap_or(group.len());

        if deeper_end > 0 {
            debug_assert!(component < args.len());
            child_edits.push((
                component,
                insert_paths_at_depth(
                    args[component].clone(),
                    item,
                    &group[..deeper_end],
                    depth + 1,
                ),
            ));
        }
        if deeper_end < group.len() {
            debug_assert_eq!(group.len() - deeper_end, 1);
            insertion_positions.push(component);
        }
        group_start = group_end;
    }

    insertion_positions.reverse();
    child_edits.reverse();
    #[cfg(test)]
    INSERT_EDITED_ELEMENTS.set(INSERT_EDITED_ELEMENTS.get() + args.len());
    let mut output = Vec::with_capacity(original_len + insertion_positions.len());
    let mut insertion_index = 0usize;
    let mut child_edits = child_edits.into_iter().peekable();
    for boundary in 0..=original_len {
        if insertion_positions.get(insertion_index) == Some(&boundary) {
            output.push(item.clone());
            insertion_index += 1;
        }
        if boundary < original_len {
            if child_edits.peek().map(|(index, _)| *index) == Some(boundary) {
                output.push(
                    child_edits
                        .next()
                        .expect("a peeked child edit remains available")
                        .1,
                );
            } else {
                output.push(args[boundary].clone());
            }
        }
    }

    Expr::call_shared(head, SharedArgs::new(output))
}

#[cold]
#[inline(never)]
pub(super) fn batch_delete_canonical_paths(expr: Expr, paths: &[Vec<isize>]) -> Expr {
    delete_paths_at_depth(expr, paths, 0)
}

#[cold]
#[inline(never)]
fn delete_paths_at_depth(expr: Expr, paths: &[Vec<isize>], depth: usize) -> Expr {
    let Expr::Call { head, args, .. } = expr else {
        return expr;
    };
    let mut replaced_head = None;
    let mut child_edits = Vec::new();
    let mut delete_head = false;
    let mut deletion_indices = Vec::new();
    let mut group_start = 0usize;

    while group_start < paths.len() {
        let component = paths[group_start][depth];
        let mut group_end = group_start + 1;
        while group_end < paths.len() && paths[group_end][depth] == component {
            group_end += 1;
        }
        let group = &paths[group_start..group_end];
        let deeper_end = group
            .iter()
            .position(|path| path.len() == depth + 1)
            .unwrap_or(group.len());
        let deletes_component = deeper_end < group.len();

        if component == 0 {
            if deletes_component {
                delete_head = true;
            } else {
                replaced_head = Some(delete_paths_at_depth(
                    head.as_ref().clone(),
                    &group[..deeper_end],
                    depth + 1,
                ));
            }
        } else {
            let index = component as usize - 1;
            debug_assert!(index < args.len());
            if deletes_component {
                deletion_indices.push(index);
            } else {
                child_edits.push((
                    index,
                    delete_paths_at_depth(args[index].clone(), &group[..deeper_end], depth + 1),
                ));
            }
        }
        group_start = group_end;
    }

    if deletion_indices.is_empty() && child_edits.is_empty() && !delete_head {
        return match replaced_head {
            Some(head) => Expr::call_with_shared_args(head, args),
            None => Expr::call_shared(head, args),
        };
    }

    deletion_indices.reverse();
    child_edits.reverse();
    #[cfg(test)]
    DELETE_EDITED_ELEMENTS.set(DELETE_EDITED_ELEMENTS.get() + args.len());
    let mut output = Vec::with_capacity(args.len() - deletion_indices.len());
    let mut deletion_index = 0usize;
    let mut child_edits = child_edits.into_iter().peekable();
    for (original_index, arg) in args.iter().enumerate() {
        if deletion_indices.get(deletion_index) == Some(&original_index) {
            deletion_index += 1;
            continue;
        }
        if child_edits.peek().map(|(index, _)| *index) == Some(original_index) {
            output.push(
                child_edits
                    .next()
                    .expect("a peeked child edit remains available")
                    .1,
            );
        } else {
            output.push(arg.clone());
        }
    }
    if delete_head {
        return expression_without_head(output);
    }

    let args = SharedArgs::new(output);
    match replaced_head {
        Some(head) => Expr::call_with_shared_args(head, args),
        None => Expr::call_shared(head, args),
    }
}

fn expression_without_head(args: Vec<Expr>) -> Expr {
    if args.len() == 1 {
        return args
            .into_iter()
            .next()
            .expect("a one-element argument vector has one value");
    }
    Expr::call(system_expr("Sequence"), args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn many_insertions_use_one_linear_parent_pass() {
        let expr = list_of_integers(64);
        let mut paths = (0..64).map(|index| vec![index]).collect::<Vec<_>>();
        paths.sort_by(|left, right| right.cmp(left));
        INSERT_EDITED_ELEMENTS.set(0);

        let inserted = batch_insert_canonical_paths(expr, &Expr::symbol("x"), &paths);

        assert_eq!(INSERT_EDITED_ELEMENTS.get(), 64);
        assert_eq!(call_len(&inserted), 128);
    }

    #[test]
    fn many_deletions_use_one_linear_parent_pass() {
        let expr = list_of_integers(64);
        let mut paths = (0..64)
            .step_by(2)
            .map(|index| vec![index + 1])
            .collect::<Vec<_>>();
        paths.sort_by(|left, right| right.cmp(left));
        DELETE_EDITED_ELEMENTS.set(0);

        let deleted = batch_delete_canonical_paths(expr, &paths);

        assert_eq!(DELETE_EDITED_ELEMENTS.get(), 64);
        assert_eq!(call_len(&deleted), 32);
    }

    #[test]
    fn batched_insert_matches_descending_sequential_reference_with_overlaps() {
        let expr = nested_fixture();
        let paths = vec![vec![0], vec![0, 1], vec![1], vec![1, 0], vec![2]];

        assert_eq!(
            batch_insert_paths(expr.clone(), &Expr::symbol("x"), paths.clone()),
            reference_insert_paths(expr, &Expr::symbol("x"), paths),
        );
    }

    #[test]
    fn batched_delete_matches_descending_sequential_reference_with_overlaps_and_head() {
        let expr = nested_fixture();
        let paths = vec![vec![0], vec![0, 1], vec![1], vec![1, 1], vec![2, 1]];

        assert_eq!(
            batch_delete_paths(expr.clone(), paths.clone()),
            reference_delete_paths(expr, paths),
        );
    }

    #[test]
    fn batched_insert_matches_reference_for_every_nested_path_subset() {
        let catalog = [
            vec![0],
            vec![1],
            vec![2],
            vec![0, 0],
            vec![0, 1],
            vec![0, 2],
            vec![1, 0],
            vec![1, 1],
        ];
        for mask in 1usize..(1usize << catalog.len()) {
            let paths = selected_paths(&catalog, mask);
            let expr = nested_fixture();
            assert_eq!(
                batch_insert_paths(expr.clone(), &Expr::symbol("x"), paths.clone()),
                reference_insert_paths(expr, &Expr::symbol("x"), paths),
                "insert path subset {mask:#010b}",
            );
        }
    }

    #[test]
    fn batched_delete_matches_reference_for_every_nested_path_subset() {
        let catalog = [
            vec![0],
            vec![1],
            vec![2],
            vec![1, 0],
            vec![1, 1],
            vec![1, 2],
            vec![2, 0],
            vec![2, 1],
        ];
        for mask in 1usize..(1usize << catalog.len()) {
            let paths = selected_paths(&catalog, mask);
            let expr = nested_fixture();
            assert_eq!(
                batch_delete_paths(expr.clone(), paths.clone()),
                reference_delete_paths(expr, paths),
                "delete path subset {mask:#010b}",
            );
        }
    }

    fn selected_paths<T: Clone>(catalog: &[Vec<T>], mask: usize) -> Vec<Vec<T>> {
        catalog
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1usize << index) != 0)
            .map(|(_, path)| path.clone())
            .collect()
    }

    fn batch_insert_paths(expr: Expr, item: &Expr, mut paths: Vec<Vec<usize>>) -> Expr {
        paths.sort_by(|left, right| right.cmp(left));
        paths.dedup();
        batch_insert_canonical_paths(expr, item, &paths)
    }

    fn batch_delete_paths(expr: Expr, mut paths: Vec<Vec<isize>>) -> Expr {
        paths.sort_by(|left, right| right.cmp(left));
        paths.dedup();
        batch_delete_canonical_paths(expr, &paths)
    }

    fn list_of_integers(len: usize) -> Expr {
        Expr::call(
            system_expr(heads::structure::LIST),
            (0..len)
                .map(|value| Expr::integer(BigInt::from(value)))
                .collect(),
        )
    }

    fn nested_fixture() -> Expr {
        Expr::call(
            Expr::symbol("f"),
            vec![
                Expr::call(
                    Expr::symbol("g"),
                    vec![Expr::symbol("a"), Expr::symbol("b")],
                ),
                Expr::call(Expr::symbol("h"), vec![Expr::symbol("c")]),
            ],
        )
    }

    fn call_len(expr: &Expr) -> usize {
        match expr {
            Expr::Call { args, .. } => args.len(),
            Expr::Atom(_) => 0,
        }
    }

    fn reference_insert_paths(expr: Expr, item: &Expr, mut paths: Vec<Vec<usize>>) -> Expr {
        paths.sort_by(|left, right| right.cmp(left));
        paths.dedup();
        paths
            .into_iter()
            .fold(expr, |expr, path| reference_insert_path(expr, &path, item))
    }

    fn reference_insert_path(expr: Expr, path: &[usize], item: &Expr) -> Expr {
        let Some((first, rest)) = path.split_first() else {
            return expr;
        };
        let Expr::Call { head, args, .. } = expr else {
            return expr;
        };
        let mut args = Expr::unwrap_shared_args(args);
        if rest.is_empty() {
            args.insert(*first, item.clone());
        } else {
            args[*first] = reference_insert_path(args[*first].clone(), rest, item);
        }
        Expr::call_shared(head, SharedArgs::new(args))
    }

    fn reference_delete_paths(expr: Expr, mut paths: Vec<Vec<isize>>) -> Expr {
        paths.sort_by(|left, right| right.cmp(left));
        paths.dedup();
        paths
            .into_iter()
            .fold(expr, |expr, path| reference_delete_path(expr, &path))
    }

    fn reference_delete_path(expr: Expr, path: &[isize]) -> Expr {
        let Some((first, rest)) = path.split_first() else {
            return expr;
        };
        let Expr::Call { head, args, .. } = expr else {
            return expr;
        };
        if *first == 0 {
            if rest.is_empty() {
                return expression_without_head(args.into_vec());
            }
            return Expr::call_with_shared_args(
                reference_delete_path(head.as_ref().clone(), rest),
                args,
            );
        }

        let index = *first as usize - 1;
        let mut args = Expr::unwrap_shared_args(args);
        if rest.is_empty() {
            args.remove(index);
        } else {
            args[index] = reference_delete_path(args[index].clone(), rest);
        }
        Expr::call_shared(head, SharedArgs::new(args))
    }
}
