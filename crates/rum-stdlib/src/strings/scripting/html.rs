use super::*;
use crate::option_consumers::OptionConsumer;
use rum_core::SharedArgs;

#[cfg(test)]
std::thread_local! {
    static HTML_UNESCAPE_SCAN_STEPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn record_html_unescape_scan_step() {
    HTML_UNESCAPE_SCAN_STEPS.set(HTML_UNESCAPE_SCAN_STEPS.get() + 1);
}

#[cfg(test)]
fn reset_html_unescape_scan_steps() {
    HTML_UNESCAPE_SCAN_STEPS.set(0);
}

#[cfg(test)]
fn html_unescape_scan_steps() -> usize {
    HTML_UNESCAPE_SCAN_STEPS.get()
}

pub(crate) fn eval_html_escape(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [text] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("HtmlEscape", args, 1));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(html_escape(text)))
}

pub(crate) fn eval_html_unescape(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_html_unescape_with_options(session, head, args, None)
}

pub(crate) fn eval_html_unescape_with_options(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    resolved_options: Option<rum_runtime::kernel_api::RetainedOptions<'_>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [text, option_args @ ..] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("HtmlUnescape", args, 1));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::HtmlUnescape,
                head,
                args,
                option_args,
            );
        }
    };
    let Some(strict) = parse_strict_option(option_exprs)? else {
        return Ok(session.invalid_call_with_exact_arg_count("HtmlUnescape", args, 1));
    };

    if strict {
        return match html_unescape_strict(text) {
            Ok(value) => Ok(Expr::string(value)),
            Err(message) => Ok(failed_with_message(
                session,
                "HtmlUnescape",
                "entity",
                message,
            )),
        };
    }

    Ok(Expr::string(html_unescape(text)))
}

fn html_escape(text: &str) -> String {
    let mut output = String::new();
    for ch in text.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            other => output.push(other),
        }
    }
    output
}

fn html_unescape(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '&' {
            output.push(chars[index]);
            index += 1;
            continue;
        }

        let Some(end_offset) = chars[index + 1..].iter().position(|ch| {
            #[cfg(test)]
            record_html_unescape_scan_step();
            *ch == ';'
        }) else {
            output.extend(chars[index..].iter());
            break;
        };
        let end = index + 1 + end_offset;
        let entity = chars[index + 1..end].iter().collect::<String>();
        if let Some(decoded) = decode_entity(&entity) {
            output.push(decoded);
            index = end + 1;
        } else if let Some(decoded) = decode_named_entity(&entity) {
            output.push_str(decoded);
            index = end + 1;
        } else {
            output.extend(chars[index..=end].iter());
            index = end + 1;
        }
    }

    output
}

fn html_unescape_strict(text: &str) -> Result<String, String> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '&' {
            output.push(chars[index]);
            index += 1;
            continue;
        }

        let Some(end_offset) = chars[index + 1..].iter().position(|ch| *ch == ';') else {
            return Err("HtmlUnescape found an unterminated entity.".to_string());
        };
        let end = index + 1 + end_offset;
        let entity = chars[index + 1..end].iter().collect::<String>();
        if entity.is_empty() {
            return Err("HtmlUnescape found an empty entity.".to_string());
        }
        if let Some(decoded) = decode_entity(&entity) {
            output.push(decoded);
            index = end + 1;
        } else if entity.starts_with('#') {
            return Err("HtmlUnescape found a malformed numeric entity.".to_string());
        } else if let Some(decoded) = decode_named_entity(&entity) {
            output.push_str(decoded);
            index = end + 1;
        } else {
            output.extend(chars[index..=end].iter());
            index = end + 1;
        }
    }

    Ok(output)
}

fn decode_named_entity(entity: &str) -> Option<&'static str> {
    match entity {
        "amp" => Some("&"),
        "lt" => Some("<"),
        "gt" => Some(">"),
        "quot" => Some("\""),
        "apos" => Some("'"),
        _ => None,
    }
}

fn decode_entity(entity: &str) -> Option<char> {
    let numeric = entity.strip_prefix('#')?;
    let value = if let Some(hex) = numeric
        .strip_prefix('x')
        .or_else(|| numeric.strip_prefix('X'))
    {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        numeric.parse::<u32>().ok()?
    };
    char::from_u32(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unterminated_entity_scan_work_probe() {
        let width = 256;
        let input = "&".repeat(width);

        reset_html_unescape_scan_steps();
        assert_eq!(html_unescape(&input), input);
        assert_eq!(
            html_unescape_scan_steps(),
            width - 1,
            "an unterminated suffix should be searched only once"
        );
        assert_eq!(html_unescape("prefix &&é"), "prefix &&é");
    }
}
