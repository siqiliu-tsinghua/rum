//! Held schema entry points. Validation itself is an iterative, resumable walk.
mod association;
pub(crate) mod continuation;
mod format;
mod machine;
mod parse;
pub(crate) mod state;

use continuation::suspend_validation;
use format::{fail_schema, finish_validation};
use machine::schema_drive;
use parse::parse_schema_call;
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{EvalError, FrameRootTrace, FrameRootVisitor, Session};
use state::{LiveSchemaState as SchemaState, SchemaMode, SchemaPath, SchemaTask};

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

    finish_schema_errors(session, None, args_slice)
}

fn eval_schema_errors_with_value(
    session: &mut Session,
    value: Expr,
    tail: &[Expr],
) -> Result<Expr, EvalError> {
    finish_schema_errors(session, Some(&value), tail)
}

fn finish_schema_errors(
    session: &mut Session,
    value: Option<&Expr>,
    tail: &[Expr],
) -> Result<Expr, EvalError> {
    let (value, schema, options) = match parse_schema_call("SchemaErrors", value, tail) {
        Ok(parsed) => parsed,
        Err(message) => return Ok(fail_schema(session, "SchemaErrors", message)),
    };

    start_validation(session, SchemaMode::Errors, value, schema, options)
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

    finish_conform_schema(session, None, args_slice)
}

fn eval_conform_schema_with_value(
    session: &mut Session,
    value: Expr,
    tail: &[Expr],
) -> Result<Expr, EvalError> {
    finish_conform_schema(session, Some(&value), tail)
}

fn finish_conform_schema(
    session: &mut Session,
    value: Option<&Expr>,
    tail: &[Expr],
) -> Result<Expr, EvalError> {
    let (value, schema, options) = match parse_schema_call("ConformSchema", value, tail) {
        Ok(parsed) => parsed,
        Err(message) => return Ok(fail_schema(session, "ConformSchema", message)),
    };

    start_validation(session, SchemaMode::Conform, value, schema, options)
}

fn start_validation(
    session: &mut Session,
    mode: SchemaMode,
    value: &Expr,
    schema: &Expr,
    options: state::SchemaOptions,
) -> Result<Expr, EvalError> {
    let mut state = SchemaState {
        mode,
        options,
        stack: Vec::new(),
        path: SchemaPath::root(),
        errors: Vec::new(),
    };
    let task = SchemaTask::Visit {
        value: value.clone(),
        schema: schema.clone(),
    };
    match schema_drive(session, &mut state, task) {
        Ok(value) => finish_validation(session, state, value),
        Err(error) => suspend_validation(session, state, error),
    }
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
