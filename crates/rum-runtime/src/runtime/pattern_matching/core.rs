mod resume_frames;
use resume_frames::*;

mod key_value;
use key_value::*;

use super::{outcome::*, *};
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

impl Session {
    #[allow(dead_code)]
    pub(in crate::runtime) fn match_expr(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut SymbolExprMap,
    ) -> Result<bool, EvalError> {
        let outcome = self.match_expr_owned(pattern.clone(), value.clone(), bindings.clone())?;
        *bindings = outcome.bindings;
        Ok(outcome.matched)
    }

    pub(in crate::runtime) fn match_expr_owned(
        &mut self,
        pattern: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let matcher = self.compile_matcher_for_expr(&pattern);
        let normalized_value = self.normalize_match_expr(&value);
        self.match_compiled_owned(matcher, normalized_value, bindings)
    }

    pub(in crate::runtime) fn match_compiled_owned(
        &mut self,
        matcher: MatcherNode,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = BindingSnapshot::capture(&bindings, &matcher);
        match self.match_compiled_inner_owned(matcher, value, bindings) {
            Ok(outcome) => Ok(finish_compiled_match(snapshot, outcome)),
            Err(EvalError::InputRequired { request }) => {
                let snapshot = snapshot.freeze();
                crate::resume_or_suspend!(insert self, MatchCompiledOwnedSuspensionFrame { snapshot });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime) fn evaluate_pattern_test_owned(
        &mut self,
        test: Expr,
        value: Expr,
        bindings_for_true: SymbolExprMap,
        bindings_for_false: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let test_result = self.evaluate(Expr::call(test, vec![value]));
        match test_result {
            Ok(result) => Ok(if is_system_symbol_name(&result, "True") {
                match_outcome(true, bindings_for_true)
            } else {
                match_outcome(false, bindings_for_false)
            }),
            Err(EvalError::InputRequired { request }) => {
                let bindings_for_true = FrozenMatchBindings::freeze(bindings_for_true);
                let bindings_for_false = FrozenMatchBindings::freeze(bindings_for_false);
                crate::resume_or_suspend!(insert self, EvaluatePatternTestOwnedSuspensionFrame { bindings_for_true, bindings_for_false });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn match_compiled_alternatives_owned(
        &mut self,
        alternatives: Vec<MatcherNode>,
        value: Expr,
        bindings: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(alternative) = alternatives.get(index).cloned() else {
            return Ok(match_outcome(false, bindings));
        };

        let value_for_retry = value.clone();
        let alternatives_for_retry = alternatives.clone();
        let alternative_result = self.match_compiled_owned(alternative, value, bindings);
        match alternative_result {
            Ok(outcome) => self.finish_compiled_alternative(
                alternatives_for_retry,
                value_for_retry,
                index,
                outcome,
            ),
            Err(EvalError::InputRequired { request }) => {
                let alternatives = FrozenMatcherSequence::freeze(alternatives_for_retry);
                crate::resume_or_suspend!(insert self, MatchCompiledAlternativesOwnedSuspensionFrame { alternatives, value_for_retry, index });
                crate::resume_or_suspend!(insert self, ContinueCompiledAlternativeFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_compiled_alternative(
        &mut self,
        alternatives: Vec<MatcherNode>,
        value: Expr,
        index: usize,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.match_compiled_alternatives_owned(alternatives, value, outcome.bindings, index + 1)
        }
    }

    fn match_compiled_named_owned(
        &mut self,
        name: Symbol,
        inner: MatcherNode,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_compiled_owned(inner, value.clone(), bindings);
        match inner_result {
            Ok(outcome) => {
                if !outcome.matched {
                    return Ok(outcome);
                }

                let mut bindings = outcome.bindings;
                Ok(match_outcome(
                    bind_pattern_name(&name, value.clone(), &mut bindings),
                    bindings,
                ))
            }
            Err(EvalError::InputRequired { request }) => {
                self.suspend_compiled_named_match(name, value, request)
            }
            Err(error) => Err(error),
        }
    }

    #[cold]
    #[inline(never)]
    fn suspend_compiled_named_match(
        &mut self,
        name: Symbol,
        value: Expr,
        request: RuntimeInputRequest,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        crate::resume_or_suspend!(insert self, MatchCompiledNamedOwnedSuspensionFrame { name, value });
        Err(EvalError::InputRequired { request })
    }

    fn match_compiled_condition_owned(
        &mut self,
        inner: MatcherNode,
        condition: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_compiled_owned(inner, value, bindings);
        crate::resume_or_suspend!(
            self,
            inner_result,
            ok(outcome) => { let session = self; (|| {
                    if !outcome.matched {
                        return Ok(outcome);
                    }

                    let snapshot = outcome.bindings.clone();
                    let bindings = outcome.bindings;
                    session.evaluate_guard_owned(condition, bindings, snapshot, None)
                })() },
            suspend => MatchCompiledConditionOwnedSuspensionFrame { condition },
        )
    }

    fn match_compiled_pattern_test_owned(
        &mut self,
        inner: MatcherNode,
        test: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_compiled_owned(inner, value.clone(), bindings);
        crate::resume_or_suspend!(
            self,
            inner_result,
            ok(outcome) => { let session = self; (|| {
                    if !outcome.matched {
                        return Ok(outcome);
                    }

                    let false_bindings = outcome.bindings.clone();
                    session.evaluate_pattern_test_owned(
                        test,
                        value.clone(),
                        outcome.bindings,
                        false_bindings,
                    )
                })() },
            suspend => MatchCompiledPatternTestOwnedSuspensionFrame { test, value },
        )
    }

    fn match_compiled_except_owned(
        &mut self,
        excluded: MatcherNode,
        inner: MatcherNode,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_compiled_owned(inner, value.clone(), bindings);
        match inner_result {
            Ok(outcome) => self.finish_compiled_except_inner(excluded, value, outcome),
            Err(EvalError::InputRequired { request }) => {
                let excluded = FrozenMatcherValue::freeze(excluded);
                crate::resume_or_suspend!(insert self, MatchCompiledExceptOwnedSuspensionFrame { excluded, value });
                crate::resume_or_suspend!(insert self, ContinueCompiledExceptFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_compiled_except_inner(
        &mut self,
        excluded: MatcherNode,
        value: Expr,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        let excluded_result = self.match_compiled_owned(excluded, value, outcome.bindings);
        match excluded_result {
            Ok(excluded_outcome) => Ok(match_outcome(
                !excluded_outcome.matched,
                excluded_outcome.bindings,
            )),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, FinishCompiledExceptInnerSuspensionFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn match_compiled_call_owned(
        &mut self,
        head: MatcherNode,
        args: Vec<MatcherArg>,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let one_identity_head = match &head {
            MatcherNode::Exact(Expr::Atom(Atom::Symbol(symbol)))
                if self
                    .attribute_set_for_symbol(symbol)
                    .contains(Attribute::OneIdentity) =>
            {
                Some(*symbol)
            }
            _ => None,
        };
        let orderless = matcher_head_has_attribute(&head, self, Attribute::Orderless);
        let flat = matcher_head_has_attribute(&head, self, Attribute::Flat);

        match value {
            Expr::Call {
                head: value_head,
                args: value_args,
            } => {
                let flat_head = value_head.as_ref().clone();
                let value_head = value_head.into_owned();
                let value_args = value_args.into_vec();
                let head_result = self.match_compiled_owned(head, value_head, bindings);
                match head_result {
                    Ok(outcome) => self.finish_compiled_call_head(
                        args,
                        value_args,
                        flat_head,
                        one_identity_head,
                        orderless,
                        flat,
                        outcome,
                    ),
                    Err(EvalError::InputRequired { request }) => {
                        let args = FrozenMatcherSequence::freeze(args);
                        let value_args = SharedArgs::new(value_args);
                        crate::resume_or_suspend!(insert self, MatchCompiledCallOwnedSuspensionFrame { args, value_args, flat_head, one_identity_head, orderless, flat });
                        crate::resume_or_suspend!(insert self, ContinueCompiledCallFrame);
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
            other if one_identity_head.is_some() => self.match_compiled_promoted_singleton_owned(
                args,
                other,
                bindings,
                orderless,
                flat,
                Expr::Atom(Atom::Symbol(
                    one_identity_head.expect("guarded OneIdentity head should be present"),
                )),
            ),
            _ => Ok(match_outcome(false, bindings)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_compiled_call_head(
        &mut self,
        args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        flat_head: Expr,
        one_identity_head: Option<Symbol>,
        orderless: bool,
        flat: bool,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            let Some(one_identity_head) = one_identity_head else {
                return Ok(outcome);
            };

            return self.match_compiled_promoted_singleton_owned(
                args,
                Expr::call(flat_head, value_args),
                outcome.bindings,
                orderless,
                flat,
                Expr::Atom(Atom::Symbol(one_identity_head)),
            );
        }

        if orderless {
            if flat {
                self.match_compiled_orderless_args_with_flat_head_owned(
                    args,
                    value_args,
                    outcome.bindings,
                    flat_head,
                )
            } else {
                self.match_compiled_orderless_args_owned(args, value_args, outcome.bindings)
            }
        } else if flat {
            self.match_compiled_args_with_flat_head_owned(
                args,
                value_args,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.match_compiled_args_owned(args, value_args, outcome.bindings)
        }
    }

    fn match_compiled_promoted_singleton_owned(
        &mut self,
        args: Vec<MatcherArg>,
        value: Expr,
        bindings: SymbolExprMap,
        orderless: bool,
        flat: bool,
        flat_head: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let value_args = vec![value];

        if orderless {
            if flat {
                self.match_compiled_orderless_args_with_flat_head_owned(
                    args, value_args, bindings, flat_head,
                )
            } else {
                self.match_compiled_orderless_args_owned(args, value_args, bindings)
            }
        } else if flat {
            self.match_compiled_args_with_flat_head_owned(args, value_args, bindings, flat_head)
        } else {
            self.match_compiled_args_owned(args, value_args, bindings)
        }
    }

    pub(super) fn evaluate_sequence_pattern_checks_owned(
        &mut self,
        checks: Vec<SequencePatternCheck>,
        sequence_value: Expr,
        bindings: SymbolExprMap,
        snapshot: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(check) = checks.get(index).cloned() else {
            return Ok(match_outcome(true, bindings));
        };

        match check {
            SequencePatternCheck::Condition(condition) => {
                let checks_for_next = checks.clone();
                let sequence_for_next = sequence_value.clone();
                let snapshot_for_next = snapshot.clone();
                let condition_result =
                    self.evaluate_guard_owned(condition, bindings, snapshot, None);
                match condition_result {
                    Ok(outcome) => self.finish_sequence_condition_check(
                        checks_for_next,
                        sequence_for_next,
                        snapshot_for_next,
                        index,
                        outcome,
                    ),
                    Err(EvalError::InputRequired { request }) => {
                        let checks = FrozenMatcherSequence::freeze(checks_for_next);
                        let snapshot = FrozenMatchBindings::freeze(snapshot_for_next);
                        crate::resume_or_suspend!(insert self, EvaluateSequencePatternChecksOwnedSuspensionFrame1 { checks, sequence_for_next, snapshot, index });
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
            SequencePatternCheck::PatternTest(test) => {
                let checks_for_next = checks.clone();
                let sequence_for_test = sequence_value.clone();
                let sequence_for_next = sequence_value.clone();
                let bindings_for_true = bindings.clone();
                let snapshot_for_false = snapshot.clone();
                let test_result =
                    self.evaluate_pattern_test_owned(test, sequence_for_test, bindings, snapshot);
                match test_result {
                    Ok(outcome) => self.finish_sequence_pattern_test_check(
                        checks_for_next,
                        sequence_for_next,
                        bindings_for_true,
                        snapshot_for_false,
                        index,
                        outcome,
                    ),
                    Err(EvalError::InputRequired { request }) => {
                        let checks = FrozenMatcherSequence::freeze(checks_for_next);
                        let bindings = FrozenMatchBindings::freeze(bindings_for_true);
                        let snapshot = FrozenMatchBindings::freeze(snapshot_for_false);
                        crate::resume_or_suspend!(insert self, EvaluateSequencePatternChecksOwnedSuspensionFrame2 { checks, sequence_for_next, bindings, snapshot, index });
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }

    fn finish_sequence_condition_check(
        &mut self,
        checks: Vec<SequencePatternCheck>,
        sequence_value: Expr,
        snapshot: SymbolExprMap,
        index: usize,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            self.evaluate_sequence_pattern_checks_owned(
                checks,
                sequence_value,
                outcome.bindings,
                snapshot,
                index + 1,
            )
        } else {
            Ok(outcome)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_sequence_pattern_test_check(
        &mut self,
        checks: Vec<SequencePatternCheck>,
        sequence_value: Expr,
        bindings: SymbolExprMap,
        snapshot: SymbolExprMap,
        index: usize,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            self.evaluate_sequence_pattern_checks_owned(
                checks,
                sequence_value,
                bindings,
                snapshot,
                index + 1,
            )
        } else {
            Ok(outcome)
        }
    }

    fn match_compiled_inner_owned(
        &mut self,
        matcher: MatcherNode,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match matcher {
            MatcherNode::Exact(pattern) => Ok(match_outcome(pattern == value, bindings)),
            MatcherNode::Blank { head_constraint } => Ok(match_outcome(
                match_blank(head_constraint.as_ref(), &value),
                bindings,
            )),
            MatcherNode::KeyValuePattern(entries) => {
                self.match_compiled_key_value_pattern_owned(entries, value, bindings)
            }
            MatcherNode::Named { name, inner } => {
                self.match_compiled_named_owned(name, *inner, value, bindings)
            }
            MatcherNode::Condition { inner, condition } => {
                self.match_compiled_condition_owned(*inner, condition, value, bindings)
            }
            MatcherNode::DefinitionCondition(matcher) => {
                self.match_compiled_definition_condition_owned(*matcher, value, bindings)
            }
            MatcherNode::PatternTest { inner, test } => {
                self.match_compiled_pattern_test_owned(*inner, test, value, bindings)
            }
            MatcherNode::Except { excluded, inner } => {
                self.match_compiled_except_owned(*excluded, *inner, value, bindings)
            }
            MatcherNode::Alternatives(alternatives) => {
                self.match_compiled_alternatives_owned(alternatives, value, bindings, 0)
            }
            MatcherNode::Call { head, args } => {
                self.match_compiled_call_owned(*head, args, value, bindings)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct BindingSnapshot {
    entries: Vec<(Symbol, Option<Expr>)>,
}

impl BindingSnapshot {
    fn capture(bindings: &SymbolExprMap, matcher: &MatcherNode) -> Self {
        let mut names = BTreeSet::new();
        collect_matcher_binding_names(matcher, &mut names);
        let entries = names
            .into_iter()
            .map(|name| {
                let previous = bindings.get(&name).cloned();
                (name, previous)
            })
            .collect();
        Self { entries }
    }

    fn restore(self, mut bindings: SymbolExprMap) -> SymbolExprMap {
        for (name, previous) in self.entries {
            match previous {
                Some(value) => {
                    bindings.insert(name, value);
                }
                None => {
                    bindings.remove(&name);
                }
            }
        }
        bindings
    }

    fn freeze(self) -> FrozenMatcherSequence<(Symbol, Option<Expr>)> {
        FrozenMatcherSequence::freeze(self.entries)
    }
}

fn finish_compiled_match(
    snapshot: BindingSnapshot,
    outcome: CompiledMatchOutcome,
) -> CompiledMatchOutcome {
    if outcome.matched {
        outcome
    } else {
        match_outcome(false, snapshot.restore(outcome.bindings))
    }
}

fn finish_suspended_compiled_match(
    snapshot: FrozenMatcherSequence<(Symbol, Option<Expr>)>,
    outcome: CompiledMatchOutcome,
) -> CompiledMatchOutcome {
    finish_compiled_match(
        BindingSnapshot {
            entries: snapshot.into_vec(),
        },
        outcome,
    )
}

fn collect_matcher_binding_names(matcher: &MatcherNode, names: &mut BTreeSet<Symbol>) {
    match matcher {
        MatcherNode::Exact(_) | MatcherNode::Blank { .. } => {}
        MatcherNode::KeyValuePattern(entries) => {
            for (key, value, _) in entries {
                collect_matcher_binding_names(key, names);
                collect_matcher_binding_names(value, names);
            }
        }
        MatcherNode::Named { name, inner } => {
            names.insert(name.clone());
            collect_matcher_binding_names(inner, names);
        }
        MatcherNode::Condition { inner, .. } | MatcherNode::PatternTest { inner, .. } => {
            collect_matcher_binding_names(inner, names);
        }
        MatcherNode::DefinitionCondition(matcher) => {
            collect_matcher_binding_names(&matcher.inner, names);
        }
        MatcherNode::Except { excluded, inner } => {
            collect_matcher_binding_names(excluded, names);
            collect_matcher_binding_names(inner, names);
        }
        MatcherNode::Alternatives(alternatives) => {
            for alternative in alternatives {
                collect_matcher_binding_names(alternative, names);
            }
        }
        MatcherNode::Call { head, args } => {
            collect_matcher_binding_names(head, names);
            for arg in args {
                collect_matcher_arg_binding_names(arg, names);
            }
        }
    }
}

fn collect_matcher_arg_binding_names(arg: &MatcherArg, names: &mut BTreeSet<Symbol>) {
    match arg {
        MatcherArg::Single(matcher) => collect_matcher_binding_names(matcher, names),
        MatcherArg::Sequence(spec) => {
            if let Some(name) = &spec.name {
                names.insert(name.clone());
            }
            collect_matcher_binding_names(&spec.matcher, names);
        }
        MatcherArg::Optional(spec) => collect_matcher_arg_binding_names(&spec.inner, names),
        MatcherArg::PatternSequenceGroup(items)
        | MatcherArg::OrderlessPatternSequenceGroup(items) => {
            for item in items {
                collect_matcher_arg_binding_names(item, names);
            }
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedKeyValueChoice>();
};
