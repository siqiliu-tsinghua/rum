use super::super::*;

pub(in crate::runtime) fn evaluate_wrapper_arg(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Evaluate") && args.len() == 1 => {
            Some(&args[0])
        }
        _ => None,
    }
}

pub(in crate::runtime) fn unevaluated_wrapper_arg(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Unevaluated") && args.len() == 1 =>
        {
            Some(&args[0])
        }
        _ => None,
    }
}

pub(in crate::runtime) fn strip_hold_pattern(mut expr: &Expr) -> &Expr {
    while let Expr::Call { head, args } = expr {
        if !MatcherPatternHead::HoldPattern.matches(head) || args.len() != 1 {
            break;
        }
        expr = &args[0];
    }
    expr
}

pub(in crate::runtime) fn release_hold_once(expr: &Expr) -> Option<Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if expr_as_symbol(head)
        .filter(|symbol| symbol.context() == SYSTEM_CONTEXT)
        .is_some_and(|symbol| {
            matches!(
                symbol.name(),
                "Hold"
                    | "HoldForm"
                    | "HoldFirst"
                    | "HoldRest"
                    | "HoldAll"
                    | "HoldAllComplete"
                    | "HoldComplete"
                    | "HoldPattern"
            )
        })
    {
        return Some(match args.as_slice() {
            [single] => single.clone(),
            _ => sequence_expr(args.to_vec()),
        });
    }

    None
}

pub(in crate::runtime) fn splice_sequence_argument(args: &mut Vec<Expr>, expr: Expr) {
    if let Some(items) = sequence_items(&expr) {
        args.extend(items.iter().cloned());
    } else {
        args.push(expr);
    }
}

pub(in crate::runtime) enum FunctionSpec<'a> {
    SlotBased {
        body: &'a Expr,
        attributes: AttributeSet,
    },
    Explicit {
        params: Vec<Symbol>,
        body: &'a Expr,
        attributes: AttributeSet,
    },
}

impl FunctionSpec<'_> {
    pub(in crate::runtime) fn attributes(&self) -> AttributeSet {
        match self {
            Self::SlotBased { attributes, .. } | Self::Explicit { attributes, .. } => *attributes,
        }
    }
}

pub(in crate::runtime) fn parse_function(expr: &Expr) -> Option<FunctionSpec<'_>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") && args.len() == 1 => {
            Some(FunctionSpec::SlotBased {
                body: &args[0],
                attributes: AttributeSet::empty(),
            })
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") && args.len() == 2 => {
            if let Some(params) = parse_function_parameters(&args[0]) {
                Some(FunctionSpec::Explicit {
                    params,
                    body: &args[1],
                    attributes: AttributeSet::empty(),
                })
            } else {
                Some(FunctionSpec::SlotBased {
                    body: &args[0],
                    attributes: parse_attribute_set_expr(&args[1])?,
                })
            }
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") && args.len() == 3 => {
            Some(FunctionSpec::Explicit {
                params: parse_function_parameters(&args[0])?,
                body: &args[1],
                attributes: parse_attribute_set_expr(&args[2])?,
            })
        }
        _ => None,
    }
}

pub(in crate::runtime) fn parse_function_parameters(expr: &Expr) -> Option<Vec<Symbol>> {
    if let Some(symbol) = expr_as_symbol(expr) {
        return Some(vec![symbol.clone()]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter()
        .map(|arg| expr_as_symbol(arg).cloned())
        .collect::<Option<Vec<_>>>()
}

pub(in crate::runtime) fn parse_with_binding_item(expr: &Expr) -> Option<(Symbol, Expr)> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2 =>
        {
            Some((expr_as_symbol(&args[0])?.clone(), args[1].clone()))
        }
        _ => None,
    }
}

pub(in crate::runtime) fn parse_with_binding_list(expr: &Expr) -> Option<Vec<(Symbol, Expr)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(parse_with_binding_item).collect()
}

pub(in crate::runtime) fn parse_module_binding_list(
    expr: &Expr,
) -> Option<Vec<(Symbol, Option<Expr>)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(parse_module_binding_item).collect()
}

pub(in crate::runtime) fn parse_module_binding_item(expr: &Expr) -> Option<(Symbol, Option<Expr>)> {
    if let Some(symbol) = expr_as_symbol(expr) {
        return Some((symbol.clone(), None));
    }

    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2 =>
        {
            Some((expr_as_symbol(&args[0])?.clone(), Some(args[1].clone())))
        }
        _ => None,
    }
}
