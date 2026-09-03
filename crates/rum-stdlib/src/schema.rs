use rum_core::SharedArgs;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::collections::HashMap;

use rum_core::Expr;
use rum_runtime::kernel_api::{
    EvalError, Session, association_entries_from_expr, expr_as_string, expr_as_symbol,
    is_system_symbol_name, list_items, render_association, render_input_like_expr, system_expr,
};

const ERROR_LIMIT: usize = 64;
const ASSOCIATION_SCHEMA_INDEX_THRESHOLD: usize = 16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SchemaOptions {
    allow_extra_keys: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SchemaError {
    path: String,
    message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SchemaEntry {
    key: String,
    spec: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EntrySpec {
    Required(Expr),
    Optional { spec: Expr, default: Option<Expr> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SchemaPath {
    rendered: String,
}

#[derive(Debug)]
enum SchemaAbort {
    Malformed { path: String, message: String },
    Eval(EvalError),
}

struct SchemaValidator<'a> {
    session: &'a mut Session,
    options: SchemaOptions,
    errors: Vec<SchemaError>,
}

pub(crate) fn eval_schema_errors(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    if args_slice.len() == 1 {
        return Ok(Expr::call_with_shared_args(head.clone(), args.clone()));
    }
    if args_slice.len() >= 2 {
        let value_result = session.evaluate(args_slice[0].clone());
        let value = rum_runtime::resume_or_suspend!(
            session,
            value_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalSchemaErrorsSuspensionFrame { args }
        },
        )?;
        return eval_schema_errors_with_value(session, value, &args_slice[1..]);
    }

    finish_schema_errors(session, args_slice)
}

fn eval_schema_errors_with_value(
    session: &mut Session,
    value: Expr,
    tail: &[Expr],
) -> Result<Expr, EvalError> {
    let mut args = Vec::with_capacity(tail.len() + 1);
    args.push(value);
    args.extend_from_slice(tail);
    finish_schema_errors(session, &args)
}

fn finish_schema_errors(session: &mut Session, args: &[Expr]) -> Result<Expr, EvalError> {
    let (value, schema, options) = match parse_schema_call("SchemaErrors", args) {
        Ok(parsed) => parsed,
        Err(message) => return Ok(fail_schema(session, "SchemaErrors", message)),
    };

    let mut validator = SchemaValidator::new(session, options);
    if let Err(abort) = validator.conform_value(value, schema, &SchemaPath::root()) {
        return finish_abort(validator.session, "SchemaErrors", abort);
    }

    Ok(render_error_list(capped_errors(&validator.errors)))
}

pub(crate) fn eval_conform_schema(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    if args_slice.len() == 1 {
        return Ok(Expr::call_with_shared_args(head.clone(), args.clone()));
    }
    if args_slice.len() >= 2 {
        let value_result = session.evaluate(args_slice[0].clone());
        let value = rum_runtime::resume_or_suspend!(
            session,
            value_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalConformSchemaSuspensionFrame { args }
        },
        )?;
        return eval_conform_schema_with_value(session, value, &args_slice[1..]);
    }

    finish_conform_schema(session, args_slice)
}

fn eval_conform_schema_with_value(
    session: &mut Session,
    value: Expr,
    tail: &[Expr],
) -> Result<Expr, EvalError> {
    let mut args = Vec::with_capacity(tail.len() + 1);
    args.push(value);
    args.extend_from_slice(tail);
    finish_conform_schema(session, &args)
}

fn finish_conform_schema(session: &mut Session, args: &[Expr]) -> Result<Expr, EvalError> {
    let (value, schema, options) = match parse_schema_call("ConformSchema", args) {
        Ok(parsed) => parsed,
        Err(message) => return Ok(fail_schema(session, "ConformSchema", message)),
    };

    let mut validator = SchemaValidator::new(session, options);
    let conformed = match validator.conform_value(value, schema, &SchemaPath::root()) {
        Ok(conformed) => conformed,
        Err(abort) => return finish_abort(validator.session, "ConformSchema", abort),
    };

    let errors = capped_errors(&validator.errors);
    if errors.is_empty() {
        Ok(conformed)
    } else {
        for error in errors {
            validator
                .session
                .push_message("ConformSchema", "schema", schema_message_text(&error));
        }
        Ok(system_expr("$Failed"))
    }
}

impl<'a> SchemaValidator<'a> {
    fn new(session: &'a mut Session, options: SchemaOptions) -> Self {
        Self {
            session,
            options,
            errors: Vec::new(),
        }
    }

    fn conform_value(
        &mut self,
        value: &Expr,
        schema: &Expr,
        path: &SchemaPath,
    ) -> Result<Expr, SchemaAbort> {
        if is_association_call(schema) {
            return self.conform_association(value, schema, path);
        }

        if is_list_call(schema) {
            return self.conform_list(value, schema, path);
        }

        self.conform_leaf(value, schema, path)
    }

    fn conform_association(
        &mut self,
        value: &Expr,
        schema: &Expr,
        path: &SchemaPath,
    ) -> Result<Expr, SchemaAbort> {
        let schema_entries = parse_association_schema(schema, path)?;
        let Some(value_entries) = association_entries_from_expr(value) else {
            self.push_error(path, format!("expected Association, got {}", render(value)));
            return Ok(value.clone());
        };

        let mut matched = vec![false; value_entries.len()];
        let mut conformed_by_index = vec![None; value_entries.len()];
        let mut default_entries = Vec::new();
        let value_positions = (schema_entries.len() > ASSOCIATION_SCHEMA_INDEX_THRESHOLD
            && value_entries.len() > ASSOCIATION_SCHEMA_INDEX_THRESHOLD)
            .then(|| {
                value_entries
                    .iter()
                    .enumerate()
                    .filter_map(|(index, (key, _))| expr_as_string(key).map(|key| (key, index)))
                    .collect::<HashMap<_, _>>()
            });

        for schema_entry in &schema_entries {
            let child_path = path.key(&schema_entry.key);
            let entry_spec = parse_entry_spec(&schema_entry.spec, &child_path)?;

            let value_index = match &value_positions {
                Some(positions) => positions.get(schema_entry.key.as_str()).copied(),
                None => value_entries
                    .iter()
                    .position(|(key, _)| expr_as_string(key) == Some(schema_entry.key.as_str())),
            };
            if let Some(index) = value_index {
                matched[index] = true;
                let value = &value_entries[index].1;
                let spec = entry_spec.validation_spec();
                let conformed = self.conform_value(value, spec, &child_path)?;
                conformed_by_index[index] = Some(conformed);
                continue;
            }

            match entry_spec {
                EntrySpec::Required(_) => {
                    self.push_error(&child_path, "missing required key");
                }
                EntrySpec::Optional { default, .. } if default.is_none() => {}
                EntrySpec::Optional { spec, default } => {
                    let default = default.expect("default was checked above");
                    let evaluated_default =
                        self.session.evaluate(default).map_err(SchemaAbort::Eval)?;
                    let conformed = self.conform_value(&evaluated_default, &spec, &child_path)?;
                    default_entries.push((Expr::string(schema_entry.key.clone()), conformed));
                }
            }
        }

        for (index, (key, _)) in value_entries.iter().enumerate() {
            if matched[index] {
                continue;
            }

            if !self.options.allow_extra_keys {
                self.push_error(
                    &path.key(&render_key_segment(key)),
                    "extra key is not allowed",
                );
            }
        }
        drop(value_positions);

        let mut output_entries = Vec::new();
        for (index, (key, value)) in value_entries.into_iter().enumerate() {
            if matched[index] {
                let conformed = conformed_by_index[index]
                    .take()
                    .expect("matched schema value should have a conformed value");
                output_entries.push((key, conformed));
            } else if self.options.allow_extra_keys {
                output_entries.push((key, value));
            }
        }
        output_entries.extend(default_entries);

        Ok(render_association(output_entries))
    }

    fn conform_list(
        &mut self,
        value: &Expr,
        schema: &Expr,
        path: &SchemaPath,
    ) -> Result<Expr, SchemaAbort> {
        let item_schema = parse_list_schema(schema, path)?;
        let Some(items) = list_items(value) else {
            self.push_error(path, format!("expected List, got {}", render(value)));
            return Ok(value.clone());
        };

        let mut output_items = Vec::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            output_items.push(self.conform_value(item, item_schema, &path.index(index + 1))?);
        }

        Ok(Expr::call(system_expr("List"), output_items))
    }

    fn conform_leaf(
        &mut self,
        value: &Expr,
        schema: &Expr,
        path: &SchemaPath,
    ) -> Result<Expr, SchemaAbort> {
        if !self.match_leaf(value, schema)? {
            self.push_error(
                path,
                format!(
                    "expected to match {}, got {}",
                    render(schema),
                    render(value)
                ),
            );
        }
        Ok(value.clone())
    }

    fn match_leaf(&mut self, value: &Expr, schema: &Expr) -> Result<bool, SchemaAbort> {
        let result = self
            .session
            .evaluate(Expr::call(
                system_expr("MatchQ"),
                vec![value.clone(), schema.clone()],
            ))
            .map_err(SchemaAbort::Eval)?;
        Ok(is_system_symbol_name(&result, "True"))
    }

    fn push_error(&mut self, path: &SchemaPath, message: impl Into<String>) {
        self.errors.push(SchemaError {
            path: path.rendered.clone(),
            message: message.into(),
        });
    }
}

impl EntrySpec {
    fn validation_spec(&self) -> &Expr {
        match self {
            Self::Required(spec) | Self::Optional { spec, .. } => spec,
        }
    }
}

impl SchemaPath {
    fn root() -> Self {
        Self {
            rendered: String::new(),
        }
    }

    fn key(&self, key: &str) -> Self {
        if self.rendered.is_empty() {
            Self {
                rendered: key.to_string(),
            }
        } else {
            Self {
                rendered: format!("{}.{}", self.rendered, key),
            }
        }
    }

    fn index(&self, index: usize) -> Self {
        Self {
            rendered: format!("{}[{index}]", self.rendered),
        }
    }
}

impl From<EvalError> for SchemaAbort {
    fn from(error: EvalError) -> Self {
        Self::Eval(error)
    }
}

fn parse_schema_call<'a>(
    symbol: &str,
    args: &'a [Expr],
) -> Result<(&'a Expr, &'a Expr, SchemaOptions), String> {
    let [value, schema, option_exprs @ ..] = args else {
        return Err(format!(
            "{symbol} expects a value, a schema, and optional option rules."
        ));
    };

    let options = parse_options(option_exprs)?;
    Ok((value, schema, options))
}

fn parse_options(option_exprs: &[Expr]) -> Result<SchemaOptions, String> {
    let mut options = SchemaOptions::default();
    for option in option_exprs {
        let Some((name, value)) = option_rule(option) else {
            return Err(format!("Invalid schema option {}.", render(option)));
        };
        if name != "AllowExtraKeys" {
            return Err(format!("Unknown schema option {name}."));
        }

        if is_system_symbol_name(value, "True") {
            options.allow_extra_keys = true;
        } else if is_system_symbol_name(value, "False") {
            options.allow_extra_keys = false;
        } else {
            return Err("AllowExtraKeys must be True or False.".to_string());
        }
    }
    Ok(options)
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

fn parse_association_schema(
    schema: &Expr,
    path: &SchemaPath,
) -> Result<Vec<SchemaEntry>, SchemaAbort> {
    let entries = association_entries_from_expr(schema)
        .ok_or_else(|| malformed(path, "association schema entries must be rules"))?;

    entries
        .into_iter()
        .map(|(key, spec)| {
            let Some(key) = expr_as_string(&key) else {
                return Err(malformed(
                    path,
                    format!(
                        "association schema keys must be Strings; got {}",
                        render(&key)
                    ),
                ));
            };
            Ok(SchemaEntry {
                key: key.to_string(),
                spec,
            })
        })
        .collect()
}

fn parse_list_schema<'a>(schema: &'a Expr, path: &SchemaPath) -> Result<&'a Expr, SchemaAbort> {
    let Some(items) = list_items(schema) else {
        return Err(malformed(path, "list schema must use List"));
    };
    match items {
        [item] => Ok(item),
        _ => Err(malformed(
            path,
            format!(
                "list schema must contain exactly one element; got {}",
                items.len()
            ),
        )),
    }
}

fn parse_entry_spec(spec: &Expr, path: &SchemaPath) -> Result<EntrySpec, SchemaAbort> {
    let Expr::Call { head, args } = spec else {
        return Ok(EntrySpec::Required(spec.clone()));
    };
    if !is_system_symbol_name(head, "Optional") {
        return Ok(EntrySpec::Required(spec.clone()));
    }

    match args.as_slice() {
        [inner] => Ok(EntrySpec::Optional {
            spec: inner.clone(),
            default: None,
        }),
        [inner, default] => Ok(EntrySpec::Optional {
            spec: inner.clone(),
            default: Some(default.clone()),
        }),
        _ => Err(malformed(
            path,
            format!(
                "Optional schema must have one or two arguments; got {}",
                args.len()
            ),
        )),
    }
}

fn is_association_call(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Association"))
}

fn is_list_call(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "List"))
}

fn malformed(path: &SchemaPath, message: impl Into<String>) -> SchemaAbort {
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

fn render_key_segment(key: &Expr) -> String {
    expr_as_string(key)
        .map(str::to_string)
        .unwrap_or_else(|| render(key))
}

fn render(expr: &Expr) -> String {
    render_input_like_expr(expr)
}

fn fail_schema(session: &mut Session, symbol: &str, message: impl Into<String>) -> Expr {
    session.push_message(symbol, "schema", message);
    system_expr("$Failed")
}

fn finish_abort(
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

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct EvalSchemaErrorsSuspensionFrame {
    args: rum_core::SharedArgs,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalSchemaErrorsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(value) => (|| eval_schema_errors_with_value(session, value, &args.as_ref()[1..]))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct EvalConformSchemaSuspensionFrame {
    args: rum_core::SharedArgs,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalConformSchemaSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(value) => (|| eval_conform_schema_with_value(session, value, &args.as_ref()[1..]))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
