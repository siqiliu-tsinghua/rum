use super::parse::{
    parse_part_list_item_selector, parse_part_selector, parse_scalar_part_selector,
};
use super::*;

pub(super) fn parse_extract_position_spec(expr: &Expr) -> Option<ExtractPositionSpec> {
    if is_empty_plain_list_expr(expr) {
        return Some(ExtractPositionSpec {
            position_specs: Vec::new(),
            multiple: true,
        });
    }

    if let Some(position_spec) = parse_single_position_spec_with_nested_lists(expr, false) {
        return Some(ExtractPositionSpec {
            position_specs: vec![position_spec],
            multiple: false,
        });
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let position_specs = args
        .iter()
        .map(|arg| parse_single_position_spec_with_nested_lists(arg, true))
        .collect::<Option<Vec<_>>>()?;
    Some(ExtractPositionSpec {
        position_specs,
        multiple: true,
    })
}

pub(in crate::runtime::kernel_data::structure_data) fn parse_single_position_spec(
    expr: &Expr,
) -> Option<Vec<PartSelector>> {
    parse_single_position_spec_with_nested_lists(expr, false)
}

fn parse_single_position_spec_with_nested_lists(
    expr: &Expr,
    allow_nested_plain_lists: bool,
) -> Option<Vec<PartSelector>> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(vec![selector]);
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    if args.is_empty() {
        return Some(Vec::new());
    }

    if !allow_nested_plain_lists && args.iter().all(is_plain_list_expr) {
        return None;
    }

    let parse_selector = if allow_nested_plain_lists {
        parse_extract_position_selector
    } else {
        parse_part_selector
    };

    args.iter().map(parse_selector).collect()
}

fn parse_extract_position_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(selector);
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    Some(PartSelector::Multi(
        args.iter()
            .map(parse_part_list_item_selector)
            .collect::<Option<Vec<_>>>()?,
    ))
}

pub(super) fn wrap_extracted_value(value: Expr, wrapper: Option<&Expr>) -> Expr {
    wrapper
        .cloned()
        .map(|head| Expr::call(head, vec![value.clone()]))
        .unwrap_or(value)
}

pub(super) fn extract_operator_form(position_spec: Expr) -> Expr {
    let slot = Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1))]);
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr("Extract"),
            vec![slot, position_spec],
        )],
    )
}
