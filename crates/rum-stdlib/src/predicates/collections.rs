mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

const STACK_MATCHING_THRESHOLD: usize = 16;

#[derive(Clone, Copy, FrameRootTrace)]
enum SameTestRelationKind {
    Intersecting,
    Disjoint,
    Subset,
    ContainsAll,
    ContainsAny,
    ContainsExactly,
    ContainsNone,
    ContainsOnly,
}

#[derive(FrameRootTrace)]
struct DuplicateFreeSameTestState {
    same_test: Expr,
    items: Vec<Expr>,
    lhs_index: usize,
    rhs_index: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedDuplicateFreeSameTestState {
    same_test: Expr,
    items: FrozenPredicateSequence<Expr>,
    lhs_index: usize,
    rhs_index: usize,
}

#[derive(FrameRootTrace)]
struct SameTestMatrixState {
    same_test: Expr,
    lhs: Vec<Expr>,
    rhs: Vec<Expr>,
    lhs_index: usize,
    rhs_index: usize,
    matches: Vec<Vec<bool>>,
    kind: SameTestRelationKind,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedSameTestMatrixState {
    same_test: Expr,
    lhs: FrozenPredicateSequence<Expr>,
    rhs: FrozenPredicateSequence<Expr>,
    lhs_index: usize,
    rhs_index: usize,
    matches: PersistentPredicateMatches,
    kind: SameTestRelationKind,
}

pub(crate) fn eval_duplicate_free_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (data_args, explicit_options) = split_same_test_options(args, 1);
    let [expr] = data_args else {
        return Ok(session.invalid_call_with_exact_arg_count("DuplicateFreeQ", args, 1));
    };
    let same_test = effective_same_test(session, "DuplicateFreeQ", &explicit_options)?;

    let Some(items) = list_items(expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(same_test) = same_test {
        return eval_duplicate_free_same_test(session, same_test, items);
    }

    let mut seen = std::collections::HashSet::new();
    for item in items {
        if !seen.insert(item.clone()) {
            return Ok(predicate_bool_expr(false));
        }
    }

    Ok(predicate_bool_expr(true))
}

pub(crate) fn eval_intersecting_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_list_containment_relation(
        session,
        head,
        "IntersectingQ",
        args,
        SameTestRelationKind::Intersecting,
        |lhs, rhs| {
            let rhs_set = expr_set(rhs);
            lhs.iter().any(|item| rhs_set.contains(item))
        },
    )
}

pub(crate) fn eval_disjoint_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_list_containment_relation(
        session,
        head,
        "DisjointQ",
        args,
        SameTestRelationKind::Disjoint,
        |lhs, rhs| {
            let rhs_set = expr_set(rhs);
            !lhs.iter().any(|item| rhs_set.contains(item))
        },
    )
}

pub(crate) fn eval_subset_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_list_containment_relation(
        session,
        head,
        "SubsetQ",
        args,
        SameTestRelationKind::Subset,
        |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .all(|item| lhs_set.contains(&item))
        },
    )
}

pub(crate) fn eval_contains_all(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_list_containment_relation(
        session,
        head,
        "ContainsAll",
        args,
        SameTestRelationKind::ContainsAll,
        |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .all(|item| lhs_set.contains(&item))
        },
    )
}

pub(crate) fn eval_contains_any(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_list_containment_relation(
        session,
        head,
        "ContainsAny",
        args,
        SameTestRelationKind::ContainsAny,
        |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .any(|item| lhs_set.contains(&item))
        },
    )
}

pub(crate) fn eval_contains_exactly(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_list_containment_relation(
        session,
        head,
        "ContainsExactly",
        args,
        SameTestRelationKind::ContainsExactly,
        |lhs, rhs| expr_counts(lhs) == expr_counts(rhs),
    )
}

pub(crate) fn eval_contains_none(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_list_containment_relation(
        session,
        head,
        "ContainsNone",
        args,
        SameTestRelationKind::ContainsNone,
        |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .all(|item| !lhs_set.contains(&item))
        },
    )
}

pub(crate) fn eval_contains_only(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_list_containment_relation(
        session,
        head,
        "ContainsOnly",
        args,
        SameTestRelationKind::ContainsOnly,
        |lhs, rhs| {
            let rhs_set = expr_set(rhs);
            lhs.iter().all(|item| rhs_set.contains(item))
        },
    )
}

fn eval_list_containment_relation<F>(
    session: &mut Session,
    head: &Expr,
    name: &str,
    args: &[Expr],
    same_test_kind: SameTestRelationKind,
    predicate: F,
) -> Result<Expr, EvalError>
where
    F: FnOnce(&[Expr], &[Expr]) -> bool,
{
    let (data_args, explicit_options) = split_same_test_options(args, 2);
    let same_test = effective_same_test(session, name, &explicit_options)?;
    let Some((lhs, rhs)) = two_list_args(data_args) else {
        return Ok(if data_args.len() == 2 {
            Expr::call(head.clone(), args.to_vec())
        } else if data_args.len() == 1 && is_contains_operator_kind(same_test_kind) {
            Expr::call(head.clone(), args.to_vec())
        } else {
            session.invalid_call_with_exact_arg_count(name, args, 2)
        });
    };

    if let Some(same_test) = same_test {
        return eval_same_test_relation(session, same_test, lhs, rhs, same_test_kind);
    }

    Ok(predicate_bool_expr(predicate(lhs, rhs)))
}

fn is_contains_operator_kind(kind: SameTestRelationKind) -> bool {
    matches!(
        kind,
        SameTestRelationKind::ContainsAll
            | SameTestRelationKind::ContainsAny
            | SameTestRelationKind::ContainsExactly
            | SameTestRelationKind::ContainsNone
            | SameTestRelationKind::ContainsOnly
    )
}

fn split_same_test_options(args: &[Expr], min_data_args: usize) -> (&[Expr], Vec<Expr>) {
    let mut option_start = args.len();
    let mut suffix_groups = Vec::<Vec<Expr>>::new();

    while option_start > min_data_args {
        let Some(entries) = same_test_option_entries(&args[option_start - 1]) else {
            break;
        };
        suffix_groups.push(entries);
        option_start -= 1;
    }

    suffix_groups.reverse();
    let explicit_options = suffix_groups.into_iter().flatten().collect();
    (&args[..option_start], explicit_options)
}

fn same_test_option_entries(expr: &Expr) -> Option<Vec<Expr>> {
    if same_test_option_rule_value(expr).is_some() {
        return Some(vec![expr.clone()]);
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) || args.is_empty() {
        return None;
    }

    let mut entries = Vec::new();
    for arg in args {
        same_test_option_rule_value(arg)?;
        entries.push(arg.clone());
    }
    Some(entries)
}

fn same_test_option_rule_value(expr: &Expr) -> Option<&Expr> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }
    same_test_option_name(&args[0]).filter(|name| *name == "SameTest")?;
    Some(&args[1])
}

fn same_test_option_name(expr: &Expr) -> Option<&str> {
    if let Some(name) = expr_as_string(expr) {
        return Some(name);
    }
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name()),
        _ => None,
    }
}

fn effective_same_test(
    session: &mut Session,
    symbol: &str,
    explicit_options: &[Expr],
) -> Result<Option<Expr>, EvalError> {
    let option_exprs = session.merged_option_exprs_for_system_symbol(symbol, explicit_options)?;
    let mut same_test = None;

    for expr in option_exprs {
        let Some(value) = same_test_option_rule_value(&expr) else {
            return Ok(None);
        };
        same_test = Some(value.clone());
    }

    Ok(same_test.filter(|value| !is_system_symbol_name(value, "Automatic")))
}

fn eval_duplicate_free_same_test(
    session: &mut Session,
    same_test: Expr,
    items: &[Expr],
) -> Result<Expr, EvalError> {
    resume_duplicate_free_same_test(
        session,
        DuplicateFreeSameTestState {
            same_test,
            items: items.to_vec(),
            lhs_index: 0,
            rhs_index: 1,
        },
    )
}

fn resume_duplicate_free_same_test(
    session: &mut Session,
    mut state: DuplicateFreeSameTestState,
) -> Result<Expr, EvalError> {
    loop {
        loop {
            if state.lhs_index >= state.items.len().saturating_sub(1) {
                return Ok(predicate_bool_expr(true));
            }
            if state.rhs_index < state.items.len() {
                break;
            }
            state.lhs_index += 1;
            state.rhs_index = state.lhs_index + 1;
        }

        let value = match eval_same_test_pair(
            session,
            &state.same_test,
            &state.items[state.lhs_index],
            &state.items[state.rhs_index],
        ) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedDuplicateFreeSameTestState {
                                            same_test: state.same_test,
                                            items: FrozenPredicateSequence::freeze(state.items),
                                            lhs_index: state.lhs_index,
                                            rhs_index: state.rhs_index,
                                        };
                ResumeDuplicateFreeSameTestSuspensionFrame { state }
                },
                );
            }
            Err(error) => return Err(error),
        };

        if is_true_expr(&value) {
            return Ok(predicate_bool_expr(false));
        }
        state.rhs_index += 1;
    }
}

fn resume_suspended_duplicate_free_same_test(
    session: &mut Session,
    mut state: SuspendedDuplicateFreeSameTestState,
) -> Result<Expr, EvalError> {
    loop {
        loop {
            if state.lhs_index >= state.items.len().saturating_sub(1) {
                return Ok(predicate_bool_expr(true));
            }
            if state.rhs_index < state.items.len() {
                break;
            }
            state.lhs_index += 1;
            state.rhs_index = state.lhs_index + 1;
        }
        let value = match eval_same_test_pair(
            session,
            &state.same_test,
            state.items.get(state.lhs_index).expect("lhs should exist"),
            state.items.get(state.rhs_index).expect("rhs should exist"),
        ) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedDuplicateFreeSameTestSuspensionFrame { state },
                );
            }
            Err(error) => return Err(error),
        };
        if is_true_expr(&value) {
            return Ok(predicate_bool_expr(false));
        }
        state.rhs_index += 1;
    }
}

fn resume_suspended_duplicate_free_same_test_with_value(
    session: &mut Session,
    mut state: SuspendedDuplicateFreeSameTestState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        return Ok(predicate_bool_expr(false));
    }
    state.rhs_index += 1;
    resume_suspended_duplicate_free_same_test(session, state)
}

fn eval_same_test_relation(
    session: &mut Session,
    same_test: Expr,
    lhs: &[Expr],
    rhs: &[Expr],
    kind: SameTestRelationKind,
) -> Result<Expr, EvalError> {
    resume_same_test_matrix(
        session,
        SameTestMatrixState {
            same_test,
            lhs: lhs.to_vec(),
            rhs: rhs.to_vec(),
            lhs_index: 0,
            rhs_index: 0,
            matches: vec![vec![false; rhs.len()]; lhs.len()],
            kind,
        },
    )
}

fn resume_same_test_matrix(
    session: &mut Session,
    mut state: SameTestMatrixState,
) -> Result<Expr, EvalError> {
    loop {
        loop {
            if state.lhs_index >= state.lhs.len() {
                return Ok(predicate_bool_expr(same_test_relation_result(
                    state.kind,
                    &state.matches,
                    state.lhs.len(),
                    state.rhs.len(),
                )));
            }
            if state.rhs_index < state.rhs.len() {
                break;
            }
            state.lhs_index += 1;
            state.rhs_index = 0;
        }

        let value = match eval_same_test_pair(
            session,
            &state.same_test,
            &state.lhs[state.lhs_index],
            &state.rhs[state.rhs_index],
        ) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedSameTestMatrixState {
                                            same_test: state.same_test,
                                            lhs: FrozenPredicateSequence::freeze(state.lhs),
                                            rhs: FrozenPredicateSequence::freeze(state.rhs),
                                            lhs_index: state.lhs_index,
                                            rhs_index: state.rhs_index,
                                            matches: PersistentPredicateMatches::freeze(state.matches),
                                            kind: state.kind,
                                        };
                ResumeSameTestMatrixSuspensionFrame { state }
                },
                );
            }
            Err(error) => return Err(error),
        };

        if is_true_expr(&value) {
            state.matches[state.lhs_index][state.rhs_index] = true;
        }
        state.rhs_index += 1;
    }
}

fn resume_suspended_same_test_matrix(
    session: &mut Session,
    mut state: SuspendedSameTestMatrixState,
) -> Result<Expr, EvalError> {
    loop {
        loop {
            if state.lhs_index >= state.lhs.len() {
                let matches = state.matches.to_vec();
                return Ok(predicate_bool_expr(same_test_relation_result(
                    state.kind,
                    &matches,
                    state.lhs.len(),
                    state.rhs.len(),
                )));
            }
            if state.rhs_index < state.rhs.len() {
                break;
            }
            state.lhs_index += 1;
            state.rhs_index = 0;
        }
        let value = match eval_same_test_pair(
            session,
            &state.same_test,
            state.lhs.get(state.lhs_index).expect("lhs should exist"),
            state.rhs.get(state.rhs_index).expect("rhs should exist"),
        ) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedSameTestMatrixSuspensionFrame { state },
                );
            }
            Err(error) => return Err(error),
        };
        if is_true_expr(&value) {
            state.matches = state.matches.matched(state.lhs_index, state.rhs_index);
        }
        state.rhs_index += 1;
    }
}

fn resume_suspended_same_test_matrix_with_value(
    session: &mut Session,
    mut state: SuspendedSameTestMatrixState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        state.matches = state.matches.matched(state.lhs_index, state.rhs_index);
    }
    state.rhs_index += 1;
    resume_suspended_same_test_matrix(session, state)
}

fn eval_same_test_pair(
    session: &mut Session,
    same_test: &Expr,
    lhs: &Expr,
    rhs: &Expr,
) -> Result<Expr, EvalError> {
    session.evaluate(Expr::call(
        same_test.clone(),
        vec![lhs.clone(), rhs.clone()],
    ))
}

fn is_true_expr(expr: &Expr) -> bool {
    is_system_symbol_name(expr, heads::structure::TRUE)
}

fn same_test_relation_result(
    kind: SameTestRelationKind,
    matches: &[Vec<bool>],
    lhs_len: usize,
    rhs_len: usize,
) -> bool {
    match kind {
        SameTestRelationKind::Intersecting | SameTestRelationKind::ContainsAny => {
            any_same_test_match(matches)
        }
        SameTestRelationKind::Disjoint | SameTestRelationKind::ContainsNone => {
            !any_same_test_match(matches)
        }
        SameTestRelationKind::Subset | SameTestRelationKind::ContainsAll => {
            all_rhs_items_matched(matches, rhs_len)
        }
        SameTestRelationKind::ContainsOnly => all_lhs_items_matched(matches),
        SameTestRelationKind::ContainsExactly => {
            lhs_len == rhs_len && has_perfect_same_test_matching(matches, rhs_len)
        }
    }
}

fn any_same_test_match(matches: &[Vec<bool>]) -> bool {
    matches.iter().any(|row| row.iter().any(|value| *value))
}

fn all_rhs_items_matched(matches: &[Vec<bool>], rhs_len: usize) -> bool {
    (0..rhs_len).all(|rhs_index| matches.iter().any(|row| row[rhs_index]))
}

fn all_lhs_items_matched(matches: &[Vec<bool>]) -> bool {
    matches.iter().all(|row| row.iter().any(|value| *value))
}

fn has_perfect_same_test_matching(matches: &[Vec<bool>], rhs_len: usize) -> bool {
    if matches.len() <= STACK_MATCHING_THRESHOLD && rhs_len <= STACK_MATCHING_THRESHOLD {
        let mut used_rhs = [false; STACK_MATCHING_THRESHOLD];
        if greedy_same_test_matching(matches, &mut used_rhs[..rhs_len]) {
            return true;
        }
        let mut rhs_matches = [None; STACK_MATCHING_THRESHOLD];
        let mut queue = [0; STACK_MATCHING_THRESHOLD];
        let mut visited_lhs = [false; STACK_MATCHING_THRESHOLD];
        let mut visited_rhs = [false; STACK_MATCHING_THRESHOLD];
        let mut predecessor = [None; STACK_MATCHING_THRESHOLD];
        return augmenting_same_test_matching_with_scratch(
            matches,
            &mut rhs_matches[..rhs_len],
            &mut queue[..matches.len()],
            &mut visited_lhs[..matches.len()],
            &mut visited_rhs[..rhs_len],
            &mut predecessor[..matches.len()],
        );
    }

    let mut used_rhs = vec![false; rhs_len];
    if greedy_same_test_matching(matches, &mut used_rhs) {
        return true;
    }
    drop(used_rhs);

    let mut rhs_matches = vec![None; rhs_len];
    let mut queue = vec![0; matches.len()];
    let mut visited_lhs = vec![false; matches.len()];
    let mut visited_rhs = vec![false; rhs_len];
    let mut predecessor = vec![None; matches.len()];

    augmenting_same_test_matching_with_scratch(
        matches,
        &mut rhs_matches,
        &mut queue,
        &mut visited_lhs,
        &mut visited_rhs,
        &mut predecessor,
    )
}

fn augmenting_same_test_matching_with_scratch(
    matches: &[Vec<bool>],
    rhs_matches: &mut [Option<usize>],
    queue: &mut [usize],
    visited_lhs: &mut [bool],
    visited_rhs: &mut [bool],
    predecessor: &mut [Option<(usize, usize)>],
) -> bool {
    for lhs_index in 0..matches.len() {
        if !augment_same_test_matching(
            lhs_index,
            matches,
            rhs_matches,
            queue,
            visited_lhs,
            visited_rhs,
            predecessor,
        ) {
            return false;
        }
    }
    true
}

fn greedy_same_test_matching(matches: &[Vec<bool>], used_rhs: &mut [bool]) -> bool {
    for row in matches {
        let mut available_rhs = None;
        for rhs_index in 0..used_rhs.len() {
            if row[rhs_index] && !used_rhs[rhs_index] {
                available_rhs = Some(rhs_index);
                break;
            }
        }
        let Some(rhs_index) = available_rhs else {
            return false;
        };
        used_rhs[rhs_index] = true;
    }
    true
}

fn augment_same_test_matching(
    root_lhs: usize,
    matches: &[Vec<bool>],
    rhs_matches: &mut [Option<usize>],
    queue: &mut [usize],
    visited_lhs: &mut [bool],
    visited_rhs: &mut [bool],
    predecessor: &mut [Option<(usize, usize)>],
) -> bool {
    visited_lhs.fill(false);
    visited_rhs.fill(false);
    predecessor.fill(None);
    queue[0] = root_lhs;
    let mut queue_index = 0;
    let mut queue_len = 1;
    visited_lhs[root_lhs] = true;

    while queue_index < queue_len {
        let lhs_index = queue[queue_index];
        queue_index += 1;
        for rhs_index in 0..rhs_matches.len() {
            if !matches[lhs_index][rhs_index] || visited_rhs[rhs_index] {
                continue;
            }
            visited_rhs[rhs_index] = true;

            let Some(next_lhs) = rhs_matches[rhs_index] else {
                install_same_test_augmenting_path(lhs_index, rhs_index, predecessor, rhs_matches);
                return true;
            };
            if !visited_lhs[next_lhs] {
                visited_lhs[next_lhs] = true;
                predecessor[next_lhs] = Some((lhs_index, rhs_index));
                queue[queue_len] = next_lhs;
                queue_len += 1;
            }
        }
    }

    false
}

fn install_same_test_augmenting_path(
    mut lhs_index: usize,
    mut rhs_index: usize,
    predecessor: &[Option<(usize, usize)>],
    rhs_matches: &mut [Option<usize>],
) {
    loop {
        rhs_matches[rhs_index] = Some(lhs_index);
        let Some((previous_lhs, previous_rhs)) = predecessor[lhs_index] else {
            return;
        };
        lhs_index = previous_lhs;
        rhs_index = previous_rhs;
    }
}

#[cfg(test)]
mod matching_tests {
    use rum_core::Symbol;

    use super::{
        DuplicateFreeSameTestState, Expr, FrameRootTrace, FrameRootVisitor,
        has_perfect_same_test_matching,
    };

    #[derive(Default)]
    struct RecordedRoots {
        exprs: Vec<*const Expr>,
        symbols: Vec<*const Symbol>,
    }

    impl FrameRootVisitor for RecordedRoots {
        fn visit_expr(&mut self, expr: &Expr) {
            self.exprs.push(expr as *const Expr);
        }

        fn visit_symbol(&mut self, symbol: &Symbol) {
            self.symbols.push(symbol as *const Symbol);
        }
    }

    #[test]
    fn duplicate_free_state_visits_the_exact_expression_roots() {
        let state = DuplicateFreeSameTestState {
            same_test: Expr::symbol("same-test"),
            items: vec![Expr::symbol("first"), Expr::symbol("second")],
            lhs_index: 0,
            rhs_index: 1,
        };
        let mut roots = RecordedRoots::default();

        state.trace_roots(&mut roots);

        assert_eq!(
            roots.exprs,
            vec![
                &state.same_test as *const Expr,
                &state.items[0] as *const Expr,
                &state.items[1] as *const Expr,
            ]
        );
        assert!(roots.symbols.is_empty());
    }

    #[test]
    fn augmenting_path_matching_agrees_with_exhaustive_small_graphs() {
        for size in 0..=4 {
            let edge_count = size * size;
            for mask in 0..(1_usize << edge_count) {
                let mut matches = vec![vec![false; size]; size];
                for edge in 0..edge_count {
                    matches[edge / size][edge % size] = mask & (1_usize << edge) != 0;
                }

                assert_eq!(
                    has_perfect_same_test_matching(&matches, size),
                    exhaustive_matching_from(0, &matches, &mut vec![false; size]),
                    "matching disagreement for size {size}, mask {mask}"
                );
            }
        }
    }

    fn exhaustive_matching_from(
        lhs_index: usize,
        matches: &[Vec<bool>],
        used_rhs: &mut [bool],
    ) -> bool {
        if lhs_index == matches.len() {
            return true;
        }

        for rhs_index in 0..used_rhs.len() {
            if matches[lhs_index][rhs_index] && !used_rhs[rhs_index] {
                used_rhs[rhs_index] = true;
                if exhaustive_matching_from(lhs_index + 1, matches, used_rhs) {
                    return true;
                }
                used_rhs[rhs_index] = false;
            }
        }
        false
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<DuplicateFreeSameTestState>();
    assert_traces::<SuspendedDuplicateFreeSameTestState>();
    assert_traces::<SameTestMatrixState>();
    assert_traces::<SuspendedSameTestMatrixState>();
    assert_traces::<SameTestRelationKind>();
};
