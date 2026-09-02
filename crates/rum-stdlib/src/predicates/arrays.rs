use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Debug, PartialEq, Eq, FrameRootTrace)]
struct ArrayLeafTestState {
    leaves: Vec<Expr>,
    test: Expr,
    index: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedArrayLeafTestState {
    leaves: FrozenPredicateSequence<Expr>,
    test: Expr,
    index: usize,
}

impl SuspendedArrayLeafTestState {
    fn freeze(state: ArrayLeafTestState) -> Self {
        Self {
            leaves: FrozenPredicateSequence::freeze(state.leaves),
            test: state.test,
            index: state.index,
        }
    }
}

pub(crate) fn eval_vector_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr] => Ok(predicate_bool_expr(
            array_shape(expr).is_some_and(|shape| shape.len() == 1),
        )),
        [expr, test] => {
            let Some(shape) = array_shape(expr) else {
                return Ok(predicate_bool_expr(false));
            };
            if shape.len() != 1 {
                return Ok(predicate_bool_expr(false));
            }

            resume_array_leaf_test_from_index(session, ArrayLeafTestState::new(expr, test))
        }
        _ => Ok(session.invalid_call_with_arg_range("VectorQ", args, 1, 2)),
    }
}

pub(crate) fn eval_matrix_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr] => Ok(predicate_bool_expr(
            array_shape(expr).is_some_and(|shape| shape.len() == 2),
        )),
        [expr, test] => {
            let Some(shape) = array_shape(expr) else {
                return Ok(predicate_bool_expr(false));
            };
            if shape.len() != 2 {
                return Ok(predicate_bool_expr(false));
            }

            resume_array_leaf_test_from_index(session, ArrayLeafTestState::new(expr, test))
        }
        _ => Ok(session.invalid_call_with_arg_range("MatrixQ", args, 1, 2)),
    }
}

pub(crate) fn eval_array_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=3).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("ArrayQ", args, 1, 3));
    }

    let Some(shape) = array_shape(&args[0]) else {
        return Ok(predicate_bool_expr(false));
    };

    if args.len() == 1 {
        return Ok(predicate_bool_expr(true));
    }

    let leaf_test = args
        .get(2)
        .map(|test| ArrayLeafTestState::new(&args[0], test));
    let rank_value = match rank_pattern_match_result(session, shape.len(), args[1].clone()) {
        Ok(value) => value,
        Err(EvalError::InputRequired { request }) => {
            return rum_runtime::resume_or_suspend!(
                session,
                Err(EvalError::InputRequired { request }),
                ok(value) => Ok(value),
                suspend => {
            let leaf_test = leaf_test.map(SuspendedArrayLeafTestState::freeze);
            EvalArrayQSuspensionFrame { leaf_test }
            },
            );
        }
        Err(error) => return Err(error),
    };

    finish_array_q_with_rank_value(session, leaf_test, rank_value)
}

fn finish_suspended_array_q_with_rank_value(
    session: &mut Session,
    leaf_test: Option<SuspendedArrayLeafTestState>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&value) != Some(true) {
        return Ok(predicate_bool_expr(false));
    }
    match leaf_test {
        Some(state) => resume_suspended_array_leaf_test_from_index(session, state),
        None => Ok(predicate_bool_expr(true)),
    }
}

impl ArrayLeafTestState {
    fn new(expr: &Expr, test: &Expr) -> Self {
        let mut leaves = Vec::new();
        collect_array_leaves(expr, &mut leaves);
        Self {
            leaves,
            test: test.clone(),
            index: 0,
        }
    }
}

fn rank_pattern_match_result(
    session: &mut Session,
    rank: usize,
    pattern: Expr,
) -> Result<Expr, EvalError> {
    session.eval_match_q_data_builtin(&[Expr::integer(BigInt::from(rank)), pattern])
}

fn finish_array_q_with_rank_value(
    session: &mut Session,
    leaf_test: Option<ArrayLeafTestState>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&value) != Some(true) {
        return Ok(predicate_bool_expr(false));
    }

    match leaf_test {
        Some(state) => resume_array_leaf_test_from_index(session, state),
        None => Ok(predicate_bool_expr(true)),
    }
}

fn resume_array_leaf_test_from_index(
    session: &mut Session,
    mut state: ArrayLeafTestState,
) -> Result<Expr, EvalError> {
    while let Some(leaf) = state.leaves.get(state.index).cloned() {
        if state.index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }

        let call = Expr::call(state.test.clone(), vec![leaf]);
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedArrayLeafTestState::freeze(state);
                ResumeArrayLeafTestFromIndexSuspensionFrame { state }
                },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&value) != Some(true) {
            return Ok(predicate_bool_expr(false));
        }
        state.index += 1;
    }

    Ok(predicate_bool_expr(true))
}

fn resume_suspended_array_leaf_test_from_index(
    session: &mut Session,
    mut state: SuspendedArrayLeafTestState,
) -> Result<Expr, EvalError> {
    while let Some(leaf) = state.leaves.get(state.index).cloned() {
        if state.index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }
        let call = Expr::call(state.test.clone(), vec![leaf]);
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedArrayLeafTestFromIndexSuspensionFrame { state },
                );
            }
            Err(error) => return Err(error),
        };
        if truthiness_symbol(&value) != Some(true) {
            return Ok(predicate_bool_expr(false));
        }
        state.index += 1;
    }
    Ok(predicate_bool_expr(true))
}

fn resume_suspended_array_leaf_test_with_value(
    session: &mut Session,
    mut state: SuspendedArrayLeafTestState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&value) != Some(true) {
        return Ok(predicate_bool_expr(false));
    }
    state.index += 1;
    resume_suspended_array_leaf_test_from_index(session, state)
}

fn collect_array_leaves(expr: &Expr, leaves: &mut Vec<Expr>) {
    match list_items(expr) {
        Some(items) => {
            for item in items {
                collect_array_leaves(item, leaves);
            }
        }
        None => leaves.push(expr.clone()),
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ArrayLeafTestState>();
    assert_traces::<SuspendedArrayLeafTestState>();
};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct EvalArrayQSuspensionFrame {
    leaf_test: std::option::Option<crate::predicates::arrays::SuspendedArrayLeafTestState>,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalArrayQSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut leaf_test = self.leaf_test.clone();
        let resumed = match outcome {
            Ok(value) => (|| finish_suspended_array_q_with_rank_value(session, leaf_test, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeArrayLeafTestFromIndexSuspensionFrame {
    state: crate::predicates::arrays::SuspendedArrayLeafTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeArrayLeafTestFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_array_leaf_test_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedArrayLeafTestFromIndexSuspensionFrame {
    state: crate::predicates::arrays::SuspendedArrayLeafTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedArrayLeafTestFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_array_leaf_test_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
