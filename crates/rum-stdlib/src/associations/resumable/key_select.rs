use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

pub(crate) fn eval_key_select(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [criterion] => Ok(keyed_operator_form_slot_first(
            "KeySelect",
            criterion.clone(),
        )),
        [expr, criterion] => {
            let Some(entries) = keyed_entry_forms_from_expr(expr) else {
                return Ok(session.invalid_call_with_message(
                    "KeySelect",
                    args,
                    "invru",
                    invalid_association_or_rule_list_message(expr),
                ));
            };

            resume_key_select_from_index(session, entries, criterion.clone(), 0, Vec::new())
        }
        _ => Ok(session.invalid_call_with_message(
            "KeySelect",
            args,
            "argt",
            one_or_two_argument_count_message("KeySelect", args.len()),
        )),
    }
}

fn resume_key_select_from_index(
    session: &mut Session,
    entries: Vec<AssociationEntryForm>,
    criterion: Expr,
    mut index: usize,
    mut selected: Vec<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return Ok(render_association_entry_forms_preserving_heads(selected));
        };

        let call = Expr::call(criterion.clone(), vec![entry.key.clone()]);
        let criterion_result = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let entries = FrozenAssociationSequence::freeze(entries);
                                        let selected = PersistentAssociationSequence::freeze(selected);
                ResumeKeySelectFromIndexSuspensionFrame { entries, criterion, index, selected, entry }
                },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&criterion_result) == Some(true) {
            selected.push(entry);
        }
        index += 1;
    }
}

fn resume_suspended_key_select_from_index(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    criterion: Expr,
    mut index: usize,
    mut selected: PersistentAssociationSequence<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return Ok(render_association_entry_forms_preserving_heads(
                selected.to_vec(),
            ));
        };

        let call = Expr::call(criterion.clone(), vec![entry.key.clone()]);
        let criterion_result = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedKeySelectFromIndexSuspensionFrame { entries, criterion, index, selected, entry },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&criterion_result) == Some(true) {
            selected = selected.appended(entry);
        }
        index += 1;
    }
}

fn resume_suspended_key_select_with_value(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    criterion: Expr,
    index: usize,
    selected: PersistentAssociationSequence<AssociationEntryForm>,
    entry: AssociationEntryForm,
    criterion_value: Expr,
) -> Result<Expr, EvalError> {
    let selected = if truthiness_symbol(&criterion_value) == Some(true) {
        selected.appended(entry)
    } else {
        selected
    };

    resume_suspended_key_select_from_index(session, entries, criterion, index + 1, selected)
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeKeySelectFromIndexSuspensionFrame {
    entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    criterion: rum_core::Expr,
    index: usize,
    selected: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeKeySelectFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut selected = self.selected.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(outcome) => (|| {
                resume_suspended_key_select_with_value(
                    session, entries, criterion, index, selected, entry, outcome,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedKeySelectFromIndexSuspensionFrame {
    entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    criterion: rum_core::Expr,
    index: usize,
    selected: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedKeySelectFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut selected = self.selected.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(outcome) => (|| {
                resume_suspended_key_select_with_value(
                    session, entries, criterion, index, selected, entry, outcome,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
