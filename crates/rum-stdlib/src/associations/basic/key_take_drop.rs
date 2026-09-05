use std::collections::HashMap;

use super::*;

const KEY_SELECT_INDEX_THRESHOLD: usize = 16;
const RECURSIVE_KEY_TAKE_DROP_CALL_DEPTH: usize = 16;

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

#[derive(Clone, Debug, PartialEq, Eq)]
struct KeyTakeDropInvalid {
    tag: &'static str,
    invalid: Expr,
}

pub(crate) fn eval_key_take(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [key_spec] => Ok(keyed_operator_form_slot_first("KeyTake", key_spec.clone())),
        [expr, key_spec] => match keyed_take_drop_result_for_expr(expr, key_spec, true) {
            Ok(value) => Ok(value),
            Err(error) => Ok(session.invalid_call_with_message(
                "KeyTake",
                args,
                error.tag,
                invalid_association_or_rule_list_message(&error.invalid),
            )),
        },
        _ => Ok(session.invalid_call_with_message(
            "KeyTake",
            args,
            "argt",
            one_or_two_argument_count_message("KeyTake", args.len()),
        )),
    }
}

pub(crate) fn eval_key_drop(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [key_spec] => Ok(keyed_operator_form_slot_first("KeyDrop", key_spec.clone())),
        [expr, key_spec] => match keyed_take_drop_result_for_expr(expr, key_spec, false) {
            Ok(value) => Ok(value),
            Err(error) => Ok(session.invalid_call_with_message(
                "KeyDrop",
                args,
                error.tag,
                invalid_association_or_rule_list_message(&error.invalid),
            )),
        },
        _ => Ok(session.invalid_call_with_message(
            "KeyDrop",
            args,
            "argt",
            one_or_two_argument_count_message("KeyDrop", args.len()),
        )),
    }
}

fn keyed_take_drop_result_for_expr(
    expr: &Expr,
    key_spec: &Expr,
    keep_selected: bool,
) -> Result<Expr, KeyTakeDropInvalid> {
    let selected_keys = parse_key_spec(key_spec).ok_or_else(|| KeyTakeDropInvalid {
        tag: "invrl",
        invalid: expr.clone(),
    })?;

    keyed_take_drop_result_for_expr_with_front_depth(
        expr,
        &selected_keys,
        keep_selected,
        RECURSIVE_KEY_TAKE_DROP_CALL_DEPTH,
    )
    .into_value()
}

fn keyed_take_drop_result_for_expr_with_front_depth(
    expr: &Expr,
    selected_keys: &[Expr],
    keep_selected: bool,
    remaining_call_depth: usize,
) -> BoundedTraversal<Result<Expr, KeyTakeDropInvalid>> {
    match keyed_take_drop_result_for_expr_recursive_bounded(
        expr,
        selected_keys,
        keep_selected,
        remaining_call_depth,
    ) {
        Some(result) => BoundedTraversal::Complete(result),
        None => BoundedTraversal::Exhausted(keyed_take_drop_result_for_expr_iterative(
            expr,
            selected_keys,
            keep_selected,
        )),
    }
}

fn keyed_take_drop_result_for_expr_recursive_bounded(
    expr: &Expr,
    selected_keys: &[Expr],
    keep_selected: bool,
    remaining_call_depth: usize,
) -> Option<Result<Expr, KeyTakeDropInvalid>> {
    if list_items(expr).is_some_and(|items| items.is_empty()) {
        return Some(Ok(Expr::call(system_expr(heads::structure::LIST), vec![])));
    }

    if let Some(result) = keyed_take_drop_terminal_result(expr, selected_keys, keep_selected) {
        return Some(Ok(result));
    }

    let Some(items) = list_items(expr) else {
        return Some(Err(KeyTakeDropInvalid {
            tag: "invrl",
            invalid: expr.clone(),
        }));
    };
    let child_call_depth = remaining_call_depth.checked_sub(1)?;

    let mut results = Vec::with_capacity(items.len());
    for item in items {
        match keyed_take_drop_result_for_expr_recursive_bounded(
            item,
            selected_keys,
            keep_selected,
            child_call_depth,
        )? {
            Ok(value) => results.push(value),
            Err(error) => {
                return Some(Err(KeyTakeDropInvalid {
                    tag: "invru",
                    invalid: if keep_selected {
                        expr.clone()
                    } else {
                        error.invalid
                    },
                }));
            }
        }
    }

    Some(Ok(Expr::call(system_expr(heads::structure::LIST), results)))
}

fn keyed_take_drop_terminal_result(
    expr: &Expr,
    selected_keys: &[Expr],
    keep_selected: bool,
) -> Option<Expr> {
    let entries = single_rule_or_keyed_entry_forms_from_expr(expr)?;
    let indexed = entries.len() > KEY_SELECT_INDEX_THRESHOLD
        && selected_keys.len() > KEY_SELECT_INDEX_THRESHOLD;
    let filtered = if keep_selected {
        if indexed {
            // The result keeps selection order, so index the entries by key
            // and walk the selected keys. Overwriting inserts keep the last
            // entry for a duplicated key — what rendering's canonicalization
            // would pick — and `remove` emits a repeated selection once while
            // missing keys fall through silently.
            let mut by_key = HashMap::with_capacity(entries.len());
            for entry in &entries {
                by_key.insert(entry.key.clone(), entry.clone());
            }
            let mut taken = Vec::new();
            for selected_key in selected_keys {
                if let Some(entry) = by_key.remove(selected_key) {
                    taken.push(entry);
                }
            }
            taken
        } else {
            let mut taken = Vec::new();
            for selected_key in selected_keys {
                taken.extend(
                    entries
                        .iter()
                        .filter(|entry| &entry.key == selected_key)
                        .cloned(),
                );
            }
            taken
        }
    } else if indexed {
        let selected = selected_keys
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        entries
            .into_iter()
            .filter(|entry| !selected.contains(&entry.key))
            .collect()
    } else {
        entries
            .into_iter()
            .filter(|entry| {
                !selected_keys
                    .iter()
                    .any(|candidate| candidate == &entry.key)
            })
            .collect()
    };

    Some(render_association_entry_forms_preserving_heads(filtered))
}

enum KeyTakeDropWork {
    Visit { expr: Expr, is_root: bool },
    FinishList { item_count: usize },
}

#[cold]
#[inline(never)]
fn keyed_take_drop_result_for_expr_iterative(
    expr: &Expr,
    selected_keys: &[Expr],
    keep_selected: bool,
) -> Result<Expr, KeyTakeDropInvalid> {
    let root = expr.clone();
    let mut work = vec![KeyTakeDropWork::Visit {
        expr: root.clone(),
        is_root: true,
    }];
    let mut completed = Vec::new();

    while let Some(task) = work.pop() {
        match task {
            KeyTakeDropWork::Visit { expr, is_root } => {
                if list_items(&expr).is_some_and(|items| items.is_empty()) {
                    completed.push(Expr::call(system_expr(heads::structure::LIST), vec![]));
                    continue;
                }

                if let Some(result) =
                    keyed_take_drop_terminal_result(&expr, selected_keys, keep_selected)
                {
                    completed.push(result);
                    continue;
                }

                let Some(items) = list_items(&expr) else {
                    return Err(KeyTakeDropInvalid {
                        tag: if is_root { "invrl" } else { "invru" },
                        invalid: if keep_selected && !is_root {
                            root.clone()
                        } else {
                            expr
                        },
                    });
                };

                let item_count = items.len();
                work.push(KeyTakeDropWork::FinishList { item_count });
                work.extend(
                    items
                        .iter()
                        .rev()
                        .cloned()
                        .map(|expr| KeyTakeDropWork::Visit {
                            expr,
                            is_root: false,
                        }),
                );
            }
            KeyTakeDropWork::FinishList { item_count } => {
                let first_item = completed
                    .len()
                    .checked_sub(item_count)
                    .expect("key take/drop list results remain balanced");
                let items = completed.split_off(first_item);
                completed.push(Expr::call(system_expr(heads::structure::LIST), items));
            }
        }
    }

    match completed.pop() {
        Some(result) if completed.is_empty() => Ok(result),
        _ => panic!("key take/drop traversal completes with one result"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(items: Vec<Expr>) -> Expr {
        Expr::call(system_expr(heads::structure::LIST), items)
    }

    fn rule(key: &str, value: String, delayed: bool) -> Expr {
        Expr::call(
            system_expr(if delayed { "RuleDelayed" } else { "Rule" }),
            vec![Expr::string(key), Expr::string(value)],
        )
    }

    fn association(entries: Vec<Expr>) -> Expr {
        Expr::call(system_expr("Association"), entries)
    }

    fn terminal(prefix: &str, association_form: bool) -> Expr {
        let entries = vec![
            rule("a", format!("{prefix}-a"), false),
            rule("stay-left", format!("{prefix}-stay-left"), true),
            rule("b", format!("{prefix}-b"), true),
            rule("stay-right", format!("{prefix}-stay-right"), false),
        ];
        if association_form {
            association(entries)
        } else {
            list(entries)
        }
    }

    fn expected_terminal(prefix: &str, keep_selected: bool) -> Expr {
        let entries = if keep_selected {
            vec![
                rule("b", format!("{prefix}-b"), true),
                rule("a", format!("{prefix}-a"), false),
            ]
        } else {
            vec![
                rule("stay-left", format!("{prefix}-stay-left"), true),
                rule("stay-right", format!("{prefix}-stay-right"), false),
            ]
        };
        association(entries)
    }

    fn branched_tree(mut leaf: impl FnMut(&str, bool) -> Expr) -> Expr {
        list(vec![
            list(vec![
                list(vec![
                    leaf("left-left-left", true),
                    leaf("left-left-right", false),
                ]),
                list(vec![
                    leaf("left-right-left", false),
                    leaf("left-right-right", true),
                ]),
            ]),
            list(vec![
                list(vec![
                    leaf("right-left-left", true),
                    leaf("right-left-right", false),
                ]),
                list(vec![
                    leaf("right-right-left", false),
                    leaf("right-right-right", true),
                ]),
            ]),
        ])
    }

    fn fixture() -> Expr {
        branched_tree(terminal)
    }

    fn exact_expected(keep_selected: bool) -> Expr {
        branched_tree(|prefix, _| expected_terminal(prefix, keep_selected))
    }

    fn selected_keys() -> Vec<Expr> {
        vec![
            Expr::string("b"),
            Expr::string("a"),
            Expr::string("missing"),
            Expr::string("a"),
        ]
    }

    fn invalid_fixture() -> Expr {
        list(vec![
            list(vec![
                terminal("valid-before-invalid", true),
                list(vec![
                    Expr::string("first-invalid-leaf"),
                    terminal("unvisited-after-invalid", false),
                ]),
            ]),
            list(vec![
                terminal("unvisited-right-left", false),
                terminal("unvisited-right-right", true),
            ]),
        ])
    }

    #[test]
    fn key_take_drop_forced_front_tail_matches_complete_front_for_both_flags() {
        for keep_selected in [true, false] {
            let tail = keyed_take_drop_result_for_expr_with_front_depth(
                &fixture(),
                &selected_keys(),
                keep_selected,
                0,
            );
            assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
            let tail = tail.into_value();

            let front = keyed_take_drop_result_for_expr_with_front_depth(
                &fixture(),
                &selected_keys(),
                keep_selected,
                16,
            );
            assert!(matches!(&front, BoundedTraversal::Complete(_)));
            let front = front.into_value();

            let expected = Ok(exact_expected(keep_selected));
            assert_eq!(tail, front);
            assert_eq!(tail, expected);
        }
    }

    #[test]
    fn key_take_drop_forced_tail_preserves_invalid_payload_distinction() {
        let fixture = invalid_fixture();
        for keep_selected in [true, false] {
            let tail = keyed_take_drop_result_for_expr_with_front_depth(
                &fixture,
                &selected_keys(),
                keep_selected,
                0,
            );
            assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
            let tail = tail.into_value();

            let front = keyed_take_drop_result_for_expr_with_front_depth(
                &fixture,
                &selected_keys(),
                keep_selected,
                16,
            );
            assert!(matches!(&front, BoundedTraversal::Complete(_)));
            let front = front.into_value();

            let expected = Err(KeyTakeDropInvalid {
                tag: "invru",
                invalid: if keep_selected {
                    fixture.clone()
                } else {
                    Expr::string("first-invalid-leaf")
                },
            });
            assert_eq!(tail, front);
            assert_eq!(tail, expected);
        }
    }
}
