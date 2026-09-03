mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Debug, PartialEq, Eq, FrameRootTrace)]
struct ArrayEvalState {
    operator: Expr,
    output_head: Expr,
    shape: Vec<usize>,
    origins: Vec<Expr>,
    total: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedArrayEvalState {
    operator: Expr,
    output_head: Expr,
    shape: FrozenFunctionalSequence<usize>,
    origins: FrozenFunctionalSequence<Expr>,
    total: usize,
}

impl SuspendedArrayEvalState {
    fn freeze(state: ArrayEvalState) -> Self {
        Self {
            operator: state.operator,
            output_head: state.output_head,
            shape: FrozenFunctionalSequence::freeze(state.shape),
            origins: FrozenFunctionalSequence::freeze(state.origins),
            total: state.total,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OuterInput {
    shape: Vec<usize>,
    values: Vec<Expr>,
}

#[derive(Debug, PartialEq, Eq, FrameRootTrace)]
struct OuterEvalState {
    operator: Expr,
    output_head: Expr,
    input_shapes: Vec<Vec<usize>>,
    input_values: Vec<Vec<Expr>>,
    output_shape: Vec<usize>,
    total: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedOuterEvalState {
    operator: Expr,
    output_head: Expr,
    input_shapes: FrozenFunctionalSequence<Vec<usize>>,
    input_values: FrozenFunctionalSequence<Vec<Expr>>,
    output_shape: FrozenFunctionalSequence<usize>,
    total: usize,
}

impl SuspendedOuterEvalState {
    fn freeze(state: OuterEvalState) -> Self {
        Self {
            operator: state.operator,
            output_head: state.output_head,
            input_shapes: FrozenFunctionalSequence::freeze(state.input_shapes),
            input_values: FrozenFunctionalSequence::freeze(state.input_values),
            output_shape: FrozenFunctionalSequence::freeze(state.output_shape),
            total: state.total,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OuterHeadMismatch {
    actual: Expr,
    expected: Expr,
    position: usize,
}

pub(crate) fn eval_array(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (operator, dims_expr, origins_expr, output_head) = match args {
        [operator, dims] => (operator.clone(), dims, None, system_expr("List")),
        [operator, dims, origins] => (operator.clone(), dims, Some(origins), system_expr("List")),
        [operator, dims, origins, output_head] => {
            (operator.clone(), dims, Some(origins), output_head.clone())
        }
        _ => return Ok(session.invalid_call_with_arg_range("Array", args, 2, 4)),
    };

    let Some(shape_big) = parse_array_dimensions(dims_expr) else {
        return Ok(session.invalid_call_with_arg_form(
            "Array",
            args,
            "expected a non-negative integer dimension or a list of non-negative integer dimensions",
        ));
    };
    let Some(origins) = parse_array_origins(origins_expr, shape_big.len()) else {
        return Ok(session.invalid_call_with_arg_form(
            "Array",
            args,
            "expected a scalar origin or a list of origins matching the array rank",
        ));
    };
    let total = checked_generated_shape_counts(session, &shape_big)?;
    let Some(shape) = shape_big
        .iter()
        .map(bigint_count_to_usize)
        .collect::<Option<Vec<_>>>()
    else {
        return Ok(session.invalid_call_with_arg_form(
            "Array",
            args,
            "expected dimensions representable by the current runtime",
        ));
    };

    let state = ArrayEvalState {
        operator,
        output_head,
        shape,
        origins,
        total,
    };
    resume_array_from_index(session, state, 0, Vec::with_capacity(total))
}

pub(crate) fn eval_outer(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() < 3 {
        return Ok(session.invalid_call_with_arg_range("Outer", args, 3, usize::MAX));
    }

    let operator = args[0].clone();
    let (inputs, level) = parse_outer_inputs_and_level(&args[1..]);
    if inputs.len() < 2 {
        return Ok(session.invalid_call_with_arg_range("Outer", args, 3, usize::MAX));
    }

    let mut output_head = None::<Expr>;
    let mut parsed_inputs = Vec::with_capacity(inputs.len());
    for (offset, input) in inputs.iter().enumerate() {
        let expected = output_head.as_ref();
        match parse_outer_input(input, expected, level) {
            Ok(parsed) => {
                if output_head.is_none() {
                    output_head = Some(
                        outer_top_head(input)
                            .expect("parsed Outer inputs with positive rank should have a head"),
                    );
                }
                parsed_inputs.push(parsed);
            }
            Err(mismatch) => {
                return Ok(invalid_outer_heads_call(
                    session,
                    args,
                    mismatch,
                    offset + 2,
                ));
            }
        }
    }

    let output_head = output_head.unwrap_or_else(|| system_expr("List"));
    let output_shape = parsed_inputs
        .iter()
        .flat_map(|input| input.shape.iter().copied())
        .collect::<Vec<_>>();
    let output_shape_big = output_shape
        .iter()
        .map(|dim| BigInt::from(*dim))
        .collect::<Vec<_>>();
    let total = checked_generated_shape_counts(session, &output_shape_big)?;
    let state = OuterEvalState {
        operator,
        output_head,
        input_shapes: parsed_inputs
            .iter()
            .map(|input| input.shape.clone())
            .collect(),
        input_values: parsed_inputs
            .into_iter()
            .map(|input| input.values)
            .collect(),
        output_shape,
        total,
    };

    resume_outer_from_index(session, state, 0, Vec::with_capacity(total))
}

fn checked_generated_shape_counts(session: &Session, shape: &[BigInt]) -> Result<usize, EvalError> {
    let scalar_count = shape_scalar_count(shape);
    let node_count = shape_generated_node_count(shape);
    let generated_count = scalar_count.clone().max(node_count.clone());
    session.enforce_active_memory_limit_for_projected_bytes(
        generated_expr_nodes_memory_proxy_bytes(
            bigint_count_to_usize(&node_count).unwrap_or(usize::MAX),
        ),
    )?;
    session.enforce_generated_element_limit_for_big_count(&generated_count)?;
    session.enforce_iteration_work_limit_for_big_count(&scalar_count)?;
    Ok(bigint_count_to_usize(&scalar_count)
        .expect("resource limit prechecks keep generated counts representable"))
}

fn resume_array_from_index(
    session: &mut Session,
    state: ArrayEvalState,
    mut index: usize,
    mut outputs: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.total {
            return Ok(rebuild_tensor_expr(
                &state.output_head,
                &state.shape,
                &outputs,
            ));
        }

        let call = Expr::call(
            state.operator.clone(),
            array_index_args(&state.shape, &state.origins, index),
        );
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedArrayEvalState::freeze(state);
                                        let outputs = PersistentFunctionalSequence::freeze(outputs);
                ResumeArrayFromIndexSuspensionFrame { state, index, outputs }
                },
                );
            }
            Err(error) => return Err(error),
        };

        outputs.push(value);
        index += 1;
    }
}

fn resume_suspended_array_from_index(
    session: &mut Session,
    state: SuspendedArrayEvalState,
    mut index: usize,
    mut outputs: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.total {
            return Ok(rebuild_tensor_expr(
                &state.output_head,
                state.shape.as_slice(),
                &outputs.to_vec(),
            ));
        }

        let call = Expr::call(
            state.operator.clone(),
            array_index_args(state.shape.as_slice(), state.origins.as_slice(), index),
        );
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedArrayFromIndexSuspensionFrame { state, index, outputs },
                );
            }
            Err(error) => return Err(error),
        };

        outputs = outputs.appended(value);
        index += 1;
    }
}

fn resume_suspended_array_with_value(
    session: &mut Session,
    state: SuspendedArrayEvalState,
    index: usize,
    outputs: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let outputs = outputs.appended(value);
    resume_suspended_array_from_index(session, state, index + 1, outputs)
}

fn resume_outer_from_index(
    session: &mut Session,
    state: OuterEvalState,
    mut index: usize,
    mut outputs: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.total {
            return Ok(rebuild_tensor_expr(
                &state.output_head,
                &state.output_shape,
                &outputs,
            ));
        }

        let call = Expr::call(state.operator.clone(), outer_call_args(&state, index));
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedOuterEvalState::freeze(state);
                                        let outputs = PersistentFunctionalSequence::freeze(outputs);
                ResumeOuterFromIndexSuspensionFrame { state, index, outputs }
                },
                );
            }
            Err(error) => return Err(error),
        };

        outputs.push(value);
        index += 1;
    }
}

fn resume_suspended_outer_from_index(
    session: &mut Session,
    state: SuspendedOuterEvalState,
    mut index: usize,
    mut outputs: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if index >= state.total {
            return Ok(rebuild_tensor_expr(
                &state.output_head,
                state.output_shape.as_slice(),
                &outputs.to_vec(),
            ));
        }

        let call = Expr::call(
            state.operator.clone(),
            suspended_outer_call_args(&state, index),
        );
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedOuterFromIndexSuspensionFrame { state, index, outputs },
                );
            }
            Err(error) => return Err(error),
        };

        outputs = outputs.appended(value);
        index += 1;
    }
}

fn resume_suspended_outer_with_value(
    session: &mut Session,
    state: SuspendedOuterEvalState,
    index: usize,
    outputs: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let outputs = outputs.appended(value);
    resume_suspended_outer_from_index(session, state, index + 1, outputs)
}

fn invalid_outer_heads_call(
    session: &mut Session,
    args: &[Expr],
    mismatch: OuterHeadMismatch,
    position: usize,
) -> Expr {
    let actual_position = mismatch.position.max(position);
    session.invalid_call_with_message(
        "Outer",
        args,
        "heads",
        format!(
            "Heads {} and {} at positions {} and 2 are expected to be the same.",
            mismatch.actual, mismatch.expected, actual_position
        ),
    )
}

fn parse_array_dimensions(expr: &Expr) -> Option<Vec<BigInt>> {
    if let Some(dim) = expr_as_non_negative_bigint(expr) {
        return Some(vec![dim]);
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(expr_as_non_negative_bigint).collect()
}

fn expr_as_non_negative_bigint(expr: &Expr) -> Option<BigInt> {
    let value = expr_as_exact_integer(expr)?;
    (value >= BigInt::from(0_u8)).then_some(value)
}

fn parse_array_origins(origin: Option<&Expr>, rank: usize) -> Option<Vec<Expr>> {
    let Some(origin) = origin else {
        return Some(vec![Expr::integer(BigInt::from(1_u8)); rank]);
    };

    if let Expr::Call { head, args, .. } = origin {
        if is_system_symbol_name(head, "List") {
            if args.len() != rank {
                return None;
            }
            return Some(args.to_vec());
        }
    }

    Some(vec![origin.clone(); rank])
}

fn shape_scalar_count(shape: &[BigInt]) -> BigInt {
    shape.iter().cloned().product::<BigInt>()
}

fn shape_generated_node_count(shape: &[BigInt]) -> BigInt {
    let Some((first, rest)) = shape.split_first() else {
        return BigInt::from(1_u8);
    };

    BigInt::from(1_u8) + first * shape_generated_node_count(rest)
}

fn bigint_count_to_usize(count: &BigInt) -> Option<usize> {
    count.to_string().parse().ok()
}

fn array_index_args(shape: &[usize], origins: &[Expr], flat_index: usize) -> Vec<Expr> {
    decode_tensor_index(flat_index, shape)
        .into_iter()
        .zip(origins.iter())
        .map(|(offset, origin)| array_index_expr(origin, offset))
        .collect()
}

fn array_index_expr(origin: &Expr, offset: usize) -> Expr {
    if offset == 0 {
        return origin.clone();
    }

    if let Some(value) = expr_as_exact_integer(origin) {
        return Expr::integer(value + BigInt::from(offset));
    }

    Expr::call(
        system_expr("Plus"),
        vec![Expr::integer(BigInt::from(offset)), origin.clone()],
    )
}

fn parse_outer_inputs_and_level(args: &[Expr]) -> (&[Expr], Option<usize>) {
    if args.len() >= 3 {
        if let Some(level) = expr_as_non_negative_usize(args.last().expect("non-empty args")) {
            return (&args[..args.len() - 1], Some(level));
        }
    }

    (args, None)
}

fn parse_outer_input(
    expr: &Expr,
    expected_head: Option<&Expr>,
    level: Option<usize>,
) -> Result<OuterInput, OuterHeadMismatch> {
    let head = outer_top_head(expr).ok_or_else(|| OuterHeadMismatch {
        actual: structural_head_name_expr(expr),
        expected: expected_head
            .cloned()
            .unwrap_or_else(|| system_expr("List")),
        position: 2,
    })?;

    if let Some(expected) = expected_head {
        if &head != expected {
            return Err(OuterHeadMismatch {
                actual: head,
                expected: expected.clone(),
                position: 2,
            });
        }
    }

    let tensor = parse_tensor_node(expr, &head);
    let Some(shape) = tensor_shape(&tensor) else {
        return Err(OuterHeadMismatch {
            actual: head,
            expected: expected_head
                .cloned()
                .unwrap_or_else(|| outer_top_head(expr).unwrap()),
            position: 2,
        });
    };

    let effective_level = level.map_or(shape.len(), |level| level.min(shape.len()));
    let selected_shape = shape
        .iter()
        .copied()
        .take(effective_level)
        .collect::<Vec<_>>();
    let mut values = Vec::new();
    collect_tensor_values_at_level(expr, &head, effective_level, 0, &mut values);

    Ok(OuterInput {
        shape: selected_shape,
        values,
    })
}

fn outer_top_head(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Call { head, .. } => Some(head.as_ref().clone()),
        Expr::Atom(_) => None,
    }
}

fn structural_head_name_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Atom(Atom::Integer(_)) => system_expr("Integer"),
        Expr::Atom(Atom::Rational(_)) => system_expr("Rational"),
        Expr::Atom(Atom::String(_)) => system_expr("String"),
        Expr::Atom(Atom::Symbol(_)) => system_expr("Symbol"),
        Expr::Call { head, .. } => head.as_ref().clone(),
    }
}

fn collect_tensor_values_at_level(
    expr: &Expr,
    expected_head: &Expr,
    target_level: usize,
    current_level: usize,
    values: &mut Vec<Expr>,
) {
    if current_level == target_level {
        values.push(expr.clone());
        return;
    }

    match expr {
        Expr::Call { head, args, .. } if head.as_ref() == expected_head => {
            for arg in args {
                collect_tensor_values_at_level(
                    arg,
                    expected_head,
                    target_level,
                    current_level + 1,
                    values,
                );
            }
        }
        _ => values.push(expr.clone()),
    }
}

fn outer_call_args(state: &OuterEvalState, flat_index: usize) -> Vec<Expr> {
    let output_index = decode_tensor_index(flat_index, &state.output_shape);
    let mut offset = 0usize;
    let mut args = Vec::with_capacity(state.input_shapes.len());

    for (shape, values) in state.input_shapes.iter().zip(state.input_values.iter()) {
        let end = offset + shape.len();
        let input_index = &output_index[offset..end];
        let flat = tensor_flat_offset(shape, input_index)
            .expect("decoded Outer output indices should map into input tensors");
        args.push(values[flat].clone());
        offset = end;
    }

    args
}

fn suspended_outer_call_args(state: &SuspendedOuterEvalState, flat_index: usize) -> Vec<Expr> {
    let output_index = decode_tensor_index(flat_index, state.output_shape.as_slice());
    let mut offset = 0usize;
    let mut args = Vec::with_capacity(state.input_shapes.len());

    for (shape, values) in state
        .input_shapes
        .as_slice()
        .iter()
        .zip(state.input_values.as_slice())
    {
        let end = offset + shape.len();
        let input_index = &output_index[offset..end];
        let flat = tensor_flat_offset(shape, input_index)
            .expect("decoded Outer output indices should map into input tensors");
        args.push(values[flat].clone());
        offset = end;
    }

    args
}

fn decode_tensor_index(mut index: usize, shape: &[usize]) -> Vec<usize> {
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
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ArrayEvalState>();
    assert_traces::<OuterEvalState>();
    assert_traces::<SuspendedArrayEvalState>();
    assert_traces::<SuspendedOuterEvalState>();
};
