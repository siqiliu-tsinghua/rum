use super::*;

pub(super) fn resolve_unqualified_symbol(raw: &str) -> Option<Symbol> {
    split_symbol_name(raw)
        .map(|(context, name)| Symbol::new(normalize_context_name(context), name))
        .or_else(|| is_known_system_symbol_name(raw).then(|| Symbol::system(raw)))
}

pub fn system_expr(name: &str) -> Expr {
    Expr::system_symbol(name)
}

pub(super) fn removed_symbol_expr(symbol: &Symbol) -> Expr {
    Expr::call(
        system_expr("Removed"),
        vec![Expr::string(symbol.to_string())],
    )
}

pub fn expr_as_symbol(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol),
        _ => None,
    }
}

pub fn expr_as_string(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Atom(Atom::String(value)) => Some(value),
        _ => None,
    }
}

pub(super) fn expr_is_self_evaluating_atom(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Atom(Atom::Integer(_) | Atom::Rational(_) | Atom::String(_))
    )
}

pub(super) fn load_spec_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Atom(Atom::String(spec)) => Some(spec.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "File") && args.len() == 1 => {
            match &args[0] {
                Expr::Atom(Atom::String(spec)) => Some(spec.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Extracts a System`List of string atoms into owned Rust strings.
pub fn expr_as_string_list(expr: &Expr) -> Option<Vec<String>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter()
        .map(|arg| match arg {
            Expr::Atom(Atom::String(value)) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

pub fn expr_as_non_negative_usize(expr: &Expr) -> Option<usize> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => value.to_bigint().to_string().parse::<usize>().ok(),
        _ => None,
    }
}

pub fn is_negative_one_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(Atom::Integer(value)) if value.as_i64() == Some(-1))
}

pub fn expr_as_signed_isize(expr: &Expr) -> Option<isize> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => value.to_bigint().to_string().parse::<isize>().ok(),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") && args.len() == 2 => {
            if is_negative_one_expr(&args[0]) {
                expr_as_non_negative_usize(&args[1])
                    .and_then(|value| isize::try_from(value).ok())
                    .map(|value| -value)
            } else if is_negative_one_expr(&args[1]) {
                expr_as_non_negative_usize(&args[0])
                    .and_then(|value| isize::try_from(value).ok())
                    .map(|value| -value)
            } else {
                None
            }
        }
        _ => None,
    }
}
