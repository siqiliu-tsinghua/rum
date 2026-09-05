use std::collections::HashMap;

use super::*;

const LOOKUP_INDEX_THRESHOLD: usize = 16;
const RECURSIVE_LOOKUP_QUERY_COUNT_DEPTH: usize = 16;
const RECURSIVE_LOOKUP_PLANNING_DEPTH: usize = 16;
const RECURSIVE_LOOKUP_INTO_EXPR_DEPTH: usize = 16;
const RECURSIVE_LOOKUP_TO_EXPR_DEPTH: usize = 16;

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

impl Drop for LookupOutcome {
    fn drop(&mut self) {
        let Self::List(items) = self else {
            return;
        };
        if items.is_empty() {
            return;
        }

        drop_lookup_outcomes_iterative(std::mem::take(items));
    }
}

#[cold]
#[inline(never)]
fn drop_lookup_outcomes_iterative(mut work: Vec<LookupOutcome>) {
    while let Some(mut outcome) = work.pop() {
        if let LookupOutcome::List(items) = &mut outcome {
            work.append(items);
        }
    }
}

impl LookupOutcome {
    fn into_expr(self, default: Option<&Expr>, wrapper: Option<&Expr>) -> Expr {
        self.into_expr_with_front_depth(default, wrapper, RECURSIVE_LOOKUP_INTO_EXPR_DEPTH)
            .into_value()
    }

    fn into_expr_with_front_depth(
        self,
        default: Option<&Expr>,
        wrapper: Option<&Expr>,
        remaining_call_depth: usize,
    ) -> BoundedTraversal<Expr> {
        let mut exhausted = false;
        let rendered = lookup_outcome_into_expr_recursive_bounded(
            self,
            default,
            wrapper,
            remaining_call_depth,
            &mut exhausted,
        );
        if exhausted {
            BoundedTraversal::Exhausted(rendered)
        } else {
            BoundedTraversal::Complete(rendered)
        }
    }

    pub(super) fn to_expr(&self, default: Option<&Expr>, wrapper: Option<&Expr>) -> Expr {
        self.to_expr_with_front_depth(default, wrapper, RECURSIVE_LOOKUP_TO_EXPR_DEPTH)
            .into_value()
    }

    fn to_expr_with_front_depth(
        &self,
        default: Option<&Expr>,
        wrapper: Option<&Expr>,
        remaining_call_depth: usize,
    ) -> BoundedTraversal<Expr> {
        let mut exhausted = false;
        let rendered = lookup_outcome_to_expr_recursive_bounded(
            self,
            default,
            wrapper,
            remaining_call_depth,
            &mut exhausted,
        );
        if exhausted {
            BoundedTraversal::Exhausted(rendered)
        } else {
            BoundedTraversal::Complete(rendered)
        }
    }
}

fn lookup_outcome_into_expr_recursive_bounded(
    mut outcome: LookupOutcome,
    default: Option<&Expr>,
    wrapper: Option<&Expr>,
    remaining_call_depth: usize,
    exhausted: &mut bool,
) -> Expr {
    if remaining_call_depth == 0 && matches!(&outcome, LookupOutcome::List(_)) {
        *exhausted = true;
        return lookup_outcome_into_expr_iterative(outcome, default, wrapper);
    }

    match &mut outcome {
        LookupOutcome::Hit(value) => {
            let value = std::mem::replace(value, Expr::small_integer(0));
            wrap_keyed_result(value, wrapper)
        }
        LookupOutcome::Missing(key) => {
            let key = std::mem::replace(key, Expr::small_integer(0));
            default
                .cloned()
                .unwrap_or_else(|| missing_key_absent_expr(key))
        }
        LookupOutcome::List(items) => {
            let child_call_depth = remaining_call_depth - 1;
            let rendered = std::mem::take(items)
                .into_iter()
                .map(|item| {
                    lookup_outcome_into_expr_recursive_bounded(
                        item,
                        default,
                        wrapper,
                        child_call_depth,
                        exhausted,
                    )
                })
                .collect();
            Expr::call(system_expr("List"), rendered)
        }
    }
}

enum LookupOutcomeIntoExprWork {
    Visit(LookupOutcome),
    FinishList { item_count: usize },
}

#[cold]
#[inline(never)]
fn lookup_outcome_into_expr_iterative(
    outcome: LookupOutcome,
    default: Option<&Expr>,
    wrapper: Option<&Expr>,
) -> Expr {
    let mut work = vec![LookupOutcomeIntoExprWork::Visit(outcome)];
    let mut completed = Vec::new();

    while let Some(task) = work.pop() {
        match task {
            LookupOutcomeIntoExprWork::Visit(mut outcome) => match &mut outcome {
                LookupOutcome::Hit(value) => {
                    let value = std::mem::replace(value, Expr::small_integer(0));
                    completed.push(wrap_keyed_result(value, wrapper));
                }
                LookupOutcome::Missing(key) => {
                    let key = std::mem::replace(key, Expr::small_integer(0));
                    completed.push(
                        default
                            .cloned()
                            .unwrap_or_else(|| missing_key_absent_expr(key)),
                    );
                }
                LookupOutcome::List(items) => {
                    let items = std::mem::take(items);
                    work.push(LookupOutcomeIntoExprWork::FinishList {
                        item_count: items.len(),
                    });
                    work.extend(
                        items
                            .into_iter()
                            .rev()
                            .map(LookupOutcomeIntoExprWork::Visit),
                    );
                }
            },
            LookupOutcomeIntoExprWork::FinishList { item_count } => {
                let first_item = completed
                    .len()
                    .checked_sub(item_count)
                    .expect("owned Lookup render results remain balanced");
                let items = completed.split_off(first_item);
                completed.push(Expr::call(system_expr("List"), items));
            }
        }
    }

    match completed.pop() {
        Some(result) if completed.is_empty() => result,
        _ => panic!("owned Lookup rendering completes with one result"),
    }
}

fn lookup_outcome_to_expr_recursive_bounded(
    outcome: &LookupOutcome,
    default: Option<&Expr>,
    wrapper: Option<&Expr>,
    remaining_call_depth: usize,
    exhausted: &mut bool,
) -> Expr {
    if remaining_call_depth == 0 && matches!(outcome, LookupOutcome::List(_)) {
        *exhausted = true;
        return lookup_outcome_to_expr_iterative(outcome, default, wrapper);
    }

    match outcome {
        LookupOutcome::Hit(value) => wrap_keyed_result(value.clone(), wrapper),
        LookupOutcome::Missing(key) => default
            .cloned()
            .unwrap_or_else(|| missing_key_absent_expr(key.clone())),
        LookupOutcome::List(items) => {
            let child_call_depth = remaining_call_depth - 1;
            let mut rendered = Vec::with_capacity(items.len());
            for item in items {
                rendered.push(lookup_outcome_to_expr_recursive_bounded(
                    item,
                    default,
                    wrapper,
                    child_call_depth,
                    exhausted,
                ));
            }
            Expr::call(system_expr("List"), rendered)
        }
    }
}

enum LookupOutcomeToExprWork<'a> {
    Visit(&'a LookupOutcome),
    FinishList { item_count: usize },
}

#[cold]
#[inline(never)]
fn lookup_outcome_to_expr_iterative(
    outcome: &LookupOutcome,
    default: Option<&Expr>,
    wrapper: Option<&Expr>,
) -> Expr {
    let mut work = vec![LookupOutcomeToExprWork::Visit(outcome)];
    let mut completed = Vec::new();

    while let Some(task) = work.pop() {
        match task {
            LookupOutcomeToExprWork::Visit(outcome) => match outcome {
                LookupOutcome::Hit(value) => {
                    completed.push(wrap_keyed_result(value.clone(), wrapper));
                }
                LookupOutcome::Missing(key) => completed.push(
                    default
                        .cloned()
                        .unwrap_or_else(|| missing_key_absent_expr(key.clone())),
                ),
                LookupOutcome::List(items) => {
                    work.push(LookupOutcomeToExprWork::FinishList {
                        item_count: items.len(),
                    });
                    work.extend(items.iter().rev().map(LookupOutcomeToExprWork::Visit));
                }
            },
            LookupOutcomeToExprWork::FinishList { item_count } => {
                let first_item = completed
                    .len()
                    .checked_sub(item_count)
                    .expect("borrowed Lookup render results remain balanced");
                let items = completed.split_off(first_item);
                completed.push(Expr::call(system_expr("List"), items));
            }
        }
    }

    match completed.pop() {
        Some(result) if completed.is_empty() => result,
        _ => panic!("borrowed Lookup rendering completes with one result"),
    }
}

struct PreparedLookupEntries {
    entries: Vec<(Expr, Expr)>,
    positions: Option<HashMap<Expr, usize>>,
}

impl PreparedLookupEntries {
    fn new(entries: Vec<(Expr, Expr)>, query_count: usize) -> Self {
        let positions = if entries.len() > LOOKUP_INDEX_THRESHOLD && query_count > 1 {
            let mut positions = HashMap::with_capacity(entries.len());
            for (index, (key, _)) in entries.iter().enumerate() {
                positions.entry(key.clone()).or_insert(index);
            }
            Some(positions)
        } else {
            None
        };
        Self { entries, positions }
    }

    fn value(&self, key: &Expr) -> Option<&Expr> {
        if let Some(positions) = &self.positions {
            return positions.get(key).map(|index| &self.entries[*index].1);
        }

        self.entries
            .iter()
            .find(|(entry_key, _)| entry_key == key)
            .map(|(_, value)| value)
    }
}

impl LookupPlan {
    fn combine(self, other: Self) -> Self {
        Self {
            has_hit: self.has_hit || other.has_hit,
            has_missing: self.has_missing || other.has_missing,
        }
    }
}

pub(crate) fn eval_lookup(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    if args_slice.is_empty() || args_slice.len() > 4 {
        return Ok(session.invalid_call_with_message(
            "Lookup",
            args_slice,
            "argb",
            lookup_arg_count_message(args_slice.len()),
        ));
    }

    if let [key] = args_slice {
        return Ok(keyed_operator_form_slot_first("Lookup", key.clone()));
    }

    let [expr, key, ..] = args_slice else {
        return Ok(Expr::call_with_shared_args(head.clone(), args.clone()));
    };

    let expr_result = session.evaluate(expr.clone());
    let expr = rum_runtime::resume_or_suspend!(
        session,
        expr_result,
        ok(value) => Ok(value),
        suspend => {
    let head = head.clone();
            let key = key.clone();
            let call_args = args.clone();
    EvalLookupSuspensionFrame { head, key, call_args }
    },
    )?;

    eval_lookup_with_evaluated_data(session, head.clone(), expr, key.clone(), args.clone())
}

pub(super) fn eval_lookup_with_evaluated_data(
    session: &mut Session,
    head: Expr,
    expr: Expr,
    key: Expr,
    call_args: SharedArgs,
) -> Result<Expr, EvalError> {
    let key_result = session.evaluate(key);
    let key = rum_runtime::resume_or_suspend!(
        session,
        key_result,
        ok(value) => Ok(value),
        suspend => {
    let head = head.clone();
            let expr = expr.clone();
            let call_args = call_args.clone();
    EvalLookupWithEvaluatedDataSuspensionFrame { head, expr, call_args }
    },
    )?;

    finish_eval_lookup(session, head, expr, key, call_args)
}

pub(super) fn finish_eval_lookup(
    session: &mut Session,
    head: Expr,
    expr: Expr,
    key: Expr,
    call_args: SharedArgs,
) -> Result<Expr, EvalError> {
    let default = call_args.get(2).cloned();
    let wrapper = call_args.get(3).cloned();
    let Some(lookup) = prepare_lookup(&expr, &key) else {
        session.push_message(
            "Lookup",
            "invrl",
            invalid_association_or_rule_list_message(&expr),
        );
        let mut args = vec![expr, key];
        args.extend(call_args.as_ref().iter().skip(2).cloned());
        return Ok(Expr::call(head, args));
    };

    let default = if lookup.plan.has_missing {
        if let Some(default) = default {
            match session.evaluate(default) {
                Ok(default) => Some(default),
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let lookup = FrozenAssociationValue::freeze(lookup);
                    FinishEvalLookupSuspensionFrame { lookup, wrapper }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        }
    } else {
        None
    };

    finish_prepared_lookup(session, lookup, default, wrapper)
}

fn finish_prepared_lookup(
    session: &mut Session,
    lookup: PreparedLookup,
    default: Option<Expr>,
    wrapper: Option<Expr>,
) -> Result<Expr, EvalError> {
    let wrapper = if lookup.plan.has_hit {
        if let Some(wrapper) = wrapper {
            match session.evaluate(wrapper) {
                Ok(wrapper) => Some(wrapper),
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let lookup = FrozenAssociationValue::freeze(lookup);
                    FinishPreparedLookupSuspensionFrame { lookup, default }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(lookup.outcome.into_expr(default.as_ref(), wrapper.as_ref()))
}

pub(super) fn finish_suspended_prepared_lookup(
    session: &mut Session,
    lookup: FrozenAssociationValue<PreparedLookup>,
    default: Option<Expr>,
    wrapper: Option<Expr>,
) -> Result<Expr, EvalError> {
    let wrapper = if lookup.as_ref().plan.has_hit {
        if let Some(wrapper) = wrapper {
            match session.evaluate(wrapper) {
                Ok(wrapper) => Some(wrapper),
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => FinishSuspendedPreparedLookupSuspensionFrame { lookup, default },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(lookup
        .as_ref()
        .outcome
        .to_expr(default.as_ref(), wrapper.as_ref()))
}

fn lookup_arg_count_message(count: usize) -> String {
    format!("Lookup called with {count} arguments; between 2 and 4 arguments are expected.")
}

fn prepare_lookup(data: &Expr, key_spec: &Expr) -> Option<PreparedLookup> {
    let query_count = lookup_query_count(key_spec);
    if let Some(entries) = single_rule_or_keyed_entries_from_expr(data) {
        return Some(lookup_entries_outcome(
            &PreparedLookupEntries::new(entries, query_count),
            key_spec,
        ));
    }

    let items = call_items_with_head(data, &system_expr("List"))?;
    let mut plan = LookupPlan::default();
    let mut outcomes = Vec::with_capacity(items.len());
    for item in items {
        let entries = keyed_entries_from_expr(item)?;
        let lookup = lookup_entries_outcome(
            &PreparedLookupEntries::new(entries.entries, query_count),
            key_spec,
        );
        plan = plan.combine(lookup.plan);
        outcomes.push(lookup.outcome);
    }
    Some(PreparedLookup {
        outcome: LookupOutcome::List(outcomes),
        plan,
    })
}

fn lookup_query_count(key_spec: &Expr) -> usize {
    lookup_query_count_with_front_depth(key_spec, RECURSIVE_LOOKUP_QUERY_COUNT_DEPTH).into_value()
}

fn lookup_query_count_with_front_depth(
    key_spec: &Expr,
    remaining_call_depth: usize,
) -> BoundedTraversal<usize> {
    match lookup_query_count_recursive_bounded(key_spec, remaining_call_depth) {
        Some(count) => BoundedTraversal::Complete(count),
        None => BoundedTraversal::Exhausted(lookup_query_count_iterative(key_spec)),
    }
}

fn lookup_query_count_recursive_bounded(
    key_spec: &Expr,
    remaining_call_depth: usize,
) -> Option<usize> {
    let Some(keys) = call_items_with_head(key_spec, &system_expr("List")) else {
        return Some(1);
    };
    let child_call_depth = remaining_call_depth.checked_sub(1)?;

    let mut count = 0usize;
    for key in keys {
        count = count.saturating_add(lookup_query_count_recursive_bounded(key, child_call_depth)?);
    }
    Some(count)
}

#[cold]
#[inline(never)]
fn lookup_query_count_iterative(key_spec: &Expr) -> usize {
    let mut work = vec![key_spec];
    let mut count = 0usize;
    while let Some(key) = work.pop() {
        if let Some(keys) = call_items_with_head(key, &system_expr("List")) {
            work.extend(keys.iter().rev());
        } else {
            count = count.saturating_add(1);
        }
    }
    count
}

fn lookup_entries_outcome(entries: &PreparedLookupEntries, key_spec: &Expr) -> PreparedLookup {
    lookup_entries_outcome_with_front_depth(entries, key_spec, RECURSIVE_LOOKUP_PLANNING_DEPTH)
        .into_value()
}

fn lookup_entries_outcome_with_front_depth(
    entries: &PreparedLookupEntries,
    key_spec: &Expr,
    remaining_call_depth: usize,
) -> BoundedTraversal<PreparedLookup> {
    match lookup_entries_outcome_recursive_bounded(entries, key_spec, remaining_call_depth) {
        Some(lookup) => BoundedTraversal::Complete(lookup),
        None => BoundedTraversal::Exhausted(lookup_entries_outcome_iterative(entries, key_spec)),
    }
}

fn lookup_entries_outcome_recursive_bounded(
    entries: &PreparedLookupEntries,
    key_spec: &Expr,
    remaining_call_depth: usize,
) -> Option<PreparedLookup> {
    let Some(keys) = call_items_with_head(key_spec, &system_expr("List")) else {
        return Some(lookup_terminal_outcome(entries, key_spec));
    };
    let child_call_depth = remaining_call_depth.checked_sub(1)?;

    let mut plan = LookupPlan::default();
    let mut outcomes = Vec::with_capacity(keys.len());
    for key in keys {
        let lookup = lookup_entries_outcome_recursive_bounded(entries, key, child_call_depth)?;
        plan = plan.combine(lookup.plan);
        outcomes.push(lookup.outcome);
    }
    Some(PreparedLookup {
        outcome: LookupOutcome::List(outcomes),
        plan,
    })
}

enum LookupPlanningWork<'a> {
    Visit(&'a Expr),
    FinishList { item_count: usize },
}

#[cold]
#[inline(never)]
fn lookup_entries_outcome_iterative(
    entries: &PreparedLookupEntries,
    key_spec: &Expr,
) -> PreparedLookup {
    let mut work = vec![LookupPlanningWork::Visit(key_spec)];
    let mut completed = Vec::new();
    let mut plan = LookupPlan::default();

    while let Some(task) = work.pop() {
        match task {
            LookupPlanningWork::Visit(key) => {
                if let Some(keys) = call_items_with_head(key, &system_expr("List")) {
                    work.push(LookupPlanningWork::FinishList {
                        item_count: keys.len(),
                    });
                    work.extend(keys.iter().rev().map(LookupPlanningWork::Visit));
                } else {
                    let lookup = lookup_terminal_outcome(entries, key);
                    plan = plan.combine(lookup.plan);
                    completed.push(lookup.outcome);
                }
            }
            LookupPlanningWork::FinishList { item_count } => {
                let first_item = completed
                    .len()
                    .checked_sub(item_count)
                    .expect("Lookup planning results remain balanced");
                let items = completed.split_off(first_item);
                completed.push(LookupOutcome::List(items));
            }
        }
    }

    match completed.pop() {
        Some(outcome) if completed.is_empty() => PreparedLookup { outcome, plan },
        _ => panic!("Lookup planning completes with one outcome"),
    }
}

#[inline]
fn lookup_terminal_outcome(entries: &PreparedLookupEntries, key_spec: &Expr) -> PreparedLookup {
    let lookup_key = lookup_key_expr(key_spec);
    match entries.value(lookup_key) {
        Some(value) => PreparedLookup {
            outcome: LookupOutcome::Hit(value.clone()),
            plan: LookupPlan {
                has_hit: true,
                has_missing: false,
            },
        },
        None => PreparedLookup {
            outcome: LookupOutcome::Missing(lookup_key.clone()),
            plan: LookupPlan {
                has_hit: false,
                has_missing: true,
            },
        },
    }
}

fn lookup_key_expr(key_spec: &Expr) -> &Expr {
    match key_spec {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            &args[0]
        }
        _ => key_spec,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(items: Vec<Expr>) -> Expr {
        Expr::call(system_expr("List"), items)
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

    fn key(prefix: &str, hit: bool) -> Expr {
        Expr::string(format!("{prefix}-{}", if hit { "hit" } else { "miss" }))
    }

    fn key_spec_fixture() -> Expr {
        branched_tree(key)
    }

    fn entries_fixture() -> PreparedLookupEntries {
        let entries = [
            "left-left-left",
            "left-right-right",
            "right-left-left",
            "right-right-right",
        ]
        .into_iter()
        .map(|prefix| (key(prefix, true), Expr::string(format!("{prefix}-value"))))
        .collect();
        PreparedLookupEntries::new(entries, 8)
    }

    fn expected_result(wrapper: &Expr, default: &Expr) -> Expr {
        branched_tree(|prefix, hit| {
            if hit {
                Expr::call(
                    wrapper.clone(),
                    vec![Expr::string(format!("{prefix}-value"))],
                )
            } else {
                default.clone()
            }
        })
    }

    fn outcome_fixture() -> LookupOutcome {
        lookup_entries_outcome(&entries_fixture(), &key_spec_fixture()).outcome
    }

    #[test]
    fn lookup_query_count_forced_front_tail_matches_complete_front_exactly() {
        let fixture = key_spec_fixture();

        let tail = lookup_query_count_with_front_depth(&fixture, 0);
        assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
        let tail = tail.into_value();

        let front = lookup_query_count_with_front_depth(&fixture, 16);
        assert!(matches!(&front, BoundedTraversal::Complete(_)));
        let front = front.into_value();

        assert_eq!(tail, front);
        assert_eq!(tail, 8);
    }

    #[test]
    fn lookup_planning_forced_front_tail_matches_complete_front_exactly() {
        let entries = entries_fixture();
        let fixture = key_spec_fixture();
        let wrapper = system_expr("lookupPlanningWitnessWrapper");
        let default = Expr::string("lookup-planning-default");

        let tail = lookup_entries_outcome_with_front_depth(&entries, &fixture, 0);
        assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
        let tail = tail.into_value();

        let front = lookup_entries_outcome_with_front_depth(&entries, &fixture, 16);
        assert!(matches!(&front, BoundedTraversal::Complete(_)));
        let front = front.into_value();

        assert_eq!(tail.plan, front.plan);
        assert_eq!(
            tail.plan,
            LookupPlan {
                has_hit: true,
                has_missing: true,
            }
        );
        let tail = tail.outcome.into_expr(Some(&default), Some(&wrapper));
        let front = front.outcome.into_expr(Some(&default), Some(&wrapper));
        let expected = expected_result(&wrapper, &default);
        assert_eq!(tail, front);
        assert_eq!(tail, expected);
    }

    fn render_with_front_depth(front_depth: usize) -> (Expr, Expr) {
        let wrapper = system_expr("lookupRenderWitnessWrapper");
        let default = Expr::string("lookup-render-default");
        let outcome = outcome_fixture();

        let borrowed =
            outcome.to_expr_with_front_depth(Some(&default), Some(&wrapper), front_depth);
        if front_depth == 0 {
            assert!(matches!(&borrowed, BoundedTraversal::Exhausted(_)));
        } else {
            assert!(matches!(&borrowed, BoundedTraversal::Complete(_)));
        }
        let borrowed = borrowed.into_value();

        let owned = outcome.into_expr_with_front_depth(Some(&default), Some(&wrapper), front_depth);
        if front_depth == 0 {
            assert!(matches!(&owned, BoundedTraversal::Exhausted(_)));
        } else {
            assert!(matches!(&owned, BoundedTraversal::Complete(_)));
        }
        let owned = owned.into_value();

        assert_eq!(owned, borrowed);
        assert_eq!(owned, expected_result(&wrapper, &default));
        (owned, borrowed)
    }

    #[test]
    fn lookup_rendering_forced_fronts_agree_owned_borrowed_and_exactly() {
        let (owned_tail, borrowed_tail) = render_with_front_depth(0);
        let (owned_front, borrowed_front) = render_with_front_depth(16);

        assert_eq!(owned_tail, owned_front);
        assert_eq!(borrowed_tail, borrowed_front);
        assert_eq!(owned_tail, borrowed_front);
    }

    #[test]
    #[ignore = "release-only deployment-stack drop certificate; run explicitly with --release"]
    fn lookup_outcome_drop_is_bounded_at_physical_height_4097_on_two_mib() {
        let joined = std::thread::Builder::new()
            .name("lookup-outcome-drop-depth-4097".to_string())
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                let mut outcome = LookupOutcome::Missing(Expr::string("deep-drop-witness"));
                for _ in 1..4_097 {
                    outcome = LookupOutcome::List(vec![outcome]);
                }

                drop(outcome);
            })
            .expect("2 MiB LookupOutcome drop certificate thread should spawn")
            .join();
        assert!(joined.is_ok(), "LookupOutcome drop certificate panicked");
    }
}
