mod resume_frames;
use resume_frames::*;

mod traversal;
use traversal::*;

use rum_core::SharedArgs;

use super::grouping::IndexedGroups;
use super::*;

pub(crate) fn eval_key_group_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [function] => Ok(keyed_operator_form_slot_first(
            "KeyGroupBy",
            function.clone(),
        )),
        [expr, function] => {
            let Some(entries) = association_entry_forms_from_expr(expr) else {
                return Ok(inert_call(head, args));
            };

            resume_key_group_by_from_index(
                session,
                function.clone(),
                None,
                entries,
                0,
                IndexedGroups::from_entries(Vec::new()),
            )
        }
        [expr, function, reducer] => {
            let Some(entries) = association_entry_forms_from_expr(expr) else {
                return Ok(inert_call(head, args));
            };

            resume_key_group_by_from_index(
                session,
                function.clone(),
                Some(reducer.clone()),
                entries,
                0,
                IndexedGroups::from_entries(Vec::new()),
            )
        }
        _ => Ok(inert_call(head, args)),
    }
}

pub(crate) fn eval_association_key_deflatten(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(inert_call(head, args));
    };

    let Some(entries) = association_entry_forms_from_expr(expr) else {
        return Ok(inert_call(head, args));
    };

    let Some(deflattened) = association_key_deflatten_entries(entries) else {
        return Ok(inert_call(head, args));
    };

    Ok(deflattened)
}

pub(crate) fn eval_association_key_flatten(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(inert_call(head, args));
    };

    let Some(entries) = association_entry_forms_from_expr(expr) else {
        return Ok(inert_call(head, args));
    };

    Ok(render_association_entry_forms_preserving_heads(
        flatten_association_entries(Vec::new(), entries),
    ))
}

fn resume_key_group_by_from_index(
    session: &mut Session,
    function: Expr,
    reducer: Option<Expr>,
    entries: Vec<AssociationEntryForm>,
    mut index: usize,
    mut grouped: IndexedGroups<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return finish_key_group_by_grouped(session, reducer, grouped.into_entries());
        };

        let group_key = match session
            .evaluate(Expr::call(function.clone(), vec![entry.key.clone()]))
        {
            Ok(group_key) => group_key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let entries = FrozenAssociationSequence::freeze(entries);
                                            let grouped = PersistentAssociationGroups::freeze(grouped);
                ResumeKeyGroupByFromIndexSuspensionFrame { function, reducer, entries, index, grouped, entry }
                },
                );
            }
            Err(error) => return Err(error),
        };

        grouped.push(group_key, entry);
        index += 1;
    }
}

fn resume_suspended_key_group_by_from_index(
    session: &mut Session,
    function: Expr,
    reducer: Option<Expr>,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    mut index: usize,
    mut grouped: PersistentAssociationGroups<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(entry) = entries.get(index).cloned() else {
            return finish_key_group_by_grouped(session, reducer, grouped.to_entries());
        };

        let group_key = match session
            .evaluate(Expr::call(function.clone(), vec![entry.key.clone()]))
        {
            Ok(group_key) => group_key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedKeyGroupByFromIndexSuspensionFrame { function, reducer, entries, index, grouped, entry },
                );
            }
            Err(error) => return Err(error),
        };

        grouped = grouped.appended(group_key, entry);
        index += 1;
    }
}

fn resume_suspended_key_group_by_with_key(
    session: &mut Session,
    function: Expr,
    reducer: Option<Expr>,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    index: usize,
    grouped: PersistentAssociationGroups<AssociationEntryForm>,
    entry: AssociationEntryForm,
    group_key: Expr,
) -> Result<Expr, EvalError> {
    let grouped = grouped.appended(group_key, entry);
    resume_suspended_key_group_by_from_index(
        session,
        function,
        reducer,
        entries,
        index + 1,
        grouped,
    )
}

fn finish_key_group_by_grouped(
    session: &mut Session,
    reducer: Option<Expr>,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
) -> Result<Expr, EvalError> {
    match reducer {
        None => Ok(render_association(key_group_by_groups(grouped))),
        Some(reducer) => {
            resume_key_group_by_reduce_from_index(session, reducer, grouped, 0, Vec::new())
        }
    }
}

fn resume_key_group_by_reduce_from_index(
    session: &mut Session,
    reducer: Expr,
    grouped: Vec<(Expr, Vec<AssociationEntryForm>)>,
    mut index: usize,
    mut reduced: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((group_key, entries)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced));
        };

        let group = render_association_entry_forms_preserving_heads(entries);
        let value = match session.evaluate(Expr::call(reducer.clone(), vec![group])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let grouped = FrozenAssociationSequence::freeze(grouped);
                                        let reduced = PersistentAssociationSequence::freeze(reduced);
                ResumeKeyGroupByReduceFromIndexSuspensionFrame { reducer, grouped, index, reduced, group_key }
                },
                );
            }
            Err(error) => return Err(error),
        };

        reduced.push((group_key, value));
        index += 1;
    }
}

fn resume_suspended_key_group_by_reduce_from_index(
    session: &mut Session,
    reducer: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<AssociationEntryForm>)>,
    mut index: usize,
    mut reduced: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some((group_key, entries)) = grouped.get(index).cloned() else {
            return Ok(render_association(reduced.to_vec()));
        };

        let group = render_association_entry_forms_preserving_heads(entries);
        let value = match session.evaluate(Expr::call(reducer.clone(), vec![group])) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedKeyGroupByReduceFromIndexSuspensionFrame { reducer, grouped, index, reduced, group_key },
                );
            }
            Err(error) => return Err(error),
        };

        reduced = reduced.appended((group_key, value));
        index += 1;
    }
}

fn resume_suspended_key_group_by_reduce_with_value(
    session: &mut Session,
    reducer: Expr,
    grouped: FrozenAssociationSequence<(Expr, Vec<AssociationEntryForm>)>,
    index: usize,
    reduced: PersistentAssociationSequence<(Expr, Expr)>,
    group_key: Expr,
    value: Expr,
) -> Result<Expr, EvalError> {
    let reduced = reduced.appended((group_key, value));
    resume_suspended_key_group_by_reduce_from_index(session, reducer, grouped, index + 1, reduced)
}

fn key_group_by_groups(grouped: Vec<(Expr, Vec<AssociationEntryForm>)>) -> Vec<(Expr, Expr)> {
    grouped
        .into_iter()
        .map(|(group_key, entries)| {
            (
                group_key,
                render_association_entry_forms_preserving_heads(entries),
            )
        })
        .collect()
}

fn inert_call(head: &Expr, args: &[Expr]) -> Expr {
    Expr::call(head.clone(), args.to_vec())
}
