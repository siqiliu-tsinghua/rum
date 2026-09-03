use super::MAX_XML_DEPTH;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct XmlEncodeError {
    kind: XmlEncodeErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum XmlEncodeErrorKind {
    Message(String),
    Depth,
}

impl XmlEncodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            kind: XmlEncodeErrorKind::Message(message.into()),
        }
    }

    fn depth() -> Self {
        Self {
            kind: XmlEncodeErrorKind::Depth,
        }
    }

    pub(super) fn message_tag(&self) -> &'static str {
        match self.kind {
            XmlEncodeErrorKind::Message(_) => "xml",
            XmlEncodeErrorKind::Depth => "depth",
        }
    }

    pub(super) fn message(&self) -> String {
        match &self.kind {
            XmlEncodeErrorKind::Message(message) => message.clone(),
            XmlEncodeErrorKind::Depth => {
                format!("XML nesting exceeds the fixed depth limit of {MAX_XML_DEPTH}.")
            }
        }
    }
}

type EncodeResult<T> = Result<T, XmlEncodeError>;

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

pub(super) fn to_xml(expr: &Expr, indent: Option<usize>) -> EncodeResult<String> {
    let config = indent.map_or_else(EncodeConfig::compact, EncodeConfig::pretty);
    let mut out = String::new();
    write_element(expr, config, 0, &mut out)?;
    Ok(out)
}

fn write_element(
    expr: &Expr,
    config: EncodeConfig,
    level: usize,
    out: &mut String,
) -> EncodeResult<()> {
    if level >= MAX_XML_DEPTH {
        return Err(XmlEncodeError::depth());
    }
    let Expr::Call { head, args, .. } = expr else {
        return Err(XmlEncodeError::new(
            "ToXml expects an XmlElement[tag, attrs, children] tree.",
        ));
    };
    if !is_system_symbol_name(head, "XmlElement") || args.len() != 3 {
        return Err(XmlEncodeError::new(
            "ToXml expects an XmlElement[tag, attrs, children] tree.",
        ));
    }

    let tag = xml_string(&args[0], "XmlElement tag must be a String.")?;
    validate_name(tag, "tag")?;

    let attrs = association_entries_from_expr(&args[1])
        .ok_or_else(|| XmlEncodeError::new("XmlElement attributes must be an Association."))?;
    let children = list_items(&args[2])
        .ok_or_else(|| XmlEncodeError::new("XmlElement children must be a List."))?;
    validate_children(children)?;

    out.push('<');
    out.push_str(tag);
    for (key, value) in attrs {
        let key = xml_string(&key, "XmlElement attribute names must be Strings.")?;
        let value = xml_string(&value, "XmlElement attribute values must be Strings.")?;
        validate_name(key, "attribute name")?;
        validate_text(value, "attribute value")?;
        out.push(' ');
        out.push_str(key);
        out.push_str("=\"");
        write_escaped_attribute(value, out);
        out.push('"');
    }

    if children.is_empty() {
        out.push_str("/>");
        return Ok(());
    }

    out.push('>');
    if config.is_pretty() && element_only(children) {
        out.push('\n');
        for child in children {
            config.push_indent(out, level + 1);
            write_element(child, config, level + 1, out)?;
            out.push('\n');
        }
        config.push_indent(out, level);
    } else {
        for child in children {
            write_child(child, config, level + 1, out)?;
        }
    }
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
    Ok(())
}

fn write_child(
    child: &Expr,
    config: EncodeConfig,
    level: usize,
    out: &mut String,
) -> EncodeResult<()> {
    match child {
        Expr::Atom(Atom::String(value)) => {
            validate_text(value, "text node")?;
            write_escaped_text(value, out);
            Ok(())
        }
        Expr::Call { .. } if is_xml_element_expr(child) => write_element(child, config, level, out),
        _ => Err(XmlEncodeError::new(
            "XmlElement children must be Strings or XmlElement expressions.",
        )),
    }
}

fn validate_children(children: &[Expr]) -> EncodeResult<()> {
    let has_element_child = children.iter().any(is_xml_element_expr);
    for child in children {
        match child {
            Expr::Atom(Atom::String(value)) => {
                if value.is_empty() {
                    return Err(XmlEncodeError::new(
                        "ToXml cannot encode empty text nodes because XML cannot round-trip them.",
                    ));
                }
                if has_element_child && value.chars().all(char::is_whitespace) {
                    return Err(XmlEncodeError::new(
                        "ToXml cannot encode whitespace-only mixed-content text nodes because FromXml drops inter-element whitespace.",
                    ));
                }
            }
            Expr::Call { .. } if is_xml_element_expr(child) => {}
            _ => {
                return Err(XmlEncodeError::new(
                    "XmlElement children must be Strings or XmlElement expressions.",
                ));
            }
        }
    }
    Ok(())
}

fn element_only(children: &[Expr]) -> bool {
    !children.is_empty() && children.iter().all(is_xml_element_expr)
}

fn xml_string<'a>(expr: &'a Expr, message: &str) -> EncodeResult<&'a str> {
    let Expr::Atom(Atom::String(value)) = expr else {
        return Err(XmlEncodeError::new(message));
    };
    Ok(value)
}

fn validate_name(name: &str, kind: &str) -> EncodeResult<()> {
    if is_xml_name(name) {
        Ok(())
    } else {
        Err(XmlEncodeError::new(format!(
            "XML {kind} {name:?} is outside the supported XML name subset."
        )))
    }
}

fn validate_text(value: &str, kind: &str) -> EncodeResult<()> {
    if value.chars().all(is_xml_char) {
        Ok(())
    } else {
        Err(XmlEncodeError::new(format!(
            "XML {kind} contains a character that is not valid XML text."
        )))
    }
}

fn write_escaped_text(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            other => out.push(other),
        }
    }
}

fn write_escaped_attribute(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
}

#[cfg(test)]
mod depth_tests {
    use super::*;

    fn nested_xml_elements(levels: usize) -> Expr {
        let mut expr = xml_element_expr("a".to_string(), Vec::new(), Vec::new());
        for _ in 1..levels {
            expr = xml_element_expr("a".to_string(), Vec::new(), vec![expr]);
        }
        expr
    }

    #[test]
    fn to_xml_accepts_nesting_at_the_depth_limit() {
        assert!(to_xml(&nested_xml_elements(MAX_XML_DEPTH), None).is_ok());
    }

    #[test]
    fn to_xml_refuses_nesting_one_past_the_depth_limit() {
        let error = to_xml(&nested_xml_elements(MAX_XML_DEPTH + 1), None).unwrap_err();
        assert_eq!(error, XmlEncodeError::depth());
        assert_eq!(error.message_tag(), "depth");
        assert_eq!(
            error.message(),
            format!("XML nesting exceeds the fixed depth limit of {MAX_XML_DEPTH}.")
        );
    }
}
