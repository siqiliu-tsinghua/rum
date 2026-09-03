use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_permutations(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [list] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Permutations", args, 1));
    };
    let Some(items) = list_items(list) else {
        return Ok(invalid_permutations_normal_call(session, args));
    };

    let mut current = items.to_vec();
    current.sort_by(compare_expr_canonical);

    let permutation_count = distinct_permutation_count(&current);
    session.enforce_generated_element_limit_for_big_count(&permutation_count)?;
    session.enforce_active_memory_limit_for_projected_bytes(
        generated_expr_nodes_memory_proxy_bytes(
            bigint_count_to_usize(&permutation_count).unwrap_or(usize::MAX),
        ),
    )?;

    let mut permutations = Vec::new();
    loop {
        permutations.push(Expr::call(
            system_expr(heads::structure::LIST),
            current.clone(),
        ));
        if !next_permutation(&mut current) {
            break;
        }
    }

    Ok(Expr::call(
        system_expr(heads::structure::LIST),
        permutations,
    ))
}

pub(crate) fn eval_subsets(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, size_spec) = match args {
        [expr] => (expr, None),
        [expr, size_spec] => (expr, Some(size_spec)),
        _ => return Ok(session.invalid_call_with_arg_range("Subsets", args, 1, 2)),
    };

    let Expr::Call {
        head: expr_head,
        args: items,
        ..
    } = expr
    else {
        return Ok(invalid_subsets_normal_call(session, args));
    };

    let Some(sizes) = parse_subset_sizes(size_spec, items.len()) else {
        return Ok(invalid_subsets_size_spec_call(session, args));
    };

    let subset_count = subset_sizes_count(items.len(), &sizes);
    session.enforce_generated_element_limit_for_big_count(&subset_count)?;
    session.enforce_active_memory_limit_for_projected_bytes(
        generated_expr_nodes_memory_proxy_bytes(
            bigint_count_to_usize(&subset_count).unwrap_or(usize::MAX),
        ),
    )?;

    let mut subsets = Vec::new();
    for size in sizes {
        append_subsets_of_size(
            expr_head.as_ref(),
            items,
            size,
            0,
            &mut Vec::new(),
            &mut subsets,
        );
    }

    Ok(Expr::call(system_expr(heads::structure::LIST), subsets))
}

pub(crate) fn eval_tuples(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [factors_expr] => eval_cartesian_tuples(session, head, args, factors_expr),
        [expr, length_expr] => eval_repeated_tuples(session, head, args, expr, length_expr),
        _ => Ok(session.invalid_call_with_arg_range("Tuples", args, 1, 2)),
    }
}

pub(crate) fn eval_partition(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, size_expr, offset_expr, alignment_expr, padding_expr) = match args {
        [expr, size_expr] => (expr, size_expr, None, None, None),
        [expr, size_expr, offset_expr] => (expr, size_expr, Some(offset_expr), None, None),
        [expr, size_expr, offset_expr, alignment_expr] => (
            expr,
            size_expr,
            Some(offset_expr),
            Some(alignment_expr),
            None,
        ),
        [expr, size_expr, offset_expr, alignment_expr, padding_expr] => (
            expr,
            size_expr,
            Some(offset_expr),
            Some(alignment_expr),
            Some(padding_expr),
        ),
        _ => {
            if args.len() == 1 {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
            return Ok(session.invalid_call_with_arg_range("Partition", args, 2, 5));
        }
    };

    let Some(size) = expr_as_positive_usize(size_expr) else {
        return Ok(invalid_partition_positive_integer_call(session, args, 2));
    };
    let offset = match offset_expr {
        Some(offset_expr) => {
            let Some(offset) = expr_as_positive_usize(offset_expr) else {
                return Ok(invalid_partition_positive_integer_call(session, args, 3));
            };
            offset
        }
        None => size,
    };
    let alignment = match alignment_expr {
        Some(alignment_expr) => {
            let Some(alignment) = parse_partition_alignment(alignment_expr, size) else {
                return Ok(invalid_partition_alignment_call(session, args, size));
            };
            Some(alignment)
        }
        None => None,
    };
    let Expr::Call {
        head: expr_head,
        args: items,
        ..
    } = expr
    else {
        return Ok(invalid_partition_normal_call(session, args));
    };

    if let Some(alignment) = alignment {
        let padding = match padding_expr {
            Some(padding_expr) => parse_partition_padding(padding_expr),
            None => PartitionPadding::Cycle(items),
        };
        let block_count = aligned_partition_block_count(items.len(), size, offset, alignment);
        session.enforce_generated_element_limit_for_count(block_count)?;
        session.enforce_active_memory_limit_for_projected_bytes(
            generated_expr_nodes_memory_proxy_bytes(block_count.saturating_mul(size + 1)),
        )?;

        let blocks =
            aligned_partition_blocks(expr_head.as_ref(), items, size, offset, alignment, padding);
        return Ok(Expr::call(expr_head.as_ref().clone(), blocks));
    }

    let block_count = partition_block_count(items.len(), size, offset);
    session.enforce_generated_element_limit_for_count(block_count)?;
    session.enforce_active_memory_limit_for_projected_bytes(
        generated_expr_nodes_memory_proxy_bytes(block_count.saturating_mul(size + 1)),
    )?;

    let mut blocks = Vec::with_capacity(block_count);
    let mut start = 0_usize;
    while start.saturating_add(size) <= items.len() {
        blocks.push(Expr::call(
            expr_head.as_ref().clone(),
            items[start..start + size].to_vec(),
        ));
        start = start.saturating_add(offset);
    }

    Ok(Expr::call(expr_head.as_ref().clone(), blocks))
}

fn eval_repeated_tuples(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
    expr: &Expr,
    length_expr: &Expr,
) -> Result<Expr, EvalError> {
    let Some(length) = expr_as_non_negative_usize(length_expr) else {
        return Ok(invalid_tuples_length_call(session, args));
    };

    let Expr::Call {
        head: tuple_head,
        args: items,
        ..
    } = expr
    else {
        return Ok(invalid_tuples_normal_call(session, args, "1"));
    };

    let tuple_count = repeated_tuple_count(items.len(), length);
    session.enforce_generated_element_limit_for_big_count(&tuple_count)?;
    session.enforce_active_memory_limit_for_projected_bytes(
        generated_expr_nodes_memory_proxy_bytes(
            bigint_count_to_usize(&tuple_count)
                .unwrap_or(usize::MAX)
                .saturating_mul(length + 1),
        ),
    )?;

    let mut tuples = Vec::new();
    append_repeated_tuples(
        tuple_head.as_ref(),
        items,
        length,
        &mut Vec::new(),
        &mut tuples,
    );
    Ok(Expr::call(system_expr(heads::structure::LIST), tuples))
}

fn eval_cartesian_tuples(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
    factors_expr: &Expr,
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: tuple_head,
        args: factor_exprs,
        ..
    } = factors_expr
    else {
        return Ok(invalid_tuples_normal_call(session, args, "1"));
    };

    let mut factors = Vec::with_capacity(factor_exprs.len());
    for (index, factor) in factor_exprs.iter().enumerate() {
        let Expr::Call { args, .. } = factor else {
            return Ok(invalid_tuples_normal_call(
                session,
                args,
                &format!("{{1, {}}}", index + 1),
            ));
        };
        factors.push(args.as_slice());
    }

    let tuple_count = cartesian_tuple_count(&factors);
    session.enforce_generated_element_limit_for_big_count(&tuple_count)?;
    session.enforce_active_memory_limit_for_projected_bytes(
        generated_expr_nodes_memory_proxy_bytes(
            bigint_count_to_usize(&tuple_count)
                .unwrap_or(usize::MAX)
                .saturating_mul(factors.len() + 1),
        ),
    )?;

    let mut tuples = Vec::new();
    append_cartesian_tuples(
        tuple_head.as_ref(),
        &factors,
        0,
        &mut Vec::new(),
        &mut tuples,
    );
    Ok(Expr::call(system_expr(heads::structure::LIST), tuples))
}

fn invalid_subsets_normal_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Subsets",
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr("Subsets"), args.to_vec())
        ),
    )
}

fn invalid_permutations_normal_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Permutations",
        args,
        "normal",
        format!(
            "List expected at position 1 in {}.",
            Expr::call(system_expr("Permutations"), args.to_vec())
        ),
    )
}

fn invalid_subsets_size_spec_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
            "Subsets",
            args,
            "nninfseq",
            format!(
                "Position 2 of {} must be All, Infinity, nmax, {{nmin}}, {{nmin, nmax}} or {{nmin, nmax, dn}}, where nmin is a non-negative integer, nmax is non-negative integer or Infinity and dn is a nonzero integer.",
                Expr::call(system_expr("Subsets"), args.to_vec())
            ),
        )
}

fn invalid_tuples_normal_call(session: &mut Session, args: &[Expr], position: &str) -> Expr {
    session.invalid_call_with_message(
        "Tuples",
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position {position} in {}.",
            Expr::call(system_expr("Tuples"), args.to_vec())
        ),
    )
}

fn invalid_tuples_length_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Tuples",
        args,
        "ilsmn",
        format!(
            "Single or list of non-negative machine-sized integers expected at position 2 of {}.",
            Expr::call(system_expr("Tuples"), args.to_vec())
        ),
    )
}

fn invalid_partition_normal_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Partition",
        args,
        "npart",
        format!("The expression {} cannot be partitioned.", args[0]),
    )
}

fn invalid_partition_positive_integer_call(
    session: &mut Session,
    args: &[Expr],
    position: usize,
) -> Expr {
    session.invalid_call_with_message(
        "Partition",
        args,
        "ilsmp",
        format!(
            "Positive machine-sized integer expected at position {position} of {}.",
            Expr::call(system_expr("Partition"), args.to_vec())
        ),
    )
}

fn invalid_partition_alignment_call(session: &mut Session, args: &[Expr], size: usize) -> Expr {
    session.invalid_call_with_message(
        "Partition",
        args,
        "align",
        format!(
            "Position 4 of {} must be {{kL, kR}}, where kL and kR are positive machine-sized integers not greater than the block size {size}.",
            Expr::call(system_expr("Partition"), args.to_vec())
        ),
    )
}

fn parse_subset_sizes(size_spec: Option<&Expr>, item_count: usize) -> Option<Vec<usize>> {
    match size_spec {
        None => Some((0..=item_count).collect()),
        Some(expr) => {
            if is_system_symbol_name(expr, heads::structure::ALL)
                || is_system_symbol_name(expr, heads::structure::INFINITY)
            {
                return Some((0..=item_count).collect());
            }

            if let Some(max) = expr_as_non_negative_usize(expr) {
                return Some((0..=max.min(item_count)).collect());
            }

            let items = list_items(expr)?;
            match items {
                [exact] => {
                    let exact = expr_as_non_negative_usize(exact)?;
                    Some((exact <= item_count).then_some(exact).into_iter().collect())
                }
                [min, max] => {
                    let min = expr_as_non_negative_usize(min)?;
                    let max = parse_subset_max(max, item_count)?;
                    Some(ascending_subset_sizes(min, max))
                }
                [min, max, step] => {
                    let min = expr_as_non_negative_usize(min)?;
                    let max = parse_subset_max(max, item_count)?;
                    let step = expr_as_signed_isize(step)?;
                    (step != 0).then(|| stepped_subset_sizes(min, max, step))
                }
                _ => None,
            }
        }
    }
}

fn parse_subset_max(expr: &Expr, item_count: usize) -> Option<usize> {
    if is_system_symbol_name(expr, heads::structure::INFINITY) {
        return Some(item_count);
    }

    expr_as_non_negative_usize(expr).map(|max| max.min(item_count))
}

fn ascending_subset_sizes(min: usize, max: usize) -> Vec<usize> {
    if min > max {
        return Vec::new();
    }

    (min..=max).collect()
}

fn stepped_subset_sizes(min: usize, max: usize, step: isize) -> Vec<usize> {
    if min > max {
        return Vec::new();
    }

    if step > 0 {
        return (min..=max).step_by(step as usize).collect();
    }

    let step = step.unsigned_abs();
    let mut sizes = Vec::new();
    let mut size = max;
    loop {
        if size < min {
            break;
        }
        sizes.push(size);
        let Some(next) = size.checked_sub(step) else {
            break;
        };
        size = next;
    }
    sizes
}

fn subset_sizes_count(item_count: usize, sizes: &[usize]) -> BigInt {
    sizes
        .iter()
        .map(|size| binomial_count(item_count, *size))
        .sum()
}

fn distinct_permutation_count(sorted_items: &[Expr]) -> BigInt {
    let mut count = factorial_count(sorted_items.len());
    let mut run_start = 0;

    while run_start < sorted_items.len() {
        let mut run_end = run_start + 1;
        while run_end < sorted_items.len()
            && compare_expr_canonical(&sorted_items[run_start], &sorted_items[run_end]).is_eq()
        {
            run_end += 1;
        }
        count /= factorial_count(run_end - run_start);
        run_start = run_end;
    }

    count
}

fn factorial_count(n: usize) -> BigInt {
    (2..=n).map(BigInt::from).product()
}

pub(crate) fn next_permutation(items: &mut [Expr]) -> bool {
    let Some(pivot) = (0..items.len().saturating_sub(1))
        .rev()
        .find(|index| compare_expr_canonical(&items[*index], &items[*index + 1]).is_lt())
    else {
        return false;
    };
    let successor = (pivot + 1..items.len())
        .rev()
        .find(|index| compare_expr_canonical(&items[*index], &items[pivot]).is_gt())
        .expect("an ascending pivot has a greater successor");

    items.swap(pivot, successor);
    items[pivot + 1..].reverse();
    true
}

fn binomial_count(item_count: usize, size: usize) -> BigInt {
    if size > item_count {
        return BigInt::from(0_u8);
    }

    let size = size.min(item_count - size);
    let mut count = BigInt::from(1_u8);
    for step in 1..=size {
        count *= BigInt::from(item_count - size + step);
        count /= BigInt::from(step);
    }
    count
}

fn append_subsets_of_size(
    subset_head: &Expr,
    items: &[Expr],
    size: usize,
    start: usize,
    current: &mut Vec<Expr>,
    subsets: &mut Vec<Expr>,
) {
    if current.len() == size {
        subsets.push(Expr::call(subset_head.clone(), current.clone()));
        return;
    }

    let needed = size - current.len();
    let available = items.len().saturating_sub(start);
    if needed > available {
        return;
    }

    let last_start = items.len() - needed;
    for index in start..=last_start {
        current.push(items[index].clone());
        append_subsets_of_size(subset_head, items, size, index + 1, current, subsets);
        current.pop();
    }
}

fn repeated_tuple_count(item_count: usize, length: usize) -> BigInt {
    let mut count = BigInt::from(1_u8);
    for _ in 0..length {
        count *= BigInt::from(item_count);
    }
    count
}

fn cartesian_tuple_count(factors: &[&[Expr]]) -> BigInt {
    factors
        .iter()
        .map(|items| BigInt::from(items.len()))
        .product()
}

fn append_repeated_tuples(
    tuple_head: &Expr,
    items: &[Expr],
    remaining: usize,
    current: &mut Vec<Expr>,
    tuples: &mut Vec<Expr>,
) {
    if remaining == 0 {
        tuples.push(Expr::call(tuple_head.clone(), current.clone()));
        return;
    }

    for item in items {
        current.push(item.clone());
        append_repeated_tuples(tuple_head, items, remaining - 1, current, tuples);
        current.pop();
    }
}

fn append_cartesian_tuples(
    tuple_head: &Expr,
    factors: &[&[Expr]],
    index: usize,
    current: &mut Vec<Expr>,
    tuples: &mut Vec<Expr>,
) {
    let Some(items) = factors.get(index) else {
        tuples.push(Expr::call(tuple_head.clone(), current.clone()));
        return;
    };

    for item in *items {
        current.push(item.clone());
        append_cartesian_tuples(tuple_head, factors, index + 1, current, tuples);
        current.pop();
    }
}

fn partition_block_count(item_count: usize, size: usize, offset: usize) -> usize {
    if item_count < size {
        0
    } else {
        ((item_count - size) / offset) + 1
    }
}

#[derive(Clone, Copy, Debug)]
struct PartitionAlignment {
    left: usize,
    right: usize,
}

#[derive(Clone, Copy)]
enum PartitionPadding<'a> {
    Ragged,
    Single(&'a Expr),
    Cycle(&'a [Expr]),
}

fn parse_partition_alignment(expr: &Expr, size: usize) -> Option<PartitionAlignment> {
    let [left, right] = list_items(expr)? else {
        return None;
    };
    let left = expr_as_positive_usize(left)?;
    let right = expr_as_positive_usize(right)?;
    (left <= size && right <= size).then_some(PartitionAlignment { left, right })
}

fn parse_partition_padding(expr: &Expr) -> PartitionPadding<'_> {
    match list_items(expr) {
        Some([]) => PartitionPadding::Ragged,
        Some(items) => PartitionPadding::Cycle(items),
        None => PartitionPadding::Single(expr),
    }
}

fn aligned_partition_block_count(
    item_count: usize,
    size: usize,
    offset: usize,
    alignment: PartitionAlignment,
) -> usize {
    if item_count == 0 {
        return 0;
    }

    let first_start = aligned_partition_first_start(alignment);
    let last_acceptable_start = item_count as i128 - alignment.right as i128;
    if first_start > last_acceptable_start {
        return 0;
    }

    let first_containing_last_start = item_count as i128 - size as i128;
    let stop_start = if first_start >= first_containing_last_start {
        first_start
    } else {
        let distance = first_containing_last_start - first_start;
        let steps = (distance + offset as i128 - 1) / offset as i128;
        first_start + steps * offset as i128
    };
    let final_start = if stop_start <= last_acceptable_start {
        stop_start
    } else {
        let steps = (last_acceptable_start - first_start) / offset as i128;
        first_start + steps * offset as i128
    };

    let count = ((final_start - first_start) / offset as i128) + 1;
    usize::try_from(count).unwrap_or(usize::MAX)
}

fn aligned_partition_blocks(
    block_head: &Expr,
    items: &[Expr],
    size: usize,
    offset: usize,
    alignment: PartitionAlignment,
    padding: PartitionPadding<'_>,
) -> Vec<Expr> {
    let block_count = aligned_partition_block_count(items.len(), size, offset, alignment);
    let mut blocks = Vec::with_capacity(block_count);
    if block_count == 0 {
        return blocks;
    }

    let mut start = aligned_partition_first_start(alignment);
    for _ in 0..block_count {
        let block = aligned_partition_block(block_head, items, size, start, padding);
        blocks.push(block);
        start += offset as i128;
    }
    blocks
}

fn aligned_partition_first_start(alignment: PartitionAlignment) -> i128 {
    1_i128 - alignment.left as i128
}

fn aligned_partition_block(
    block_head: &Expr,
    items: &[Expr],
    size: usize,
    start: i128,
    padding: PartitionPadding<'_>,
) -> Expr {
    let mut block_items = Vec::with_capacity(size);
    for offset in 0..size {
        let index = start + offset as i128;
        if let Some(item) = partition_item_at(items, index) {
            block_items.push(item.clone());
            continue;
        }

        match padding {
            PartitionPadding::Ragged => {}
            PartitionPadding::Single(padding) => block_items.push(padding.clone()),
            PartitionPadding::Cycle(cycle) => {
                block_items.push(cyclic_partition_padding_item(cycle, index, items.len()))
            }
        }
    }
    Expr::call(block_head.clone(), block_items)
}

fn partition_item_at(items: &[Expr], index: i128) -> Option<&Expr> {
    if !(0..items.len() as i128).contains(&index) {
        return None;
    }

    items.get(usize::try_from(index).ok()?)
}

fn cyclic_partition_padding_item(cycle: &[Expr], index: i128, item_count: usize) -> Expr {
    let cycle_len = cycle.len() as i128;
    let cycle_index = if index < 0 {
        index.rem_euclid(cycle_len)
    } else {
        (index - item_count as i128).rem_euclid(cycle_len)
    };
    cycle[cycle_index as usize].clone()
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    expr_as_non_negative_usize(expr).filter(|value| *value > 0)
}

fn bigint_count_to_usize(count: &BigInt) -> Option<usize> {
    count.to_string().parse().ok()
}

fn generated_expr_nodes_memory_proxy_bytes(count: usize) -> usize {
    count.saturating_mul(std::mem::size_of::<Expr>())
}
