use super::*;

pub(in crate::runtime::kernel_data::structure_data) fn parse_part_selectors(
    exprs: &[Expr],
) -> Option<Vec<PartSelector>> {
    exprs.iter().map(parse_part_selector).collect()
}

pub(in crate::runtime::kernel_data::structure_data) fn parse_part_selectors_for_part(
    exprs: &[Expr],
) -> Option<Vec<PartSelector>> {
    exprs.iter().map(parse_part_selector_for_part).collect()
}

pub(in crate::runtime::kernel_data::structure_data) fn invalid_part_selector_for_part(
    exprs: &[Expr],
) -> Option<&Expr> {
    exprs
        .iter()
        .find(|selector| parse_part_selector_for_part(selector).is_none())
}

pub(super) fn parse_part_selector_for_part(expr: &Expr) -> Option<PartSelector> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(selector);
    }

    let Expr::Call { head, args } = expr else {
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

pub(super) fn parse_part_list_item_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(PartSelector::Single(AtomicPartSpec::Index(index)));
    }

    parse_key_part_selector(expr).map(PartSelector::Key)
}

pub(super) fn parse_part_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(selector) = parse_scalar_part_selector(expr) {
        return Some(selector);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    Some(PartSelector::Multi(
        args.iter()
            .map(parse_part_selector)
            .collect::<Option<Vec<_>>>()?,
    ))
}

pub(super) fn parse_scalar_part_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(spec) = parse_atomic_part_spec(expr) {
        return Some(PartSelector::Single(spec));
    }

    parse_key_part_selector(expr).map(PartSelector::Key)
}

fn parse_key_part_selector(expr: &Expr) -> Option<Expr> {
    if matches!(expr, Expr::Atom(Atom::String(_))) {
        return Some(expr.clone());
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Key") || args.len() != 1 {
        return None;
    }

    Some(args[0].clone())
}

fn parse_atomic_part_spec(expr: &Expr) -> Option<AtomicPartSpec> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(AtomicPartSpec::Index(index));
    }

    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(AtomicPartSpec::All);
    }

    parse_span_part_spec(expr)
}

fn parse_span_part_spec(expr: &Expr) -> Option<AtomicPartSpec> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::SPAN) {
        return None;
    }

    let spec = match args.as_slice() {
        [] => PartSpanSpec {
            start: SpanEndpoint::All,
            end: SpanEndpoint::All,
            step: 1,
        },
        [value] => {
            let endpoint = parse_span_endpoint(value)?;
            single_endpoint_span_spec(endpoint)
        }
        [start, end] => PartSpanSpec {
            start: parse_span_endpoint(start)?,
            end: parse_span_endpoint(end)?,
            step: 1,
        },
        [start, end, step] => {
            let step = expr_as_signed_isize(step)?;
            PartSpanSpec {
                start: parse_span_endpoint(start)?,
                end: parse_span_endpoint(end)?,
                step,
            }
        }
        _ => return None,
    };

    Some(AtomicPartSpec::Span(spec))
}

fn single_endpoint_span_spec(endpoint: SpanEndpoint) -> PartSpanSpec {
    match endpoint {
        SpanEndpoint::All => PartSpanSpec {
            start: SpanEndpoint::All,
            end: SpanEndpoint::All,
            step: 1,
        },
        SpanEndpoint::UpTo(limit) => PartSpanSpec {
            start: SpanEndpoint::Index(1),
            end: SpanEndpoint::UpTo(limit),
            step: 1,
        },
        SpanEndpoint::Index(index) if index < 0 => PartSpanSpec {
            start: SpanEndpoint::Index(index),
            end: SpanEndpoint::Index(-1),
            step: 1,
        },
        SpanEndpoint::Index(0) => PartSpanSpec {
            start: SpanEndpoint::Index(1),
            end: SpanEndpoint::Index(0),
            step: 1,
        },
        SpanEndpoint::Index(index) => PartSpanSpec {
            start: SpanEndpoint::Index(1),
            end: SpanEndpoint::Index(index),
            step: 1,
        },
    }
}

fn parse_span_endpoint(expr: &Expr) -> Option<SpanEndpoint> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(SpanEndpoint::All);
    }

    if let Some(limit) = parse_upto_span_endpoint(expr) {
        return Some(SpanEndpoint::UpTo(limit));
    }

    expr_as_signed_isize(expr).map(SpanEndpoint::Index)
}

fn parse_upto_span_endpoint(expr: &Expr) -> Option<Option<isize>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    let [limit] = args.as_slice() else {
        return None;
    };
    if !is_system_symbol_name(head, "UpTo") {
        return None;
    }

    if is_system_symbol_name(limit, heads::structure::INFINITY) {
        return Some(None);
    }

    let limit = expr_as_signed_isize(limit)?;
    (limit >= 0).then_some(Some(limit))
}

pub(super) fn is_invalid_span_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. }
            if is_system_symbol_name(head, heads::structure::SPAN)
                && parse_span_part_spec(expr).is_none()
    )
}

pub(super) fn first_invalid_span_expr(expr: &Expr) -> Option<&Expr> {
    if is_invalid_span_expr(expr) {
        return Some(expr);
    }

    let Expr::Call { args, .. } = expr else {
        return None;
    };

    args.iter().find_map(first_invalid_span_expr)
}

pub(super) fn is_invalid_numeric_upto_limit(expr: &Expr) -> bool {
    if is_system_symbol_name(expr, heads::structure::INFINITY) {
        return false;
    }

    match expr {
        Expr::Atom(Atom::Integer(value)) => value.to_bigint() < BigInt::from(0),
        Expr::Atom(Atom::Rational(_)) => true,
        _ => false,
    }
}
