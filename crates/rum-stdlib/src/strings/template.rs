use super::*;
use rum_core::SharedArgs;

#[derive(Clone, Debug, PartialEq, Eq)]
enum TemplateSegment {
    Text(String),
    PositionalSlot,
    NamedSlot(String),
}

pub(crate) fn eval_string_form(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.is_empty() {
        return Ok(session.invalid_call_with_arg_range("StringForm", args, 1, usize::MAX));
    }

    let Some(template) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    match string_form_value(template, &args[1..]) {
        Ok(value) => Ok(Expr::string(value)),
        Err(message) => Ok(session.invalid_call_with_message("StringForm", args, "slot", message)),
    }
}

pub(crate) fn eval_string_template(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count("StringTemplate", args, 1));
    }

    let Some(template) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(render_template_object(template))
}

pub(crate) fn eval_template_apply(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [template] => Ok(Expr::call(
            system_expr("Function"),
            vec![Expr::call(
                system_expr("TemplateApply"),
                vec![template.clone(), string_slot_expr()],
            )],
        )),
        [template, data] => {
            let Some(template) = expr_as_template_source(template) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            render_template_apply(session, head, args, template, data, &[])
        }
        [template, data, option_args @ ..] => {
            let Some(template) = expr_as_template_source(template) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            render_template_apply(session, head, args, template, data, option_args)
        }
        _ => Ok(session.invalid_call_with_arg_range("TemplateApply", args, 1, 2)),
    }
}

fn render_template_apply(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    template: &str,
    data: &Expr,
    option_args: &[Expr],
) -> Result<Expr, EvalError> {
    let Some(strict) = parse_strict_option(session, "TemplateApply", option_args)? else {
        return Ok(session.invalid_call_with_arg_range("TemplateApply", args, 1, 2));
    };

    if strict {
        return match template_apply_value_strict(template, data) {
            Ok(Some(rendered)) => Ok(Expr::string(rendered)),
            Ok(None) => Ok(Expr::call(head.clone(), args.to_vec())),
            Err(message) => {
                session.push_message("TemplateApply", "slot", message);
                Ok(system_expr("$Failed"))
            }
        };
    }

    let Some(rendered) = template_apply_value(template, data) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    Ok(Expr::string(rendered))
}

fn render_template_object(template: &str) -> Expr {
    Expr::call(
        system_expr("TemplateObject"),
        vec![Expr::string(template.to_string())],
    )
}

fn expr_as_template_source(expr: &Expr) -> Option<&str> {
    if let Some(template) = expr_as_string(expr) {
        return Some(template);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "TemplateObject") || args.len() != 1 {
        return None;
    }

    expr_as_string(&args[0])
}

fn template_apply_value(template: &str, data: &Expr) -> Option<String> {
    let segments = parse_template_segments(template)?;
    let named_slots = template_named_slot_values(data);
    let positional_slots = template_positional_slot_values(data)?;
    let mut positional_index = 0usize;
    let mut output = String::new();

    for segment in segments {
        match segment {
            TemplateSegment::Text(text) => output.push_str(&text),
            TemplateSegment::PositionalSlot => {
                let value = positional_slots.get(positional_index)?;
                output.push_str(value);
                positional_index += 1;
            }
            TemplateSegment::NamedSlot(name) => {
                let value = named_slots.get(&name)?;
                output.push_str(value);
            }
        }
    }

    Some(output)
}

fn template_apply_value_strict(template: &str, data: &Expr) -> Result<Option<String>, String> {
    let Some(segments) = parse_template_segments(template) else {
        return Ok(None);
    };
    let named_slots = template_named_slot_values(data);
    let Some(positional_slots) = template_positional_slot_values(data) else {
        return Ok(None);
    };
    let mut positional_index = 0usize;
    let mut output = String::new();

    for segment in segments {
        match segment {
            TemplateSegment::Text(text) => output.push_str(&text),
            TemplateSegment::PositionalSlot => {
                let Some(value) = positional_slots.get(positional_index) else {
                    return Err(format!(
                        "TemplateApply has no value for positional slot {}.",
                        positional_index + 1
                    ));
                };
                output.push_str(value);
                positional_index += 1;
            }
            TemplateSegment::NamedSlot(name) => {
                let Some(value) = named_slots.get(&name) else {
                    return Err(format!("TemplateApply has no value for slot `{name}`."));
                };
                output.push_str(value);
            }
        }
    }

    Ok(Some(output))
}

fn parse_template_segments(template: &str) -> Option<Vec<TemplateSegment>> {
    let chars = template.chars().collect::<Vec<_>>();
    let mut segments = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '`' {
            index += 1;
            continue;
        }

        let closing = chars[index + 1..]
            .iter()
            .position(|ch| *ch == '`')
            .map(|offset| index + 1 + offset)?;

        if start < index {
            segments.push(TemplateSegment::Text(chars[start..index].iter().collect()));
        }

        let slot_name = chars[index + 1..closing].iter().collect::<String>();
        if slot_name.is_empty() {
            segments.push(TemplateSegment::PositionalSlot);
        } else {
            segments.push(TemplateSegment::NamedSlot(slot_name));
        }

        index = closing + 1;
        start = index;
    }

    if start < chars.len() {
        segments.push(TemplateSegment::Text(chars[start..].iter().collect()));
    }

    Some(segments)
}

fn template_named_slot_values(data: &Expr) -> BTreeMap<String, String> {
    let mut named = BTreeMap::new();

    if let Some(entries) = association_entries_from_expr(data) {
        for (key, value) in entries {
            if let Some(name) = template_slot_name_from_expr(&key) {
                named.insert(name, template_fragment_value(&value));
            }
        }
        return named;
    }

    if let Some(rules) = template_rule_entries(data) {
        for (key, value) in rules {
            if let Some(name) = template_slot_name_from_expr(&key) {
                named.insert(name, template_fragment_value(&value));
            }
        }
    }

    named
}

fn template_positional_slot_values(data: &Expr) -> Option<Vec<String>> {
    if let Some(items) = call_items_with_head(data, &system_expr("List")) {
        if items.iter().all(|item| parse_template_rule(item).is_some()) {
            return Some(Vec::new());
        }
        return Some(items.iter().map(template_fragment_value).collect());
    }

    if association_entries_from_expr(data).is_some() || template_rule_entries(data).is_some() {
        return Some(Vec::new());
    }

    Some(vec![template_fragment_value(data)])
}

fn template_rule_entries(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    if let Some(rule) = parse_template_rule(expr) {
        return Some(vec![rule]);
    }

    let items = call_items_with_head(expr, &system_expr("List"))?;
    items.iter().map(parse_template_rule).collect()
}

fn parse_template_rule(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return None;
    }

    Some((args[0].clone(), args[1].clone()))
}

fn template_slot_name_from_expr(expr: &Expr) -> Option<String> {
    if let Some(value) = expr_as_string(expr) {
        return Some(value.to_string());
    }

    expr_as_symbol(expr).map(|symbol| symbol.name().to_string())
}

fn template_fragment_value(expr: &Expr) -> String {
    match expr {
        Expr::Atom(Atom::String(value)) => value.clone(),
        _ => expr.to_string(),
    }
}

fn string_form_value(template: &str, args: &[Expr]) -> Result<String, String> {
    let chars = template.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;
    let mut next_arg = 0usize;

    while index < chars.len() {
        match chars[index] {
            '\\' if chars.get(index + 1) == Some(&'`') => {
                output.push('`');
                index += 2;
            }
            '`' if chars.get(index + 1) == Some(&'`') => {
                let Some(arg) = args.get(next_arg) else {
                    return Err(
                        "StringForm has more sequential placeholders than arguments.".to_string(),
                    );
                };
                output.push_str(&render_to_string_expr(arg, ToStringFormat::OutputLike));
                next_arg += 1;
                index += 2;
            }
            '`' => {
                let Some(end_offset) = chars[index + 1..].iter().position(|ch| *ch == '`') else {
                    return Err("StringForm found an unterminated placeholder.".to_string());
                };
                let end = index + 1 + end_offset;
                let slot = chars[index + 1..end].iter().collect::<String>();
                let Some(slot_index) = parse_positive_slot_index(&slot) else {
                    return Err(format!(
                        "StringForm placeholder `{slot}` is not a positive integer slot."
                    ));
                };
                let Some(arg) = args.get(slot_index - 1) else {
                    return Err(format!(
                        "StringForm placeholder `{slot}` refers to a missing argument."
                    ));
                };
                output.push_str(&render_to_string_expr(arg, ToStringFormat::OutputLike));
                index = end + 1;
            }
            other => {
                output.push(other);
                index += 1;
            }
        }
    }

    Ok(output)
}

fn parse_positive_slot_index(slot: &str) -> Option<usize> {
    if slot.is_empty() || !slot.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let index = slot.parse::<usize>().ok()?;
    (index > 0).then_some(index)
}
