use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_take_list(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr, counts] = args else {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_exact_arg_count("TakeList", args, 2));
    };

    let Expr::Call { .. } = expr else {
        return Ok(invalid_sequence_normal_expression_call(
            session, "TakeList", args,
        ));
    };

    let Some(counts) = parse_take_list_counts(counts) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let Some(chunks) = apply_take_list_counts(expr, &counts) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::call(system_expr("List"), chunks))
}

fn invalid_sequence_normal_expression_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
) -> Expr {
    session.push_message(
        symbol,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    );
    Expr::call(system_expr(symbol), args.to_vec())
}

fn parse_take_list_counts(counts_expr: &Expr) -> Option<Vec<usize>> {
    let Expr::Call { head, args, .. } = counts_expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    args.iter()
        .map(|arg| {
            let count = expr_as_signed_isize(arg)?;
            (count >= 0).then_some(count as usize)
        })
        .collect()
}

fn apply_take_list_counts(expr: &Expr, counts: &[usize]) -> Option<Vec<Expr>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    let mut offset = 0_usize;
    let mut chunks = Vec::with_capacity(counts.len());
    for count in counts {
        let end = offset.checked_add(*count)?;
        if end > args.len() {
            return None;
        }

        chunks.push(Expr::call(
            head.as_ref().clone(),
            args[offset..end].to_vec(),
        ));
        offset = end;
    }

    Some(chunks)
}
