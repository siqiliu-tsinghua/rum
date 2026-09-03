use super::MAX_XML_DEPTH;
use super::*;
use crate::adaptive_set::AdaptiveSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct XmlParseError {
    offset: usize,
    kind: XmlParseErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum XmlParseErrorKind {
    Syntax(String),
    Depth,
}

impl XmlParseError {
    fn syntax(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            kind: XmlParseErrorKind::Syntax(message.into()),
        }
    }

    fn depth(offset: usize) -> Self {
        Self {
            offset,
            kind: XmlParseErrorKind::Depth,
        }
    }

    fn dtd(offset: usize) -> Self {
        Self::syntax(
            offset,
            "DTD/DOCTYPE declarations are deliberately unsupported; FromXml never processes DTDs or external entities",
        )
    }

    pub(super) fn message_tag(&self) -> &'static str {
        match self.kind {
            XmlParseErrorKind::Syntax(_) => "xml",
            XmlParseErrorKind::Depth => "depth",
        }
    }

    pub(super) fn message(&self) -> String {
        match &self.kind {
            XmlParseErrorKind::Syntax(message) => {
                format!("Invalid XML at byte offset {}: {message}.", self.offset)
            }
            XmlParseErrorKind::Depth => format!(
                "XML nesting exceeds the fixed depth limit of {MAX_XML_DEPTH} at byte offset {}.",
                self.offset
            ),
        }
    }
}

type ParseResult<T> = Result<T, XmlParseError>;

pub(super) fn from_xml(source: &str) -> ParseResult<Expr> {
    let mut parser = Parser::new(source);
    parser.parse_document()
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

    fn parse_document(&mut self) -> ParseResult<Expr> {
        self.skip_xml_whitespace();
        if self.starts_with("<?xml") {
            if self.is_xml_declaration_start() {
                self.parse_xml_declaration()?;
            } else {
                return Err(self.processing_instruction_error());
            }
        } else if self.starts_with("<?") {
            return Err(self.processing_instruction_error());
        }

        self.skip_misc()?;
        if self.is_eof() {
            return Err(XmlParseError::syntax(self.pos, "expected a root element"));
        }
        if self.starts_with("<!DOCTYPE") {
            return Err(XmlParseError::dtd(self.pos));
        }

        let root = self.parse_element(0)?;
        self.skip_misc()?;
        if self.is_eof() {
            return Ok(root);
        }
        if self.starts_with("<!DOCTYPE") {
            return Err(XmlParseError::dtd(self.pos));
        }
        if self.starts_with("<?") {
            return Err(self.processing_instruction_error());
        }
        if self.peek() == Some(b'<') {
            return Err(XmlParseError::syntax(self.pos, "multiple root elements"));
        }
        Err(XmlParseError::syntax(
            self.pos,
            "non-whitespace text after the root element",
        ))
    }

    fn parse_xml_declaration(&mut self) -> ParseResult<()> {
        let declaration_offset = self.pos;
        self.pos += "<?xml".len();
        self.require_xml_whitespace(self.pos, "XML declaration requires whitespace after <?xml")?;

        let mut fields = Vec::new();
        loop {
            self.skip_xml_whitespace();
            if self.starts_with("?>") {
                self.pos += 2;
                break;
            }
            if self.is_eof() {
                return Err(XmlParseError::syntax(
                    self.pos,
                    "unterminated XML declaration",
                ));
            }

            let name_offset = self.pos;
            let name = self
                .parse_name()
                .ok_or_else(|| XmlParseError::syntax(self.pos, "expected XML declaration field"))?;
            self.skip_xml_whitespace();
            self.expect_byte(b'=', "expected '=' after XML declaration field")?;
            self.skip_xml_whitespace();
            let value = self.parse_quoted_value()?;
            fields.push((name, value, name_offset));
            if !self.starts_with("?>") && !matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t'))
            {
                return Err(XmlParseError::syntax(
                    self.pos,
                    "XML declaration fields must be separated by whitespace",
                ));
            }
        }

        self.validate_xml_declaration(declaration_offset, &fields)
    }

    fn validate_xml_declaration(
        &self,
        declaration_offset: usize,
        fields: &[(String, String, usize)],
    ) -> ParseResult<()> {
        if fields.is_empty() || fields[0].0 != "version" {
            return Err(XmlParseError::syntax(
                declaration_offset,
                "XML declaration requires version as its first field",
            ));
        }

        let mut order = 0_u8;
        for (name, value, offset) in fields {
            match name.as_str() {
                "version" if order == 0 => {
                    if value != "1.0" && value != "1.1" {
                        return Err(XmlParseError::syntax(
                            *offset,
                            "XML declaration version must be \"1.0\" or \"1.1\"",
                        ));
                    }
                    order = 1;
                }
                "encoding" if order == 1 => {
                    if !valid_encoding_name(value) {
                        return Err(XmlParseError::syntax(
                            *offset,
                            "XML declaration encoding is not a valid encoding name",
                        ));
                    }
                    order = 2;
                }
                "standalone" if order >= 1 && order <= 2 => {
                    if value != "yes" && value != "no" {
                        return Err(XmlParseError::syntax(
                            *offset,
                            "XML declaration standalone must be \"yes\" or \"no\"",
                        ));
                    }
                    order = 3;
                }
                "version" | "encoding" | "standalone" => {
                    return Err(XmlParseError::syntax(
                        *offset,
                        "XML declaration fields must appear as version, optional encoding, optional standalone",
                    ));
                }
                _ => {
                    return Err(XmlParseError::syntax(
                        *offset,
                        format!("unsupported XML declaration field {name:?}"),
                    ));
                }
            }
        }

        Ok(())
    }

    fn parse_element(&mut self, depth: usize) -> ParseResult<Expr> {
        if depth >= MAX_XML_DEPTH {
            return Err(XmlParseError::depth(self.pos));
        }
        let element_offset = self.pos;
        if self.starts_with("<!DOCTYPE") {
            return Err(XmlParseError::dtd(self.pos));
        }
        if self.starts_with("<?") {
            return Err(self.processing_instruction_error());
        }
        if self.starts_with("</") {
            return Err(XmlParseError::syntax(self.pos, "unexpected closing tag"));
        }
        self.expect_byte(b'<', "expected element start")?;

        let tag_offset = self.pos;
        let tag = self
            .parse_name()
            .ok_or_else(|| XmlParseError::syntax(tag_offset, "expected element name"))?;

        let mut seen_attrs = AdaptiveSet::new();
        let mut attrs = Vec::new();
        loop {
            self.skip_xml_whitespace();
            if self.starts_with("/>") {
                self.pos += 2;
                return Ok(xml_element_expr(tag, attrs, Vec::new()));
            }
            if self.peek() == Some(b'>') {
                self.pos += 1;
                break;
            }
            if self.is_eof() {
                return Err(XmlParseError::syntax(
                    self.pos,
                    format!("unterminated start tag <{tag}>"),
                ));
            }

            let attr_offset = self.pos;
            let name = self.parse_name().ok_or_else(|| {
                XmlParseError::syntax(self.pos, "expected attribute name or tag close")
            })?;
            if !seen_attrs.insert(name.clone()) {
                return Err(XmlParseError::syntax(
                    attr_offset,
                    format!("duplicate attribute {name:?}"),
                ));
            }
            self.skip_xml_whitespace();
            self.expect_byte(b'=', "expected '=' after attribute name")?;
            self.skip_xml_whitespace();
            let value = self.parse_quoted_value()?;
            attrs.push((name, value));
        }

        let children = self.parse_children(&tag, element_offset, depth)?;
        Ok(xml_element_expr(tag, attrs, children))
    }

    fn parse_children(
        &mut self,
        tag: &str,
        element_offset: usize,
        depth: usize,
    ) -> ParseResult<Vec<Expr>> {
        let mut children = Vec::new();
        let mut pending_text = String::new();

        loop {
            if self.is_eof() {
                return Err(XmlParseError::syntax(
                    self.pos,
                    format!("unclosed tag <{tag}> starting at byte offset {element_offset}"),
                ));
            }

            if self.starts_with("</") {
                self.flush_text(&mut children, &mut pending_text);
                let close_offset = self.pos;
                self.pos += 2;
                let close_name_offset = self.pos;
                let close_name = self.parse_name().ok_or_else(|| {
                    XmlParseError::syntax(close_name_offset, "expected closing tag name")
                })?;
                self.skip_xml_whitespace();
                self.expect_byte(b'>', "expected '>' after closing tag name")?;
                if close_name != tag {
                    return Err(XmlParseError::syntax(
                        close_offset,
                        format!("mismatched closing tag </{close_name}>; expected </{tag}>"),
                    ));
                }
                return Ok(normalize_children(children));
            }

            if self.starts_with("<![CDATA[") {
                pending_text.push_str(&self.parse_cdata()?);
            } else if self.starts_with("<!--") {
                self.parse_comment()?;
            } else if self.starts_with("<!DOCTYPE") {
                return Err(XmlParseError::dtd(self.pos));
            } else if self.starts_with("<?") {
                return Err(self.processing_instruction_error());
            } else if self.peek() == Some(b'<') {
                if self.byte_at(self.pos + 1).is_some_and(is_xml_name_start) {
                    self.flush_text(&mut children, &mut pending_text);
                    children.push(self.parse_element(depth + 1)?);
                } else {
                    return Err(XmlParseError::syntax(self.pos, "bare '<' in text"));
                }
            } else {
                self.parse_text(&mut pending_text)?;
            }
        }
    }

    fn parse_text(&mut self, out: &mut String) -> ParseResult<()> {
        while !self.is_eof() && self.peek() != Some(b'<') {
            if self.peek() == Some(b'&') {
                out.push(self.parse_entity()?);
            } else {
                let offset = self.pos;
                let ch = self
                    .next_char()
                    .expect("non-EOF position should have a char");
                if !is_xml_char(ch) {
                    return Err(XmlParseError::syntax(
                        offset,
                        "character is not valid in XML text",
                    ));
                }
                out.push(ch);
            }
        }
        Ok(())
    }

    fn parse_quoted_value(&mut self) -> ParseResult<String> {
        let Some(quote @ (b'\'' | b'"')) = self.peek() else {
            return Err(XmlParseError::syntax(
                self.pos,
                "attribute values must be single- or double-quoted",
            ));
        };
        self.pos += 1;
        let mut out = String::new();

        loop {
            let Some(byte) = self.peek() else {
                return Err(XmlParseError::syntax(
                    self.pos,
                    "unterminated attribute value",
                ));
            };
            if byte == quote {
                self.pos += 1;
                return Ok(out);
            }
            if byte == b'&' {
                out.push(self.parse_entity()?);
            } else if byte == b'<' {
                return Err(XmlParseError::syntax(
                    self.pos,
                    "bare '<' in attribute value",
                ));
            } else {
                let offset = self.pos;
                let ch = self
                    .next_char()
                    .expect("non-EOF position should have a char");
                if !is_xml_char(ch) {
                    return Err(XmlParseError::syntax(
                        offset,
                        "character is not valid in an XML attribute value",
                    ));
                }
                out.push(ch);
            }
        }
    }

    fn parse_entity(&mut self) -> ParseResult<char> {
        let amp_offset = self.pos;
        self.pos += 1;
        let body_start = self.pos;
        while !self.is_eof() && !matches!(self.peek(), Some(b';' | b'<')) {
            self.pos += 1;
        }
        if self.peek() != Some(b';') {
            return Err(XmlParseError::syntax(
                amp_offset,
                "bare '&' in entity reference",
            ));
        }

        let body = &self.source[body_start..self.pos];
        self.pos += 1;
        match body {
            "lt" => Ok('<'),
            "gt" => Ok('>'),
            "amp" => Ok('&'),
            "apos" => Ok('\''),
            "quot" => Ok('"'),
            _ if body.starts_with("#x") || body.starts_with("#X") => {
                self.parse_numeric_entity(amp_offset, &body[2..], 16)
            }
            _ if body.starts_with('#') => self.parse_numeric_entity(amp_offset, &body[1..], 10),
            _ => Err(XmlParseError::syntax(
                amp_offset,
                format!("unknown named entity &{body};"),
            )),
        }
    }

    fn parse_numeric_entity(&self, offset: usize, digits: &str, radix: u32) -> ParseResult<char> {
        if digits.is_empty() || !digits.bytes().all(|byte| (byte as char).is_digit(radix)) {
            return Err(XmlParseError::syntax(
                offset,
                "invalid numeric entity reference",
            ));
        }
        let value = u32::from_str_radix(digits, radix).map_err(|_| {
            XmlParseError::syntax(offset, "numeric entity reference is out of range")
        })?;
        let Some(ch) = char::from_u32(value) else {
            return Err(XmlParseError::syntax(
                offset,
                "numeric entity reference is not a Unicode scalar value",
            ));
        };
        if !is_xml_char(ch) {
            return Err(XmlParseError::syntax(
                offset,
                "numeric entity reference is not valid XML text",
            ));
        }
        Ok(ch)
    }

    fn parse_comment(&mut self) -> ParseResult<()> {
        let start = self.pos;
        let content_start = self.pos + "<!--".len();
        let Some(relative_end) = self.source[content_start..].find("-->") else {
            return Err(XmlParseError::syntax(start, "unterminated XML comment"));
        };
        self.pos = content_start + relative_end + "-->".len();
        Ok(())
    }

    fn parse_cdata(&mut self) -> ParseResult<String> {
        let start = self.pos;
        let content_start = self.pos + "<![CDATA[".len();
        let Some(relative_end) = self.source[content_start..].find("]]>") else {
            return Err(XmlParseError::syntax(start, "unterminated CDATA section"));
        };
        let content_end = content_start + relative_end;
        self.pos = content_end + "]]>".len();
        Ok(self.source[content_start..content_end].to_string())
    }

    fn parse_name(&mut self) -> Option<String> {
        let first = self.peek()?;
        if !is_xml_name_start(first) {
            return None;
        }
        let start = self.pos;
        self.pos += 1;
        while self.peek().is_some_and(is_xml_name_continue) {
            self.pos += 1;
        }
        Some(self.source[start..self.pos].to_string())
    }

    fn skip_misc(&mut self) -> ParseResult<()> {
        loop {
            self.skip_xml_whitespace();
            if self.starts_with("<!--") {
                self.parse_comment()?;
            } else if self.starts_with("<!DOCTYPE") {
                return Err(XmlParseError::dtd(self.pos));
            } else if self.starts_with("<?") {
                return Err(self.processing_instruction_error());
            } else {
                return Ok(());
            }
        }
    }

    fn require_xml_whitespace(&mut self, offset: usize, message: &str) -> ParseResult<()> {
        if !matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            return Err(XmlParseError::syntax(offset, message));
        }
        self.skip_xml_whitespace();
        Ok(())
    }

    fn skip_xml_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn expect_byte(&mut self, expected: u8, message: &str) -> ParseResult<()> {
        match self.bump() {
            Some(byte) if byte == expected => Ok(()),
            Some(_) => Err(XmlParseError::syntax(self.pos.saturating_sub(1), message)),
            None => Err(XmlParseError::syntax(self.pos, message)),
        }
    }

    fn processing_instruction_error(&self) -> XmlParseError {
        XmlParseError::syntax(
            self.pos,
            "processing instructions other than the XML declaration are unsupported",
        )
    }

    fn flush_text(&self, children: &mut Vec<Expr>, pending_text: &mut String) {
        if !pending_text.is_empty() {
            children.push(Expr::string(std::mem::take(pending_text)));
        }
    }

    fn is_xml_declaration_start(&self) -> bool {
        self.starts_with("<?xml")
            && self
                .byte_at(self.pos + "<?xml".len())
                .is_some_and(|byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
    }

    fn starts_with(&self, needle: &str) -> bool {
        self.source[self.pos..].starts_with(needle)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.byte_at(self.pos)
    }

    fn byte_at(&self, pos: usize) -> Option<u8> {
        self.bytes.get(pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        Some(byte)
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.source[self.pos..].chars().next()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }
}

fn normalize_children(children: Vec<Expr>) -> Vec<Expr> {
    let has_element_child = children.iter().any(is_xml_element_expr);
    if !has_element_child {
        return children;
    }

    children
        .into_iter()
        .filter(|child| match child {
            Expr::Atom(Atom::String(value)) => !value.chars().all(char::is_whitespace),
            _ => true,
        })
        .collect()
}

fn valid_encoding_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}
