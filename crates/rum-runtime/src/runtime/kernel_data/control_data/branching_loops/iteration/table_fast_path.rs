use super::spec::parse_do_spec_iteration_count;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum IteratorListTemplateArg {
    Iterator,
    Constant(Expr),
}

pub(super) fn eval_single_spec_constant_table_fast_path(
    session: &Session,
    body: &Expr,
    spec_expr: &Expr,
) -> Result<Option<Expr>, EvalError> {
    if !table_fast_path_preserves_observability(session)
        || !table_body_is_repeated_constant(session, body)
    {
        return Ok(None);
    }

    let Some(count) = parse_do_spec_iteration_count(session, spec_expr, IterationConsumer::Table)?
    else {
        return Ok(None);
    };

    Ok(Some(Expr::call(
        system_expr("List"),
        vec![body.clone(); count],
    )))
}

pub(super) fn eval_single_spec_table_fast_path(
    session: &Session,
    body: &Expr,
    spec: DoSpec,
) -> Result<Expr, DoSpec> {
    if !table_fast_path_preserves_observability(session) {
        return Err(spec);
    }

    if table_body_is_repeated_constant(session, body) {
        return Ok(Expr::call(
            system_expr("List"),
            vec![body.clone(); spec.iteration_count()],
        ));
    }

    let DoSpec::Iterator { symbol, values } = spec else {
        return Err(spec);
    };
    if expr_as_symbol(body).is_some_and(|body_symbol| *body_symbol == symbol) {
        return Ok(Expr::call(system_expr("List"), values));
    }

    match eval_single_spec_integer_plus_table_fast_path(session, body, symbol, values) {
        Ok(result) => Ok(result),
        Err(DoSpec::Iterator { symbol, values }) => {
            eval_single_spec_integer_list_template_table_fast_path(session, body, symbol, values)
        }
        Err(spec) => Err(spec),
    }
}

fn eval_single_spec_integer_plus_table_fast_path(
    session: &Session,
    body: &Expr,
    symbol: Symbol,
    values: Vec<Expr>,
) -> Result<Expr, DoSpec> {
    let Some(offset) = iterator_integer_plus_offset(session, body, &symbol) else {
        return Err(DoSpec::Iterator { symbol, values });
    };
    if !values
        .iter()
        .all(|value| matches!(value, Expr::Atom(Atom::Integer(_))))
    {
        return Err(DoSpec::Iterator { symbol, values });
    }

    let items = values
        .into_iter()
        .map(|value| match value {
            Expr::Atom(Atom::Integer(integer)) => {
                Expr::integer(integer.to_bigint() + offset.clone())
            }
            _ => unreachable!("values were prechecked as integers"),
        })
        .collect();
    Ok(Expr::call(system_expr("List"), items))
}

fn eval_single_spec_integer_list_template_table_fast_path(
    session: &Session,
    body: &Expr,
    symbol: Symbol,
    values: Vec<Expr>,
) -> Result<Expr, DoSpec> {
    let Some(template) = iterator_integer_list_template(session, body, &symbol) else {
        return Err(DoSpec::Iterator { symbol, values });
    };
    if !values
        .iter()
        .all(|value| matches!(value, Expr::Atom(Atom::Integer(_))))
    {
        return Err(DoSpec::Iterator { symbol, values });
    }

    let list_head = SharedExpr::new(system_expr("List"));
    let items = values
        .into_iter()
        .map(|value| instantiate_integer_list_template(&template, value, &list_head))
        .collect();
    Ok(Expr::call_shared(list_head, SharedArgs::new(items)))
}

fn iterator_integer_list_template(
    session: &Session,
    body: &Expr,
    symbol: &Symbol,
) -> Option<Vec<IteratorListTemplateArg>> {
    if !system_builtin_symbol_has_default_runtime_semantics(session, "List") {
        return None;
    }

    let Expr::Call { head, args } = body else {
        return None;
    };
    if !is_system_symbol_name(head, "List")
        || !args
            .iter()
            .all(|arg| iterator_list_template_arg_is_plain(arg, symbol))
    {
        return None;
    }

    args.iter()
        .map(|arg| iterator_list_template_arg(arg, symbol))
        .collect()
}

fn iterator_list_template_arg(arg: &Expr, symbol: &Symbol) -> Option<IteratorListTemplateArg> {
    if expr_as_symbol(arg).is_some_and(|arg_symbol| arg_symbol == symbol) {
        return Some(IteratorListTemplateArg::Iterator);
    }
    iteration_body_is_self_evaluating_atom(arg)
        .then(|| IteratorListTemplateArg::Constant(arg.clone()))
}

fn iterator_list_template_arg_is_plain(arg: &Expr, symbol: &Symbol) -> bool {
    iterator_list_template_arg(arg, symbol).is_some()
}

fn instantiate_integer_list_template(
    template: &[IteratorListTemplateArg],
    value: Expr,
    list_head: &SharedExpr,
) -> Expr {
    let mut remaining_iterator_slots = template
        .iter()
        .filter(|arg| matches!(arg, IteratorListTemplateArg::Iterator))
        .count();
    let mut value = Some(value);
    let items = template
        .iter()
        .map(|arg| match arg {
            IteratorListTemplateArg::Iterator => {
                remaining_iterator_slots -= 1;
                if remaining_iterator_slots == 0 {
                    value.take().expect("last iterator slot consumes value")
                } else {
                    value
                        .as_ref()
                        .expect("iterator value remains available before last slot")
                        .clone()
                }
            }
            IteratorListTemplateArg::Constant(expr) => expr.clone(),
        })
        .collect();
    Expr::call_shared(list_head.clone(), SharedArgs::new(items))
}

fn iterator_integer_plus_offset(session: &Session, body: &Expr, symbol: &Symbol) -> Option<BigInt> {
    if !system_builtin_symbol_has_default_runtime_semantics(session, "Plus") {
        return None;
    }

    let Expr::Call { head, args } = body else {
        return None;
    };
    if !is_system_symbol_name(head, "Plus") || args.len() != 2 {
        return None;
    }

    match args.as_slice() {
        [lhs, rhs] if expr_as_symbol(lhs).is_some_and(|lhs_symbol| lhs_symbol == symbol) => {
            expr_as_exact_integer(rhs)
        }
        [lhs, rhs] if expr_as_symbol(rhs).is_some_and(|rhs_symbol| rhs_symbol == symbol) => {
            expr_as_exact_integer(lhs)
        }
        _ => None,
    }
}

fn system_builtin_symbol_has_default_runtime_semantics(session: &Session, name: &str) -> bool {
    let symbol = Symbol::system(name);
    session.attribute_set_for_symbol(&symbol) == default_system_attribute_set(&symbol)
        && session
            .definitions
            .get(&symbol)
            .is_none_or(SymbolDefinition::is_empty)
        && !session
            .dynamic_own_values
            .iter()
            .any(|frame| frame.symbol == symbol)
        && !session.removed_symbols.contains(&symbol)
}

fn table_fast_path_preserves_observability(session: &Session) -> bool {
    !session.tracing_enabled
        && (session.monitor_scopes.is_empty() || !session.monitor_callbacks_enabled)
}

fn table_body_is_repeated_constant(session: &Session, expr: &Expr) -> bool {
    iteration_body_is_self_evaluating_atom(expr)
        || expr_as_symbol(expr).is_some_and(|symbol| {
            symbol.context() == SYSTEM_CONTEXT
                && matches!(symbol.name(), "Null" | "True" | "False")
                && system_builtin_symbol_has_default_runtime_semantics(session, symbol.name())
        })
}

fn iteration_body_is_self_evaluating_atom(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Atom(Atom::Integer(_) | Atom::Rational(_) | Atom::String(_))
    )
}
