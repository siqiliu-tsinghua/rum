use rum_core::SharedArgs;
use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::{
    EvalError, Session, association_entries_from_expr, expr_as_non_negative_usize, expr_as_string,
    expr_as_symbol, is_system_symbol_name, list_items, render_association, system_expr,
};

mod decode;
mod encode;

const MAX_XML_DEPTH: usize = 256;

pub(crate) fn eval_from_xml(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [source] = args else {
        return Ok(fail_xml(
            session,
            "FromXml",
            "xml",
            "FromXml expects exactly one string argument.",
        ));
    };

    let Some(source) = expr_as_string(source) else {
        return Ok(fail_xml(
            session,
            "FromXml",
            "xml",
            "FromXml expects an XML text string.",
        ));
    };

    match decode::from_xml(source) {
        Ok(expr) => Ok(expr),
        Err(error) => Ok(fail_xml(
            session,
            "FromXml",
            error.message_tag(),
            error.message(),
        )),
    }
}

pub(crate) fn eval_to_xml(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, indent) = match parse_to_xml_args(args) {
        Ok(parsed) => parsed,
        Err(message) => return Ok(fail_xml(session, "ToXml", "xml", message)),
    };

    match encode::to_xml(expr, indent) {
        Ok(xml) => Ok(Expr::string(xml)),
        Err(error) => Ok(fail_xml(
            session,
            "ToXml",
            error.message_tag(),
            error.message(),
        )),
    }
}

fn parse_to_xml_args(args: &[Expr]) -> Result<(&Expr, Option<usize>), String> {
    match args {
        [expr] => Ok((expr, None)),
        [expr, option] => {
            let Some((name, value)) = option_rule(option) else {
                return Err(
                    "ToXml expects the optional second argument to be \"Indent\" -> n.".to_string(),
                );
            };
            if name != "Indent" {
                return Err(format!("Unknown ToXml option {name}."));
            }
            let Some(indent) = expr_as_non_negative_usize(value) else {
                return Err("ToXml option \"Indent\" must be a non-negative integer.".to_string());
            };
            Ok((expr, Some(indent)))
        }
        _ => Err(
            "ToXml expects one XmlElement tree and an optional \"Indent\" -> n option.".to_string(),
        ),
    }
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

    let name = expr_as_string(&args[0])
        .or_else(|| expr_as_symbol(&args[0]).map(|symbol| symbol.name()))?;
    Some((name, &args[1]))
}

fn fail_xml(
    session: &mut Session,
    symbol: &str,
    tag: &'static str,
    message: impl Into<String>,
) -> Expr {
    session.push_message(symbol, tag, message);
    system_expr("$Failed")
}

fn xml_element_expr(tag: String, attrs: Vec<(String, String)>, children: Vec<Expr>) -> Expr {
    let attrs = render_association(
        attrs
            .into_iter()
            .map(|(key, value)| (Expr::string(key), Expr::string(value)))
            .collect(),
    );
    let children = Expr::call(system_expr("List"), children);
    Expr::call(
        system_expr("XmlElement"),
        vec![Expr::string(tag), attrs, children],
    )
}

fn is_xml_element_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "XmlElement"))
}

fn is_xml_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    is_xml_name_start(first) && bytes.all(is_xml_name_continue)
}

fn is_xml_name_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b':')
}

fn is_xml_name_continue(byte: u8) -> bool {
    is_xml_name_start(byte) || byte.is_ascii_digit() || matches!(byte, b'-' | b'.')
}

fn is_xml_char(ch: char) -> bool {
    matches!(
        ch as u32,
        0x09 | 0x0a | 0x0d | 0x20..=0xd7ff | 0xe000..=0xfffd | 0x10000..=0x10ffff
    )
}
