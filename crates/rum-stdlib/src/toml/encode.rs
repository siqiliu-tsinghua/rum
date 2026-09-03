use std::fmt::Write;

use rum_core::{Atom, BigInt, Expr, IntValue, Rational};
use rum_runtime::kernel_api::{association_entries_from_expr, is_system_symbol_name, list_items};

use super::MAX_TOML_DEPTH;
use super::TomlEncodeError;
use super::model::bare_key_byte;

pub(super) fn to_toml(expr: &Expr) -> Result<String, TomlEncodeError> {
    let Some(entries) = association_entries_from_expr(expr) else {
        return Err(TomlEncodeError::new(
            "ToToml expects a top-level Association with String keys.",
        ));
    };
    if entries.is_empty() {
        return Ok(String::new());
    }

    let mut out = String::new();
    write_table_body(&entries, &[], &mut out)?;
    Ok(out)
}

fn write_table_body(
    entries: &[(Expr, Expr)],
    path: &[String],
    out: &mut String,
) -> Result<(), TomlEncodeError> {
    if path.len() > MAX_TOML_DEPTH {
        return Err(TomlEncodeError::depth());
    }
    if !path.is_empty() {
        push_section_spacing(out);
        write_table_header(path, out);
        out.push('\n');
    }

    let last_plain_index = last_plain_assignment_index(entries);
    let mut wrote_assignment = false;
    for (index, (key, value)) in entries.iter().enumerate() {
        if table_like(value) && !has_plain_assignment_after(last_plain_index, index) {
            continue;
        }
        let key = string_key(key, "ToToml keys must be Strings.")?;
        if wrote_assignment {
            out.push('\n');
        }
        write_key(key, out);
        out.push_str(" = ");
        write_inline_value(value, path.len(), out)?;
        wrote_assignment = true;
    }

    for (index, (key, value)) in entries.iter().enumerate() {
        if !table_like(value) || has_plain_assignment_after(last_plain_index, index) {
            continue;
        }
        let key = string_key(key, "ToToml keys must be Strings.")?;
        let mut child_path = path.to_vec();
        child_path.push(key.to_string());

        if let Some(child_entries) = association_entries_from_expr(value) {
            write_table_body(&child_entries, &child_path, out)?;
        } else if let Some(items) = list_of_associations(value) {
            write_array_of_tables(&items, &child_path, out)?;
        }
    }

    Ok(())
}

fn write_array_of_tables(
    items: &[Vec<(Expr, Expr)>],
    path: &[String],
    out: &mut String,
) -> Result<(), TomlEncodeError> {
    if path.len() > MAX_TOML_DEPTH {
        return Err(TomlEncodeError::depth());
    }
    for entries in items {
        push_section_spacing(out);
        out.push_str("[[");
        write_key_path(path, out);
        out.push_str("]]\n");
        write_table_assignments_then_children(entries, path, out)?;
    }
    Ok(())
}

fn write_table_assignments_then_children(
    entries: &[(Expr, Expr)],
    path: &[String],
    out: &mut String,
) -> Result<(), TomlEncodeError> {
    let last_plain_index = last_plain_assignment_index(entries);
    let mut wrote_assignment = false;
    for (index, (key, value)) in entries.iter().enumerate() {
        if table_like(value) && !has_plain_assignment_after(last_plain_index, index) {
            continue;
        }
        if wrote_assignment {
            out.push('\n');
        }
        write_key(string_key(key, "ToToml keys must be Strings.")?, out);
        out.push_str(" = ");
        write_inline_value(value, path.len(), out)?;
        wrote_assignment = true;
    }

    for (index, (key, value)) in entries.iter().enumerate() {
        if !table_like(value) || has_plain_assignment_after(last_plain_index, index) {
            continue;
        }
        let key = string_key(key, "ToToml keys must be Strings.")?;
        let mut child_path = path.to_vec();
        child_path.push(key.to_string());
        if let Some(child_entries) = association_entries_from_expr(value) {
            write_table_body(&child_entries, &child_path, out)?;
        } else if let Some(items) = list_of_associations(value) {
            write_array_of_tables(&items, &child_path, out)?;
        }
    }
    Ok(())
}

fn write_inline_value(expr: &Expr, depth: usize, out: &mut String) -> Result<(), TomlEncodeError> {
    if let Some(entries) = association_entries_from_expr(expr) {
        if depth + 1 > MAX_TOML_DEPTH {
            return Err(TomlEncodeError::depth());
        }
        return write_inline_table(&entries, depth + 1, out);
    }

    match expr {
        Expr::Atom(Atom::Integer(value)) => out.push_str(&int_value_to_bigint(value).to_string()),
        Expr::Atom(Atom::Rational(value)) => out.push_str(&terminating_decimal(value)?),
        Expr::Atom(Atom::String(value)) => write_basic_string(value, out),
        Expr::Atom(Atom::Symbol(_)) if is_system_symbol_name(expr, "True") => {
            out.push_str("true");
        }
        Expr::Atom(Atom::Symbol(_)) if is_system_symbol_name(expr, "False") => {
            out.push_str("false");
        }
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
            if depth + 1 > MAX_TOML_DEPTH {
                return Err(TomlEncodeError::depth());
            }
            write_inline_array(args, depth + 1, out)?;
        }
        _ => {
            return Err(TomlEncodeError::new(format!(
                "Cannot encode expression {expr} as TOML."
            )));
        }
    }
    Ok(())
}

fn write_inline_array(
    items: &[Expr],
    depth: usize,
    out: &mut String,
) -> Result<(), TomlEncodeError> {
    out.push('[');
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        write_inline_value(item, depth, out)?;
    }
    out.push(']');
    Ok(())
}

fn write_inline_table(
    entries: &[(Expr, Expr)],
    depth: usize,
    out: &mut String,
) -> Result<(), TomlEncodeError> {
    out.push('{');
    for (index, (key, value)) in entries.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        write_key(
            string_key(key, "ToToml inline table keys must be Strings.")?,
            out,
        );
        out.push_str(" = ");
        write_inline_value(value, depth, out)?;
    }
    out.push('}');
    Ok(())
}

fn list_of_associations(expr: &Expr) -> Option<Vec<Vec<(Expr, Expr)>>> {
    let items = list_items(expr)?;
    if items.is_empty() {
        return None;
    }
    items.iter().map(association_entries_from_expr).collect()
}

fn table_like(expr: &Expr) -> bool {
    association_shaped(expr) || list_of_associations_shaped(expr)
}

// Shape-only mirrors of `association_entries_from_expr` / `list_of_associations`
// success: they answer the same classification question by walking the
// expression, without building and canonicalizing an entry table per probe.
fn association_shaped(expr: &Expr) -> bool {
    let Expr::Call { head, args, .. } = expr else {
        return false;
    };
    is_system_symbol_name(head, "Association") && args.iter().all(deep_entry_shaped)
}

fn deep_entry_shaped(expr: &Expr) -> bool {
    let Expr::Call { head, args, .. } = expr else {
        return false;
    };
    if is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed") {
        return args.len() == 2;
    }
    if is_system_symbol_name(head, "Association") {
        return args.iter().all(deep_entry_shaped);
    }
    is_system_symbol_name(head, "List") && args.iter().all(deep_entry_shaped)
}

fn list_of_associations_shaped(expr: &Expr) -> bool {
    let Some(items) = list_items(expr) else {
        return false;
    };
    !items.is_empty() && items.iter().all(association_shaped)
}

fn has_plain_assignment_after(last_plain_index: Option<usize>, index: usize) -> bool {
    last_plain_index.is_some_and(|last| last > index)
}

fn last_plain_assignment_index(entries: &[(Expr, Expr)]) -> Option<usize> {
    entries.iter().rposition(|(_, value)| !table_like(value))
}

fn string_key<'a>(expr: &'a Expr, message: &str) -> Result<&'a str, TomlEncodeError> {
    let Expr::Atom(Atom::String(value)) = expr else {
        return Err(TomlEncodeError::new(message));
    };
    Ok(value)
}

fn write_table_header(path: &[String], out: &mut String) {
    out.push('[');
    write_key_path(path, out);
    out.push(']');
}

fn write_key_path(path: &[String], out: &mut String) {
    for (index, segment) in path.iter().enumerate() {
        if index > 0 {
            out.push('.');
        }
        write_key(segment, out);
    }
}

fn write_key(key: &str, out: &mut String) {
    if bare_key_safe(key) {
        out.push_str(key);
    } else {
        write_basic_string(key, out);
    }
}

fn write_basic_string(value: &str, out: &mut String) {
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

fn terminating_decimal(value: &Rational) -> Result<String, TomlEncodeError> {
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
        return Err(TomlEncodeError::new(format!(
            "Cannot encode non-terminating Rational {value} as TOML; convert it explicitly before calling ToToml."
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

fn push_section_spacing(out: &mut String) {
    if out.is_empty() {
        return;
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
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

fn bare_key_safe(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(bare_key_byte)
}

#[cfg(test)]
mod depth_tests {
    use super::*;
    use rum_runtime::kernel_api::{render_association, system_expr};

    fn nested_inline_arrays(levels: usize) -> Expr {
        let mut expr = Expr::integer(BigInt::from(0));
        for _ in 0..levels {
            expr = Expr::call(system_expr("List"), vec![expr]);
        }
        render_association(vec![(Expr::string("a"), expr)])
    }

    // `tables` nested sub-tables under the top-level document Association; the
    // document itself is not a collection, mirroring FromToml's decode count.
    fn nested_tables(tables: usize) -> Expr {
        let mut expr = Expr::integer(BigInt::from(0));
        for _ in 0..tables {
            expr = render_association(vec![(Expr::string("k"), expr)]);
        }
        render_association(vec![(Expr::string("k"), expr)])
    }

    #[test]
    fn to_toml_accepts_nesting_at_the_depth_limit() {
        assert!(to_toml(&nested_inline_arrays(MAX_TOML_DEPTH)).is_ok());
        assert!(to_toml(&nested_tables(MAX_TOML_DEPTH)).is_ok());
    }

    #[test]
    fn to_toml_refuses_nesting_one_past_the_depth_limit() {
        let error = to_toml(&nested_inline_arrays(MAX_TOML_DEPTH + 1)).unwrap_err();
        assert_eq!(error, TomlEncodeError::depth());
        assert_eq!(error.message_tag(), "depth");
        assert_eq!(
            error.message(),
            format!("TOML nesting exceeds the fixed depth limit of {MAX_TOML_DEPTH}.")
        );

        let error = to_toml(&nested_tables(MAX_TOML_DEPTH + 1)).unwrap_err();
        assert_eq!(error, TomlEncodeError::depth());
    }
}
