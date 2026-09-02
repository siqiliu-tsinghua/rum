use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct FoldPairEvalState {
    operator: Expr,
    items: Vec<Expr>,
    projector: Option<Expr>,
    fallback: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedFoldPairEvalState {
    operator: Expr,
    items: FrozenFunctionalSequence<Expr>,
    projector: Option<Expr>,
    fallback: Expr,
}

impl SuspendedFoldPairEvalState {
    fn freeze(state: FoldPairEvalState) -> Self {
        Self {
            operator: state.operator,
            items: FrozenFunctionalSequence::freeze(state.items),
            projector: state.projector,
            fallback: state.fallback,
        }
    }
}

fn resume_fold_pair_from_index(
    session: &mut Session,
    state: FoldPairEvalState,
    mut index: usize,
    mut current_state: Expr,
    mut last_output: Option<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.items.len() {
            return Ok(finish_fold_pair_result(state, last_output));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![current_state, state.items[index].clone()],
        );
        let pair = match session.evaluate(call) {
            Ok(pair) => pair,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedFoldPairEvalState::freeze(state);
                ResumeFoldPairFromIndexSuspensionFrame1 { state, index }
                },
                );
            }
            Err(error) => return Err(error),
        };

        let Some((x_value, y_value)) = parse_pair_result(&pair) else {
            return Ok(fold_pair_unevaluated(state));
        };

        let output = if let Some(projector) = state.projector.as_ref() {
            let call = Expr::call(
                projector.clone(),
                vec![Expr::call(
                    system_expr("List"),
                    vec![x_value.clone(), y_value.clone()],
                )],
            );
            match session.evaluate(call) {
                Ok(output) => output,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let state = SuspendedFoldPairEvalState::freeze(state);
                    ResumeFoldPairFromIndexSuspensionFrame2 { state, index, y_value }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            x_value
        };

        current_state = y_value;
        last_output = Some(output);
        index += 1;
    }
}

fn resume_suspended_fold_pair_from_index(
    session: &mut Session,
    state: SuspendedFoldPairEvalState,
    mut index: usize,
    mut current_state: Expr,
    mut last_output: Option<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.items.len() {
            return Ok(finish_suspended_fold_pair_result(state, last_output));
        }

        let call = Expr::call(
            state.operator.clone(),
            vec![
                current_state,
                state
                    .items
                    .get(index)
                    .expect("fold-pair item should exist")
                    .clone(),
            ],
        );
        let pair = match session.evaluate(call) {
            Ok(pair) => pair,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedFoldPairFromIndexSuspensionFrame1 { state, index },
                );
            }
            Err(error) => return Err(error),
        };

        let Some((x_value, y_value)) = parse_pair_result(&pair) else {
            return Ok(fold_pair_unevaluated_suspended(state));
        };

        let output = if let Some(projector) = state.projector.as_ref() {
            let call = Expr::call(
                projector.clone(),
                vec![Expr::call(
                    system_expr("List"),
                    vec![x_value.clone(), y_value.clone()],
                )],
            );
            match session.evaluate(call) {
                Ok(output) => output,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeSuspendedFoldPairFromIndexSuspensionFrame2 { state, index, y_value },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            x_value
        };

        current_state = y_value;
        last_output = Some(output);
        index += 1;
    }
}

fn resume_suspended_fold_pair_with_pair_result(
    session: &mut Session,
    state: SuspendedFoldPairEvalState,
    index: usize,
    pair: Expr,
) -> Result<Expr, EvalError> {
    let Some((x_value, y_value)) = parse_pair_result(&pair) else {
        return Ok(fold_pair_unevaluated_suspended(state));
    };

    let Some(projector) = state.projector.as_ref() else {
        return resume_suspended_fold_pair_with_output(session, state, index, y_value, x_value);
    };

    let call = Expr::call(
        projector.clone(),
        vec![Expr::call(
            system_expr("List"),
            vec![x_value.clone(), y_value.clone()],
        )],
    );
    let output = match session.evaluate(call) {
        Ok(output) => output,
        Err(EvalError::InputRequired { request }) => {
            return rum_runtime::resume_or_suspend!(
                session,
                Err(EvalError::InputRequired { request }),
                ok(value) => Ok(value),
                suspend => ResumeSuspendedFoldPairWithPairResultSuspensionFrame { state, index, y_value },
            );
        }
        Err(error) => return Err(error),
    };

    resume_suspended_fold_pair_with_output(session, state, index, y_value, output)
}

fn resume_suspended_fold_pair_with_output(
    session: &mut Session,
    state: SuspendedFoldPairEvalState,
    index: usize,
    next_state: Expr,
    output: Expr,
) -> Result<Expr, EvalError> {
    resume_suspended_fold_pair_from_index(session, state, index + 1, next_state, Some(output))
}

pub(super) fn apply_fold_pair(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    initial: Option<Expr>,
    list_expr: &Expr,
    projector: Option<Expr>,
    raw_args: &rum_core::SharedArgs,
) -> Result<Expr, EvalError> {
    let Some(items) = iterable_items(list_expr) else {
        return Ok(Expr::call_with_shared_args(head.clone(), raw_args.clone()));
    };

    let (state, start_index) = if let Some(initial) = initial {
        (initial, 0usize)
    } else {
        let Some((first, rest)) = items.split_first() else {
            return Ok(Expr::call_with_shared_args(head.clone(), raw_args.clone()));
        };
        (first.clone(), items.len() - rest.len())
    };

    resume_fold_pair_from_index(
        session,
        FoldPairEvalState {
            operator: operator.clone(),
            items,
            projector,
            fallback: Expr::call_with_shared_args(head.clone(), raw_args.clone()),
        },
        start_index,
        state,
        None,
    )
}

fn parse_pair_result(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { args, .. } = expr else {
        return None;
    };

    match args.as_slice() {
        [first, second] => Some((first.clone(), second.clone())),
        _ => None,
    }
}

fn finish_fold_pair_result(state: FoldPairEvalState, last_output: Option<Expr>) -> Expr {
    last_output.unwrap_or(state.fallback)
}

fn fold_pair_unevaluated(state: FoldPairEvalState) -> Expr {
    state.fallback
}

fn finish_suspended_fold_pair_result(
    state: SuspendedFoldPairEvalState,
    last_output: Option<Expr>,
) -> Expr {
    last_output.unwrap_or(state.fallback)
}

fn fold_pair_unevaluated_suspended(state: SuspendedFoldPairEvalState) -> Expr {
    state.fallback
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FoldPairEvalState>();
    assert_traces::<SuspendedFoldPairEvalState>();
};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeFoldPairFromIndexSuspensionFrame1 {
    state: crate::functional_ops::sequence::fold::pair::SuspendedFoldPairEvalState,
    index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeFoldPairFromIndexSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        let resumed = match outcome {
            Ok(pair) => {
                (|| resume_suspended_fold_pair_with_pair_result(session, state, index, pair))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeFoldPairFromIndexSuspensionFrame2 {
    state: crate::functional_ops::sequence::fold::pair::SuspendedFoldPairEvalState,
    index: usize,
    y_value: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeFoldPairFromIndexSuspensionFrame2 {
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
        let mut y_value = self.y_value.clone();
        let resumed = match outcome {
            Ok(output) => {
                (|| resume_suspended_fold_pair_with_output(session, state, index, y_value, output))(
                )
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedFoldPairFromIndexSuspensionFrame1 {
    state: crate::functional_ops::sequence::fold::pair::SuspendedFoldPairEvalState,
    index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedFoldPairFromIndexSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        let resumed = match outcome {
            Ok(pair) => {
                (|| resume_suspended_fold_pair_with_pair_result(session, state, index, pair))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedFoldPairFromIndexSuspensionFrame2 {
    state: crate::functional_ops::sequence::fold::pair::SuspendedFoldPairEvalState,
    index: usize,
    y_value: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedFoldPairFromIndexSuspensionFrame2 {
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
        let mut y_value = self.y_value.clone();
        let resumed = match outcome {
            Ok(output) => {
                (|| resume_suspended_fold_pair_with_output(session, state, index, y_value, output))(
                )
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedFoldPairWithPairResultSuspensionFrame {
    state: crate::functional_ops::sequence::fold::pair::SuspendedFoldPairEvalState,
    index: usize,
    y_value: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedFoldPairWithPairResultSuspensionFrame
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
        let mut y_value = self.y_value.clone();
        let resumed = match outcome {
            Ok(output) => {
                (|| resume_suspended_fold_pair_with_output(session, state, index, y_value, output))(
                )
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
