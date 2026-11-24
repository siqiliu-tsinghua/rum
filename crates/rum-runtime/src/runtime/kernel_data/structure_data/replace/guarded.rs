mod resume_frames;
use resume_frames::*;

use super::guarded_plan::{
    activate_guarded_replace_part_pattern_target, append_guarded_replace_part_exact_targets,
    append_guarded_replace_part_pattern_candidates, build_guarded_replace_plan_from_targets,
};
use super::plan::{
    replace_part_pattern_includes_heads, replace_part_position_expr,
    wrap_replace_part_pattern_position,
};
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
struct SuspendedGuardedReplacePartPlanning {
    expr: Expr,
    rules: FrozenReplaceSequence<ReplacePartRule>,
    targets: SuspendedGuardedTargetList,
    rule_index: usize,
    position_spec_index: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedGuardedReplacePartPatternMatch {
    planning: SuspendedGuardedReplacePartPlanning,
    pattern: Expr,
    range_end: usize,
    index: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedGuardedReplacePartSelection {
    expr: Expr,
    rules: FrozenReplaceSequence<ReplacePartRule>,
    targets: SuspendedGuardedTargetList,
    remaining: usize,
}

impl Session {
    pub(super) fn apply_guarded_replace_part_rules_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
    ) -> Result<Expr, EvalError> {
        debug_assert!(rules.iter().any(|rule| !rule.guards.is_empty()));
        self.plan_guarded_replace_part_targets_owned(expr, rules, Vec::new(), 0, 0)
    }

    fn plan_guarded_replace_part_targets_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut targets: Vec<GuardedReplacePartTarget>,
        mut rule_index: usize,
        mut position_spec_index: usize,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(rule) = rules.get(rule_index) else {
                targets
                    .retain(|target| target.state == GuardedReplacePartTargetState::PendingGuard);
                let remaining = targets.len();
                return (if crate::cm740_probe::driven() {
                    self.select_guarded_replace_part_targets_owned_cm740_driven(
                        expr, rules, targets, remaining, None,
                    )
                } else {
                    self.select_guarded_replace_part_targets_owned(
                        expr, rules, targets, remaining, None,
                    )
                });
            };

            let Some(position_spec) = rule.position_specs.get(position_spec_index).cloned() else {
                rule_index += 1;
                position_spec_index = 0;
                continue;
            };

            match position_spec {
                ReplacePartPositionSpec::Exact(selectors) => {
                    append_guarded_replace_part_exact_targets(
                        &expr,
                        &selectors,
                        rule.heads,
                        rule_index,
                        &mut targets,
                    );
                    position_spec_index += 1;
                }
                ReplacePartPositionSpec::Pattern(pattern_expr) => {
                    let pattern_result = (if crate::cm740_probe::driven() {
                        self.resolve_matcher_pattern_expr_owned_cm740_driven(pattern_expr)
                    } else {
                        self.resolve_matcher_pattern_expr_owned(pattern_expr)
                    });
                    return match pattern_result {
                        Ok(pattern) => self.plan_guarded_replace_part_pattern_targets_owned(
                            expr,
                            rules,
                            targets,
                            rule_index,
                            position_spec_index,
                            pattern,
                        ),
                        Err(EvalError::InputRequired { request }) => {
                            let state = SuspendedGuardedReplacePartPlanning {
                                expr,
                                rules: FrozenReplaceSequence::freeze(rules),
                                targets: SuspendedGuardedTargetList::freeze(targets),
                                rule_index,
                                position_spec_index,
                            };
                            crate::resume_or_suspend!(insert self, PlanGuardedReplacePartTargetsOwnedSuspensionFrame { state });
                            Err(EvalError::InputRequired { request })
                        }
                        Err(error) => Err(error),
                    };
                }
            }
        }
    }

    fn plan_guarded_replace_part_pattern_targets_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut targets: Vec<GuardedReplacePartTarget>,
        rule_index: usize,
        position_spec_index: usize,
        resolved_pattern: Expr,
    ) -> Result<Expr, EvalError> {
        let pattern = wrap_replace_part_pattern_position(resolved_pattern);
        let heads = rules[rule_index].heads;
        let include_heads = replace_part_pattern_includes_heads(&pattern, heads);
        let range = append_guarded_replace_part_pattern_candidates(
            &expr,
            include_heads,
            rule_index,
            &mut targets,
        );

        self.match_guarded_replace_part_pattern_targets_owned(
            expr,
            rules,
            targets,
            rule_index,
            position_spec_index,
            pattern,
            range.end,
            range.start,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn match_guarded_replace_part_pattern_targets_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut targets: Vec<GuardedReplacePartTarget>,
        rule_index: usize,
        position_spec_index: usize,
        pattern: Expr,
        range_end: usize,
        mut index: usize,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Expr, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                if outcome.matched {
                    let position = replace_part_position_expr(&targets[index].path);
                    let option_scope = self.option_value_scope_for_definition(&pattern, &position);
                    activate_guarded_replace_part_pattern_target(
                        &mut targets,
                        index,
                        outcome.bindings,
                        option_scope,
                    );
                } else {
                    targets[index].state = GuardedReplacePartTargetState::Inactive;
                }
                index += 1;
            }

            if index == range_end {
                return self.plan_guarded_replace_part_targets_owned(
                    expr,
                    rules,
                    targets,
                    rule_index,
                    position_spec_index + 1,
                );
            }

            let position = replace_part_position_expr(&targets[index].path);
            let match_result = (if crate::cm740_probe::driven() {
                self.match_expr_owned_cm740_driven(
                    pattern.clone(),
                    position,
                    SymbolExprMap::default(),
                )
            } else {
                self.match_expr_owned(pattern.clone(), position, SymbolExprMap::default())
            });
            match match_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedGuardedReplacePartPatternMatch {
                        planning: SuspendedGuardedReplacePartPlanning {
                            expr,
                            rules: FrozenReplaceSequence::freeze(rules),
                            targets: SuspendedGuardedTargetList::freeze(targets),
                            rule_index,
                            position_spec_index,
                        },
                        pattern,
                        range_end,
                        index,
                    };
                    crate::resume_or_suspend!(insert self, MatchGuardedReplacePartPatternTargetsOwnedSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn select_guarded_replace_part_targets_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut targets: Vec<GuardedReplacePartTarget>,
        mut remaining: usize,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Expr, EvalError> {
        // Later rules have priority at a contested position. Walking the
        // rule-major target vector backward lets an accepting later rule claim
        // the position before any earlier guard at that position is run. A
        // candidate advances only after its one guard outcome is recorded, so
        // suspension cannot replay it and duplicate position specs cannot run
        // it twice.
        loop {
            if let Some(outcome) = pending.take() {
                let index = remaining - 1;
                targets[index].state = if outcome.matched {
                    GuardedReplacePartTargetState::Accepted
                } else {
                    GuardedReplacePartTargetState::Inactive
                };
                remaining -= 1;
                continue;
            }

            if remaining == 0 {
                let plan = build_guarded_replace_plan_from_targets(targets, &rules);
                return self.apply_replace_plan_owned(expr, plan);
            }

            let index = remaining - 1;
            let claimed_by_later_rule = targets[index + 1..].iter().any(|later| {
                later.state == GuardedReplacePartTargetState::Accepted
                    && later.path == targets[index].path
            });
            if claimed_by_later_rule {
                targets[index].state = GuardedReplacePartTargetState::Inactive;
                remaining -= 1;
                continue;
            }

            let target = &targets[index];
            let rule = &rules[target.rule_index];
            if rule.guards.is_empty() {
                targets[index].state = GuardedReplacePartTargetState::Accepted;
                remaining -= 1;
                continue;
            }

            let guard_result = self.evaluate_guard_sequence_owned(
                rule.guards.clone(),
                target.bindings.clone(),
                target.bindings.clone(),
                target.option_scope.clone(),
            );
            match guard_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    return self.suspend_guarded_replace_part_selection_owned(
                        expr, rules, targets, remaining, request,
                    );
                }
                Err(error) => return Err(error),
            }
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn select_guarded_replace_part_targets_owned_cm740_driven(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        mut targets: Vec<GuardedReplacePartTarget>,
        mut remaining: usize,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Expr, EvalError> {
        // Later rules have priority at a contested position. Walking the
        // rule-major target vector backward lets an accepting later rule claim
        // the position before any earlier guard at that position is run. A
        // candidate advances only after its one guard outcome is recorded, so
        // suspension cannot replay it and duplicate position specs cannot run
        // it twice.
        loop {
            if let Some(outcome) = pending.take() {
                let index = remaining - 1;
                targets[index].state = if outcome.matched {
                    GuardedReplacePartTargetState::Accepted
                } else {
                    GuardedReplacePartTargetState::Inactive
                };
                remaining -= 1;
                continue;
            }

            if remaining == 0 {
                let plan = build_guarded_replace_plan_from_targets(targets, &rules);
                return self.apply_replace_plan_owned(expr, plan);
            }

            let index = remaining - 1;
            let claimed_by_later_rule = targets[index + 1..].iter().any(|later| {
                later.state == GuardedReplacePartTargetState::Accepted
                    && later.path == targets[index].path
            });
            if claimed_by_later_rule {
                targets[index].state = GuardedReplacePartTargetState::Inactive;
                remaining -= 1;
                continue;
            }

            let target = &targets[index];
            let rule = &rules[target.rule_index];
            if rule.guards.is_empty() {
                targets[index].state = GuardedReplacePartTargetState::Accepted;
                remaining -= 1;
                continue;
            }

            let guard_result = crate::cm740_probe::around("P1070", || {
                self.evaluate_guard_sequence_owned_cm740_driven(
                    rule.guards.clone(),
                    target.bindings.clone(),
                    target.bindings.clone(),
                    target.option_scope.clone(),
                )
            });
            match guard_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    return self.suspend_guarded_replace_part_selection_owned(
                        expr, rules, targets, remaining, request,
                    );
                }
                Err(error) => return Err(error),
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn suspend_guarded_replace_part_selection_owned(
        &mut self,
        expr: Expr,
        rules: Vec<ReplacePartRule>,
        targets: Vec<GuardedReplacePartTarget>,
        remaining: usize,
        request: RuntimeInputRequest,
    ) -> Result<Expr, EvalError> {
        let state = SuspendedGuardedReplacePartSelection {
            expr,
            rules: FrozenReplaceSequence::freeze(rules),
            targets: SuspendedGuardedTargetList::freeze(targets),
            remaining,
        };
        crate::resume_or_suspend!(insert self, SelectGuardedReplacePartTargetsOwnedSuspensionFrame { state });
        Err(EvalError::InputRequired { request })
    }

    fn plan_guarded_replace_part_targets_suspended(
        &mut self,
        mut state: SuspendedGuardedReplacePartPlanning,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(rule) = state.rules.as_slice().get(state.rule_index) else {
                let remaining = state.targets.len();
                return (if crate::cm740_probe::driven() {
                    self.select_guarded_replace_part_targets_suspended_cm740_driven(
                        SuspendedGuardedReplacePartSelection {
                            expr: state.expr,
                            rules: state.rules,
                            targets: state.targets,
                            remaining,
                        },
                        None,
                    )
                } else {
                    self.select_guarded_replace_part_targets_suspended(
                        SuspendedGuardedReplacePartSelection {
                            expr: state.expr,
                            rules: state.rules,
                            targets: state.targets,
                            remaining,
                        },
                        None,
                    )
                });
            };
            let Some(position_spec) = rule.position_specs.get(state.position_spec_index).cloned()
            else {
                state.rule_index += 1;
                state.position_spec_index = 0;
                continue;
            };

            match position_spec {
                ReplacePartPositionSpec::Exact(selectors) => {
                    let mut appended = Vec::new();
                    append_guarded_replace_part_exact_targets(
                        &state.expr,
                        &selectors,
                        rule.heads,
                        state.rule_index,
                        &mut appended,
                    );
                    state.targets.append_exact_targets(appended);
                    state.position_spec_index += 1;
                }
                ReplacePartPositionSpec::Pattern(pattern_expr) => {
                    let pattern_result = (if crate::cm740_probe::driven() {
                        self.resolve_matcher_pattern_expr_owned_cm740_driven(pattern_expr)
                    } else {
                        self.resolve_matcher_pattern_expr_owned(pattern_expr)
                    });
                    return match pattern_result {
                        Ok(pattern) => {
                            self.plan_guarded_replace_part_pattern_targets_suspended(state, pattern)
                        }
                        Err(EvalError::InputRequired { request }) => {
                            crate::resume_or_suspend!(insert self, PlanGuardedReplacePartTargetsSuspendedSuspensionFrame { state });
                            Err(EvalError::InputRequired { request })
                        }
                        Err(error) => Err(error),
                    };
                }
            }
        }
    }

    fn plan_guarded_replace_part_pattern_targets_suspended(
        &mut self,
        mut state: SuspendedGuardedReplacePartPlanning,
        resolved_pattern: Expr,
    ) -> Result<Expr, EvalError> {
        let pattern = wrap_replace_part_pattern_position(resolved_pattern);
        let heads = state.rules.as_slice()[state.rule_index].heads;
        let include_heads = replace_part_pattern_includes_heads(&pattern, heads);
        let mut appended = Vec::new();
        append_guarded_replace_part_pattern_candidates(
            &state.expr,
            include_heads,
            state.rule_index,
            &mut appended,
        );
        let range = state.targets.append_pattern_candidates(appended);
        self.match_guarded_replace_part_pattern_targets_suspended(
            SuspendedGuardedReplacePartPatternMatch {
                planning: state,
                pattern,
                range_end: range.end,
                index: range.start,
            },
            None,
        )
    }

    fn match_guarded_replace_part_pattern_targets_suspended(
        &mut self,
        mut state: SuspendedGuardedReplacePartPatternMatch,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Expr, EvalError> {
        loop {
            if let Some(outcome) = pending.take() {
                if outcome.matched {
                    let position =
                        replace_part_position_expr(state.planning.targets.path(state.index));
                    let option_scope =
                        self.option_value_scope_for_definition(&state.pattern, &position);
                    state.planning.targets.activate_pattern_target(
                        state.index,
                        outcome.bindings,
                        option_scope,
                    );
                } else {
                    state
                        .planning
                        .targets
                        .set_state(state.index, GuardedReplacePartTargetState::Inactive);
                }
                state.index += 1;
            }

            if state.index == state.range_end {
                state.planning.position_spec_index += 1;
                return self.plan_guarded_replace_part_targets_suspended(state.planning);
            }

            let position = replace_part_position_expr(state.planning.targets.path(state.index));
            let match_result = (if crate::cm740_probe::driven() {
                self.match_expr_owned_cm740_driven(
                    state.pattern.clone(),
                    position,
                    SymbolExprMap::default(),
                )
            } else {
                self.match_expr_owned(state.pattern.clone(), position, SymbolExprMap::default())
            });
            match match_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, MatchGuardedReplacePartPatternTargetsSuspendedSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn select_guarded_replace_part_targets_suspended(
        &mut self,
        mut state: SuspendedGuardedReplacePartSelection,
        mut pending: Option<bool>,
    ) -> Result<Expr, EvalError> {
        loop {
            if let Some(matched) = pending.take() {
                let index = state.remaining - 1;
                state.targets.set_state(
                    index,
                    if matched {
                        GuardedReplacePartTargetState::Accepted
                    } else {
                        GuardedReplacePartTargetState::Inactive
                    },
                );
                state.remaining -= 1;
                continue;
            }

            if state.remaining == 0 {
                let plan = state.targets.build_plan(state.rules.as_slice());
                return self.apply_replace_plan_owned(state.expr, plan);
            }
            let index = state.remaining - 1;
            if state.targets.state(index) != GuardedReplacePartTargetState::PendingGuard {
                state.remaining -= 1;
                continue;
            }
            let claimed_by_later_rule = (index + 1..state.targets.len()).any(|later| {
                state.targets.state(later) == GuardedReplacePartTargetState::Accepted
                    && state.targets.path(later) == state.targets.path(index)
            });
            if claimed_by_later_rule {
                state
                    .targets
                    .set_state(index, GuardedReplacePartTargetState::Inactive);
                state.remaining -= 1;
                continue;
            }

            let rule_index = state.targets.rule_index(index);
            let rule = &state.rules.as_slice()[rule_index];
            if rule.guards.is_empty() {
                state
                    .targets
                    .set_state(index, GuardedReplacePartTargetState::Accepted);
                state.remaining -= 1;
                continue;
            }
            let guards = SharedArgs::new(rule.guards.to_vec());
            let bindings = state.targets.frozen_bindings(index);
            let option_scope = state.targets.option_scope(index);
            let guard_result =
                self.evaluate_guard_sequence_readonly_suspended(guards, bindings, option_scope);
            match guard_result {
                Ok(matched) => pending = Some(matched),
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, SelectGuardedReplacePartTargetsSuspendedSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn select_guarded_replace_part_targets_suspended_cm740_driven(
        &mut self,
        mut state: SuspendedGuardedReplacePartSelection,
        mut pending: Option<bool>,
    ) -> Result<Expr, EvalError> {
        loop {
            if let Some(matched) = pending.take() {
                let index = state.remaining - 1;
                if crate::cm740_probe::frame("P1070") {
                    crate::cm740_probe::hit("P1070", "OmitActualGuardedTargetAcceptance");
                } else {
                    state.targets.set_state(
                        index,
                        if matched {
                            GuardedReplacePartTargetState::Accepted
                        } else {
                            GuardedReplacePartTargetState::Inactive
                        },
                    );
                }
                state.remaining -= 1;
                continue;
            }

            if state.remaining == 0 {
                let plan = state.targets.build_plan(state.rules.as_slice());
                return self.apply_replace_plan_owned(state.expr, plan);
            }
            let index = state.remaining - 1;
            if state.targets.state(index) != GuardedReplacePartTargetState::PendingGuard {
                state.remaining -= 1;
                continue;
            }
            let claimed_by_later_rule = (index + 1..state.targets.len()).any(|later| {
                state.targets.state(later) == GuardedReplacePartTargetState::Accepted
                    && state.targets.path(later) == state.targets.path(index)
            });
            if claimed_by_later_rule {
                state
                    .targets
                    .set_state(index, GuardedReplacePartTargetState::Inactive);
                state.remaining -= 1;
                continue;
            }

            let rule_index = state.targets.rule_index(index);
            let rule = &state.rules.as_slice()[rule_index];
            if rule.guards.is_empty() {
                state
                    .targets
                    .set_state(index, GuardedReplacePartTargetState::Accepted);
                state.remaining -= 1;
                continue;
            }
            let guards = SharedArgs::new(rule.guards.to_vec());
            let bindings = state.targets.frozen_bindings(index);
            let option_scope = state.targets.option_scope(index);
            let guard_result = self.evaluate_guard_sequence_readonly_suspended_cm740_driven(
                guards,
                bindings,
                option_scope,
            );
            match guard_result {
                Ok(matched) => pending = Some(matched),
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, SelectGuardedReplacePartTargetsSuspendedSuspensionFrame { state });
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
    assert_traces::<SuspendedGuardedReplacePartPatternMatch>();
    assert_traces::<SuspendedGuardedReplacePartPlanning>();
    assert_traces::<SuspendedGuardedReplacePartSelection>();
};
