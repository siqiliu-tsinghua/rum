use super::*;

pub(super) fn parse_replace_part_rules(
    expr: &Expr,
    heads: ReplacePartHeadsMode,
) -> Result<Option<Vec<ReplacePartRule>>, Expr> {
    if let Some(rule) = parse_replace_part_rule_with_heads(expr, heads)? {
        return Ok(Some(vec![rule]));
    }

    let Expr::Call { head, args } = expr else {
        return Ok(None);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Ok(None);
    }

    let mut rules = Vec::with_capacity(args.len());
    for item in args {
        let Some(rule) = parse_replace_part_rule_with_heads(item, heads)? else {
            return Ok(None);
        };
        rules.push(rule);
    }
    Ok(Some(rules))
}

fn parse_replace_part_rule_with_heads(
    expr: &Expr,
    heads: ReplacePartHeadsMode,
) -> Result<Option<ReplacePartRule>, Expr> {
    let Expr::Call { head, args } = expr else {
        return Ok(None);
    };

    let delayed = if is_system_symbol_name(head, heads::rules::RULE) && args.len() == 2 {
        false
    } else if is_system_symbol_name(head, heads::rules::RULE_DELAYED) && args.len() == 2 {
        true
    } else {
        return Ok(None);
    };
    let (rhs, guards) = split_definition_rhs_guards(args[1].clone());

    Ok(Some(ReplacePartRule {
        position_specs: parse_replace_part_position_specs(&args[0])?,
        rhs,
        guards,
        delayed,
        heads,
    }))
}

fn parse_replace_part_position_specs(expr: &Expr) -> Result<Vec<ReplacePartPositionSpec>, Expr> {
    if is_empty_plain_list_expr(expr) {
        return Ok(Vec::new());
    }

    if let Some(position_spec) = parse_replace_part_exact_position_spec(expr)? {
        return Ok(vec![ReplacePartPositionSpec::Exact(position_spec)]);
    }

    let Expr::Call { head, args } = expr else {
        return Ok(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Ok(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    }

    if !args.iter().any(is_plain_list_expr) {
        return Ok(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    }

    let mut position_specs = Vec::with_capacity(args.len());
    for item in args {
        position_specs.push(
            parse_replace_part_exact_position_spec(item)?
                .map(ReplacePartPositionSpec::Exact)
                .unwrap_or_else(|| ReplacePartPositionSpec::Pattern(item.clone())),
        );
    }
    Ok(position_specs)
}

fn parse_replace_part_exact_position_spec(expr: &Expr) -> Result<Option<Vec<PartSelector>>, Expr> {
    if let Some(selector) = parse_replace_part_exact_selector(expr)? {
        return Ok(Some(vec![selector]));
    }

    let Expr::Call { head, args } = expr else {
        return Ok(None);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Ok(None);
    }

    if args.is_empty() {
        return Ok(Some(Vec::new()));
    }

    if args.iter().all(is_plain_list_expr) {
        return Ok(None);
    }

    let mut selectors = Vec::with_capacity(args.len());
    let mut all_exact = true;
    for item in args {
        match parse_replace_part_exact_selector(item)? {
            Some(selector) => selectors.push(selector),
            None => all_exact = false,
        }
    }
    Ok(all_exact.then_some(selectors))
}

fn parse_replace_part_exact_selector(expr: &Expr) -> Result<Option<PartSelector>, Expr> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Ok(Some(PartSelector::Single(AtomicPartSpec::Index(index))));
    }

    if matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::SPAN)
    ) {
        let selector = parse_part_selectors(std::slice::from_ref(expr))
            .and_then(|mut selectors| selectors.pop())
            .ok_or_else(|| expr.clone())?;
        if matches!(
            &selector,
            PartSelector::Single(AtomicPartSpec::Span(span)) if span.step == 0
        ) {
            return Err(expr.clone());
        }
        return Ok(Some(selector));
    }

    Ok(parse_replace_part_key_selector(expr).map(PartSelector::Key))
}

fn parse_replace_part_key_selector(expr: &Expr) -> Option<Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Key") || args.len() != 1 {
        return None;
    }

    Some(args[0].clone())
}

pub(super) fn parse_replace_part_heads_option(expr: &Expr) -> Option<ReplacePartHeadsMode> {
    match parse_heads_option(expr)? {
        true => Some(ReplacePartHeadsMode::Include),
        false => Some(ReplacePartHeadsMode::Exclude),
    }
}
