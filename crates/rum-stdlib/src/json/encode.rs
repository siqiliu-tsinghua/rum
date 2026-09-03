use std::fmt::Write;

use super::MAX_JSON_DEPTH;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JsonEncodeError {
    kind: JsonEncodeErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum JsonEncodeErrorKind {
    Message(String),
    Depth,
}

impl JsonEncodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            kind: JsonEncodeErrorKind::Message(message.into()),
        }
    }

    fn depth() -> Self {
        Self {
            kind: JsonEncodeErrorKind::Depth,
        }
    }

    pub(super) fn message_tag(&self) -> &'static str {
        match self.kind {
            JsonEncodeErrorKind::Message(_) => "json",
            JsonEncodeErrorKind::Depth => "depth",
        }
    }

    pub(super) fn message(&self) -> String {
        match &self.kind {
            JsonEncodeErrorKind::Message(message) => message.clone(),
            JsonEncodeErrorKind::Depth => {
                format!("JSON nesting exceeds the fixed depth limit of {MAX_JSON_DEPTH}.")
            }
        }
    }
}

type EncodeResult<T> = Result<T, JsonEncodeError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EncodeConfig {
    indent: Option<usize>,
}

impl EncodeConfig {
    fn compact() -> Self {
        Self { indent: None }
    }

    fn pretty(indent: usize) -> Self {
        Self {
            indent: Some(indent),
        }
    }

    fn is_pretty(self) -> bool {
        self.indent.is_some()
    }

    fn push_indent(self, out: &mut String, level: usize) {
        if let Some(indent) = self.indent {
            out.extend(std::iter::repeat_n(' ', indent.saturating_mul(level)));
        }
    }
}

pub(super) fn to_json(expr: &Expr, indent: Option<usize>) -> EncodeResult<String> {
    let config = indent.map_or_else(EncodeConfig::compact, EncodeConfig::pretty);
    let mut out = String::new();
    write_value(expr, config, 0, &mut out)?;
    Ok(out)
}

fn write_value(
    expr: &Expr,
    config: EncodeConfig,
    level: usize,
    out: &mut String,
) -> EncodeResult<()> {
    if let Some(entries) = association_entries_from_expr(expr) {
        if level >= MAX_JSON_DEPTH {
            return Err(JsonEncodeError::depth());
        }
        return write_object(expr, &entries, config, level, out);
    }

    match expr {
        Expr::Atom(Atom::Integer(value)) => out.push_str(&value.to_string()),
        Expr::Atom(Atom::Rational(value)) => out.push_str(&terminating_decimal(value)?),
        Expr::Atom(Atom::String(value)) => write_json_string(value, out),
        Expr::Atom(Atom::Symbol(_)) if is_system_symbol_name(expr, "True") => {
            out.push_str("true");
        }
        Expr::Atom(Atom::Symbol(_)) if is_system_symbol_name(expr, "False") => {
            out.push_str("false");
        }
        Expr::Atom(Atom::Symbol(_)) if is_system_symbol_name(expr, "Null") => {
            out.push_str("null");
        }
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
            if level >= MAX_JSON_DEPTH {
                return Err(JsonEncodeError::depth());
            }
            write_array(args, config, level, out)?;
        }
        _ => {
            return Err(JsonEncodeError::new(format!(
                "Cannot encode expression {expr} as JSON."
            )));
        }
    }

    Ok(())
}

fn write_array(
    items: &[Expr],
    config: EncodeConfig,
    level: usize,
    out: &mut String,
) -> EncodeResult<()> {
    if items.is_empty() {
        out.push_str("[]");
        return Ok(());
    }

    out.push('[');
    if config.is_pretty() {
        out.push('\n');
    }

    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
            if config.is_pretty() {
                out.push('\n');
            }
        }
        if config.is_pretty() {
            config.push_indent(out, level + 1);
        }
        write_value(item, config, level + 1, out)?;
    }

    if config.is_pretty() {
        out.push('\n');
        config.push_indent(out, level);
    }
    out.push(']');
    Ok(())
}

fn write_object(
    source: &Expr,
    entries: &[(Expr, Expr)],
    config: EncodeConfig,
    level: usize,
    out: &mut String,
) -> EncodeResult<()> {
    if entries.is_empty() {
        out.push_str("{}");
        return Ok(());
    }

    out.push('{');
    if config.is_pretty() {
        out.push('\n');
    }

    for (index, (key, value)) in entries.iter().enumerate() {
        let Expr::Atom(Atom::String(key)) = key else {
            return Err(JsonEncodeError::new(format!(
                "Cannot encode Association key {key} as a JSON object key in {source}."
            )));
        };

        if index > 0 {
            out.push(',');
            if config.is_pretty() {
                out.push('\n');
            }
        }
        if config.is_pretty() {
            config.push_indent(out, level + 1);
        }
        write_json_string(key, out);
        out.push(':');
        if config.is_pretty() {
            out.push(' ');
        }
        write_value(value, config, level + 1, out)?;
    }

    if config.is_pretty() {
        out.push('\n');
        config.push_indent(out, level);
    }
    out.push('}');
    Ok(())
}

fn write_json_string(value: &str, out: &mut String) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if other <= '\u{001f}' => push_hex_escape(out, other as u32),
            other if other as u32 > 0xffff => push_surrogate_pair(out, other as u32),
            other => out.push(other),
        }
    }
    out.push('"');
}

fn terminating_decimal(value: &Rational) -> EncodeResult<String> {
    let mut denominator = value.denominator().clone();
    let mut twos = 0_usize;
    let mut fives = 0_usize;
    let two = BigInt::from(2_u8);
    let five = BigInt::from(5_u8);
    let zero = BigInt::from(0_u8);
    let one = BigInt::from(1_u8);

    while &denominator % &two == zero {
        denominator /= &two;
        twos += 1;
    }
    while &denominator % &five == zero {
        denominator /= &five;
        fives += 1;
    }

    if denominator != one {
        return Err(JsonEncodeError::new(format!(
            "Cannot encode non-terminating Rational {value} as JSON; convert it explicitly before calling ToJson."
        )));
    }

    let scale = twos.max(fives);
    let mut scaled = value.numerator().clone();
    scaled *= pow_small(2, scale - twos);
    scaled *= pow_small(5, scale - fives);
    Ok(format_scaled_decimal(scaled, scale))
}

fn format_scaled_decimal(value: BigInt, scale: usize) -> String {
    if scale == 0 {
        return value.to_string();
    }

    let mut digits = value.to_string();
    let negative = digits.starts_with('-');
    if negative {
        digits.remove(0);
    }

    let mut rendered = String::new();
    if negative {
        rendered.push('-');
    }

    if digits.len() <= scale {
        rendered.push_str("0.");
        rendered.extend(std::iter::repeat_n('0', scale - digits.len()));
        rendered.push_str(&digits);
    } else {
        let split = digits.len() - scale;
        rendered.push_str(&digits[..split]);
        rendered.push('.');
        rendered.push_str(&digits[split..]);
    }
    rendered
}

fn pow_small(base: u8, exponent: usize) -> BigInt {
    let mut value = BigInt::from(1_u8);
    for _ in 0..exponent {
        value *= base;
    }
    value
}

fn push_hex_escape(out: &mut String, value: u32) {
    write!(out, "\\u{value:04X}").expect("writing to String should not fail");
}

fn push_surrogate_pair(out: &mut String, value: u32) {
    let scalar = value - 0x10000;
    let high = 0xd800 + (scalar >> 10);
    let low = 0xdc00 + (scalar & 0x3ff);
    write!(out, "\\u{high:04X}\\u{low:04X}").expect("writing to String should not fail");
}

#[cfg(test)]
mod depth_tests {
    use super::*;

    fn nested_lists(levels: usize) -> Expr {
        let mut expr = Expr::integer(BigInt::from(0));
        for _ in 0..levels {
            expr = Expr::call(system_expr("List"), vec![expr]);
        }
        expr
    }

    #[test]
    fn to_json_accepts_nesting_at_the_depth_limit() {
        assert!(to_json(&nested_lists(MAX_JSON_DEPTH), None).is_ok());
    }

    #[test]
    fn to_json_refuses_nesting_one_past_the_depth_limit() {
        let error = to_json(&nested_lists(MAX_JSON_DEPTH + 1), None).unwrap_err();
        assert_eq!(error, JsonEncodeError::depth());
        assert_eq!(error.message_tag(), "depth");
        assert_eq!(
            error.message(),
            format!("JSON nesting exceeds the fixed depth limit of {MAX_JSON_DEPTH}.")
        );
    }
}
