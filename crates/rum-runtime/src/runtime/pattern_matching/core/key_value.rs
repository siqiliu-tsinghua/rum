use super::*;

impl Session {
    pub(in crate::runtime) fn match_compiled_key_value_pattern_owned(
        &mut self,
        entries: Vec<(MatcherNode, MatcherNode, bool)>,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(candidates) = keyed_entry_forms_from_expr(&value) else {
            return Ok(match_outcome(false, bindings));
        };

        self.match_compiled_key_value_pattern_entries_owned(entries, candidates, bindings)
    }

    fn match_compiled_key_value_pattern_entries_owned(
        &mut self,
        entries: Vec<(MatcherNode, MatcherNode, bool)>,
        candidates: Vec<AssociationEntryForm>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some((key_pattern, value_pattern, delayed)) = entries.first().cloned() else {
            return Ok(match_outcome(true, bindings));
        };

        self.match_compiled_key_value_pattern_choice_owned(
            key_pattern,
            value_pattern,
            delayed,
            entries.into_iter().skip(1).collect(),
            candidates,
            bindings,
            0,
        )
    }

    fn match_compiled_key_value_pattern_choice_owned(
        &mut self,
        key_pattern: MatcherNode,
        value_pattern: MatcherNode,
        delayed: bool,
        remaining_entries: Vec<(MatcherNode, MatcherNode, bool)>,
        candidates: Vec<AssociationEntryForm>,
        bindings: SymbolExprMap,
        candidate_index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(candidate) = candidates.get(candidate_index).cloned() else {
            return Ok(match_outcome(false, bindings));
        };

        if candidate.delayed != delayed {
            return self.match_compiled_key_value_pattern_choice_owned(
                key_pattern,
                value_pattern,
                delayed,
                remaining_entries,
                candidates,
                bindings,
                candidate_index + 1,
            );
        }
        let candidate_key = candidate.key;
        let candidate_value = candidate.value;
        let state = KeyValueChoiceState {
            key_pattern,
            value_pattern,
            delayed,
            remaining_entries,
            candidates,
            candidate_index,
        };

        let key_result =
            self.match_compiled_owned(state.key_pattern.clone(), candidate_key, bindings);
        match key_result {
            Ok(outcome) => self.finish_key_value_key_match(state, candidate_value, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedKeyValueChoice::freeze(state);
                crate::resume_or_suspend!(insert self, MatchCompiledKeyValuePatternChoiceOwnedSuspensionFrame { state, candidate_value });
                crate::resume_or_suspend!(insert self, ContinueKeyValueKeyMatchFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn finish_key_value_key_match(
        &mut self,
        state: KeyValueChoiceState,
        candidate_value: Expr,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return self.match_compiled_key_value_pattern_choice_owned(
                state.key_pattern,
                state.value_pattern,
                state.delayed,
                state.remaining_entries,
                state.candidates,
                outcome.bindings,
                state.candidate_index + 1,
            );
        }

        let value_result = self.match_compiled_owned(
            state.value_pattern.clone(),
            candidate_value,
            outcome.bindings,
        );
        match value_result {
            Ok(outcome) => self.finish_key_value_value_match(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedKeyValueChoice::freeze(state);
                crate::resume_or_suspend!(insert self, FinishKeyValueKeyMatchSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueKeyValueValueMatchFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn finish_key_value_value_match(
        &mut self,
        mut state: KeyValueChoiceState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return self.match_compiled_key_value_pattern_choice_owned(
                state.key_pattern,
                state.value_pattern,
                state.delayed,
                state.remaining_entries,
                state.candidates,
                outcome.bindings,
                state.candidate_index + 1,
            );
        }

        state.candidates.remove(state.candidate_index);
        self.match_compiled_key_value_pattern_entries_owned(
            state.remaining_entries,
            state.candidates,
            outcome.bindings,
        )
    }
}

pub(super) struct KeyValueChoiceState {
    key_pattern: MatcherNode,
    value_pattern: MatcherNode,
    delayed: bool,
    remaining_entries: Vec<(MatcherNode, MatcherNode, bool)>,
    candidates: Vec<AssociationEntryForm>,
    candidate_index: usize,
}

impl KeyValueChoiceState {
    pub(super) fn into_continuation_expr(self) -> Expr {
        let remaining_entries = self
            .remaining_entries
            .into_iter()
            .map(|(key, value, optional)| {
                Expr::call(
                    system_expr("InternalContinuationKeyValuePatternEntry"),
                    vec![
                        matcher_node_into_continuation_expr(key),
                        matcher_node_into_continuation_expr(value),
                        optional.into_continuation_expr(),
                    ],
                )
            })
            .collect();
        let candidates = self
            .candidates
            .into_iter()
            .map(|candidate| {
                Expr::call(
                    system_expr("InternalContinuationKeyValueCandidate"),
                    vec![
                        candidate.key,
                        candidate.value,
                        candidate.delayed.into_continuation_expr(),
                    ],
                )
            })
            .collect();
        Expr::call(
            system_expr("InternalContinuationKeyValueChoiceState"),
            vec![
                matcher_node_into_continuation_expr(self.key_pattern),
                matcher_node_into_continuation_expr(self.value_pattern),
                self.delayed.into_continuation_expr(),
                Expr::call(
                    system_expr("InternalContinuationKeyValuePatternEntries"),
                    remaining_entries,
                ),
                Expr::call(
                    system_expr("InternalContinuationKeyValueCandidates"),
                    candidates,
                ),
                usize_into_continuation_expr(self.candidate_index),
            ],
        )
    }

    pub(super) fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args } = expr else {
            panic!("unexpected key-value-choice continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationKeyValueChoiceState"),
            "unexpected key-value-choice continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let key_pattern = matcher_node_from_continuation_expr(
            args.next()
                .expect("key-value choice stores its key pattern"),
        );
        let value_pattern = matcher_node_from_continuation_expr(
            args.next()
                .expect("key-value choice stores its value pattern"),
        );
        let delayed = bool::from_continuation_expr(
            args.next()
                .expect("key-value choice stores its delayed flag"),
        );
        let entries_expr = args
            .next()
            .expect("key-value choice stores its remaining entries");
        let Expr::Call {
            head: entries_head,
            args: entries,
        } = entries_expr
        else {
            panic!("unexpected key-value-pattern entries continuation payload")
        };
        assert!(
            is_system_symbol_name(&entries_head, "InternalContinuationKeyValuePatternEntries"),
            "unexpected key-value-pattern entries tag: {entries_head:?}"
        );
        let remaining_entries = entries
            .into_iter()
            .map(|entry| {
                let Expr::Call { head, args } = entry else {
                    panic!("unexpected key-value-pattern entry continuation payload")
                };
                assert!(
                    is_system_symbol_name(&head, "InternalContinuationKeyValuePatternEntry"),
                    "unexpected key-value-pattern entry tag: {head:?}"
                );
                let mut args = args.into_iter();
                let key = matcher_node_from_continuation_expr(
                    args.next().expect("key-value entry stores its key pattern"),
                );
                let value = matcher_node_from_continuation_expr(
                    args.next()
                        .expect("key-value entry stores its value pattern"),
                );
                let optional = bool::from_continuation_expr(
                    args.next()
                        .expect("key-value entry stores its optional flag"),
                );
                debug_assert!(args.next().is_none());
                (key, value, optional)
            })
            .collect();
        let candidates_expr = args
            .next()
            .expect("key-value choice stores its candidate entries");
        let Expr::Call {
            head: candidates_head,
            args: candidates,
        } = candidates_expr
        else {
            panic!("unexpected key-value candidates continuation payload")
        };
        assert!(
            is_system_symbol_name(&candidates_head, "InternalContinuationKeyValueCandidates"),
            "unexpected key-value candidates tag: {candidates_head:?}"
        );
        let candidates = candidates
            .into_iter()
            .map(|candidate| {
                let Expr::Call { head, args } = candidate else {
                    panic!("unexpected key-value candidate continuation payload")
                };
                assert!(
                    is_system_symbol_name(&head, "InternalContinuationKeyValueCandidate"),
                    "unexpected key-value candidate tag: {head:?}"
                );
                let mut args = args.into_iter();
                let key = args.next().expect("key-value candidate stores its key");
                let value = args.next().expect("key-value candidate stores its value");
                let delayed = bool::from_continuation_expr(
                    args.next()
                        .expect("key-value candidate stores its delayed flag"),
                );
                debug_assert!(args.next().is_none());
                AssociationEntryForm {
                    key,
                    value,
                    delayed,
                }
            })
            .collect();
        let candidate_index =
            usize_from_continuation_expr(args.next().expect("key-value choice stores its cursor"));
        debug_assert!(args.next().is_none());
        Self {
            key_pattern,
            value_pattern,
            delayed,
            remaining_entries,
            candidates,
            candidate_index,
        }
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedKeyValueChoice {
    key_pattern: FrozenMatcherValue<MatcherNode>,
    value_pattern: FrozenMatcherValue<MatcherNode>,
    delayed: bool,
    remaining_entries: FrozenMatcherSequence<(MatcherNode, MatcherNode, bool)>,
    candidates: FrozenMatcherSequence<AssociationEntryForm>,
    candidate_index: usize,
}

impl SuspendedKeyValueChoice {
    fn freeze(state: KeyValueChoiceState) -> Self {
        Self {
            key_pattern: FrozenMatcherValue::freeze(state.key_pattern),
            value_pattern: FrozenMatcherValue::freeze(state.value_pattern),
            delayed: state.delayed,
            remaining_entries: FrozenMatcherSequence::freeze(state.remaining_entries),
            candidates: FrozenMatcherSequence::freeze(state.candidates),
            candidate_index: state.candidate_index,
        }
    }

    pub(super) fn into_owned(self) -> KeyValueChoiceState {
        KeyValueChoiceState {
            key_pattern: self.key_pattern.into_inner(),
            value_pattern: self.value_pattern.into_inner(),
            delayed: self.delayed,
            remaining_entries: self.remaining_entries.into_vec(),
            candidates: self.candidates.into_vec(),
            candidate_index: self.candidate_index,
        }
    }
}
