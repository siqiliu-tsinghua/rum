use super::super::*;
use super::evaluated_part_selectors::EvaluatedPartSelectors;

mod exact_paths;
mod replace;
mod selectors;

use exact_paths::{
    collect_exact_paths_for_position_spec, exact_part_component, normalize_structural_arg_position,
    select_exact_part,
};
use replace::ReplacePartHeadsMode;
use selectors::{
    AtomicPartSpec, PartSelector, apply_part_selectors, invalid_part_selector_for_part,
    parse_part_selectors, parse_part_selectors_for_part, parse_single_position_spec,
    select_positioned_parts_for_span,
};

pub fn exact_position_paths_from_expr(expr: &Expr, spec_expr: &Expr) -> Option<Vec<Vec<isize>>> {
    exact_paths::exact_position_paths_from_expr(expr, spec_expr)
}

/// Returns exact structural paths for Part-style selector expressions.
pub fn exact_position_paths_from_part_selector_exprs(
    expr: &Expr,
    selector_exprs: &EvaluatedPartSelectors,
) -> Option<Vec<Vec<isize>>> {
    let selectors = parse_part_selectors(selector_exprs.as_slice())?;
    Some(collect_exact_paths_for_position_spec(
        expr,
        &selectors,
        ReplacePartHeadsMode::Automatic,
    ))
}

/// Returns the first evaluated expression that is not a valid Part selector.
pub fn invalid_part_selector_expr(selector_exprs: &EvaluatedPartSelectors) -> Option<&Expr> {
    selector_exprs
        .as_slice()
        .iter()
        .find(|selector| parse_part_selectors(std::slice::from_ref(selector)).is_none())
}

pub(in crate::runtime) fn exact_position_paths_from_part_target_selector_exprs(
    expr: &Expr,
    selector_exprs: &EvaluatedPartSelectors,
) -> Option<Vec<Vec<isize>>> {
    let selectors = parse_part_selectors_for_part(selector_exprs.as_slice())?;
    Some(collect_exact_paths_for_position_spec(
        expr,
        &selectors,
        ReplacePartHeadsMode::Automatic,
    ))
}

pub(in crate::runtime) fn invalid_part_target_selector_expr(
    selector_exprs: &EvaluatedPartSelectors,
) -> Option<&Expr> {
    invalid_part_selector_for_part(selector_exprs.as_slice())
}

pub(in crate::runtime) fn selected_expr_from_part_target_selector_exprs(
    session: &mut Session,
    expr: &Expr,
    selector_exprs: &EvaluatedPartSelectors,
) -> Option<Expr> {
    let selectors = parse_part_selectors_for_part(selector_exprs.as_slice())?;
    apply_part_selectors(session, expr, &selectors)
}

pub fn exact_expr_at_path(expr: &Expr, path: &[isize]) -> Option<Expr> {
    exact_paths::exact_expr_at_path(expr, path)
}

pub fn replace_exact_path(expr: Expr, path: &[isize], rhs: Expr) -> Expr {
    exact_paths::replace_exact_path(expr, path, rhs)
}

fn is_plain_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST)
    )
}

fn is_empty_plain_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::structure::LIST) && args.is_empty()
    )
}

fn structural_head_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Atom(Atom::Integer(_)) => system_expr("Integer"),
        Expr::Atom(Atom::Rational(_)) => system_expr("Rational"),
        Expr::Atom(Atom::String(_)) => system_expr("String"),
        Expr::Atom(Atom::Symbol(_)) => system_expr("Symbol"),
        Expr::Call { head, .. } => head.as_ref().clone(),
    }
}
