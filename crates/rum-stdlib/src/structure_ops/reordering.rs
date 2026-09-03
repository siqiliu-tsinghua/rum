use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_reverse(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, levels) = match args {
        [expr] => (expr, ReverseLevels::Selected(BTreeSet::from([1_usize]))),
        [expr, level_spec] => {
            let Some(levels) = parse_reverse_levels(level_spec) else {
                return Ok(invalid_reverse_level_call(session, args));
            };
            (expr, levels)
        }
        _ => return Ok(session.invalid_call_with_arg_range("Reverse", args, 1, 2)),
    };

    let Expr::Call { .. } = expr else {
        return Ok(invalid_reverse_normal_call(session, args));
    };

    Ok(reverse_at_levels(expr, &levels, 1))
}

pub(crate) fn eval_transpose(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, permutation) = match args {
        [expr] => (expr, TransposePermutation::SwapFirstTwo),
        [expr, permutation] => {
            let Some(permutation) = parse_transpose_permutation(permutation) else {
                return Ok(invalid_transpose_permutation_call(session, args));
            };
            (expr, permutation)
        }
        _ => return Ok(session.invalid_call_with_arg_range("Transpose", args, 1, 2)),
    };

    let Some(items) = list_items(expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if matches!(permutation, TransposePermutation::Identity) {
        return Ok(expr.clone());
    }

    if matches!(permutation, TransposePermutation::SwapFirstTwo) {
        return match transpose_plain_list_matrix(items) {
            Some(result) => Ok(result),
            None => Ok(invalid_transpose_matrix_call(session, args)),
        };
    }

    let TransposePermutation::General(permutation) = permutation else {
        unreachable!("identity and first-two swap handled above");
    };
    let Some(shape) = plain_list_tensor_shape(expr) else {
        return Ok(invalid_transpose_matrix_call(session, args));
    };
    if !is_valid_tensor_permutation(&permutation, shape.len()) {
        return Ok(invalid_transpose_permutation_call(session, args));
    }
    let element_count = tensor_element_count(&shape);
    session.enforce_generated_element_limit_for_big_count(&element_count)?;

    match transpose_plain_list_tensor(expr, &shape, &permutation) {
        Some(result) => Ok(result),
        None => Ok(invalid_transpose_matrix_call(session, args)),
    }
}

fn invalid_reverse_normal_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Reverse",
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr("Reverse"), args.to_vec())
        ),
    )
}

fn invalid_reverse_level_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Reverse",
        args,
        "ilsmp",
        format!(
            "All, a positive machine-sized integer, or a list of positive machine-sized integers expected at position 2 of {}.",
            Expr::call(system_expr("Reverse"), args.to_vec())
        ),
    )
}

fn invalid_transpose_matrix_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Transpose",
        args,
        "nmtx",
        format!("The first two levels of {} cannot be transposed.", args[0]),
    )
}

fn invalid_transpose_permutation_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Transpose",
        args,
        "permspec",
        format!(
            "Invalid permutation specification found at position 2 in {}.",
            Expr::call(system_expr("Transpose"), args.to_vec())
        ),
    )
}

enum ReverseLevels {
    All,
    Selected(BTreeSet<usize>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TransposePermutation {
    Identity,
    SwapFirstTwo,
    General(Vec<usize>),
}

fn parse_reverse_levels(expr: &Expr) -> Option<ReverseLevels> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(ReverseLevels::All);
    }

    if let Some(level) = expr_as_positive_usize(expr) {
        return Some(ReverseLevels::Selected(BTreeSet::from([level])));
    }

    let items = list_items(expr)?;
    Some(ReverseLevels::Selected(
        items
            .iter()
            .map(expr_as_positive_usize)
            .collect::<Option<BTreeSet<_>>>()?,
    ))
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    expr_as_non_negative_usize(expr).filter(|value| *value > 0)
}

fn reverse_at_levels(expr: &Expr, levels: &ReverseLevels, depth: usize) -> Expr {
    match expr {
        Expr::Atom(_) => expr.clone(),
        Expr::Call { head, args } => {
            let mut reversed = args
                .iter()
                .map(|arg| reverse_at_levels(arg, levels, depth + 1))
                .collect::<Vec<_>>();
            if reverse_applies_at_level(levels, depth) {
                reversed.reverse();
            }
            Expr::call(head.as_ref().clone(), reversed)
        }
    }
}

fn reverse_applies_at_level(levels: &ReverseLevels, depth: usize) -> bool {
    match levels {
        ReverseLevels::All => true,
        ReverseLevels::Selected(levels) => levels.contains(&depth),
    }
}

fn parse_transpose_permutation(expr: &Expr) -> Option<TransposePermutation> {
    let items = list_items(expr)?;
    match items {
        [first, second]
            if expr_as_non_negative_usize(first) == Some(1)
                && expr_as_non_negative_usize(second) == Some(2) =>
        {
            Some(TransposePermutation::Identity)
        }
        [first, second]
            if expr_as_non_negative_usize(first) == Some(2)
                && expr_as_non_negative_usize(second) == Some(1) =>
        {
            Some(TransposePermutation::SwapFirstTwo)
        }
        _ => {
            let permutation = items
                .iter()
                .map(expr_as_positive_usize)
                .collect::<Option<Vec<_>>>()?;
            Some(TransposePermutation::General(permutation))
        }
    }
}

fn transpose_plain_list_matrix(items: &[Expr]) -> Option<Expr> {
    if items.is_empty() {
        return Some(Expr::call(system_expr("List"), Vec::new()));
    }

    if items.iter().all(|item| !is_plain_list_expr(item)) {
        return Some(Expr::call(system_expr("List"), items.to_vec()));
    }

    let rows = items
        .iter()
        .map(list_items)
        .collect::<Option<Vec<&[Expr]>>>()?;
    let width = rows.first().map(|row| row.len()).unwrap_or(0);
    if rows.iter().any(|row| row.len() != width) {
        return None;
    }

    Some(Expr::call(
        system_expr("List"),
        (0..width)
            .map(|column| {
                Expr::call(
                    system_expr("List"),
                    rows.iter().map(|row| row[column].clone()).collect(),
                )
            })
            .collect(),
    ))
}

fn plain_list_tensor_shape(expr: &Expr) -> Option<Vec<usize>> {
    let Expr::Call { head, args } = expr else {
        return Some(Vec::new());
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Some(Vec::new());
    }

    let mut child_shapes = args
        .iter()
        .map(plain_list_tensor_shape)
        .collect::<Option<Vec<_>>>()?
        .into_iter();
    let first_shape = child_shapes.next();
    if child_shapes.any(|shape| Some(&shape) != first_shape.as_ref()) {
        return None;
    }

    let mut shape = vec![args.len()];
    if let Some(first_shape) = first_shape {
        shape.extend(first_shape);
    }
    Some(shape)
}

fn is_valid_tensor_permutation(permutation: &[usize], rank: usize) -> bool {
    if permutation.len() != rank {
        return false;
    }

    let mut seen = vec![false; rank];
    for &axis in permutation {
        if axis == 0 || axis > rank || seen[axis - 1] {
            return false;
        }
        seen[axis - 1] = true;
    }
    true
}

fn tensor_element_count(shape: &[usize]) -> BigInt {
    shape.iter().map(|dim| BigInt::from(*dim)).product()
}

fn transpose_plain_list_tensor(
    expr: &Expr,
    shape: &[usize],
    permutation: &[usize],
) -> Option<Expr> {
    let output_shape = permutation
        .iter()
        .map(|axis| shape[axis - 1])
        .collect::<Vec<_>>();
    build_transposed_tensor(expr, &output_shape, permutation, &mut Vec::new())
}

fn build_transposed_tensor(
    expr: &Expr,
    output_shape: &[usize],
    permutation: &[usize],
    output_indices: &mut Vec<usize>,
) -> Option<Expr> {
    let Some((&dim, rest)) = output_shape.split_first() else {
        let mut input_indices = vec![0; permutation.len()];
        for (output_axis, &input_axis) in permutation.iter().enumerate() {
            input_indices[input_axis - 1] = output_indices[output_axis];
        }
        return tensor_element_at(expr, &input_indices).cloned();
    };

    let mut items = Vec::with_capacity(dim);
    for index in 0..dim {
        output_indices.push(index);
        items.push(build_transposed_tensor(
            expr,
            rest,
            permutation,
            output_indices,
        )?);
        output_indices.pop();
    }
    Some(Expr::call(system_expr(heads::structure::LIST), items))
}

fn tensor_element_at<'a>(expr: &'a Expr, indices: &[usize]) -> Option<&'a Expr> {
    let mut current = expr;
    for &index in indices {
        let Expr::Call { head, args } = current else {
            return None;
        };
        if !is_system_symbol_name(head, heads::structure::LIST) {
            return None;
        }
        current = args.get(index)?;
    }
    Some(current)
}
