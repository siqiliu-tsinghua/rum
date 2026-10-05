//! Heap-held depth-first validation. Both descent and ascent stay in this loop.
use super::association::{enter_association, finish_association, next_association};
use super::format::{finish_leaf, push_error, render};
use super::parse::parse_list_schema;
use super::state::{
    ListWork, SchemaAbort, SchemaAwait, SchemaContainer, SchemaDestination, SchemaDriveError,
    SchemaState, SchemaTask,
};
use rum_core::Expr;
use rum_runtime::kernel_api::{EvalError, Session, is_system_symbol_name, system_expr};

pub(super) fn schema_drive(
    session: &mut Session,
    state: &mut SchemaState,
    mut task: SchemaTask,
) -> Result<Expr, SchemaDriveError> {
    loop {
        task = match task {
            SchemaTask::Visit { value, schema } => {
                if is_association_call(&schema) {
                    if let Some(work) = enter_association(state, &value, &schema)? {
                        state.stack.push(SchemaContainer::Association(work));
                        SchemaTask::Next
                    } else {
                        SchemaTask::ChildDone { value }
                    }
                } else if is_list_call(&schema) {
                    let item_schema = parse_list_schema(&schema, &state.path)?;
                    if !is_list_call(&value) {
                        push_error(
                            &mut state.errors,
                            &state.path,
                            format!("expected List, got {}", render(&value)),
                        );
                        SchemaTask::ChildDone { value }
                    } else {
                        let Expr::Call { args: items, .. } = value else {
                            unreachable!()
                        };
                        let output = Vec::with_capacity(items.len());
                        state.stack.push(SchemaContainer::List(ListWork {
                            items,
                            item_schema: item_schema.clone(),
                            next: 0,
                            output,
                            path_len: state.path.rendered.len(),
                            waiting: None,
                        }));
                        SchemaTask::Next
                    }
                } else {
                    // No reference into a work record crosses an evaluation.
                    let result = session.evaluate(Expr::call(
                        system_expr("MatchQ"),
                        vec![value.clone(), schema.clone()],
                    ));
                    match result {
                        Ok(answer) => {
                            finish_leaf(state, &value, &schema, &answer);
                            SchemaTask::ChildDone { value }
                        }
                        Err(error) => return Err(leaf_failure(error, value, schema)),
                    }
                }
            }
            SchemaTask::Default { value, schema } => match session.evaluate(value) {
                Ok(value) => SchemaTask::Visit { value, schema },
                Err(error) => return Err(default_failure(error, schema)),
            },
            SchemaTask::Next => {
                let next = match state.stack.last_mut().expect("an open schema container") {
                    SchemaContainer::List(work) => {
                        if let Some(item) = work.items.get(work.next) {
                            work.next += 1;
                            state.path.append_index(work.next);
                            work.waiting = Some(SchemaDestination::ListItem);
                            Some(SchemaTask::Visit {
                                value: item.clone(),
                                schema: work.item_schema.clone(),
                            })
                        } else {
                            None
                        }
                    }
                    SchemaContainer::Association(work) => {
                        next_association(work, &mut state.path, &mut state.errors)?
                    }
                };
                if let Some(next) = next {
                    next
                } else {
                    let value = match state.stack.pop().expect("the completed schema container") {
                        SchemaContainer::List(work) => Expr::call(system_expr("List"), work.output),
                        SchemaContainer::Association(work) => finish_association(
                            work,
                            state.options,
                            &mut state.path,
                            &mut state.errors,
                        ),
                    };
                    SchemaTask::ChildDone { value }
                }
            }
            SchemaTask::ChildDone { value } => {
                let Some(parent) = state.stack.last_mut() else {
                    return Ok(value);
                };
                let path_len = match parent {
                    SchemaContainer::List(work) => {
                        let Some(SchemaDestination::ListItem) = work.waiting.take() else {
                            unreachable!()
                        };
                        work.output.push(value);
                        work.path_len
                    }
                    SchemaContainer::Association(work) => {
                        match work.waiting.take().expect("one waiting schema child") {
                            SchemaDestination::ExistingEntry { index } => {
                                work.conformed_by_index[index] = Some(value)
                            }
                            SchemaDestination::DefaultEntry { key } => {
                                work.default_entries.push((Expr::string(key), value))
                            }
                            SchemaDestination::ListItem => unreachable!(),
                        }
                        work.path_len
                    }
                };
                state.path.truncate(path_len);
                SchemaTask::Next
            }
        };
    }
}

#[cold]
#[inline(never)]
fn leaf_failure(error: EvalError, value: Expr, schema: Expr) -> SchemaDriveError {
    let pending = if let EvalError::InputRequired { .. } = &error {
        Some(SchemaAwait::Leaf { value, schema })
    } else {
        None
    };
    SchemaDriveError {
        abort: SchemaAbort::Eval(error),
        pending,
    }
}

#[cold]
#[inline(never)]
fn default_failure(error: EvalError, spec: Expr) -> SchemaDriveError {
    let pending = if let EvalError::InputRequired { .. } = &error {
        Some(SchemaAwait::Default { spec })
    } else {
        None
    };
    SchemaDriveError {
        abort: SchemaAbort::Eval(error),
        pending,
    }
}

fn is_association_call(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Association"))
}

fn is_list_call(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "List"))
}
