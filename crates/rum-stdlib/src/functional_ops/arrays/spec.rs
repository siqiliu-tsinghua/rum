use super::{tensor::*, *};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TensorNode {
    Scalar(Expr),
    Array(Vec<TensorNode>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ArrayFilterPadding {
    Uniform(ArrayFilterPaddingLeaf),
    PerAxis(Vec<ArrayFilterAxisPadding>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ArrayFilterAxisPadding {
    pub(super) before: ArrayFilterPaddingLeaf,
    pub(super) after: ArrayFilterPaddingLeaf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ArrayFilterPaddingLeaf {
    Fixed,
    Periodic,
    Constant(Expr),
    None,
    Cycle(Vec<Expr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ArrayFilterSpec {
    Radii(Vec<usize>),
    Template(ArrayFilterTemplate),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ArrayFilterTemplate {
    pub(super) shape: Vec<usize>,
    pub(super) center: Vec<usize>,
    pub(super) mask: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ArrayReduceSpec {
    Elementwise,
    Grouped(Vec<Vec<usize>>),
}

pub(super) fn parse_array_filter_padding(expr: &Expr, rank: usize) -> Option<ArrayFilterPadding> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    let symbol = expr_as_symbol(&args[0])?;
    if symbol.name() != "Padding" {
        return None;
    }

    if let Some(per_axis) = parse_array_filter_per_axis_padding(&args[1], rank) {
        return Some(ArrayFilterPadding::PerAxis(per_axis));
    }

    parse_array_filter_padding_leaf(&args[1]).map(ArrayFilterPadding::Uniform)
}

pub(super) fn parse_array_filter_per_axis_padding(
    expr: &Expr,
    rank: usize,
) -> Option<Vec<ArrayFilterAxisPadding>> {
    let items = call_items_with_head(expr, &system_expr("List"))?;
    if items.len() != rank {
        return None;
    }
    if !items
        .iter()
        .any(|item| call_items_with_head(item, &system_expr("List")).is_some())
    {
        return None;
    }

    let mut paddings = Vec::with_capacity(rank);
    for item in items {
        let side_items = call_items_with_head(item, &system_expr("List"))?;
        let axis_padding = match side_items {
            [leaf] => {
                let leaf = parse_array_filter_padding_leaf(leaf)?;
                ArrayFilterAxisPadding {
                    before: leaf.clone(),
                    after: leaf,
                }
            }
            [before, after] => ArrayFilterAxisPadding {
                before: parse_array_filter_padding_leaf(before)?,
                after: parse_array_filter_padding_leaf(after)?,
            },
            _ => return None,
        };
        paddings.push(axis_padding);
    }

    Some(paddings)
}

pub(super) fn parse_array_filter_padding_leaf(expr: &Expr) -> Option<ArrayFilterPaddingLeaf> {
    match expr {
        value if expr_as_symbol(value).is_some_and(|symbol| symbol.name() == "None") => {
            Some(ArrayFilterPaddingLeaf::None)
        }
        Expr::Atom(Atom::String(value)) if value == "Fixed" => Some(ArrayFilterPaddingLeaf::Fixed),
        Expr::Atom(Atom::String(value)) if value == "Periodic" => {
            Some(ArrayFilterPaddingLeaf::Periodic)
        }
        Expr::Call { head, args, .. }
            if is_system_symbol_name(head, "List") && !args.is_empty() =>
        {
            Some(ArrayFilterPaddingLeaf::Cycle(args.to_vec()))
        }
        other => Some(ArrayFilterPaddingLeaf::Constant(other.clone())),
    }
}

pub(super) fn parse_rectangular_list_array(expr: &Expr) -> Option<(Vec<usize>, Vec<Expr>)> {
    let Expr::Call { head, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    let tensor = parse_tensor_node(expr, &system_expr("List"));
    let shape = tensor_shape(&tensor)?;
    let flat = flatten_tensor_scalars(&tensor);
    Some((shape, flat))
}

pub(super) fn parse_array_filter_spec(expr: &Expr, rank: usize) -> Option<ArrayFilterSpec> {
    if let Some(radius) = expr_as_non_negative_usize(expr) {
        return Some(ArrayFilterSpec::Radii(vec![radius; rank]));
    }

    if let Some(items) = call_items_with_head(expr, &system_expr("List")) {
        if items.len() == rank {
            if let Some(radii) = items.iter().map(expr_as_non_negative_usize).collect() {
                return Some(ArrayFilterSpec::Radii(radii));
            }
        }
    }

    parse_array_filter_template(expr, rank).map(ArrayFilterSpec::Template)
}

pub(super) fn parse_array_filter_template(expr: &Expr, rank: usize) -> Option<ArrayFilterTemplate> {
    let (shape, flat) = parse_rectangular_list_array(expr)?;
    if shape.len() != rank || shape.contains(&0) {
        return None;
    }

    let mask = flat
        .into_iter()
        .map(|item| match item {
            Expr::Atom(Atom::Integer(value)) if value.as_i64() == Some(1) => Some(true),
            Expr::Atom(Atom::Integer(value)) if value.as_i64() == Some(0) => Some(false),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    // For even-sized templates, rum anchors neighborhoods with maximal
    // overhang at the beginning side, which corresponds to floor(len/2).
    let center = shape.iter().map(|len| len / 2).collect();

    Some(ArrayFilterTemplate {
        shape,
        center,
        mask,
    })
}

pub(super) fn parse_array_reduce_spec(expr: &Expr, rank: usize) -> Option<ArrayReduceSpec> {
    if let Some(dimension) = expr_as_positive_usize(expr) {
        return (dimension <= rank).then_some(ArrayReduceSpec::Grouped(vec![vec![dimension - 1]]));
    }

    if let Some(span) = parse_array_reduce_span(expr, rank) {
        return Some(ArrayReduceSpec::Grouped(vec![span]));
    }

    let items = call_items_with_head(expr, &system_expr("List"))?;
    if items.is_empty() {
        return Some(ArrayReduceSpec::Elementwise);
    }

    if items
        .iter()
        .all(|item| call_items_with_head(item, &system_expr("List")).is_some())
    {
        let mut seen = std::collections::HashSet::new();
        let mut groups = Vec::with_capacity(items.len());
        for group_expr in items {
            let group_items = call_items_with_head(group_expr, &system_expr("List"))?;
            if group_items.is_empty() {
                return None;
            }

            let mut group = Vec::with_capacity(group_items.len());
            for item in group_items {
                let dimension = expr_as_positive_usize(item)?;
                if dimension > rank {
                    return None;
                }
                let zero_based = dimension - 1;
                if !seen.insert(zero_based) {
                    return None;
                }
                group.push(zero_based);
            }
            group.sort_unstable();
            groups.push(group);
        }

        return Some(ArrayReduceSpec::Grouped(groups));
    }

    let mut seen = std::collections::HashSet::new();
    let mut dimensions = Vec::with_capacity(items.len());
    for item in items {
        let dimension = expr_as_positive_usize(item)?;
        if dimension > rank {
            return None;
        }
        let zero_based = dimension - 1;
        if !seen.insert(zero_based) {
            return None;
        }
        dimensions.push(zero_based);
    }
    dimensions.sort_unstable();
    Some(ArrayReduceSpec::Grouped(vec![dimensions]))
}

pub(super) fn parse_array_reduce_span(expr: &Expr, rank: usize) -> Option<Vec<usize>> {
    let items = call_items_with_head(expr, &system_expr("Span"))?;

    let (start, end) = match items {
        [end] => (1usize, parse_array_reduce_span_endpoint(end, rank, rank)?),
        [start, end] => (
            parse_array_reduce_span_endpoint(start, 1, rank)?,
            parse_array_reduce_span_endpoint(end, rank, rank)?,
        ),
        [start, end, step] if expr_as_positive_usize(step) == Some(1) => (
            parse_array_reduce_span_endpoint(start, 1, rank)?,
            parse_array_reduce_span_endpoint(end, rank, rank)?,
        ),
        _ => return None,
    };

    (start <= end).then(|| ((start - 1)..end).collect())
}

pub(super) fn parse_array_reduce_span_endpoint(
    expr: &Expr,
    all_value: usize,
    rank: usize,
) -> Option<usize> {
    if is_system_symbol_name(expr, "All") {
        return Some(all_value);
    }

    let value = expr_as_positive_usize(expr)?;
    (value <= rank).then_some(value)
}

pub(super) fn array_filter_window_shape(spec: &ArrayFilterSpec) -> Vec<usize> {
    match spec {
        ArrayFilterSpec::Radii(radii) => radii
            .iter()
            .map(|radius| radius.saturating_mul(2).saturating_add(1))
            .collect(),
        ArrayFilterSpec::Template(template) => template.shape.clone(),
    }
}

pub(super) fn array_filter_window_center(spec: &ArrayFilterSpec) -> Vec<usize> {
    match spec {
        ArrayFilterSpec::Radii(radii) => radii.clone(),
        ArrayFilterSpec::Template(template) => template.center.clone(),
    }
}

pub(super) fn array_filter_padding_leaf<'a>(
    padding: &'a ArrayFilterPadding,
    axis: usize,
    before: bool,
) -> &'a ArrayFilterPaddingLeaf {
    match padding {
        ArrayFilterPadding::Uniform(leaf) => leaf,
        ArrayFilterPadding::PerAxis(axis_paddings) => {
            let axis_padding = &axis_paddings[axis];
            if before {
                &axis_padding.before
            } else {
                &axis_padding.after
            }
        }
    }
}

pub(super) fn array_filter_axis_trim(
    padding: &ArrayFilterPadding,
    axis: usize,
    window_len: usize,
    center: usize,
) -> (usize, usize) {
    let before_trim = if matches!(
        array_filter_padding_leaf(padding, axis, true),
        ArrayFilterPaddingLeaf::None
    ) {
        center
    } else {
        0
    };
    let after_trim = if matches!(
        array_filter_padding_leaf(padding, axis, false),
        ArrayFilterPaddingLeaf::None
    ) {
        window_len.saturating_sub(center + 1)
    } else {
        0
    };

    (before_trim, after_trim)
}

pub(super) fn array_filter_template_includes(
    spec: &ArrayFilterSpec,
    window_index: &[usize],
) -> bool {
    match spec {
        ArrayFilterSpec::Radii(_) => true,
        ArrayFilterSpec::Template(template) => {
            let offset = tensor_flat_offset(&template.shape, window_index)
                .expect("template positions should follow the template shape");
            template.mask[offset]
        }
    }
}
