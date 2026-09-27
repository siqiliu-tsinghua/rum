use crate::option_consumers::OptionConsumer;
mod resume_frames;
use resume_frames::*;

use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::cmp::Ordering;
use std::collections::HashSet;

use super::resumable_merge_sort::{
    ComparatorResultPolicy, MergeCompletion, MergeSortItem, sort_with_comparator,
};
use super::*;

mod suspended;

use self::suspended::{SuspendedSetSameTestState, resume_suspended_set_same_test_with_value};

const SET_MEMBERSHIP_INDEX_THRESHOLD: usize = 16;

#[derive(Clone, Debug, FrameRootTrace)]
struct SortByEntry {
    item: Expr,
    key: Expr,
    index: usize,
}

#[derive(FrameRootTrace)]
struct SortCompletion {
    expr_head: Expr,
}

impl MergeCompletion<Expr> for SortCompletion {
    fn complete(self, items: Vec<Expr>) -> Expr {
        Expr::call(self.expr_head, items)
    }
}

#[derive(FrameRootTrace)]
struct SortByCompletion {
    expr_head: Expr,
}

impl MergeCompletion<SortByEntry> for SortByCompletion {
    fn complete(self, entries: Vec<SortByEntry>) -> Expr {
        Expr::call(
            self.expr_head,
            entries.into_iter().map(|entry| entry.item).collect(),
        )
    }
}

impl MergeSortItem for SortByEntry {
    fn ordering_value(&self) -> &Expr {
        &self.key
    }
}

struct AdaptiveSetMembership<'a> {
    values: &'a [Expr],
    memo: [Option<(Expr, bool)>; SET_MEMBERSHIP_INDEX_THRESHOLD],
    memo_len: usize,
    index: Option<HashSet<&'a Expr>>,
}

impl<'a> AdaptiveSetMembership<'a> {
    fn new(values: &'a [Expr]) -> Self {
        Self {
            values,
            memo: std::array::from_fn(|_| None),
            memo_len: 0,
            index: None,
        }
    }

    fn contains(&mut self, item: &Expr) -> bool {
        if let Some(index) = &self.index {
            return index.contains(item);
        }
        if let Some((_, present)) = self.memo[..self.memo_len]
            .iter()
            .flatten()
            .find(|(cached, _)| cached == item)
        {
            return *present;
        }
        if self.memo_len == SET_MEMBERSHIP_INDEX_THRESHOLD {
            let index = adaptive_membership_index(self.values);
            let present = index.contains(item);
            self.index = Some(index);
            return present;
        }

        let present = self.values.iter().any(|candidate| candidate == item);
        self.memo[self.memo_len] = Some((item.clone(), present));
        self.memo_len += 1;
        present
    }
}

#[derive(FrameRootTrace)]
enum SetOperationKind {
    Union,
    Intersection { groups: Vec<Vec<Expr>> },
    Complement { excluded: Vec<Expr> },
}

#[derive(Clone, Copy, FrameRootTrace)]
enum SetSameTestPhase {
    Scan,
    Deduplicate {
        kept_index: usize,
    },
    Intersection {
        group_index: usize,
        item_index: usize,
    },
    Complement {
        excluded_index: usize,
    },
}

#[derive(FrameRootTrace)]
struct SetSameTestState {
    expr_head: Expr,
    same_test: Expr,
    scan: Vec<Expr>,
    index: usize,
    kept: Vec<Expr>,
    kind: SetOperationKind,
    candidate: Option<Expr>,
    phase: SetSameTestPhase,
}

pub(crate) fn eval_sort(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr] => sort_normal_expression(session, "Sort", args, expr),
        [expr, ordering] => {
            let Expr::Call {
                head: expr_head,
                args: expr_args,
            } = expr
            else {
                return Ok(invalid_ordering_normal_call(session, "Sort", args, 1));
            };

            sort_with_comparator(
                session,
                expr_args.to_vec(),
                ordering.clone(),
                ComparatorResultPolicy::ExactTrue,
                SortCompletion {
                    expr_head: expr_head.as_ref().clone(),
                },
            )
        }
        _ => Ok(session.invalid_call_with_arg_range("Sort", args, 1, 2)),
    }
}

pub(crate) fn eval_sort_by(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [function] => Ok(sort_by_operator_form(function)),
        [expr, function] => {
            let Expr::Call {
                head: expr_head,
                args: expr_args,
            } = expr
            else {
                return Ok(invalid_ordering_normal_call(session, "SortBy", args, 1));
            };

            resume_sort_by_from_index(
                session,
                expr_head.as_ref().clone(),
                function.clone(),
                expr_args.to_vec(),
                0,
                Vec::new(),
            )
        }
        [expr, function, ordering] => {
            let Expr::Call {
                head: expr_head,
                args: expr_args,
            } = expr
            else {
                return Ok(invalid_ordering_normal_call(session, "SortBy", args, 1));
            };

            resume_sort_by_ordering_from_index(
                session,
                expr_head.as_ref().clone(),
                function.clone(),
                ordering.clone(),
                expr_args.to_vec(),
                0,
                Vec::new(),
            )
        }
        _ => Ok(session.invalid_call_with_arg_range("SortBy", args, 1, 3)),
    }
}

pub(crate) fn eval_union(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_union_with_options(session, _head, args, None)
}

pub(crate) fn eval_union_with_options(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
    resolved_options: Option<rum_runtime::kernel_api::RetainedOptions<'_>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.is_empty() {
        return Ok(Expr::call(system_expr("List"), Vec::new()));
    }

    let (data_args, explicit_options) = split_same_test_options(args);
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::Union,
                _head,
                args,
                &explicit_options,
            );
        }
    };
    let same_test = effective_same_test(option_exprs)?;
    enforce_union_memory_constraint(session, data_args)?;
    let (expr_head, groups) = match same_head_argument_groups(session, "Union", data_args) {
        Ok(groups) => groups,
        Err(value) => return Ok(value),
    };
    let item_count = groups
        .iter()
        .fold(0usize, |total, group| total.saturating_add(group.len()));
    let mut items = Vec::with_capacity(item_count);
    items.extend(groups.into_iter().flatten());
    if let Some(same_test) = same_test {
        let mut scan = items;
        scan.sort_by(compare_structural_order);
        let kept = Vec::with_capacity(scan.len());
        return resume_set_same_test(
            session,
            SetSameTestState {
                expr_head,
                same_test,
                scan,
                index: 0,
                kept,
                kind: SetOperationKind::Union,
                candidate: None,
                phase: SetSameTestPhase::Scan,
            },
        );
    }

    Ok(Expr::call(expr_head, sorted_unique_exprs(items)))
}

fn enforce_union_memory_constraint(session: &Session, args: &[Expr]) -> Result<(), EvalError> {
    if session.active_memory_limit().is_none() {
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    }

    let Some((first_head, _)) = args.first().and_then(call_head_and_args) else {
        return Ok(());
    };
    if !args
        .iter()
        .all(|arg| call_head_and_args(arg).is_some_and(|(head, _)| head == first_head))
    {
        return Ok(());
    }

    super::surface::enforce_projected_call_memory_constraint(
        session,
        first_head,
        args.iter()
            .filter_map(call_head_and_args)
            .flat_map(|(_, items)| items.iter()),
    )
}

pub(crate) fn eval_intersection(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_intersection_with_options(session, _head, args, None)
}

pub(crate) fn eval_intersection_with_options(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
    resolved_options: Option<rum_runtime::kernel_api::RetainedOptions<'_>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Intersection",
            args,
            "argm",
            "Intersection called with 0 arguments; 1 or more arguments are expected.",
        ));
    }

    let (data_args, explicit_options) = split_same_test_options(args);
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::Intersection,
                _head,
                args,
                &explicit_options,
            );
        }
    };
    let same_test = effective_same_test(option_exprs)?;
    let (expr_head, mut groups) =
        match same_head_argument_groups(session, "Intersection", data_args) {
            Ok(groups) => groups,
            Err(value) => return Ok(value),
        };
    let first = groups.remove(0);
    if let Some(same_test) = same_test {
        let mut scan = first;
        scan.sort_by(compare_structural_order);
        return resume_set_same_test(
            session,
            SetSameTestState {
                expr_head,
                same_test,
                scan,
                index: 0,
                kept: Vec::new(),
                kind: SetOperationKind::Intersection { groups },
                candidate: None,
                phase: SetSameTestPhase::Scan,
            },
        );
    }

    let first = sorted_unique_exprs(first);
    let group_indices = groups
        .iter()
        .map(|group| {
            (first.len() > SET_MEMBERSHIP_INDEX_THRESHOLD
                && group.len() > SET_MEMBERSHIP_INDEX_THRESHOLD)
                .then(|| adaptive_membership_index(group))
        })
        .collect::<Vec<_>>();
    let common = first
        .into_iter()
        .filter(|item| {
            groups.iter().zip(&group_indices).all(|(group, index)| {
                index.as_ref().map_or_else(
                    || group.iter().any(|candidate| candidate == item),
                    |index| index.contains(item),
                )
            })
        })
        .collect::<Vec<_>>();
    Ok(Expr::call(expr_head, common))
}

pub(crate) fn eval_complement(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_complement_with_options(session, _head, args, None)
}

pub(crate) fn eval_complement_with_options(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
    resolved_options: Option<rum_runtime::kernel_api::RetainedOptions<'_>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Complement",
            args,
            "argm",
            "Complement called with 0 arguments; 1 or more arguments are expected.",
        ));
    }

    let (data_args, explicit_options) = split_same_test_options(args);
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::Complement,
                _head,
                args,
                &explicit_options,
            );
        }
    };
    let same_test = effective_same_test(option_exprs)?;
    let (expr_head, mut groups) = match same_head_argument_groups(session, "Complement", data_args)
    {
        Ok(groups) => groups,
        Err(value) => return Ok(value),
    };
    let first = groups.remove(0);
    let excluded = groups.into_iter().flatten().collect::<Vec<_>>();
    if let Some(same_test) = same_test {
        let mut scan = first;
        scan.sort_by(compare_structural_order);
        return resume_set_same_test(
            session,
            SetSameTestState {
                expr_head,
                same_test,
                scan,
                index: 0,
                kept: Vec::new(),
                kind: SetOperationKind::Complement { excluded },
                candidate: None,
                phase: SetSameTestPhase::Scan,
            },
        );
    }

    let mut excluded_membership = AdaptiveSetMembership::new(&excluded);
    let kept = first
        .into_iter()
        .filter(|item| !excluded_membership.contains(item))
        .collect::<Vec<_>>();
    Ok(Expr::call(expr_head, sorted_unique_exprs(kept)))
}

fn sort_normal_expression(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    expr: &Expr,
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: expr_head,
        args: expr_args,
    } = expr
    else {
        return Ok(invalid_ordering_normal_call(session, symbol, args, 1));
    };

    let mut sorted = expr_args.to_vec();
    sorted.sort_by(compare_structural_order);
    Ok(Expr::call(expr_head.as_ref().clone(), sorted))
}

fn same_head_argument_groups(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
) -> Result<(Expr, Vec<Vec<Expr>>), Expr> {
    let Some((first_head, first_args)) = call_head_and_args(&args[0]) else {
        return Err(invalid_ordering_normal_call(session, symbol, args, 1));
    };

    let mut groups = vec![first_args.to_vec()];
    for (index, arg) in args.iter().enumerate().skip(1) {
        let Some((head, items)) = call_head_and_args(arg) else {
            return Err(invalid_ordering_normal_call(
                session,
                symbol,
                args,
                index + 1,
            ));
        };
        if head != first_head {
            return Err(invalid_ordering_heads_call(
                session,
                symbol,
                args,
                index + 1,
                head,
                first_head,
            ));
        }
        groups.push(items.to_vec());
    }

    Ok((first_head.clone(), groups))
}

fn adaptive_membership_index(values: &[Expr]) -> HashSet<&Expr> {
    let mut index = HashSet::with_capacity(SET_MEMBERSHIP_INDEX_THRESHOLD + 1);
    let mut reserved_for_all = false;
    for (position, value) in values.iter().enumerate() {
        index.insert(value);
        if !reserved_for_all && index.len() > SET_MEMBERSHIP_INDEX_THRESHOLD {
            index.reserve(values.len().saturating_sub(position + 1));
            reserved_for_all = true;
        }
    }
    index
}

fn split_same_test_options(args: &[Expr]) -> (&[Expr], Vec<Expr>) {
    let mut option_start = args.len();
    let mut suffix_groups = Vec::<Vec<Expr>>::new();

    while option_start > 1 {
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

    let Expr::Call { head, args } = expr else {
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
    let Expr::Call { head, args } = expr else {
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
    option_exprs: rum_runtime::kernel_api::RetainedOptions<'_>,
) -> Result<Option<Expr>, EvalError> {
    let mut same_test = None;

    for expr in option_exprs {
        let Some(value) = same_test_option_rule_value(&expr) else {
            return Ok(None);
        };
        same_test = Some(value.clone());
    }

    Ok(same_test.filter(|value| !is_system_symbol_name(value, "Automatic")))
}

fn resume_set_same_test(
    session: &mut Session,
    mut state: SetSameTestState,
) -> Result<Expr, EvalError> {
    loop {
        match state.phase {
            SetSameTestPhase::Scan => {
                let Some(candidate) = state.scan.get(state.index).cloned() else {
                    return Ok(Expr::call(state.expr_head, state.kept));
                };
                state.candidate = Some(candidate);
                if state.kept.is_empty() {
                    begin_set_same_test_membership(&mut state);
                } else {
                    state.phase = SetSameTestPhase::Deduplicate { kept_index: 0 };
                }
            }
            SetSameTestPhase::Deduplicate { kept_index } => {
                let Some(existing) = state.kept.get(kept_index) else {
                    begin_set_same_test_membership(&mut state);
                    continue;
                };
                let candidate = state
                    .candidate
                    .as_ref()
                    .expect("deduplication should have a candidate");
                let value =
                    match eval_binary_predicate(session, &state.same_test, candidate, existing) {
                        Ok(value) => value,
                        Err(EvalError::InputRequired { request }) => {
                            return rum_runtime::resume_or_suspend!(
                                session,
                                Err(EvalError::InputRequired { request }),
                                ok(value) => Ok(value),
                                suspend => {
                            let state = SuspendedSetSameTestState::freeze(state);
                            ResumeSetSameTestSuspensionFrame1 { state }
                            },
                            );
                        }
                        Err(error) => return Err(error),
                    };
                advance_set_same_test_after_value(&mut state, &value);
            }
            SetSameTestPhase::Intersection {
                group_index,
                item_index,
            } => {
                let SetOperationKind::Intersection { groups } = &state.kind else {
                    unreachable!("intersection phase requires intersection state")
                };
                let Some(group) = groups.get(group_index) else {
                    finish_set_same_test_candidate(&mut state, true);
                    continue;
                };
                let Some(item) = group.get(item_index) else {
                    finish_set_same_test_candidate(&mut state, false);
                    continue;
                };
                let candidate = state
                    .candidate
                    .as_ref()
                    .expect("intersection should have a candidate");
                let value = match eval_binary_predicate(session, &state.same_test, candidate, item)
                {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        return rum_runtime::resume_or_suspend!(
                            session,
                            Err(EvalError::InputRequired { request }),
                            ok(value) => Ok(value),
                            suspend => {
                        let state = SuspendedSetSameTestState::freeze(state);
                        ResumeSetSameTestSuspensionFrame2 { state }
                        },
                        );
                    }
                    Err(error) => return Err(error),
                };
                advance_set_same_test_after_value(&mut state, &value);
            }
            SetSameTestPhase::Complement { excluded_index } => {
                let SetOperationKind::Complement { excluded } = &state.kind else {
                    unreachable!("complement phase requires complement state")
                };
                let Some(item) = excluded.get(excluded_index) else {
                    finish_set_same_test_candidate(&mut state, true);
                    continue;
                };
                let candidate = state
                    .candidate
                    .as_ref()
                    .expect("complement should have a candidate");
                let value = match eval_binary_predicate(session, &state.same_test, candidate, item)
                {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        return rum_runtime::resume_or_suspend!(
                            session,
                            Err(EvalError::InputRequired { request }),
                            ok(value) => Ok(value),
                            suspend => {
                        let state = SuspendedSetSameTestState::freeze(state);
                        ResumeSetSameTestSuspensionFrame3 { state }
                        },
                        );
                    }
                    Err(error) => return Err(error),
                };
                advance_set_same_test_after_value(&mut state, &value);
            }
        }
    }
}

fn begin_set_same_test_membership(state: &mut SetSameTestState) {
    match &state.kind {
        SetOperationKind::Union => finish_set_same_test_candidate(state, true),
        SetOperationKind::Intersection { .. } => {
            state.phase = SetSameTestPhase::Intersection {
                group_index: 0,
                item_index: 0,
            };
        }
        SetOperationKind::Complement { .. } => {
            state.phase = SetSameTestPhase::Complement { excluded_index: 0 };
        }
    }
}

fn advance_set_same_test_after_value(state: &mut SetSameTestState, value: &Expr) {
    let matched = is_true_expr(value);
    match state.phase {
        SetSameTestPhase::Scan => unreachable!("scan phase does not evaluate SameTest"),
        SetSameTestPhase::Deduplicate { kept_index } => {
            if matched {
                finish_set_same_test_candidate(state, false);
            } else {
                state.phase = SetSameTestPhase::Deduplicate {
                    kept_index: kept_index + 1,
                };
            }
        }
        SetSameTestPhase::Intersection {
            group_index,
            item_index,
        } => {
            state.phase = if matched {
                SetSameTestPhase::Intersection {
                    group_index: group_index + 1,
                    item_index: 0,
                }
            } else {
                SetSameTestPhase::Intersection {
                    group_index,
                    item_index: item_index + 1,
                }
            };
        }
        SetSameTestPhase::Complement { excluded_index } => {
            if matched {
                finish_set_same_test_candidate(state, false);
            } else {
                state.phase = SetSameTestPhase::Complement {
                    excluded_index: excluded_index + 1,
                };
            }
        }
    }
}

fn finish_set_same_test_candidate(state: &mut SetSameTestState, keep: bool) {
    let candidate = state
        .candidate
        .take()
        .expect("SameTest candidate should exist while finishing it");
    if keep {
        state.kept.push(candidate);
    }
    state.index += 1;
    state.phase = SetSameTestPhase::Scan;
}

fn resume_sort_by_from_index(
    session: &mut Session,
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    mut index: usize,
    mut decorated: Vec<SortByEntry>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            let mut sorted = decorated;
            sorted.sort_by(|lhs, rhs| {
                compare_structural_order(&lhs.key, &rhs.key).then(lhs.index.cmp(&rhs.index))
            });
            return Ok(Expr::call(
                expr_head,
                sorted.into_iter().map(|entry| entry.item).collect(),
            ));
        };

        let key = match session.evaluate(sort_by_key_expression(&function, &item)) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSortByFromIndexSuspensionFrame {
                        expr_head,
                        function,
                        items: FrozenStructureSequence::freeze(items),
                        index,
                        decorated: PersistentStructureSequence::freeze(decorated),
                        item,
                    },
                );
            }
            Err(error) => return Err(error),
        };

        decorated.push(SortByEntry { item, key, index });
        index += 1;
    }
}

fn resume_suspended_sort_by_from_index(
    session: &mut Session,
    expr_head: Expr,
    function: Expr,
    items: FrozenStructureSequence<Expr>,
    mut index: usize,
    mut decorated: PersistentStructureSequence<SortByEntry>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            let mut sorted = decorated.to_vec();
            sorted.sort_by(|lhs, rhs| {
                compare_structural_order(&lhs.key, &rhs.key).then(lhs.index.cmp(&rhs.index))
            });
            return Ok(Expr::call(
                expr_head,
                sorted.into_iter().map(|entry| entry.item).collect(),
            ));
        };

        let key = match session.evaluate(sort_by_key_expression(&function, &item)) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSortByFromIndexSuspensionFrame {
                        expr_head,
                        function,
                        items,
                        index,
                        decorated,
                        item,
                    },
                );
            }
            Err(error) => return Err(error),
        };

        decorated = decorated.appended(SortByEntry { item, key, index });
        index += 1;
    }
}

fn resume_suspended_sort_by_with_key(
    session: &mut Session,
    expr_head: Expr,
    function: Expr,
    items: FrozenStructureSequence<Expr>,
    index: usize,
    decorated: PersistentStructureSequence<SortByEntry>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    let decorated = decorated.appended(SortByEntry { item, key, index });
    resume_suspended_sort_by_from_index(session, expr_head, function, items, index + 1, decorated)
}

#[allow(clippy::too_many_arguments)]
fn resume_sort_by_ordering_from_index(
    session: &mut Session,
    expr_head: Expr,
    function: Expr,
    ordering: Expr,
    items: Vec<Expr>,
    mut index: usize,
    mut decorated: Vec<SortByEntry>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            return sort_with_comparator(
                session,
                decorated,
                ordering,
                ComparatorResultPolicy::ExactTrue,
                SortByCompletion { expr_head },
            );
        };

        let key = match session.evaluate(sort_by_key_expression(&function, &item)) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSortByOrderingFromIndexSuspensionFrame {
                        expr_head,
                        function,
                        ordering,
                        items: FrozenStructureSequence::freeze(items),
                        index,
                        decorated: PersistentStructureSequence::freeze(decorated),
                        item,
                    },
                );
            }
            Err(error) => return Err(error),
        };

        decorated.push(SortByEntry { item, key, index });
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_sort_by_ordering_from_index(
    session: &mut Session,
    expr_head: Expr,
    function: Expr,
    ordering: Expr,
    items: FrozenStructureSequence<Expr>,
    mut index: usize,
    mut decorated: PersistentStructureSequence<SortByEntry>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            return sort_with_comparator(
                session,
                decorated.to_vec(),
                ordering,
                ComparatorResultPolicy::ExactTrue,
                SortByCompletion { expr_head },
            );
        };

        let key = match session.evaluate(sort_by_key_expression(&function, &item)) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSortByOrderingFromIndexSuspensionFrame {
                        expr_head,
                        function,
                        ordering,
                        items,
                        index,
                        decorated,
                        item,
                    },
                );
            }
            Err(error) => return Err(error),
        };

        decorated = decorated.appended(SortByEntry { item, key, index });
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_sort_by_ordering_with_key(
    session: &mut Session,
    expr_head: Expr,
    function: Expr,
    ordering: Expr,
    items: FrozenStructureSequence<Expr>,
    index: usize,
    decorated: PersistentStructureSequence<SortByEntry>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    let decorated = decorated.appended(SortByEntry { item, key, index });
    resume_suspended_sort_by_ordering_from_index(
        session,
        expr_head,
        function,
        ordering,
        items,
        index + 1,
        decorated,
    )
}

fn invalid_ordering_normal_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    position: usize,
) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position {position} in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    )
}

fn invalid_ordering_heads_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    position: usize,
    actual_head: &Expr,
    expected_head: &Expr,
) -> Expr {
    session.invalid_call_with_message(
            symbol,
            args,
            "heads",
            format!(
                "Heads {actual_head} and {expected_head} at positions {position} and 1 are expected to be the same."
            ),
        )
}

fn sort_by_operator_form(function: &Expr) -> Expr {
    let arg = Expr::symbol("rum$sortArg");
    Expr::call(
        system_expr("Function"),
        vec![
            arg.clone(),
            Expr::call(system_expr("SortBy"), vec![arg, function.clone()]),
        ],
    )
}

fn sort_by_key_expression(function: &Expr, item: &Expr) -> Expr {
    let Some(functions) = list_items(function) else {
        return Expr::call(function.clone(), vec![item.clone()]);
    };

    Expr::call(
        system_expr("List"),
        functions
            .iter()
            .map(|function| Expr::call(function.clone(), vec![item.clone()]))
            .collect(),
    )
}

fn call_head_and_args(expr: &Expr) -> Option<(&Expr, &[Expr])> {
    match expr {
        Expr::Call { head, args } => Some((head.as_ref(), args)),
        Expr::Atom(_) => None,
    }
}

fn sorted_unique_exprs(mut items: Vec<Expr>) -> Vec<Expr> {
    items.sort_by(compare_structural_order);
    items.dedup();
    items
}

pub(super) fn compare_structural_order(lhs: &Expr, rhs: &Expr) -> Ordering {
    match (lhs, rhs) {
        (Expr::Atom(lhs), Expr::Atom(rhs)) => compare_atom_order(lhs, rhs),
        (Expr::Atom(_), Expr::Call { .. }) => Ordering::Less,
        (Expr::Call { .. }, Expr::Atom(_)) => Ordering::Greater,
        (
            Expr::Call {
                head: lhs_head,
                args: lhs_args,
            },
            Expr::Call {
                head: rhs_head,
                args: rhs_args,
            },
        ) => compare_structural_order(lhs_head, rhs_head)
            .then_with(|| compare_expr_slices(lhs_args, rhs_args)),
    }
}

fn compare_atom_order(lhs: &Atom, rhs: &Atom) -> Ordering {
    atom_order_rank(lhs)
        .cmp(&atom_order_rank(rhs))
        .then_with(|| match (lhs, rhs) {
            (Atom::Integer(lhs), Atom::Integer(rhs)) => lhs.cmp(rhs),
            (Atom::String(lhs), Atom::String(rhs)) => lhs.cmp(rhs),
            (Atom::Symbol(lhs), Atom::Symbol(rhs)) => lhs.cmp_qualified_name(rhs),
            (Atom::Rational(lhs), Atom::Rational(rhs)) => lhs.cmp(rhs),
            _ => lhs.to_string().cmp(&rhs.to_string()),
        })
}

fn atom_order_rank(atom: &Atom) -> u8 {
    match atom {
        Atom::Integer(_) | Atom::Rational(_) => 0,
        Atom::String(_) => 1,
        Atom::Symbol(_) => 2,
    }
}

fn compare_expr_slices(lhs: &[Expr], rhs: &[Expr]) -> Ordering {
    for (lhs, rhs) in lhs.iter().zip(rhs) {
        let ordering = compare_structural_order(lhs, rhs);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    lhs.len().cmp(&rhs.len())
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SetOperationKind>();
    assert_traces::<SetSameTestPhase>();
    assert_traces::<SetSameTestState>();
    assert_traces::<SuspendedSetSameTestState>();
    assert_traces::<SortByCompletion>();
    assert_traces::<SortByEntry>();
    assert_traces::<SortCompletion>();
};
