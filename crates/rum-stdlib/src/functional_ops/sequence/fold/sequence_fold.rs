use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct SequenceFoldEvalState {
    operator: Expr,
    items: Vec<Expr>,
    seed_len: usize,
    consumed_per_step: usize,
    collect_all: bool,
    fallback: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedSequenceFoldEvalState {
    operator: Expr,
    items: FrozenFunctionalSequence<Expr>,
    seed_len: usize,
    consumed_per_step: usize,
    collect_all: bool,
    fallback: Expr,
}

impl SuspendedSequenceFoldEvalState {
    fn freeze(state: SequenceFoldEvalState) -> Self {
        Self {
            operator: state.operator,
            items: FrozenFunctionalSequence::freeze(state.items),
            seed_len: state.seed_len,
            consumed_per_step: state.consumed_per_step,
            collect_all: state.collect_all,
            fallback: state.fallback,
        }
    }
}

fn resume_sequence_fold_from_offset(
    session: &mut Session,
    state: SequenceFoldEvalState,
    mut offset: usize,
    mut history: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if offset + state.consumed_per_step > state.items.len() {
            return Ok(finish_sequence_fold_result(state, history));
        }

        let mut call_args = if state.seed_len == 0 {
            Vec::new()
        } else {
            history
                .iter()
                .rev()
                .take(state.seed_len)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
        };
        call_args.extend(
            state.items[offset..offset + state.consumed_per_step]
                .iter()
                .cloned(),
        );

        let value = match session.evaluate(Expr::call(state.operator.clone(), call_args)) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedSequenceFoldEvalState::freeze(state);
                                        let history = PersistentFunctionalSequence::freeze(history);
                ResumeSequenceFoldFromOffsetSuspensionFrame { state, offset, history }
                },
                );
            }
            Err(error) => return Err(error),
        };

        history.push(value);
        offset += state.consumed_per_step;
    }
}

fn resume_suspended_sequence_fold_from_offset(
    session: &mut Session,
    state: SuspendedSequenceFoldEvalState,
    mut offset: usize,
    mut history: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if offset + state.consumed_per_step > state.items.len() {
            return Ok(finish_suspended_sequence_fold_result(state, history));
        }

        let mut call_args = if state.seed_len == 0 {
            Vec::new()
        } else {
            history.recent(state.seed_len)
        };
        call_args.extend(
            state.items.as_slice()[offset..offset + state.consumed_per_step]
                .iter()
                .cloned(),
        );

        let value = match session.evaluate(Expr::call(state.operator.clone(), call_args)) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedSequenceFoldFromOffsetSuspensionFrame { state, offset, history },
                );
            }
            Err(error) => return Err(error),
        };

        history = history.appended(value);
        offset += state.consumed_per_step;
    }
}

fn resume_suspended_sequence_fold_with_value(
    session: &mut Session,
    state: SuspendedSequenceFoldEvalState,
    offset: usize,
    history: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let history = history.appended(value);
    let next_offset = offset + state.consumed_per_step;
    resume_suspended_sequence_fold_from_offset(session, state, next_offset, history)
}

pub(super) fn apply_sequence_fold(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    seeds_expr: &Expr,
    items_expr: &Expr,
    arity_expr: Option<&Expr>,
    collect_all: bool,
    raw_args: &rum_core::SharedArgs,
) -> Result<Expr, EvalError> {
    let Some(seeds) = iterable_items(seeds_expr) else {
        return Ok(Expr::call_with_shared_args(head.clone(), raw_args.clone()));
    };
    let Some(items) = iterable_items(items_expr) else {
        return Ok(Expr::call_with_shared_args(head.clone(), raw_args.clone()));
    };

    let seed_len = seeds.len();
    let arity = match arity_expr {
        Some(expr) => {
            let Some(arity) = expr_as_non_negative_usize(expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    if collect_all {
                        "SequenceFoldList"
                    } else {
                        "SequenceFold"
                    },
                    raw_args,
                    "expected a non-negative integer arity as the fourth argument",
                ));
            };
            arity
        }
        None => seed_len.saturating_add(1),
    };

    if arity <= seed_len {
        return Ok(session.invalid_call_with_arg_form(
            if collect_all {
                "SequenceFoldList"
            } else {
                "SequenceFold"
            },
            raw_args,
            "expected an arity larger than the seed length in the current first-pass subset",
        ));
    }

    resume_sequence_fold_from_offset(
        session,
        SequenceFoldEvalState {
            operator: operator.clone(),
            items,
            seed_len,
            consumed_per_step: arity - seed_len,
            collect_all,
            fallback: Expr::call_with_shared_args(head.clone(), raw_args.clone()),
        },
        0,
        seeds,
    )
}

fn finish_sequence_fold_result(state: SequenceFoldEvalState, history: Vec<Expr>) -> Expr {
    if state.collect_all {
        Expr::call(system_expr("List"), history)
    } else {
        history.last().cloned().unwrap_or(state.fallback)
    }
}

fn finish_suspended_sequence_fold_result(
    state: SuspendedSequenceFoldEvalState,
    history: PersistentFunctionalSequence<Expr>,
) -> Expr {
    if state.collect_all {
        Expr::call(system_expr("List"), history.to_vec())
    } else {
        history.last().cloned().unwrap_or(state.fallback)
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SequenceFoldEvalState>();
    assert_traces::<SuspendedSequenceFoldEvalState>();
};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSequenceFoldFromOffsetSuspensionFrame {
    state: crate::functional_ops::sequence::fold::sequence_fold::SuspendedSequenceFoldEvalState,
    offset: usize,
    history: crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSequenceFoldFromOffsetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut offset = self.offset.clone();
        #[allow(unused_mut)]
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_sequence_fold_with_value(session, state, offset, history, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedSequenceFoldFromOffsetSuspensionFrame {
    state: crate::functional_ops::sequence::fold::sequence_fold::SuspendedSequenceFoldEvalState,
    offset: usize,
    history: crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedSequenceFoldFromOffsetSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut offset = self.offset.clone();
        #[allow(unused_mut)]
        let mut history = self.history.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_sequence_fold_with_value(session, state, offset, history, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
