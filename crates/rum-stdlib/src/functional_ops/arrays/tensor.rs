mod resume_frames;
use resume_frames::*;

use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
pub(super) struct TensorCallbackEvalState {
    pub(super) operator: Expr,
    pub(super) output_head: Expr,
    pub(super) output_shape: Vec<usize>,
    pub(super) operator_args: Vec<Expr>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedTensorCallbackEvalState {
    operator: Expr,
    output_head: Expr,
    output_shape: FrozenFunctionalSequence<usize>,
    operator_args: FrozenFunctionalSequence<Expr>,
}

impl SuspendedTensorCallbackEvalState {
    fn freeze(state: TensorCallbackEvalState) -> Self {
        Self {
            operator: state.operator,
            output_head: state.output_head,
            output_shape: FrozenFunctionalSequence::freeze(state.output_shape),
            operator_args: FrozenFunctionalSequence::freeze(state.operator_args),
        }
    }
}

#[derive(FrameRootTrace)]
struct InnerEvalState {
    f: Expr,
    g: Expr,
    head: Expr,
    left_shape: Vec<usize>,
    right_shape: Vec<usize>,
    left_flat: Vec<Expr>,
    right_flat: Vec<Expr>,
    output_shape: Vec<usize>,
    output_indices: Vec<Vec<usize>>,
    contract_dim: usize,
    left_prefix_rank: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedInnerEvalState {
    f: Expr,
    g: Expr,
    head: Expr,
    left_shape: FrozenFunctionalSequence<usize>,
    right_shape: FrozenFunctionalSequence<usize>,
    left_flat: FrozenFunctionalSequence<Expr>,
    right_flat: FrozenFunctionalSequence<Expr>,
    output_shape: FrozenFunctionalSequence<usize>,
    output_indices: FrozenFunctionalSequence<Vec<usize>>,
    contract_dim: usize,
    left_prefix_rank: usize,
}

impl SuspendedInnerEvalState {
    fn freeze(state: InnerEvalState) -> Self {
        Self {
            f: state.f,
            g: state.g,
            head: state.head,
            left_shape: FrozenFunctionalSequence::freeze(state.left_shape),
            right_shape: FrozenFunctionalSequence::freeze(state.right_shape),
            left_flat: FrozenFunctionalSequence::freeze(state.left_flat),
            right_flat: FrozenFunctionalSequence::freeze(state.right_flat),
            output_shape: FrozenFunctionalSequence::freeze(state.output_shape),
            output_indices: FrozenFunctionalSequence::freeze(state.output_indices),
            contract_dim: state.contract_dim,
            left_prefix_rank: state.left_prefix_rank,
        }
    }
}

pub(super) fn apply_array_reduce(
    session: &mut Session,
    operator: &Expr,
    shape: &[usize],
    flat: &[Expr],
    spec: &ArrayReduceSpec,
) -> Result<Expr, EvalError> {
    let list_head = system_expr("List");

    let ArrayReduceSpec::Grouped(groups) = spec else {
        return resume_tensor_callback_from_index(
            session,
            TensorCallbackEvalState {
                operator: operator.clone(),
                output_head: list_head,
                output_shape: shape.to_vec(),
                operator_args: flat.to_vec(),
            },
            0,
            Vec::with_capacity(flat.len()),
        );
    };

    let rank = shape.len();
    let reduced_dims = groups.iter().flatten().copied().collect::<Vec<_>>();
    let kept_dims = (0..rank)
        .filter(|axis| !reduced_dims.contains(axis))
        .collect::<Vec<_>>();
    let output_shape = kept_dims
        .iter()
        .map(|axis| shape[*axis])
        .collect::<Vec<_>>();
    let kept_indices = enumerate_indices(&output_shape);
    let grouped_shape = groups
        .iter()
        .map(|group| group.iter().map(|axis| shape[*axis]).product())
        .collect::<Vec<_>>();
    let grouped_indices = enumerate_indices(&grouped_shape);

    let mut operator_args = Vec::with_capacity(kept_indices.len());
    for kept_index in kept_indices {
        let mut values = Vec::with_capacity(grouped_indices.len());
        for grouped_index in &grouped_indices {
            let full_index = array_reduce_full_index(
                rank,
                &kept_dims,
                &kept_index,
                groups,
                grouped_index,
                shape,
            );
            let value = tensor_scalar_at(flat, shape, &full_index)
                .expect("validated array reductions should resolve a scalar")
                .clone();
            values.push(value);
        }

        let operator_arg = rebuild_tensor_expr(&list_head, &grouped_shape, &values);
        operator_args.push(operator_arg);
    }

    resume_tensor_callback_from_index(
        session,
        TensorCallbackEvalState {
            operator: operator.clone(),
            output_head: list_head,
            output_shape,
            operator_args,
        },
        0,
        Vec::with_capacity(kept_dims.iter().map(|axis| shape[*axis]).product()),
    )
}

pub(super) fn array_reduce_full_index(
    rank: usize,
    kept_dims: &[usize],
    kept_index: &[usize],
    groups: &[Vec<usize>],
    grouped_index: &[usize],
    shape: &[usize],
) -> Vec<usize> {
    let mut full_index = vec![0usize; rank];
    for (position, axis) in kept_dims.iter().copied().enumerate() {
        full_index[axis] = kept_index[position];
    }

    for (group_position, group) in groups.iter().enumerate() {
        let group_shape = group.iter().map(|axis| shape[*axis]).collect::<Vec<_>>();
        let decoded = decode_grouped_axis_index(grouped_index[group_position], &group_shape);
        for (axis_position, axis) in group.iter().copied().enumerate() {
            full_index[axis] = decoded[axis_position];
        }
    }

    full_index
}

pub(super) fn compute_inner(
    session: &mut Session,
    caller: &str,
    f: &Expr,
    left: &Expr,
    right: &Expr,
    g: &Expr,
) -> Result<Option<Expr>, EvalError> {
    let (
        Expr::Call {
            head: left_head, ..
        },
        Expr::Call {
            head: right_head, ..
        },
    ) = (left, right)
    else {
        return Ok(None);
    };

    if left_head.as_ref() != right_head.as_ref() {
        return Ok(None);
    }

    let head = left_head.as_ref().clone();
    let left_tensor = parse_tensor_node(left, &head);
    let right_tensor = parse_tensor_node(right, &head);
    let Some(left_shape) = tensor_shape(&left_tensor) else {
        return Ok(None);
    };
    let Some(right_shape) = tensor_shape(&right_tensor) else {
        return Ok(None);
    };
    let Some((&contract_dim, right_suffix)) = right_shape.split_first() else {
        return Ok(None);
    };
    let Some((&left_contract_dim, left_prefix)) = left_shape.split_last() else {
        return Ok(None);
    };

    if left_contract_dim != contract_dim {
        let tag = if caller == "Dot" { "dot" } else { "dim" };
        push_tensor_message_once(
            session,
            caller,
            tag,
            format!(
                "Tensor contraction dimensions {} and {} do not match for {} and {}.",
                left_contract_dim, contract_dim, left, right
            ),
        );
        return Ok(None);
    }

    let left_flat = flatten_tensor_scalars(&left_tensor);
    let right_flat = flatten_tensor_scalars(&right_tensor);
    let output_shape = left_prefix
        .iter()
        .chain(right_suffix.iter())
        .copied()
        .collect::<Vec<_>>();
    let output_indices = enumerate_indices(&output_shape);
    let left_prefix_rank = left_prefix.len();
    let state = InnerEvalState {
        f: f.clone(),
        g: g.clone(),
        head,
        left_shape,
        right_shape,
        left_flat,
        right_flat,
        output_shape,
        output_indices,
        contract_dim,
        left_prefix_rank,
    };

    resume_inner_output_from_index(session, state, 0, Vec::new()).map(Some)
}

fn push_tensor_message_once(session: &mut Session, symbol: &str, tag: &str, text: String) {
    if session
        .messages()
        .iter()
        .any(|message| message.symbol == symbol && message.tag == tag && message.text == text)
    {
        return;
    }

    session.push_message(symbol, tag, text);
}

pub(super) fn resume_tensor_callback_from_index(
    session: &mut Session,
    state: TensorCallbackEvalState,
    mut index: usize,
    mut outputs: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(operator_arg) = state.operator_args.get(index).cloned() else {
            return Ok(rebuild_tensor_expr(
                &state.output_head,
                &state.output_shape,
                &outputs,
            ));
        };

        let value = match session.evaluate(Expr::call(state.operator.clone(), vec![operator_arg])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedTensorCallbackEvalState::freeze(state);
                                        let outputs = PersistentFunctionalSequence::freeze(outputs);
                ResumeTensorCallbackFromIndexSuspensionFrame { state, index, outputs }
                },
                );
            }
            Err(error) => return Err(error),
        };

        outputs.push(value);
        index += 1;
    }
}

fn resume_suspended_tensor_callback_from_index(
    session: &mut Session,
    state: SuspendedTensorCallbackEvalState,
    mut index: usize,
    mut outputs: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(operator_arg) = state.operator_args.get(index).cloned() else {
            return Ok(rebuild_tensor_expr(
                &state.output_head,
                state.output_shape.as_slice(),
                &outputs.to_vec(),
            ));
        };

        let value = match session.evaluate(Expr::call(state.operator.clone(), vec![operator_arg])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedTensorCallbackFromIndexSuspensionFrame { state, index, outputs },
                );
            }
            Err(error) => return Err(error),
        };

        outputs = outputs.appended(value);
        index += 1;
    }
}

fn resume_suspended_tensor_callback_with_value(
    session: &mut Session,
    state: SuspendedTensorCallbackEvalState,
    index: usize,
    outputs: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let outputs = outputs.appended(value);
    resume_suspended_tensor_callback_from_index(session, state, index + 1, outputs)
}

fn resume_inner_output_from_index(
    session: &mut Session,
    state: InnerEvalState,
    output_index: usize,
    output_scalars: Vec<Expr>,
) -> Result<Expr, EvalError> {
    resume_inner_from_indices(session, state, output_index, 0, Vec::new(), output_scalars)
}

fn resume_inner_from_indices(
    session: &mut Session,
    state: InnerEvalState,
    mut output_index: usize,
    mut contract_index: usize,
    mut terms: Vec<Expr>,
    mut output_scalars: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if output_index >= state.output_indices.len() {
            return Ok(rebuild_tensor_expr(
                &state.head,
                &state.output_shape,
                &output_scalars,
            ));
        }

        while contract_index < state.contract_dim {
            let Some(output_indices) = state.output_indices.get(output_index) else {
                return Ok(rebuild_tensor_expr(
                    &state.head,
                    &state.output_shape,
                    &output_scalars,
                ));
            };
            let (left_index, right_index) = output_indices.split_at(state.left_prefix_rank);
            let mut lhs = left_index.to_vec();
            lhs.push(contract_index);
            let mut rhs = vec![contract_index];
            rhs.extend_from_slice(right_index);

            let Some(lhs_value) = tensor_scalar_at(&state.left_flat, &state.left_shape, &lhs)
            else {
                return Ok(rebuild_tensor_expr(
                    &state.head,
                    &state.output_shape,
                    &output_scalars,
                ));
            };
            let Some(rhs_value) = tensor_scalar_at(&state.right_flat, &state.right_shape, &rhs)
            else {
                return Ok(rebuild_tensor_expr(
                    &state.head,
                    &state.output_shape,
                    &output_scalars,
                ));
            };

            let term = match session.evaluate(Expr::call(
                state.f.clone(),
                vec![lhs_value.clone(), rhs_value.clone()],
            )) {
                Ok(term) => term,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let state = SuspendedInnerEvalState::freeze(state);
                                                let terms = PersistentFunctionalSequence::freeze(terms);
                                                let output_scalars =
                                                    PersistentFunctionalSequence::freeze(output_scalars);
                    ResumeInnerFromIndicesSuspensionFrame1 { state, output_index, contract_index, terms, output_scalars }
                    },
                    );
                }
                Err(error) => return Err(error),
            };

            terms.push(term);
            contract_index += 1;
        }

        let reduced = match session.evaluate(Expr::call(state.g.clone(), terms)) {
            Ok(reduced) => reduced,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedInnerEvalState::freeze(state);
                                        let output_scalars = PersistentFunctionalSequence::freeze(output_scalars);
                ResumeInnerFromIndicesSuspensionFrame2 { state, output_index, output_scalars }
                },
                );
            }
            Err(error) => return Err(error),
        };

        output_scalars.push(reduced);
        output_index += 1;
        contract_index = 0;
        terms = Vec::new();
    }
}

fn resume_suspended_inner_from_indices(
    session: &mut Session,
    state: SuspendedInnerEvalState,
    mut output_index: usize,
    mut contract_index: usize,
    mut terms: PersistentFunctionalSequence<Expr>,
    mut output_scalars: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if output_index >= state.output_indices.len() {
            return Ok(rebuild_tensor_expr(
                &state.head,
                state.output_shape.as_slice(),
                &output_scalars.to_vec(),
            ));
        }

        while contract_index < state.contract_dim {
            let Some(output_indices) = state.output_indices.get(output_index) else {
                return Ok(rebuild_tensor_expr(
                    &state.head,
                    state.output_shape.as_slice(),
                    &output_scalars.to_vec(),
                ));
            };
            let (left_index, right_index) = output_indices.split_at(state.left_prefix_rank);
            let mut lhs = left_index.to_vec();
            lhs.push(contract_index);
            let mut rhs = vec![contract_index];
            rhs.extend_from_slice(right_index);

            let Some(lhs_value) = tensor_scalar_at(
                state.left_flat.as_slice(),
                state.left_shape.as_slice(),
                &lhs,
            ) else {
                return Ok(rebuild_tensor_expr(
                    &state.head,
                    state.output_shape.as_slice(),
                    &output_scalars.to_vec(),
                ));
            };
            let Some(rhs_value) = tensor_scalar_at(
                state.right_flat.as_slice(),
                state.right_shape.as_slice(),
                &rhs,
            ) else {
                return Ok(rebuild_tensor_expr(
                    &state.head,
                    state.output_shape.as_slice(),
                    &output_scalars.to_vec(),
                ));
            };

            let term = match session.evaluate(Expr::call(
                state.f.clone(),
                vec![lhs_value.clone(), rhs_value.clone()],
            )) {
                Ok(term) => term,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeSuspendedInnerFromIndicesSuspensionFrame1 { state, output_index, contract_index, terms, output_scalars },
                    );
                }
                Err(error) => return Err(error),
            };

            terms = terms.appended(term);
            contract_index += 1;
        }

        let reduced = match session.evaluate(Expr::call(state.g.clone(), terms.to_vec())) {
            Ok(reduced) => reduced,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedInnerFromIndicesSuspensionFrame2 { state, output_index, output_scalars },
                );
            }
            Err(error) => return Err(error),
        };

        output_scalars = output_scalars.appended(reduced);
        output_index += 1;
        contract_index = 0;
        terms = PersistentFunctionalSequence::freeze(Vec::new());
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_inner_term_with_value(
    session: &mut Session,
    state: SuspendedInnerEvalState,
    output_index: usize,
    contract_index: usize,
    terms: PersistentFunctionalSequence<Expr>,
    output_scalars: PersistentFunctionalSequence<Expr>,
    term: Expr,
) -> Result<Expr, EvalError> {
    let terms = terms.appended(term);
    resume_suspended_inner_from_indices(
        session,
        state,
        output_index,
        contract_index + 1,
        terms,
        output_scalars,
    )
}

fn resume_suspended_inner_reduced_with_value(
    session: &mut Session,
    state: SuspendedInnerEvalState,
    output_index: usize,
    output_scalars: PersistentFunctionalSequence<Expr>,
    reduced: Expr,
) -> Result<Expr, EvalError> {
    let output_scalars = output_scalars.appended(reduced);
    resume_suspended_inner_from_indices(
        session,
        state,
        output_index + 1,
        0,
        PersistentFunctionalSequence::freeze(Vec::new()),
        output_scalars,
    )
}

pub(super) fn parse_tensor_node(expr: &Expr, expected_head: &Expr) -> TensorNode {
    match expr {
        Expr::Call { head, args } if head.as_ref() == expected_head => TensorNode::Array(
            args.iter()
                .map(|arg| parse_tensor_node(arg, expected_head))
                .collect(),
        ),
        _ => TensorNode::Scalar(expr.clone()),
    }
}

pub(super) fn tensor_shape(node: &TensorNode) -> Option<Vec<usize>> {
    match node {
        TensorNode::Scalar(_) => Some(Vec::new()),
        TensorNode::Array(items) => {
            let Some(first) = items.first() else {
                return Some(vec![0]);
            };
            let child_shape = tensor_shape(first)?;
            for item in items.iter().skip(1) {
                if tensor_shape(item)? != child_shape {
                    return None;
                }
            }

            let mut shape = vec![items.len()];
            shape.extend(child_shape);
            Some(shape)
        }
    }
}

pub(super) fn flatten_tensor_scalars(node: &TensorNode) -> Vec<Expr> {
    let mut flat = Vec::new();
    flatten_tensor_scalars_into(node, &mut flat);
    flat
}

pub(super) fn flatten_tensor_scalars_into(node: &TensorNode, flat: &mut Vec<Expr>) {
    match node {
        TensorNode::Scalar(expr) => flat.push(expr.clone()),
        TensorNode::Array(items) => {
            for item in items {
                flatten_tensor_scalars_into(item, flat);
            }
        }
    }
}

pub(super) fn tensor_scalar_at<'a>(
    flat: &'a [Expr],
    shape: &[usize],
    index: &[usize],
) -> Option<&'a Expr> {
    let offset = tensor_flat_offset(shape, index)?;
    flat.get(offset)
}

pub(super) fn tensor_flat_offset(shape: &[usize], index: &[usize]) -> Option<usize> {
    if shape.len() != index.len() {
        return None;
    }

    if shape.is_empty() {
        return Some(0);
    }

    let mut offset = 0usize;
    let mut stride = shape.iter().product::<usize>();
    for (&axis_len, &axis_index) in shape.iter().zip(index.iter()) {
        if axis_index >= axis_len || axis_len == 0 {
            return None;
        }
        stride /= axis_len;
        offset += axis_index * stride;
    }

    Some(offset)
}

pub(super) fn enumerate_indices(shape: &[usize]) -> Vec<Vec<usize>> {
    if shape.is_empty() {
        return vec![Vec::new()];
    }

    let suffixes = enumerate_indices(&shape[1..]);
    let mut indices = Vec::new();
    for first in 0..shape[0] {
        for suffix in &suffixes {
            let mut index = Vec::with_capacity(shape.len());
            index.push(first);
            index.extend_from_slice(suffix);
            indices.push(index);
        }
    }
    indices
}

pub(super) fn decode_grouped_axis_index(mut index: usize, shape: &[usize]) -> Vec<usize> {
    if shape.is_empty() {
        return Vec::new();
    }

    let mut decoded = vec![0usize; shape.len()];
    for axis in (0..shape.len()).rev() {
        let len = shape[axis];
        if len == 0 {
            decoded[axis] = 0;
        } else {
            decoded[axis] = index % len;
            index /= len;
        }
    }
    decoded
}

pub(super) fn rebuild_tensor_expr(head: &Expr, shape: &[usize], flat: &[Expr]) -> Expr {
    let mut offset = 0usize;
    rebuild_tensor_expr_from_offset(head, shape, flat, &mut offset)
}

pub(super) fn rebuild_tensor_expr_from_offset(
    head: &Expr,
    shape: &[usize],
    flat: &[Expr],
    offset: &mut usize,
) -> Expr {
    if shape.is_empty() {
        let expr = flat
            .get(*offset)
            .cloned()
            .unwrap_or_else(|| Expr::call(head.clone(), Vec::new()));
        *offset += 1;
        return expr;
    }

    let items = (0..shape[0])
        .map(|_| rebuild_tensor_expr_from_offset(head, &shape[1..], flat, offset))
        .collect();
    Expr::call(head.clone(), items)
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<InnerEvalState>();
    assert_traces::<TensorCallbackEvalState>();
    assert_traces::<SuspendedInnerEvalState>();
    assert_traces::<SuspendedTensorCallbackEvalState>();
};
