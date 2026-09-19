use super::*;

const RECURSIVE_VALUES_CALL_DEPTH: usize = 16;

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

pub(crate) fn eval_values(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=2).contains(&args.len()) {
        return Ok(session.invalid_call_with_message(
            "Values",
            args,
            "argt",
            one_or_two_argument_count_message("Values", args.len()),
        ));
    }

    let wrapper = args.get(1);
    match values_result_for_expr(&args[0], wrapper) {
        Ok(value) => Ok(value),
        Err(invalid) => Ok(session.invalid_call_with_message(
            "Values",
            args,
            "invrl",
            invalid_association_or_rule_list_message(&invalid),
        )),
    }
}

fn values_result_for_expr(expr: &Expr, wrapper: Option<&Expr>) -> Result<Expr, Expr> {
    values_result_for_expr_with_front_depth(expr, wrapper, RECURSIVE_VALUES_CALL_DEPTH).into_value()
}

fn values_result_for_expr_with_front_depth(
    expr: &Expr,
    wrapper: Option<&Expr>,
    remaining_call_depth: usize,
) -> BoundedTraversal<Result<Expr, Expr>> {
    match values_result_for_expr_recursive_bounded(expr, wrapper, remaining_call_depth) {
        Some(result) => BoundedTraversal::Complete(result),
        None => BoundedTraversal::Exhausted(values_result_for_expr_iterative(expr, wrapper)),
    }
}

fn values_result_for_expr_recursive_bounded(
    expr: &Expr,
    wrapper: Option<&Expr>,
    remaining_call_depth: usize,
) -> Option<Result<Expr, Expr>> {
    if let Some(result) = values_terminal_result(expr, wrapper) {
        return Some(Ok(result));
    }

    let Some(items) = list_items(expr) else {
        return Some(Err(expr.clone()));
    };
    let child_call_depth = remaining_call_depth.checked_sub(1)?;

    let mut values = Vec::with_capacity(items.len());
    for item in items {
        match values_result_for_expr_recursive_bounded(item, wrapper, child_call_depth)? {
            Ok(value) => values.push(value),
            Err(_) => return Some(Err(expr.clone())),
        }
    }

    Some(Ok(Expr::call(system_expr("List"), values)))
}

fn values_terminal_result(expr: &Expr, wrapper: Option<&Expr>) -> Option<Expr> {
    if let Some((_, value)) = parse_association_entry(expr) {
        return Some(wrap_keyed_result(value, wrapper));
    }

    let keyed = keyed_entries_from_expr(expr)?;
    Some(Expr::call(
        system_expr("List"),
        keyed
            .entries
            .into_iter()
            .map(|(_, value)| wrap_keyed_result(value, wrapper))
            .collect(),
    ))
}

enum ValuesWork<'a> {
    Visit(&'a Expr),
    FinishList { item_count: usize },
}

#[cold]
#[inline(never)]
fn values_result_for_expr_iterative(root: &Expr, wrapper: Option<&Expr>) -> Result<Expr, Expr> {
    let mut work = vec![ValuesWork::Visit(root)];
    let mut completed = Vec::new();

    while let Some(task) = work.pop() {
        match task {
            ValuesWork::Visit(expr) => {
                if let Some(result) = values_terminal_result(expr, wrapper) {
                    completed.push(result);
                    continue;
                }

                // The recursive front replaces a child's failure payload with
                // each enclosing level, so any failure surfaces the whole
                // top-level argument — the payload the diagnostic reports.
                let Some(items) = list_items(expr) else {
                    return Err(root.clone());
                };

                work.push(ValuesWork::FinishList {
                    item_count: items.len(),
                });
                work.extend(items.iter().rev().map(ValuesWork::Visit));
            }
            ValuesWork::FinishList { item_count } => {
                let first_item = completed
                    .len()
                    .checked_sub(item_count)
                    .expect("Values list results remain balanced");
                let items = completed.split_off(first_item);
                completed.push(Expr::call(system_expr("List"), items));
            }
        }
    }

    match completed.pop() {
        Some(result) if completed.is_empty() => Ok(result),
        _ => panic!("Values traversal completes with one result"),
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
            rule(format!("{prefix}-key-first"), format!("{prefix}-value-1")),
            rule(format!("{prefix}-key-second"), format!("{prefix}-value-2")),
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
                vec![Expr::string(format!("{prefix}-value-1"))],
            ),
            Expr::call(
                wrapper.clone(),
                vec![Expr::string(format!("{prefix}-value-2"))],
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
    fn values_forced_front_tail_matches_complete_front_exactly() {
        let wrapper = system_expr("valuesWitnessWrapper");
        let fixture = branched_tree(terminal);

        let tail = values_result_for_expr_with_front_depth(&fixture, Some(&wrapper), 0);
        assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
        let tail = tail.into_value();

        let front = values_result_for_expr_with_front_depth(&fixture, Some(&wrapper), 16);
        assert!(matches!(&front, BoundedTraversal::Complete(_)));
        let front = front.into_value();

        let expected = Ok(branched_tree(|prefix, _| {
            expected_terminal(prefix, &wrapper)
        }));
        assert_eq!(tail, front);
        assert_eq!(tail, expected);
    }

    #[test]
    fn values_forced_tail_preserves_outer_expression_error_payload() {
        let fixture = invalid_fixture();

        let tail = values_result_for_expr_with_front_depth(&fixture, None, 0);
        assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
        let tail = tail.into_value();

        let front = values_result_for_expr_with_front_depth(&fixture, None, 16);
        assert!(matches!(&front, BoundedTraversal::Complete(_)));
        let front = front.into_value();

        let expected = Err(fixture.clone());
        assert_eq!(tail, front);
        assert_eq!(tail, expected);
    }
}
