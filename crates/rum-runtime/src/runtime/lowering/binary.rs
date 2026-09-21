use super::*;

mod comparison;
mod left_chain;

use comparison::comparison_head;
pub(super) use comparison::lower_comparison_chain;
use left_chain::{is_iterative_left_chain_op, lower_left_chain};

pub(super) fn is_tail_left_chain_op(op: BinaryOp) -> bool {
    is_iterative_left_chain_op(op)
}

pub(super) fn lower_binary_sugar<F>(
    op: BinaryOp,
    lhs: &SyntaxExpr,
    rhs: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    if matches!(op, BinaryOp::CompoundExpression) {
        return lower_compound_expression_sugar(lhs, rhs, resolve_symbol);
    }

    if is_iterative_left_chain_op(op) {
        return lower_left_chain(op, lhs, rhs, resolve_symbol);
    }

    let lowered_lhs = lower_syntax_expr_with(lhs, resolve_symbol)?;
    let lowered_rhs = lower_syntax_expr_with(rhs, resolve_symbol)?;

    lower_binary_values(op, lowered_lhs, lowered_rhs)
}

pub(super) fn lower_binary_values(
    op: BinaryOp,
    lowered_lhs: Expr,
    lowered_rhs: Expr,
) -> Result<Expr, EvalError> {
    match op {
        BinaryOp::TagSet => lower_tag_set_sugar(lowered_lhs, lowered_rhs),
        BinaryOp::Set => Ok(canonical_call(
            heads::rules::SET,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::UpSet => Ok(canonical_call("UpSet", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::AddTo => Ok(canonical_call("AddTo", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::SubtractFrom => Ok(canonical_call(
            "SubtractFrom",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::TimesBy => Ok(canonical_call("TimesBy", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::DivideBy => Ok(canonical_call("DivideBy", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::ApplyTo => Ok(canonical_call("ApplyTo", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::SetDelayed => Ok(canonical_call(
            heads::rules::SET_DELAYED,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::UpSetDelayed => Ok(canonical_call(
            "UpSetDelayed",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Rule => Ok(canonical_call(
            heads::rules::RULE,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::RuleDelayed => Ok(canonical_call(
            heads::rules::RULE_DELAYED,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Condition => Ok(canonical_call(
            heads::patterns::CONDITION,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Or => Ok(canonical_call("Or", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::And => Ok(canonical_call("And", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::Apply => Ok(Expr::call(lowered_lhs, vec![lowered_rhs])),
        BinaryOp::Map => Ok(canonical_call("Map", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::MapAll => Ok(canonical_call("MapAll", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::ApplyHead => Ok(canonical_call("Apply", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::MapApply => Ok(canonical_call("MapApply", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::Composition => Ok(canonical_call(
            "Composition",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Power => Ok(canonical_call(
            heads::arithmetic::POWER,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Function => Ok(canonical_call(
            heads::functions::FUNCTION,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::SameQ
        | BinaryOp::UnsameQ
        | BinaryOp::Equal
        | BinaryOp::Unequal
        | BinaryOp::Less
        | BinaryOp::Greater
        | BinaryOp::LessEqual
        | BinaryOp::GreaterEqual => Ok(canonical_call(
            comparison_head(op).expect("comparison operator has a canonical head"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::CompoundExpression => {
            unreachable!("compound expression handled before lowering operands")
        }
        BinaryOp::MessageName
        | BinaryOp::ReplaceAll
        | BinaryOp::ReplaceRepeated
        | BinaryOp::Pattern
        | BinaryOp::Alternatives
        | BinaryOp::PatternTest
        | BinaryOp::StringExpression
        | BinaryOp::StringJoin
        | BinaryOp::PostfixApply
        | BinaryOp::RightComposition
        | BinaryOp::Add
        | BinaryOp::Subtract
        | BinaryOp::Times
        | BinaryOp::Dot
        | BinaryOp::NonCommutativeMultiply
        | BinaryOp::Divide => unreachable!("left-chain ops handled before lowering operands"),
    }
}

/// Lowers a `;` spine iteratively: the right-nested syntax chain is walked in
/// a loop, so statement count no longer consumes lowering recursion depth.
/// The output is byte-identical to the recursive form — one flat n-ary
/// `CompoundExpression` with the items in source order.
fn lower_compound_expression_sugar<F>(
    lhs: &SyntaxExpr,
    rhs: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let mut items = Vec::new();
    let mut lhs = lhs;
    let mut rhs = rhs;

    loop {
        let lowered = lower_syntax_expr_with(lhs, resolve_symbol)?;
        push_compound_expression_item(&mut items, lowered);

        match &rhs.kind {
            SyntaxExprKind::Binary {
                op: BinaryOp::CompoundExpression,
                lhs: next_lhs,
                rhs: next_rhs,
            } => {
                lhs = next_lhs;
                rhs = next_rhs;
            }
            _ => break,
        }
    }

    let lowered_tail = lower_syntax_expr_with(rhs, resolve_symbol)?;
    push_compound_expression_item(&mut items, lowered_tail);

    Ok(canonical_call("CompoundExpression", items))
}

pub(super) fn push_compound_expression_item(items: &mut Vec<Expr>, expr: Expr) {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(&head, "CompoundExpression") => {
            items.extend(args);
        }
        other => items.push(other),
    }
}

pub(super) fn lower_message_tag_expr<F>(
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    match &expr.kind {
        SyntaxExprKind::Symbol(value) => Ok(Expr::string(value.clone())),
        SyntaxExprKind::String(value) => Ok(Expr::string(value.clone())),
        _ => lower_syntax_expr_with(expr, resolve_symbol),
    }
}

pub(super) fn lower_message_tag_string(expr: &SyntaxExpr) -> String {
    match &expr.kind {
        SyntaxExprKind::Symbol(value) | SyntaxExprKind::String(value) => value.clone(),
        _ => expr.to_string(),
    }
}

pub(super) fn lower_tag_set_sugar(tag: Expr, rhs: Expr) -> Result<Expr, EvalError> {
    match rhs {
        Expr::Call { head, args }
            if is_system_symbol_name(&head, heads::rules::SET) && args.len() == 2 =>
        {
            Ok(canonical_call(
                heads::rules::TAG_SET,
                vec![tag, args[0].clone(), args[1].clone()],
            ))
        }
        Expr::Call { head, args }
            if is_system_symbol_name(&head, heads::rules::SET_DELAYED) && args.len() == 2 =>
        {
            Ok(canonical_call(
                heads::rules::TAG_SET_DELAYED,
                vec![tag, args[0].clone(), args[1].clone()],
            ))
        }
        Expr::Call { head, args }
            if is_system_symbol_name(&head, heads::rules::UNSET) && args.len() == 1 =>
        {
            Ok(canonical_call(
                heads::rules::TAG_UNSET,
                vec![tag, args[0].clone()],
            ))
        }
        _ => Ok(canonical_call(heads::rules::TAG_SET, vec![tag, rhs])),
    }
}

/// Links are returned outside-in, ready for the tail's LIFO work stack. The
/// recursive front continues using the established left-chain implementation.
pub(super) fn tail_left_spine<'a>(
    op: BinaryOp,
    mut lhs: &'a SyntaxExpr,
    rhs: &'a SyntaxExpr,
) -> Result<(&'a SyntaxExpr, Vec<(BinaryOp, &'a SyntaxExpr)>), EvalError> {
    let mut links = vec![(op, rhs)];
    while let SyntaxExprKind::Binary {
        op,
        lhs: next_lhs,
        rhs,
    } = &lhs.kind
    {
        if !is_iterative_left_chain_op(*op) {
            break;
        }
        if links.len() >= rum_syntax::MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS {
            return Err(SyntaxError::LeftAssociativeChainTooLong {
                max: rum_syntax::MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
                span: lhs.span(),
            }
            .into());
        }
        links.push((*op, rhs));
        lhs = next_lhs;
    }
    Ok((lhs, links))
}

pub(super) enum TailLeftAccumulator {
    Value(Expr),
    Flat(&'static str, Vec<Expr>),
}

impl TailLeftAccumulator {
    pub(super) fn into_expr(self) -> Expr {
        match self {
            Self::Value(value) => value,
            Self::Flat(head, args) => canonical_call(head, args),
        }
    }

    fn append(self, head: &'static str, rhs: Expr) -> Self {
        match self {
            Self::Flat(active, mut args) if active == head => {
                args.push(rhs);
                Self::Flat(head, args)
            }
            other => Self::Flat(head, vec![other.into_expr(), rhs]),
        }
    }

    pub(super) fn apply(self, op: BinaryOp, rhs: Expr) -> Self {
        match op {
            BinaryOp::Add => return self.append(heads::arithmetic::PLUS, rhs),
            BinaryOp::Subtract => {
                return self.append(
                    heads::arithmetic::PLUS,
                    canonical_call(
                        heads::arithmetic::TIMES,
                        vec![Expr::integer(BigInt::from(-1)), rhs],
                    ),
                );
            }
            BinaryOp::Times => return self.append(heads::arithmetic::TIMES, rhs),
            BinaryOp::Divide => {
                let reciprocal = |rhs| {
                    canonical_call(
                        heads::arithmetic::POWER,
                        vec![rhs, Expr::integer(BigInt::from(-1))],
                    )
                };
                if let Self::Flat(heads::arithmetic::TIMES, mut args) = self {
                    args.push(reciprocal(rhs));
                    return Self::Flat(heads::arithmetic::TIMES, args);
                }
                let lhs = self.into_expr();
                let rational = |value: &Expr| match value {
                    Expr::Atom(Atom::Integer(value)) => {
                        Some(Rational::from_integer(value.to_bigint()))
                    }
                    Expr::Atom(Atom::Rational(value)) => Some(value.as_ref().clone()),
                    _ => None,
                };
                if let Some(value) = rational(&lhs).and_then(|lhs| lhs.div(&rational(&rhs)?)) {
                    return Self::Value(Expr::rational(value));
                }
                return Self::Flat(heads::arithmetic::TIMES, vec![lhs, reciprocal(rhs)]);
            }
            BinaryOp::RightComposition => return self.append("RightComposition", rhs),
            BinaryOp::Dot => return self.append("Dot", rhs),
            BinaryOp::StringJoin => return self.append("StringJoin", rhs),
            BinaryOp::Alternatives => return self.append("Alternatives", rhs),
            BinaryOp::StringExpression => return self.append("StringExpression", rhs),
            _ => {}
        }
        let lhs = self.into_expr();
        if op == BinaryOp::PostfixApply {
            return Self::Value(Expr::call(rhs, vec![lhs]));
        }
        let head = match op {
            BinaryOp::ReplaceAll => heads::rules::REPLACE_ALL,
            BinaryOp::ReplaceRepeated => heads::rules::REPLACE_REPEATED,
            BinaryOp::Pattern if matches!(lhs, Expr::Atom(Atom::Symbol(_))) => {
                heads::patterns::PATTERN
            }
            BinaryOp::Pattern => "Optional",
            BinaryOp::PatternTest => "PatternTest",
            BinaryOp::NonCommutativeMultiply => "NonCommutativeMultiply",
            _ => unreachable!("nonvariadic left-chain operator"),
        };
        Self::Value(canonical_call(head, vec![lhs, rhs]))
    }
}

pub(super) fn check_tail_comparison(
    operands: &[SyntaxExpr],
    operators: &[BinaryOp],
    span: rum_syntax::Span,
) -> Result<(), EvalError> {
    debug_assert_eq!(operands.len(), operators.len() + 1);
    debug_assert!(operators.iter().all(|op| comparison_head(*op).is_some()));
    if operators.len() > rum_syntax::MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS {
        return Err(SyntaxError::LeftAssociativeChainTooLong {
            max: rum_syntax::MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            span,
        }
        .into());
    }
    Ok(())
}

pub(super) fn finish_tail_comparison(mut operands: Vec<Expr>, operators: &[BinaryOp]) -> Expr {
    let Some(first) = operators.first() else {
        return operands
            .pop()
            .unwrap_or_else(|| canonical_call("Inequality", Vec::new()));
    };
    if operators.iter().all(|op| op == first) {
        return canonical_call(comparison_head(*first).unwrap(), operands);
    }
    let mut args = Vec::with_capacity(operators.len() * 2 + 1);
    for (index, operand) in operands.into_iter().enumerate() {
        args.push(operand);
        if let Some(op) = operators.get(index) {
            args.push(system_expr(comparison_head(*op).unwrap()));
        }
    }
    canonical_call("Inequality", args)
}
