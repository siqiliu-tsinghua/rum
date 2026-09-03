use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum CompositionDirection {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub(in crate::runtime) struct CompositionSpec<'a> {
    pub(in crate::runtime) direction: CompositionDirection,
    pub(in crate::runtime) functions: Vec<&'a Expr>,
}

pub(in crate::runtime) fn parse_composition(expr: &Expr) -> Option<CompositionSpec<'_>> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "Composition") => {
            Some(CompositionSpec {
                direction: CompositionDirection::Left,
                functions: args.iter().collect(),
            })
        }
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "RightComposition") => {
            Some(CompositionSpec {
                direction: CompositionDirection::Right,
                functions: args.iter().collect(),
            })
        }
        _ => None,
    }
}

pub(in crate::runtime) fn apply_composition_to_args(
    spec: &CompositionSpec<'_>,
    args: &[Expr],
) -> Expr {
    match spec.direction {
        CompositionDirection::Left => {
            let Some((last, rest)) = spec.functions.split_last() else {
                return Expr::call(system_expr("Identity"), args.to_vec());
            };

            let mut current = Expr::call((*last).clone(), args.to_vec());
            for function in rest.iter().rev() {
                current = Expr::call((**function).clone(), vec![current]);
            }
            current
        }
        CompositionDirection::Right => {
            let Some((first, rest)) = spec.functions.split_first() else {
                return Expr::call(system_expr("Identity"), args.to_vec());
            };

            let mut current = Expr::call((*first).clone(), args.to_vec());
            for function in rest {
                current = Expr::call((**function).clone(), vec![current]);
            }
            current
        }
    }
}

pub(in crate::runtime) fn is_callable_operator_wrapper(expr: &Expr) -> bool {
    if !matches!(expr, Expr::Call { .. }) {
        return false;
    }

    is_curried_operator_wrapper(expr)
        || is_reverse_applied_wrapper(expr)
        || is_comparison_operator_wrapper(expr)
        || is_nearest_to_wrapper(expr)
        || is_mutation_operator_wrapper(expr)
        || is_schema_operator_wrapper(expr)
        || is_string_unary_operator_wrapper(expr)
        || is_structure_operator_wrapper(expr)
        || is_predicate_operator_wrapper(expr)
        || is_rewrite_operator_wrapper(expr)
        || parse_key_wrapper(expr).is_some()
}

pub(in crate::runtime) fn apply_callable_operator_wrapper_to_args(
    head: &Expr,
    args: &[Expr],
) -> Option<Expr> {
    if let Some(expr) = apply_curried_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = apply_reverse_applied_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = apply_comparison_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = apply_nearest_to_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = apply_mutation_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = apply_schema_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = apply_string_unary_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = apply_structure_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = apply_predicate_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    if let Some(expr) = apply_rewrite_operator_wrapper_to_args(head, args) {
        return Some(expr);
    }

    apply_key_wrapper_to_args(head, args)
}

pub(in crate::runtime) fn invalid_callable_operator_wrapper_argx_symbol(
    head: &Expr,
    args: &[Expr],
) -> Option<&'static str> {
    if let Some(symbol) = invalid_mutation_operator_wrapper_argx_symbol(head, args) {
        return Some(symbol);
    }

    if let Some(symbol) = invalid_schema_operator_wrapper_argx_symbol(head, args) {
        return Some(symbol);
    }

    if let Some(symbol) = invalid_string_unary_operator_wrapper_argx_symbol(head, args) {
        return Some(symbol);
    }

    if let Some(symbol) = invalid_structure_operator_wrapper_argx_symbol(head, args) {
        return Some(symbol);
    }

    if let Some(symbol) = invalid_predicate_operator_wrapper_argx_symbol(head, args) {
        return Some(symbol);
    }

    if let Some(symbol) = invalid_rewrite_operator_wrapper_argx_symbol(head, args) {
        return Some(symbol);
    }

    invalid_comparison_operator_wrapper_argx_symbol(head, args)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonOperatorKind {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Equal,
    Unequal,
    Same,
}

#[derive(Clone, Debug)]
struct ComparisonOperatorSpec<'a> {
    kind: ComparisonOperatorKind,
    bound: &'a Expr,
}

impl ComparisonOperatorKind {
    fn wrapper_symbol(self) -> &'static str {
        match self {
            Self::Less => "LessThan",
            Self::Greater => "GreaterThan",
            Self::LessEqual => "LessEqualThan",
            Self::GreaterEqual => "GreaterEqualThan",
            Self::Equal => "EqualTo",
            Self::Unequal => "UnequalTo",
            Self::Same => "SameAs",
        }
    }
}

fn is_comparison_operator_wrapper(expr: &Expr) -> bool {
    parse_comparison_operator_wrapper(expr).is_some()
}

fn invalid_comparison_operator_wrapper_argx_symbol(
    head: &Expr,
    args: &[Expr],
) -> Option<&'static str> {
    let spec = parse_comparison_operator_wrapper(head)?;
    if args.len() == 1 {
        return None;
    }

    match spec.kind {
        ComparisonOperatorKind::Less
        | ComparisonOperatorKind::Greater
        | ComparisonOperatorKind::LessEqual
        | ComparisonOperatorKind::GreaterEqual => Some(spec.kind.wrapper_symbol()),
        ComparisonOperatorKind::Equal
        | ComparisonOperatorKind::Unequal
        | ComparisonOperatorKind::Same => None,
    }
}

fn apply_comparison_operator_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_comparison_operator_wrapper(head)?;
    let head = match spec.kind {
        ComparisonOperatorKind::Less => "Less",
        ComparisonOperatorKind::Greater => "Greater",
        ComparisonOperatorKind::LessEqual => "LessEqual",
        ComparisonOperatorKind::GreaterEqual => "GreaterEqual",
        ComparisonOperatorKind::Equal => "Equal",
        ComparisonOperatorKind::Unequal => "Unequal",
        ComparisonOperatorKind::Same => "SameQ",
    };

    match spec.kind {
        ComparisonOperatorKind::Equal
        | ComparisonOperatorKind::Unequal
        | ComparisonOperatorKind::Same => {
            let mut comparison_args = args.to_vec();
            comparison_args.push(spec.bound.clone());
            Some(Expr::call(system_expr(head), comparison_args))
        }
        ComparisonOperatorKind::Less
        | ComparisonOperatorKind::Greater
        | ComparisonOperatorKind::LessEqual
        | ComparisonOperatorKind::GreaterEqual => {
            let [arg] = args else {
                return None;
            };
            Some(Expr::call(
                system_expr(head),
                vec![arg.clone(), spec.bound.clone()],
            ))
        }
    }
}

fn parse_comparison_operator_wrapper(expr: &Expr) -> Option<ComparisonOperatorSpec<'_>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    let [bound] = args.as_slice() else {
        return None;
    };

    let kind = if is_system_symbol_name(head, "LessThan") {
        ComparisonOperatorKind::Less
    } else if is_system_symbol_name(head, "GreaterThan") {
        ComparisonOperatorKind::Greater
    } else if is_system_symbol_name(head, "LessEqualThan") {
        ComparisonOperatorKind::LessEqual
    } else if is_system_symbol_name(head, "GreaterEqualThan") {
        ComparisonOperatorKind::GreaterEqual
    } else if is_system_symbol_name(head, "EqualTo") {
        ComparisonOperatorKind::Equal
    } else if is_system_symbol_name(head, "UnequalTo") {
        ComparisonOperatorKind::Unequal
    } else if is_system_symbol_name(head, "SameAs") {
        ComparisonOperatorKind::Same
    } else {
        return None;
    };

    Some(ComparisonOperatorSpec { kind, bound })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CurriedOperatorKind {
    OperatorApplied,
    CurryApplied,
}

#[derive(Clone, Debug)]
struct CurriedOperatorSpec<'a> {
    kind: CurriedOperatorKind,
    function: &'a Expr,
    arity: usize,
    order: Vec<usize>,
    accumulated: Vec<Expr>,
}

impl CurriedOperatorKind {
    fn symbol(self) -> &'static str {
        match self {
            Self::OperatorApplied => "OperatorApplied",
            Self::CurryApplied => "CurryApplied",
        }
    }
}

fn is_curried_operator_wrapper(expr: &Expr) -> bool {
    parse_curried_operator(expr).is_some()
}

fn apply_curried_operator_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    parse_curried_operator(head).map(|spec| apply_curried_operator_to_args(&spec, args))
}

fn parse_curried_operator(expr: &Expr) -> Option<CurriedOperatorSpec<'_>> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "OperatorApplied") => {
            match args.as_slice() {
                [function] => Some(CurriedOperatorSpec {
                    kind: CurriedOperatorKind::OperatorApplied,
                    function,
                    arity: 2,
                    order: vec![1, 0],
                    accumulated: Vec::new(),
                }),
                [function, spec] => {
                    let (arity, order) = parse_curried_operator_spec(spec)?;
                    Some(CurriedOperatorSpec {
                        kind: CurriedOperatorKind::OperatorApplied,
                        function,
                        arity,
                        order,
                        accumulated: Vec::new(),
                    })
                }
                [function, spec, accumulated] => {
                    let (arity, order) = parse_curried_operator_spec(spec)?;
                    Some(CurriedOperatorSpec {
                        kind: CurriedOperatorKind::OperatorApplied,
                        function,
                        arity,
                        order,
                        accumulated: parse_callable_wrapper_accumulated(accumulated)?,
                    })
                }
                _ => None,
            }
        }
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "CurryApplied") => {
            match args.as_slice() {
                [function, spec] => {
                    let (arity, order) = parse_curried_operator_spec(spec)?;
                    Some(CurriedOperatorSpec {
                        kind: CurriedOperatorKind::CurryApplied,
                        function,
                        arity,
                        order,
                        accumulated: Vec::new(),
                    })
                }
                [function, spec, accumulated] => {
                    let (arity, order) = parse_curried_operator_spec(spec)?;
                    Some(CurriedOperatorSpec {
                        kind: CurriedOperatorKind::CurryApplied,
                        function,
                        arity,
                        order,
                        accumulated: parse_callable_wrapper_accumulated(accumulated)?,
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn parse_curried_operator_spec(spec: &Expr) -> Option<(usize, Vec<usize>)> {
    if let Some(arity) = expr_as_non_negative_usize(spec) {
        return Some((arity, (0..arity).collect()));
    }

    if let Some(order) = expr_as_index_list(spec) {
        let arity = order.iter().copied().max().map_or(0usize, |max| max + 1);
        return Some((arity, order));
    }

    match spec {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "Rule") && args.len() == 2 => {
            let arity = expr_as_non_negative_usize(&args[0])?;
            let order = expr_as_index_list(&args[1])?;
            if order.iter().any(|index| *index >= arity) {
                return None;
            }
            Some((arity, order))
        }
        _ => None,
    }
}

fn expr_as_index_list(expr: &Expr) -> Option<Vec<usize>> {
    let items = call_items_with_head(expr, &system_expr("List"))?;
    items
        .iter()
        .map(|item| {
            let one_based = expr_as_non_negative_usize(item)?;
            if one_based == 0 {
                None
            } else {
                Some(one_based - 1)
            }
        })
        .collect()
}

fn parse_callable_wrapper_accumulated(expr: &Expr) -> Option<Vec<Expr>> {
    call_items_with_head(expr, &system_expr("List")).map(|items| items.to_vec())
}

fn apply_curried_operator_to_args(spec: &CurriedOperatorSpec<'_>, args: &[Expr]) -> Expr {
    let mut collected = spec.accumulated.clone();
    collected.extend(args.iter().cloned());

    if collected.len() < spec.arity {
        return render_curried_operator_expr(
            spec.kind,
            spec.function,
            spec.arity,
            &spec.order,
            &collected,
        );
    }

    let ordered = spec
        .order
        .iter()
        .filter_map(|index| collected.get(*index).cloned())
        .collect::<Vec<_>>();
    let applied = Expr::call(spec.function.clone(), ordered);
    let remaining = collected.into_iter().skip(spec.arity).collect::<Vec<_>>();

    if remaining.is_empty() {
        applied
    } else {
        Expr::call(applied, remaining)
    }
}

fn render_curried_operator_expr(
    kind: CurriedOperatorKind,
    function: &Expr,
    arity: usize,
    order: &[usize],
    accumulated: &[Expr],
) -> Expr {
    if kind == CurriedOperatorKind::OperatorApplied
        && arity == 2
        && order == [1usize, 0usize]
        && accumulated.is_empty()
    {
        return Expr::call(system_expr("OperatorApplied"), vec![function.clone()]);
    }

    let mut args = vec![
        function.clone(),
        render_curried_operator_spec_expr(arity, order),
    ];
    if !accumulated.is_empty() {
        args.push(Expr::call(system_expr("List"), accumulated.to_vec()));
    }

    Expr::call(system_expr(kind.symbol()), args)
}

fn render_curried_operator_spec_expr(arity: usize, order: &[usize]) -> Expr {
    let max_index = order.iter().copied().max().map_or(0usize, |max| max + 1);
    let index_list = Expr::call(
        system_expr("List"),
        order
            .iter()
            .map(|index| Expr::integer(BigInt::from(index + 1)))
            .collect(),
    );

    if order.iter().enumerate().all(|(index, item)| *item == index) {
        Expr::integer(BigInt::from(arity))
    } else if arity == max_index {
        index_list
    } else {
        Expr::call(
            system_expr("Rule"),
            vec![Expr::integer(BigInt::from(arity)), index_list],
        )
    }
}

#[derive(Clone, Debug)]
struct ReverseAppliedSpec<'a> {
    function: &'a Expr,
    reversed_count: Option<usize>,
}

fn is_reverse_applied_wrapper(expr: &Expr) -> bool {
    parse_reverse_applied_wrapper(expr).is_some()
}

fn apply_reverse_applied_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    parse_reverse_applied_wrapper(head).map(|spec| apply_reverse_applied_to_args(&spec, args))
}

fn parse_reverse_applied_wrapper(expr: &Expr) -> Option<ReverseAppliedSpec<'_>> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "ReverseApplied") => {
            match args.as_slice() {
                [function] => Some(ReverseAppliedSpec {
                    function,
                    reversed_count: None,
                }),
                [function, count] => Some(ReverseAppliedSpec {
                    function,
                    reversed_count: Some(expr_as_non_negative_usize(count)?),
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

fn apply_reverse_applied_to_args(spec: &ReverseAppliedSpec<'_>, args: &[Expr]) -> Expr {
    let reverse_count = spec.reversed_count.unwrap_or(args.len()).min(args.len());
    let mut reordered = args[..reverse_count].to_vec();
    reordered.reverse();
    reordered.extend_from_slice(&args[reverse_count..]);
    Expr::call(spec.function.clone(), reordered)
}

#[derive(Clone, Debug)]
struct NearestToSpec<'a> {
    target: &'a Expr,
    selection: Option<&'a Expr>,
}

fn is_nearest_to_wrapper(expr: &Expr) -> bool {
    parse_nearest_to_wrapper(expr).is_some()
}

fn apply_nearest_to_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_nearest_to_wrapper(head)?;
    let [data] = args else {
        return None;
    };

    let mut nearest_args = vec![data.clone(), spec.target.clone()];
    if let Some(selection) = spec.selection {
        nearest_args.push(selection.clone());
    }
    Some(Expr::call(system_expr("Nearest"), nearest_args))
}

fn parse_nearest_to_wrapper(expr: &Expr) -> Option<NearestToSpec<'_>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "NearestTo") {
        return None;
    }

    match args.as_slice() {
        [target] => Some(NearestToSpec {
            target,
            selection: None,
        }),
        [target, selection] if is_nearest_to_selection_spec(selection) => Some(NearestToSpec {
            target,
            selection: Some(selection),
        }),
        _ => None,
    }
}

fn is_nearest_to_selection_spec(expr: &Expr) -> bool {
    if expr_as_positive_usize(expr).is_some() {
        return true;
    }

    let Expr::Call { head, args, .. } = expr else {
        return false;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return false;
    }
    let [count, radius] = args.as_slice() else {
        return false;
    };

    (is_system_symbol_name(count, heads::structure::ALL) || expr_as_positive_usize(count).is_some())
        && is_non_negative_nearest_to_radius(radius)
}

fn is_non_negative_nearest_to_radius(expr: &Expr) -> bool {
    if is_system_symbol_name(expr, heads::structure::INFINITY) {
        return true;
    }
    expr_as_exact_rational(expr).is_some_and(|value| {
        compare_rationals_for_nearest_to(&value, &Rational::zero()) != std::cmp::Ordering::Less
    })
}

fn compare_rationals_for_nearest_to(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

#[derive(Clone, Copy, Debug)]
struct MutationOperatorSpec<'a> {
    symbol: &'static str,
    value: &'a Expr,
}

fn is_mutation_operator_wrapper(expr: &Expr) -> bool {
    parse_mutation_operator_wrapper(expr).is_some()
}

fn apply_mutation_operator_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_mutation_operator_wrapper(head)?;
    let [target] = args else {
        return None;
    };

    Some(Expr::call(
        system_expr(spec.symbol),
        vec![target.clone(), spec.value.clone()],
    ))
}

fn invalid_mutation_operator_wrapper_argx_symbol(
    head: &Expr,
    args: &[Expr],
) -> Option<&'static str> {
    let spec = parse_mutation_operator_wrapper(head)?;
    (args.len() != 1).then_some(spec.symbol)
}

fn parse_mutation_operator_wrapper(expr: &Expr) -> Option<MutationOperatorSpec<'_>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    let [value] = args.as_slice() else {
        return None;
    };

    let symbol = if is_system_symbol_name(head, "AddTo") {
        "AddTo"
    } else if is_system_symbol_name(head, "AppendTo") {
        "AppendTo"
    } else if is_system_symbol_name(head, "ApplyTo") {
        "ApplyTo"
    } else if is_system_symbol_name(head, "AssociateTo") {
        "AssociateTo"
    } else if is_system_symbol_name(head, "DivideBy") {
        "DivideBy"
    } else if is_system_symbol_name(head, "KeyDropFrom") {
        "KeyDropFrom"
    } else if is_system_symbol_name(head, "PrependTo") {
        "PrependTo"
    } else if is_system_symbol_name(head, "SubtractFrom") {
        "SubtractFrom"
    } else if is_system_symbol_name(head, "TimesBy") {
        "TimesBy"
    } else {
        return None;
    };

    Some(MutationOperatorSpec { symbol, value })
}

#[derive(Clone, Copy, Debug)]
struct SchemaOperatorSpec<'a> {
    symbol: &'static str,
    schema: &'a Expr,
}

fn is_schema_operator_wrapper(expr: &Expr) -> bool {
    parse_schema_operator_wrapper(expr).is_some()
}

fn apply_schema_operator_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_schema_operator_wrapper(head)?;
    let [value] = args else {
        return None;
    };

    Some(Expr::call(
        system_expr(spec.symbol),
        vec![value.clone(), spec.schema.clone()],
    ))
}

fn invalid_schema_operator_wrapper_argx_symbol(head: &Expr, args: &[Expr]) -> Option<&'static str> {
    let spec = parse_schema_operator_wrapper(head)?;
    (args.len() != 1).then_some(spec.symbol)
}

fn parse_schema_operator_wrapper(expr: &Expr) -> Option<SchemaOperatorSpec<'_>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    let [schema] = args.as_slice() else {
        return None;
    };

    let symbol = if is_system_symbol_name(head, "ConformSchema") {
        "ConformSchema"
    } else if is_system_symbol_name(head, "SchemaErrors") {
        "SchemaErrors"
    } else {
        return None;
    };

    Some(SchemaOperatorSpec { symbol, schema })
}

#[derive(Clone, Copy, Debug)]
struct StringUnaryOperatorSpec<'a> {
    symbol: &'static str,
    tail: &'a [Expr],
}

fn is_string_unary_operator_wrapper(expr: &Expr) -> bool {
    parse_string_unary_operator_wrapper(expr).is_some()
}

fn apply_string_unary_operator_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_string_unary_operator_wrapper(head)?;
    let [value] = args else {
        return None;
    };

    let mut ordinary_args = Vec::with_capacity(spec.tail.len() + 1);
    ordinary_args.push(value.clone());
    ordinary_args.extend(spec.tail.iter().cloned());
    Some(Expr::call(system_expr(spec.symbol), ordinary_args))
}

fn invalid_string_unary_operator_wrapper_argx_symbol(
    head: &Expr,
    args: &[Expr],
) -> Option<&'static str> {
    let spec = parse_string_unary_operator_wrapper(head)?;
    (args.len() != 1).then_some(spec.symbol)
}

fn parse_string_unary_operator_wrapper(expr: &Expr) -> Option<StringUnaryOperatorSpec<'_>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if args.is_empty() {
        return None;
    }

    Some(StringUnaryOperatorSpec {
        symbol: string_unary_operator_symbol(head)?,
        tail: args,
    })
}

fn string_unary_operator_symbol(head: &Expr) -> Option<&'static str> {
    if is_system_symbol_name(head, "StringContainsQ") {
        Some("StringContainsQ")
    } else if is_system_symbol_name(head, "StringStartsQ") {
        Some("StringStartsQ")
    } else if is_system_symbol_name(head, "StringEndsQ") {
        Some("StringEndsQ")
    } else if is_system_symbol_name(head, "StringMatchQ") {
        Some("StringMatchQ")
    } else if is_system_symbol_name(head, "StringFreeQ") {
        Some("StringFreeQ")
    } else if is_system_symbol_name(head, "StringReplace") {
        Some("StringReplace")
    } else if is_system_symbol_name(head, "StringCases") {
        Some("StringCases")
    } else if is_system_symbol_name(head, "StringDelete") {
        Some("StringDelete")
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug)]
struct StructureOperatorSpec<'a> {
    symbol: &'static str,
    tail: &'a [Expr],
}

fn is_structure_operator_wrapper(expr: &Expr) -> bool {
    parse_structure_operator_wrapper(expr).is_some()
}

fn apply_structure_operator_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_structure_operator_wrapper(head)?;
    let [data] = args else {
        return None;
    };

    let mut ordinary_args = Vec::with_capacity(spec.tail.len() + 1);
    ordinary_args.push(data.clone());
    ordinary_args.extend(spec.tail.iter().cloned());
    Some(Expr::call(system_expr(spec.symbol), ordinary_args))
}

fn invalid_structure_operator_wrapper_argx_symbol(
    head: &Expr,
    args: &[Expr],
) -> Option<&'static str> {
    let spec = parse_structure_operator_wrapper(head)?;
    (args.len() != 1).then_some(spec.symbol)
}

fn parse_structure_operator_wrapper(expr: &Expr) -> Option<StructureOperatorSpec<'_>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    Some(StructureOperatorSpec {
        symbol: structure_operator_symbol(head, args.len())?,
        tail: args,
    })
}

fn structure_operator_symbol(head: &Expr, arity: usize) -> Option<&'static str> {
    // An arity becomes an operator form only when it cannot be a valid
    // data-form call of the same builtin; never sniff the first argument type.
    match arity {
        1 if is_system_symbol_name(head, "Take") => Some("Take"),
        1 if is_system_symbol_name(head, "Drop") => Some("Drop"),
        1 if is_system_symbol_name(head, "Delete") => Some("Delete"),
        1 if is_system_symbol_name(head, "TakeList") => Some("TakeList"),
        1 if is_system_symbol_name(head, "Partition") => Some("Partition"),
        1 if is_system_symbol_name(head, "Riffle") => Some("Riffle"),
        1 if is_system_symbol_name(head, "GatherBy") => Some("GatherBy"),
        1 if is_system_symbol_name(head, "DatePlus") => Some("DatePlus"),
        1 if is_system_symbol_name(head, "MaxBy") => Some("MaxBy"),
        1 if is_system_symbol_name(head, "MinBy") => Some("MinBy"),
        1 if is_system_symbol_name(head, "DeleteDuplicatesBy") => Some("DeleteDuplicatesBy"),
        1 if is_system_symbol_name(head, "KeyGroupBy") => Some("KeyGroupBy"),
        1 if is_system_symbol_name(head, "MergeNested") => Some("MergeNested"),
        1 if is_system_symbol_name(head, "ReverseSortBy") => Some("ReverseSortBy"),
        2 if is_system_symbol_name(head, "Insert") => Some("Insert"),
        2 if is_system_symbol_name(head, "TakeLargestBy") => Some("TakeLargestBy"),
        2 if is_system_symbol_name(head, "TakeSmallestBy") => Some("TakeSmallestBy"),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug)]
struct PredicateOperatorSpec<'a> {
    symbol: &'static str,
    tail: &'a [Expr],
}

fn is_predicate_operator_wrapper(expr: &Expr) -> bool {
    parse_predicate_operator_wrapper(expr).is_some()
}

fn apply_predicate_operator_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_predicate_operator_wrapper(head)?;
    let [data] = args else {
        return None;
    };

    let mut ordinary_args = Vec::with_capacity(spec.tail.len() + 1);
    ordinary_args.push(data.clone());
    ordinary_args.extend(spec.tail.iter().cloned());
    Some(Expr::call(system_expr(spec.symbol), ordinary_args))
}

fn invalid_predicate_operator_wrapper_argx_symbol(
    head: &Expr,
    args: &[Expr],
) -> Option<&'static str> {
    let spec = parse_predicate_operator_wrapper(head)?;
    (args.len() != 1).then_some(spec.symbol)
}

fn parse_predicate_operator_wrapper(expr: &Expr) -> Option<PredicateOperatorSpec<'_>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    Some(PredicateOperatorSpec {
        symbol: predicate_operator_symbol(head, args.len())?,
        tail: args,
    })
}

fn predicate_operator_symbol(head: &Expr, arity: usize) -> Option<&'static str> {
    // An arity becomes an operator form only when it cannot be a valid
    // data-form call of the same builtin; never sniff the first argument type.
    // Excluded by that rule: OrderedQ[p] collides with OrderedQ[list], and
    // Lookup[key, default] collides with Lookup[assoc, key].
    match arity {
        1 if is_system_symbol_name(head, "ContainsAll") => Some("ContainsAll"),
        1 if is_system_symbol_name(head, "ContainsAny") => Some("ContainsAny"),
        1 if is_system_symbol_name(head, "ContainsExactly") => Some("ContainsExactly"),
        1 if is_system_symbol_name(head, "ContainsNone") => Some("ContainsNone"),
        1 if is_system_symbol_name(head, "ContainsOnly") => Some("ContainsOnly"),
        1 if is_system_symbol_name(head, "Between") => Some("Between"),
        1 if is_system_symbol_name(head, "AllTrue") => Some("AllTrue"),
        1 if is_system_symbol_name(head, "AnyTrue") => Some("AnyTrue"),
        1 if is_system_symbol_name(head, "NoneTrue") => Some("NoneTrue"),
        1 if is_system_symbol_name(head, "MatchQ") => Some("MatchQ"),
        1 if is_system_symbol_name(head, "MemberQ") => Some("MemberQ"),
        1 if is_system_symbol_name(head, "FreeQ") => Some("FreeQ"),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug)]
struct RewriteOperatorSpec<'a> {
    symbol: &'static str,
    tail: &'a [Expr],
}

fn is_rewrite_operator_wrapper(expr: &Expr) -> bool {
    parse_rewrite_operator_wrapper(expr).is_some()
}

fn apply_rewrite_operator_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_rewrite_operator_wrapper(head)?;
    let [data] = args else {
        return None;
    };

    let mut ordinary_args = Vec::with_capacity(spec.tail.len() + 1);
    ordinary_args.push(data.clone());
    ordinary_args.extend(spec.tail.iter().cloned());
    Some(Expr::call(system_expr(spec.symbol), ordinary_args))
}

fn invalid_rewrite_operator_wrapper_argx_symbol(
    head: &Expr,
    args: &[Expr],
) -> Option<&'static str> {
    let spec = parse_rewrite_operator_wrapper(head)?;
    (args.len() != 1).then_some(spec.symbol)
}

fn parse_rewrite_operator_wrapper(expr: &Expr) -> Option<RewriteOperatorSpec<'_>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    Some(RewriteOperatorSpec {
        symbol: rewrite_operator_symbol(head, args.len())?,
        tail: args,
    })
}

fn rewrite_operator_symbol(head: &Expr, arity: usize) -> Option<&'static str> {
    // An arity becomes an operator form only when it cannot be a valid
    // data-form call of the same builtin; never sniff the first argument type.
    match arity {
        1 if is_system_symbol_name(head, "Cases") => Some("Cases"),
        1 if is_system_symbol_name(head, "DeleteCases") => Some("DeleteCases"),
        1 if is_system_symbol_name(head, "Position") => Some("Position"),
        1 if is_system_symbol_name(head, "Count") => Some("Count"),
        1 if is_system_symbol_name(head, "Replace") => Some("Replace"),
        1 if is_system_symbol_name(head, "ReplaceAll") => Some("ReplaceAll"),
        1 if is_system_symbol_name(head, "ReplaceAllUnheld") => Some("ReplaceAllUnheld"),
        1 if is_system_symbol_name(head, "ReplaceRepeated") => Some("ReplaceRepeated"),
        _ => None,
    }
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    let value = expr_as_non_negative_usize(expr)?;
    (value > 0).then_some(value)
}

fn parse_key_wrapper(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "Key") => {
            match args.as_slice() {
                [key] => Some(key),
                _ => None,
            }
        }
        _ => None,
    }
}

fn apply_key_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let [expr] = args else {
        return None;
    };

    let key = parse_key_wrapper(head)?;
    let entries = association_entries_from_expr(expr)?;
    Some(
        entries
            .iter()
            .find(|(entry_key, _)| entry_key == key)
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| missing_key_absent_expr(key.clone())),
    )
}
