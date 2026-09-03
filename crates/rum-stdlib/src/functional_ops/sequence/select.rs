mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum SelectLimit {
    All,
    Count(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
enum SelectSource {
    Call { head: Expr, items: Vec<Expr> },
    Association(Vec<(Expr, Expr)>),
}

#[derive(Clone, FrameRootTrace)]
enum SuspendedSelectSource {
    Call {
        head: Expr,
        items: FrozenFunctionalSequence<Expr>,
    },
    Association(FrozenFunctionalSequence<(Expr, Expr)>),
}

impl SuspendedSelectSource {
    fn freeze(source: SelectSource) -> Self {
        match source {
            SelectSource::Call { head, items } => Self::Call {
                head,
                items: FrozenFunctionalSequence::freeze(items),
            },
            SelectSource::Association(entries) => {
                Self::Association(FrozenFunctionalSequence::freeze(entries))
            }
        }
    }
}

impl SelectLimit {
    fn reached(self, count: usize) -> bool {
        match self {
            Self::All => false,
            Self::Count(limit) => count >= limit,
        }
    }
}

pub(crate) fn eval_select(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [criterion] => Ok(unary_operator_form_slot_first("Select", criterion)),
        [criterion, limit_expr] if sequence_like_from_expr(criterion).is_none() => {
            let Some(limit) = parse_select_limit(limit_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "Select",
                    args,
                    "expected a sequence expression or a non-negative integer or Infinity selection limit",
                ));
            };
            Ok(select_operator_form(criterion.clone(), limit))
        }
        [expr, criterion] => {
            let Some(source) = sequence_like_from_expr(expr) else {
                return Ok(Expr::call(system_expr("Select"), args.to_vec()));
            };

            resume_select_from_index(
                session,
                source,
                criterion.clone(),
                SelectLimit::All,
                0,
                Vec::new(),
            )
        }
        [expr, criterion, limit_expr] => {
            let Some(source) = sequence_like_from_expr(expr) else {
                return Ok(Expr::call(system_expr("Select"), args.to_vec()));
            };
            let Some(limit) = parse_select_limit(limit_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "Select",
                    args,
                    "expected a non-negative integer or Infinity selection limit",
                ));
            };

            resume_select_from_index(session, source, criterion.clone(), limit, 0, Vec::new())
        }
        _ => Ok(session.invalid_call_with_arg_range("Select", args, 1, 3)),
    }
}

pub(crate) fn eval_select_first(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [criterion] => Ok(unary_operator_form_slot_first("SelectFirst", criterion)),
        [expr, criterion] => {
            let Some(source) = sequence_like_from_expr(expr) else {
                return Ok(Expr::call(system_expr("SelectFirst"), args.to_vec()));
            };

            resume_select_first_from_index(
                session,
                source,
                criterion.clone(),
                missing_not_found_expr(),
                0,
            )
        }
        [expr, criterion, default] => {
            let Some(source) = sequence_like_from_expr(expr) else {
                return Ok(Expr::call(system_expr("SelectFirst"), args.to_vec()));
            };

            resume_select_first_from_index(session, source, criterion.clone(), default.clone(), 0)
        }
        _ => Ok(session.invalid_call_with_arg_range("SelectFirst", args, 1, 3)),
    }
}

fn resume_select_from_index(
    session: &mut Session,
    source: SelectSource,
    criterion: Expr,
    limit: SelectLimit,
    mut index: usize,
    mut selected: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if limit.reached(selected.len()) || index >= source_len(&source) {
            return Ok(render_select_result(source, selected));
        }
        if index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }

        let Some(item) = source_item_value(&source, index) else {
            return Ok(render_select_result(source, selected));
        };
        let call = Expr::call(criterion.clone(), vec![item]);
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let source = SuspendedSelectSource::freeze(source);
                                        let selected = PersistentFunctionalSequence::freeze(selected);
                ResumeSelectFromIndexSuspensionFrame { source, criterion, limit, index, selected }
                },
                );
            }
            Err(error) => return Err(error),
        };

        push_selected_value_if_true(session, &source, index, &mut selected, value)?;
        index += 1;
    }
}

fn resume_suspended_select_from_index(
    session: &mut Session,
    source: SuspendedSelectSource,
    criterion: Expr,
    limit: SelectLimit,
    mut index: usize,
    mut selected: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        if limit.reached(selected.len()) || index >= suspended_source_len(&source) {
            return Ok(render_suspended_select_result(source, selected));
        }
        if index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }

        let Some(item) = suspended_source_item_value(&source, index) else {
            return Ok(render_suspended_select_result(source, selected));
        };
        let call = Expr::call(criterion.clone(), vec![item]);
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedSelectFromIndexSuspensionFrame { source, criterion, limit, index, selected },
                );
            }
            Err(error) => return Err(error),
        };

        selected =
            append_suspended_selected_value_if_true(session, &source, index, selected, value)?;
        index += 1;
    }
}

fn resume_suspended_select_with_value(
    session: &mut Session,
    source: SuspendedSelectSource,
    criterion: Expr,
    limit: SelectLimit,
    index: usize,
    selected: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let selected =
        append_suspended_selected_value_if_true(session, &source, index, selected, value)?;
    resume_suspended_select_from_index(session, source, criterion, limit, index + 1, selected)
}

fn push_selected_value_if_true(
    session: &mut Session,
    source: &SelectSource,
    index: usize,
    selected: &mut Vec<Expr>,
    value: Expr,
) -> Result<(), EvalError> {
    if truthiness_symbol(&value) == Some(true) {
        let next_len = selected.len() + 1;
        session.enforce_active_memory_limit_for_projected_bytes(
            generated_expr_nodes_memory_proxy_bytes(next_len + 1),
        )?;
        session.enforce_generated_element_limit_for_count(next_len)?;
        selected.push(source_item_expr(source, index));
    }

    Ok(())
}

fn append_suspended_selected_value_if_true(
    session: &mut Session,
    source: &SuspendedSelectSource,
    index: usize,
    selected: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<PersistentFunctionalSequence<Expr>, EvalError> {
    if truthiness_symbol(&value) != Some(true) {
        return Ok(selected);
    }

    let next_len = selected.len() + 1;
    session.enforce_active_memory_limit_for_projected_bytes(
        generated_expr_nodes_memory_proxy_bytes(next_len + 1),
    )?;
    session.enforce_generated_element_limit_for_count(next_len)?;
    Ok(selected.appended(suspended_source_item_expr(source, index)))
}

fn resume_select_first_from_index(
    session: &mut Session,
    source: SelectSource,
    criterion: Expr,
    default: Expr,
    mut index: usize,
) -> Result<Expr, EvalError> {
    loop {
        if index >= source_len(&source) {
            return Ok(default);
        }
        if index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }

        let Some(item) = source_item_value(&source, index) else {
            return Ok(default);
        };
        let call = Expr::call(criterion.clone(), vec![item]);
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let source = SuspendedSelectSource::freeze(source);
                ResumeSelectFirstFromIndexSuspensionFrame { source, criterion, default, index }
                },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&value) == Some(true) {
            return Ok(source_item_value(&source, index).unwrap_or(default));
        }
        index += 1;
    }
}

fn resume_suspended_select_first_from_index(
    session: &mut Session,
    source: SuspendedSelectSource,
    criterion: Expr,
    default: Expr,
    mut index: usize,
) -> Result<Expr, EvalError> {
    loop {
        if index >= suspended_source_len(&source) {
            return Ok(default);
        }
        if index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }

        let Some(item) = suspended_source_item_value(&source, index) else {
            return Ok(default);
        };
        let call = Expr::call(criterion.clone(), vec![item]);
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedSelectFirstFromIndexSuspensionFrame { source, criterion, default, index },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&value) == Some(true) {
            return Ok(suspended_source_item_value(&source, index).unwrap_or(default));
        }
        index += 1;
    }
}

fn resume_suspended_select_first_with_value(
    session: &mut Session,
    source: SuspendedSelectSource,
    criterion: Expr,
    default: Expr,
    index: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&value) == Some(true) {
        return Ok(suspended_source_item_value(&source, index).unwrap_or(default));
    }

    resume_suspended_select_first_from_index(session, source, criterion, default, index + 1)
}

fn parse_select_limit(expr: &Expr) -> Option<SelectLimit> {
    if is_system_symbol_name(expr, "Infinity") {
        return Some(SelectLimit::All);
    }

    expr_as_non_negative_usize(expr).map(SelectLimit::Count)
}

fn select_operator_form(criterion: Expr, limit: SelectLimit) -> Expr {
    let limit_expr = match limit {
        SelectLimit::All => system_expr("Infinity"),
        SelectLimit::Count(limit) => Expr::integer(BigInt::from(limit)),
    };
    function_body_operator(Expr::call(
        system_expr("Select"),
        vec![slot_expr(), criterion, limit_expr],
    ))
}

fn sequence_like_from_expr(expr: &Expr) -> Option<SelectSource> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(SelectSource::Association(entries));
    }

    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    Some(SelectSource::Call {
        head: head.as_ref().clone(),
        items: args.to_vec(),
    })
}

fn source_len(source: &SelectSource) -> usize {
    match source {
        SelectSource::Call { items, .. } => items.len(),
        SelectSource::Association(entries) => entries.len(),
    }
}

fn source_item_value(source: &SelectSource, index: usize) -> Option<Expr> {
    match source {
        SelectSource::Call { items, .. } => items.get(index).cloned(),
        SelectSource::Association(entries) => entries.get(index).map(|(_, value)| value.clone()),
    }
}

fn source_item_expr(source: &SelectSource, index: usize) -> Expr {
    match source {
        SelectSource::Call { items, .. } => items[index].clone(),
        SelectSource::Association(entries) => {
            let (key, value) = &entries[index];
            Expr::call(system_expr("Rule"), vec![key.clone(), value.clone()])
        }
    }
}

fn suspended_source_len(source: &SuspendedSelectSource) -> usize {
    match source {
        SuspendedSelectSource::Call { items, .. } => items.len(),
        SuspendedSelectSource::Association(entries) => entries.len(),
    }
}

fn suspended_source_item_value(source: &SuspendedSelectSource, index: usize) -> Option<Expr> {
    match source {
        SuspendedSelectSource::Call { items, .. } => items.get(index).cloned(),
        SuspendedSelectSource::Association(entries) => {
            entries.get(index).map(|(_, value)| value.clone())
        }
    }
}

fn suspended_source_item_expr(source: &SuspendedSelectSource, index: usize) -> Expr {
    match source {
        SuspendedSelectSource::Call { items, .. } => items
            .get(index)
            .expect("selected call item should exist")
            .clone(),
        SuspendedSelectSource::Association(entries) => {
            let (key, value) = entries
                .get(index)
                .expect("selected association entry should exist");
            Expr::call(system_expr("Rule"), vec![key.clone(), value.clone()])
        }
    }
}

fn render_select_result(source: SelectSource, selected: Vec<Expr>) -> Expr {
    match source {
        SelectSource::Call { head, .. } => Expr::call(head, selected),
        SelectSource::Association(_) => render_association(
            selected
                .into_iter()
                .filter_map(|expr| {
                    let Expr::Call { head, args, .. } = expr else {
                        return None;
                    };
                    if !is_system_symbol_name(&head, "Rule") || args.len() != 2 {
                        return None;
                    }
                    Some((args[0].clone(), args[1].clone()))
                })
                .collect(),
        ),
    }
}

fn render_suspended_select_result(
    source: SuspendedSelectSource,
    selected: PersistentFunctionalSequence<Expr>,
) -> Expr {
    let selected = selected.to_vec();
    match source {
        SuspendedSelectSource::Call { head, .. } => Expr::call(head, selected),
        SuspendedSelectSource::Association(_) => render_association(
            selected
                .into_iter()
                .filter_map(|expr| {
                    let Expr::Call { head, args, .. } = expr else {
                        return None;
                    };
                    if !is_system_symbol_name(&head, "Rule") || args.len() != 2 {
                        return None;
                    }
                    Some((args[0].clone(), args[1].clone()))
                })
                .collect(),
        ),
    }
}

fn missing_not_found_expr() -> Expr {
    Expr::call(system_expr("Missing"), vec![Expr::string("NotFound")])
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SelectSource>();
    assert_traces::<SuspendedSelectSource>();
};
