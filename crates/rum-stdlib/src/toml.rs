use rum_core::SharedArgs;
use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::{EvalError, Session, system_expr};

mod decode;
mod encode;
mod model;
mod number;
mod temporal;

const MAX_TOML_DEPTH: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
struct TomlParseError {
    offset: usize,
    kind: TomlParseErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TomlParseErrorKind {
    Syntax(String),
    Depth,
}

impl TomlParseError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            kind: TomlParseErrorKind::Syntax(message.into()),
        }
    }

    fn depth(offset: usize) -> Self {
        Self {
            offset,
            kind: TomlParseErrorKind::Depth,
        }
    }

    fn message_tag(&self) -> &'static str {
        match self.kind {
            TomlParseErrorKind::Syntax(_) => "toml",
            TomlParseErrorKind::Depth => "depth",
        }
    }

    fn message(&self) -> String {
        match &self.kind {
            TomlParseErrorKind::Syntax(message) => {
                format!("Invalid TOML at byte offset {}: {message}.", self.offset)
            }
            TomlParseErrorKind::Depth => format!(
                "TOML nesting exceeds the fixed depth limit of {MAX_TOML_DEPTH} at byte offset {}.",
                self.offset
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TomlEncodeError {
    kind: TomlEncodeErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TomlEncodeErrorKind {
    Message(String),
    Depth,
}

impl TomlEncodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            kind: TomlEncodeErrorKind::Message(message.into()),
        }
    }

    fn depth() -> Self {
        Self {
            kind: TomlEncodeErrorKind::Depth,
        }
    }

    fn message_tag(&self) -> &'static str {
        match self.kind {
            TomlEncodeErrorKind::Message(_) => "toml",
            TomlEncodeErrorKind::Depth => "depth",
        }
    }

    fn message(&self) -> String {
        match &self.kind {
            TomlEncodeErrorKind::Message(message) => message.clone(),
            TomlEncodeErrorKind::Depth => {
                format!("TOML nesting exceeds the fixed depth limit of {MAX_TOML_DEPTH}.")
            }
        }
    }
}

pub(crate) fn eval_from_toml(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [source] = args else {
        return Ok(fail_toml(
            session,
            "FromToml",
            "toml",
            "FromToml expects exactly one string argument.",
        ));
    };

    let Expr::Atom(Atom::String(source)) = source else {
        return Ok(fail_toml(
            session,
            "FromToml",
            "toml",
            "FromToml expects a TOML text string.",
        ));
    };

    match decode::from_toml(source) {
        Ok(expr) => Ok(expr),
        Err(error) => Ok(fail_toml(
            session,
            "FromToml",
            error.message_tag(),
            error.message(),
        )),
    }
}

pub(crate) fn eval_to_toml(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(fail_toml(
            session,
            "ToToml",
            "toml",
            "ToToml expects exactly one Association argument.",
        ));
    };

    match encode::to_toml(expr) {
        Ok(output) => Ok(Expr::string(output)),
        Err(error) => Ok(fail_toml(
            session,
            "ToToml",
            error.message_tag(),
            error.message(),
        )),
    }
}

fn fail_toml(
    session: &mut Session,
    symbol: &str,
    tag: &'static str,
    message: impl Into<String>,
) -> Expr {
    session.push_message(symbol, tag, message);
    system_expr("$Failed")
}
