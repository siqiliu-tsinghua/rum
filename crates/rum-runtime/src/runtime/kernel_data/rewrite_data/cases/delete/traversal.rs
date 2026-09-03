use super::*;

impl Session {
    fn finish_delete_cases_at_current_node(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        depth: usize,
        deleted_before: usize,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        if traversal.limit_reached(deleted_before) {
            return Ok(DeleteCasesOutcome {
                result: DeleteCasesResult::Keep(expr),
                depth,
                deleted_count: 0,
                exhausted: true,
            });
        }

        let match_result = if traversal.levels.matches(level, depth) {
            self.match_expr_owned(pattern, expr.clone(), SymbolExprMap::default())
        } else {
            Ok(CompiledMatchOutcome {
                matched: false,
                bindings: SymbolExprMap::default(),
            })
        };

        crate::resume_or_suspend!(
            self,
            match_result,
            ok(outcome) => { (|| {
                    if outcome.matched {
                        Ok(DeleteCasesOutcome {
                            result: DeleteCasesResult::Delete,
                            depth,
                            deleted_count: 1,
                            exhausted: traversal.limit_reached(deleted_before + 1),
                        })
                    } else {
                        Ok(DeleteCasesOutcome {
                            result: DeleteCasesResult::Keep(expr),
                            depth,
                            deleted_count: 0,
                            exhausted: false,
                        })
                    }
                })() },
            suspend => FinishDeleteCasesAtCurrentNodeSuspensionFrame { depth, traversal, deleted_before, expr },
        )
    }

    pub(super) fn delete_cases_owned(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        deleted_before: usize,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        match expr {
            Expr::Atom(_) => self.finish_delete_cases_at_current_node(
                expr,
                pattern,
                traversal,
                level,
                1,
                deleted_before,
            ),
            Expr::Call { head, args } if is_system_symbol_name(&head, "Association") => self
                .delete_cases_association_children(
                    head.into_owned(),
                    args.into_vec(),
                    pattern,
                    traversal,
                    level + 1,
                    deleted_before,
                    Vec::new(),
                    1,
                    false,
                ),
            Expr::Call { head, args } => self.delete_cases_call_children(
                head.into_owned(),
                args.into_vec(),
                pattern,
                traversal,
                level + 1,
                deleted_before,
                Vec::new(),
                1,
                false,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn delete_cases_call_children(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        deleted_before: usize,
        rebuilt_args: Vec<Expr>,
        max_child_depth: usize,
        head_deleted: bool,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        let mut children = Vec::new();
        if traversal.include_heads {
            children.push((true, head.clone(), false));
        }
        children.extend(args.into_iter().map(|arg| (false, arg, true)));

        self.delete_cases_call_children_from_queue(
            children,
            0,
            pattern,
            traversal,
            level,
            deleted_before,
            deleted_before,
            rebuilt_args,
            max_child_depth,
            Some(head),
            head_deleted,
            None,
        )
    }

    /// Walks the child queue one element per loop iteration, keeping the
    /// rebuilt arguments in place instead of recursing per element. `pending`
    /// carries an already-computed child outcome across a suspension resume,
    /// so the merge step runs exactly once per element on both the live and
    /// the resumed paths.
    #[allow(clippy::too_many_arguments)]
    fn delete_cases_call_children_from_queue(
        &mut self,
        children: Vec<(bool, Expr, bool)>,
        mut child_index: usize,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        subtree_start: usize,
        mut deleted_before: usize,
        mut rebuilt_args: Vec<Expr>,
        mut max_child_depth: usize,
        mut rebuilt_head: Option<Expr>,
        mut head_deleted: bool,
        mut pending: Option<DeleteCasesOutcome>,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                let (is_head, contributes_depth) = match children.get(child_index) {
                    Some((is_head, _, contributes_depth)) => (*is_head, *contributes_depth),
                    None => unreachable!("pending outcome belongs to a live child index"),
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
                    let untouched_rest = children
                        .iter()
                        .skip(child_index + 1)
                        .filter_map(|(is_head, expr, _)| (!*is_head).then_some(expr.clone()))
                        .collect::<Vec<_>>();
                    rebuilt_args.extend(untouched_rest);

                    if head_deleted {
                        return Ok(DeleteCasesOutcome {
                            result: DeleteCasesResult::Splice(rebuilt_args),
                            depth: max_child_depth,
                            deleted_count: deleted_before - subtree_start,
                            exhausted: true,
                        });
                    }

                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Keep(Expr::call(
                            rebuilt_head.unwrap_or_else(|| system_expr("Sequence")),
                            rebuilt_args,
                        )),
                        depth: max_child_depth.saturating_add(1),
                        deleted_count: deleted_before - subtree_start,
                        exhausted: true,
                    });
                }

                child_index += 1;
            }

            let Some((_, child, _)) = children.get(child_index).cloned() else {
                if head_deleted {
                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Splice(rebuilt_args),
                        depth: max_child_depth,
                        deleted_count: deleted_before - subtree_start,
                        exhausted: traversal.limit_reached(deleted_before),
                    });
                }

                let depth = max_child_depth.saturating_add(1);
                let expr = Expr::call(
                    rebuilt_head.unwrap_or_else(|| system_expr("Sequence")),
                    rebuilt_args,
                );
                let finish_result = self.finish_delete_cases_at_current_node(
                    expr,
                    pattern,
                    traversal,
                    level - 1,
                    depth,
                    deleted_before,
                );
                return crate::resume_or_suspend!(
                    self,
                    finish_result,
                    ok(mut outcome) => { (|| {
                                    outcome.deleted_count += deleted_before - subtree_start;
                                    Ok(outcome)
                                })() },
                    suspend => DeleteCasesCallChildrenFromQueueSuspensionFrame1 { deleted_before, subtree_start },
                );
            };

            let child_result =
                self.delete_cases_owned(child, pattern.clone(), traversal, level, deleted_before);

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedDeleteCallChildren {
                        children: FrozenRewriteQueue::freeze(children),
                        child_index,
                        pattern,
                        traversal,
                        level,
                        subtree_start,
                        deleted_before,
                        rebuilt_args: PersistentRewriteExprs::freeze(rebuilt_args),
                        max_child_depth,
                        rebuilt_head,
                        head_deleted,
                    };
                    crate::resume_or_suspend!(insert self, DeleteCasesCallChildrenFromQueueSuspensionFrame2 { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn delete_cases_association_children(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        deleted_before: usize,
        rebuilt_entries: Vec<Expr>,
        max_child_depth: usize,
        head_deleted: bool,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        let mut children = Vec::new();
        if traversal.include_heads {
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

        self.delete_cases_association_children_from_queue(
            children,
            0,
            pattern,
            traversal,
            level,
            deleted_before,
            deleted_before,
            rebuilt_entries,
            max_child_depth,
            Some(head),
            head_deleted,
            None,
        )
    }

    /// Iterative association-entry queue walk; `pending` carries an
    /// already-computed entry outcome across a suspension resume, so the merge
    /// step runs exactly once per entry on both the live and the resumed
    /// paths.
    #[allow(clippy::too_many_arguments)]
    fn delete_cases_association_children_from_queue(
        &mut self,
        children: Vec<(bool, Option<Expr>, Expr, bool)>,
        mut child_index: usize,
        pattern: Expr,
        traversal: CasesTraversalConfig,
        level: usize,
        subtree_start: usize,
        mut deleted_before: usize,
        mut rebuilt_entries: Vec<Expr>,
        mut max_child_depth: usize,
        mut rebuilt_head: Option<Expr>,
        mut head_deleted: bool,
        mut pending: Option<DeleteCasesOutcome>,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        loop {
            if let Some(child_outcome) = pending.take() {
                let (is_head, entry_key, contributes_depth) = match children.get(child_index) {
                    Some((is_head, entry_key, _, contributes_depth)) => {
                        (*is_head, entry_key.clone(), *contributes_depth)
                    }
                    None => unreachable!("pending outcome belongs to a live child index"),
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
                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Keep(render_association(Vec::new())),
                        depth: 2,
                        deleted_count: deleted_before - subtree_start,
                        exhausted: traversal.limit_reached(deleted_before),
                    });
                }

                if child_outcome.exhausted {
                    let untouched_rest = children
                        .iter()
                        .skip(child_index + 1)
                        .filter_map(|(is_head, key, expr, _)| {
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
                        })
                        .collect::<Vec<_>>();
                    rebuilt_entries.extend(untouched_rest);

                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Keep(Expr::call(
                            rebuilt_head.unwrap_or_else(|| system_expr("Association")),
                            rebuilt_entries,
                        )),
                        depth: max_child_depth.saturating_add(1),
                        deleted_count: deleted_before - subtree_start,
                        exhausted: true,
                    });
                }

                child_index += 1;
            }

            let Some((_, _, child, _)) = children.get(child_index).cloned() else {
                if head_deleted {
                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Keep(render_association(Vec::new())),
                        depth: 2,
                        deleted_count: deleted_before - subtree_start,
                        exhausted: traversal.limit_reached(deleted_before),
                    });
                }

                let depth = max_child_depth.saturating_add(1);
                let expr = Expr::call(
                    rebuilt_head.unwrap_or_else(|| system_expr("Association")),
                    rebuilt_entries,
                );
                let finish_result = self.finish_delete_cases_at_current_node(
                    expr,
                    pattern,
                    traversal,
                    level - 1,
                    depth,
                    deleted_before,
                );
                return crate::resume_or_suspend!(
                    self,
                    finish_result,
                    ok(mut outcome) => { (|| {
                                    outcome.deleted_count += deleted_before - subtree_start;
                                    Ok(outcome)
                                })() },
                    suspend => DeleteCasesAssociationChildrenFromQueueSuspensionFrame1 { deleted_before, subtree_start },
                );
            };

            let child_result =
                self.delete_cases_owned(child, pattern.clone(), traversal, level, deleted_before);

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedDeleteAssociationChildren {
                        children: FrozenRewriteQueue::freeze(children),
                        child_index,
                        pattern,
                        traversal,
                        level,
                        subtree_start,
                        deleted_before,
                        rebuilt_entries: PersistentRewriteExprs::freeze(rebuilt_entries),
                        max_child_depth,
                        rebuilt_head,
                        head_deleted,
                    };
                    crate::resume_or_suspend!(insert self, DeleteCasesAssociationChildrenFromQueueSuspensionFrame2 { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(super) fn resume_suspended_delete_call_children(
        &mut self,
        mut state: SuspendedDeleteCallChildren,
        mut child_outcome: DeleteCasesOutcome,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        loop {
            let (is_head, contributes_depth) =
                match state.children.as_slice().get(state.child_index) {
                    Some((is_head, _, contributes_depth)) => (*is_head, *contributes_depth),
                    None => unreachable!("pending outcome belongs to a live child index"),
                };

            state.deleted_before += child_outcome.deleted_count;
            if contributes_depth {
                state.max_child_depth = state.max_child_depth.max(child_outcome.depth);
            }
            if is_head {
                match child_outcome.result {
                    DeleteCasesResult::Keep(expr) => state.rebuilt_head = Some(expr),
                    DeleteCasesResult::Delete | DeleteCasesResult::Splice(_) => {
                        state.head_deleted = true;
                    }
                }
            } else {
                match child_outcome.result {
                    DeleteCasesResult::Keep(expr) => {
                        state.rebuilt_args = state.rebuilt_args.appended(expr);
                    }
                    DeleteCasesResult::Delete => {}
                    DeleteCasesResult::Splice(items) => {
                        state.rebuilt_args = state.rebuilt_args.appended_values(items);
                    }
                }
            }

            if child_outcome.exhausted {
                let mut rebuilt_args = state.rebuilt_args.into_vec();
                rebuilt_args.extend(
                    state
                        .children
                        .as_slice()
                        .iter()
                        .skip(state.child_index + 1)
                        .filter_map(|(is_head, expr, _)| (!*is_head).then_some(expr.clone())),
                );
                if state.head_deleted {
                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Splice(rebuilt_args),
                        depth: state.max_child_depth,
                        deleted_count: state.deleted_before - state.subtree_start,
                        exhausted: true,
                    });
                }
                return Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Keep(Expr::call(
                        state
                            .rebuilt_head
                            .unwrap_or_else(|| system_expr("Sequence")),
                        rebuilt_args,
                    )),
                    depth: state.max_child_depth.saturating_add(1),
                    deleted_count: state.deleted_before - state.subtree_start,
                    exhausted: true,
                });
            }

            state.child_index += 1;
            let Some((_, child, _)) = state.children.as_slice().get(state.child_index).cloned()
            else {
                let rebuilt_args = state.rebuilt_args.into_vec();
                if state.head_deleted {
                    return Ok(DeleteCasesOutcome {
                        result: DeleteCasesResult::Splice(rebuilt_args),
                        depth: state.max_child_depth,
                        deleted_count: state.deleted_before - state.subtree_start,
                        exhausted: state.traversal.limit_reached(state.deleted_before),
                    });
                }

                let depth = state.max_child_depth.saturating_add(1);
                let expr = Expr::call(
                    state
                        .rebuilt_head
                        .unwrap_or_else(|| system_expr("Sequence")),
                    rebuilt_args,
                );
                let finish_result = self.finish_delete_cases_at_current_node(
                    expr,
                    state.pattern,
                    state.traversal,
                    state.level - 1,
                    depth,
                    state.deleted_before,
                );
                let added = state.deleted_before - state.subtree_start;
                return crate::resume_or_suspend!(
                    self,
                    finish_result,
                    ok(mut outcome) => { (|| {
                                    outcome.deleted_count += added;
                                    Ok(outcome)
                                })() },
                    suspend => ResumeSuspendedDeleteCallChildrenSuspensionFrame1 { added },
                );
            };

            let child_result = self.delete_cases_owned(
                child,
                state.pattern.clone(),
                state.traversal,
                state.level,
                state.deleted_before,
            );
            match child_result {
                Ok(outcome) => child_outcome = outcome,
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedDeleteCallChildrenSuspensionFrame2 { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(super) fn resume_suspended_delete_association_children(
        &mut self,
        mut state: SuspendedDeleteAssociationChildren,
        mut child_outcome: DeleteCasesOutcome,
    ) -> Result<DeleteCasesOutcome, EvalError> {
        loop {
            let (is_head, entry_key, contributes_depth) =
                match state.children.as_slice().get(state.child_index) {
                    Some((is_head, entry_key, _, contributes_depth)) => {
                        (*is_head, entry_key.clone(), *contributes_depth)
                    }
                    None => unreachable!("pending outcome belongs to a live child index"),
                };

            state.deleted_before += child_outcome.deleted_count;
            if contributes_depth {
                state.max_child_depth = state.max_child_depth.max(child_outcome.depth);
            }
            if is_head {
                match child_outcome.result {
                    DeleteCasesResult::Keep(expr) => state.rebuilt_head = Some(expr),
                    DeleteCasesResult::Delete | DeleteCasesResult::Splice(_) => {
                        state.head_deleted = true;
                    }
                }
            } else if let Some(key) = entry_key {
                if let DeleteCasesResult::Keep(value) = child_outcome.result {
                    state.rebuilt_entries = state
                        .rebuilt_entries
                        .appended(Expr::call(system_expr("Rule"), vec![key, value]));
                }
            } else {
                match child_outcome.result {
                    DeleteCasesResult::Keep(expr) => {
                        state.rebuilt_entries = state.rebuilt_entries.appended(expr);
                    }
                    DeleteCasesResult::Delete => {}
                    DeleteCasesResult::Splice(items) => {
                        state.rebuilt_entries = state.rebuilt_entries.appended_values(items);
                    }
                }
            }

            if state.head_deleted {
                return Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Keep(render_association(Vec::new())),
                    depth: 2,
                    deleted_count: state.deleted_before - state.subtree_start,
                    exhausted: state.traversal.limit_reached(state.deleted_before),
                });
            }

            if child_outcome.exhausted {
                let mut rebuilt_entries = state.rebuilt_entries.into_vec();
                rebuilt_entries.extend(
                    state
                        .children
                        .as_slice()
                        .iter()
                        .skip(state.child_index + 1)
                        .filter_map(|(is_head, key, expr, _)| {
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
                        }),
                );
                return Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Keep(Expr::call(
                        state
                            .rebuilt_head
                            .unwrap_or_else(|| system_expr("Association")),
                        rebuilt_entries,
                    )),
                    depth: state.max_child_depth.saturating_add(1),
                    deleted_count: state.deleted_before - state.subtree_start,
                    exhausted: true,
                });
            }

            state.child_index += 1;
            let Some((_, _, child, _)) = state.children.as_slice().get(state.child_index).cloned()
            else {
                let rebuilt_entries = state.rebuilt_entries.into_vec();
                let depth = state.max_child_depth.saturating_add(1);
                let expr = Expr::call(
                    state
                        .rebuilt_head
                        .unwrap_or_else(|| system_expr("Association")),
                    rebuilt_entries,
                );
                let finish_result = self.finish_delete_cases_at_current_node(
                    expr,
                    state.pattern,
                    state.traversal,
                    state.level - 1,
                    depth,
                    state.deleted_before,
                );
                let added = state.deleted_before - state.subtree_start;
                return crate::resume_or_suspend!(
                    self,
                    finish_result,
                    ok(mut outcome) => { (|| {
                                    outcome.deleted_count += added;
                                    Ok(outcome)
                                })() },
                    suspend => ResumeSuspendedDeleteAssociationChildrenSuspensionFrame1 { added },
                );
            };

            let child_result = self.delete_cases_owned(
                child,
                state.pattern.clone(),
                state.traversal,
                state.level,
                state.deleted_before,
            );
            match child_result {
                Ok(outcome) => child_outcome = outcome,
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedDeleteAssociationChildrenSuspensionFrame2 { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}
