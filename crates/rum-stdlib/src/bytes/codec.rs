use rum_core::Expr;
use rum_runtime::kernel_api::{Session, expr_as_string, is_system_symbol_name, system_expr};

pub(super) enum ByteInput {
    Bytes(Vec<u8>),
    InvalidByteArray,
    WrongType,
}

pub(super) fn input_bytes(expr: &Expr) -> ByteInput {
    if let Some(value) = expr_as_string(expr) {
        return ByteInput::Bytes(value.as_bytes().to_vec());
    }

    if !has_byte_array_head(expr) {
        return ByteInput::WrongType;
    }

    match expr_as_byte_array_bytes(expr) {
        Some(bytes) => ByteInput::Bytes(bytes),
        None => ByteInput::InvalidByteArray,
    }
}

pub(super) fn has_byte_array_head(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args, .. }
            if is_system_symbol_name(head, "ByteArray") && args.len() == 1
    )
}

pub(super) fn expr_as_byte_array_bytes(expr: &Expr) -> Option<Vec<u8>> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "ByteArray") || args.len() != 1 {
        return None;
    }
    let encoded = expr_as_string(&args[0])?;
    base64_decode(encoded)
}

pub(super) fn byte_array_expr(bytes: &[u8]) -> Expr {
    Expr::call(
        system_expr("ByteArray"),
        vec![Expr::string(base64_encode(bytes))],
    )
}

pub(super) fn bool_expr(value: bool) -> Expr {
    system_expr(if value { "True" } else { "False" })
}

pub(super) fn fail_bytes(
    session: &mut Session,
    symbol: &str,
    tag: &'static str,
    message: impl Into<String>,
) -> Expr {
    session.push_message(symbol, tag, message);
    system_expr("$Failed")
}

pub(super) fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        rendered.push(nibble_to_hex(byte >> 4));
        rendered.push(nibble_to_hex(byte & 0x0f));
    }
    rendered
}

pub(super) fn hex_to_bytes(source: &str) -> Option<Vec<u8>> {
    let bytes = source.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }

    let mut output = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = hex_digit_value(pair[0])?;
        let low = hex_digit_value(pair[1])?;
        output.push((high << 4) | low);
    }
    Some(output)
}

fn hex_digit_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + (nibble - 10)),
        _ => unreachable!("nibbles are always in 0..=15"),
    }
}

pub(super) fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut rendered = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut index = 0;
    while index + 3 <= bytes.len() {
        let chunk = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | (bytes[index + 2] as u32);
        rendered.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
        rendered.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
        rendered.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
        rendered.push(ALPHABET[(chunk & 0x3f) as usize] as char);
        index += 3;
    }

    match bytes.len() - index {
        1 => {
            let chunk = (bytes[index] as u32) << 16;
            rendered.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            rendered.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
            rendered.push('=');
            rendered.push('=');
        }
        2 => {
            let chunk = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
            rendered.push(ALPHABET[((chunk >> 18) & 0x3f) as usize] as char);
            rendered.push(ALPHABET[((chunk >> 12) & 0x3f) as usize] as char);
            rendered.push(ALPHABET[((chunk >> 6) & 0x3f) as usize] as char);
            rendered.push('=');
        }
        _ => {}
    }

    rendered
}

pub(super) fn base64_decode(encoded: &str) -> Option<Vec<u8>> {
    let bytes = encoded.as_bytes();
    if bytes.len() % 4 != 0 {
        return None;
    }

    let total_chunks = bytes.len() / 4;
    let mut output = Vec::with_capacity(total_chunks * 3);
    for (chunk_index, chunk) in bytes.chunks_exact(4).enumerate() {
        let a = decode_base64_value(chunk[0])?;
        let b = decode_base64_value(chunk[1])?;
        let c = chunk[2];
        let d = chunk[3];

        if c == b'=' && d != b'=' {
            return None;
        }
        let c_value = if c == b'=' {
            0
        } else {
            decode_base64_value(c)?
        };
        let d_value = if d == b'=' {
            0
        } else {
            decode_base64_value(d)?
        };

        if (c == b'=' || d == b'=') && chunk_index + 1 != total_chunks {
            return None;
        }
        if c == b'=' && (b & 0x0f) != 0 {
            return None;
        }
        if d == b'=' && c != b'=' && (c_value & 0x03) != 0 {
            return None;
        }

        let packed =
            ((a as u32) << 18) | ((b as u32) << 12) | ((c_value as u32) << 6) | d_value as u32;
        output.push(((packed >> 16) & 0xff) as u8);
        if c != b'=' {
            output.push(((packed >> 8) & 0xff) as u8);
        }
        if d != b'=' {
            output.push((packed & 0xff) as u8);
        }
    }

    Some(output)
}

fn decode_base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
