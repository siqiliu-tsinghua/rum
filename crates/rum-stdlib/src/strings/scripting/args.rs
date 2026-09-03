use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_parse_args(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [items, option_args @ ..] = args else {
        return Ok(session.invalid_call_with_arg_range("ParseArgs", args, 1, usize::MAX));
    };
    let Expr::Call {
        head: items_head,
        args: item_exprs,
    } = items
    else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if !is_system_symbol_name(items_head, "List")
        || !item_exprs
            .iter()
            .all(|item| matches!(item, Expr::Atom(Atom::String(_))))
    {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }
    let Some(options) = parse_parse_args_options(session, option_args)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    enforce_parse_args_memory_constraint(session, item_exprs, options.short_options)?;
    let items = item_exprs
        .iter()
        .map(|item| match item {
            Expr::Atom(Atom::String(value)) => value.clone(),
            _ => unreachable!("ParseArgs string-list validation should be stable"),
        })
        .collect();

    Ok(parse_args(items, options.short_options).into_expr())
}

fn enforce_parse_args_memory_constraint(
    session: &Session,
    items: &[Expr],
    short_options: bool,
) -> Result<(), EvalError> {
    if session.active_memory_limit().is_none() {
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    }

    let mut parse_options = true;
    let mut positional_count = 0usize;
    let mut option_count = 0usize;
    let mut flag_count = 0usize;
    // Fixed string payload: the three field keys (22 bytes), two System`
    // Association heads (36), three System`Rule heads (33), and two
    // System`List heads (22).
    let mut payload_bytes = 113usize;

    for item in items {
        let Expr::Atom(Atom::String(item)) = item else {
            continue;
        };
        if parse_options && item == "--" {
            parse_options = false;
        } else if parse_options && item.starts_with("--") {
            let item = &item[2..];
            if let Some((key, value)) = item.split_once('=') {
                option_count = option_count.saturating_add(1);
                payload_bytes = payload_bytes
                    .saturating_add(11) // The option's System`Rule head.
                    .saturating_add(key.len())
                    .saturating_add(value.len());
            } else {
                flag_count = flag_count.saturating_add(1);
                payload_bytes = payload_bytes.saturating_add(item.len());
            }
        } else if parse_options && short_options && item.starts_with('-') && item != "-" {
            for flag in item[1..].chars() {
                flag_count = flag_count.saturating_add(1);
                payload_bytes = payload_bytes.saturating_add(flag.len_utf8());
            }
        } else {
            positional_count = positional_count.saturating_add(1);
            payload_bytes = payload_bytes.saturating_add(item.len());
        }
    }

    // Association + three field rules/lists contribute 17 logical Expr nodes.
    // Canonicalizing the outer three-entry association leaves five additional
    // Expr slots in its reused pair allocation. Each option contributes its
    // Rule plus key/value (four nodes), and the nested association likewise
    // reuses a pair allocation with capacity max(4, next_power_of_two(count)).
    let option_entry_capacity = if option_count == 0 {
        0
    } else {
        option_count
            .checked_next_power_of_two()
            .unwrap_or(usize::MAX)
            .max(4)
    };
    let option_spare_nodes = option_entry_capacity
        .checked_mul(2)
        .and_then(|slots| slots.checked_sub(option_count))
        .unwrap_or(usize::MAX);
    let output_nodes = 22usize
        .saturating_add(positional_count)
        .saturating_add(flag_count)
        .saturating_add(option_count.saturating_mul(4))
        .saturating_add(option_spare_nodes);
    let projected_bytes = output_nodes
        .saturating_mul(std::mem::size_of::<Expr>())
        .saturating_add(payload_bytes);
    session.enforce_active_memory_limit_for_projected_bytes(projected_bytes)
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ParsedArgs {
    positional: Vec<String>,
    options: Vec<(String, String)>,
    flags: Vec<String>,
}

impl ParsedArgs {
    fn into_expr(self) -> Expr {
        render_association(vec![
            (
                Expr::string("Positional"),
                string_list_expr(self.positional),
            ),
            (
                Expr::string("Options"),
                render_association(
                    self.options
                        .into_iter()
                        .map(|(key, value)| (Expr::string(key), Expr::string(value)))
                        .collect(),
                ),
            ),
            (Expr::string("Flags"), string_list_expr(self.flags)),
        ])
    }
}

struct ParseArgsOptions {
    short_options: bool,
}

fn parse_parse_args_options(
    session: &mut Session,
    explicit: &[Expr],
) -> Result<Option<ParseArgsOptions>, EvalError> {
    let option_exprs = session.merged_option_exprs_for_system_symbol("ParseArgs", explicit)?;
    let mut options = ParseArgsOptions {
        short_options: false,
    };

    for expr in option_exprs {
        let Some((name, value)) = option_rule(&expr) else {
            return Ok(None);
        };
        match name {
            "ShortOptions" => {
                let Some(value) = bool_option_value(value) else {
                    return Ok(None);
                };
                options.short_options = value;
            }
            _ => return Ok(None),
        }
    }

    Ok(Some(options))
}

fn parse_args(items: Vec<String>, short_options: bool) -> ParsedArgs {
    let mut parsed = ParsedArgs::default();
    let mut parse_options = true;

    for item in items {
        if parse_options && item == "--" {
            parse_options = false;
        } else if parse_options && item.starts_with("--") {
            let item = &item[2..];
            if let Some((key, value)) = item.split_once('=') {
                parsed.options.push((key.to_string(), value.to_string()));
            } else {
                parsed.flags.push(item.to_string());
            }
        } else if parse_options && short_options && item.starts_with('-') && item != "-" {
            for flag in item[1..].chars() {
                parsed.flags.push(flag.to_string());
            }
        } else {
            parsed.positional.push(item);
        }
    }

    parsed
}

fn option_rule(expr: &Expr) -> Option<(&str, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }
    Some((option_name(&args[0])?, &args[1]))
}

fn option_name(expr: &Expr) -> Option<&str> {
    if let Some(name) = expr_as_string(expr) {
        return Some(name);
    }
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name()),
        _ => None,
    }
}

fn bool_option_value(expr: &Expr) -> Option<bool> {
    if is_system_symbol_name(expr, "True") {
        Some(true)
    } else if is_system_symbol_name(expr, "False") {
        Some(false)
    } else {
        None
    }
}
