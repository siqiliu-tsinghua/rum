use std::fmt::Write;

use rum_core::{Atom, BigInt, Expr, IntValue, Rational};
use rum_runtime::kernel_api::{association_entries_from_expr, is_system_symbol_name, list_items};

use super::number::parses_as_yaml_number;
use super::{MAX_YAML_DEPTH, YamlEncodeError, string_atom};

pub(super) fn to_yaml(expr: &Expr) -> Result<String, YamlEncodeError> {
    let mut out = String::new();
    write_value(expr, 0, &mut out)?;
    Ok(out)
}

fn write_value(expr: &Expr, level: usize, out: &mut String) -> Result<(), YamlEncodeError> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return write_mapping(&entries, level, out);
    }
    if let Some(items) = list_items(expr) {
        return write_sequence(items, level, out);
    }
    write_inline_value(expr, out)
}

fn write_mapping(
    entries: &[(Expr, Expr)],
    level: usize,
    out: &mut String,
) -> Result<(), YamlEncodeError> {
    if level >= MAX_YAML_DEPTH {
        return Err(YamlEncodeError::depth());
    }
    if entries.is_empty() {
        out.push_str("{}");
        return Ok(());
    }

    for (index, (key, value)) in entries.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        push_indent(out, level);
        let key = string_atom(key)
            .ok_or_else(|| YamlEncodeError::new("ToYaml mapping keys must be Strings."))?;
        write_key(key, out);
        out.push(':');
        write_child_value(value, level, out)?;
    }
    Ok(())
}

fn write_sequence(items: &[Expr], level: usize, out: &mut String) -> Result<(), YamlEncodeError> {
    if level >= MAX_YAML_DEPTH {
        return Err(YamlEncodeError::depth());
    }
    if items.is_empty() {
        out.push_str("[]");
        return Ok(());
    }

    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        push_indent(out, level);
        out.push('-');
        write_child_value(item, level, out)?;
    }
    Ok(())
}

fn write_child_value(value: &Expr, level: usize, out: &mut String) -> Result<(), YamlEncodeError> {
    if let Some(entries) = association_entries_from_expr(value) {
        if !entries.is_empty() {
            out.push('\n');
            return write_mapping(&entries, level + 1, out);
        }
    }
    if let Some(items) = list_items(value) {
        if !items.is_empty() {
            out.push('\n');
            return write_sequence(items, level + 1, out);
        }
    }
    if let Some(value) = string_atom(value) {
        if value.contains('\n') {
            out.push(' ');
            return write_block_string(value, level + 1, out);
        }
    }

    out.push(' ');
    write_inline_value(value, out)
}

fn write_inline_value(expr: &Expr, out: &mut String) -> Result<(), YamlEncodeError> {
    if let Some(entries) = association_entries_from_expr(expr) {
        if entries.is_empty() {
            out.push_str("{}");
            return Ok(());
        }
    }
    if let Some(items) = list_items(expr) {
        if items.is_empty() {
            out.push_str("[]");
            return Ok(());
        }
    }

    match expr {
        Expr::Atom(Atom::Integer(value)) => out.push_str(&int_value_to_bigint(value).to_string()),
        Expr::Atom(Atom::Rational(value)) => out.push_str(&terminating_decimal(value)?),
        Expr::Atom(Atom::String(value)) => write_string(value, out)?,
        Expr::Atom(Atom::Symbol(_)) if is_system_symbol_name(expr, "True") => out.push_str("true"),
        Expr::Atom(Atom::Symbol(_)) if is_system_symbol_name(expr, "False") => {
            out.push_str("false");
        }
        Expr::Atom(Atom::Symbol(_)) if is_system_symbol_name(expr, "Null") => out.push_str("null"),
        _ => {
            return Err(YamlEncodeError::new(format!(
                "Cannot encode expression {expr} as YAML."
            )));
        }
    }
    Ok(())
}

fn write_key(value: &str, out: &mut String) {
    if safe_plain(value) {
        out.push_str(value);
    } else {
        write_double_quoted(value, out);
    }
}

fn write_string(value: &str, out: &mut String) -> Result<(), YamlEncodeError> {
    if value.contains('\n') {
        return write_block_string(value, 0, out);
    }
    if safe_plain(value) {
        out.push_str(value);
    } else {
        write_double_quoted(value, out);
    }
    Ok(())
}

fn write_block_string(value: &str, level: usize, out: &mut String) -> Result<(), YamlEncodeError> {
    if value == "\n" || value.ends_with("\n\n") {
        return Err(YamlEncodeError::new(
            "ToYaml cannot encode multiline Strings whose block-scalar value needs YAML's unsupported |+ chomping mode.",
        ));
    }

    let trailing_newline = value.ends_with('\n');
    out.push_str(if trailing_newline { "|" } else { "|-" });

    let body = if trailing_newline {
        &value[..value.len() - 1]
    } else {
        value
    };

    for line in body.split('\n') {
        out.push('\n');
        push_indent(out, level);
        out.push_str(line);
    }
    Ok(())
}

fn write_double_quoted(value: &str, out: &mut String) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{000c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            other if other <= '\u{001f}' || other == '\u{007f}' => {
                write!(out, "\\u{:04X}", other as u32).expect("writing to String should not fail");
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

fn safe_plain(value: &str) -> bool {
    if value.is_empty()
        || value.starts_with(' ')
        || value.ends_with(' ')
        || !value.is_ascii()
        || !value.as_bytes()[0].is_ascii_alphabetic()
    {
        return false;
    }
    if !value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'_' | b'.' | b'/' | b'-')
    }) {
        return false;
    }

    let lower = value.to_ascii_lowercase();
    !matches!(
        lower.as_str(),
        "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "~"
    ) && !parses_as_yaml_number(value)
}

fn terminating_decimal(value: &Rational) -> Result<String, YamlEncodeError> {
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
        return Err(YamlEncodeError::new(format!(
            "Cannot encode non-terminating Rational {value} as YAML; convert it explicitly before calling ToYaml."
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

fn push_indent(out: &mut String, level: usize) {
    out.extend(std::iter::repeat_n(' ', level.saturating_mul(2)));
}

fn pow_small(base: u8, exponent: usize) -> BigInt {
    let mut value = BigInt::from(1_u8);
    for _ in 0..exponent {
        value *= base;
    }
    value
}

fn int_value_to_bigint(value: &IntValue) -> BigInt {
    value.to_bigint()
}

#[cfg(test)]
mod depth_tests {
    use super::super::association_expr;
    use super::*;
    use rum_runtime::kernel_api::system_expr;

    fn nested_sequences(levels: usize) -> Expr {
        let mut expr = Expr::integer(BigInt::from(0));
        for _ in 0..levels {
            expr = Expr::call(system_expr("List"), vec![expr]);
        }
        expr
    }

    fn nested_mappings(levels: usize) -> Expr {
        let mut expr = Expr::integer(BigInt::from(0));
        for _ in 0..levels {
            expr = association_expr(vec![("k".to_string(), expr)]);
        }
        expr
    }

    #[test]
    fn to_yaml_accepts_nesting_at_the_depth_limit() {
        assert!(to_yaml(&nested_sequences(MAX_YAML_DEPTH)).is_ok());
        assert!(to_yaml(&nested_mappings(MAX_YAML_DEPTH)).is_ok());
    }

    #[test]
    fn to_yaml_refuses_nesting_one_past_the_depth_limit() {
        let error = to_yaml(&nested_sequences(MAX_YAML_DEPTH + 1)).unwrap_err();
        assert_eq!(error, YamlEncodeError::depth());
        assert_eq!(error.message_tag(), "depth");
        assert_eq!(
            error.message(),
            format!("YAML nesting exceeds the fixed depth limit of {MAX_YAML_DEPTH}.")
        );

        let error = to_yaml(&nested_mappings(MAX_YAML_DEPTH + 1)).unwrap_err();
        assert_eq!(error, YamlEncodeError::depth());
    }
}
