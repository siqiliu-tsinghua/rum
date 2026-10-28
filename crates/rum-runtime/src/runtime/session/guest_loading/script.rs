use super::scope::GuestCallerStack;
use super::*;
use rum_syntax::{ScriptForm, parse_script_form_with_source};

/// Syntax and source only: no runtime Expr/Symbol roots. The source allocation
/// and parsed syntax remain raw R11 payloads, not certified retained capacity.
pub(super) struct GuestScriptPlan {
    source: Arc<GuestFileSource>,
    forms: Vec<ScriptForm>,
}
impl FrameRootTrace for GuestScriptPlan {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

#[derive(Clone, rum_derive::FrameRootTrace)]
struct GuestScriptCursor {
    plan: Arc<GuestScriptPlan>,
    form: usize,
    last_value: Expr,
    last_termination: Option<EvalTerminationReason>,
    start: OutputMark,
    completed_end: OutputMark,
    caller_stack: Arc<GuestCallerStack>,
}

/// Owns only this form's ambient scope and the remaining script cursor. File
/// rollback belongs exclusively to the later GuestFileCloseFrame.
#[derive(Clone, rum_derive::FrameRootTrace)]
struct GuestScriptFormFrame {
    cursor: GuestScriptCursor,
    scope: GuestFormScope,
}

impl SuspensionFrame for GuestScriptFormFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let mut cursor = self.cursor.clone();
        let result = session
            .finish_guest_form(&mut cursor, &self.scope, outcome, true)
            .and_then(|()| session.drive_guest_script(cursor));
        FrameAction::complete(result)
    }
}
impl FrameAbandon for GuestScriptFormFrame {
    fn abandon(self: Arc<Self>, context: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        context.restore_guest_form_scope(&self.scope);
        FrameAbandonAction::Discard
    }
}

impl Session {
    pub(super) fn start_guest_script(
        &mut self,
        source: Arc<GuestFileSource>,
    ) -> Result<Expr, EvalError> {
        let forms = split_script_into_forms(&source.text)?;
        let cursor = GuestScriptCursor {
            plan: Arc::new(GuestScriptPlan { source, forms }),
            form: 0,
            last_value: system_expr("Null"),
            last_termination: None,
            start: self.output_mark(),
            completed_end: self.output_mark(),
            caller_stack: Arc::new(GuestCallerStack::capture(self)),
        };
        self.drive_guest_script(cursor)
    }

    fn drive_guest_script(&mut self, mut cursor: GuestScriptCursor) -> Result<Expr, EvalError> {
        while cursor.form < cursor.plan.forms.len() {
            let scope = GuestFormScope::enter(
                self,
                GuestFormIdentity {
                    file_depth: self.load_stack.len(),
                    form: cursor.form,
                },
                cursor.caller_stack.clone(),
            );
            // Guest forms inherit the enclosing request's budget. In particular,
            // this is not eval_input_with_terminated_handling's top-level entry.
            let outcome = self.evaluate_guest_form(&cursor.plan, cursor.form);
            if matches!(outcome, Err(EvalError::InputRequired { .. })) {
                scope.restore(self, false);
                crate::resume_or_suspend!(insert self, GuestScriptFormFrame { cursor, scope });
                return outcome;
            }
            self.finish_guest_form(&mut cursor, &scope, outcome, false)?;
        }
        debug_assert!(!self.messages.has_after(cursor.completed_end));
        debug_assert!(!self.prints.has_after(cursor.completed_end));
        self.last_message_list = self.messages.after_vec(cursor.start);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );
        // Get returns just the last value. A converted per-form cycle does not
        // become the enclosing request's termination reason.
        Ok(cursor.last_value)
    }

    fn evaluate_guest_form(
        &mut self,
        plan: &GuestScriptPlan,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if self.active_accepted_request_is_cancelled() {
            return Err(EvalError::Cancelled);
        }
        let expr = match &plan.forms[index] {
            ScriptForm::Parsed { syntax, .. } => self.lower_syntax_expr(syntax)?,
            ScriptForm::Unparsed { text, source_span } => {
                let source = plan
                    .source
                    .text
                    .get(source_span.start..source_span.end)
                    .expect("guest form span belongs to its retained source");
                let syntax = parse_script_form_with_source(text, source, source_span.start)?;
                self.lower_syntax_expr(&syntax)?
            }
        };
        self.evaluate(expr)
    }

    fn finish_guest_form(
        &mut self,
        cursor: &mut GuestScriptCursor,
        scope: &GuestFormScope,
        outcome: Result<Expr, EvalError>,
        resumed: bool,
    ) -> Result<(), EvalError> {
        debug_assert!(!matches!(outcome, Err(EvalError::InputRequired { .. })));
        let result = self.finalize_cycle_termination_value(outcome, scope.mark);
        if let Err(error) = &result {
            self.emit_non_loader_eval_message(scope.mark, error);
            self.emit_top_level_control_flow_messages(error);
        }
        self.last_message_list = self.messages.after_vec(scope.mark);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );
        scope.restore(self, resumed);
        let (value, termination) = result?;
        cursor.last_value = value;
        cursor.last_termination = termination;
        cursor.form += 1;
        cursor.completed_end = self.output_mark();
        Ok(())
    }
}
