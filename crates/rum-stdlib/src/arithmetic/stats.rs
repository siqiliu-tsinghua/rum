use rum_core::SharedArgs;
use rum_core::{Atom, BigInt, Expr, IntValue, Rational};
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{
    EvalError, Session, ToPrimitive, call_items_with_head, compare_expr_canonical,
    expr_as_exact_integer, expr_as_exact_rational, system_expr,
};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

use crate::structure_ops::stable_groups::StableCounts;

const COMMONEST_INDEX_THRESHOLD: usize = 64;

pub(crate) fn eval_median(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Median", args, 1));
    };

    let Some(items) = call_items_with_head(expr, &system_expr("List")) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if items.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Median",
            args,
            "empty",
            "Median cannot aggregate an empty list.",
        ));
    }

    let mut sorted = items.to_vec();
    sorted.sort_by(compare_expr_canonical);
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        return Ok(sorted[middle].clone());
    }

    average_pair(session, sorted[middle - 1].clone(), sorted[middle].clone())
}

fn enforce_commonest_memory_constraint(session: &Session, items: &[Expr]) -> Result<(), EvalError> {
    if session.active_memory_limit().is_none() {
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    }

    let projected_bytes = items.iter().fold(
        std::mem::size_of::<Expr>()
            .saturating_add(projected_expr_memory_proxy_bytes(&system_expr("List"))),
        |total, item| total.saturating_add(projected_expr_memory_proxy_bytes(item)),
    );
    session.enforce_active_memory_limit_for_projected_bytes(projected_bytes)
}

fn projected_expr_memory_proxy_bytes(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(Atom::Integer(IntValue::Small(_))) => std::mem::size_of::<Expr>(),
        Expr::Atom(Atom::Integer(IntValue::Big(value))) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<BigInt>())
            .saturating_add(projected_bigint_payload_bytes(value)),
        Expr::Atom(Atom::Rational(value)) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<Rational>())
            .saturating_add(projected_bigint_payload_bytes(value.numerator()))
            .saturating_add(projected_bigint_payload_bytes(value.denominator())),
        Expr::Atom(Atom::String(value)) => std::mem::size_of::<Expr>().saturating_add(value.len()),
        Expr::Atom(Atom::Symbol(symbol)) => std::mem::size_of::<Expr>()
            .saturating_add(symbol.context().len())
            .saturating_add(symbol.name().len()),
        Expr::Call { head, args, .. } => args.iter().fold(
            std::mem::size_of::<Expr>().saturating_add(projected_expr_memory_proxy_bytes(head)),
            |total, arg| total.saturating_add(projected_expr_memory_proxy_bytes(arg)),
        ),
    }
}

fn projected_bigint_payload_bytes(value: &BigInt) -> usize {
    let bytes = u128::from(value.bits()).saturating_add(7) / 8;
    usize::try_from(bytes.saturating_add(1)).unwrap_or(usize::MAX)
}

pub(crate) fn eval_commonest(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, n_expr) = match args {
        [expr] => (expr, None),
        [expr, n_expr] => (expr, Some(n_expr)),
        _ => return Ok(session.invalid_call_with_exact_arg_count("Commonest", args, 1)),
    };

    let Some(items) = call_items_with_head(expr, &system_expr("List")) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    enforce_commonest_memory_constraint(session, items)?;

    let mut counts = if items.len() >= COMMONEST_INDEX_THRESHOLD {
        StableCounts::collect(items.iter().cloned()).into_entries()
    } else {
        linear_counts(items)
    };

    if let Some(n_expr) = n_expr {
        let Some(n) = expr_as_exact_integer(n_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if n < BigInt::from(0_u8) {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        let take_count = if n > BigInt::from(counts.len()) {
            counts.len()
        } else {
            n.to_usize()
                .expect("count bounded by frequency class count should fit usize")
        };
        counts.sort_by(|(_, left_count), (_, right_count)| right_count.cmp(left_count));
        let mut common = Vec::with_capacity(take_count);
        common.extend(counts.into_iter().take(take_count).map(|(item, _)| item));
        return Ok(Expr::call(system_expr("List"), common));
    }

    let (max_count, common_count) = counts.iter().fold(
        (0usize, 0usize),
        |(max_count, common_count), (_, count)| match count.cmp(&max_count) {
            std::cmp::Ordering::Greater => (*count, 1),
            std::cmp::Ordering::Equal => (max_count, common_count.saturating_add(1)),
            std::cmp::Ordering::Less => (max_count, common_count),
        },
    );
    let mut common = Vec::with_capacity(common_count);
    common.extend(
        counts
            .into_iter()
            .filter_map(|(item, count)| (count == max_count).then_some(item)),
    );
    Ok(Expr::call(system_expr("List"), common))
}

fn linear_counts(items: &[Expr]) -> Vec<(Expr, usize)> {
    let mut counts = Vec::new();
    for item in items {
        if let Some((_, count)) = counts.iter_mut().find(|(existing, _)| existing == item) {
            *count += 1;
        } else {
            counts.push((item.clone(), 1));
        }
    }
    counts
}

pub(crate) fn eval_quantile(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr, q_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Quantile", args, 2));
    };

    let Some(items) = call_items_with_head(expr, &system_expr("List")) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if items.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Quantile",
            args,
            "empty",
            "Quantile cannot aggregate an empty list.",
        ));
    }

    if let Some(q_items) = call_items_with_head(q_expr, &system_expr("List")) {
        let mut ranks = Vec::with_capacity(q_items.len());
        for q_expr in q_items {
            let Some(q) = expr_as_exact_rational(q_expr) else {
                return Ok(session.invalid_call_with_message(
                    "Quantile",
                    args,
                    "range",
                    "Quantile rank parameter must be an exact number between 0 and 1.",
                ));
            };
            if q.numerator() < &BigInt::from(0_u8) || q.numerator() > q.denominator() {
                return Ok(session.invalid_call_with_message(
                    "Quantile",
                    args,
                    "range",
                    "Quantile rank parameter must be an exact number between 0 and 1.",
                ));
            }
            ranks.push(q);
        }

        let mut sorted = items.to_vec();
        sorted.sort_by(compare_expr_canonical);
        return Ok(Expr::call(
            system_expr("List"),
            ranks
                .into_iter()
                .map(|q| {
                    let rank = nearest_rank_index(&q, sorted.len());
                    sorted[rank].clone()
                })
                .collect(),
        ));
    }

    let Some(q) = expr_as_exact_rational(q_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if q.numerator() < &BigInt::from(0_u8) || q.numerator() > q.denominator() {
        return Ok(session.invalid_call_with_message(
            "Quantile",
            args,
            "range",
            "Quantile rank parameter must be an exact number between 0 and 1.",
        ));
    }

    let mut sorted = items.to_vec();
    sorted.sort_by(compare_expr_canonical);
    let rank = nearest_rank_index(&q, sorted.len());
    Ok(sorted[rank].clone())
}

fn average_pair(session: &mut Session, lhs: Expr, rhs: Expr) -> Result<Expr, EvalError> {
    let sum = Expr::call(system_expr("Plus"), vec![lhs, rhs]);
    let half = Expr::call(
        system_expr("Divide"),
        vec![sum, Expr::integer(BigInt::from(2_u8))],
    );
    let evaluated = session.evaluate(half);
    rum_runtime::resume_or_suspend!(
        session,
        evaluated,
        ok(value) => Ok(value),
        suspend => AveragePairSuspensionFrame,
    )
}

fn nearest_rank_index(q: &Rational, len: usize) -> usize {
    let len_integer = BigInt::from(len);
    let raw = q.numerator() * len_integer;
    let mut rank = ceiling_div_non_negative(&raw, q.denominator());
    if rank < BigInt::from(1_u8) {
        rank = BigInt::from(1_u8);
    }
    if rank > BigInt::from(len) {
        rank = BigInt::from(len);
    }
    rank.to_usize()
        .expect("clamped nearest-rank index should fit usize")
        - 1
}

fn ceiling_div_non_negative(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    if numerator % denominator == BigInt::from(0_u8) {
        quotient
    } else {
        quotient + BigInt::from(1_u8)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct AveragePairSuspensionFrame;

impl rum_runtime::kernel_api::SuspensionFrame for AveragePairSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
