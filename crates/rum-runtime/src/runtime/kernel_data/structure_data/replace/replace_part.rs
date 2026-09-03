mod resume_frames;
use resume_frames::*;

use super::parsing::{parse_replace_part_heads_option, parse_replace_part_rules};
use super::plan::{
    build_replace_plan_from_targets, collect_replace_part_candidate_paths,
    merge_replace_part_rule_into_plan, replace_part_pattern_includes_heads,
    replace_part_position_expr, replace_part_rule_is_merge_safe,
    wrap_replace_part_pattern_position,
};
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct SuspendedReplacePartTargetsCompletion(PersistentReplacePartTargets);

impl SuspendedReplacePartTargetsCompletion {
    fn finish(self) -> Vec<ReplacePartTarget> {
        self.0.into_vec()
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplacePartRules {
    expr: Expr,
    rules: FrozenReplaceSequence<ReplacePartRule>,
    rule_index: usize,
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedReplacePartPatternCandidates>();
    assert_traces::<SuspendedReplacePartRuleTargets>();
    assert_traces::<SuspendedReplacePartRules>();
};

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplacePartRuleTargets {
    expr: Expr,
    position_specs: FrozenReplaceSequence<ReplacePartPositionSpec>,
    rhs: Expr,
    delayed: bool,
    heads: ReplacePartHeadsMode,
    collected: PersistentReplacePartTargets,
    index: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplacePartPatternCandidates {
    pattern: Expr,
    candidates: FrozenReplaceSequence<Vec<isize>>,
    rhs: Expr,
    delayed: bool,
    collected: PersistentReplacePartTargets,
    index: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_replace_part(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ReplacePart", args, 2, 3));
        }

        let heads = if let Some(option_expr) = args.get(2) {
            let Some(heads) = parse_replace_part_heads_option(option_expr) else {
                return Ok(self.invalid_call_with_arg_form(
                    "ReplacePart",
                    args,
                    "expected a Heads -> True/False option as the third argument",
                ));
            };
            heads
        } else {
            ReplacePartHeadsMode::Automatic
        };

        let rules = match parse_replace_part_rules(&args[1], heads) {
            Ok(Some(rules)) => rules,
            Ok(None) => return Ok(Expr::call(head.clone(), args.to_vec())),
            Err(span) => {
                self.push_message(
                    "ReplacePart",
                    "span",
                    format!(
                        "{span} is not a valid Span specification for ReplacePart; expected a valid Part Span with integer, All, or UpTo endpoints and a nonzero integer step."
                    ),
                );
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
        };

        let has_guards = rules.iter().any(|rule| !rule.guards.is_empty());
        if has_guards {
            self.apply_guarded_replace_part_rules_owned(args[0].clone(), rules)
        } else {
            self.apply_replace_part_rules_owned(args[0].clone(), rules, 0, None)
        }
    }
    fn apply_replace_part_rules_owned(
        &mut self,
        mut expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut rule_index: usize,
        mut pending_targets: Option<Vec<ReplacePartTarget>>,
    ) -> Result<Expr, EvalError> {
        let mut merged_plan = ReplacePlanNode::default();

        loop {
            if let Some(targets) = pending_targets.take() {
                if !targets.is_empty() {
                    expr = self
                        .apply_replace_plan_owned(expr, build_replace_plan_from_targets(targets))?;
                }
                rule_index += 1;
                continue;
            }

            let Some(rule) = rules.get(rule_index) else {
                return if merged_plan.action.is_none() && merged_plan.children.is_empty() {
                    Ok(expr)
                } else {
                    self.apply_replace_plan_owned(expr, merged_plan)
                };
            };

            if replace_part_rule_is_merge_safe(rule) {
                merge_replace_part_rule_into_plan(&expr, &mut merged_plan, rule);
                rule_index += 1;
                continue;
            }

            if merged_plan.action.is_some() || !merged_plan.children.is_empty() {
                expr = self.apply_replace_plan_owned(expr, merged_plan)?;
                merged_plan = ReplacePlanNode::default();
            }

            let targets_result = self.resolve_rule_targets_owned(expr.clone(), rule.clone());
            match targets_result {
                Ok(targets) => pending_targets = Some(targets),
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedReplacePartRules {
                        expr,
                        rules: FrozenReplaceSequence::freeze(rules),
                        rule_index,
                    };
                    crate::resume_or_suspend!(insert self, ApplyReplacePartRulesOwnedSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resolve_rule_targets_owned(
        &mut self,
        expr: Expr,
        rule: ReplacePartRule,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        self.collect_rule_targets_owned(
            expr,
            rule.position_specs,
            rule.rhs,
            rule.delayed,
            rule.heads,
            Vec::new(),
            0,
        )
    }

    fn collect_rule_targets_owned(
        &mut self,
        expr: Expr,
        position_specs: Vec<ReplacePartPositionSpec>,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
        mut collected: Vec<ReplacePartTarget>,
        index: usize,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let Some(position_spec) = position_specs.get(index).cloned() else {
            return Ok(collected);
        };

        let targets_result = self.collect_position_spec_targets_owned(
            expr.clone(),
            position_spec,
            rhs.clone(),
            delayed,
            heads,
        );
        match targets_result {
            Ok(mut targets) => {
                collected.append(&mut targets);
                self.collect_rule_targets_owned(
                    expr,
                    position_specs,
                    rhs,
                    delayed,
                    heads,
                    collected,
                    index + 1,
                )
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedReplacePartRuleTargets {
                    expr,
                    position_specs: FrozenReplaceSequence::freeze(position_specs),
                    rhs,
                    delayed,
                    heads,
                    collected: PersistentReplacePartTargets::freeze(collected),
                    index,
                };
                crate::resume_or_suspend!(insert self, CollectRuleTargetsOwnedSuspensionFrame { state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn collect_position_spec_targets_owned(
        &mut self,
        expr: Expr,
        position_spec: ReplacePartPositionSpec,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        match position_spec {
            ReplacePartPositionSpec::Exact(selectors) => Ok(collect_exact_paths_for_position_spec(
                &expr, &selectors, heads,
            )
            .into_iter()
            .map(|path| ReplacePartTarget {
                path,
                rhs: rhs.clone(),
                delayed,
            })
            .collect()),
            ReplacePartPositionSpec::Pattern(pattern_expr) => {
                self.collect_pattern_position_targets_owned(expr, pattern_expr, rhs, delayed, heads)
            }
        }
    }

    fn collect_pattern_position_targets_owned(
        &mut self,
        expr: Expr,
        pattern_expr: Expr,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        match pattern_result {
            Ok(resolved_pattern) => self.finish_replace_part_pattern_resolution(
                expr,
                rhs,
                delayed,
                heads,
                resolved_pattern,
            ),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, CollectPatternPositionTargetsOwnedSuspensionFrame { expr, rhs, delayed, heads });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn collect_pattern_position_targets_from_candidates_owned(
        &mut self,
        pattern: Expr,
        candidates: Vec<Vec<isize>>,
        rhs: Expr,
        delayed: bool,
        mut collected: Vec<ReplacePartTarget>,
        index: usize,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let Some(path) = candidates.get(index).cloned() else {
            return Ok(collected);
        };

        let position = replace_part_position_expr(&path);
        let match_result =
            self.match_expr_owned(pattern.clone(), position, SymbolExprMap::default());
        match match_result {
            Ok(outcome) => {
                if outcome.matched {
                    collected.push(ReplacePartTarget {
                        path,
                        rhs: substitute_rule_bindings(&rhs, &outcome.bindings),
                        delayed,
                    });
                }
                self.collect_pattern_position_targets_from_candidates_owned(
                    pattern,
                    candidates,
                    rhs,
                    delayed,
                    collected,
                    index + 1,
                )
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedReplacePartPatternCandidates {
                    pattern,
                    candidates: FrozenReplaceSequence::freeze(candidates),
                    rhs,
                    delayed,
                    collected: PersistentReplacePartTargets::freeze(collected),
                    index,
                };
                crate::resume_or_suspend!(insert self, CollectPatternPositionTargetsFromCandidatesOwnedSuspensionFrame {
                    state,
                    path: std::sync::Arc::from(path),
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_replace_part_pattern_resolution(
        &mut self,
        expr: Expr,
        rhs: Expr,
        delayed: bool,
        heads: ReplacePartHeadsMode,
        resolved_pattern: Expr,
    ) -> Result<Vec<ReplacePartTarget>, EvalError> {
        let pattern = wrap_replace_part_pattern_position(resolved_pattern);
        let include_heads = replace_part_pattern_includes_heads(&pattern, heads);
        let candidates = collect_replace_part_candidate_paths(&expr, include_heads);
        self.collect_pattern_position_targets_from_candidates_owned(
            pattern,
            candidates,
            rhs,
            delayed,
            Vec::new(),
            0,
        )
    }

    fn resume_suspended_replace_part_rules(
        &mut self,
        mut state: SuspendedReplacePartRules,
        targets: Vec<ReplacePartTarget>,
    ) -> Result<Expr, EvalError> {
        if !targets.is_empty() {
            state.expr = self
                .apply_replace_plan_owned(state.expr, build_replace_plan_from_targets(targets))?;
        }
        state.rule_index += 1;
        let mut merged_plan = ReplacePlanNode::default();
        loop {
            let Some(rule) = state.rules.as_slice().get(state.rule_index) else {
                return if merged_plan.action.is_none() && merged_plan.children.is_empty() {
                    Ok(state.expr)
                } else {
                    self.apply_replace_plan_owned(state.expr, merged_plan)
                };
            };
            if replace_part_rule_is_merge_safe(rule) {
                merge_replace_part_rule_into_plan(&state.expr, &mut merged_plan, rule);
                state.rule_index += 1;
                continue;
            }
            if merged_plan.action.is_some() || !merged_plan.children.is_empty() {
                state.expr = self.apply_replace_plan_owned(state.expr, merged_plan)?;
            }

            let targets_result = self.resolve_rule_targets_owned(state.expr.clone(), rule.clone());
            match targets_result {
                Ok(targets) => {
                    if !targets.is_empty() {
                        state.expr = self.apply_replace_plan_owned(
                            state.expr,
                            build_replace_plan_from_targets(targets),
                        )?;
                    }
                    state.rule_index += 1;
                    merged_plan = ReplacePlanNode::default();
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedReplacePartRulesSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_replace_part_rule_targets(
        &mut self,
        mut state: SuspendedReplacePartRuleTargets,
        targets: Vec<ReplacePartTarget>,
    ) -> Result<SuspendedReplacePartTargetsCompletion, EvalError> {
        state.collected = state.collected.appended_values(targets);
        state.index += 1;
        loop {
            let Some(position_spec) = state.position_specs.as_slice().get(state.index).cloned()
            else {
                return Ok(SuspendedReplacePartTargetsCompletion(state.collected));
            };
            let targets_result = self.collect_position_spec_targets_owned(
                state.expr.clone(),
                position_spec,
                state.rhs.clone(),
                state.delayed,
                state.heads,
            );
            match targets_result {
                Ok(targets) => {
                    state.collected = state.collected.appended_values(targets);
                    state.index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedReplacePartRuleTargetsSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_replace_part_pattern_candidates(
        &mut self,
        mut state: SuspendedReplacePartPatternCandidates,
        path: std::sync::Arc<[isize]>,
        outcome: CompiledMatchOutcome,
    ) -> Result<SuspendedReplacePartTargetsCompletion, EvalError> {
        if outcome.matched {
            state.collected = state.collected.appended(ReplacePartTarget {
                path: path.as_ref().to_vec(),
                rhs: substitute_rule_bindings(&state.rhs, &outcome.bindings),
                delayed: state.delayed,
            });
        }
        state.index += 1;
        loop {
            let Some(path) = state.candidates.as_slice().get(state.index).cloned() else {
                return Ok(SuspendedReplacePartTargetsCompletion(state.collected));
            };
            let position = replace_part_position_expr(&path);
            let match_result =
                self.match_expr_owned(state.pattern.clone(), position, SymbolExprMap::default());
            match match_result {
                Ok(outcome) => {
                    if outcome.matched {
                        state.collected = state.collected.appended(ReplacePartTarget {
                            path,
                            rhs: substitute_rule_bindings(&state.rhs, &outcome.bindings),
                            delayed: state.delayed,
                        });
                    }
                    state.index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedReplacePartPatternCandidatesSuspensionFrame {
                        state,
                        path: std::sync::Arc::from(path),
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(super) fn apply_replace_plan_owned(
        &mut self,
        mut expr: Expr,
        mut plan: ReplacePlanNode,
    ) -> Result<Expr, EvalError> {
        if let Some(action) = plan.action.take() {
            expr = match action {
                ReplacePlanAction::Immediate(rhs) => rhs,
            };
        }

        if plan.children.is_empty() {
            return Ok(expr);
        }

        match expr {
            Expr::Atom(_) => {
                if let Some(head_plan) = plan.children.remove(&0) {
                    let replaced_head =
                        self.apply_replace_plan_owned(structural_head_expr(&expr), head_plan)?;
                    Ok(Expr::call(replaced_head, Vec::new()))
                } else {
                    Ok(expr)
                }
            }
            Expr::Call { head, args } => {
                let mut replaced_head = head.as_ref().clone();
                if let Some(head_plan) = plan.children.remove(&0) {
                    replaced_head = self.apply_replace_plan_owned(replaced_head, head_plan)?;
                }

                let mut args = Expr::unwrap_shared_args(args);
                for (index, arg) in args.iter_mut().enumerate() {
                    if let Some(child_plan) = plan.children.remove(&(index as isize + 1)) {
                        *arg = self.apply_replace_plan_owned(arg.clone(), child_plan)?;
                    }
                }

                Ok(Expr::call_with_shared_args(
                    replaced_head,
                    SharedArgs::new(args),
                ))
            }
        }
    }
}
