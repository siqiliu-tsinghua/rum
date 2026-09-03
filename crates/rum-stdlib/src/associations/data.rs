use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeyedCollectionKind {
    Association,
    RuleList,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct KeyedEntries {
    pub(crate) kind: KeyedCollectionKind,
    pub(crate) entries: Vec<(Expr, Expr)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum KeyedEntriesListError {
    NotList,
    InvalidElement(Expr),
}

pub(crate) fn rule_list_entries_from_expr(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entries(args)
}

pub(crate) fn keyed_entries_from_expr(expr: &Expr) -> Option<KeyedEntries> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(KeyedEntries {
            kind: KeyedCollectionKind::Association,
            entries,
        });
    }

    rule_list_entries_from_expr(expr).map(|entries| KeyedEntries {
        kind: KeyedCollectionKind::RuleList,
        entries,
    })
}

pub(crate) fn single_rule_or_keyed_entries_from_expr(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    if let Some(entry) = parse_association_entry(expr) {
        return Some(vec![entry]);
    }

    keyed_entries_from_expr(expr).map(|keyed| keyed.entries)
}

pub(crate) fn single_rule_or_keyed_entry_forms_from_expr(
    expr: &Expr,
) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entry) = parse_association_entry_form(expr) {
        return Some(vec![entry]);
    }

    if let Some(entries) = association_entry_forms_from_expr(expr) {
        return Some(entries);
    }

    rule_list_entry_forms_from_expr(expr)
}

pub(crate) fn keyed_entry_forms_from_expr(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entries) = association_entry_forms_from_expr(expr) {
        return Some(entries);
    }

    rule_list_entry_forms_from_expr(expr)
}

pub(crate) fn parse_association_entries(items: &[Expr]) -> Option<Vec<(Expr, Expr)>> {
    items.iter().map(parse_association_entry).collect()
}

pub(crate) fn render_association_preserving_rule_heads(args: &[Expr]) -> Option<Expr> {
    Some(render_association_entry_forms_preserving_heads(
        parse_association_entry_forms_deep(args)?,
    ))
}

pub(crate) fn association_rule_exprs_preserving_heads(expr: &Expr) -> Option<Vec<Expr>> {
    Some(
        association_entry_forms_from_expr(expr)?
            .into_iter()
            .map(association_entry_form_expr)
            .collect(),
    )
}

pub(crate) fn parse_key_spec(expr: &Expr) -> Option<Vec<Expr>> {
    let Expr::Call { head, args } = expr else {
        return Some(vec![key_spec_key_expr(expr)]);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Some(vec![key_spec_key_expr(expr)]);
    }

    Some(args.iter().map(key_spec_key_expr).collect())
}

fn key_spec_key_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            args[0].clone()
        }
        _ => expr.clone(),
    }
}

pub(crate) fn parse_association_update_spec_forms(
    spec: &Expr,
) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entry) = parse_association_entry_form(spec) {
        return Some(vec![entry]);
    }

    if let Some(entries) = association_entry_forms_from_expr(spec) {
        return Some(entries);
    }

    let Expr::Call { head, args } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entry_forms_deep(args)
}

pub(crate) fn normalize_key_value_pattern_spec(spec: &Expr) -> Option<Expr> {
    if let Some(entry) = parse_association_entry_form(spec) {
        return Some(Expr::call(
            system_expr("List"),
            vec![association_entry_form_expr(entry)],
        ));
    }

    let Expr::Call { head, args } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let entries = parse_association_entry_forms(args)?;
    Some(Expr::call(
        system_expr("List"),
        entries
            .into_iter()
            .map(association_entry_form_expr)
            .collect(),
    ))
}

pub(crate) fn parse_keyed_entries_list_expr_with_diagnostics(
    expr: &Expr,
) -> Result<Vec<Vec<(Expr, Expr)>>, KeyedEntriesListError> {
    let items = list_items(expr).ok_or(KeyedEntriesListError::NotList)?;
    items
        .iter()
        .map(parse_merge_keyed_entries_collection_with_diagnostics)
        .collect()
}

fn parse_merge_keyed_entries_collection_with_diagnostics(
    expr: &Expr,
) -> Result<Vec<(Expr, Expr)>, KeyedEntriesListError> {
    if let Some(entries) = single_rule_or_keyed_entries_from_expr(expr) {
        return Ok(entries);
    }

    let Some(items) = list_items(expr) else {
        return Err(KeyedEntriesListError::InvalidElement(expr.clone()));
    };

    let mut entries = Vec::new();
    for item in items {
        entries.extend(parse_merge_keyed_entries_collection_with_diagnostics(item)?);
    }
    Ok(entries)
}

pub(crate) fn parse_keyed_entry_forms_list_expr_with_diagnostics(
    expr: &Expr,
) -> Result<Vec<Vec<AssociationEntryForm>>, KeyedEntriesListError> {
    let items = list_items(expr).ok_or(KeyedEntriesListError::NotList)?;
    items
        .iter()
        .map(|item| {
            single_rule_or_keyed_entry_forms_from_expr(item)
                .ok_or_else(|| KeyedEntriesListError::InvalidElement(item.clone()))
        })
        .collect()
}

pub(crate) fn parse_association_entry(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    Some((args[0].clone(), args[1].clone()))
}

fn rule_list_entry_forms_from_expr(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entry_forms(args)
}

fn parse_association_entry_forms(items: &[Expr]) -> Option<Vec<AssociationEntryForm>> {
    items.iter().map(parse_association_entry_form).collect()
}

pub(crate) fn parse_association_entry_forms_deep(
    items: &[Expr],
) -> Option<Vec<AssociationEntryForm>> {
    let mut entries = Vec::new();
    for item in items {
        entries.extend(parse_association_item_forms_deep(item)?);
    }
    Some(entries)
}

fn parse_association_item_forms_deep(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entry) = parse_association_entry_form(expr) {
        return Some(vec![entry]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "Association")
        || is_system_symbol_name(head, heads::structure::LIST)
    {
        return parse_association_entry_forms_deep(args);
    }

    None
}

pub(crate) fn association_entry_form_expr(entry: AssociationEntryForm) -> Expr {
    Expr::call(
        system_expr(if entry.delayed { "RuleDelayed" } else { "Rule" }),
        vec![entry.key, entry.value],
    )
}

pub(crate) fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

pub(crate) fn invalid_association_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association.")
}

pub(crate) fn invalid_association_or_rule_list_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association or a list of rules.")
}

pub(crate) fn invalid_association_or_rule_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association or rule.")
}

pub(crate) fn invalid_rule_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid rule.")
}

pub(crate) fn invalid_association_or_list_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association or a list.")
}

pub(crate) fn invalid_keyed_collection_list_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid list of Associations or rules or lists of rules.")
}

pub(crate) fn invalid_keyed_collection_outer_list_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid list of Associations or rules.")
}

pub(crate) fn invalid_association_update_spec_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a list, Rule or Association.")
}

pub(crate) fn missing_immediate_value_message(symbol: &Symbol, position: usize) -> String {
    format!("The symbol {symbol} at position {position} should have an immediate value defined.")
}

pub(crate) fn list_expected_message(position: usize, expr: &Expr) -> String {
    format!("List expected at position {position} in {expr}.")
}

pub(crate) fn nonempty_list_message(expr: &Expr) -> String {
    format!("Argument {expr} should be a nonempty list.")
}

pub(crate) fn same_length_message(lhs: &Expr, rhs: &Expr) -> String {
    format!("{lhs} and {rhs} must have the same length.")
}

pub(crate) fn protected_tag_write_message(expr: &Expr) -> Option<String> {
    let Expr::Call { head, .. } = expr else {
        return None;
    };

    Some(format!("Tag {head} in {expr} is Protected."))
}

pub(crate) fn keyed_operator_form_slot_first(symbol: &str, tail: Expr) -> Expr {
    let parameter = Expr::symbol("rum$keyedArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(system_expr(symbol), vec![parameter, tail]),
        ],
    )
}

pub(crate) fn keyed_operator_form_slot_second(symbol: &str, head_arg: Expr) -> Expr {
    let parameter = Expr::symbol("rum$keyedArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(system_expr(symbol), vec![head_arg, parameter]),
        ],
    )
}
