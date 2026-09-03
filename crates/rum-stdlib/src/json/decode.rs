use std::str::FromStr;

use super::MAX_JSON_DEPTH;
use super::*;
use crate::adaptive_set::AdaptiveSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JsonParseError {
    offset: usize,
    kind: JsonParseErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum JsonParseErrorKind {
    Syntax(String),
    Depth,
}

impl JsonParseError {
    fn syntax(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            kind: JsonParseErrorKind::Syntax(message.into()),
        }
    }

    fn depth(offset: usize) -> Self {
        Self {
            offset,
            kind: JsonParseErrorKind::Depth,
        }
    }

    pub(super) fn message_tag(&self) -> &'static str {
        match self.kind {
            JsonParseErrorKind::Syntax(_) => "json",
            JsonParseErrorKind::Depth => "depth",
        }
    }

    pub(super) fn message(&self) -> String {
        match &self.kind {
            JsonParseErrorKind::Syntax(message) => {
                format!("Invalid JSON at byte offset {}: {message}.", self.offset)
            }
            JsonParseErrorKind::Depth => format!(
                "JSON nesting exceeds the fixed depth limit of {MAX_JSON_DEPTH} at byte offset {}.",
                self.offset
            ),
        }
    }
}

type ParseResult<T> = Result<T, JsonParseError>;

pub(super) fn from_json(source: &str) -> ParseResult<Expr> {
    let mut parser = Parser::new(source);
    parser.skip_json_whitespace();
    let value = parser.parse_value(0)?;
    parser.skip_json_whitespace();
    if !parser.is_eof() {
        return Err(JsonParseError::syntax(
            parser.pos,
            "trailing non-whitespace after JSON value",
        ));
    }
    Ok(value)
}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        Some(byte)
    }

    fn skip_json_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn parse_value(&mut self, depth: usize) -> ParseResult<Expr> {
        self.skip_json_whitespace();
        match self.peek() {
            Some(b'{') => {
                if depth >= MAX_JSON_DEPTH {
                    return Err(JsonParseError::depth(self.pos));
                }
                self.parse_object(depth + 1)
            }
            Some(b'[') => {
                if depth >= MAX_JSON_DEPTH {
                    return Err(JsonParseError::depth(self.pos));
                }
                self.parse_array(depth + 1)
            }
            Some(b'"') => self.parse_string().map(Expr::string),
            Some(b't') => self.parse_literal("true", system_expr("True")),
            Some(b'f') => self.parse_literal("false", system_expr("False")),
            Some(b'n') => self.parse_literal("null", system_expr("Null")),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(b'\'') => Err(JsonParseError::syntax(
                self.pos,
                "JSON strings and object keys must use double quotes",
            )),
            Some(_) => Err(JsonParseError::syntax(self.pos, "expected a JSON value")),
            None => Err(JsonParseError::syntax(self.pos, "expected a JSON value")),
        }
    }

    fn parse_object(&mut self, depth: usize) -> ParseResult<Expr> {
        self.expect_byte(b'{', "expected object start")?;
        self.skip_json_whitespace();

        let mut seen_keys = AdaptiveSet::new();
        let mut entries = Vec::new();

        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Expr::call(system_expr("Association"), entries));
        }

        loop {
            self.skip_json_whitespace();
            let key_offset = self.pos;
            if self.peek() != Some(b'"') {
                return Err(JsonParseError::syntax(
                    self.pos,
                    "object keys must be double-quoted strings",
                ));
            }
            let key = self.parse_string()?;
            if !seen_keys.insert(key.clone()) {
                return Err(JsonParseError::syntax(
                    key_offset,
                    format!("duplicate object key {key:?}"),
                ));
            }
            self.skip_json_whitespace();
            self.expect_byte(b':', "expected ':' after object key")?;
            let value = self.parse_value(depth)?;
            entries.push(Expr::call(
                system_expr("Rule"),
                vec![Expr::string(key), value],
            ));

            self.skip_json_whitespace();
            match self.bump() {
                Some(b',') => continue,
                Some(b'}') => break,
                Some(_) => {
                    return Err(JsonParseError::syntax(
                        self.pos.saturating_sub(1),
                        "expected ',' or '}' after object member",
                    ));
                }
                None => {
                    return Err(JsonParseError::syntax(self.pos, "unterminated JSON object"));
                }
            }
        }

        Ok(Expr::call(system_expr("Association"), entries))
    }

    fn parse_array(&mut self, depth: usize) -> ParseResult<Expr> {
        self.expect_byte(b'[', "expected array start")?;
        self.skip_json_whitespace();

        let mut values = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Expr::call(system_expr("List"), values));
        }

        loop {
            values.push(self.parse_value(depth)?);
            self.skip_json_whitespace();
            match self.bump() {
                Some(b',') => continue,
                Some(b']') => break,
                Some(_) => {
                    return Err(JsonParseError::syntax(
                        self.pos.saturating_sub(1),
                        "expected ',' or ']' after array element",
                    ));
                }
                None => {
                    return Err(JsonParseError::syntax(self.pos, "unterminated JSON array"));
                }
            }
        }

        Ok(Expr::call(system_expr("List"), values))
    }

    fn parse_string(&mut self) -> ParseResult<String> {
        self.expect_byte(b'"', "expected string start")?;
        let mut out = String::new();

        loop {
            let Some(byte) = self.peek() else {
                return Err(JsonParseError::syntax(self.pos, "unterminated string"));
            };

            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let escape_offset = self.pos.saturating_sub(1);
                    self.parse_escape(escape_offset, &mut out)?;
                }
                0x00..=0x1f => {
                    return Err(JsonParseError::syntax(
                        self.pos,
                        "unescaped control character in string",
                    ));
                }
                _ => {
                    let ch = self.source[self.pos..]
                        .chars()
                        .next()
                        .expect("valid UTF-8 input should yield a char before EOF");
                    self.pos += ch.len_utf8();
                    out.push(ch);
                }
            }
        }
    }

    fn parse_escape(&mut self, escape_offset: usize, out: &mut String) -> ParseResult<()> {
        match self.bump() {
            Some(b'"') => out.push('"'),
            Some(b'\\') => out.push('\\'),
            Some(b'/') => out.push('/'),
            Some(b'b') => out.push('\u{0008}'),
            Some(b'f') => out.push('\u{000c}'),
            Some(b'n') => out.push('\n'),
            Some(b'r') => out.push('\r'),
            Some(b't') => out.push('\t'),
            Some(b'u') => self.parse_unicode_escape(escape_offset, out)?,
            Some(_) => {
                return Err(JsonParseError::syntax(
                    escape_offset,
                    "invalid string escape",
                ));
            }
            None => {
                return Err(JsonParseError::syntax(
                    escape_offset,
                    "unterminated string escape",
                ));
            }
        }
        Ok(())
    }

    fn parse_unicode_escape(&mut self, escape_offset: usize, out: &mut String) -> ParseResult<()> {
        let first = self.read_hex_u16(escape_offset)?;

        if is_high_surrogate(first) {
            if self.peek() != Some(b'\\') || self.peek_next() != Some(b'u') {
                return Err(JsonParseError::syntax(
                    escape_offset,
                    "high surrogate must be followed by a low surrogate escape",
                ));
            }
            self.pos += 2;
            let low_offset = self.pos.saturating_sub(2);
            let second = self.read_hex_u16(low_offset)?;
            if !is_low_surrogate(second) {
                return Err(JsonParseError::syntax(
                    low_offset,
                    "high surrogate must be followed by a low surrogate escape",
                ));
            }

            let codepoint = 0x10000 + (((first as u32 - 0xd800) << 10) | (second as u32 - 0xdc00));
            let ch = char::from_u32(codepoint).ok_or_else(|| {
                JsonParseError::syntax(escape_offset, "invalid Unicode scalar value")
            })?;
            out.push(ch);
            return Ok(());
        }

        if is_low_surrogate(first) {
            return Err(JsonParseError::syntax(
                escape_offset,
                "low surrogate cannot appear without a preceding high surrogate",
            ));
        }

        let ch = char::from_u32(first as u32)
            .ok_or_else(|| JsonParseError::syntax(escape_offset, "invalid Unicode scalar value"))?;
        out.push(ch);
        Ok(())
    }

    fn read_hex_u16(&mut self, offset: usize) -> ParseResult<u16> {
        if self.pos + 4 > self.bytes.len() {
            return Err(JsonParseError::syntax(
                offset,
                "Unicode escape must contain four hexadecimal digits",
            ));
        }

        let mut value = 0_u16;
        for _ in 0..4 {
            let byte = self.bump().expect("length checked above");
            let Some(digit) = hex_value(byte) else {
                return Err(JsonParseError::syntax(
                    self.pos.saturating_sub(1),
                    "Unicode escape must contain four hexadecimal digits",
                ));
            };
            value = value * 16 + digit as u16;
        }
        Ok(value)
    }

    fn parse_literal(&mut self, literal: &str, expr: Expr) -> ParseResult<Expr> {
        if self.source[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(expr)
        } else {
            Err(JsonParseError::syntax(
                self.pos,
                format!("expected literal {literal}"),
            ))
        }
    }

    fn parse_number(&mut self) -> ParseResult<Expr> {
        let start = self.pos;
        let negative = self.peek() == Some(b'-');
        if negative {
            self.pos += 1;
        }

        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(JsonParseError::syntax(
                        self.pos,
                        "leading zeros are not valid in JSON numbers",
                    ));
                }
            }
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(JsonParseError::syntax(start, "expected digits after '-'")),
        }

        let mut has_fraction = false;
        if self.peek() == Some(b'.') {
            has_fraction = true;
            self.pos += 1;
            let fraction_start = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            if self.pos == fraction_start {
                return Err(JsonParseError::syntax(
                    self.pos,
                    "expected digits after decimal point",
                ));
            }
        }

        let mut has_exponent = false;
        if matches!(self.peek(), Some(b'e' | b'E')) {
            has_exponent = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            let exponent_digits_start = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            if self.pos == exponent_digits_start {
                return Err(JsonParseError::syntax(self.pos, "expected exponent digits"));
            }
        }

        let raw = &self.source[start..self.pos];
        number_expr(raw, negative, has_fraction || has_exponent, start)
    }

    fn expect_byte(&mut self, expected: u8, message: &'static str) -> ParseResult<()> {
        match self.bump() {
            Some(byte) if byte == expected => Ok(()),
            Some(_) => Err(JsonParseError::syntax(self.pos.saturating_sub(1), message)),
            None => Err(JsonParseError::syntax(self.pos, message)),
        }
    }
}

fn number_expr(
    raw: &str,
    negative: bool,
    decimal_or_exponent: bool,
    offset: usize,
) -> ParseResult<Expr> {
    if !decimal_or_exponent {
        let value = BigInt::from_str(raw)
            .map_err(|_| JsonParseError::syntax(offset, "invalid integer literal"))?;
        return Ok(Expr::integer(value));
    }

    let mut digits = String::new();
    let mut fraction_len = 0_usize;
    let mut in_fraction = false;
    let mut exponent_start = None;

    for (index, byte) in raw.bytes().enumerate() {
        match byte {
            b'-' if index == 0 => {}
            b'.' => in_fraction = true,
            b'e' | b'E' => {
                exponent_start = Some(index + 1);
                break;
            }
            b'0'..=b'9' => {
                digits.push(byte as char);
                if in_fraction {
                    fraction_len += 1;
                }
            }
            _ => {}
        }
    }

    let exponent = exponent_start
        .map(|start| {
            raw[start..]
                .parse::<i64>()
                .map_err(|_| JsonParseError::syntax(offset + start, "number exponent is too large"))
        })
        .transpose()?
        .unwrap_or(0);
    let scale = exponent as i128 - fraction_len as i128;

    let mut numerator = BigInt::from_str(&digits)
        .map_err(|_| JsonParseError::syntax(offset, "invalid decimal literal"))?;
    if negative {
        numerator = -numerator;
    }

    if scale >= 0 {
        let scale = usize::try_from(scale)
            .map_err(|_| JsonParseError::syntax(offset, "number exponent is too large"))?;
        numerator *= pow10(scale);
        return Ok(Expr::integer(numerator));
    }

    let denominator_scale = usize::try_from(-scale)
        .map_err(|_| JsonParseError::syntax(offset, "number exponent is too large"))?;
    let denominator = pow10(denominator_scale);
    let rational = Rational::new(numerator, denominator)
        .expect("decimal literal denominator is always non-zero");
    Ok(Expr::rational(rational))
}

fn pow10(exponent: usize) -> BigInt {
    let mut value = BigInt::from(1_u8);
    for _ in 0..exponent {
        value *= 10_u8;
    }
    value
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn is_high_surrogate(value: u16) -> bool {
    (0xd800..=0xdbff).contains(&value)
}

fn is_low_surrogate(value: u16) -> bool {
    (0xdc00..=0xdfff).contains(&value)
}
