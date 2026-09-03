use super::super::*;

pub(in crate::runtime) fn symbol_from_raw_name(raw: &str) -> Symbol {
    if let Some((context, name)) = split_symbol_name(raw) {
        Symbol::new(context, name)
    } else {
        Symbol::from_name(raw)
    }
}

pub(in crate::runtime) fn split_symbol_name(raw: &str) -> Option<(&str, &str)> {
    raw.rfind('`')
        .map(|split_at| (&raw[..=split_at], &raw[split_at + 1..]))
}

pub(in crate::runtime) fn message_name_expr(symbol: Symbol, tag: impl Into<String>) -> Expr {
    Expr::call(
        system_expr("MessageName"),
        vec![Expr::Atom(Atom::Symbol(symbol)), Expr::string(tag.into())],
    )
}

pub(in crate::runtime) fn runtime_message_name_expr(message: RuntimeMessage) -> Expr {
    message_name_expr(symbol_from_raw_name(&message.symbol), message.tag)
}

pub(in crate::runtime) fn held_runtime_message_name_expr(message: RuntimeMessage) -> Expr {
    Expr::call(
        system_expr("HoldForm"),
        vec![runtime_message_name_expr(message)],
    )
}

pub(in crate::runtime) fn format_message_template_with_named(
    template: &str,
    key: &MessageNameKey,
    args: &[Expr],
    named_args: &[(String, Expr)],
) -> String {
    let mut rendered = String::with_capacity(template.len());
    let mut cursor = 0;
    let mut implicit_index = 0;

    while let Some(start_rel) = template[cursor..].find('`') {
        let start = cursor + start_rel;
        rendered.push_str(&template[cursor..start]);
        let token_start = start + 1;

        let Some(end_rel) = template[token_start..].find('`') else {
            rendered.push_str(&template[start..]);
            return rendered;
        };

        let end = token_start + end_rel;
        let token = &template[token_start..end];

        let formatted = if token.is_empty() {
            let formatted = args
                .get(implicit_index)
                .and_then(|arg| format_message_arg_placeholder(arg, None));
            if formatted.is_some() {
                implicit_index += 1;
            }
            formatted
        } else {
            format_message_placeholder(token, key, args)
                .or_else(|| format_named_message_placeholder(token, named_args))
        };

        if let Some(formatted) = formatted {
            rendered.push_str(&formatted);
        } else {
            rendered.push_str(&template[start..=end]);
        }

        cursor = end + 1;
    }

    rendered.push_str(&template[cursor..]);
    rendered
}

fn format_named_message_placeholder(token: &str, named_args: &[(String, Expr)]) -> Option<String> {
    let (base, spec) = token
        .split_once(':')
        .map_or((token, None), |(base, spec)| (base, Some(spec)));
    let (_, value) = named_args.iter().find(|(name, _)| name == base)?;
    format_message_arg_placeholder(value, spec)
}

pub(in crate::runtime) fn format_message_placeholder(
    token: &str,
    key: &MessageNameKey,
    args: &[Expr],
) -> Option<String> {
    let (base, spec) = token
        .split_once(':')
        .map_or((token, None), |(base, spec)| (base, Some(spec)));

    match base {
        "0" => format_message_name_placeholder(key, spec),
        "*" => format_message_star_placeholder(args, spec),
        _ => {
            let index = base.parse::<usize>().ok()?;
            let arg = index.checked_sub(1).and_then(|index| args.get(index))?;
            format_message_arg_placeholder(arg, spec)
        }
    }
}

pub(in crate::runtime) fn format_message_name_placeholder(
    key: &MessageNameKey,
    spec: Option<&str>,
) -> Option<String> {
    match spec {
        None => Some(message_name_expr(key.symbol.clone(), key.tag.clone()).to_string()),
        Some("symbol") => Some(key.symbol.to_string()),
        Some("tag") => Some(key.tag.clone()),
        Some("fullsymbol") => Some(key.symbol.fully_qualified_name()),
        _ => None,
    }
}

pub(in crate::runtime) fn format_message_star_placeholder(
    args: &[Expr],
    spec: Option<&str>,
) -> Option<String> {
    match spec {
        None => Some(
            args.iter()
                .map(format_message_default_fragment)
                .collect::<Vec<_>>()
                .join(", "),
        ),
        Some("count") => Some(args.len().to_string()),
        Some("quoted") => Some(
            args.iter()
                .map(|arg| quote_message_fragment(&arg.to_string()))
                .collect::<Vec<_>>()
                .join(", "),
        ),
        _ => None,
    }
}

pub(in crate::runtime) fn format_message_arg_placeholder(
    arg: &Expr,
    spec: Option<&str>,
) -> Option<String> {
    match spec {
        None => Some(format_message_default_fragment(arg)),
        Some("head") => Some(message_arg_head(arg)),
        Some("name") => Some(expr_as_symbol(arg)?.name().to_string()),
        Some("context") => Some(expr_as_symbol(arg)?.context().to_string()),
        Some("full") => Some(expr_as_symbol(arg)?.fully_qualified_name()),
        Some("quoted") => Some(quote_message_fragment(&arg.to_string())),
        Some("count") => match arg {
            Expr::Call { args, .. } => Some(args.len().to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn format_message_default_fragment(arg: &Expr) -> String {
    match arg {
        Expr::Atom(Atom::String(value)) => value.clone(),
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "HoldForm")
                || is_system_symbol_name(head, "HoldCompleteForm")
                || is_system_symbol_name(head, "Defer")
                || is_system_symbol_name(head, "Unevaluated"))
                && args.len() == 1 =>
        {
            render_input_like_expr(&args[0])
        }
        other => other.to_string(),
    }
}

pub(in crate::runtime) fn message_arg_head(expr: &Expr) -> String {
    match expr {
        Expr::Atom(Atom::Integer(_)) => "Integer".to_string(),
        Expr::Atom(Atom::Rational(_)) => "Rational".to_string(),
        Expr::Atom(Atom::String(_)) => "String".to_string(),
        Expr::Atom(Atom::Symbol(_)) => "Symbol".to_string(),
        Expr::Call { head, .. } => head.to_string(),
    }
}

pub(in crate::runtime) fn quote_message_fragment(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');

    for ch in text.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\t' => quoted.push_str("\\t"),
            other => quoted.push(other),
        }
    }

    quoted.push('"');
    quoted
}
