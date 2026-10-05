//! One mutable UTF-8 path, error presentation, and shared completion.
use super::state::{SchemaAbort, SchemaError, SchemaMode, SchemaPath, SchemaState};
use rum_core::Expr;
use rum_runtime::kernel_api::{
    EvalError, Session, expr_as_string, is_system_symbol_name, render_association,
    render_input_like_expr, system_expr,
};
use std::fmt::Write;

const ERROR_LIMIT: usize = 64;

impl SchemaPath {
    pub(super) fn root() -> Self {
        Self {
            rendered: String::new(),
        }
    }

    pub(super) fn append_key(&mut self, key: &str) {
        if !self.rendered.is_empty() {
            self.rendered.push('.');
        }
        self.rendered.push_str(key);
    }

    pub(super) fn append_index(&mut self, index: usize) {
        write!(&mut self.rendered, "[{index}]").expect("writing to a String cannot fail");
    }

    pub(super) fn truncate(&mut self, len: usize) {
        self.rendered.truncate(len);
    }
}

pub(super) fn push_error(
    errors: &mut Vec<SchemaError>,
    path: &SchemaPath,
    message: impl Into<String>,
) {
    errors.push(SchemaError {
        path: path.rendered.clone(),
        message: message.into(),
    });
}

pub(super) fn finish_leaf(state: &mut SchemaState, value: &Expr, schema: &Expr, result: &Expr) {
    if !is_system_symbol_name(result, "True") {
        push_error(
            &mut state.errors,
            &state.path,
            format!(
                "expected to match {}, got {}",
                render(schema),
                render(value)
            ),
        );
    }
}

pub(super) fn finish_validation(
    session: &mut Session,
    state: SchemaState,
    value: Expr,
) -> Result<Expr, EvalError> {
    let errors = capped_errors(&state.errors);
    match state.mode {
        SchemaMode::Errors => Ok(render_error_list(errors)),
        SchemaMode::Conform if errors.is_empty() => Ok(value),
        SchemaMode::Conform => {
            for error in errors {
                session.push_message("ConformSchema", "schema", schema_message_text(&error));
            }
            Ok(system_expr("$Failed"))
        }
    }
}

pub(super) fn malformed(path: &SchemaPath, message: impl Into<String>) -> SchemaAbort {
    SchemaAbort::Malformed {
        path: path.rendered.clone(),
        message: message.into(),
    }
}

fn render_error_list(errors: Vec<SchemaError>) -> Expr {
    Expr::call(
        system_expr("List"),
        errors.into_iter().map(render_error).collect(),
    )
}

fn render_error(error: SchemaError) -> Expr {
    render_association(vec![
        (Expr::string("Path"), Expr::string(error.path)),
        (Expr::string("Message"), Expr::string(error.message)),
    ])
}

fn capped_errors(errors: &[SchemaError]) -> Vec<SchemaError> {
    if errors.len() <= ERROR_LIMIT {
        return errors.to_vec();
    }

    let mut capped = errors[..ERROR_LIMIT].to_vec();
    capped.push(SchemaError {
        path: String::new(),
        message: format!("… and {} more errors", errors.len() - ERROR_LIMIT),
    });
    capped
}

fn schema_message_text(error: &SchemaError) -> String {
    if error.path.is_empty() {
        error.message.clone()
    } else {
        format!("{}: {}", error.path, error.message)
    }
}

pub(super) fn render_key_segment(key: &Expr) -> String {
    expr_as_string(key)
        .map(str::to_string)
        .unwrap_or_else(|| render(key))
}

pub(super) fn render(expr: &Expr) -> String {
    render_input_like_expr(expr)
}

pub(super) fn fail_schema(session: &mut Session, symbol: &str, message: impl Into<String>) -> Expr {
    session.push_message(symbol, "schema", message);
    system_expr("$Failed")
}

pub(super) fn finish_abort(
    session: &mut Session,
    symbol: &str,
    abort: SchemaAbort,
) -> Result<Expr, EvalError> {
    match abort {
        SchemaAbort::Malformed { path, message } => Ok(fail_schema(
            session,
            symbol,
            format!("Schema path {}: {}", empty_path_label(&path), message),
        )),
        SchemaAbort::Eval(error) => Err(error),
    }
}

fn empty_path_label(path: &str) -> &str {
    if path.is_empty() { "<root>" } else { path }
}
