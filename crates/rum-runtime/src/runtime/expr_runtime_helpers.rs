use super::*;

pub(super) fn is_slot_sequence_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => {
            is_system_symbol_name(head, heads::functions::SLOT_SEQUENCE) && args.len() == 1
        }
        _ => false,
    }
}

pub(super) fn sequence_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::functions::SEQUENCE) => {
            Some(args)
        }
        _ => None,
    }
}

pub fn call_items_with_head<'a>(expr: &'a Expr, expected_head: &Expr) -> Option<&'a [Expr]> {
    match expr {
        Expr::Call { head, args } if head.as_ref() == expected_head => Some(&args[..]),
        _ => None,
    }
}

/// Encodes bytes using the kernel's current base64 transport alphabet.
pub fn base64_encode(bytes: &[u8]) -> String {
    const BASE64_ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut rendered = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut index = 0;
    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | (bytes[index + 2] as u32);
        rendered.push(BASE64_ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        rendered.push(BASE64_ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        rendered.push(BASE64_ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
        rendered.push(BASE64_ALPHABET[(chunk & 0x3f) as usize] as char);
        index += 3;
    }

    match bytes.len() - index {
        1 => {
            let chunk = (bytes[index] as u32) << 16;
            rendered.push(BASE64_ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            rendered.push(BASE64_ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
            rendered.push('=');
            rendered.push('=');
        }
        2 => {
            let chunk = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
            rendered.push(BASE64_ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            rendered.push(BASE64_ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
            rendered.push(BASE64_ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
            rendered.push('=');
        }
        _ => {}
    }

    rendered
}

impl Session {
    /// Restores a transport payload according to the stored original form marker.
    pub fn restore_transport_expr(&self, payload: &[u8], original_form: Expr) -> Option<Expr> {
        if is_system_symbol_name(&original_form, "String") {
            return Some(Expr::string(std::str::from_utf8(payload).ok()?.to_string()));
        }
        if is_system_symbol_name(&original_form, "ByteArray") {
            return Some(Expr::call(
                system_expr("ByteArray"),
                vec![Expr::string(base64_encode(payload))],
            ));
        }
        if is_system_symbol_name(&original_form, "InputForm") {
            let source = std::str::from_utf8(payload).ok()?;
            let syntax = parse_expr(source).ok()?;
            return self.lower_syntax_expr(&syntax).ok();
        }

        None
    }
}

pub(super) fn format_list_lengths(lengths: &[usize]) -> String {
    lengths
        .iter()
        .map(|length| length.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn flatten_flat_call_args(head: &Expr, args: Vec<Expr>) -> Vec<Expr> {
    let mut flattened = Vec::new();

    for arg in args {
        match arg {
            Expr::Call {
                head: nested_head,
                args: nested_args,
            } if nested_head.as_ref() == head => flattened.extend(nested_args),
            other => flattened.push(other),
        }
    }

    flattened
}

pub fn compare_expr_canonical(lhs: &Expr, rhs: &Expr) -> std::cmp::Ordering {
    if let Some(ordering) = compare_negative_power_canonical(lhs, rhs) {
        return ordering;
    }

    match (lhs, rhs) {
        (Expr::Atom(lhs), Expr::Atom(rhs)) => compare_atom_canonical(lhs, rhs),
        (
            Expr::Call {
                head: lhs_head,
                args: lhs_args,
            },
            Expr::Call {
                head: rhs_head,
                args: rhs_args,
            },
        ) => compare_call_canonical(lhs_head, lhs_args, rhs_head, rhs_args),
        _ => expr_order_rank(lhs).cmp(&expr_order_rank(rhs)),
    }
}

fn compare_atom_canonical(lhs: &Atom, rhs: &Atom) -> std::cmp::Ordering {
    let rank_ordering = atom_order_rank(lhs).cmp(&atom_order_rank(rhs));
    if rank_ordering != std::cmp::Ordering::Equal {
        return rank_ordering;
    }

    match (lhs, rhs) {
        (Atom::Integer(lhs), Atom::Integer(rhs)) => lhs.cmp(rhs),
        (Atom::Integer(lhs), Atom::Rational(rhs)) => {
            compare_rational_values(&Rational::from_integer(lhs.to_bigint()), rhs)
        }
        (Atom::Rational(lhs), Atom::Integer(rhs)) => {
            compare_rational_values(lhs, &Rational::from_integer(rhs.to_bigint()))
        }
        (Atom::Rational(lhs), Atom::Rational(rhs)) => compare_rational_values(lhs, rhs),
        (Atom::String(lhs), Atom::String(rhs)) => lhs.cmp(rhs),
        (Atom::Symbol(lhs), Atom::Symbol(rhs)) => compare_symbol_canonical(lhs, rhs),
        _ => std::cmp::Ordering::Equal,
    }
}

fn compare_call_canonical(
    lhs_head: &Expr,
    lhs_args: &[Expr],
    rhs_head: &Expr,
    rhs_args: &[Expr],
) -> std::cmp::Ordering {
    let head_ordering = compare_call_head_canonical(lhs_head, rhs_head);
    if head_ordering != std::cmp::Ordering::Equal {
        return head_ordering;
    }

    for (lhs_arg, rhs_arg) in lhs_args.iter().zip(rhs_args) {
        let arg_ordering = compare_expr_canonical(lhs_arg, rhs_arg);
        if arg_ordering != std::cmp::Ordering::Equal {
            return arg_ordering;
        }
    }

    lhs_args.len().cmp(&rhs_args.len())
}

fn compare_negative_power_canonical(lhs: &Expr, rhs: &Expr) -> Option<std::cmp::Ordering> {
    match (
        negative_power_components(lhs),
        negative_power_components(rhs),
    ) {
        (Some((lhs_base, lhs_exponent)), Some((rhs_base, rhs_exponent))) => {
            let base_ordering = compare_expr_canonical(lhs_base, rhs_base);
            if base_ordering != std::cmp::Ordering::Equal {
                return Some(base_ordering);
            }
            Some(compare_rational_values(&lhs_exponent, &rhs_exponent))
        }
        (Some((lhs_base, _)), None) => {
            let rhs_base = power_base(rhs).unwrap_or(rhs);
            let base_ordering = compare_expr_canonical(lhs_base, rhs_base);
            if base_ordering != std::cmp::Ordering::Equal {
                return Some(base_ordering);
            }
            Some(std::cmp::Ordering::Less)
        }
        (None, Some((rhs_base, _))) => {
            let lhs_base = power_base(lhs).unwrap_or(lhs);
            let base_ordering = compare_expr_canonical(lhs_base, rhs_base);
            if base_ordering != std::cmp::Ordering::Equal {
                return Some(base_ordering);
            }
            Some(std::cmp::Ordering::Greater)
        }
        (None, None) => None,
    }
}

fn negative_power_components(expr: &Expr) -> Option<(&Expr, Rational)> {
    let [base, exponent] = power_args(expr)?;
    let exponent = exact_atom_numeric_value(exponent)?;
    (exponent.numerator() < &BigInt::from(0_u8)).then_some((base, exponent))
}

fn power_base(expr: &Expr) -> Option<&Expr> {
    let [base, _] = power_args(expr)?;
    Some(base)
}

fn power_args(expr: &Expr) -> Option<&[Expr; 2]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Power") && args.len() == 2 => {
            args.as_slice().try_into().ok()
        }
        _ => None,
    }
}

fn exact_atom_numeric_value(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.to_bigint())),
        Expr::Atom(Atom::Rational(value)) => Some(value.as_ref().clone()),
        _ => None,
    }
}

fn compare_call_head_canonical(lhs: &Expr, rhs: &Expr) -> std::cmp::Ordering {
    match (expr_symbol(lhs), expr_symbol(rhs)) {
        (Some(lhs), Some(rhs)) => compare_call_head_symbols(lhs, rhs),
        _ => compare_expr_canonical(lhs, rhs),
    }
}

fn compare_call_head_symbols(lhs: &Symbol, rhs: &Symbol) -> std::cmp::Ordering {
    let lhs_rank = system_call_head_rank(lhs);
    let rhs_rank = system_call_head_rank(rhs);
    match (lhs_rank, rhs_rank) {
        (Some(lhs_rank), Some(rhs_rank)) => lhs_rank.cmp(&rhs_rank),
        (Some(lhs_rank), None) => lhs_rank.cmp(&(2, 0)),
        (None, Some(rhs_rank)) => (2, 0).cmp(&rhs_rank),
        (None, None) => compare_symbol_canonical(lhs, rhs),
    }
}

fn system_call_head_rank(symbol: &Symbol) -> Option<(u8, u8)> {
    if symbol.context() != SYSTEM_CONTEXT {
        return None;
    }

    // rum's expression order gives common call heads precedence that is
    // distinct from standalone symbol order; keep this as an explicit subset.
    match symbol.name() {
        "Association" => Some((0, 0)),
        "Power" => Some((1, 0)),
        "Times" => Some((1, 1)),
        "Plus" => Some((1, 2)),
        _ => None,
    }
}

fn compare_symbol_canonical(lhs: &Symbol, rhs: &Symbol) -> std::cmp::Ordering {
    let name_ordering = compare_symbol_names(lhs.name(), rhs.name());
    if name_ordering != std::cmp::Ordering::Equal {
        return name_ordering;
    }

    let context_ordering = lhs.context().cmp(rhs.context());
    if context_ordering != std::cmp::Ordering::Equal {
        return context_ordering;
    }

    lhs.identity().cmp(&rhs.identity())
}

fn compare_symbol_names(lhs: &str, rhs: &str) -> std::cmp::Ordering {
    let mut lhs_chars = lhs.chars();
    let mut rhs_chars = rhs.chars();

    loop {
        match (lhs_chars.next(), rhs_chars.next()) {
            (Some(lhs), Some(rhs)) => {
                let ordering = symbol_name_char_key(lhs).cmp(&symbol_name_char_key(rhs));
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (None, None) => return std::cmp::Ordering::Equal,
        }
    }
}

fn symbol_name_char_key(ch: char) -> (u8, u32, u8) {
    if ch.is_ascii_alphabetic() {
        return (
            0,
            ch.to_ascii_lowercase() as u32,
            u8::from(ch.is_ascii_uppercase()),
        );
    }

    if ch.is_ascii_digit() {
        return (1, ch as u32, 0);
    }

    (2, ch as u32, 0)
}

fn expr_symbol(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol),
        _ => None,
    }
}

fn compare_rational_values(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

fn expr_order_rank(expr: &Expr) -> u8 {
    match expr {
        Expr::Atom(atom) => atom_order_rank(atom),
        Expr::Call { .. } => 3,
    }
}

fn atom_order_rank(atom: &Atom) -> u8 {
    match atom {
        Atom::Integer(_) | Atom::Rational(_) => 0,
        Atom::String(_) => 1,
        Atom::Symbol(_) => 2,
    }
}

pub(super) fn is_known_system_symbol(symbol: &Symbol) -> bool {
    if !symbol.is_system() {
        return false;
    }

    if rum_core::is_canonical_core_head_name(symbol.name()) {
        return true;
    }

    if default_builtin_dispatch_spec_for_symbol(symbol).is_some() {
        return true;
    }

    is_extra_known_system_symbol_name(symbol.name())
}

pub(super) fn is_known_system_symbol_name(name: &str) -> bool {
    if rum_core::is_canonical_core_head_name(name) {
        return true;
    }

    if default_builtin_dispatch_spec(name).is_some() {
        return true;
    }

    is_extra_known_system_symbol_name(name)
}

fn is_extra_known_system_symbol_name(name: &str) -> bool {
    matches!(
        name,
        "$AssertFunction"
            | "Alternatives"
            | "DateObject"
            | "DistanceFunction"
            | "Flat"
            | "Heads"
            | "HoldAll"
            | "HoldAllComplete"
            | "HoldFirst"
            | "HoldRest"
            | "Identity"
            | "Listable"
            | "NHoldAll"
            | "NearestTo"
            | "NumericFunction"
            | "OptionsPattern"
            | "EvaluationMonitor"
            | "DigitCharacter"
            | "LetterCharacter"
            | "NonThreadable"
            | "OneIdentity"
            | "Orderless"
            | "OrderlessPatternSequence"
            | "PatternTest"
            | "PatternSequence"
            | "PropagateAborts"
            | "Protected"
            | "ReadProtected"
            | "RegularExpression"
            | "StepMonitor"
            | "StringExpression"
            | "SubValuesHoldAll"
            | "Temporary"
            | "TimeObject"
            | "UpTo"
            | "Verbatim"
            | "WhitespaceCharacter"
            | "WordCharacter"
            | "SequenceHold"
    )
}

pub fn is_system_symbol_name(expr: &Expr, name: &str) -> bool {
    expr_as_symbol(expr)
        .map(|symbol| symbol.is_system_name(name))
        .unwrap_or(false)
}
