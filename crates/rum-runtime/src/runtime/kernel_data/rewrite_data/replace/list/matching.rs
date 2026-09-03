mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplaceListCallMatch {
    matcher_args: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    flat_head: Expr,
    orderless: bool,
    flat: bool,
}

impl SuspendedReplaceListCallMatch {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationReplaceListCallMatch"),
            vec![
                matcher_args_into_continuation_expr(self.matcher_args.into_vec()),
                self.value_args.into_vec().into_continuation_expr(),
                self.flat_head,
                self.orderless.into_continuation_expr(),
                self.flat.into_continuation_expr(),
            ],
        )
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplaceListGuardFilter {
    guards: SharedArgs,
    bindings: PersistentRewriteQueue<SymbolExprMap>,
    option_scope: Option<FrozenRewriteValue<OptionValueScope>>,
    accepted: PersistentMatchResults,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplaceListRuleResults {
    scan: SuspendedReplaceListScan,
    bindings: PersistentRewriteQueue<SymbolExprMap>,
}

impl Session {
    pub(super) fn collect_replace_list_rule_bindings_owned(
        &mut self,
        matcher: MatcherNode,
        normalized_target: Expr,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        match matcher {
            MatcherNode::DefinitionCondition(matcher) => {
                let DefinitionConditionMatcher { inner, lhs, guards } = *matcher;
                let option_scope = self.option_value_scope_for_definition(&lhs, &normalized_target);
                let bindings_result =
                    self.collect_replace_list_rule_bindings_owned(inner, normalized_target);
                match bindings_result {
                    Ok(bindings) => self.filter_replace_list_guard_bindings_owned(
                        guards,
                        bindings,
                        option_scope,
                        Vec::new(),
                        0,
                        None,
                    ),
                    Err(EvalError::InputRequired { request }) => {
                        let guards = SharedArgs::new(guards.into_vec());
                        let option_scope = option_scope.map(FrozenRewriteValue::freeze);
                        crate::resume_or_suspend!(insert self, CollectReplaceListRuleBindingsOwnedSuspensionFrame1 { guards, option_scope });
                        crate::resume_or_suspend!(insert self, ContinueReplaceListInitialGuardFilterFrame);
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
            MatcherNode::Call { head, args } => {
                let one_identity = matcher_head_has_attribute(&head, self, Attribute::OneIdentity);
                let orderless = matcher_head_has_attribute(&head, self, Attribute::Orderless);
                let flat = matcher_head_has_attribute(&head, self, Attribute::Flat);

                match normalized_target {
                    Expr::Call {
                        head: value_head,
                        args: value_args,
                        ..
                    } => {
                        let flat_head = value_head.as_ref().clone();
                        let value_head = value_head.into_owned();
                        let value_args = value_args.into_vec();
                        let head_result =
                            self.match_compiled_owned(*head, value_head, SymbolExprMap::default());
                        match head_result {
                            Ok(outcome) => self.finish_replace_list_call_head_match(
                                args, value_args, flat_head, orderless, flat, outcome,
                            ),
                            Err(EvalError::InputRequired { request }) => {
                                let state = SuspendedReplaceListCallMatch {
                                    matcher_args: FrozenMatcherSequence::freeze(args),
                                    value_args: SharedArgs::new(value_args),
                                    flat_head,
                                    orderless,
                                    flat,
                                };
                                crate::resume_or_suspend!(insert self, CollectReplaceListRuleBindingsOwnedSuspensionFrame2 { state });
                                crate::resume_or_suspend!(insert self, ContinueReplaceListCallMatchFrame);
                                Err(EvalError::InputRequired { request })
                            }
                            Err(error) => Err(error),
                        }
                    }
                    other if one_identity => {
                        if orderless {
                            self.collect_compiled_orderless_arg_matches_owned(
                                args,
                                vec![other],
                                SymbolExprMap::default(),
                            )
                        } else {
                            self.collect_compiled_args_matches_owned(
                                args,
                                vec![other],
                                SymbolExprMap::default(),
                            )
                        }
                    }
                    _ => Ok(Vec::new()),
                }
            }
            other => {
                let match_result =
                    self.match_compiled_owned(other, normalized_target, SymbolExprMap::default());
                crate::resume_or_suspend!(
                    self,
                    match_result,
                    ok(outcome) => { (|| {
                                    Ok(if outcome.matched {
                                        vec![outcome.bindings]
                                    } else {
                                        Vec::new()
                                    })
                                })() },
                    suspend => CollectReplaceListRuleBindingsOwnedSuspensionFrame3,
                )
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_replace_list_call_head_match(
        &mut self,
        args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        flat_head: Expr,
        orderless: bool,
        flat: bool,
        outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if !outcome.matched {
            return Ok(Vec::new());
        }

        if orderless {
            if flat {
                self.collect_compiled_orderless_arg_matches_with_flat_head_owned(
                    args,
                    value_args,
                    outcome.bindings,
                    flat_head,
                )
            } else {
                self.collect_compiled_orderless_arg_matches_owned(
                    args,
                    value_args,
                    outcome.bindings,
                )
            }
        } else if flat {
            self.collect_compiled_args_matches_with_flat_head_owned(
                args,
                value_args,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.collect_compiled_args_matches_owned(args, value_args, outcome.bindings)
        }
    }

    fn filter_replace_list_guard_bindings_owned(
        &mut self,
        guards: Box<[Expr]>,
        bindings: Vec<SymbolExprMap>,
        option_scope: Option<OptionValueScope>,
        mut accepted: Vec<SymbolExprMap>,
        mut index: usize,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                if outcome.matched {
                    accepted.push(outcome.bindings);
                }
                index += 1;
            }

            let Some(current) = bindings.get(index).cloned() else {
                return Ok(accepted);
            };
            let false_bindings = current.clone();
            let guard_result = self.evaluate_guard_sequence_owned(
                guards.clone(),
                current,
                false_bindings,
                option_scope.clone(),
            );

            match guard_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    let remaining = bindings.into_iter().skip(index + 1).collect();
                    let state = SuspendedReplaceListGuardFilter {
                        guards: SharedArgs::new(guards.into_vec()),
                        bindings: PersistentRewriteQueue::freeze(remaining),
                        option_scope: option_scope.map(FrozenRewriteValue::freeze),
                        accepted: PersistentMatchResults::freeze(accepted),
                    };
                    crate::resume_or_suspend!(insert self, FilterReplaceListGuardBindingsOwnedSuspensionFrame { state });
                    crate::resume_or_suspend!(insert self, ContinueReplaceListGuardFilterFrame);
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(super) fn collect_replace_list_rule_results_owned(
        &mut self,
        target: Expr,
        normalized_target: Expr,
        rules: RewriteRulePlan,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        rule_index: usize,
        bindings: Vec<SymbolExprMap>,
    ) -> Result<Vec<Expr>, EvalError> {
        if max_results.is_some_and(|limit| collected.len() >= limit) {
            return Ok(collected);
        }

        let mut bindings = bindings.into_iter();
        let Some(current_bindings) = bindings.next() else {
            return self.collect_replace_list_results_owned(
                target,
                normalized_target,
                rules,
                max_results,
                collected,
                rule_index + 1,
            );
        };
        let remaining_bindings = bindings.collect::<Vec<_>>();
        let rule = rules
            .get(rule_index)
            .expect("ReplaceList rule remains in its immutable plan");

        let replaced = substitute_rule_bindings_for_rhs(
            &rule.rhs,
            &current_bindings,
            rule.rhs_may_splice_without_bindings,
        );
        self.push_rule_applied_trace(
            RuleApplicationSource::ReplaceList,
            &target,
            &rule.lhs,
            &replaced,
        );
        let replaced_result = self.evaluate(replaced);
        match replaced_result {
            Ok(value) => {
                let mut collected = collected;
                collected.push(value);
                self.collect_replace_list_rule_results_owned(
                    target,
                    normalized_target,
                    rules,
                    max_results,
                    collected,
                    rule_index,
                    remaining_bindings,
                )
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedReplaceListRuleResults {
                    scan: SuspendedReplaceListScan {
                        target,
                        normalized_target,
                        rules,
                        max_results,
                        collected: PersistentRewriteExprs::freeze(collected),
                        index: rule_index,
                    },
                    bindings: PersistentRewriteQueue::freeze(remaining_bindings),
                };
                let next = ContinueReplaceListRuleResultsFrame {
                    target: state.scan.target.clone(),
                    normalized_target: state.scan.normalized_target.clone(),
                    rules: state.scan.rules.clone(),
                    max_results: state.scan.max_results,
                    rule_index: state.scan.index,
                };
                crate::resume_or_suspend!(insert self, CollectReplaceListRuleResultsOwnedSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, next);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedReplaceListCallMatch>();
    assert_traces::<SuspendedReplaceListGuardFilter>();
    assert_traces::<SuspendedReplaceListRuleResults>();
};
