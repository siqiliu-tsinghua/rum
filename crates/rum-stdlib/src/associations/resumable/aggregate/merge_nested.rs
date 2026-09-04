use super::*;

const RECURSIVE_MERGE_NESTED_GROUP_DEPTH: usize = 16;

type SuspendedMergeNestedGroup = (
    Expr,
    FrozenAssociationSequence<(Expr, Vec<Expr>)>,
    usize,
    PersistentAssociationSequence<(Expr, Expr)>,
);
type EnterGroup = SuspendedMergeNestedGroup;
type FinishGroup = (SuspendedMergeNestedGroup, Expr);
pub(super) type MergeNestedWork = Result<EnterGroup, FinishGroup>;

pub(crate) fn eval_merge_nested(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [combiner] => Ok(keyed_operator_form_slot_first(
            "MergeNested",
            combiner.clone(),
        )),
        [associations_expr, combiner] => {
            let Some(items) = list_items(associations_expr) else {
                return Ok(inert_call(head, args));
            };

            let Some(associations) = parse_association_list(items) else {
                return Ok(inert_call(head, args));
            };

            resume_merge_nested_from_index(
                session,
                combiner.clone(),
                group_merge_nested_values(associations),
                0,
                Vec::new(),
                RECURSIVE_MERGE_NESTED_GROUP_DEPTH,
            )
        }
        _ => Ok(inert_call(head, args)),
    }
}

fn resume_merge_nested_from_index(
    session: &mut Session,
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    mut index: usize,
    mut merged: Vec<(Expr, Expr)>,
    remaining_group_depth: usize,
) -> Result<Expr, EvalError> {
    loop {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(merged));
        };

        let value = match evaluate_merge_nested_group(
            session,
            combiner.clone(),
            values,
            remaining_group_depth,
        ) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeMergeNestedFromIndexSuspensionFrame {
                        work: FrozenAssociationSequence::freeze(vec![finish_group_work(
                            combiner, grouped, index, merged, key,
                        )]),
                    },
                );
            }
            Err(error) => return Err(error),
        };

        merged.push((key, value));
        index += 1;
    }
}

fn evaluate_merge_nested_group(
    session: &mut Session,
    combiner: Expr,
    values: Vec<Expr>,
    remaining_group_depth: usize,
) -> Result<Expr, EvalError> {
    if let Some(nested_associations) = nested_association_values(&values) {
        let grouped = group_merge_nested_values(nested_associations);
        if remaining_group_depth == 0 {
            return evaluate_merge_nested_group_iterative(session, combiner, grouped);
        }
        return resume_merge_nested_from_index(
            session,
            combiner,
            grouped,
            0,
            Vec::new(),
            remaining_group_depth - 1,
        );
    }

    session.evaluate(Expr::call(
        combiner,
        vec![Expr::call(system_expr("List"), values)],
    ))
}

#[cold]
#[inline(never)]
fn evaluate_merge_nested_group_iterative(
    session: &mut Session,
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
) -> Result<Expr, EvalError> {
    resume_merge_nested_work(
        session,
        vec![enter_group_work(freeze_group(
            combiner,
            grouped,
            0,
            Vec::new(),
        ))],
        None,
    )
}

fn resume_merge_nested_work(
    session: &mut Session,
    mut work: Vec<MergeNestedWork>,
    mut completed: Option<Expr>,
) -> Result<Expr, EvalError> {
    while let Some(task) = work.pop() {
        match task {
            Ok(group) => {
                debug_assert!(completed.is_none());
                let (combiner, grouped, index, merged) = group;
                let Some((key, values)) = grouped.get(index).cloned() else {
                    completed = Some(render_association(merged.to_vec()));
                    continue;
                };

                let nested_associations = nested_association_values(&values);
                let child_combiner = combiner.clone();
                work.push(finish_frozen_group_work(
                    (combiner, grouped, index, merged),
                    key,
                ));
                if let Some(nested_associations) = nested_associations {
                    work.push(enter_group_work(freeze_group(
                        child_combiner,
                        group_merge_nested_values(nested_associations),
                        0,
                        Vec::new(),
                    )));
                    continue;
                }

                let value = match session.evaluate(Expr::call(
                    child_combiner,
                    vec![Expr::call(system_expr("List"), values)],
                )) {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        return rum_runtime::resume_or_suspend!(
                            session,
                            Err(EvalError::InputRequired { request }),
                            ok(value) => Ok(value),
                            suspend => ResumeSuspendedMergeNestedFromIndexSuspensionFrame {
                                work: FrozenAssociationSequence::freeze(work),
                            },
                        );
                    }
                    Err(error) => return Err(error),
                };
                completed = Some(value);
            }
            Err((group, key)) => {
                let (combiner, grouped, mut index, mut merged) = group;
                let value = completed.take().expect("MergeNested work remains balanced");
                merged = merged.appended((key, value));
                index += 1;
                work.push(enter_group_work((combiner, grouped, index, merged)));
            }
        }
    }

    Ok(completed.expect("MergeNested work completes with one result"))
}

pub(super) fn resume_suspended_merge_nested_work(
    session: &mut Session,
    work: FrozenAssociationSequence<MergeNestedWork>,
    value: Expr,
) -> Result<Expr, EvalError> {
    resume_merge_nested_work(session, work.as_slice().to_vec(), Some(value))
}

fn finish_group_work(
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    merged: Vec<(Expr, Expr)>,
    key: Expr,
) -> MergeNestedWork {
    finish_frozen_group_work(freeze_group(combiner, grouped, index, merged), key)
}

fn enter_group_work(group: EnterGroup) -> MergeNestedWork {
    Ok(group)
}

fn finish_frozen_group_work(group: SuspendedMergeNestedGroup, key: Expr) -> MergeNestedWork {
    Err((group, key))
}

fn freeze_group(
    combiner: Expr,
    grouped: Vec<(Expr, Vec<Expr>)>,
    index: usize,
    merged: Vec<(Expr, Expr)>,
) -> SuspendedMergeNestedGroup {
    (
        combiner,
        FrozenAssociationSequence::freeze(grouped),
        index,
        PersistentAssociationSequence::freeze(merged),
    )
}

fn parse_association_list(items: &[Expr]) -> Option<Vec<Vec<(Expr, Expr)>>> {
    items.iter().map(association_entries_from_expr).collect()
}

fn group_merge_nested_values(associations: Vec<Vec<(Expr, Expr)>>) -> Vec<(Expr, Vec<Expr>)> {
    IndexedGroups::collect(associations.into_iter().flatten()).into_entries()
}

fn nested_association_values(values: &[Expr]) -> Option<Vec<Vec<(Expr, Expr)>>> {
    values.iter().map(association_entries_from_expr).collect()
}
