use super::*;
use crate::option_consumers::OptionConsumer;
use rum_core::SharedArgs;

pub(crate) fn eval_string_wrap(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_string_wrap_with_options(session, head, args, None)
}

pub(crate) fn eval_string_wrap_with_options(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    resolved_options: Option<Vec<Expr>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [text, width, option_args @ ..] = args else {
        return Ok(session.invalid_call_with_arg_range("StringWrap", args, 2, usize::MAX));
    };
    let (Some(text), Some(width)) = (expr_as_string(text), expr_as_non_negative_usize(width))
    else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if width == 0 {
        return Ok(session.invalid_call_with_arg_form(
            "StringWrap",
            args,
            "width must be a positive integer",
        ));
    }
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::StringWrap,
                head,
                args,
                option_args,
            );
        }
    };
    let Some(options) = parse_string_wrap_options(option_exprs)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(if options.preserve_newlines {
        string_wrap_preserving_newlines(text, width)
    } else {
        string_wrap_value(text, width)
    }))
}

pub(crate) fn eval_string_indent(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_string_indent_with_options(session, head, args, None)
}

pub(crate) fn eval_string_indent_with_options(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    resolved_options: Option<Vec<Expr>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [text, prefix, option_args @ ..] = args else {
        return Ok(session.invalid_call_with_arg_range("StringIndent", args, 2, usize::MAX));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let prefix = if let Some(prefix) = expr_as_string(prefix) {
        StringIndentPrefix::Text(prefix)
    } else if let Some(count) = expr_as_non_negative_usize(prefix) {
        StringIndentPrefix::Spaces(count)
    } else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::StringIndent,
                head,
                args,
                option_args,
            );
        }
    };
    let Some(options) = parse_string_indent_options(option_exprs)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let prefix = match prefix {
        StringIndentPrefix::Text(prefix) => prefix.to_string(),
        StringIndentPrefix::Spaces(count) => {
            let line_count = text
                .split('\n')
                .filter(|line| options.blank_lines || !line.trim().is_empty())
                .count();
            enforce_generated_product(session, count, line_count)?;
            " ".repeat(count)
        }
    };

    Ok(Expr::string(indent_lines(
        text,
        &prefix,
        options.blank_lines,
    )))
}

pub(crate) fn eval_string_dedent(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [text] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("StringDedent", args, 1));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(dedent_lines(text)))
}

pub(crate) fn eval_string_expand_tabs(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (text, width) = match args {
        [text] => (text, 8),
        [text, width] => {
            let Some(width) = expr_as_non_negative_usize(width) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            (text, width)
        }
        _ => return Ok(session.invalid_call_with_arg_range("StringExpandTabs", args, 1, 2)),
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if width == 0 {
        return Ok(session.invalid_call_with_arg_form(
            "StringExpandTabs",
            args,
            "tab width must be a positive integer",
        ));
    }

    enforce_expanded_tab_space_count(session, text, width)?;
    Ok(Expr::string(expand_tabs(text, width)))
}

pub(crate) fn eval_align_columns(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_align_columns_with_options(session, head, args, None)
}

pub(crate) fn eval_align_columns_with_options(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    resolved_options: Option<Vec<Expr>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [rows, option_args @ ..] = args else {
        return Ok(session.invalid_call_with_arg_range("AlignColumns", args, 1, usize::MAX));
    };
    let Some(rows) = nested_string_rows(rows) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::AlignColumns,
                head,
                args,
                option_args,
            );
        }
    };
    let Some(options) = parse_align_columns_options(option_exprs)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(align_columns(rows, &options.separator)))
}

struct StringWrapOptions {
    preserve_newlines: bool,
}

struct StringIndentOptions {
    blank_lines: bool,
}

enum StringIndentPrefix<'a> {
    Text(&'a str),
    Spaces(usize),
}

struct AlignColumnsOptions {
    separator: String,
}

fn parse_string_wrap_options(
    option_exprs: Vec<Expr>,
) -> Result<Option<StringWrapOptions>, EvalError> {
    let mut options = StringWrapOptions {
        preserve_newlines: false,
    };

    for expr in option_exprs {
        let Some((name, value)) = option_rule(&expr) else {
            return Ok(None);
        };
        match name {
            "PreserveNewlines" => {
                let Some(value) = bool_option_value(value) else {
                    return Ok(None);
                };
                options.preserve_newlines = value;
            }
            _ => return Ok(None),
        }
    }

    Ok(Some(options))
}

fn parse_string_indent_options(
    option_exprs: Vec<Expr>,
) -> Result<Option<StringIndentOptions>, EvalError> {
    let mut options = StringIndentOptions { blank_lines: false };

    for expr in option_exprs {
        let Some((name, value)) = option_rule(&expr) else {
            return Ok(None);
        };
        match name {
            "BlankLines" => {
                let Some(value) = bool_option_value(value) else {
                    return Ok(None);
                };
                options.blank_lines = value;
            }
            _ => return Ok(None),
        }
    }

    Ok(Some(options))
}

fn parse_align_columns_options(
    option_exprs: Vec<Expr>,
) -> Result<Option<AlignColumnsOptions>, EvalError> {
    let mut options = AlignColumnsOptions {
        separator: "  ".to_string(),
    };

    for expr in option_exprs {
        let Some((name, value)) = option_rule(&expr) else {
            return Ok(None);
        };
        match name {
            "Separator" => {
                let Some(value) = expr_as_string(value) else {
                    return Ok(None);
                };
                options.separator = value.to_string();
            }
            _ => return Ok(None),
        }
    }

    Ok(Some(options))
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

fn enforce_generated_product(
    session: &Session,
    count: usize,
    repetitions: usize,
) -> Result<(), EvalError> {
    if let Some(generated_count) = count.checked_mul(repetitions) {
        return session.enforce_generated_element_limit_for_count(generated_count);
    }

    let generated_count = BigInt::from(count) * BigInt::from(repetitions);
    session.enforce_generated_element_limit_for_big_count(&generated_count)
}

fn enforce_expanded_tab_space_count(
    session: &Session,
    text: &str,
    width: usize,
) -> Result<(), EvalError> {
    let Some(generated_count) = expanded_tab_space_count(text, width) else {
        let overflow_count = BigInt::from(usize::MAX) + BigInt::from(1_u8);
        return session.enforce_generated_element_limit_for_big_count(&overflow_count);
    };
    session.enforce_generated_element_limit_for_count(generated_count)
}

fn expanded_tab_space_count(text: &str, width: usize) -> Option<usize> {
    let mut generated_count = 0_usize;
    let mut column = 0_usize;
    for ch in text.chars() {
        match ch {
            '\t' => {
                let spaces = width - (column % width);
                generated_count = generated_count.checked_add(spaces)?;
                column = column.checked_add(spaces)?;
            }
            '\n' | '\r' => column = 0,
            _ => column = column.checked_add(1)?,
        }
    }
    Some(generated_count)
}

fn string_wrap_value(text: &str, width: usize) -> String {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;

    for word in text.split_whitespace() {
        let word_width = word.chars().count();
        if word_width > width {
            if !current.is_empty() {
                lines.push(current);
                current = String::new();
                current_width = 0;
            }
            lines.push(word.to_string());
            continue;
        }

        if current.is_empty() {
            current.push_str(word);
            current_width = word_width;
        } else if current_width + 1 + word_width <= width {
            current.push(' ');
            current.push_str(word);
            current_width += 1 + word_width;
        } else {
            lines.push(current);
            current = word.to_string();
            current_width = word_width;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    lines.join("\n")
}

fn string_wrap_preserving_newlines(text: &str, width: usize) -> String {
    text.split('\n')
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                string_wrap_value(line, width)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn indent_lines(text: &str, prefix: &str, blank_lines: bool) -> String {
    text.split('\n')
        .map(|line| {
            if !blank_lines && line.trim().is_empty() {
                line.to_string()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn dedent_lines(text: &str) -> String {
    let lines = text.split('\n').collect::<Vec<_>>();
    let Some(prefix) = common_leading_whitespace(&lines) else {
        return text.to_string();
    };
    if prefix.is_empty() {
        return text.to_string();
    }

    lines
        .into_iter()
        .map(|line| line.strip_prefix(&prefix).unwrap_or(line).to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn common_leading_whitespace(lines: &[&str]) -> Option<String> {
    let mut common: Option<Vec<char>> = None;
    for line in lines.iter().copied().filter(|line| !line.trim().is_empty()) {
        let leading = line
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .collect::<Vec<_>>();
        common = Some(match common {
            None => leading,
            Some(current) => current
                .into_iter()
                .zip(leading)
                .take_while(|(left, right)| left == right)
                .map(|(ch, _)| ch)
                .collect(),
        });
    }
    common.map(|chars| chars.into_iter().collect())
}

fn expand_tabs(text: &str, width: usize) -> String {
    let mut output = String::new();
    let mut column = 0usize;
    for ch in text.chars() {
        match ch {
            '\t' => {
                let spaces = width - (column % width);
                output.push_str(&" ".repeat(spaces));
                column += spaces;
            }
            '\n' => {
                output.push(ch);
                column = 0;
            }
            '\r' => {
                output.push(ch);
                column = 0;
            }
            other => {
                output.push(other);
                column += 1;
            }
        }
    }
    output
}

fn nested_string_rows(expr: &Expr) -> Option<Vec<Vec<String>>> {
    let rows = call_items_with_head(expr, &system_expr("List"))?;
    rows.iter()
        .map(|row| {
            call_items_with_head(row, &system_expr("List"))?
                .iter()
                .map(|cell| expr_as_string(cell).map(str::to_string))
                .collect()
        })
        .collect()
}

fn align_columns(rows: Vec<Vec<String>>, separator: &str) -> String {
    let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0usize; column_count];
    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.chars().count());
        }
    }

    rows.into_iter()
        .map(|row| {
            let mut line = String::new();
            for (index, cell) in row.iter().enumerate() {
                if index > 0 {
                    line.push_str(separator);
                }
                line.push_str(cell);
                if index + 1 < row.len() {
                    let padding = widths[index].saturating_sub(cell.chars().count());
                    line.push_str(&" ".repeat(padding));
                }
            }
            line.trim_end().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
