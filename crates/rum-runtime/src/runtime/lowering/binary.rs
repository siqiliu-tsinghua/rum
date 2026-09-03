use super::*;

mod comparison;
mod left_chain;

use comparison::comparison_head;
pub(super) use comparison::lower_comparison_chain;
use left_chain::{is_iterative_left_chain_op, lower_left_chain};

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

fn push_compound_expression_item(items: &mut Vec<Expr>, expr: Expr) {
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
