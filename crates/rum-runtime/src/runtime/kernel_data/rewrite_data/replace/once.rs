mod depth;
mod options;
mod resume_frames;
mod traversal;
use depth::{replace_expr_depth, replace_expr_depth_for_heads, replace_expr_depth_with_heads};
use resume_frames::*;
use traversal::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

const RECURSIVE_REPLACE_CALL_DEPTH: usize = 16;

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplaceCallChildren {
    head: Expr,
    args: FrozenRewriteQueue<Expr>,
    child_index: usize,
    rule_plan: RewriteRulePlan,
    levels: CasesLevelSpec,
    child_level: usize,
    include_heads: bool,
    remaining_call_depth: usize,
    rebuilt_args: PersistentRewriteExprs,
    max_child_depth: usize,
    changed: bool,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplaceAssociationChildren {
    head: Expr,
    args: FrozenRewriteQueue<Expr>,
    index: usize,
    rule_plan: RewriteRulePlan,
    levels: CasesLevelSpec,
    child_level: usize,
    include_heads: bool,
    remaining_call_depth: usize,
    rebuilt_entries: PersistentRewriteExprs,
    max_child_depth: usize,
    changed: bool,
}

struct SuspendedReplaceCompletion {
    head: Expr,
    rebuilt: PersistentRewriteExprs,
    max_child_depth: usize,
    changed: bool,
}

impl SuspendedReplaceCompletion {
    fn finish(self) -> ReplaceOutcome {
        ReplaceOutcome {
            expr: Expr::call(self.head, self.rebuilt.into_vec()),
            depth: self.max_child_depth.saturating_add(1),
            changed: self.changed,
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_replace(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("Replace", args_slice));
        }
        if args_slice.len() < 2 {
            return Ok(self.invalid_call_with_arg_range("Replace", args_slice, 2, 3));
        }
        let Some((levels_expr, heads_option_exprs)) =
            partition_replace_trailing_args(&args_slice[2..])
        else {
            return Ok(self.invalid_call_with_arg_range("Replace", args_slice, 2, 3));
        };

        let target_result = self.evaluate(args_slice[0].clone());
        match target_result {
            Ok(target) => self.resume_replace_after_target(
                target,
                args_slice[1].clone(),
                levels_expr,
                heads_option_exprs,
                args_slice.to_vec(),
            ),
            Err(EvalError::InputRequired { request }) => {
                let heads_option_exprs = SharedArgs::new(heads_option_exprs);
                let args = args.clone();
                crate::resume_or_suspend!(insert self, EvalReplaceSuspensionFrame { args, levels_expr, heads_option_exprs });
                crate::resume_or_suspend!(insert self, ContinueReplaceSetupFrame { rule_plan: None });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_replace_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        levels_expr: Option<Expr>,
        heads_option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(
            rules_expr,
            RewriteRuleExprContinuation::Replace {
                target,
                levels_expr,
                heads_option_exprs,
                call_args,
            },
        )
    }

    pub(in crate::runtime) fn resume_replace_after_rule_exprs(
        &mut self,
        target: Expr,
        levels_expr: Option<Expr>,
        heads_option_exprs: Vec<Expr>,
        normalized_rule_exprs: Option<Vec<Expr>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "Replace",
                &call_args,
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");
        let rule_plan = Self::shared_rewrite_rule_plan(rules);

        self.system_option_bool(
            "Replace",
            "Heads",
            false,
            options::ReplaceOptionWorking {
                target,
                rule_plan,
                levels_expr,
                heads_option_exprs,
                call_args,
            },
        )
    }

    fn finish_replace_heads_resolution(
        &mut self,
        target: Expr,
        rule_plan: RewriteRulePlan,
        levels_expr: Option<Expr>,
        call_args: Vec<Expr>,
        include_heads: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(include_heads_expr) = include_heads else {
            return Ok(self.invalid_call_with_arg_form(
                "Replace",
                &call_args,
                "expected Heads -> True/False as a trailing option",
            ));
        };
        let include_heads = is_system_symbol_name(&include_heads_expr, "True");
        let levels_result = self.resolve_replace_level_spec_owned(levels_expr);
        match levels_result {
            Ok(levels) => self.finish_replace_levels_resolution(
                target,
                rule_plan,
                call_args,
                include_heads,
                levels,
            ),
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                let next = ContinueReplaceSetupFrame {
                    rule_plan: Some(rule_plan.clone()),
                };
                crate::resume_or_suspend!(insert self, FinishReplaceHeadsResolutionSuspensionFrame { target, rule_plan, call_args, include_heads });
                crate::resume_or_suspend!(insert self, next);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_replace_levels_resolution(
        &mut self,
        target: Expr,
        rule_plan: RewriteRulePlan,
        call_args: Vec<Expr>,
        include_heads: bool,
        levels: Option<CasesLevelSpec>,
    ) -> Result<Expr, EvalError> {
        let Some(levels) = levels else {
            return Ok(self.invalid_call_with_arg_form(
                "Replace",
                &call_args,
                "expected a standard levelspec as the optional third argument",
            ));
        };
        let replaced_result = self.replace_owned(target, rule_plan, levels, 0, include_heads);
        crate::resume_or_suspend!(
            self,
            replaced_result,
            ok(outcome) => { let session = self; (|| {
                    let evaluated_result = session.evaluate(outcome.expr);
                    crate::resume_or_suspend!(
            session,
            evaluated_result,
            ok(value) => Ok(value),
            suspend => FinishReplaceLevelsResolutionSuspensionFrame2,
        )
                })() },
            suspend => FinishReplaceLevelsResolutionSuspensionFrame1,
        )
    }

    fn resolve_replace_heads_options_owned(
        &mut self,
        option_exprs: Vec<Expr>,
        default: bool,
    ) -> Result<Option<Expr>, EvalError> {
        if option_exprs.is_empty() {
            return Ok(Some(bool_expr(default)));
        }

        let resolved_result = self.resolve_argument_exprs_owned(option_exprs);
        crate::resume_or_suspend!(
            self,
            resolved_result,
            ok(resolved_exprs) => { (|| {
                    let mut include_heads = default;
                    for expr in resolved_exprs {
                        let Some(resolved) = parse_heads_option_with_default(&expr, include_heads) else {
                            return Ok(None);
                        };
                        include_heads = resolved;
                    }
                    Ok(Some(bool_expr(include_heads)))
                })() },
            suspend => ResolveReplaceHeadsOptionsOwnedSuspensionFrame { default },
        )
    }

    fn resolve_replace_level_spec_owned(
        &mut self,
        levels_expr: Option<Expr>,
    ) -> Result<Option<CasesLevelSpec>, EvalError> {
        let Some(levels_expr) = levels_expr else {
            return Ok(Some(CasesLevelSpec::exact(0)));
        };

        let resolved_result = self.evaluate(levels_expr);
        crate::resume_or_suspend!(
            self,
            resolved_result,
            ok(resolved_expr) => { let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resolved_expr); (|| {
                    Ok(parse_standard_level_spec(&resolved))
                })() },
            suspend => ResolveReplaceLevelSpecOwnedSuspensionFrame,
        )
    }

    fn replace_owned(
        &mut self,
        expr: Expr,
        rule_plan: RewriteRulePlan,
        levels: CasesLevelSpec,
        level: usize,
        include_heads: bool,
    ) -> Result<ReplaceOutcome, EvalError> {
        self.replace_owned_recursive_bounded(
            expr,
            rule_plan,
            levels,
            level,
            include_heads,
            RECURSIVE_REPLACE_CALL_DEPTH,
        )
    }

    fn replace_owned_recursive_bounded(
        &mut self,
        expr: Expr,
        rule_plan: RewriteRulePlan,
        levels: CasesLevelSpec,
        level: usize,
        include_heads: bool,
        remaining_call_depth: usize,
    ) -> Result<ReplaceOutcome, EvalError> {
        if remaining_call_depth == 0 && matches!(expr, Expr::Call { .. }) {
            return self.replace_owned_iterative(expr, rule_plan, levels, level, include_heads);
        }

        let depth = if include_heads {
            replace_expr_depth_with_heads(&expr)
        } else {
            replace_expr_depth(&expr)
        };
        if levels.matches(level, depth) {
            let replaced_result = self.apply_rewrite_rule_plan_once(
                expr.clone(),
                rule_plan.clone(),
                RuleApplicationSource::Replace,
            );
            return crate::resume_or_suspend!(
                self,
                replaced_result,
                ok(replaced) => { let session = self; (|| {
                            if let Some(replaced) = replaced {
                                let depth = if include_heads {
                                    replace_expr_depth_with_heads(&replaced)
                                } else {
                                    replace_expr_depth(&replaced)
                                };
                                Ok(ReplaceOutcome {
                                    depth,
                                    expr: replaced,
                                    changed: true,
                                })
                            } else {
                                session.replace_children_owned(
                                    expr,
                                    rule_plan,
                                    levels,
                                    level + 1,
                                    include_heads,
                                    remaining_call_depth,
                                )
                            }
                        })() },
                suspend => ReplaceOwnedSuspensionFrame { include_heads, expr, rule_plan, levels, level, remaining_call_depth },
            );
        }

        self.replace_children_owned(
            expr,
            rule_plan,
            levels,
            level + 1,
            include_heads,
            remaining_call_depth,
        )
    }

    fn replace_children_owned(
        &mut self,
        expr: Expr,
        rule_plan: RewriteRulePlan,
        levels: CasesLevelSpec,
        child_level: usize,
        include_heads: bool,
        remaining_call_depth: usize,
    ) -> Result<ReplaceOutcome, EvalError> {
        match expr {
            Expr::Atom(_) => Ok(ReplaceOutcome {
                expr,
                depth: 1,
                changed: false,
            }),
            Expr::Call { head, args } if is_system_symbol_name(&head, "Association") => {
                let remaining_call_depth = remaining_call_depth
                    .checked_sub(1)
                    .expect("iterative fallback handles the bounded traversal tail");
                let head = head.into_owned();
                let args = args.into_vec();
                if include_heads {
                    let head_result = self.replace_owned_recursive_bounded(
                        head,
                        rule_plan.clone(),
                        levels,
                        child_level,
                        include_heads,
                        remaining_call_depth,
                    );
                    match head_result {
                        Ok(outcome) => self.replace_association_children_owned(
                            outcome.expr,
                            args,
                            rule_plan,
                            levels,
                            child_level,
                            include_heads,
                            remaining_call_depth,
                            Vec::new(),
                            outcome.depth,
                            outcome.changed,
                            0,
                            None,
                        ),
                        Err(EvalError::InputRequired { request }) => {
                            let args = FrozenRewriteQueue::freeze(args);
                            crate::resume_or_suspend!(insert self, ReplaceChildrenOwnedSuspensionFrame1 { args, rule_plan, levels, child_level, include_heads, remaining_call_depth });
                            Err(EvalError::InputRequired { request })
                        }
                        Err(error) => Err(error),
                    }
                } else {
                    self.replace_association_children_owned(
                        head,
                        args,
                        rule_plan,
                        levels,
                        child_level,
                        include_heads,
                        remaining_call_depth,
                        Vec::new(),
                        1,
                        false,
                        0,
                        None,
                    )
                }
            }
            Expr::Call { head, args } => {
                let remaining_call_depth = remaining_call_depth
                    .checked_sub(1)
                    .expect("iterative fallback handles the bounded traversal tail");
                let head = head.into_owned();
                let args = args.into_vec();
                if include_heads {
                    let head_result = self.replace_owned_recursive_bounded(
                        head,
                        rule_plan.clone(),
                        levels,
                        child_level,
                        include_heads,
                        remaining_call_depth,
                    );
                    match head_result {
                        Ok(outcome) => self.replace_call_children_owned(
                            outcome.expr,
                            args,
                            0,
                            rule_plan,
                            levels,
                            child_level,
                            include_heads,
                            remaining_call_depth,
                            Vec::new(),
                            outcome.depth,
                            outcome.changed,
                            None,
                        ),
                        Err(EvalError::InputRequired { request }) => {
                            let args = FrozenRewriteQueue::freeze(args);
                            crate::resume_or_suspend!(insert self, ReplaceChildrenOwnedSuspensionFrame2 { args, rule_plan, levels, child_level, include_heads, remaining_call_depth });
                            Err(EvalError::InputRequired { request })
                        }
                        Err(error) => Err(error),
                    }
                } else {
                    self.replace_call_children_owned(
                        head,
                        args,
                        0,
                        rule_plan,
                        levels,
                        child_level,
                        include_heads,
                        remaining_call_depth,
                        Vec::new(),
                        1,
                        false,
                        None,
                    )
                }
            }
        }
    }

    /// Walks the children one per loop iteration, keeping the accumulated
    /// `rebuilt_args` in place instead of recursing per element. `pending`
    /// carries an already-computed child outcome across a suspension resume,
    /// so the merge step runs exactly once per element on both the live and
    /// the resumed paths.
    fn replace_call_children_owned(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        mut child_index: usize,
        rule_plan: RewriteRulePlan,
        levels: CasesLevelSpec,
        child_level: usize,
        include_heads: bool,
        remaining_call_depth: usize,
        mut rebuilt_args: Vec<Expr>,
        mut max_child_depth: usize,
        mut changed: bool,
        mut pending: Option<ReplaceOutcome>,
    ) -> Result<ReplaceOutcome, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                let child_changed = outcome.changed;
                let appended_depth = if child_changed {
                    let appended_start = rebuilt_args.len();
                    let spliced = self.append_changed_rewrite_argument(
                        &head,
                        &mut rebuilt_args,
                        outcome.expr,
                    );
                    if spliced {
                        rebuilt_args[appended_start..]
                            .iter()
                            .map(|expr| replace_expr_depth_for_heads(expr, include_heads))
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
                changed = changed || child_changed;
                child_index += 1;
            }

            let Some(child) = args.get(child_index).cloned() else {
                return Ok(ReplaceOutcome {
                    expr: Expr::call(head, rebuilt_args),
                    depth: max_child_depth.saturating_add(1),
                    changed,
                });
            };

            let child_result = self.replace_owned_recursive_bounded(
                child,
                rule_plan.clone(),
                levels,
                child_level,
                include_heads,
                remaining_call_depth,
            );

            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedReplaceCallChildren {
                        head,
                        args: FrozenRewriteQueue::freeze(args),
                        child_index,
                        rule_plan,
                        levels,
                        child_level,
                        include_heads,
                        remaining_call_depth,
                        rebuilt_args: PersistentRewriteExprs::freeze(rebuilt_args),
                        max_child_depth,
                        changed,
                    };
                    crate::resume_or_suspend!(insert self, ReplaceCallChildrenOwnedSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    /// Iterative association-entry walk; `pending` carries an already-computed
    /// entry outcome across a suspension resume so the merge step runs exactly
    /// once per entry on both the live and the resumed paths.
    fn replace_association_children_owned(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        rule_plan: RewriteRulePlan,
        levels: CasesLevelSpec,
        child_level: usize,
        include_heads: bool,
        remaining_call_depth: usize,
        mut rebuilt_entries: Vec<Expr>,
        mut max_child_depth: usize,
        mut changed: bool,
        mut index: usize,
        mut pending: Option<ReplaceOutcome>,
    ) -> Result<ReplaceOutcome, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                match args.get(index) {
                    Some(Expr::Call {
                        head: rule_head,
                        args: rule_args,
                    }) if is_system_symbol_name(rule_head, "Rule") && rule_args.len() == 2 => {
                        rebuilt_entries.push(Expr::call(
                            system_expr("Rule"),
                            vec![rule_args[0].clone(), outcome.expr],
                        ));
                    }
                    _ => {
                        rebuilt_entries.push(outcome.expr);
                    }
                }
                max_child_depth = max_child_depth.max(outcome.depth);
                changed = changed || outcome.changed;
                index += 1;
            }

            let Some(entry) = args.get(index).cloned() else {
                return Ok(ReplaceOutcome {
                    expr: Expr::call(head, rebuilt_entries),
                    depth: max_child_depth.saturating_add(1),
                    changed,
                });
            };

            let child = match entry {
                Expr::Call {
                    head: rule_head,
                    args: rule_args,
                } if is_system_symbol_name(&rule_head, "Rule") && rule_args.len() == 2 => {
                    Expr::unwrap_shared_args(rule_args)
                        .pop()
                        .expect("association Rule entry stores a value")
                }
                other => other,
            };
            let entry_result = self.replace_owned_recursive_bounded(
                child,
                rule_plan.clone(),
                levels,
                child_level,
                include_heads,
                remaining_call_depth,
            );

            match entry_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedReplaceAssociationChildren {
                        head,
                        args: FrozenRewriteQueue::freeze(args),
                        index,
                        rule_plan,
                        levels,
                        child_level,
                        include_heads,
                        remaining_call_depth,
                        rebuilt_entries: PersistentRewriteExprs::freeze(rebuilt_entries),
                        max_child_depth,
                        changed,
                    };
                    crate::resume_or_suspend!(insert self, ReplaceAssociationChildrenOwnedSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_suspended_replace_call_head(
        &mut self,
        args: FrozenRewriteQueue<Expr>,
        rule_plan: RewriteRulePlan,
        levels: CasesLevelSpec,
        child_level: usize,
        include_heads: bool,
        remaining_call_depth: usize,
        outcome: ReplaceOutcome,
    ) -> Result<SuspendedReplaceCompletion, EvalError> {
        self.drive_suspended_replace_call_children(
            SuspendedReplaceCallChildren {
                head: outcome.expr,
                args,
                child_index: 0,
                rule_plan,
                levels,
                child_level,
                include_heads,
                remaining_call_depth,
                rebuilt_args: PersistentRewriteExprs::freeze(Vec::new()),
                max_child_depth: outcome.depth,
                changed: outcome.changed,
            },
            None,
        )
    }

    fn resume_suspended_replace_call_children(
        &mut self,
        state: SuspendedReplaceCallChildren,
        outcome: ReplaceOutcome,
    ) -> Result<SuspendedReplaceCompletion, EvalError> {
        self.drive_suspended_replace_call_children(state, Some(outcome))
    }

    fn drive_suspended_replace_call_children(
        &mut self,
        mut state: SuspendedReplaceCallChildren,
        mut pending: Option<ReplaceOutcome>,
    ) -> Result<SuspendedReplaceCompletion, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                let child_changed = outcome.changed;
                let appended_depth = if child_changed {
                    match outcome.expr {
                        Expr::Call { head, args }
                            if is_system_symbol_name(&head, heads::functions::SEQUENCE)
                                && !self.attribute_set_for_head(&state.head).sequence_hold() =>
                        {
                            let appended_depth = args
                                .iter()
                                .map(|expr| replace_expr_depth_for_heads(expr, state.include_heads))
                                .max()
                                .unwrap_or(0);
                            state.rebuilt_args =
                                state.rebuilt_args.appended_values(args.into_vec());
                            appended_depth
                        }
                        other => {
                            state.rebuilt_args = state.rebuilt_args.appended(other);
                            outcome.depth
                        }
                    }
                } else {
                    state.rebuilt_args = state.rebuilt_args.appended(outcome.expr);
                    outcome.depth
                };
                state.max_child_depth = state.max_child_depth.max(appended_depth);
                state.changed |= child_changed;
                state.child_index += 1;
            }

            let Some(child) = state.args.as_slice().get(state.child_index).cloned() else {
                return Ok(SuspendedReplaceCompletion {
                    head: state.head,
                    rebuilt: state.rebuilt_args,
                    max_child_depth: state.max_child_depth,
                    changed: state.changed,
                });
            };
            let child_result = self.replace_owned_recursive_bounded(
                child,
                state.rule_plan.clone(),
                state.levels,
                state.child_level,
                state.include_heads,
                state.remaining_call_depth,
            );
            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, DriveSuspendedReplaceCallChildrenSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_suspended_replace_association_head(
        &mut self,
        args: FrozenRewriteQueue<Expr>,
        rule_plan: RewriteRulePlan,
        levels: CasesLevelSpec,
        child_level: usize,
        include_heads: bool,
        remaining_call_depth: usize,
        outcome: ReplaceOutcome,
    ) -> Result<SuspendedReplaceCompletion, EvalError> {
        self.drive_suspended_replace_association_children(
            SuspendedReplaceAssociationChildren {
                head: outcome.expr,
                args,
                index: 0,
                rule_plan,
                levels,
                child_level,
                include_heads,
                remaining_call_depth,
                rebuilt_entries: PersistentRewriteExprs::freeze(Vec::new()),
                max_child_depth: outcome.depth,
                changed: outcome.changed,
            },
            None,
        )
    }

    fn resume_suspended_replace_association_children(
        &mut self,
        state: SuspendedReplaceAssociationChildren,
        outcome: ReplaceOutcome,
    ) -> Result<SuspendedReplaceCompletion, EvalError> {
        self.drive_suspended_replace_association_children(state, Some(outcome))
    }

    fn drive_suspended_replace_association_children(
        &mut self,
        mut state: SuspendedReplaceAssociationChildren,
        mut pending: Option<ReplaceOutcome>,
    ) -> Result<SuspendedReplaceCompletion, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                let rebuilt = match state.args.as_slice().get(state.index) {
                    Some(Expr::Call {
                        head: rule_head,
                        args: rule_args,
                    }) if is_system_symbol_name(rule_head, "Rule") && rule_args.len() == 2 => {
                        Expr::call(
                            system_expr("Rule"),
                            vec![rule_args[0].clone(), outcome.expr],
                        )
                    }
                    _ => outcome.expr,
                };
                state.rebuilt_entries = state.rebuilt_entries.appended(rebuilt);
                state.max_child_depth = state.max_child_depth.max(outcome.depth);
                state.changed |= outcome.changed;
                state.index += 1;
            }

            let Some(entry) = state.args.as_slice().get(state.index).cloned() else {
                return Ok(SuspendedReplaceCompletion {
                    head: state.head,
                    rebuilt: state.rebuilt_entries,
                    max_child_depth: state.max_child_depth,
                    changed: state.changed,
                });
            };
            let child = match entry {
                Expr::Call {
                    head: rule_head,
                    args: rule_args,
                } if is_system_symbol_name(&rule_head, "Rule") && rule_args.len() == 2 => {
                    Expr::unwrap_shared_args(rule_args)
                        .pop()
                        .expect("association Rule entry stores a value")
                }
                other => other,
            };
            let child_result = self.replace_owned_recursive_bounded(
                child,
                state.rule_plan.clone(),
                state.levels,
                state.child_level,
                state.include_heads,
                state.remaining_call_depth,
            );
            match child_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, DriveSuspendedReplaceAssociationChildrenSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedReplaceAssociationChildren>();
    assert_traces::<SuspendedReplaceCallChildren>();
};

fn bool_expr(value: bool) -> Expr {
    system_expr(if value { "True" } else { "False" })
}

fn partition_replace_trailing_args(args: &[Expr]) -> Option<(Option<Expr>, Vec<Expr>)> {
    let mut levels_expr = None;
    let mut heads_option_exprs = Vec::new();

    for arg in args {
        if is_system_option_rule(arg, "Heads") {
            heads_option_exprs.push(arg.clone());
        } else if levels_expr.replace(arg.clone()).is_some() {
            return None;
        }
    }

    Some((levels_expr, heads_option_exprs))
}

fn is_system_option_rule(expr: &Expr, option_name: &str) -> bool {
    let Expr::Call { head, args } = expr else {
        return false;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return false;
    }

    option_symbol_matches(&args[0], option_name)
}

fn option_symbol_matches(expr: &Expr, option_name: &str) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbol.name() == option_name,
        Expr::Atom(Atom::String(name)) => {
            name == option_name
                || resolve_unqualified_symbol(name)
                    .is_some_and(|symbol| symbol.name() == option_name)
        }
        _ => false,
    }
}
