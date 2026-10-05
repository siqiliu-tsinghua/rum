//! Freeze only on InputRequired. A resumed frame stays Session-rooted while
//! a cold, flat working copy runs; abandonment evaluates no skipped work.
use super::format::{finish_abort, finish_leaf, finish_validation};
use super::machine::schema_drive;
use super::state::{SchemaAbort, SchemaAwait, SchemaDriveError, SchemaState, SchemaTask};
use rum_core::Expr;
use rum_runtime::kernel_api::{
    AbandonmentContext, EvalError, FrameAbandon, FrameAbandonAction, FrameAction, FrameRootTrace,
    FrameRootVisitor, PureFrameAbandon, Session, SuspensionFrame,
};
use std::sync::Arc;

#[derive(FrameRootTrace, PureFrameAbandon)]
pub(crate) struct SchemaValidationSuspensionFrame {
    pub(crate) state: SchemaState,
    pub(crate) pending: SchemaAwait,
}

#[cold]
#[inline(never)]
pub(super) fn suspend_validation(
    session: &mut Session,
    state: SchemaState,
    error: SchemaDriveError,
) -> Result<Expr, EvalError> {
    let mode = state.mode;
    let SchemaDriveError { abort, pending } = error;
    if let SchemaAbort::Eval(EvalError::InputRequired { .. }) = &abort {
        let pending = pending.expect("schema input edge must retain its completion phase");
        rum_runtime::resume_or_suspend!(insert session, SchemaValidationSuspensionFrame { state, pending });
    }
    finish_abort(session, mode.symbol(), abort)
}

impl SuspensionFrame for SchemaValidationSuspensionFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        match outcome {
            Err(error) => FrameAction::complete(Err(error)),
            Ok(answer) => FrameAction::complete(resume_validation(&self, session, answer)),
        }
    }
}

#[cold]
#[inline(never)]
fn resume_validation(
    frame: &SchemaValidationSuspensionFrame,
    session: &mut Session,
    answer: Expr,
) -> Result<Expr, EvalError> {
    // Each Vec clones iteratively over a fixed-depth type graph. Expr handles
    // share their subtrees; neither the queued owner nor its suffix is stolen.
    let mut state = frame.state.clone();
    let task = match frame.pending.clone() {
        SchemaAwait::Leaf { value, schema } => {
            finish_leaf(&mut state, &value, &schema, &answer);
            SchemaTask::ChildDone { value }
        }
        SchemaAwait::Default { spec } => SchemaTask::Visit {
            value: answer,
            schema: spec,
        },
    };
    match schema_drive(session, &mut state, task) {
        Ok(value) => finish_validation(session, state, value),
        Err(error) => suspend_validation(session, state, error),
    }
}
