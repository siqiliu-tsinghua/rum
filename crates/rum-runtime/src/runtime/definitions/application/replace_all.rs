mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod plan;

pub(in crate::runtime) use plan::RewriteRulePlan;
use plan::RewriteRuleScanCursor;

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct ReplaceAllTraversalOutcome {
    expr: Expr,
    changed: bool,
}

#[derive(Clone, FrameRootTrace)]
enum ReplaceAllTraversalWork {
    Visit(Expr),
    FinishCall {
        original_head: SharedExpr,
        original_args: SharedArgs,
        rewrite_args: Vec<bool>,
    },
}

#[derive(Clone, FrameRootTrace)]
struct ReplaceAllTraversalState {
    work: Vec<ReplaceAllTraversalWork>,
    completed: Vec<ReplaceAllTraversalOutcome>,
    rules: RewriteRulePlan,
    source: RuleApplicationSource,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedRewriteRuleMatch {
    expr: Expr,
    normalized_expr: Expr,
    rules: RewriteRulePlan,
}

impl ContinuationValue for ReplaceAllTraversalOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationReplaceAllTraversalOutcome"),
            vec![
                self.expr,
                system_expr(if self.changed { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(
                    &head,
                    "InternalContinuationReplaceAllTraversalOutcome",
                ) && args.len() == 2 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let changed_expr = args
                    .pop()
                    .expect("replace-all traversal outcome stores changed");
                let expr = args
                    .pop()
                    .expect("replace-all traversal outcome stores expr");
                Self {
                    expr,
                    changed: is_system_symbol_name(&changed_expr, "True"),
                }
            }
            other => panic!("unexpected replace-all traversal outcome payload: {other:?}"),
        }
    }
}

impl Session {
    pub(in crate::runtime) fn apply_replace_all(
        &mut self,
        expr: Expr,
        rules: &[RewriteRule],
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        let outcome_result =
            self.apply_replace_all_owned(expr, RewriteRulePlan::from_slice(rules), source);
        crate::resume_or_suspend!(
            self,
            outcome_result,
            ok(outcome) => { (|| { Ok(outcome.expr) })() },
            suspend => ApplyReplaceAllSuspensionFrame,
        )
    }

    pub(in crate::runtime) fn apply_replace_all_with_plan(
        &mut self,
        expr: Expr,
        rules: RewriteRulePlan,
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        let outcome_result = self.apply_replace_all_owned(expr, rules, source);
        crate::resume_or_suspend!(
            self,
            outcome_result,
            ok(outcome) => { (|| { Ok(outcome.expr) })() },
            suspend => ApplyReplaceAllWithPlanSuspensionFrame,
        )
    }

    pub(in crate::runtime) fn shared_rewrite_rule_plan(rules: Vec<RewriteRule>) -> RewriteRulePlan {
        if rules
            .iter()
            .any(|rule| matches!(rule.matcher, MatcherNode::Exact(_)))
        {
            RewriteRulePlan::from_slice(&rules)
        } else {
            RewriteRulePlan::linear(rules)
        }
    }

    pub(in crate::runtime) fn apply_rewrite_rule_plan_once(
        &mut self,
        expr: Expr,
        rules: RewriteRulePlan,
        source: RuleApplicationSource,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        // Replace and ReplaceAt build this indexed, Arc-backed plan once per
        // traversal; their cloneable applier carries the same plan across
        // targets and suspension frames.
        let cursor = rules.start_scan(&normalized_expr);
        self.apply_rules_once_from_list(expr, normalized_expr, rules, source, cursor)
    }

    fn apply_replace_all_owned(
        &mut self,
        expr: Expr,
        rules: RewriteRulePlan,
        source: RuleApplicationSource,
    ) -> Result<ReplaceAllTraversalOutcome, EvalError> {
        if matches!(expr, Expr::Atom(_)) {
            let replace_result =
                self.apply_rewrite_rule_plan_once(expr.clone(), rules.clone(), source);
            return match replace_result {
                Ok(Some(replaced)) => Ok(ReplaceAllTraversalOutcome {
                    expr: replaced,
                    changed: true,
                }),
                Ok(None) => Ok(ReplaceAllTraversalOutcome {
                    expr,
                    changed: false,
                }),
                Err(EvalError::InputRequired { request }) => {
                    let state = ReplaceAllTraversalState {
                        work: Vec::new(),
                        completed: Vec::new(),
                        rules,
                        source,
                    };
                    crate::resume_or_suspend!(insert self, ApplyReplaceAllOwnedSuspensionFrame { state, expr });
                    Err(EvalError::InputRequired { request })
                }
                Err(error) => Err(error),
            };
        }

        self.drive_replace_all_traversal(ReplaceAllTraversalState {
            work: vec![ReplaceAllTraversalWork::Visit(expr)],
            completed: Vec::new(),
            rules,
            source,
        })
    }

    fn drive_replace_all_traversal(
        &mut self,
        mut state: ReplaceAllTraversalState,
    ) -> Result<ReplaceAllTraversalOutcome, EvalError> {
        while let Some(work) = state.work.pop() {
            match work {
                ReplaceAllTraversalWork::Visit(expr) => {
                    let replace_result = self.apply_rewrite_rule_plan_once(
                        expr.clone(),
                        state.rules.clone(),
                        state.source,
                    );
                    match replace_result {
                        Ok(replaced) => {
                            self.finish_replace_all_visit(&mut state, expr, replaced);
                        }
                        Err(EvalError::InputRequired { request }) => {
                            crate::resume_or_suspend!(insert self, ApplyReplaceAllOwnedSuspensionFrame { state, expr });
                            return Err(EvalError::InputRequired { request });
                        }
                        Err(error) => return Err(error),
                    }
                }
                ReplaceAllTraversalWork::FinishCall {
                    original_head,
                    original_args,
                    rewrite_args,
                } => {
                    let rewritten_child_count =
                        1 + rewrite_args.iter().filter(|rewrite| **rewrite).count();
                    let first_child = state
                        .completed
                        .len()
                        .checked_sub(rewritten_child_count)
                        .expect("ReplaceAll traversal results remain balanced");
                    let mut children = state.completed.split_off(first_child).into_iter();
                    let head_outcome = children
                        .next()
                        .expect("ReplaceAll call traversal completes its head");
                    let mut changed = head_outcome.changed;
                    let head = head_outcome.expr;
                    let mut rewritten_args = Vec::with_capacity(original_args.len());
                    for (index, original_arg) in original_args.iter().enumerate() {
                        if rewrite_args[index] {
                            let outcome = children
                                .next()
                                .expect("ReplaceAll call traversal completes each selected arg");
                            changed |= outcome.changed;
                            if outcome.changed {
                                self.append_changed_rewrite_argument(
                                    &head,
                                    &mut rewritten_args,
                                    outcome.expr,
                                );
                            } else {
                                rewritten_args.push(outcome.expr);
                            }
                        } else {
                            rewritten_args.push(original_arg.clone());
                        }
                    }
                    debug_assert!(children.next().is_none());
                    let expr = if changed {
                        Expr::call(head, rewritten_args)
                    } else {
                        Expr::call_shared(original_head, original_args)
                    };
                    state
                        .completed
                        .push(ReplaceAllTraversalOutcome { expr, changed });
                }
            }
        }

        debug_assert_eq!(state.completed.len(), 1);
        Ok(state
            .completed
            .pop()
            .expect("ReplaceAll traversal completes its root"))
    }

    fn resume_replace_all_traversal(
        &mut self,
        mut state: ReplaceAllTraversalState,
        expr: Expr,
        replaced: Option<Expr>,
    ) -> Result<ReplaceAllTraversalOutcome, EvalError> {
        self.finish_replace_all_visit(&mut state, expr, replaced);
        self.drive_replace_all_traversal(state)
    }

    fn finish_replace_all_visit(
        &mut self,
        state: &mut ReplaceAllTraversalState,
        expr: Expr,
        replaced: Option<Expr>,
    ) {
        if let Some(replaced) = replaced {
            state.completed.push(ReplaceAllTraversalOutcome {
                expr: replaced,
                changed: true,
            });
            return;
        }

        let Expr::Call { head, args, .. } = expr else {
            state.completed.push(ReplaceAllTraversalOutcome {
                expr,
                changed: false,
            });
            return;
        };

        let hold_attribute = if state.source == RuleApplicationSource::ReplaceAllUnheld {
            self.replace_all_unheld_hold_attribute_for_head(&head)
        } else {
            HoldAttribute::None
        };
        let rewrite_args = (0..args.len())
            .map(|index| {
                state.source != RuleApplicationSource::ReplaceAllUnheld
                    || !hold_attribute.holds_argument(index)
            })
            .collect::<Vec<_>>();

        state.work.push(ReplaceAllTraversalWork::FinishCall {
            original_head: head.clone(),
            original_args: args.clone(),
            rewrite_args: rewrite_args.clone(),
        });
        for (index, arg) in args.iter().enumerate().rev() {
            if rewrite_args[index] {
                state.work.push(ReplaceAllTraversalWork::Visit(arg.clone()));
            }
        }
        state
            .work
            .push(ReplaceAllTraversalWork::Visit(head.into_owned()));
    }

    fn replace_all_unheld_hold_attribute_for_head(&mut self, head: &Expr) -> HoldAttribute {
        let mut attributes = self.cached_attribute_set_for_head(head);

        if expr_as_symbol(head).is_none() {
            if let Some(symbol) = base_head_symbol_of_nested_call(head) {
                if self
                    .cached_attribute_set_for_symbol(symbol)
                    .contains(Attribute::SubValuesHoldAll)
                {
                    attributes = attributes.with(Attribute::HoldAll);
                }
            }
        }

        attributes.hold_attribute()
    }

    pub(in crate::runtime) fn append_changed_rewrite_argument(
        &self,
        parent_head: &Expr,
        rebuilt_args: &mut Vec<Expr>,
        rewritten: Expr,
    ) -> bool {
        match rewritten {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, heads::functions::SEQUENCE)
                    && !self.attribute_set_for_head(parent_head).sequence_hold() =>
            {
                rebuilt_args.extend(args);
                true
            }
            other => {
                rebuilt_args.push(other);
                false
            }
        }
    }

    fn apply_rules_once_from_list(
        &mut self,
        expr: Expr,
        normalized_expr: Expr,
        rules: RewriteRulePlan,
        source: RuleApplicationSource,
        mut cursor: RewriteRuleScanCursor,
    ) -> Result<Option<Expr>, EvalError> {
        loop {
            let Some(index) = rules.next_rule_index(&mut cursor) else {
                return Ok(None);
            };
            let Some(rule) = rules.get(index) else {
                return Ok(None);
            };
            let would_reject = !self.definition_lhs_could_match_candidate(
                &rule.lhs,
                &rule.matcher,
                &normalized_expr,
            );
            record_profile_rewrite_rule_prefilter_candidate(would_reject);
            if would_reject {
                continue;
            }
            let matcher = rule.matcher.clone();

            let match_result = self.match_compiled_owned(
                matcher,
                normalized_expr.clone(),
                SymbolExprMap::default(),
            );

            let outcome = match match_result {
                Ok(outcome) => outcome,
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedRewriteRuleMatch {
                        expr,
                        normalized_expr,
                        rules,
                    };
                    crate::resume_or_suspend!(insert self, ApplyRulesOnceFromListSuspensionFrame { state, source, cursor, index, would_reject });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            };
            record_profile_rewrite_rule_match_outcome(would_reject, outcome.matched);

            if !outcome.matched {
                continue;
            }

            let rule = rules
                .get(index)
                .expect("rewrite rule should still exist after match");
            let replaced = substitute_rule_bindings_for_rhs(
                &rule.rhs,
                &outcome.bindings,
                rule.rhs_may_splice_without_bindings,
            );
            self.push_rule_applied_trace(source, &expr, &rule.lhs, &replaced);
            return Ok(Some(replaced));
        }
    }

    fn finish_suspended_rewrite_rule_match(
        &mut self,
        state: SuspendedRewriteRuleMatch,
        source: RuleApplicationSource,
        cursor: RewriteRuleScanCursor,
        index: usize,
        would_reject: bool,
        outcome: CompiledMatchOutcome,
    ) -> Result<Option<Expr>, EvalError> {
        record_profile_rewrite_rule_match_outcome(would_reject, outcome.matched);
        if !outcome.matched {
            return self.apply_rules_once_from_list(
                state.expr,
                state.normalized_expr,
                state.rules,
                source,
                cursor,
            );
        }

        let rule = state
            .rules
            .get(index)
            .expect("rewrite rule should still exist for resumed match");
        let replaced = substitute_rule_bindings_for_rhs(
            &rule.rhs,
            &outcome.bindings,
            rule.rhs_may_splice_without_bindings,
        );
        self.push_rule_applied_trace(source, &state.expr, &rule.lhs, &replaced);
        Ok(Some(replaced))
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ReplaceAllTraversalOutcome>();
    assert_traces::<ReplaceAllTraversalWork>();
    assert_traces::<ReplaceAllTraversalState>();
    assert_traces::<SuspendedRewriteRuleMatch>();
};
