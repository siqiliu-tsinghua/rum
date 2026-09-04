use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
pub(super) enum DeleteCasesTraversalWork {
    Visit {
        expr: Expr,
        level: usize,
        deleted_before: usize,
    },
    FinishNode {
        expr: Expr,
        level: usize,
        depth: usize,
        subtree_start: usize,
        deleted_before: usize,
    },
    FinishCall {
        children: Vec<(bool, Expr, bool)>,
        child_index: usize,
        level: usize,
        subtree_start: usize,
        deleted_before: usize,
        rebuilt_args: Vec<Expr>,
        max_child_depth: usize,
        rebuilt_head: Option<Expr>,
        head_deleted: bool,
    },
    FinishAssociation {
        children: Vec<(bool, Option<Expr>, Expr, bool)>,
        child_index: usize,
        level: usize,
        subtree_start: usize,
        deleted_before: usize,
        rebuilt_entries: Vec<Expr>,
        max_child_depth: usize,
        rebuilt_head: Option<Expr>,
        head_deleted: bool,
    },
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct DeleteCasesTraversalState {
    work: Vec<DeleteCasesTraversalWork>,
    pattern: Expr,
    traversal: CasesTraversalConfig,
}

impl Session {
    #[cold]
    #[inline(never)]
    pub(super) fn delete_cases_owned_iterative(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        deleted_before: usize,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        self.drive_delete_cases_traversal(
            DeleteCasesTraversalState {
                work: vec![DeleteCasesTraversalWork::Visit {
                    expr,
                    level,
                    deleted_before,
                }],
                pattern,
                traversal,
            },
            None,
        )
    }

    fn drive_delete_cases_traversal(
        &mut self,
        mut state: DeleteCasesTraversalState,
        mut completed: Option<DeleteCasesOutcome>,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        while let Some(work) = state.work.pop() {
            match work {
                DeleteCasesTraversalWork::Visit {
                    expr,
                    level,
                    deleted_before,
                } => self.schedule_delete_cases_visit(&mut state, expr, level, deleted_before),
                DeleteCasesTraversalWork::FinishNode {
                    expr,
                    level,
                    depth,
                    subtree_start,
                    deleted_before,
                } => {
                    debug_assert!(completed.is_none());
                    let finish_result = self.finish_delete_cases_at_current_node(
                        expr,
                        state.pattern.clone(),
                        state.traversal,
                        level,
                        depth,
                        deleted_before,
                    );
                    match finish_result {
                        Ok(mut outcome) => {
                            outcome.deleted_count += deleted_before - subtree_start;
                            completed = Some(outcome);
                        }
                        Err(EvalError::InputRequired { request }) => {
                            let added = deleted_before - subtree_start;
                            crate::resume_or_suspend!(insert self, DriveDeleteCasesTraversalSuspensionFrame { state, added });
                            return Err(EvalError::InputRequired { request });
                        }
                        Err(error) => return Err(error),
                    }
                }
                DeleteCasesTraversalWork::FinishCall {
                    children,
                    child_index,
                    level,
                    subtree_start,
                    deleted_before,
                    rebuilt_args,
                    max_child_depth,
                    rebuilt_head,
                    head_deleted,
                } => {
                    let child_outcome = completed
                        .take()
                        .expect("DeleteCases call finish consumes one completed child");
                    completed = self.finish_delete_cases_traversal_call_child(
                        &mut state,
                        children,
                        child_index,
                        level,
                        subtree_start,
                        deleted_before,
                        rebuilt_args,
                        max_child_depth,
                        rebuilt_head,
                        head_deleted,
                        child_outcome,
                    );
                }
                DeleteCasesTraversalWork::FinishAssociation {
                    children,
                    child_index,
                    level,
                    subtree_start,
                    deleted_before,
                    rebuilt_entries,
                    max_child_depth,
                    rebuilt_head,
                    head_deleted,
                } => {
                    let child_outcome = completed
                        .take()
                        .expect("DeleteCases association finish consumes one completed child");
                    completed = self.finish_delete_cases_traversal_association_child(
                        &mut state,
                        children,
                        child_index,
                        level,
                        subtree_start,
                        deleted_before,
                        rebuilt_entries,
                        max_child_depth,
                        rebuilt_head,
                        head_deleted,
                        child_outcome,
                    );
                }
            }
        }

        Ok(completed.expect("DeleteCases iterative traversal completes its root"))
    }

    pub(super) fn resume_delete_cases_traversal(
        &mut self,
        state: DeleteCasesTraversalState,
        mut outcome: DeleteCasesOutcome,
        added: usize,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        outcome.deleted_count += added;
        self.drive_delete_cases_traversal(state, Some(outcome))
    }

    fn schedule_delete_cases_visit(
        &mut self,
        state: &mut DeleteCasesTraversalState,
        expr: Expr,
        level: usize,
        deleted_before: usize,
    ) {
        match expr {
            Expr::Atom(_) => state.work.push(DeleteCasesTraversalWork::FinishNode {
                expr,
                level,
                depth: 1,
                subtree_start: deleted_before,
                deleted_before,
            }),
            Expr::Call { head, args } if is_system_symbol_name(&head, "Association") => {
                Self::schedule_delete_cases_association(
                    state,
                    head.into_owned(),
                    args.into_vec(),
                    level,
                    deleted_before,
                );
            }
            Expr::Call { head, args } => Self::schedule_delete_cases_call(
                state,
                head.into_owned(),
                args.into_vec(),
                level,
                deleted_before,
            ),
        }
    }

    fn schedule_delete_cases_call(
        state: &mut DeleteCasesTraversalState,
        head: Expr,
        args: Vec<Expr>,
        level: usize,
        deleted_before: usize,
    ) {
        let mut children =
            Vec::with_capacity(args.len() + usize::from(state.traversal.include_heads));
        if state.traversal.include_heads {
            children.push((true, head.clone(), false));
        }
        children.extend(args.into_iter().map(|arg| (false, arg, true)));

        let Some((_, child, _)) = children.first().cloned() else {
            state.work.push(DeleteCasesTraversalWork::FinishNode {
                expr: Expr::call(head, Vec::new()),
                level,
                depth: 2,
                subtree_start: deleted_before,
                deleted_before,
            });
            return;
        };

        let child_level = level + 1;
        state.work.push(DeleteCasesTraversalWork::FinishCall {
            children,
            child_index: 0,
            level,
            subtree_start: deleted_before,
            deleted_before,
            rebuilt_args: Vec::new(),
            max_child_depth: 1,
            rebuilt_head: Some(head),
            head_deleted: false,
        });
        state.work.push(DeleteCasesTraversalWork::Visit {
            expr: child,
            level: child_level,
            deleted_before,
        });
    }

    fn schedule_delete_cases_association(
        state: &mut DeleteCasesTraversalState,
        head: Expr,
        args: Vec<Expr>,
        level: usize,
        deleted_before: usize,
    ) {
        let mut children =
            Vec::with_capacity(args.len() + usize::from(state.traversal.include_heads));
        if state.traversal.include_heads {
            children.push((true, None, head.clone(), false));
        }
        for rule in args {
            match &rule {
                Expr::Call {
                    head: rule_head,
                    args: rule_args,
                } if is_system_symbol_name(rule_head, "Rule") && rule_args.len() == 2 => {
                    children.push((
                        false,
                        Some(rule_args[0].clone()),
                        rule_args[1].clone(),
                        true,
                    ));
                }
                _ => children.push((false, None, rule, true)),
            }
        }

        let Some((_, _, child, _)) = children.first().cloned() else {
            state.work.push(DeleteCasesTraversalWork::FinishNode {
                expr: Expr::call(head, Vec::new()),
                level,
                depth: 2,
                subtree_start: deleted_before,
                deleted_before,
            });
            return;
        };

        let child_level = level + 1;
        state
            .work
            .push(DeleteCasesTraversalWork::FinishAssociation {
                children,
                child_index: 0,
                level,
                subtree_start: deleted_before,
                deleted_before,
                rebuilt_entries: Vec::new(),
                max_child_depth: 1,
                rebuilt_head: Some(head),
                head_deleted: false,
            });
        state.work.push(DeleteCasesTraversalWork::Visit {
            expr: child,
            level: child_level,
            deleted_before,
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_delete_cases_traversal_call_child(
        &mut self,
        state: &mut DeleteCasesTraversalState,
        children: Vec<(bool, Expr, bool)>,
        child_index: usize,
        level: usize,
        subtree_start: usize,
        mut deleted_before: usize,
        mut rebuilt_args: Vec<Expr>,
        mut max_child_depth: usize,
        mut rebuilt_head: Option<Expr>,
        mut head_deleted: bool,
        child_outcome: DeleteCasesOutcome,
    ) -> Option<DeleteCasesOutcome> {
        let (is_head, contributes_depth) = match children.get(child_index) {
            Some((is_head, _, contributes_depth)) => (*is_head, *contributes_depth),
            None => unreachable!("completed outcome belongs to a live call child"),
        };

        deleted_before += child_outcome.deleted_count;
        if contributes_depth {
            max_child_depth = max_child_depth.max(child_outcome.depth);
        }
        if is_head {
            match child_outcome.result {
                DeleteCasesResult::Keep(expr) => rebuilt_head = Some(expr),
                DeleteCasesResult::Delete | DeleteCasesResult::Splice(_) => {
                    head_deleted = true;
                }
            }
        } else {
            match child_outcome.result {
                DeleteCasesResult::Keep(expr) => rebuilt_args.push(expr),
                DeleteCasesResult::Delete => {}
                DeleteCasesResult::Splice(items) => rebuilt_args.extend(items),
            }
        }

        if child_outcome.exhausted {
            rebuilt_args.extend(
                children
                    .iter()
                    .skip(child_index + 1)
                    .filter_map(|(is_head, expr, _)| (!*is_head).then_some(expr.clone())),
            );
            let result = if head_deleted {
                DeleteCasesResult::Splice(rebuilt_args)
            } else {
                DeleteCasesResult::Keep(Expr::call(
                    rebuilt_head.unwrap_or_else(|| system_expr("Sequence")),
                    rebuilt_args,
                ))
            };
            let depth = if head_deleted {
                max_child_depth
            } else {
                max_child_depth.saturating_add(1)
            };
            return Some(DeleteCasesOutcome {
                result,
                depth,
                deleted_count: deleted_before - subtree_start,
                exhausted: true,
            });
        }

        let next_index = child_index + 1;
        if let Some((_, child, _)) = children.get(next_index).cloned() {
            state.work.push(DeleteCasesTraversalWork::FinishCall {
                children,
                child_index: next_index,
                level,
                subtree_start,
                deleted_before,
                rebuilt_args,
                max_child_depth,
                rebuilt_head,
                head_deleted,
            });
            state.work.push(DeleteCasesTraversalWork::Visit {
                expr: child,
                level: level + 1,
                deleted_before,
            });
            return None;
        }

        if head_deleted {
            return Some(DeleteCasesOutcome {
                result: DeleteCasesResult::Splice(rebuilt_args),
                depth: max_child_depth,
                deleted_count: deleted_before - subtree_start,
                exhausted: state.traversal.limit_reached(deleted_before),
            });
        }

        state.work.push(DeleteCasesTraversalWork::FinishNode {
            expr: Expr::call(
                rebuilt_head.unwrap_or_else(|| system_expr("Sequence")),
                rebuilt_args,
            ),
            level,
            depth: max_child_depth.saturating_add(1),
            subtree_start,
            deleted_before,
        });
        None
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_delete_cases_traversal_association_child(
        &mut self,
        state: &mut DeleteCasesTraversalState,
        children: Vec<(bool, Option<Expr>, Expr, bool)>,
        child_index: usize,
        level: usize,
        subtree_start: usize,
        mut deleted_before: usize,
        mut rebuilt_entries: Vec<Expr>,
        mut max_child_depth: usize,
        mut rebuilt_head: Option<Expr>,
        mut head_deleted: bool,
        child_outcome: DeleteCasesOutcome,
    ) -> Option<DeleteCasesOutcome> {
        let (is_head, entry_key, contributes_depth) = match children.get(child_index) {
            Some((is_head, entry_key, _, contributes_depth)) => {
                (*is_head, entry_key.clone(), *contributes_depth)
            }
            None => unreachable!("completed outcome belongs to a live association child"),
        };

        deleted_before += child_outcome.deleted_count;
        if contributes_depth {
            max_child_depth = max_child_depth.max(child_outcome.depth);
        }
        if is_head {
            match child_outcome.result {
                DeleteCasesResult::Keep(expr) => rebuilt_head = Some(expr),
                DeleteCasesResult::Delete | DeleteCasesResult::Splice(_) => {
                    head_deleted = true;
                }
            }
        } else if let Some(key) = entry_key {
            if let DeleteCasesResult::Keep(value) = child_outcome.result {
                rebuilt_entries.push(Expr::call(system_expr("Rule"), vec![key, value]));
            }
        } else {
            match child_outcome.result {
                DeleteCasesResult::Keep(expr) => rebuilt_entries.push(expr),
                DeleteCasesResult::Delete => {}
                DeleteCasesResult::Splice(items) => rebuilt_entries.extend(items),
            }
        }

        if head_deleted {
            return Some(DeleteCasesOutcome {
                result: DeleteCasesResult::Keep(render_association(Vec::new())),
                depth: 2,
                deleted_count: deleted_before - subtree_start,
                exhausted: state.traversal.limit_reached(deleted_before),
            });
        }

        if child_outcome.exhausted {
            rebuilt_entries.extend(children.iter().skip(child_index + 1).filter_map(
                |(is_head, key, expr, _)| {
                    if *is_head {
                        None
                    } else if let Some(key) = key {
                        Some(Expr::call(
                            system_expr("Rule"),
                            vec![key.clone(), expr.clone()],
                        ))
                    } else {
                        Some(expr.clone())
                    }
                },
            ));
            return Some(DeleteCasesOutcome {
                result: DeleteCasesResult::Keep(Expr::call(
                    rebuilt_head.unwrap_or_else(|| system_expr("Association")),
                    rebuilt_entries,
                )),
                depth: max_child_depth.saturating_add(1),
                deleted_count: deleted_before - subtree_start,
                exhausted: true,
            });
        }

        let next_index = child_index + 1;
        if let Some((_, _, child, _)) = children.get(next_index).cloned() {
            state
                .work
                .push(DeleteCasesTraversalWork::FinishAssociation {
                    children,
                    child_index: next_index,
                    level,
                    subtree_start,
                    deleted_before,
                    rebuilt_entries,
                    max_child_depth,
                    rebuilt_head,
                    head_deleted,
                });
            state.work.push(DeleteCasesTraversalWork::Visit {
                expr: child,
                level: level + 1,
                deleted_before,
            });
            return None;
        }

        state.work.push(DeleteCasesTraversalWork::FinishNode {
            expr: Expr::call(
                rebuilt_head.unwrap_or_else(|| system_expr("Association")),
                rebuilt_entries,
            ),
            level,
            depth: max_child_depth.saturating_add(1),
            subtree_start,
            deleted_before,
        });
        None
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<DeleteCasesTraversalState>();
    assert_traces::<DeleteCasesTraversalWork>();
};
