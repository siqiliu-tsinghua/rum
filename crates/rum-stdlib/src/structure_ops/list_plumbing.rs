use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_differences(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (list, order) = match args {
        [list] => (list, 1),
        [list, order_expr] => {
            let Some(order) = expr_as_non_negative_usize(order_expr) else {
                return Ok(invalid_differences_order_call(session, args));
            };
            (list, order)
        }
        _ => return Ok(session.invalid_call_with_arg_range("Differences", args, 1, 2)),
    };

    let Some(items) = list_items(list) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let mut current = items.to_vec();
    for _ in 0..order {
        current = successive_differences(session, &current)?;
        if current.is_empty() {
            break;
        }
    }

    session.enforce_generated_element_limit_for_count(current.len())?;
    Ok(Expr::call(system_expr(heads::structure::LIST), current))
}

pub(crate) fn eval_constant_array(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [value, dims_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("ConstantArray", args, 2));
    };

    let Some(shape) = parse_non_negative_dimensions(dims_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    check_generated_shape_counts(session, &shape)?;

    Ok(constant_array_expr(value, &shape))
}

pub(crate) fn eval_pad_left(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_pad(session, args, PadSide::Left)
}

pub(crate) fn eval_pad_right(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_pad(session, args, PadSide::Right)
}

fn successive_differences(session: &mut Session, items: &[Expr]) -> Result<Vec<Expr>, EvalError> {
    if items.len() < 2 {
        return Ok(Vec::new());
    }

    let mut differences = Vec::with_capacity(items.len() - 1);
    for pair in items.windows(2) {
        differences.push(difference_expr(session, &pair[0], &pair[1])?);
    }
    Ok(differences)
}

fn difference_expr(session: &mut Session, left: &Expr, right: &Expr) -> Result<Expr, EvalError> {
    if let (Some(left), Some(right)) = (expr_as_exact_integer(left), expr_as_exact_integer(right)) {
        return Ok(Expr::integer(right - left));
    }

    session.evaluate(Expr::call(
        system_expr("Plus"),
        vec![
            right.clone(),
            Expr::call(
                system_expr("Times"),
                vec![Expr::small_integer(-1), left.clone()],
            ),
        ],
    ))
}

fn eval_pad(session: &mut Session, args: &[Expr], side: PadSide) -> Result<Expr, EvalError> {
    let symbol = side.symbol();
    let (expr, target_len, padding) = match args {
        [expr, target_len] => (expr, target_len, None),
        [expr, target_len, padding] => (expr, target_len, Some(padding)),
        _ => return Ok(session.invalid_call_with_arg_range(symbol, args, 2, 3)),
    };

    let Some(target_shape) = parse_pad_dimensions(target_len) else {
        return Ok(invalid_pad_length_call(session, symbol, args));
    };

    let Expr::Call { head, args: items } = expr else {
        return Ok(invalid_pad_normal_call(session, symbol, args));
    };

    if target_shape.len() > 1 {
        check_generated_shape_counts(session, &target_shape)?;
        let result = pad_expr_to_shape(expr, &target_shape, padding, side)
            .unwrap_or_else(|| Expr::call(system_expr(symbol), args.to_vec()));
        return Ok(result);
    }

    let target_len = target_shape[0];
    session.enforce_generated_element_limit_for_count(target_len)?;
    session.enforce_active_memory_limit_for_projected_bytes(
        generated_expr_nodes_memory_proxy_bytes(target_len.saturating_add(1)),
    )?;

    let result = pad_items(items, target_len, padding, side)
        .map(|items| Expr::call(head.as_ref().clone(), items))
        .unwrap_or_else(|| Expr::call(system_expr(symbol), args.to_vec()));
    Ok(result)
}

fn invalid_differences_order_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Differences",
        args,
        "ilsmn",
        format!(
            "Single or list of non-negative machine-sized integers expected at position 2 of {}.",
            Expr::call(system_expr("Differences"), args.to_vec())
        ),
    )
}

fn invalid_pad_normal_call(session: &mut Session, symbol: &str, args: &[Expr]) -> Expr {
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

fn invalid_pad_length_call(session: &mut Session, symbol: &str, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "ilsm",
        format!(
            "Non-negative machine-sized integer or non-empty list of non-negative machine-sized integers expected at position 2 in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    )
}

fn check_generated_shape_counts(session: &Session, shape: &[usize]) -> Result<(), EvalError> {
    let shape = shape
        .iter()
        .map(|dim| BigInt::from(*dim))
        .collect::<Vec<_>>();
    let scalar_count = shape_scalar_count(&shape);
    let node_count = shape_generated_node_count(&shape);
    let generated_count = scalar_count.max(node_count.clone());
    session.enforce_active_memory_limit_for_projected_bytes(
        generated_expr_nodes_memory_proxy_bytes(
            bigint_count_to_usize(&node_count).unwrap_or(usize::MAX),
        ),
    )?;
    session.enforce_generated_element_limit_for_big_count(&generated_count)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PadSide {
    Left,
    Right,
}

impl PadSide {
    fn symbol(self) -> &'static str {
        match self {
            Self::Left => "PadLeft",
            Self::Right => "PadRight",
        }
    }
}

fn parse_non_negative_dimensions(expr: &Expr) -> Option<Vec<usize>> {
    if let Some(dim) = expr_as_non_negative_usize(expr) {
        return Some(vec![dim]);
    }

    let items = list_items(expr)?;
    items.iter().map(expr_as_non_negative_usize).collect()
}

fn parse_pad_dimensions(expr: &Expr) -> Option<Vec<usize>> {
    let dimensions = parse_non_negative_dimensions(expr)?;
    (!dimensions.is_empty()).then_some(dimensions)
}

fn constant_array_expr(value: &Expr, shape: &[usize]) -> Expr {
    let Some((dim, rest)) = shape.split_first() else {
        return value.clone();
    };

    Expr::call(
        system_expr(heads::structure::LIST),
        (0..*dim)
            .map(|_| constant_array_expr(value, rest))
            .collect(),
    )
}

fn pad_items(
    items: &[Expr],
    target_len: usize,
    padding: Option<&Expr>,
    side: PadSide,
) -> Option<Vec<Expr>> {
    if items.len() >= target_len {
        return Some(match side {
            PadSide::Left => items[items.len() - target_len..].to_vec(),
            PadSide::Right => items[..target_len].to_vec(),
        });
    }

    let pad_count = target_len - items.len();
    let pad_items = padding_items(padding, pad_count, side)?;
    let mut result = Vec::with_capacity(target_len);

    match side {
        PadSide::Left => {
            result.extend(pad_items);
            result.extend_from_slice(items);
        }
        PadSide::Right => {
            result.extend_from_slice(items);
            result.extend(pad_items);
        }
    }

    Some(result)
}

fn padding_items(padding: Option<&Expr>, pad_count: usize, side: PadSide) -> Option<Vec<Expr>> {
    let Some(padding) = padding else {
        return Some(vec![Expr::small_integer(0); pad_count]);
    };

    if let Some(cycle) = list_items(padding) {
        if cycle.is_empty() {
            return None;
        }
        let start = match side {
            PadSide::Left => (cycle.len() - (pad_count % cycle.len())) % cycle.len(),
            PadSide::Right => 0,
        };
        return Some(
            (0..pad_count)
                .map(|index| cycle[(start + index) % cycle.len()].clone())
                .collect(),
        );
    }

    Some(vec![padding.clone(); pad_count])
}

fn pad_expr_to_shape(
    expr: &Expr,
    shape: &[usize],
    padding: Option<&Expr>,
    side: PadSide,
) -> Option<Expr> {
    let (&target_len, rest) = shape.split_first()?;
    let Expr::Call { head, args: items } = expr else {
        return None;
    };

    if rest.is_empty() {
        return pad_items(items, target_len, padding, side)
            .map(|items| Expr::call(head.as_ref().clone(), items));
    }

    let kept = kept_pad_items(items, target_len, side);
    let pad_count = target_len.saturating_sub(items.len());
    let mut result = Vec::with_capacity(target_len);

    match side {
        PadSide::Left => {
            result.extend(pad_shape_fillers(pad_count, rest, padding, side)?);
            result.extend(
                kept.into_iter()
                    .map(|item| pad_expr_to_shape(item, rest, padding, side))
                    .collect::<Option<Vec<_>>>()?,
            );
        }
        PadSide::Right => {
            result.extend(
                kept.into_iter()
                    .map(|item| pad_expr_to_shape(item, rest, padding, side))
                    .collect::<Option<Vec<_>>>()?,
            );
            result.extend(pad_shape_fillers(pad_count, rest, padding, side)?);
        }
    }

    Some(Expr::call(head.as_ref().clone(), result))
}

fn kept_pad_items(items: &[Expr], target_len: usize, side: PadSide) -> Vec<&Expr> {
    if items.len() <= target_len {
        return items.iter().collect();
    }

    match side {
        PadSide::Left => items[items.len() - target_len..].iter().collect(),
        PadSide::Right => items[..target_len].iter().collect(),
    }
}

fn pad_shape_fillers(
    pad_count: usize,
    shape: &[usize],
    padding: Option<&Expr>,
    side: PadSide,
) -> Option<Vec<Expr>> {
    let empty = Expr::call(system_expr(heads::structure::LIST), Vec::new());
    (0..pad_count)
        .map(|_| pad_expr_to_shape(&empty, shape, padding, side))
        .collect()
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

fn generated_expr_nodes_memory_proxy_bytes(count: usize) -> usize {
    count.saturating_mul(std::mem::size_of::<Expr>())
}
