use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
pub(super) enum ReplaceTraversalWork {
    Visit {
        expr: Expr,
        level: usize,
    },
    FinishCall {
        head: Expr,
        arg_count: usize,
    },
    FinishAssociation {
        head: Expr,
        entry_keys: Vec<Option<Expr>>,
    },
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct ReplaceTraversalOutcome {
    expr: Expr,
    depth: usize,
    changed: bool,
}

impl From<ReplaceTraversalOutcome> for ReplaceOutcome {
    fn from(outcome: ReplaceTraversalOutcome) -> Self {
        Self {
            expr: outcome.expr,
            depth: outcome.depth,
            changed: outcome.changed,
        }
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct ReplaceTraversalState {
    work: Vec<ReplaceTraversalWork>,
    completed: Vec<ReplaceTraversalOutcome>,
    rule_plan: RewriteRulePlan,
    levels: CasesLevelSpec,
    include_heads: bool,
}

impl Session {
    pub(super) fn replace_owned_iterative(
        &mut self,
        expr: Expr,
        rule_plan: RewriteRulePlan,
        levels: CasesLevelSpec,
        level: usize,
        include_heads: bool,
    ) -> Result<ReplaceOutcome, EvalError> {
        self.drive_replace_traversal(ReplaceTraversalState {
            work: vec![ReplaceTraversalWork::Visit { expr, level }],
            completed: Vec::new(),
            rule_plan,
            levels,
            include_heads,
        })
    }

    fn drive_replace_traversal(
        &mut self,
        mut state: ReplaceTraversalState,
    ) -> Result<ReplaceOutcome, EvalError> {
        while let Some(work) = state.work.pop() {
            match work {
                ReplaceTraversalWork::Visit { expr, level } => {
                    let depth = replace_expr_depth_for_heads(&expr, state.include_heads);
                    if state.levels.matches(level, depth) {
                        let replaced_result = self.apply_rewrite_rule_plan_once(
                            expr.clone(),
                            state.rule_plan.clone(),
                            RuleApplicationSource::Replace,
                        );
                        match replaced_result {
                            Ok(replaced) => {
                                self.finish_replace_traversal_visit(
                                    &mut state, expr, level, replaced,
                                );
                            }
                            Err(EvalError::InputRequired { request }) => {
                                crate::resume_or_suspend!(insert self, DriveReplaceTraversalSuspensionFrame { state, expr, level });
                                return Err(EvalError::InputRequired { request });
                            }
                            Err(error) => return Err(error),
                        }
                    } else {
                        self.finish_replace_traversal_visit(&mut state, expr, level, None);
                    }
                }
                ReplaceTraversalWork::FinishCall { head, arg_count } => {
                    self.finish_replace_traversal_call(&mut state, head, arg_count);
                }
                ReplaceTraversalWork::FinishAssociation { head, entry_keys } => {
                    Self::finish_replace_traversal_association(&mut state, head, entry_keys);
                }
            }
        }

        debug_assert_eq!(state.completed.len(), 1);
        Ok(state
            .completed
            .pop()
            .expect("Replace traversal completes its root")
            .into())
    }

    pub(super) fn resume_replace_traversal(
        &mut self,
        mut state: ReplaceTraversalState,
        expr: Expr,
        level: usize,
        replaced: Option<Expr>,
    ) -> Result<ReplaceOutcome, EvalError> {
        self.finish_replace_traversal_visit(&mut state, expr, level, replaced);
        self.drive_replace_traversal(state)
    }

    fn finish_replace_traversal_visit(
        &mut self,
        state: &mut ReplaceTraversalState,
        expr: Expr,
        level: usize,
        replaced: Option<Expr>,
    ) {
        if let Some(replaced) = replaced {
            state.completed.push(ReplaceTraversalOutcome {
                depth: replace_expr_depth_for_heads(&replaced, state.include_heads),
                expr: replaced,
                changed: true,
            });
            return;
        }

        match expr {
            Expr::Atom(_) => state.completed.push(ReplaceTraversalOutcome {
                expr,
                depth: 1,
                changed: false,
            }),
            Expr::Call { head, args } if is_system_symbol_name(&head, "Association") => {
                let child_level = level.saturating_add(1);
                let mut children = Vec::with_capacity(args.len());
                let mut entry_keys = Vec::with_capacity(args.len());
                for entry in args.into_vec() {
                    match entry {
                        Expr::Call {
                            head: rule_head,
                            args: rule_args,
                        } if is_system_symbol_name(&rule_head, "Rule") && rule_args.len() == 2 => {
                            let mut rule_args = Expr::unwrap_shared_args(rule_args);
                            let value = rule_args
                                .pop()
                                .expect("association Rule entry stores a value");
                            let key = rule_args
                                .pop()
                                .expect("association Rule entry stores a key");
                            entry_keys.push(Some(key));
                            children.push(value);
                        }
                        other => {
                            entry_keys.push(None);
                            children.push(other);
                        }
                    }
                }

                let head = head.into_owned();
                state.work.push(ReplaceTraversalWork::FinishAssociation {
                    head: head.clone(),
                    entry_keys,
                });
                for child in children.into_iter().rev() {
                    state.work.push(ReplaceTraversalWork::Visit {
                        expr: child,
                        level: child_level,
                    });
                }
                if state.include_heads {
                    state.work.push(ReplaceTraversalWork::Visit {
                        expr: head,
                        level: child_level,
                    });
                }
            }
            Expr::Call { head, args } => {
                let child_level = level.saturating_add(1);
                let head = head.into_owned();
                let args = args.into_vec();
                state.work.push(ReplaceTraversalWork::FinishCall {
                    head: head.clone(),
                    arg_count: args.len(),
                });
                for child in args.into_iter().rev() {
                    state.work.push(ReplaceTraversalWork::Visit {
                        expr: child,
                        level: child_level,
                    });
                }
                if state.include_heads {
                    state.work.push(ReplaceTraversalWork::Visit {
                        expr: head,
                        level: child_level,
                    });
                }
            }
        }
    }

    fn finish_replace_traversal_call(
        &mut self,
        state: &mut ReplaceTraversalState,
        original_head: Expr,
        arg_count: usize,
    ) {
        let completed_count = arg_count + usize::from(state.include_heads);
        let first_child = state
            .completed
            .len()
            .checked_sub(completed_count)
            .expect("Replace call traversal results remain balanced");
        let mut children = state.completed.split_off(first_child).into_iter();
        let (head, mut max_child_depth, mut changed) = if state.include_heads {
            let outcome = children
                .next()
                .expect("Replace call traversal completes its head");
            (outcome.expr, outcome.depth, outcome.changed)
        } else {
            (original_head, 1, false)
        };

        let mut rebuilt_args = Vec::with_capacity(arg_count);
        for outcome in children {
            let child_changed = outcome.changed;
            let appended_depth = if child_changed {
                let appended_start = rebuilt_args.len();
                if self.append_changed_rewrite_argument(&head, &mut rebuilt_args, outcome.expr) {
                    rebuilt_args[appended_start..]
                        .iter()
                        .map(|expr| replace_expr_depth_for_heads(expr, state.include_heads))
                        .max()
                        .unwrap_or(0)
                } else {
                    outcome.depth
                }
            } else {
                rebuilt_args.push(outcome.expr);
                outcome.depth
            };
            max_child_depth = max_child_depth.max(appended_depth);
            changed |= child_changed;
        }

        state.completed.push(ReplaceTraversalOutcome {
            expr: Expr::call(head, rebuilt_args),
            depth: max_child_depth.saturating_add(1),
            changed,
        });
    }

    fn finish_replace_traversal_association(
        state: &mut ReplaceTraversalState,
        original_head: Expr,
        entry_keys: Vec<Option<Expr>>,
    ) {
        let completed_count = entry_keys.len() + usize::from(state.include_heads);
        let first_child = state
            .completed
            .len()
            .checked_sub(completed_count)
            .expect("Replace association traversal results remain balanced");
        let mut children = state.completed.split_off(first_child).into_iter();
        let (head, mut max_child_depth, mut changed) = if state.include_heads {
            let outcome = children
                .next()
                .expect("Replace association traversal completes its head");
            (outcome.expr, outcome.depth, outcome.changed)
        } else {
            (original_head, 1, false)
        };

        let mut rebuilt_entries = Vec::with_capacity(entry_keys.len());
        for (key, outcome) in entry_keys.into_iter().zip(children) {
            rebuilt_entries.push(match key {
                Some(key) => Expr::call(system_expr("Rule"), vec![key, outcome.expr]),
                None => outcome.expr,
            });
            max_child_depth = max_child_depth.max(outcome.depth);
            changed |= outcome.changed;
        }

        state.completed.push(ReplaceTraversalOutcome {
            expr: Expr::call(head, rebuilt_entries),
            depth: max_child_depth.saturating_add(1),
            changed,
        });
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ReplaceTraversalOutcome>();
    assert_traces::<ReplaceTraversalState>();
    assert_traces::<ReplaceTraversalWork>();
};
