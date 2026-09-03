use rum_core::SharedArgs;
use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::{EvalError, Session, expr_as_string, system_expr};

mod decode;
mod encode;
mod number;

const MAX_YAML_DEPTH: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
struct YamlParseError {
    offset: usize,
    kind: YamlParseErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum YamlParseErrorKind {
    Syntax(String),
    Depth,
}

impl YamlParseError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            kind: YamlParseErrorKind::Syntax(message.into()),
        }
    }

    fn depth(offset: usize) -> Self {
        Self {
            offset,
            kind: YamlParseErrorKind::Depth,
        }
    }

    fn message_tag(&self) -> &'static str {
        match self.kind {
            YamlParseErrorKind::Syntax(_) => "yaml",
            YamlParseErrorKind::Depth => "depth",
        }
    }

    fn message(&self) -> String {
        match &self.kind {
            YamlParseErrorKind::Syntax(message) => {
                format!("Invalid YAML at byte offset {}: {message}.", self.offset)
            }
            YamlParseErrorKind::Depth => format!(
                "YAML nesting exceeds the fixed depth limit of {MAX_YAML_DEPTH} at byte offset {}.",
                self.offset
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct YamlEncodeError {
    kind: YamlEncodeErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum YamlEncodeErrorKind {
    Message(String),
    Depth,
}

impl YamlEncodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            kind: YamlEncodeErrorKind::Message(message.into()),
        }
    }

    fn depth() -> Self {
        Self {
            kind: YamlEncodeErrorKind::Depth,
        }
    }

    fn message_tag(&self) -> &'static str {
        match self.kind {
            YamlEncodeErrorKind::Message(_) => "yaml",
            YamlEncodeErrorKind::Depth => "depth",
        }
    }

    fn message(&self) -> String {
        match &self.kind {
            YamlEncodeErrorKind::Message(message) => message.clone(),
            YamlEncodeErrorKind::Depth => {
                format!("YAML nesting exceeds the fixed depth limit of {MAX_YAML_DEPTH}.")
            }
        }
    }
}

pub(crate) fn eval_from_yaml(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [source] = args else {
        return Ok(fail_yaml(
            session,
            "FromYaml",
            "yaml",
            "FromYaml expects exactly one string argument.",
        ));
    };

    let Some(source) = expr_as_string(source) else {
        return Ok(fail_yaml(
            session,
            "FromYaml",
            "yaml",
            "FromYaml expects a YAML text string.",
        ));
    };

    match decode::from_yaml(source) {
        Ok(expr) => Ok(expr),
        Err(error) => Ok(fail_yaml(
            session,
            "FromYaml",
            error.message_tag(),
            error.message(),
        )),
    }
}

pub(crate) fn eval_to_yaml(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(fail_yaml(
            session,
            "ToYaml",
            "yaml",
            "ToYaml expects exactly one expression argument.",
        ));
    };

    match encode::to_yaml(expr) {
        Ok(output) => Ok(Expr::string(output)),
        Err(error) => Ok(fail_yaml(
            session,
            "ToYaml",
            error.message_tag(),
            error.message(),
        )),
    }
}

fn fail_yaml(
    session: &mut Session,
    symbol: &str,
    tag: &'static str,
    message: impl Into<String>,
) -> Expr {
    session.push_message(symbol, tag, message);
    system_expr("$Failed")
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn association_expr(entries: Vec<(String, Expr)>) -> Expr {
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(|(key, value)| Expr::call(system_expr("Rule"), vec![Expr::string(key), value]))
            .collect(),
    )
}

fn symbol_value(name: &str) -> Expr {
    system_expr(name)
}

fn string_atom(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Atom(Atom::String(value)) => Some(value),
        _ => None,
    }
}
