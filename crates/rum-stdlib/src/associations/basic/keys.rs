use super::*;

const RECURSIVE_KEYS_CALL_DEPTH: usize = 16;

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

pub(crate) fn eval_keys(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=2).contains(&args.len()) {
        return Ok(session.invalid_call_with_message(
            "Keys",
            args,
            "argt",
            one_or_two_argument_count_message("Keys", args.len()),
        ));
    }

    let wrapper = args.get(1);
    match keys_result_for_expr(&args[0], wrapper) {
        Ok(value) => Ok(value),
        Err(invalid) => Ok(session.invalid_call_with_message(
            "Keys",
            args,
            "invrl",
            invalid_association_or_rule_list_message(&invalid),
        )),
    }
}

fn keys_result_for_expr(expr: &Expr, wrapper: Option<&Expr>) -> Result<Expr, Expr> {
    keys_result_for_expr_with_front_depth(expr, wrapper, RECURSIVE_KEYS_CALL_DEPTH).into_value()
}

fn keys_result_for_expr_with_front_depth(
    expr: &Expr,
    wrapper: Option<&Expr>,
    remaining_call_depth: usize,
) -> BoundedTraversal<Result<Expr, Expr>> {
    match keys_result_for_expr_recursive_bounded(expr, wrapper, remaining_call_depth) {
        Some(result) => BoundedTraversal::Complete(result),
        None => BoundedTraversal::Exhausted(keys_result_for_expr_iterative(expr, wrapper)),
    }
}

fn keys_result_for_expr_recursive_bounded(
    expr: &Expr,
    wrapper: Option<&Expr>,
    remaining_call_depth: usize,
) -> Option<Result<Expr, Expr>> {
    if let Some(result) = keys_terminal_result(expr, wrapper) {
        return Some(Ok(result));
    }

    let Some(items) = list_items(expr) else {
        return Some(Err(expr.clone()));
    };
    let child_call_depth = remaining_call_depth.checked_sub(1)?;

    let mut keys = Vec::with_capacity(items.len());
    for item in items {
        match keys_result_for_expr_recursive_bounded(item, wrapper, child_call_depth)? {
            Ok(value) => keys.push(value),
            Err(invalid) => return Some(Err(invalid)),
        }
    }

    Some(Ok(Expr::call(system_expr("List"), keys)))
}

fn keys_terminal_result(expr: &Expr, wrapper: Option<&Expr>) -> Option<Expr> {
    if let Some((key, _)) = parse_association_entry(expr) {
        return Some(wrap_keyed_result(key, wrapper));
    }

    let keyed = keyed_entries_from_expr(expr)?;
    Some(Expr::call(
        system_expr("List"),
        keyed
            .entries
            .into_iter()
            .map(|(key, _)| wrap_keyed_result(key, wrapper))
            .collect(),
    ))
}

enum KeysWork<'a> {
    Visit(&'a Expr),
    FinishList { item_count: usize },
}

#[cold]
#[inline(never)]
fn keys_result_for_expr_iterative(expr: &Expr, wrapper: Option<&Expr>) -> Result<Expr, Expr> {
    let mut work = vec![KeysWork::Visit(expr)];
    let mut completed = Vec::new();

    while let Some(task) = work.pop() {
        match task {
            KeysWork::Visit(expr) => {
                if let Some(result) = keys_terminal_result(expr, wrapper) {
                    completed.push(result);
                    continue;
                }

                let Some(items) = list_items(expr) else {
                    return Err(expr.clone());
                };

                work.push(KeysWork::FinishList {
                    item_count: items.len(),
                });
                work.extend(items.iter().rev().map(KeysWork::Visit));
            }
            KeysWork::FinishList { item_count } => {
                let first_item = completed
                    .len()
                    .checked_sub(item_count)
                    .expect("Keys list results remain balanced");
                let items = completed.split_off(first_item);
                completed.push(Expr::call(system_expr("List"), items));
            }
        }
    }

    match completed.pop() {
        Some(result) if completed.is_empty() => Ok(result),
        _ => panic!("Keys traversal completes with one result"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(items: Vec<Expr>) -> Expr {
        Expr::call(system_expr("List"), items)
    }

    fn rule(key: impl Into<String>, value: impl Into<String>) -> Expr {
        Expr::call(
            system_expr("Rule"),
            vec![Expr::string(key.into()), Expr::string(value.into())],
        )
    }

    fn association(entries: Vec<Expr>) -> Expr {
        Expr::call(system_expr("Association"), entries)
    }

    fn terminal(prefix: &str, association_form: bool) -> Expr {
        let entries = vec![
            rule(format!("{prefix}-first"), format!("{prefix}-value-1")),
            rule(format!("{prefix}-second"), format!("{prefix}-value-2")),
        ];
        if association_form {
            association(entries)
        } else {
            list(entries)
        }
    }

    fn expected_terminal(prefix: &str, wrapper: &Expr) -> Expr {
        list(vec![
            Expr::call(
                wrapper.clone(),
                vec![Expr::string(format!("{prefix}-first"))],
            ),
            Expr::call(
                wrapper.clone(),
                vec![Expr::string(format!("{prefix}-second"))],
            ),
        ])
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

    fn invalid_fixture() -> Expr {
        list(vec![
            list(vec![
                terminal("valid-before", true),
                list(vec![
                    Expr::string("first-invalid"),
                    terminal("unvisited-after-first", false),
                ]),
            ]),
            list(vec![
                Expr::string("second-invalid"),
                terminal("unvisited-after-second", true),
            ]),
        ])
    }

    #[test]
    fn keys_forced_front_tail_matches_complete_front_exactly() {
        let wrapper = system_expr("keysWitnessWrapper");
        let fixture = branched_tree(terminal);

        let tail = keys_result_for_expr_with_front_depth(&fixture, Some(&wrapper), 0);
        assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
        let tail = tail.into_value();

        let front = keys_result_for_expr_with_front_depth(&fixture, Some(&wrapper), 16);
        assert!(matches!(&front, BoundedTraversal::Complete(_)));
        let front = front.into_value();

        let expected = Ok(branched_tree(|prefix, _| {
            expected_terminal(prefix, &wrapper)
        }));
        assert_eq!(tail, front);
        assert_eq!(tail, expected);
    }

    #[test]
    fn keys_forced_tail_preserves_first_error_in_source_order() {
        let fixture = invalid_fixture();

        let tail = keys_result_for_expr_with_front_depth(&fixture, None, 0);
        assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
        let tail = tail.into_value();

        let front = keys_result_for_expr_with_front_depth(&fixture, None, 16);
        assert!(matches!(&front, BoundedTraversal::Complete(_)));
        let front = front.into_value();

        let expected = Err(Expr::string("first-invalid"));
        assert_eq!(tail, front);
        assert_eq!(tail, expected);
    }
}
