use super::*;
use crate::structure_ops::{
    ComparatorResultPolicy, MergeCompletion, MergeSortItem, sort_with_comparator,
};
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeySortOrderingDecision {
    InsertBefore,
    KeepScanning,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct KeySortByEntry {
    entry: AssociationEntryForm,
    sort_key: Expr,
}

#[derive(FrameRootTrace)]
struct KeySortCompletion;

impl MergeCompletion<AssociationEntryForm> for KeySortCompletion {
    fn complete(self, entries: Vec<AssociationEntryForm>) -> Expr {
        render_association_entry_forms_preserving_heads(entries)
    }
}

#[derive(FrameRootTrace)]
struct KeySortByCompletion;

impl MergeCompletion<KeySortByEntry> for KeySortByCompletion {
    fn complete(self, entries: Vec<KeySortByEntry>) -> Expr {
        render_association_entry_forms_preserving_heads(
            entries.into_iter().map(|entry| entry.entry).collect(),
        )
    }
}

impl MergeSortItem for AssociationEntryForm {
    fn ordering_value(&self) -> &Expr {
        &self.key
    }
}

impl MergeSortItem for KeySortByEntry {
    fn ordering_value(&self) -> &Expr {
        &self.sort_key
    }
}

struct KeySortByOrderingState {
    function: Expr,
    ordering: Expr,
    remaining: std::vec::IntoIter<AssociationEntryForm>,
    decorated: Vec<KeySortByEntry>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedKeySortByState {
    function: Expr,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    index: usize,
    decorated: PersistentAssociationSequence<(AssociationEntryForm, Expr)>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedKeySortByOrderingState {
    function: Expr,
    ordering: Expr,
    remaining: FrozenAssociationSequence<AssociationEntryForm>,
    index: usize,
    decorated: PersistentAssociationSequence<KeySortByEntry>,
}

pub(crate) fn eval_key_sort_by(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [function] => Ok(keyed_operator_form_slot_first(
            "KeySortBy",
            function.clone(),
        )),
        [expr, function] => {
            let Some(entries) = keyed_entry_forms_from_expr(expr) else {
                return Ok(session.invalid_call_with_message(
                    "KeySortBy",
                    args,
                    "invrl",
                    invalid_association_or_rule_list_message(expr),
                ));
            };

            let decorated = Vec::with_capacity(entries.len());
            resume_key_sort_by_from_index(session, function.clone(), entries, 0, decorated)
        }
        [expr, function, ordering] => {
            let Some(entries) = keyed_entry_forms_from_expr(expr) else {
                return Ok(session.invalid_call_with_message(
                    "KeySortBy",
                    args,
                    "invrl",
                    invalid_association_or_rule_list_message(expr),
                ));
            };

            let decorated = Vec::with_capacity(entries.len());
            resume_key_sort_by_ordering(
                session,
                KeySortByOrderingState {
                    function: function.clone(),
                    ordering: ordering.clone(),
                    remaining: entries.into_iter(),
                    decorated,
                },
            )
        }
        _ => Ok(session.invalid_call_with_message(
            "KeySortBy",
            args,
            "argt",
            one_to_three_argument_count_message("KeySortBy", args.len()),
        )),
    }
}

pub(crate) fn sort_key_entries_with_ordering(
    session: &mut Session,
    ordering: Expr,
    entries: Vec<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    sort_with_comparator(
        session,
        entries,
        ordering,
        ComparatorResultPolicy::Custom(key_sort_ordering_precedes),
        KeySortCompletion,
    )
}

fn resume_key_sort_by_from_index(
    session: &mut Session,
    function: Expr,
    entries: Vec<AssociationEntryForm>,
    mut index: usize,
    mut decorated: Vec<(AssociationEntryForm, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            decorated.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.1, &rhs.1));
            return Ok(render_association_entry_forms_preserving_heads(
                decorated.into_iter().map(|(entry, _)| entry).collect(),
            ));
        };

        let sort_key = match session.evaluate(Expr::call(function.clone(), vec![entry.key.clone()]))
        {
            Ok(sort_key) => sort_key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeKeySortByFromIndexSuspensionFrame {
                        state: SuspendedKeySortByState {
                            function,
                            entries: FrozenAssociationSequence::freeze(entries),
                            index,
                            decorated: PersistentAssociationSequence::freeze(decorated),
                        },
                        entry,
                    },
                );
            }
            Err(error) => return Err(error),
        };

        decorated.push((entry, sort_key));
        index += 1;
    }
}

fn resume_suspended_key_sort_by_with_value(
    session: &mut Session,
    mut state: SuspendedKeySortByState,
    entry: AssociationEntryForm,
    sort_key: Expr,
) -> Result<Expr, EvalError> {
    state.decorated = state.decorated.appended((entry, sort_key));
    state.index += 1;
    resume_suspended_key_sort_by_from_index(session, state)
}

fn resume_suspended_key_sort_by_from_index(
    session: &mut Session,
    mut state: SuspendedKeySortByState,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = state.entries.get(state.index).cloned() else {
            let mut decorated = state.decorated.to_vec();
            decorated.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.1, &rhs.1));
            return Ok(render_association_entry_forms_preserving_heads(
                decorated.into_iter().map(|(entry, _)| entry).collect(),
            ));
        };
        let sort_key =
            match session.evaluate(Expr::call(state.function.clone(), vec![entry.key.clone()])) {
                Ok(sort_key) => sort_key,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeKeySortByFromIndexSuspensionFrame { state, entry },
                    );
                }
                Err(error) => return Err(error),
            };
        state.decorated = state.decorated.appended((entry, sort_key));
        state.index += 1;
    }
}

fn resume_key_sort_by_ordering(
    session: &mut Session,
    mut state: KeySortByOrderingState,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = state.remaining.next() else {
            return sort_with_comparator(
                session,
                state.decorated,
                state.ordering,
                ComparatorResultPolicy::Custom(key_sort_ordering_precedes),
                KeySortByCompletion,
            );
        };

        let sort_key =
            match session.evaluate(Expr::call(state.function.clone(), vec![entry.key.clone()])) {
                Ok(sort_key) => sort_key,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeKeySortByOrderingSuspensionFrame {
                            state: SuspendedKeySortByOrderingState {
                                function: state.function,
                                ordering: state.ordering,
                                remaining: FrozenAssociationSequence::freeze(
                                    std::iter::once(entry.clone())
                                        .chain(state.remaining)
                                        .collect(),
                                ),
                                index: 0,
                                decorated: PersistentAssociationSequence::freeze(
                                    state.decorated,
                                ),
                            },
                            entry,
                        },
                    );
                }
                Err(error) => return Err(error),
            };

        state.decorated.push(KeySortByEntry { entry, sort_key });
    }
}

fn resume_suspended_key_sort_by_ordering(
    session: &mut Session,
    mut state: SuspendedKeySortByOrderingState,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = state.remaining.get(state.index).cloned() else {
            return sort_with_comparator(
                session,
                state.decorated.to_vec(),
                state.ordering,
                ComparatorResultPolicy::Custom(key_sort_ordering_precedes),
                KeySortByCompletion,
            );
        };
        let sort_key =
            match session.evaluate(Expr::call(state.function.clone(), vec![entry.key.clone()])) {
                Ok(sort_key) => sort_key,
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => ResumeKeySortByOrderingSuspensionFrame { state, entry },
                    );
                }
                Err(error) => return Err(error),
            };
        state.decorated = state.decorated.appended(KeySortByEntry { entry, sort_key });
        state.index += 1;
    }
}

fn key_sort_ordering_precedes(value: &Expr) -> bool {
    key_sort_ordering_decision(value) == KeySortOrderingDecision::InsertBefore
}

fn key_sort_ordering_decision(value: &Expr) -> KeySortOrderingDecision {
    match value {
        Expr::Atom(Atom::Integer(integer)) if integer.as_i64() == Some(1) => {
            KeySortOrderingDecision::InsertBefore
        }
        Expr::Atom(Atom::Integer(integer))
            if integer.as_i64() == Some(0) || integer.as_i64() == Some(-1) =>
        {
            KeySortOrderingDecision::KeepScanning
        }
        _ => match truthiness_symbol(value) {
            Some(true) => KeySortOrderingDecision::InsertBefore,
            Some(false) => KeySortOrderingDecision::KeepScanning,
            None => KeySortOrderingDecision::InsertBefore,
        },
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<KeySortByCompletion>();
    assert_traces::<KeySortByEntry>();
    assert_traces::<SuspendedKeySortByOrderingState>();
    assert_traces::<SuspendedKeySortByState>();
    assert_traces::<KeySortCompletion>();
};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeKeySortByFromIndexSuspensionFrame {
    state: crate::associations::resumable::key_sort::SuspendedKeySortByState,
    entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeKeySortByFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(sort_key) => {
                (|| resume_suspended_key_sort_by_with_value(session, state, entry, sort_key))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeKeySortByOrderingSuspensionFrame {
    state: crate::associations::resumable::key_sort::SuspendedKeySortByOrderingState,
    entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeKeySortByOrderingSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(sort_key) => (|| {
                state.decorated = state.decorated.appended(KeySortByEntry { entry, sort_key });
                state.index += 1;
                resume_suspended_key_sort_by_ordering(session, state)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
