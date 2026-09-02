use rum_core::SharedArgs;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::cmp::Ordering;
use std::collections::HashSet;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum SelectPositionsLimit {
    All,
    Count(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct SelectPositionsState {
    items: Vec<Expr>,
    criterion: Expr,
    limit: SelectPositionsLimit,
    index: usize,
    positions: Vec<Expr>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedSelectPositionsState {
    items: FrozenStructureSequence<Expr>,
    criterion: Expr,
    limit: SelectPositionsLimit,
}

impl SelectPositionsLimit {
    fn reached(self, count: usize) -> bool {
        match self {
            Self::All => false,
            Self::Count(limit) => count >= limit,
        }
    }
}

pub(crate) fn eval_select_positions(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [list, criterion] => {
            let Some(items) = list_items(list) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            resume_select_positions_from_index(
                session,
                SelectPositionsState {
                    items: items.to_vec(),
                    criterion: criterion.clone(),
                    limit: SelectPositionsLimit::All,
                    index: 0,
                    positions: Vec::new(),
                },
            )
        }
        [list, criterion, limit] => {
            let Some(items) = list_items(list) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            let Some(limit) = expr_as_non_negative_usize(limit) else {
                return Ok(session.invalid_call_with_arg_form(
                    "SelectPositions",
                    args,
                    "expected a non-negative integer selection limit",
                ));
            };
            resume_select_positions_from_index(
                session,
                SelectPositionsState {
                    items: items.to_vec(),
                    criterion: criterion.clone(),
                    limit: SelectPositionsLimit::Count(limit),
                    index: 0,
                    positions: Vec::new(),
                },
            )
        }
        _ => Ok(session.invalid_call_with_arg_range("SelectPositions", args, 2, 3)),
    }
}

pub(crate) fn eval_duplicates(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [list] = args else {
        return Ok(session.invalid_call_with_arg_range("Duplicates", args, 1, 1));
    };
    let Some(items) = list_items(list) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    const INDEX_THRESHOLD: usize = 16;

    let mut remaining = items.iter();
    let mut seen = Vec::new();
    let mut duplicates = Vec::new();
    for item in remaining.by_ref() {
        if seen.iter().any(|existing| existing == item) {
            if !duplicates.iter().any(|existing| existing == item) {
                duplicates.push(item.clone());
            }
        } else {
            seen.push(item.clone());
            if seen.len() > INDEX_THRESHOLD {
                return Ok(Expr::call(
                    system_expr("List"),
                    collect_duplicates_indexed(seen, duplicates, remaining),
                ));
            }
        }
    }

    Ok(Expr::call(system_expr("List"), duplicates))
}

#[inline(never)]
fn collect_duplicates_indexed<'a>(
    seen: Vec<Expr>,
    mut duplicates: Vec<Expr>,
    remaining: impl Iterator<Item = &'a Expr>,
) -> Vec<Expr> {
    let mut seen = seen.into_iter().collect::<HashSet<_>>();
    let mut emitted = duplicates.iter().cloned().collect::<HashSet<_>>();
    for item in remaining {
        if !seen.insert(item.clone()) && emitted.insert(item.clone()) {
            duplicates.push(item.clone());
        }
    }
    duplicates
}

pub(crate) fn eval_binary_search(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [list, value] = args else {
        return Ok(session.invalid_call_with_arg_range("BinarySearch", args, 2, 2));
    };
    let Some(items) = list_items(list) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(binary_search_position(items, value)
        .map(|position| Expr::integer(BigInt::from(position)))
        .unwrap_or_else(missing_not_found_expr))
}

fn resume_select_positions_from_index(
    session: &mut Session,
    mut state: SelectPositionsState,
) -> Result<Expr, EvalError> {
    loop {
        if state.limit.reached(state.positions.len()) || state.index >= state.items.len() {
            return Ok(Expr::call(system_expr("List"), state.positions));
        }
        if state.index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }

        let item = state.items[state.index].clone();
        let value = match session.evaluate(Expr::call(state.criterion.clone(), vec![item])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let suspended = SuspendedSelectPositionsState {
                                            items: FrozenStructureSequence::freeze(state.items),
                                            criterion: state.criterion,
                                            limit: state.limit,
                                        };
                                        let positions = PersistentStructureSequence::freeze(state.positions);
                ResumeSelectPositionsFromIndexSuspensionFrame { suspended, state_index: state.index, positions }
                },
                );
            }
            Err(error) => return Err(error),
        };

        push_position_if_true(session, &mut state, value)?;
        state.index += 1;
    }
}

fn resume_suspended_select_positions_from_index(
    session: &mut Session,
    state: SuspendedSelectPositionsState,
    mut index: usize,
    mut positions: PersistentStructureSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if state.limit.reached(positions.len()) || index >= state.items.len() {
            return Ok(Expr::call(system_expr("List"), positions.to_vec()));
        }
        if index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }

        let item = state.items.get(index).expect("item should exist").clone();
        let value = match session.evaluate(Expr::call(state.criterion.clone(), vec![item])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedSelectPositionsFromIndexSuspensionFrame { state, index, positions },
                );
            }
            Err(error) => return Err(error),
        };

        positions = append_position_if_true(session, positions, index, value)?;
        index += 1;
    }
}

fn resume_suspended_select_positions_with_value(
    session: &mut Session,
    state: SuspendedSelectPositionsState,
    index: usize,
    positions: PersistentStructureSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let positions = append_position_if_true(session, positions, index, value)?;
    resume_suspended_select_positions_from_index(session, state, index + 1, positions)
}

fn append_position_if_true(
    session: &mut Session,
    positions: PersistentStructureSequence<Expr>,
    index: usize,
    value: Expr,
) -> Result<PersistentStructureSequence<Expr>, EvalError> {
    if truthiness_symbol(&value) != Some(true) {
        return Ok(positions);
    }
    let next_len = positions.len() + 1;
    session.enforce_generated_element_limit_for_count(next_len)?;
    Ok(positions.appended(position_path_expr(index + 1)))
}

fn push_position_if_true(
    session: &mut Session,
    state: &mut SelectPositionsState,
    value: Expr,
) -> Result<(), EvalError> {
    if truthiness_symbol(&value) == Some(true) {
        let next_len = state.positions.len() + 1;
        session.enforce_generated_element_limit_for_count(next_len)?;
        state.positions.push(position_path_expr(state.index + 1));
    }

    Ok(())
}

fn position_path_expr(one_based_index: usize) -> Expr {
    Expr::call(
        system_expr("List"),
        vec![Expr::integer(BigInt::from(one_based_index))],
    )
}

fn binary_search_position(items: &[Expr], value: &Expr) -> Option<usize> {
    let mut lower = 0;
    let mut upper = items.len();

    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        match compare_expr_canonical(&items[middle], value) {
            Ordering::Less => lower = middle + 1,
            Ordering::Equal | Ordering::Greater => upper = middle,
        }
    }

    (lower < items.len() && compare_expr_canonical(&items[lower], value) == Ordering::Equal)
        .then_some(lower + 1)
}

fn missing_not_found_expr() -> Expr {
    Expr::call(system_expr("Missing"), vec![Expr::string("NotFound")])
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SelectPositionsLimit>();
    assert_traces::<SelectPositionsState>();
    assert_traces::<SuspendedSelectPositionsState>();
};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSelectPositionsFromIndexSuspensionFrame {
    suspended: crate::structure_ops::list_lookup::SuspendedSelectPositionsState,
    state_index: usize,
    positions: crate::structure_ops::suspended_state::PersistentStructureSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSelectPositionsFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut suspended = self.suspended.clone();
        #[allow(unused_mut)]
        let mut state_index = self.state_index.clone();
        #[allow(unused_mut)]
        let mut positions = self.positions.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_select_positions_with_value(
                    session,
                    suspended,
                    state_index,
                    positions,
                    value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedSelectPositionsFromIndexSuspensionFrame {
    state: crate::structure_ops::list_lookup::SuspendedSelectPositionsState,
    index: usize,
    positions: crate::structure_ops::suspended_state::PersistentStructureSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedSelectPositionsFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut positions = self.positions.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_select_positions_with_value(
                    session, state, index, positions, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
