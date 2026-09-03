mod resume_frames;
use resume_frames::*;

use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::cmp::Ordering;

use super::ordering_sets::compare_structural_order;
use super::resumable_merge_sort::{ComparatorResultPolicy, MergeCompletion, sort_with_comparator};
use super::stable_groups::{StableCounts, StableGroups};
use super::*;

#[derive(Clone, Debug, FrameRootTrace)]
struct KeyedEntry {
    item: Expr,
    key: Expr,
    index: usize,
}

#[derive(FrameRootTrace)]
struct ReverseSortCompletion {
    expr_head: Expr,
}

impl MergeCompletion<Expr> for ReverseSortCompletion {
    fn complete(self, mut items: Vec<Expr>) -> Expr {
        items.reverse();
        Expr::call(self.expr_head, items)
    }
}

#[derive(Clone, Debug)]
struct GatherPathEntry {
    item: Expr,
    keys: Vec<Expr>,
}

#[derive(Clone, FrameRootTrace)]
struct DecorateByState {
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    operation: DecoratedOperation,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedDecorateByState {
    expr_head: Expr,
    function: Expr,
    items: FrozenStructureSequence<Expr>,
    operation: DecoratedOperation,
}

impl SuspendedDecorateByState {
    fn freeze(state: DecorateByState) -> Self {
        Self {
            expr_head: state.expr_head,
            function: state.function,
            items: FrozenStructureSequence::freeze(state.items),
            operation: state.operation,
        }
    }
}

#[derive(FrameRootTrace)]
struct TallyTestState {
    test: Expr,
    items: Vec<Expr>,
    index: usize,
    counts: Vec<(Expr, usize)>,
}

#[derive(Clone, FrameRootTrace)]
struct GatherByFunctionListState {
    items: Vec<Expr>,
    functions: Vec<Expr>,
    item_index: usize,
    function_index: usize,
    keys: Vec<Vec<Expr>>,
}

#[derive(Clone, Copy, FrameRootTrace)]
enum DecoratedOperation {
    MaxBy { select_max: bool },
    ExtremalElementsBy { select_max: bool },
    ExtremeByCount { largest: bool, count: usize },
    TakeBy { largest: bool, count: usize },
    ReverseSortBy,
    GatherBy,
}

pub(crate) fn eval_tally(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, test) = match args {
        [expr] => (expr, None),
        [expr, test] => (expr, Some(test)),
        _ => return Ok(session.invalid_call_with_exact_arg_count("Tally", args, 1)),
    };

    let Some((_expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "Tally", args));
    };

    if let Some(test) = test {
        return resume_tally_test_from_count(
            session,
            TallyTestState {
                test: test.clone(),
                items,
                index: 0,
                counts: Vec::new(),
            },
            0,
        );
    }

    let counts = StableCounts::collect(items).into_entries();

    Ok(list_expr(
        counts
            .into_iter()
            .map(|(item, count)| list_expr(vec![item, Expr::integer(BigInt::from(count))]))
            .collect(),
    ))
}

fn resume_tally_test_from_count(
    session: &mut Session,
    mut state: TallyTestState,
    mut count_index: usize,
) -> Result<Expr, EvalError> {
    loop {
        if state.index >= state.items.len() {
            return Ok(render_tally_counts(state.counts));
        }

        if count_index >= state.counts.len() {
            state.counts.push((state.items[state.index].clone(), 1));
            state.index += 1;
            count_index = 0;
            continue;
        }

        let result = eval_binary_predicate(
            session,
            &state.test,
            &state.items[state.index],
            &state.counts[count_index].0,
        );
        let value = match result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let items = FrozenStructureSequence::freeze(state.items);
                                        let counts = PersistentStructureCounts::freeze(state.counts);
                ResumeTallyTestFromCountSuspensionFrame { state_test: state.test, state_index: state.index, items, counts, count_index }
                },
                );
            }
            Err(error) => return Err(error),
        };

        if is_true_expr(&value) {
            state.counts[count_index].1 += 1;
            state.index += 1;
            count_index = 0;
        } else {
            count_index += 1;
        }
    }
}

fn resume_suspended_tally_test_from_count(
    session: &mut Session,
    test: Expr,
    items: FrozenStructureSequence<Expr>,
    mut index: usize,
    mut counts: PersistentStructureCounts,
    mut count_index: usize,
) -> Result<Expr, EvalError> {
    loop {
        if index >= items.len() {
            return Ok(render_tally_counts(counts.to_vec()));
        }
        if count_index >= counts.len() {
            counts = counts.appended(items.get(index).expect("item should exist").clone());
            index += 1;
            count_index = 0;
            continue;
        }
        let count_item = counts.item(count_index).expect("count item should exist");
        let result = eval_binary_predicate(
            session,
            &test,
            items.get(index).expect("item should exist"),
            &count_item,
        );
        let value = match result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedTallyTestFromCountSuspensionFrame { test, items, index, counts, count_index },
                );
            }
            Err(error) => return Err(error),
        };
        if is_true_expr(&value) {
            counts = counts.incremented(count_index);
            index += 1;
            count_index = 0;
        } else {
            count_index += 1;
        }
    }
}

fn resume_suspended_tally_test_with_value(
    session: &mut Session,
    test: Expr,
    items: FrozenStructureSequence<Expr>,
    mut index: usize,
    mut counts: PersistentStructureCounts,
    count_index: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    let count_index = if is_true_expr(&value) {
        counts = counts.incremented(count_index);
        index += 1;
        0
    } else {
        count_index + 1
    };
    resume_suspended_tally_test_from_count(session, test, items, index, counts, count_index)
}

fn render_tally_counts(counts: Vec<(Expr, usize)>) -> Expr {
    list_expr(
        counts
            .into_iter()
            .map(|(item, count)| list_expr(vec![item, Expr::integer(BigInt::from(count))]))
            .collect(),
    )
}

pub(crate) fn eval_max_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_extreme_by(session, head, "MaxBy", args, true)
}

pub(crate) fn eval_min_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_extreme_by(session, head, "MinBy", args, false)
}

pub(crate) fn eval_maximal_by(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_extremal_elements_by(session, "MaximalBy", args, true)
}

pub(crate) fn eval_minimal_by(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_extremal_elements_by(session, "MinimalBy", args, false)
}

pub(crate) fn eval_take_largest_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_take_by(session, head, args, "TakeLargestBy", true)
}

pub(crate) fn eval_take_smallest_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_take_by(session, head, args, "TakeSmallestBy", false)
}

pub(crate) fn eval_reverse_sort(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, ordering) = match args {
        [expr] => (expr, None),
        [expr, ordering] => (expr, Some(ordering)),
        _ => return Ok(session.invalid_call_with_exact_arg_count("ReverseSort", args, 1)),
    };

    let Some((expr_head, mut items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "ReverseSort", args));
    };

    if let Some(ordering) = ordering {
        return sort_with_comparator(
            session,
            items,
            ordering.clone(),
            ComparatorResultPolicy::ExactTrue,
            ReverseSortCompletion { expr_head },
        );
    }

    items.sort_by(compare_structural_order);
    items.reverse();
    Ok(Expr::call(expr_head, items))
}

pub(crate) fn eval_reverse_sort_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr, function] = args else {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_exact_arg_count("ReverseSortBy", args, 2));
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(
            session,
            "ReverseSortBy",
            args,
        ));
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: DecoratedOperation::ReverseSortBy,
        },
        0,
        Vec::new(),
    )
}

pub(crate) fn eval_gather_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr, function] = args else {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_exact_arg_count("GatherBy", args, 2));
    };

    let Some((_expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "GatherBy", args));
    };

    if let Some(functions) = list_items(function) {
        return resume_gather_by_function_list_from_position(
            session,
            GatherByFunctionListState {
                items,
                functions: functions.to_vec(),
                item_index: 0,
                function_index: 0,
                keys: Vec::new(),
            },
        );
    }

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head: system_expr("List"),
            function: function.clone(),
            items,
            operation: DecoratedOperation::GatherBy,
        },
        0,
        Vec::new(),
    )
}

pub(crate) fn eval_riffle(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, separator, positions) = match args {
        [expr, separator] => (expr, separator, RifflePositions::Every(1)),
        [expr, separator, positions] => {
            let Some(positions) = parse_riffle_positions(positions) else {
                return Ok(invalid_riffle_position_call(session, args));
            };
            (expr, separator, positions)
        }
        _ => {
            if args.len() == 1 {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
            return Ok(session.invalid_call_with_arg_range("Riffle", args, 2, 3));
        }
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "Riffle", args));
    };

    if items.len() <= 1 {
        return Ok(Expr::call(expr_head, items));
    }

    let insertion_positions = riffle_insertion_positions(&positions, items.len());
    let separator_count = insertion_positions.iter().filter(|insert| **insert).count();
    let output_len = items.len().saturating_add(separator_count);
    session.enforce_generated_element_limit_for_count(output_len)?;

    let mut riffled = Vec::with_capacity(output_len);
    for (index, item) in items.into_iter().enumerate() {
        riffled.push(item);
        if insertion_positions[index + 1] {
            riffled.push(separator.clone());
        }
    }

    Ok(Expr::call(expr_head, riffled))
}

fn resume_gather_by_function_list_from_position(
    session: &mut Session,
    mut state: GatherByFunctionListState,
) -> Result<Expr, EvalError> {
    if state.functions.is_empty() {
        return Ok(list_expr(state.items));
    }

    loop {
        if state.item_index >= state.items.len() {
            return Ok(render_hierarchical_gather_by(state.items, state.keys));
        }
        if state.function_index >= state.functions.len() {
            state.item_index += 1;
            state.function_index = 0;
            continue;
        }

        while state.keys.len() <= state.item_index {
            state.keys.push(Vec::with_capacity(state.functions.len()));
        }

        let item = state.items[state.item_index].clone();
        let function = state.functions[state.function_index].clone();
        let key = match session.evaluate(Expr::call(function, vec![item])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let item_index = state.item_index;
                                        let function_index = state.function_index;
                                        let items = FrozenStructureSequence::freeze(state.items);
                                        let functions = FrozenStructureSequence::freeze(state.functions);
                                        let keys =
                                            PersistentStructureSequence::freeze(flatten_gather_keys(state.keys));
                ResumeGatherByFunctionListFromPositionSuspensionFrame { items, functions, item_index, function_index, keys }
                },
                );
            }
            Err(error) => return Err(error),
        };

        state.keys[state.item_index].push(key);
        state.function_index += 1;
    }
}

fn flatten_gather_keys(keys: Vec<Vec<Expr>>) -> Vec<(usize, Expr)> {
    keys.into_iter()
        .enumerate()
        .flat_map(|(item_index, keys)| keys.into_iter().map(move |key| (item_index, key)))
        .collect()
}

fn render_suspended_hierarchical_gather_by(
    items: &FrozenStructureSequence<Expr>,
    functions: &FrozenStructureSequence<Expr>,
    keys: &PersistentStructureSequence<(usize, Expr)>,
) -> Expr {
    let mut materialized = vec![Vec::with_capacity(functions.len()); items.len()];
    for (item_index, key) in keys.to_vec() {
        materialized[item_index].push(key);
    }
    render_hierarchical_gather_by(items.as_slice().to_vec(), materialized)
}

fn resume_suspended_gather_by_function_list_from_position(
    session: &mut Session,
    items: FrozenStructureSequence<Expr>,
    functions: FrozenStructureSequence<Expr>,
    mut item_index: usize,
    mut function_index: usize,
    mut keys: PersistentStructureSequence<(usize, Expr)>,
) -> Result<Expr, EvalError> {
    if functions.len() == 0 {
        return Ok(list_expr(items.as_slice().to_vec()));
    }
    loop {
        if item_index >= items.len() {
            return Ok(render_suspended_hierarchical_gather_by(
                &items, &functions, &keys,
            ));
        }
        if function_index >= functions.len() {
            item_index += 1;
            function_index = 0;
            continue;
        }
        let item = items.get(item_index).expect("item should exist").clone();
        let function = functions
            .get(function_index)
            .expect("function should exist")
            .clone();
        let key = match session.evaluate(Expr::call(function, vec![item])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedGatherByFunctionListFromPositionSuspensionFrame { items, functions, item_index, function_index, keys },
                );
            }
            Err(error) => return Err(error),
        };
        keys = keys.appended((item_index, key));
        function_index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_gather_by_function_list_with_key(
    session: &mut Session,
    items: FrozenStructureSequence<Expr>,
    functions: FrozenStructureSequence<Expr>,
    item_index: usize,
    function_index: usize,
    keys: PersistentStructureSequence<(usize, Expr)>,
    key: Expr,
) -> Result<Expr, EvalError> {
    resume_suspended_gather_by_function_list_from_position(
        session,
        items,
        functions,
        item_index,
        function_index + 1,
        keys.appended((item_index, key)),
    )
}

fn render_hierarchical_gather_by(items: Vec<Expr>, keys: Vec<Vec<Expr>>) -> Expr {
    let key_count = keys.first().map(|item_keys| item_keys.len()).unwrap_or(0);
    let entries = items
        .into_iter()
        .zip(keys)
        .map(|(item, keys)| GatherPathEntry { item, keys })
        .collect();
    list_expr(hierarchical_gather_entries(entries, 0, key_count))
}

fn hierarchical_gather_entries(
    entries: Vec<GatherPathEntry>,
    depth: usize,
    key_count: usize,
) -> Vec<Expr> {
    if depth >= key_count {
        return entries.into_iter().map(|entry| entry.item).collect();
    }

    StableGroups::collect(
        entries
            .into_iter()
            .map(|entry| (entry.keys[depth].clone(), entry)),
    )
    .into_entries()
    .into_iter()
    .map(|(_, group)| list_expr(hierarchical_gather_entries(group, depth + 1, key_count)))
    .collect()
}

fn eval_extreme_by(
    session: &mut Session,
    head: &Expr,
    symbol: &str,
    args: &[Expr],
    select_max: bool,
) -> Result<Expr, EvalError> {
    let (expr, function, count) = match args {
        [expr, function] => (expr, function, None),
        [expr, function, count] => (expr, function, Some(count)),
        _ => {
            if args.len() == 1 {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
            return Ok(session.invalid_call_with_exact_arg_count(symbol, args, 2));
        }
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, symbol, args));
    };
    if count.is_none() && items.is_empty() {
        return Ok(invalid_empty_collection_call(session, symbol, args));
    }
    let count = match count {
        Some(count) => Some(match expr_as_non_negative_usize(count) {
            Some(count) => count,
            None => return Ok(Expr::call(head.clone(), args.to_vec())),
        }),
        None => None,
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: match count {
                Some(count) => DecoratedOperation::ExtremeByCount {
                    largest: select_max,
                    count,
                },
                None => DecoratedOperation::MaxBy { select_max },
            },
        },
        0,
        Vec::new(),
    )
}

fn eval_extremal_elements_by(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    select_max: bool,
) -> Result<Expr, EvalError> {
    let (expr, function) = match args {
        [function] => return Ok(extremal_elements_by_operator_form(symbol, function)),
        [expr, function] => (expr, function),
        _ => return Ok(session.invalid_call_with_arg_range(symbol, args, 1, 2)),
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, symbol, args));
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: DecoratedOperation::ExtremalElementsBy { select_max },
        },
        0,
        Vec::new(),
    )
}

fn eval_take_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    largest: bool,
) -> Result<Expr, EvalError> {
    let [expr, function, count] = args else {
        if args.len() == 2 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_exact_arg_count(symbol, args, 3));
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, symbol, args));
    };
    let Some(count) = expr_as_non_negative_usize(count) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: DecoratedOperation::TakeBy { largest, count },
        },
        0,
        Vec::new(),
    )
}

fn resume_decorate_by_from_index(
    session: &mut Session,
    state: DecorateByState,
    mut index: usize,
    mut decorated: Vec<KeyedEntry>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = state.items.get(index).cloned() else {
            return finish_decorated(state, decorated);
        };

        let key = match session.evaluate(Expr::call(state.function.clone(), vec![item.clone()])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedDecorateByState::freeze(state);
                                        let decorated = PersistentStructureSequence::freeze(decorated);
                ResumeDecorateByFromIndexSuspensionFrame { state, index, decorated, item }
                },
                );
            }
            Err(error) => return Err(error),
        };

        decorated.push(KeyedEntry { item, key, index });
        index += 1;
    }
}

fn resume_suspended_decorate_by_from_index(
    session: &mut Session,
    state: SuspendedDecorateByState,
    mut index: usize,
    mut decorated: PersistentStructureSequence<KeyedEntry>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = state.items.get(index).cloned() else {
            return finish_decorated_parts(state.expr_head, state.operation, decorated.to_vec());
        };
        let key = match session.evaluate(Expr::call(state.function.clone(), vec![item.clone()])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedDecorateByFromIndexSuspensionFrame { state, index, decorated, item },
                );
            }
            Err(error) => return Err(error),
        };
        decorated = decorated.appended(KeyedEntry { item, key, index });
        index += 1;
    }
}

fn resume_suspended_decorate_by_with_key(
    session: &mut Session,
    state: SuspendedDecorateByState,
    index: usize,
    decorated: PersistentStructureSequence<KeyedEntry>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    resume_suspended_decorate_by_from_index(
        session,
        state,
        index + 1,
        decorated.appended(KeyedEntry { item, key, index }),
    )
}

fn finish_decorated(state: DecorateByState, decorated: Vec<KeyedEntry>) -> Result<Expr, EvalError> {
    finish_decorated_parts(state.expr_head, state.operation, decorated)
}

fn finish_decorated_parts(
    expr_head: Expr,
    operation: DecoratedOperation,
    decorated: Vec<KeyedEntry>,
) -> Result<Expr, EvalError> {
    match operation {
        DecoratedOperation::MaxBy { select_max } => Ok(select_extreme_by(decorated, select_max)
            .unwrap_or_else(|| Expr::call(expr_head, Vec::new()))),
        DecoratedOperation::ExtremalElementsBy { select_max } => Ok(Expr::call(
            expr_head,
            select_extremal_elements_by(decorated, select_max),
        )),
        DecoratedOperation::ExtremeByCount { largest, count } => {
            Ok(Expr::call(expr_head, take_by(decorated, largest, count)))
        }
        DecoratedOperation::TakeBy { largest, count } => {
            Ok(Expr::call(expr_head, take_by(decorated, largest, count)))
        }
        DecoratedOperation::ReverseSortBy => {
            let mut sorted = decorated;
            sorted.sort_by(|lhs, rhs| {
                compare_structural_order(&lhs.key, &rhs.key).then(lhs.index.cmp(&rhs.index))
            });
            sorted.reverse();
            Ok(Expr::call(
                expr_head,
                sorted.into_iter().map(|entry| entry.item).collect(),
            ))
        }
        DecoratedOperation::GatherBy => Ok(list_expr(gather_by_key(decorated))),
    }
}

fn select_extreme_by(decorated: Vec<KeyedEntry>, select_max: bool) -> Option<Expr> {
    let mut entries = decorated.into_iter();
    let mut best = entries.next()?;

    for entry in entries {
        let ordering = compare_structural_order(&entry.key, &best.key);
        if (select_max && ordering == Ordering::Greater)
            || (!select_max && ordering == Ordering::Less)
        {
            best = entry;
        }
    }

    Some(best.item)
}

fn select_extremal_elements_by(decorated: Vec<KeyedEntry>, select_max: bool) -> Vec<Expr> {
    let mut entries = decorated.into_iter();
    let Some(first) = entries.next() else {
        return Vec::new();
    };
    let mut best_key = first.key;
    let mut selected = vec![first.item];

    for entry in entries {
        let ordering = compare_structural_order(&entry.key, &best_key);
        if (select_max && ordering == Ordering::Greater)
            || (!select_max && ordering == Ordering::Less)
        {
            best_key = entry.key;
            selected.clear();
            selected.push(entry.item);
        } else if ordering == Ordering::Equal {
            selected.push(entry.item);
        }
    }

    selected
}

fn extremal_elements_by_operator_form(symbol: &str, function: &Expr) -> Expr {
    let arg = Expr::symbol("rum$extremalByArg");
    Expr::call(
        system_expr("Function"),
        vec![
            arg.clone(),
            Expr::call(system_expr(symbol), vec![arg, function.clone()]),
        ],
    )
}

fn take_by(mut decorated: Vec<KeyedEntry>, largest: bool, count: usize) -> Vec<Expr> {
    decorated.sort_by(|lhs, rhs| {
        let key_order = if largest {
            compare_structural_order(&rhs.key, &lhs.key)
        } else {
            compare_structural_order(&lhs.key, &rhs.key)
        };
        key_order.then(lhs.index.cmp(&rhs.index))
    });
    decorated.truncate(count);
    decorated.into_iter().map(|entry| entry.item).collect()
}

fn gather_by_key(decorated: Vec<KeyedEntry>) -> Vec<Expr> {
    StableGroups::collect(decorated.into_iter().map(|entry| (entry.key, entry.item)))
        .into_entries()
        .into_iter()
        .map(|(_, group)| list_expr(group))
        .collect()
}

fn normal_call_parts(expr: &Expr) -> Option<(Expr, Vec<Expr>)> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    Some((head.as_ref().clone(), args.to_vec()))
}

fn invalid_collection_normal_call(session: &mut Session, symbol: &str, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    )
}

fn invalid_empty_collection_call(session: &mut Session, symbol: &str, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "empty",
        format!("{symbol} cannot select from an empty list."),
    )
}

fn invalid_riffle_position_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Riffle",
        args,
        "seqs",
        format!(
            "Position 3 of {} must be a positive machine-sized integer or {{imin, imax, di}} with nonzero integer step.",
            Expr::call(system_expr("Riffle"), args.to_vec())
        ),
    )
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

#[derive(Clone, Copy)]
enum RifflePositions {
    Every(usize),
    Range {
        start: isize,
        end: isize,
        step: isize,
    },
}

fn parse_riffle_positions(expr: &Expr) -> Option<RifflePositions> {
    if let Some(interval) = expr_as_positive_usize(expr) {
        return Some(RifflePositions::Every(interval));
    }

    let [start, end, step] = list_items(expr)? else {
        return None;
    };
    let start = expr_as_signed_isize(start)?;
    let end = expr_as_signed_isize(end)?;
    let step = expr_as_signed_isize(step)?;
    (step != 0).then_some(RifflePositions::Range { start, end, step })
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    expr_as_non_negative_usize(expr).filter(|value| *value > 0)
}

fn riffle_insertion_positions(positions: &RifflePositions, item_count: usize) -> Vec<bool> {
    let mut insert = vec![false; item_count + 1];
    match *positions {
        RifflePositions::Every(interval) => {
            let mut position = interval;
            while position < item_count {
                insert[position] = true;
                position = position.saturating_add(interval);
            }
        }
        RifflePositions::Range { start, end, step } => {
            for position in 1..item_count {
                let positive = position as isize;
                let negative = positive - item_count as isize - 1;
                if signed_position_in_range(positive, start, end, step)
                    || signed_position_in_range(negative, start, end, step)
                {
                    insert[position] = true;
                }
            }
        }
    }
    insert
}

fn signed_position_in_range(position: isize, start: isize, end: isize, step: isize) -> bool {
    let position = position as i128;
    let start = start as i128;
    let end = end as i128;
    let step = step as i128;
    if step > 0 {
        position >= start && position <= end && (position - start) % step == 0
    } else {
        let step = -step;
        position <= start && position >= end && (start - position) % step == 0
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<DecorateByState>();
    assert_traces::<SuspendedDecorateByState>();
    assert_traces::<DecoratedOperation>();
    assert_traces::<GatherByFunctionListState>();
    assert_traces::<KeyedEntry>();
    assert_traces::<ReverseSortCompletion>();
    assert_traces::<TallyTestState>();
};
