mod lazy_completion;

use super::*;
use rum_core::{IntValue, SharedArgs};

pub(crate) fn eval_head(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr] => Ok(structural_head_expr(expr)),
        [expr, wrapper] => Ok(Expr::call(
            wrapper.clone(),
            vec![structural_head_expr(expr)],
        )),
        _ => Ok(session.invalid_call_with_arg_range("Head", args, 1, 2)),
    }
}

pub(crate) fn eval_length(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Length", args, 1));
    };

    Ok(Expr::integer(BigInt::from(structural_length(expr))))
}

pub(crate) fn eval_depth(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let default_include_heads = session.system_option_bool("Depth", "Heads", false)?;
    let (expr, include_heads) = match args {
        [expr] => (expr, default_include_heads),
        [expr, option] => {
            let classification = classify_heads_option_expr(option);
            let HeadsOptionClassification::Value(include_heads) = classification else {
                return Ok(session.invalid_heads_option_call(
                    "Depth",
                    args,
                    option,
                    1,
                    classification,
                    "expected a Heads -> True/False option as the second argument",
                ));
            };
            (expr, include_heads.unwrap_or(default_include_heads))
        }
        _ => return Ok(session.invalid_call_with_arg_range("Depth", args, 1, 2)),
    };

    Ok(Expr::integer(BigInt::from(structural_depth(
        expr,
        include_heads,
    ))))
}

pub(crate) fn eval_leaf_count(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LeafCount", args, 1));
    };

    Ok(Expr::integer(BigInt::from(structural_leaf_count(expr))))
}

pub(crate) fn eval_full_form(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("FullForm", args, 1));
    };

    Ok(Expr::string(expr.to_string()))
}

pub(crate) fn eval_first(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if matches!(args.len(), 1 | 2) && args.first().is_some_and(crate::lazy::is_lazy_list) {
        let expr = &args[0];
        let realized = match crate::lazy::realize_at_most(session, expr, 1) {
            Ok(value) => value,
            Err(error) => lazy_completion::capture(session, error, args)?,
        };
        let Some(realized) = realized else {
            return Ok(Expr::call(system_expr("First"), args.to_vec()));
        };
        return Ok(resolve_first_like(
            session,
            &realized,
            args.get(1),
            "First",
            "nofirst",
            true,
        ));
    }

    match args {
        [expr] => Ok(resolve_first_like(
            session, expr, None, "First", "nofirst", true,
        )),
        [expr, default] => Ok(resolve_first_like(
            session,
            expr,
            Some(default),
            "First",
            "nofirst",
            true,
        )),
        _ => Ok(session.invalid_call_with_arg_range("First", args, 1, 2)),
    }
}

pub(crate) fn eval_last(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr] => Ok(resolve_first_like(
            session, expr, None, "Last", "nolast", false,
        )),
        [expr, default] => Ok(resolve_first_like(
            session,
            expr,
            Some(default),
            "Last",
            "nolast",
            false,
        )),
        _ => Ok(session.invalid_call_with_arg_range("Last", args, 1, 2)),
    }
}

pub(crate) fn eval_rest(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Rest", args, 1));
    };

    if crate::lazy::is_lazy_list(expr) {
        return Ok(crate::lazy::lazy_drop(expr, 1)
            .unwrap_or_else(|| Expr::call(system_expr("Rest"), args.to_vec())));
    }

    if let Some(entries) = association_entry_forms_from_expr(expr) {
        if entries.is_empty() {
            return Ok(invalid_empty_sequence_selection_call(
                session, "Rest", "norest", expr,
            ));
        }
        return Ok(render_association_entry_forms_preserving_heads(
            entries.into_iter().skip(1).collect(),
        ));
    }

    Ok(match expr {
        Expr::Atom(_) => invalid_surface_normal_expression_call(session, "Rest", args),
        Expr::Call { args, .. } if args.is_empty() => {
            invalid_empty_sequence_selection_call(session, "Rest", "norest", expr)
        }
        Expr::Call { head, args } => Expr::call(
            head.as_ref().clone(),
            args.iter().skip(1).cloned().collect::<Vec<_>>(),
        ),
    })
}

pub(crate) fn eval_most(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Most", args, 1));
    };

    if let Some(entries) = association_entry_forms_from_expr(expr) {
        if entries.is_empty() {
            return Ok(invalid_empty_sequence_selection_call(
                session, "Most", "nomost", expr,
            ));
        }
        let retained_len = entries.len().saturating_sub(1);
        return Ok(render_association_entry_forms_preserving_heads(
            entries.into_iter().take(retained_len).collect(),
        ));
    }

    Ok(match expr {
        Expr::Atom(_) => invalid_surface_normal_expression_call(session, "Most", args),
        Expr::Call { args, .. } if args.is_empty() => {
            invalid_empty_sequence_selection_call(session, "Most", "nomost", expr)
        }
        Expr::Call { head, args } => Expr::call(
            head.as_ref().clone(),
            args.iter()
                .take(args.len().saturating_sub(1))
                .cloned()
                .collect::<Vec<_>>(),
        ),
    })
}

pub(crate) fn eval_append(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr, item] => Ok(match expr {
            Expr::Atom(_) => invalid_surface_normal_expression_call(session, "Append", args),
            Expr::Call { head, args } => {
                enforce_projected_call_memory_constraint(
                    session,
                    head,
                    args.iter().chain(std::iter::once(item)),
                )?;
                let mut new_args = Vec::with_capacity(args.len().saturating_add(1));
                new_args.extend(args.iter().cloned());
                new_args.push(item.clone());
                Expr::call_with_shared_args(head.as_ref().clone(), SharedArgs::new(new_args))
            }
        }),
        [item] => Ok(unary_operator_form("Append", item.clone())),
        _ => Ok(session.invalid_call_with_arg_range("Append", args, 1, 2)),
    }
}

pub(super) fn enforce_projected_call_memory_constraint<'a>(
    session: &Session,
    head: &Expr,
    items: impl IntoIterator<Item = &'a Expr>,
) -> Result<(), EvalError> {
    if session.active_memory_limit().is_none() {
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    }

    let projected_bytes = items.into_iter().fold(
        std::mem::size_of::<Expr>().saturating_add(projected_expr_memory_proxy_bytes(head)),
        |total, item| total.saturating_add(projected_expr_memory_proxy_bytes(item)),
    );
    session.enforce_active_memory_limit_for_projected_bytes(projected_bytes)
}

pub(super) fn projected_expr_memory_proxy_bytes(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(Atom::Integer(IntValue::Small(_))) => std::mem::size_of::<Expr>(),
        Expr::Atom(Atom::Integer(IntValue::Big(value))) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<BigInt>())
            .saturating_add(projected_bigint_payload_bytes(value)),
        Expr::Atom(Atom::Rational(value)) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<rum_core::Rational>())
            .saturating_add(projected_bigint_payload_bytes(value.numerator()))
            .saturating_add(projected_bigint_payload_bytes(value.denominator())),
        Expr::Atom(Atom::String(value)) => std::mem::size_of::<Expr>().saturating_add(value.len()),
        Expr::Atom(Atom::Symbol(symbol)) => std::mem::size_of::<Expr>()
            .saturating_add(symbol.context().len())
            .saturating_add(symbol.name().len()),
        Expr::Call { head, args } => args.iter().fold(
            std::mem::size_of::<Expr>().saturating_add(projected_expr_memory_proxy_bytes(head)),
            |total, arg| total.saturating_add(projected_expr_memory_proxy_bytes(arg)),
        ),
    }
}

fn projected_bigint_payload_bytes(value: &BigInt) -> usize {
    let bytes = u128::from(value.bits()).saturating_add(7) / 8;
    usize::try_from(bytes.saturating_add(1)).unwrap_or(usize::MAX)
}

pub(crate) fn eval_prepend(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr, item] => Ok(match expr {
            Expr::Atom(_) => invalid_surface_normal_expression_call(session, "Prepend", args),
            Expr::Call { head, args } => {
                let mut new_args = Vec::with_capacity(args.len() + 1);
                new_args.push(item.clone());
                new_args.extend(args.clone());
                Expr::call(head.as_ref().clone(), new_args)
            }
        }),
        [item] => Ok(unary_operator_form("Prepend", item.clone())),
        _ => Ok(session.invalid_call_with_arg_range("Prepend", args, 1, 2)),
    }
}

pub(crate) fn eval_symbol(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [name_expr] = args else {
        return Ok(session.invalid_call_with_message(
            "Symbol",
            args,
            "argx",
            format!(
                "Symbol called with {} arguments; 1 argument is expected.",
                args.len()
            ),
        ));
    };

    let Some(name) = expr_as_string(name_expr) else {
        return Ok(session.invalid_call_with_message(
            "Symbol",
            args,
            "string",
            format!(
                "String expected at position 1 in {}.",
                Expr::call(head.clone(), args.to_vec())
            ),
        ));
    };

    Ok(session
        .resolve_symbol_name(name, true)
        .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
        .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
}

pub(crate) fn eval_construct(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Construct",
            args,
            "argx",
            "expected at least 1 argument",
        ));
    }

    Ok(Expr::call(args[0].clone(), args[1..].to_vec()))
}

fn resolve_first_like(
    session: &mut Session,
    expr: &Expr,
    default: Option<&Expr>,
    symbol: &str,
    empty_tag: &str,
    take_first: bool,
) -> Expr {
    if let Some(entries) = association_entry_forms_from_expr(expr) {
        let selected = if take_first {
            entries.first()
        } else {
            entries.last()
        };
        return selected
            .map(|entry| entry.value.clone())
            .or_else(|| default.cloned())
            .unwrap_or_else(|| {
                invalid_empty_structural_selection_call(session, symbol, empty_tag, expr)
            });
    }

    match expr {
        Expr::Call { args, .. } if !args.is_empty() => {
            if take_first {
                args[0].clone()
            } else {
                args[args.len() - 1].clone()
            }
        }
        Expr::Call { .. } | Expr::Atom(_) if default.is_some() => {
            default.expect("checked above").clone()
        }
        Expr::Call { .. } => {
            invalid_empty_structural_selection_call(session, symbol, empty_tag, expr)
        }
        Expr::Atom(_) => invalid_surface_normal_expression_call(session, symbol, &[expr.clone()]),
    }
}

fn invalid_surface_normal_expression_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    )
}

fn invalid_empty_structural_selection_call(
    session: &mut Session,
    symbol: &str,
    tag: &str,
    expr: &Expr,
) -> Expr {
    session.push_message(
        symbol,
        tag,
        format!(
            "{} has zero length and no {} element.",
            expr,
            selection_name(symbol)
        ),
    );
    Expr::call(system_expr(symbol), vec![expr.clone()])
}

fn invalid_empty_sequence_selection_call(
    session: &mut Session,
    symbol: &str,
    tag: &str,
    expr: &Expr,
) -> Expr {
    session.push_message(
        symbol,
        tag,
        format!(
            "Cannot take {} of expression {} with length zero.",
            symbol, expr
        ),
    );
    Expr::call(system_expr(symbol), vec![expr.clone()])
}

fn unary_operator_form(symbol: &str, item: Expr) -> Expr {
    let slot = Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1))]);
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(system_expr(symbol), vec![slot, item])],
    )
}

fn selection_name(symbol: &str) -> &'static str {
    match symbol {
        "First" => "first",
        "Last" => "last",
        _ => "requested",
    }
}
