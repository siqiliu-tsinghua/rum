mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
enum SequenceLikeExpr {
    Call { head: Expr, items: Vec<Expr> },
    Association(Vec<(Expr, Expr)>),
}

#[derive(Clone, FrameRootTrace)]
enum SuspendedSequenceLikeExpr {
    Call {
        head: Expr,
        items: FrozenFunctionalSequence<Expr>,
    },
    Association(FrozenFunctionalSequence<(Expr, Expr)>),
}

impl SuspendedSequenceLikeExpr {
    fn freeze(sequence: SequenceLikeExpr) -> Self {
        match sequence {
            SequenceLikeExpr::Call { head, items } => Self::Call {
                head,
                items: FrozenFunctionalSequence::freeze(items),
            },
            SequenceLikeExpr::Association(entries) => {
                Self::Association(FrozenFunctionalSequence::freeze(entries))
            }
        }
    }
}

#[derive(Clone, Copy, FrameRootTrace)]
enum WhileSequenceMode {
    Take,
    Length,
}

#[derive(Clone, FrameRootTrace)]
struct BlockMapEvalState {
    operator: Expr,
    block_head: Expr,
    items: Vec<Expr>,
    size: usize,
    offset: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedBlockMapEvalState {
    operator: Expr,
    block_head: Expr,
    items: FrozenFunctionalSequence<Expr>,
    size: usize,
    offset: usize,
}

impl SuspendedBlockMapEvalState {
    fn freeze(state: BlockMapEvalState) -> Self {
        Self {
            operator: state.operator,
            block_head: state.block_head,
            items: FrozenFunctionalSequence::freeze(state.items),
            size: state.size,
            offset: state.offset,
        }
    }
}

pub(crate) fn eval_block_map(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    match args_slice {
        [operator, size] => Ok(unary_operator_form_with_tail(
            "BlockMap",
            operator.clone(),
            size.clone(),
        )),
        [operator, size, offset]
            if expr_as_positive_usize(size).is_some()
                && expr_as_positive_usize(offset).is_some() =>
        {
            Ok(unary_operator_form_with_tails(
                "BlockMap",
                operator.clone(),
                vec![size.clone(), offset.clone()],
            ))
        }
        [operator, expr, size_expr] => {
            let Some(size) = expr_as_positive_usize(size_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "BlockMap",
                    args,
                    "expected a positive integer block size as the third argument",
                ));
            };

            apply_block_map(session, head, operator, expr, size, size, args)
        }
        [operator, expr, size_expr, offset_expr] => {
            let Some(size) = expr_as_positive_usize(size_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "BlockMap",
                    args,
                    "expected a positive integer block size as the third argument",
                ));
            };
            let Some(offset) = expr_as_positive_usize(offset_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "BlockMap",
                    args,
                    "expected a positive integer offset as the fourth argument",
                ));
            };

            apply_block_map(session, head, operator, expr, size, offset, args)
        }
        _ => Ok(session.invalid_call_with_arg_range("BlockMap", args_slice, 2, 4)),
    }
}

pub(crate) fn eval_accumulate(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count("Accumulate", args, 1));
    }

    let Some(sequence) = sequence_like_from_expr(&args[0]) else {
        return Ok(Expr::call(system_expr("Accumulate"), args.to_vec()));
    };

    accumulate_sequence(session, sequence)
}

pub(crate) fn eval_take_while(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [criterion] => Ok(unary_operator_form_slot_first("TakeWhile", criterion)),
        [expr, criterion] => {
            let Some(sequence) = sequence_like_from_expr(expr) else {
                return Ok(Expr::call(system_expr("TakeWhile"), args.to_vec()));
            };

            take_while_sequence(session, sequence, criterion)
        }
        _ => Ok(session.invalid_call_with_arg_range("TakeWhile", args, 1, 2)),
    }
}

pub(crate) fn eval_length_while(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [criterion] => Ok(unary_operator_form_slot_first("LengthWhile", criterion)),
        [expr, criterion] => {
            let Some(sequence) = sequence_like_from_expr(expr) else {
                return Ok(Expr::call(system_expr("LengthWhile"), args.to_vec()));
            };

            length_while_sequence(session, sequence, criterion)
        }
        _ => Ok(session.invalid_call_with_arg_range("LengthWhile", args, 1, 2)),
    }
}

fn take_while_sequence(
    session: &mut Session,
    sequence: SequenceLikeExpr,
    criterion: &Expr,
) -> Result<Expr, EvalError> {
    resume_while_sequence_from_index(
        session,
        sequence,
        criterion.clone(),
        0,
        WhileSequenceMode::Take,
    )
}

fn length_while_sequence(
    session: &mut Session,
    sequence: SequenceLikeExpr,
    criterion: &Expr,
) -> Result<Expr, EvalError> {
    resume_while_sequence_from_index(
        session,
        sequence,
        criterion.clone(),
        0,
        WhileSequenceMode::Length,
    )
}

fn resume_while_sequence_from_index(
    session: &mut Session,
    sequence: SequenceLikeExpr,
    criterion: Expr,
    mut index: usize,
    mode: WhileSequenceMode,
) -> Result<Expr, EvalError> {
    loop {
        if index >= sequence.len() {
            return Ok(finish_while_sequence(sequence, index, mode));
        }

        let Some(item) = sequence.value_at(index) else {
            return Ok(finish_while_sequence(sequence, index, mode));
        };
        let value = match session.evaluate(Expr::call(criterion.clone(), vec![item])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let sequence = SuspendedSequenceLikeExpr::freeze(sequence);
                ResumeWhileSequenceFromIndexSuspensionFrame { sequence, criterion, index, mode }
                },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&value) != Some(true) {
            return Ok(finish_while_sequence(sequence, index, mode));
        }
        index += 1;
    }
}

fn resume_suspended_while_sequence_from_index(
    session: &mut Session,
    sequence: SuspendedSequenceLikeExpr,
    criterion: Expr,
    mut index: usize,
    mode: WhileSequenceMode,
) -> Result<Expr, EvalError> {
    loop {
        if index >= sequence.len() {
            return Ok(finish_suspended_while_sequence(sequence, index, mode));
        }

        let Some(item) = sequence.value_at(index) else {
            return Ok(finish_suspended_while_sequence(sequence, index, mode));
        };
        let value = match session.evaluate(Expr::call(criterion.clone(), vec![item])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedWhileSequenceFromIndexSuspensionFrame { sequence, criterion, index, mode },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&value) != Some(true) {
            return Ok(finish_suspended_while_sequence(sequence, index, mode));
        }
        index += 1;
    }
}

fn resume_suspended_while_sequence_with_value(
    session: &mut Session,
    sequence: SuspendedSequenceLikeExpr,
    criterion: Expr,
    index: usize,
    mode: WhileSequenceMode,
    value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&value) == Some(true) {
        resume_suspended_while_sequence_from_index(session, sequence, criterion, index + 1, mode)
    } else {
        Ok(finish_suspended_while_sequence(sequence, index, mode))
    }
}

fn resume_accumulate_from_index(
    session: &mut Session,
    sequence: SequenceLikeExpr,
    mut index: usize,
    mut current: Expr,
    mut accumulated: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= sequence.len() {
            return Ok(sequence.render_with_values(accumulated));
        }

        let Some(item) = sequence.value_at(index) else {
            return Ok(sequence.render_with_values(accumulated));
        };
        let value = match session.evaluate(Expr::call(system_expr("Plus"), vec![current, item])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let sequence = SuspendedSequenceLikeExpr::freeze(sequence);
                                        let accumulated = PersistentFunctionalSequence::freeze(accumulated);
                ResumeAccumulateFromIndexSuspensionFrame { sequence, index, accumulated }
                },
                );
            }
            Err(error) => return Err(error),
        };

        accumulated.push(value.clone());
        current = value;
        index += 1;
    }
}

fn resume_suspended_accumulate_from_index(
    session: &mut Session,
    sequence: SuspendedSequenceLikeExpr,
    mut index: usize,
    mut current: Expr,
    mut accumulated: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= sequence.len() {
            return Ok(sequence.render_with_values(accumulated.to_vec()));
        }

        let Some(item) = sequence.value_at(index) else {
            return Ok(sequence.render_with_values(accumulated.to_vec()));
        };
        let value = match session.evaluate(Expr::call(system_expr("Plus"), vec![current, item])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedAccumulateFromIndexSuspensionFrame { sequence, index, accumulated },
                );
            }
            Err(error) => return Err(error),
        };

        accumulated = accumulated.appended(value.clone());
        current = value;
        index += 1;
    }
}

fn resume_suspended_accumulate_with_value(
    session: &mut Session,
    sequence: SuspendedSequenceLikeExpr,
    index: usize,
    accumulated: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let accumulated = accumulated.appended(value.clone());
    resume_suspended_accumulate_from_index(session, sequence, index + 1, value, accumulated)
}

fn resume_block_map_from_index(
    session: &mut Session,
    state: BlockMapEvalState,
    mut index: usize,
    mut results: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index + state.size > state.items.len() {
            return Ok(Expr::call(system_expr("List"), results));
        }

        let block = Expr::call(
            state.block_head.clone(),
            state.items[index..index + state.size].to_vec(),
        );
        let value = match session.evaluate(Expr::call(state.operator.clone(), vec![block])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedBlockMapEvalState::freeze(state);
                                        let results = PersistentFunctionalSequence::freeze(results);
                ResumeBlockMapFromIndexSuspensionFrame { state, index, results }
                },
                );
            }
            Err(error) => return Err(error),
        };

        results.push(value);
        index += state.offset;
    }
}

fn resume_suspended_block_map_from_index(
    session: &mut Session,
    state: SuspendedBlockMapEvalState,
    mut index: usize,
    mut results: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index + state.size > state.items.len() {
            return Ok(Expr::call(system_expr("List"), results.to_vec()));
        }

        let block = Expr::call(
            state.block_head.clone(),
            state.items.as_slice()[index..index + state.size].to_vec(),
        );
        let value = match session.evaluate(Expr::call(state.operator.clone(), vec![block])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedBlockMapFromIndexSuspensionFrame { state, index, results },
                );
            }
            Err(error) => return Err(error),
        };

        results = results.appended(value);
        index += state.offset;
    }
}

fn resume_suspended_block_map_with_value(
    session: &mut Session,
    state: SuspendedBlockMapEvalState,
    index: usize,
    results: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let results = results.appended(value);
    let next_index = index + state.offset;
    resume_suspended_block_map_from_index(session, state, next_index, results)
}

impl SequenceLikeExpr {
    fn len(&self) -> usize {
        match self {
            Self::Call { items, .. } => items.len(),
            Self::Association(entries) => entries.len(),
        }
    }

    fn value_at(&self, index: usize) -> Option<Expr> {
        match self {
            Self::Call { items, .. } => items.get(index).cloned(),
            Self::Association(entries) => entries.get(index).map(|(_, value)| value.clone()),
        }
    }

    fn render_prefix(&self, count: usize) -> Expr {
        match self {
            Self::Call { head, items } => {
                Expr::call(head.clone(), items.iter().take(count).cloned().collect())
            }
            Self::Association(entries) => {
                render_association(entries.iter().take(count).cloned().collect())
            }
        }
    }

    fn render_with_values(&self, values: Vec<Expr>) -> Expr {
        match self {
            Self::Call { head, .. } => Expr::call(head.clone(), values),
            Self::Association(entries) => render_association(
                entries
                    .iter()
                    .map(|(key, _)| key.clone())
                    .zip(values)
                    .collect(),
            ),
        }
    }
}

impl SuspendedSequenceLikeExpr {
    fn len(&self) -> usize {
        match self {
            Self::Call { items, .. } => items.len(),
            Self::Association(entries) => entries.len(),
        }
    }

    fn value_at(&self, index: usize) -> Option<Expr> {
        match self {
            Self::Call { items, .. } => items.get(index).cloned(),
            Self::Association(entries) => entries.get(index).map(|(_, value)| value.clone()),
        }
    }

    fn render_prefix(&self, count: usize) -> Expr {
        match self {
            Self::Call { head, items } => Expr::call(
                head.clone(),
                items.as_slice().iter().take(count).cloned().collect(),
            ),
            Self::Association(entries) => {
                render_association(entries.as_slice().iter().take(count).cloned().collect())
            }
        }
    }

    fn render_with_values(&self, values: Vec<Expr>) -> Expr {
        match self {
            Self::Call { head, .. } => Expr::call(head.clone(), values),
            Self::Association(entries) => render_association(
                entries
                    .as_slice()
                    .iter()
                    .map(|(key, _)| key.clone())
                    .zip(values)
                    .collect(),
            ),
        }
    }
}

fn sequence_like_from_expr(expr: &Expr) -> Option<SequenceLikeExpr> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(SequenceLikeExpr::Association(entries));
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    Some(SequenceLikeExpr::Call {
        head: head.as_ref().clone(),
        items: args.to_vec(),
    })
}

fn accumulate_sequence(
    session: &mut Session,
    sequence: SequenceLikeExpr,
) -> Result<Expr, EvalError> {
    let Some(first) = sequence.value_at(0) else {
        return Ok(sequence.render_with_values(Vec::new()));
    };

    resume_accumulate_from_index(session, sequence, 1, first.clone(), vec![first])
}

fn apply_block_map(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    expr: &Expr,
    size: usize,
    offset: usize,
    raw_args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: block_head,
        args: items,
        ..
    } = expr
    else {
        return Ok(Expr::call_with_shared_args(head.clone(), raw_args.clone()));
    };

    resume_block_map_from_index(
        session,
        BlockMapEvalState {
            operator: operator.clone(),
            block_head: block_head.as_ref().clone(),
            items: items.to_vec(),
            size,
            offset,
        },
        0,
        Vec::new(),
    )
}

fn finish_suspended_while_sequence(
    sequence: SuspendedSequenceLikeExpr,
    count: usize,
    mode: WhileSequenceMode,
) -> Expr {
    match mode {
        WhileSequenceMode::Take => sequence.render_prefix(count),
        WhileSequenceMode::Length => Expr::integer(BigInt::from(count)),
    }
}

fn finish_while_sequence(
    sequence: SequenceLikeExpr,
    count: usize,
    mode: WhileSequenceMode,
) -> Expr {
    match mode {
        WhileSequenceMode::Take => sequence.render_prefix(count),
        WhileSequenceMode::Length => Expr::integer(BigInt::from(count)),
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<BlockMapEvalState>();
    assert_traces::<SequenceLikeExpr>();
    assert_traces::<SuspendedBlockMapEvalState>();
    assert_traces::<SuspendedSequenceLikeExpr>();
};
