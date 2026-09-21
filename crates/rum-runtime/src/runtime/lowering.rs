use super::*;

mod binary;

#[cfg(test)]
mod tests;

use binary::lower_binary_sugar;

use std::cell::Cell;

const RECURSIVE_LOWERING_DEPTH: usize = 16;
const MAX_LOWERING_DEPTH: usize = 5500;

thread_local! {
    // All recursive sugar helpers share this budget, including the existing
    // iterative binary spines. Leaves do not consume native recursion fuel.
    static LOWERING_FRONT_DEPTH: Cell<usize> = const { Cell::new(RECURSIVE_LOWERING_DEPTH) };
    static LOWERING_TAIL_DEPTH: Cell<usize> = const {
        Cell::new(MAX_LOWERING_DEPTH - RECURSIVE_LOWERING_DEPTH)
    };
    static LOWERING_USED_TAIL: Cell<bool> = const { Cell::new(false) };
}

struct RestoreLoweringDepth(usize);

impl Drop for RestoreLoweringDepth {
    fn drop(&mut self) {
        LOWERING_FRONT_DEPTH.set(self.0);
    }
}

pub fn lower_syntax_expr(expr: &SyntaxExpr) -> Result<Expr, EvalError> {
    lower_syntax_expr_with_front_depth(
        expr,
        &mut |raw| resolve_unqualified_symbol(raw).or_else(|| Some(symbol_from_raw_name(raw))),
        RECURSIVE_LOWERING_DEPTH,
    )
    .map(LoweredSyntax::into_expr)
}

enum LoweredSyntax {
    Front(Expr),
    Tail(Expr),
}

impl LoweredSyntax {
    fn into_expr(self) -> Expr {
        match self {
            Self::Front(value) | Self::Tail(value) => value,
        }
    }
}

fn lower_syntax_expr_with_front_depth<F>(
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
    front_depth: usize,
) -> Result<LoweredSyntax, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    struct RestoreFront {
        _depth: RestoreLoweringDepth,
        tail_depth: usize,
        used_tail: bool,
    }
    impl Drop for RestoreFront {
        fn drop(&mut self) {
            LOWERING_TAIL_DEPTH.set(self.tail_depth);
            LOWERING_USED_TAIL.set(self.used_tail);
        }
    }
    let front_depth = front_depth.min(RECURSIVE_LOWERING_DEPTH);
    let restore = RestoreFront {
        _depth: RestoreLoweringDepth(LOWERING_FRONT_DEPTH.replace(front_depth)),
        tail_depth: LOWERING_TAIL_DEPTH.replace(MAX_LOWERING_DEPTH - front_depth),
        used_tail: LOWERING_USED_TAIL.replace(false),
    };
    let value = lower_syntax_expr_with(expr, resolve_symbol)?;
    let lowered = if LOWERING_USED_TAIL.get() {
        LoweredSyntax::Tail(value)
    } else {
        LoweredSyntax::Front(value)
    };
    drop(restore);
    Ok(lowered)
}

// Keep leaf lowering in the caller: the recursive arm's traversal locals and
// unwind guard must not enlarge every symbol/integer argument's native frame.
#[inline(always)]
pub(super) fn lower_syntax_expr_with<F>(
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    match &expr.kind {
        SyntaxExprKind::Integer(value) => lower_integer_atom(value),
        SyntaxExprKind::String(value) => Ok(Expr::string(value.clone())),
        SyntaxExprKind::Symbol(value) => Ok(lower_symbol(value, resolve_symbol)),
        SyntaxExprKind::Slot(spec) => lower_slot_sugar(spec),
        SyntaxExprKind::Blank { kind, head: None } => {
            lower_blank_sugar(*kind, None, resolve_symbol)
        }
        _ => lower_recursive_syntax(expr, resolve_symbol),
    }
}

#[inline(never)]
fn lower_recursive_syntax<F>(expr: &SyntaxExpr, resolve_symbol: &mut F) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let remaining = LOWERING_FRONT_DEPTH.get();
    if remaining == 0 {
        return lower_syntax_expr_iterative(expr, resolve_symbol);
    }
    LOWERING_FRONT_DEPTH.set(remaining - 1);
    let _restore = RestoreLoweringDepth(remaining);

    // Keep a sharp boundary here:
    // - direct syntax that already matches runtime Expr shape should preserve its head
    // - parser-only sugar should be rewritten into canonical system-head forms
    match &expr.kind {
        SyntaxExprKind::List(items) => lower_list_sugar(items, resolve_symbol),
        SyntaxExprKind::Association(items) => lower_association_sugar(items, resolve_symbol),
        SyntaxExprKind::Blank { kind, head } => {
            lower_blank_sugar(*kind, head.as_deref(), resolve_symbol)
        }
        SyntaxExprKind::Pattern { name, pattern } => {
            lower_pattern_sugar(name, pattern, resolve_symbol)
        }
        SyntaxExprKind::Prefix { op, expr } => lower_prefix_sugar(*op, expr, resolve_symbol),
        SyntaxExprKind::Binary { op, lhs, rhs } => {
            lower_binary_sugar(*op, lhs, rhs, resolve_symbol)
        }
        SyntaxExprKind::ComparisonChain {
            operands,
            operators,
        } => binary::lower_comparison_chain(operands, operators, expr.span(), resolve_symbol),
        SyntaxExprKind::Postfix { op, expr } => lower_postfix_sugar(*op, expr, resolve_symbol),
        SyntaxExprKind::Call { head, args } => {
            lower_head_preserving_call(head, args, resolve_symbol)
        }
        SyntaxExprKind::Integer(_)
        | SyntaxExprKind::String(_)
        | SyntaxExprKind::Symbol(_)
        | SyntaxExprKind::Slot(_) => unreachable!("leaves handled before recursion guard"),
    }
}

fn lower_integer_atom(value: &str) -> Result<Expr, EvalError> {
    if let Some(rational) = lower_decimal_atom(value)? {
        return Ok(rational);
    }

    let bigint =
        BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| EvalError::InvalidInteger {
            literal: value.to_owned(),
        })?;

    Ok(Expr::integer(bigint))
}

fn lower_decimal_atom(value: &str) -> Result<Option<Expr>, EvalError> {
    let (mantissa, exponent) = split_decimal_exponent(value)?;
    let Some((whole, fractional)) = mantissa.split_once('.') else {
        return Ok(None);
    };
    if (whole.is_empty() && fractional.is_empty())
        || !whole.chars().all(|ch| ch.is_ascii_digit())
        || !fractional.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(EvalError::InvalidInteger {
            literal: value.to_owned(),
        });
    }

    let whole = if whole.is_empty() { "0" } else { whole };
    let numerator_text = format!("{whole}{fractional}");
    let mut numerator = BigInt::parse_bytes(numerator_text.as_bytes(), 10).ok_or_else(|| {
        EvalError::InvalidInteger {
            literal: value.to_owned(),
        }
    })?;
    let mut denominator = BigInt::from(1_u8);
    for _ in fractional.chars() {
        denominator *= BigInt::from(10_u8);
    }

    if let Some(exponent) = exponent {
        if exponent >= 0 {
            multiply_by_power_of_ten(&mut numerator, exponent as u64);
        } else {
            multiply_by_power_of_ten(&mut denominator, exponent.unsigned_abs());
        }
    }

    Ok(Some(Expr::rational(
        Rational::new(numerator, denominator)
            .expect("decimal literal denominator is always positive"),
    )))
}

fn split_decimal_exponent(value: &str) -> Result<(&str, Option<i64>), EvalError> {
    let Some(exponent_index) = value.find(|ch| matches!(ch, 'e' | 'E')) else {
        return Ok((value, None));
    };

    let exponent =
        value[exponent_index + 1..]
            .parse::<i64>()
            .map_err(|_| EvalError::InvalidInteger {
                literal: value.to_owned(),
            })?;

    Ok((&value[..exponent_index], Some(exponent)))
}

fn multiply_by_power_of_ten(value: &mut BigInt, exponent: u64) {
    for _ in 0..exponent {
        *value *= BigInt::from(10_u8);
    }
}

fn lower_syntax_args<F>(args: &[SyntaxExpr], resolve_symbol: &mut F) -> Result<Vec<Expr>, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    args.iter()
        .map(|arg| lower_syntax_expr_with(arg, resolve_symbol))
        .collect()
}

fn lower_head_preserving_call<F>(
    head: &SyntaxExpr,
    args: &[SyntaxExpr],
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered_head = match &head.kind {
        SyntaxExprKind::Symbol(raw) => lower_symbol(raw, resolve_symbol),
        _ => lower_syntax_expr_with(head, resolve_symbol)?,
    };
    let lowered_args = lower_syntax_args(args, resolve_symbol)?;
    Ok(Expr::call(lowered_head, lowered_args))
}

fn canonical_call(head: &'static str, args: Vec<Expr>) -> Expr {
    Expr::call(system_expr(head), args)
}

fn negated_numeric_literal(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Expr::integer(-value.to_bigint())),
        Expr::Atom(Atom::Rational(value)) => Some(Expr::rational(value.negated())),
        _ => None,
    }
}

// Parser sugar lowers into canonical runtime heads.
fn lower_list_sugar<F>(items: &[SyntaxExpr], resolve_symbol: &mut F) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    Ok(canonical_call(
        heads::structure::LIST,
        lower_syntax_args(items, resolve_symbol)?,
    ))
}

fn lower_association_sugar<F>(
    items: &[SyntaxExpr],
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    Ok(canonical_call(
        "Association",
        lower_syntax_args(items, resolve_symbol)?,
    ))
}

fn lower_pattern_sugar<F>(
    name: &SyntaxExpr,
    pattern: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    Ok(canonical_call(
        heads::patterns::PATTERN,
        vec![
            lower_syntax_expr_with(name, resolve_symbol)?,
            lower_syntax_expr_with(pattern, resolve_symbol)?,
        ],
    ))
}

pub(super) fn lower_slot_sugar(spec: &SlotSpec) -> Result<Expr, EvalError> {
    let (head, arg) = match spec {
        SlotSpec::Anonymous => (heads::functions::SLOT, Expr::integer(BigInt::from(1_u8))),
        SlotSpec::Number(value) => (heads::functions::SLOT, lower_integer_atom(value)?),
        SlotSpec::Named(value) => (heads::functions::SLOT, Expr::string(value.clone())),
        SlotSpec::Sequence => (
            heads::functions::SLOT_SEQUENCE,
            Expr::integer(BigInt::from(1_u8)),
        ),
        SlotSpec::SequenceNumber(value) => {
            (heads::functions::SLOT_SEQUENCE, lower_integer_atom(value)?)
        }
    };

    Ok(canonical_call(head, vec![arg]))
}

pub(super) fn lower_blank_sugar<F>(
    kind: BlankKind,
    head: Option<&SyntaxExpr>,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let head_symbol = match kind {
        BlankKind::Blank => heads::patterns::BLANK,
        BlankKind::BlankSequence => heads::patterns::BLANK_SEQUENCE,
        BlankKind::BlankNullSequence => heads::patterns::BLANK_NULL_SEQUENCE,
    };

    let args = match head {
        Some(head) => vec![lower_syntax_expr_with(head, resolve_symbol)?],
        None => Vec::new(),
    };

    Ok(canonical_call(head_symbol, args))
}

pub(super) fn lower_prefix_sugar<F>(
    op: PrefixOp,
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered = lower_syntax_expr_with(expr, resolve_symbol)?;

    Ok(lower_prefix_value(op, lowered))
}

fn lower_prefix_value(op: PrefixOp, lowered: Expr) -> Expr {
    match op {
        PrefixOp::Plus => lowered,
        PrefixOp::Minus => {
            if let Some(negated) = negated_numeric_literal(&lowered) {
                negated
            } else {
                canonical_call(
                    heads::arithmetic::TIMES,
                    vec![Expr::integer(BigInt::from(-1)), lowered],
                )
            }
        }
        PrefixOp::Not => canonical_call("Not", vec![lowered]),
        PrefixOp::PreIncrement => canonical_call("PreIncrement", vec![lowered]),
        PrefixOp::PreDecrement => canonical_call("PreDecrement", vec![lowered]),
    }
}

pub(super) fn lower_postfix_sugar<F>(
    op: PostfixOp,
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered = lower_syntax_expr_with(expr, resolve_symbol)?;

    lower_postfix_value(op, lowered)
}

fn lower_postfix_value(op: PostfixOp, lowered: Expr) -> Result<Expr, EvalError> {
    match op {
        PostfixOp::Function => Ok(canonical_call(heads::functions::FUNCTION, vec![lowered])),
        PostfixOp::Increment => Ok(canonical_call("Increment", vec![lowered])),
        PostfixOp::Decrement => Ok(canonical_call("Decrement", vec![lowered])),
        PostfixOp::Factorial => Ok(canonical_call("Factorial", vec![lowered])),
        PostfixOp::Factorial2 => Ok(canonical_call("Factorial2", vec![lowered])),
        PostfixOp::Derivative(order) => Ok(Expr::call(
            canonical_call("Derivative", vec![Expr::integer(BigInt::from(order))]),
            vec![lowered],
        )),
        PostfixOp::Optional => Ok(canonical_call("Optional", vec![lowered])),
        PostfixOp::Repeated => Ok(canonical_call("Repeated", vec![lowered])),
        PostfixOp::RepeatedNull => Ok(canonical_call("RepeatedNull", vec![lowered])),
        PostfixOp::Unset => Ok(canonical_call(heads::rules::UNSET, vec![lowered])),
    }
}

pub(super) fn lower_symbol<F>(raw: &str, resolve_symbol: &mut F) -> Expr
where
    F: FnMut(&str) -> Option<Symbol>,
{
    Expr::Atom(Atom::Symbol(
        resolve_symbol(raw).unwrap_or_else(|| symbol_from_raw_name(raw)),
    ))
}

enum LoweringTask<'a> {
    Visit(&'a SyntaxExpr, usize),
    Finish(&'a SyntaxExprKind, usize),
    Compound(usize),
    StartLeftChain,
    LeftLink(BinaryOp, &'a SyntaxExpr, usize),
    FinishLeftChain,
    MessageTag(Expr),
}

/// The heap tail owns traversal state only; syntax is always borrowed. Its
/// depth limit is independent of parser admission and also covers host ASTs.
#[cold]
#[inline(never)]
fn lower_syntax_expr_iterative<F>(
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    LOWERING_USED_TAIL.set(true);
    let limit = LOWERING_TAIL_DEPTH.get();
    let mut tasks = vec![LoweringTask::Visit(expr, 0)];
    let mut values = Vec::new();
    let mut chains = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            LoweringTask::Visit(node, depth) => {
                match &node.kind {
                    SyntaxExprKind::Integer(_)
                    | SyntaxExprKind::String(_)
                    | SyntaxExprKind::Symbol(_)
                    | SyntaxExprKind::Slot(_)
                    | SyntaxExprKind::Blank { head: None, .. } => {
                        values.push(lower_syntax_expr_with(node, resolve_symbol)?);
                        continue;
                    }
                    _ => {}
                }
                if depth >= limit {
                    return Err(EvalError::RecursionLimitExceeded {
                        limit: MAX_LOWERING_DEPTH,
                    });
                }
                let next = depth + 1;
                match &node.kind {
                    SyntaxExprKind::Binary { op, lhs, rhs }
                        if binary::is_tail_left_chain_op(*op) =>
                    {
                        let (base, links) = binary::tail_left_spine(*op, lhs, rhs)?;
                        tasks.push(LoweringTask::FinishLeftChain);
                        for (op, rhs) in links {
                            tasks.push(LoweringTask::LeftLink(op, rhs, next));
                            tasks.push(LoweringTask::Visit(rhs, next));
                        }
                        tasks.push(LoweringTask::StartLeftChain);
                        tasks.push(LoweringTask::Visit(base, next));
                    }
                    SyntaxExprKind::Binary {
                        op: BinaryOp::CompoundExpression,
                        lhs,
                        rhs,
                    } => {
                        // Preserve the existing flat right-spine traversal;
                        // statement count is width, not recursive depth.
                        let mut items = vec![lhs.as_ref()];
                        let mut rhs = rhs.as_ref();
                        while let SyntaxExprKind::Binary {
                            op: BinaryOp::CompoundExpression,
                            lhs,
                            rhs: next_rhs,
                        } = &rhs.kind
                        {
                            items.push(lhs);
                            rhs = next_rhs;
                        }
                        items.push(rhs);
                        tasks.push(LoweringTask::Compound(values.len()));
                        tasks.extend(
                            items
                                .into_iter()
                                .rev()
                                .map(|node| LoweringTask::Visit(node, next)),
                        );
                    }
                    kind => {
                        tasks.push(LoweringTask::Finish(kind, values.len()));
                        match kind {
                            SyntaxExprKind::Call { head, args } => {
                                tasks.extend(
                                    args.iter().rev().map(|arg| LoweringTask::Visit(arg, next)),
                                );
                                tasks.push(LoweringTask::Visit(head, next));
                            }
                            SyntaxExprKind::List(items) | SyntaxExprKind::Association(items) => {
                                tasks.extend(
                                    items
                                        .iter()
                                        .rev()
                                        .map(|item| LoweringTask::Visit(item, next)),
                                );
                            }
                            SyntaxExprKind::ComparisonChain {
                                operands,
                                operators,
                            } => {
                                binary::check_tail_comparison(operands, operators, node.span())?;
                                tasks.extend(
                                    operands
                                        .iter()
                                        .rev()
                                        .map(|item| LoweringTask::Visit(item, next)),
                                );
                            }
                            SyntaxExprKind::Blank {
                                head: Some(head), ..
                            } => tasks.push(LoweringTask::Visit(head, next)),
                            SyntaxExprKind::Pattern { name, pattern } => {
                                tasks.push(LoweringTask::Visit(pattern, next));
                                tasks.push(LoweringTask::Visit(name, next));
                            }
                            SyntaxExprKind::Prefix { expr, .. }
                            | SyntaxExprKind::Postfix { expr, .. } => {
                                tasks.push(LoweringTask::Visit(expr, next))
                            }
                            SyntaxExprKind::Binary { lhs, rhs, .. } => {
                                tasks.push(LoweringTask::Visit(rhs, next));
                                tasks.push(LoweringTask::Visit(lhs, next));
                            }
                            _ => unreachable!("leaf handled before scheduling children"),
                        }
                    }
                }
            }
            LoweringTask::Finish(kind, start) => {
                let mut args = values.split_off(start);
                let value = match kind {
                    SyntaxExprKind::Call { .. } => {
                        let head = args.remove(0);
                        Expr::call(head, args)
                    }
                    SyntaxExprKind::List(_) => canonical_call(heads::structure::LIST, args),
                    SyntaxExprKind::Association(_) => canonical_call("Association", args),
                    SyntaxExprKind::Blank { kind, .. } => canonical_call(
                        match kind {
                            BlankKind::Blank => heads::patterns::BLANK,
                            BlankKind::BlankSequence => heads::patterns::BLANK_SEQUENCE,
                            BlankKind::BlankNullSequence => heads::patterns::BLANK_NULL_SEQUENCE,
                        },
                        args,
                    ),
                    SyntaxExprKind::Pattern { .. } => {
                        canonical_call(heads::patterns::PATTERN, args)
                    }
                    SyntaxExprKind::Prefix { op, .. } => {
                        lower_prefix_value(*op, args.pop().unwrap())
                    }
                    SyntaxExprKind::Postfix { op, .. } => {
                        lower_postfix_value(*op, args.pop().unwrap())?
                    }
                    SyntaxExprKind::Binary { op, .. } => {
                        let rhs = args.pop().unwrap();
                        binary::lower_binary_values(*op, args.pop().unwrap(), rhs)?
                    }
                    SyntaxExprKind::ComparisonChain { operators, .. } => {
                        binary::finish_tail_comparison(args, operators)
                    }
                    _ => unreachable!("leaf has no finish task"),
                };
                values.push(value);
            }
            LoweringTask::Compound(start) => {
                let mut args = Vec::new();
                for value in values.drain(start..) {
                    binary::push_compound_expression_item(&mut args, value);
                }
                values.push(canonical_call("CompoundExpression", args));
            }
            LoweringTask::StartLeftChain => {
                chains.push(binary::TailLeftAccumulator::Value(values.pop().unwrap()))
            }
            LoweringTask::FinishLeftChain => values.push(chains.pop().unwrap().into_expr()),
            LoweringTask::LeftLink(op, raw_rhs, depth) => {
                let rhs = values.pop().unwrap();
                let accumulator = chains.pop().unwrap();
                if op != BinaryOp::MessageName {
                    chains.push(accumulator.apply(op, rhs));
                    continue;
                }
                // MessageName historically lowers its RHS first, then either
                // quotes the raw tag or lowers a nonliteral tag a second time.
                let lhs = accumulator.into_expr();
                if let Expr::Atom(Atom::Symbol(symbol)) = lhs {
                    chains.push(binary::TailLeftAccumulator::Value(message_name_expr(
                        symbol,
                        binary::lower_message_tag_string(raw_rhs),
                    )));
                } else {
                    tasks.push(LoweringTask::MessageTag(lhs));
                    match &raw_rhs.kind {
                        SyntaxExprKind::Symbol(value) | SyntaxExprKind::String(value) => {
                            values.push(Expr::string(value.clone()))
                        }
                        _ => tasks.push(LoweringTask::Visit(raw_rhs, depth)),
                    }
                }
            }
            LoweringTask::MessageTag(lhs) => {
                chains.push(binary::TailLeftAccumulator::Value(canonical_call(
                    heads::messages::MESSAGE_NAME,
                    vec![lhs, values.pop().unwrap()],
                )))
            }
        }
    }
    debug_assert!(chains.is_empty());
    debug_assert_eq!(values.len(), 1);
    Ok(values.pop().unwrap())
}
