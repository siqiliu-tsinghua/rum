use super::apply::{key_expr, select_parts_for_selector};
use super::*;

pub(super) fn diagnose_part_selection_failure(
    session: &mut Session,
    expr: &Expr,
    selectors: &[PartSelector],
) -> Option<PartSelectionFailure> {
    let (first, rest) = selectors.split_first()?;
    let failure = diagnose_selector_failure(session, expr, first, rest)?;
    Some(match failure {
        PartSelectionFailure::PartD { .. } => PartSelectionFailure::PartD {
            spec: Some(selector_path_to_expr(selectors)),
        },
        other => other,
    })
}

fn diagnose_selector_failure(
    session: &mut Session,
    expr: &Expr,
    selector: &PartSelector,
    rest: &[PartSelector],
) -> Option<PartSelectionFailure> {
    match select_parts_for_selector(session, expr, selector) {
        Some(shape) => diagnose_selected_shape_failure(session, shape, rest),
        None => diagnose_failed_selector(session, expr, selector),
    }
}

fn diagnose_selected_shape_failure(
    session: &mut Session,
    shape: SelectedPartShape,
    rest: &[PartSelector],
) -> Option<PartSelectionFailure> {
    if rest.is_empty() {
        return None;
    }

    match shape {
        SelectedPartShape::Leaf(expr) => diagnose_part_selection_failure(session, &expr, rest),
        SelectedPartShape::Branch(children) => diagnose_children_failure(session, children, rest),
        SelectedPartShape::AssociationBranch(children) => diagnose_children_failure(
            session,
            children
                .into_iter()
                .map(|(_, child)| child)
                .collect::<Vec<_>>(),
            rest,
        ),
    }
}

fn diagnose_children_failure(
    session: &mut Session,
    children: Vec<SelectedPartShape>,
    rest: &[PartSelector],
) -> Option<PartSelectionFailure> {
    if children.is_empty() {
        return Some(PartSelectionFailure::PartD { spec: None });
    }

    children
        .into_iter()
        .find_map(|child| diagnose_selected_shape_failure(session, child, rest))
}

fn diagnose_failed_selector(
    session: &mut Session,
    expr: &Expr,
    selector: &PartSelector,
) -> Option<PartSelectionFailure> {
    match selector {
        PartSelector::Key(_) => Some(PartSelectionFailure::Pspec1 {
            spec: selector_to_expr(selector),
        }),
        PartSelector::Single(AtomicPartSpec::Index(index)) => diagnose_failed_index(expr, *index),
        PartSelector::Single(AtomicPartSpec::All) => Some(PartSelectionFailure::PartD {
            spec: Some(selector_to_expr(selector)),
        }),
        PartSelector::Single(AtomicPartSpec::Span(span)) => Some(PartSelectionFailure::Take {
            span: *span,
            target: expr.clone(),
        }),
        PartSelector::Multi(specs) => {
            diagnose_failed_multi_selector(session, expr, selector, specs)
        }
    }
}

fn diagnose_failed_index(expr: &Expr, index: isize) -> Option<PartSelectionFailure> {
    if index == 0 {
        return None;
    }

    match expr {
        Expr::Atom(_) => Some(PartSelectionFailure::PartD {
            spec: Some(selector_to_expr(&PartSelector::Single(
                AtomicPartSpec::Index(index),
            ))),
        }),
        Expr::Call { .. } => Some(PartSelectionFailure::PartW {
            part: Expr::integer(BigInt::from(index)),
            target: expr.clone(),
        }),
    }
}

fn diagnose_failed_multi_selector(
    session: &mut Session,
    expr: &Expr,
    selector: &PartSelector,
    specs: &[PartSelector],
) -> Option<PartSelectionFailure> {
    if session.association_entry_cursor(expr).is_some() {
        if let Some(index) = specs.iter().find_map(selector_zero_index) {
            return Some(PartSelectionFailure::PartW {
                part: Expr::integer(BigInt::from(index)),
                target: expr.clone(),
            });
        }
    }

    for spec in specs {
        if select_parts_for_selector(session, expr, spec).is_some() {
            continue;
        }

        let failure = diagnose_failed_selector(session, expr, spec)?;
        return Some(match failure {
            PartSelectionFailure::PartW { target, .. } => PartSelectionFailure::PartW {
                part: selector_to_expr(selector),
                target,
            },
            PartSelectionFailure::Pspec1 { .. } => PartSelectionFailure::Pspec1 {
                spec: selector_to_expr(selector),
            },
            other => other,
        });
    }

    Some(PartSelectionFailure::Pspec1 {
        spec: selector_to_expr(selector),
    })
}

fn selector_zero_index(selector: &PartSelector) -> Option<isize> {
    match selector {
        PartSelector::Single(AtomicPartSpec::Index(0)) => Some(0),
        PartSelector::Multi(specs) => specs.iter().find_map(selector_zero_index),
        _ => None,
    }
}

fn selector_to_expr(selector: &PartSelector) -> Expr {
    match selector {
        PartSelector::Single(AtomicPartSpec::Index(index)) => Expr::integer(BigInt::from(*index)),
        PartSelector::Single(AtomicPartSpec::All) => system_expr(heads::structure::ALL),
        PartSelector::Single(AtomicPartSpec::Span(span)) => span_to_expr(*span),
        PartSelector::Key(key) => key_expr(key.clone()),
        PartSelector::Multi(specs) => Expr::call(
            system_expr(heads::structure::LIST),
            specs.iter().map(selector_to_expr).collect(),
        ),
    }
}

fn selector_path_to_expr(selectors: &[PartSelector]) -> Expr {
    Expr::call(
        system_expr(heads::structure::LIST),
        selectors.iter().map(selector_to_expr).collect(),
    )
}

fn span_to_expr(span: PartSpanSpec) -> Expr {
    Expr::call(
        system_expr(heads::structure::SPAN),
        vec![
            span_endpoint_to_expr(span.start),
            span_endpoint_to_expr(span.end),
            Expr::integer(BigInt::from(span.step)),
        ],
    )
}

pub(super) fn invalid_span_message_spec(expr: &Expr) -> String {
    let Expr::Call { head, args } = expr else {
        return expr.to_string();
    };

    if !is_system_symbol_name(head, heads::structure::SPAN) || !(2..=3).contains(&args.len()) {
        return expr.to_string();
    }

    args.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ;; ")
}

pub(super) fn span_take_message_range(span: PartSpanSpec) -> (Expr, Expr) {
    (
        span_take_message_start(span.start),
        span_take_message_end(span.end),
    )
}

fn span_take_message_start(endpoint: SpanEndpoint) -> Expr {
    match endpoint {
        SpanEndpoint::All => Expr::integer(BigInt::from(1)),
        _ => span_endpoint_to_expr(endpoint),
    }
}

fn span_take_message_end(endpoint: SpanEndpoint) -> Expr {
    match endpoint {
        SpanEndpoint::All => Expr::integer(BigInt::from(-1)),
        _ => span_endpoint_to_expr(endpoint),
    }
}

fn span_endpoint_to_expr(endpoint: SpanEndpoint) -> Expr {
    match endpoint {
        SpanEndpoint::Index(index) => Expr::integer(BigInt::from(index)),
        SpanEndpoint::UpTo(limit) => Expr::call(
            system_expr("UpTo"),
            vec![
                limit
                    .map(|value| Expr::integer(BigInt::from(value)))
                    .unwrap_or_else(|| system_expr(heads::structure::INFINITY)),
            ],
        ),
        SpanEndpoint::All => system_expr(heads::structure::ALL),
    }
}
