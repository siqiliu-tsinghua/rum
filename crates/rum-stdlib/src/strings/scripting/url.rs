use crate::option_consumers::OptionConsumer;
use rum_core::SharedArgs;
use std::str::FromStr;

use super::*;

pub(crate) fn eval_url_encode(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_url_encode_with_options(session, head, args, None)
}

pub(crate) fn eval_url_encode_with_options(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    resolved_options: Option<Vec<Expr>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [text, option_args @ ..] = args else {
        return Ok(session.invalid_call_with_arg_range("UrlEncode", args, 1, usize::MAX));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::UrlEncode,
                head,
                args,
                option_args,
            );
        }
    };
    let Some(options) = parse_url_form_options(option_exprs)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(url_encode(text, options.form_encoding)))
}

pub(crate) fn eval_url_decode(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_url_decode_with_options(session, head, args, None)
}

pub(crate) fn eval_url_decode_with_options(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    resolved_options: Option<Vec<Expr>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [text, option_args @ ..] = args else {
        return Ok(session.invalid_call_with_arg_range("UrlDecode", args, 1, usize::MAX));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::UrlDecode,
                head,
                args,
                option_args,
            );
        }
    };
    let Some(options) = parse_url_form_options(option_exprs)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    match url_decode(text, options.form_encoding) {
        Ok(decoded) => Ok(Expr::string(decoded)),
        Err(message) => Ok(failed_with_message(session, "UrlDecode", "escape", message)),
    }
}

pub(crate) fn eval_url_parse(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    eval_url_parse_with_options(session, head, args, None)
}

pub(crate) fn eval_url_parse_with_options(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    resolved_options: Option<Vec<Expr>>,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [text, option_args @ ..] = args else {
        return Ok(session.invalid_call_with_arg_range("UrlParse", args, 1, usize::MAX));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = match resolved_options {
        Some(options) => options,
        None => {
            return session.merged_option_exprs_for_system_symbol(
                OptionConsumer::UrlParse,
                head,
                args,
                option_args,
            );
        }
    };
    let Some(options) = parse_url_parse_options(session, text, option_exprs)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    match parse_url(text, &options) {
        Ok(Some(parsed)) => Ok(parsed.into_expr()),
        Ok(None) => Ok(failed_with_message(
            session,
            "UrlParse",
            "syntax",
            "UrlParse expects scheme://host[:port][/path][?query][#fragment].",
        )),
        Err(message) => Ok(failed_with_message(session, "UrlParse", "escape", message)),
    }
}

fn enforce_url_parse_memory_constraint(
    session: &Session,
    text: &str,
    default_scheme: Option<&str>,
) -> Result<(), EvalError> {
    if session.active_memory_limit().is_none() {
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    }

    let has_explicit_scheme = text.contains("://");
    let default_scheme_bytes = default_scheme.map_or(0, str::len);
    let effective_bytes = if has_explicit_scheme {
        text.len()
    } else {
        text.len()
            .saturating_add(default_scheme_bytes)
            .saturating_add(3)
    };
    let copied_payload_bytes = if has_explicit_scheme {
        // A validated DefaultScheme is cloned into the options even when the
        // explicit URL means parsing will not consume it.
        effective_bytes.saturating_add(default_scheme_bytes)
    } else {
        // The stored option, synthesized source, and parsed component strings
        // coexist until parsing returns.
        effective_bytes
            .saturating_mul(2)
            .saturating_add(default_scheme_bytes)
    };
    // Six fixed association fields, including the canonical pair-vector
    // capacity and all fixed head/key/Null symbol text, fit in 41 Expr slots.
    let fixed_result_bytes = 41usize.saturating_mul(std::mem::size_of::<Expr>());
    session.enforce_active_memory_limit_for_projected_bytes(
        copied_payload_bytes.saturating_add(fixed_result_bytes),
    )
}

struct UrlFormOptions {
    form_encoding: bool,
}

struct UrlParseOptions {
    default_scheme: Option<String>,
    decode_query: bool,
}

fn parse_url_form_options(option_exprs: Vec<Expr>) -> Result<Option<UrlFormOptions>, EvalError> {
    let mut options = UrlFormOptions {
        form_encoding: false,
    };

    for expr in option_exprs {
        let Some((name, value)) = option_rule(&expr) else {
            return Ok(None);
        };
        match name {
            "FormEncoding" => {
                let Some(value) = bool_option_value(value) else {
                    return Ok(None);
                };
                options.form_encoding = value;
            }
            _ => return Ok(None),
        }
    }

    Ok(Some(options))
}

fn parse_url_parse_options(
    session: &mut Session,
    text: &str,
    option_exprs: Vec<Expr>,
) -> Result<Option<UrlParseOptions>, EvalError> {
    let mut default_scheme = None;
    let mut decode_query = false;

    for expr in &option_exprs {
        let Some((name, value)) = option_rule(expr) else {
            return Ok(None);
        };
        match name {
            "DefaultScheme" => {
                if is_system_symbol_name(value, "None") {
                    default_scheme = None;
                } else {
                    let Some(value) = expr_as_string(value) else {
                        return Ok(None);
                    };
                    if !valid_scheme(value) {
                        return Ok(None);
                    }
                    default_scheme = Some(value);
                }
            }
            "DecodeQuery" => {
                let Some(value) = bool_option_value(value) else {
                    return Ok(None);
                };
                decode_query = value;
            }
            _ => return Ok(None),
        }
    }

    enforce_url_parse_memory_constraint(session, text, default_scheme)?;
    Ok(Some(UrlParseOptions {
        default_scheme: default_scheme.map(str::to_owned),
        decode_query,
    }))
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedUrl {
    scheme: String,
    host: String,
    port: Option<BigInt>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl ParsedUrl {
    fn into_expr(self) -> Expr {
        render_association(vec![
            (Expr::string("Scheme"), Expr::string(self.scheme)),
            (Expr::string("Host"), Expr::string(self.host)),
            (
                Expr::string("Port"),
                self.port
                    .map(Expr::integer)
                    .unwrap_or_else(|| system_expr("Null")),
            ),
            (Expr::string("Path"), Expr::string(self.path)),
            (
                Expr::string("Query"),
                self.query
                    .map(Expr::string)
                    .unwrap_or_else(|| system_expr("Null")),
            ),
            (
                Expr::string("Fragment"),
                self.fragment
                    .map(Expr::string)
                    .unwrap_or_else(|| system_expr("Null")),
            ),
        ])
    }
}

fn url_encode(text: &str, form_encoding: bool) -> String {
    let mut output = String::new();
    for byte in text.as_bytes() {
        if form_encoding && *byte == b' ' {
            output.push('+');
        } else if is_unreserved(*byte) {
            output.push(*byte as char);
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}

fn url_decode(text: &str, form_encoding: bool) -> Result<String, String> {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            decoded.push(if form_encoding && bytes[index] == b'+' {
                b' '
            } else {
                bytes[index]
            });
            index += 1;
            continue;
        }
        let Some(hex) = bytes.get(index + 1..index + 3) else {
            return Err("UrlDecode found an incomplete percent escape.".to_string());
        };
        let value = decode_hex_byte(hex)
            .ok_or_else(|| "UrlDecode found an invalid percent escape.".to_string())?;
        decoded.push(value);
        index += 3;
    }
    String::from_utf8(decoded)
        .map_err(|_| "UrlDecode produced bytes that are not valid UTF-8.".to_string())
}

fn parse_url(text: &str, options: &UrlParseOptions) -> Result<Option<ParsedUrl>, String> {
    let source;
    let text = if text.contains("://") {
        text
    } else if let Some(default_scheme) = &options.default_scheme {
        source = format!("{default_scheme}://{text}");
        &source
    } else {
        return Ok(None);
    };

    let Some((scheme, rest)) = text.split_once("://") else {
        return Ok(None);
    };
    if !valid_scheme(scheme) {
        return Ok(None);
    }

    let host_end = rest
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, ':' | '/' | '?' | '#').then_some(index))
        .unwrap_or(rest.len());
    let host = &rest[..host_end];
    if host.is_empty() {
        return Ok(None);
    }

    let mut tail = &rest[host_end..];
    let port = if let Some(after_colon) = tail.strip_prefix(':') {
        let port_end = after_colon
            .char_indices()
            .find_map(|(index, ch)| matches!(ch, '/' | '?' | '#').then_some(index))
            .unwrap_or(after_colon.len());
        let port_text = &after_colon[..port_end];
        if port_text.is_empty() || !port_text.chars().all(|ch| ch.is_ascii_digit()) {
            return Ok(None);
        }
        tail = &after_colon[port_end..];
        let Some(port) = BigInt::from_str(port_text).ok() else {
            return Ok(None);
        };
        Some(port)
    } else {
        None
    };

    let path = if let Some(after_slash) = tail.strip_prefix('/') {
        let path_end = after_slash
            .char_indices()
            .find_map(|(index, ch)| matches!(ch, '?' | '#').then_some(index))
            .unwrap_or(after_slash.len());
        let path = format!("/{}", &after_slash[..path_end]);
        tail = &after_slash[path_end..];
        path
    } else {
        String::new()
    };

    let query = if let Some(after_question) = tail.strip_prefix('?') {
        let query_end = after_question
            .char_indices()
            .find_map(|(index, ch)| (ch == '#').then_some(index))
            .unwrap_or(after_question.len());
        let query = if options.decode_query {
            url_decode(&after_question[..query_end], true)?
        } else {
            after_question[..query_end].to_string()
        };
        tail = &after_question[query_end..];
        Some(query)
    } else {
        None
    };

    let fragment = if let Some(after_hash) = tail.strip_prefix('#') {
        tail = "";
        Some(after_hash.to_string())
    } else {
        None
    };

    Ok(tail.is_empty().then_some(ParsedUrl {
        scheme: scheme.to_string(),
        host: host.to_string(),
        port,
        path,
        query,
        fragment,
    }))
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~')
}

fn decode_hex_byte(hex: &[u8]) -> Option<u8> {
    let high = hex_value(*hex.first()?)?;
    let low = hex_value(*hex.get(1)?)?;
    Some(high * 16 + low)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn valid_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
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
