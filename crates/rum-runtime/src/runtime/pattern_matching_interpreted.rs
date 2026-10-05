use std::sync::Arc;

use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

fn interpreted_match_outcome(matched: bool, bindings: SymbolExprMap) -> CompiledMatchOutcome {
    CompiledMatchOutcome { matched, bindings }
}

impl Session {
    fn match_named_interpreted_owned(
        &mut self,
        name: Symbol,
        inner: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
        crate::resume_or_suspend!(
            self,
            inner_result,
            ok(outcome) => finish_interpreted_named(name, value, outcome),
            suspend => MatchNamedInterpretedFrame { name, value },
        )
    }

    fn match_condition_interpreted_owned(
        &mut self,
        inner: Expr,
        condition: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_expr_interpreted_owned(inner, value, bindings);
        crate::resume_or_suspend!(
            self,
            inner_result,
            ok(outcome) => {
            if !outcome.matched {
                Ok(outcome)
            } else {
                let snapshot = outcome.bindings.clone();
                let bindings = outcome.bindings;
                self.evaluate_guard_owned(condition, bindings, snapshot, None)
            }
            },
            suspend => MatchConditionInterpretedFrame { condition },
        )
    }

    fn match_pattern_test_interpreted_owned(
        &mut self,
        inner: Expr,
        test: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
        crate::resume_or_suspend!(
            self,
            inner_result,
            ok(outcome) => {
            if !outcome.matched {
                Ok(outcome)
            } else {
                let false_bindings = outcome.bindings.clone();
                self.evaluate_pattern_test_owned(
                    test,
                    value,
                    outcome.bindings,
                    false_bindings,
                )
            }
            },
            suspend => MatchPatternTestInterpretedFrame { test, value },
        )
    }

    fn match_except_interpreted_owned(
        &mut self,
        excluded: Expr,
        inner: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();
        let inner_result = self.match_expr_interpreted_owned(inner, value.clone(), bindings);
        match inner_result {
            Ok(outcome) => self.finish_interpreted_except_inner(excluded, value, snapshot, outcome),
            Err(EvalError::InputRequired { request }) => {
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                crate::resume_or_suspend!(
                    insert self,
                    PrepareInterpretedExceptFrame {
                        excluded,
                        value,
                        snapshot,
                    },
                );
                crate::resume_or_suspend!(
                    insert self,
                    ContinueInterpretedExceptFrame,
                );
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_prepared_interpreted_except(
        &mut self,
        prepared: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let (excluded, value, snapshot, outcome) = decode_prepared_interpreted_except(prepared);
        self.finish_interpreted_except_inner(excluded, value, snapshot, outcome)
    }

    fn finish_interpreted_except_inner(
        &mut self,
        excluded: Expr,
        value: Expr,
        snapshot: SymbolExprMap,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        let accepted_bindings = outcome.bindings.clone();
        let excluded_result = self.match_expr_interpreted_owned(excluded, value, outcome.bindings);
        match excluded_result {
            Ok(excluded_outcome) => Ok(if excluded_outcome.matched {
                interpreted_match_outcome(false, snapshot)
            } else {
                interpreted_match_outcome(true, accepted_bindings)
            }),
            Err(EvalError::InputRequired { request }) => {
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                let accepted_bindings = FrozenMatchBindings::freeze(accepted_bindings);
                crate::resume_or_suspend!(
                    insert self,
                    FinishInterpretedExcludedFrame {
                        snapshot,
                        accepted_bindings,
                    },
                );
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn match_call_interpreted_owned(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match value {
            Expr::Call {
                head: value_head,
                args: value_args,
            } => {
                let head_expr = head.clone();
                let value_head = value_head.into_owned();
                let value_args = value_args.into_vec();
                let head_result = self.match_expr_interpreted_owned(head, value_head, bindings);
                match head_result {
                    Ok(outcome) => {
                        self.finish_interpreted_call_head(head_expr, args, value_args, outcome)
                    }
                    Err(EvalError::InputRequired { request }) => {
                        let args = SharedArgs::new(args);
                        let value_args = SharedArgs::new(value_args);
                        crate::resume_or_suspend!(
                            insert self,
                            PrepareInterpretedCallFrame {
                                args,
                                value_args,
                            },
                        );
                        crate::resume_or_suspend!(
                            insert self,
                            ContinueInterpretedCallFrame { head: head_expr },
                        );
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
            other
                if matcher_head_has_attribute(
                    &MatcherNode::Exact(head.clone()),
                    self,
                    Attribute::OneIdentity,
                ) =>
            {
                self.match_call_args_interpreted_owned(head, args, vec![other], bindings)
            }
            _ => Ok(interpreted_match_outcome(false, bindings)),
        }
    }

    fn finish_interpreted_call_head(
        &mut self,
        head: Expr,
        args: Vec<Expr>,
        value_args: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        self.match_call_args_interpreted_owned(head, args, value_args, outcome.bindings)
    }

    fn finish_prepared_interpreted_call(
        &mut self,
        head: Expr,
        prepared: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match decode_prepared_interpreted_call(prepared) {
            PreparedInterpretedCall::Complete(outcome) => Ok(outcome),
            PreparedInterpretedCall::Continue {
                args,
                value_args,
                outcome,
            } => self.finish_interpreted_call_head(head, args, value_args, outcome),
        }
    }

    #[allow(dead_code)]
    fn match_alternatives_interpreted_owned(
        &mut self,
        alternatives: Vec<Expr>,
        value: Expr,
        bindings: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(alternative) = alternatives.get(index).cloned() else {
            return Ok(interpreted_match_outcome(false, bindings));
        };

        let snapshot = bindings.clone();
        let value_for_retry = value.clone();
        let alternatives_for_retry = alternatives.clone();
        let alternative_result = self.match_expr_interpreted_owned(alternative, value, bindings);
        match alternative_result {
            Ok(outcome) => self.finish_interpreted_alternative(
                alternatives_for_retry,
                value_for_retry,
                snapshot,
                index,
                outcome,
            ),
            Err(EvalError::InputRequired { request }) => {
                let alternatives = SharedArgs::new(alternatives_for_retry);
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                crate::resume_or_suspend!(
                    insert self,
                    PrepareInterpretedAlternativeFrame {
                        alternatives,
                        value: value_for_retry,
                        snapshot,
                        index,
                    },
                );
                crate::resume_or_suspend!(
                    insert self,
                    ContinueInterpretedAlternativeFrame,
                );
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_interpreted_alternative(
        &mut self,
        alternatives: Vec<Expr>,
        value: Expr,
        snapshot: SymbolExprMap,
        index: usize,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.match_alternatives_interpreted_owned(alternatives, value, snapshot, index + 1)
        }
    }

    fn finish_prepared_interpreted_alternative(
        &mut self,
        prepared: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match decode_prepared_interpreted_alternative(prepared) {
            PreparedInterpretedAlternative::Complete(outcome) => Ok(outcome),
            PreparedInterpretedAlternative::Retry {
                alternatives,
                value,
                bindings,
                index,
            } => {
                self.match_alternatives_interpreted_owned(alternatives, value, bindings, index + 1)
            }
        }
    }

    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted_owned(
        &mut self,
        pattern: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let normalized_pattern = self.normalize_match_expr(&pattern);
        let normalized_value = self.normalize_match_expr(&value);
        let snapshot = bindings.clone();
        let inner_result =
            self.match_expr_interpreted_inner_owned(normalized_pattern, normalized_value, bindings);
        match inner_result {
            Ok(outcome) => Ok(finish_interpreted_match(snapshot, outcome)),
            Err(EvalError::InputRequired { request }) => {
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                crate::resume_or_suspend!(
                    insert self,
                    FinishInterpretedMatchFrame { snapshot },
                );
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    #[allow(dead_code)]
    pub(super) fn match_expr_interpreted_inner_owned(
        &mut self,
        pattern: Expr,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if let Some((name, inner)) = pattern_named_pattern(&pattern) {
            return self.match_named_interpreted_owned(
                name.clone(),
                inner.clone(),
                value,
                bindings,
            );
        }

        if let Some(entries) = pattern_key_value_pattern_entries(&pattern) {
            return self.match_compiled_key_value_pattern_owned(
                entries
                    .into_iter()
                    .map(|entry| {
                        (
                            compile_matcher(&entry.key),
                            compile_matcher(&entry.value),
                            entry.delayed,
                        )
                    })
                    .collect(),
                value,
                bindings,
            );
        }

        if let Some((inner, condition)) = pattern_condition(&pattern) {
            return self.match_condition_interpreted_owned(
                inner.clone(),
                condition.clone(),
                value,
                bindings,
            );
        }

        if let Some((inner, test)) = pattern_test(&pattern) {
            return self.match_pattern_test_interpreted_owned(
                inner.clone(),
                test.clone(),
                value,
                bindings,
            );
        }

        if let Some((excluded, inner)) = pattern_except(&pattern) {
            let inner = inner
                .cloned()
                .unwrap_or_else(|| Expr::call(system_expr("Blank"), vec![]));
            return self.match_except_interpreted_owned(excluded.clone(), inner, value, bindings);
        }

        if let Some((inner, _)) = pattern_optional(&pattern) {
            return self.match_expr_interpreted_owned(inner.clone(), value, bindings);
        }

        if let Some(alternatives) = pattern_alternatives(&pattern) {
            return self.match_alternatives_interpreted_owned(
                alternatives.to_vec(),
                value,
                bindings,
                0,
            );
        }

        if let Some(blank_head) = pattern_blank_head(&pattern) {
            return Ok(interpreted_match_outcome(
                match_blank(blank_head, &value),
                bindings,
            ));
        }

        match (pattern, value) {
            (Expr::Atom(lhs), Expr::Atom(rhs)) => {
                Ok(interpreted_match_outcome(lhs == rhs, bindings))
            }
            (Expr::Call { head, args }, value) => self.match_call_interpreted_owned(
                head.into_owned(),
                args.into_vec(),
                value,
                bindings,
            ),
            _ => Ok(interpreted_match_outcome(false, bindings)),
        }
    }

    #[allow(dead_code)]
    pub(super) fn match_call_args_interpreted_owned(
        &mut self,
        pattern_head: Expr,
        pattern_args: Vec<Expr>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let compiled_args = compile_matcher_args(&pattern_head, &pattern_args);
        self.match_compiled_args_owned(compiled_args, value_args, bindings)
    }
}

fn finish_interpreted_match(
    snapshot: SymbolExprMap,
    outcome: CompiledMatchOutcome,
) -> CompiledMatchOutcome {
    if outcome.matched {
        outcome
    } else {
        interpreted_match_outcome(false, snapshot)
    }
}

fn finish_interpreted_named(
    name: Symbol,
    value: Expr,
    outcome: CompiledMatchOutcome,
) -> Result<CompiledMatchOutcome, EvalError> {
    if !outcome.matched {
        return Ok(outcome);
    }

    let mut bindings = outcome.bindings;
    Ok(interpreted_match_outcome(
        bind_pattern_name(&name, value, &mut bindings),
        bindings,
    ))
}

fn complete_interpreted_match_action(
    result: Result<CompiledMatchOutcome, EvalError>,
) -> FrameAction {
    FrameAction::complete(match result {
        Ok(outcome) => Ok(outcome.into_continuation_expr()),
        Err(error) => Err(error),
    })
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct MatchNamedInterpretedFrame {
    name: Symbol,
    value: Expr,
}

impl SuspensionFrame for MatchNamedInterpretedFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let outcome = match outcome {
            Ok(payload) => CompiledMatchOutcome::from_continuation_expr(payload),
            Err(error) => return FrameAction::complete(Err(error)),
        };
        complete_interpreted_match_action(finish_interpreted_named(
            self.name,
            self.value.clone(),
            outcome,
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct MatchConditionInterpretedFrame {
    condition: Expr,
}

impl SuspensionFrame for MatchConditionInterpretedFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let outcome = match outcome {
            Ok(payload) => CompiledMatchOutcome::from_continuation_expr(payload),
            Err(error) => return FrameAction::complete(Err(error)),
        };
        let result = if outcome.matched {
            let snapshot = outcome.bindings.clone();
            session.evaluate_guard_owned(self.condition.clone(), outcome.bindings, snapshot, None)
        } else {
            Ok(outcome)
        };
        complete_interpreted_match_action(result)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct MatchPatternTestInterpretedFrame {
    test: Expr,
    value: Expr,
}

impl SuspensionFrame for MatchPatternTestInterpretedFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let outcome = match outcome {
            Ok(payload) => CompiledMatchOutcome::from_continuation_expr(payload),
            Err(error) => return FrameAction::complete(Err(error)),
        };
        let result = if outcome.matched {
            let false_bindings = outcome.bindings.clone();
            session.evaluate_pattern_test_owned(
                self.test.clone(),
                self.value.clone(),
                outcome.bindings,
                false_bindings,
            )
        } else {
            Ok(outcome)
        };
        complete_interpreted_match_action(result)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct PrepareInterpretedExceptFrame {
    excluded: Expr,
    value: Expr,
    snapshot: FrozenMatchBindings,
}

impl SuspensionFrame for PrepareInterpretedExceptFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        match outcome {
            Ok(payload) => FrameAction::finalize(PrepareInterpretedExceptFinalizer {
                excluded: self.excluded.clone(),
                value: self.value.clone(),
                snapshot: self.snapshot.clone(),
                outcome: CompiledMatchOutcome::from_continuation_expr(payload),
            }),
            Err(error) => FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareInterpretedExceptFinalizer {
    excluded: Expr,
    value: Expr,
    snapshot: FrozenMatchBindings,
    outcome: CompiledMatchOutcome,
}

impl PureFrameFinalizer for PrepareInterpretedExceptFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalInterpretedExcept"),
            vec![
                self.excluded,
                self.value,
                interpreted_match_outcome(false, self.snapshot.into_owned())
                    .into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

fn decode_prepared_interpreted_except(
    prepared: Expr,
) -> (Expr, Expr, SymbolExprMap, CompiledMatchOutcome) {
    let Expr::Call { head, args } = prepared else {
        panic!("prepared interpreted Except payload should be a call")
    };
    assert!(is_system_symbol_name(&head, "InternalInterpretedExcept"));
    let mut args = args.into_iter();
    let excluded = args.next().expect("Except payload stores excluded pattern");
    let value = args.next().expect("Except payload stores matched value");
    let snapshot = CompiledMatchOutcome::from_continuation_expr(
        args.next().expect("Except payload stores binding snapshot"),
    )
    .bindings;
    let outcome = CompiledMatchOutcome::from_continuation_expr(
        args.next().expect("Except payload stores inner outcome"),
    );
    assert!(args.next().is_none());
    (excluded, value, snapshot, outcome)
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueInterpretedExceptFrame;

impl SuspensionFrame for ContinueInterpretedExceptFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let prepared = match outcome {
            Ok(prepared) => prepared,
            Err(error) => return FrameAction::complete(Err(error)),
        };
        complete_interpreted_match_action(session.finish_prepared_interpreted_except(prepared))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishInterpretedExcludedFrame {
    snapshot: FrozenMatchBindings,
    accepted_bindings: FrozenMatchBindings,
}

impl SuspensionFrame for FinishInterpretedExcludedFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        match outcome {
            Ok(payload) => FrameAction::finalize(FinishInterpretedExcludedFinalizer {
                snapshot: self.snapshot.clone(),
                accepted_bindings: self.accepted_bindings.clone(),
                excluded_outcome: CompiledMatchOutcome::from_continuation_expr(payload),
            }),
            Err(error) => FrameAction::complete(Err(error)),
        }
    }
}

struct FinishInterpretedExcludedFinalizer {
    snapshot: FrozenMatchBindings,
    accepted_bindings: FrozenMatchBindings,
    excluded_outcome: CompiledMatchOutcome,
}

impl PureFrameFinalizer for FinishInterpretedExcludedFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let outcome = if self.excluded_outcome.matched {
            interpreted_match_outcome(false, self.snapshot.into_owned())
        } else {
            interpreted_match_outcome(true, self.accepted_bindings.into_owned())
        };
        Ok(outcome.into_continuation_expr())
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct PrepareInterpretedCallFrame {
    args: SharedArgs,
    value_args: SharedArgs,
}

impl SuspensionFrame for PrepareInterpretedCallFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        match outcome {
            Ok(payload) => FrameAction::finalize(PrepareInterpretedCallFinalizer {
                args: self.args.clone(),
                value_args: self.value_args.clone(),
                outcome: CompiledMatchOutcome::from_continuation_expr(payload),
            }),
            Err(error) => FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareInterpretedCallFinalizer {
    args: SharedArgs,
    value_args: SharedArgs,
    outcome: CompiledMatchOutcome,
}

impl PureFrameFinalizer for PrepareInterpretedCallFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        if !self.outcome.matched {
            return Ok(Expr::call(
                system_expr("InternalInterpretedCallComplete"),
                vec![self.outcome.into_continuation_expr()],
            ));
        }
        Ok(Expr::call(
            system_expr("InternalInterpretedCallContinue"),
            vec![
                self.args.into_vec().into_continuation_expr(),
                self.value_args.into_vec().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

enum PreparedInterpretedCall {
    Complete(CompiledMatchOutcome),
    Continue {
        args: Vec<Expr>,
        value_args: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    },
}

fn decode_prepared_interpreted_call(prepared: Expr) -> PreparedInterpretedCall {
    let Expr::Call { head, args } = prepared else {
        panic!("prepared interpreted call payload should be a call")
    };
    let mut args = args.into_iter();
    if is_system_symbol_name(&head, "InternalInterpretedCallComplete") {
        return PreparedInterpretedCall::Complete(CompiledMatchOutcome::from_continuation_expr(
            args.next()
                .expect("completed call payload stores its outcome"),
        ));
    }
    assert!(is_system_symbol_name(
        &head,
        "InternalInterpretedCallContinue"
    ));
    let pattern_args = Vec::<Expr>::from_continuation_expr(
        args.next().expect("call payload stores pattern arguments"),
    );
    let value_args = Vec::<Expr>::from_continuation_expr(
        args.next().expect("call payload stores value arguments"),
    );
    let outcome = CompiledMatchOutcome::from_continuation_expr(
        args.next().expect("call payload stores head outcome"),
    );
    assert!(args.next().is_none());
    PreparedInterpretedCall::Continue {
        args: pattern_args,
        value_args,
        outcome,
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueInterpretedCallFrame {
    head: Expr,
}

impl SuspensionFrame for ContinueInterpretedCallFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let prepared = match outcome {
            Ok(prepared) => prepared,
            Err(error) => return FrameAction::complete(Err(error)),
        };
        complete_interpreted_match_action(
            session.finish_prepared_interpreted_call(self.head.clone(), prepared),
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct PrepareInterpretedAlternativeFrame {
    alternatives: SharedArgs,
    value: Expr,
    snapshot: FrozenMatchBindings,
    index: usize,
}

impl SuspensionFrame for PrepareInterpretedAlternativeFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        match outcome {
            Ok(payload) => FrameAction::finalize(PrepareInterpretedAlternativeFinalizer {
                alternatives: self.alternatives.clone(),
                value: self.value.clone(),
                snapshot: self.snapshot.clone(),
                index: self.index,
                outcome: CompiledMatchOutcome::from_continuation_expr(payload),
            }),
            Err(error) => FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareInterpretedAlternativeFinalizer {
    alternatives: SharedArgs,
    value: Expr,
    snapshot: FrozenMatchBindings,
    index: usize,
    outcome: CompiledMatchOutcome,
}

impl PureFrameFinalizer for PrepareInterpretedAlternativeFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        if self.outcome.matched {
            return Ok(Expr::call(
                system_expr("InternalInterpretedAlternativeComplete"),
                vec![self.outcome.into_continuation_expr()],
            ));
        }
        Ok(Expr::call(
            system_expr("InternalInterpretedAlternativeRetry"),
            vec![
                self.alternatives.into_vec().into_continuation_expr(),
                self.value,
                interpreted_match_outcome(false, self.snapshot.into_owned())
                    .into_continuation_expr(),
                Expr::small_integer(self.index as i64),
            ],
        ))
    }
}

enum PreparedInterpretedAlternative {
    Complete(CompiledMatchOutcome),
    Retry {
        alternatives: Vec<Expr>,
        value: Expr,
        bindings: SymbolExprMap,
        index: usize,
    },
}

fn decode_prepared_interpreted_alternative(prepared: Expr) -> PreparedInterpretedAlternative {
    let Expr::Call { head, args } = prepared else {
        panic!("prepared interpreted alternative payload should be a call")
    };
    let mut args = args.into_iter();
    if is_system_symbol_name(&head, "InternalInterpretedAlternativeComplete") {
        return PreparedInterpretedAlternative::Complete(
            CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("completed alternative stores its outcome"),
            ),
        );
    }
    assert!(is_system_symbol_name(
        &head,
        "InternalInterpretedAlternativeRetry"
    ));
    let alternatives = Vec::<Expr>::from_continuation_expr(
        args.next().expect("alternative retry stores alternatives"),
    );
    let value = args.next().expect("alternative retry stores value");
    let bindings = CompiledMatchOutcome::from_continuation_expr(
        args.next().expect("alternative retry stores bindings"),
    )
    .bindings;
    let index_expr = args.next().expect("alternative retry stores index");
    let Expr::Atom(Atom::Integer(index)) = index_expr else {
        panic!("alternative retry index should be an integer")
    };
    let index = index
        .as_i64()
        .and_then(|index| usize::try_from(index).ok())
        .expect("alternative retry index should fit usize");
    assert!(args.next().is_none());
    PreparedInterpretedAlternative::Retry {
        alternatives,
        value,
        bindings,
        index,
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueInterpretedAlternativeFrame;

impl SuspensionFrame for ContinueInterpretedAlternativeFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let prepared = match outcome {
            Ok(prepared) => prepared,
            Err(error) => return FrameAction::complete(Err(error)),
        };
        complete_interpreted_match_action(session.finish_prepared_interpreted_alternative(prepared))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishInterpretedMatchFrame {
    snapshot: FrozenMatchBindings,
}

impl SuspensionFrame for FinishInterpretedMatchFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        match outcome {
            Ok(payload) => FrameAction::finalize(FinishInterpretedMatchFinalizer {
                snapshot: self.snapshot.clone(),
                outcome: CompiledMatchOutcome::from_continuation_expr(payload),
            }),
            Err(error) => FrameAction::complete(Err(error)),
        }
    }
}

struct FinishInterpretedMatchFinalizer {
    snapshot: FrozenMatchBindings,
    outcome: CompiledMatchOutcome,
}

impl PureFrameFinalizer for FinishInterpretedMatchFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let outcome = if self.outcome.matched {
            self.outcome
        } else {
            interpreted_match_outcome(false, self.snapshot.into_owned())
        };
        Ok(outcome.into_continuation_expr())
    }
}
