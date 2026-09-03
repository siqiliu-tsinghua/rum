mod resume_frames;
use resume_frames::*;

use super::{outcome::*, *};
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct FlatSingleChoiceState {
    flat_head: Expr,
    one_identity: bool,
    pattern: MatcherNode,
    rest_patterns: Vec<MatcherArg>,
    value_args: Vec<Expr>,
    bindings: SymbolExprMap,
    consume: usize,
    max_consume: usize,
    wrapped: bool,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedFlatSingleChoiceState {
    flat_head: Expr,
    one_identity: bool,
    pattern: FrozenMatcherValue<MatcherNode>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    bindings: FrozenMatchBindings,
    consume: usize,
    max_consume: usize,
    wrapped: bool,
}

impl SuspendedFlatSingleChoiceState {
    fn freeze(state: FlatSingleChoiceState) -> Self {
        Self {
            flat_head: state.flat_head,
            one_identity: state.one_identity,
            pattern: FrozenMatcherValue::freeze(state.pattern),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            value_args: SharedArgs::new(state.value_args),
            bindings: FrozenMatchBindings::freeze(state.bindings),
            consume: state.consume,
            max_consume: state.max_consume,
            wrapped: state.wrapped,
        }
    }

    fn into_owned(self) -> FlatSingleChoiceState {
        FlatSingleChoiceState {
            flat_head: self.flat_head,
            one_identity: self.one_identity,
            pattern: self.pattern.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            value_args: self.value_args.into_vec(),
            bindings: self.bindings.into_owned(),
            consume: self.consume,
            max_consume: self.max_consume,
            wrapped: self.wrapped,
        }
    }
}

impl FlatSingleChoiceState {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationFlatSingleChoiceState"),
            vec![
                self.flat_head,
                self.one_identity.into_continuation_expr(),
                matcher_node_into_continuation_expr(self.pattern),
                matcher_args_into_continuation_expr(self.rest_patterns),
                self.value_args.into_continuation_expr(),
                binding_map_into_continuation_expr(self.bindings),
                usize_into_continuation_expr(self.consume),
                usize_into_continuation_expr(self.max_consume),
                self.wrapped.into_continuation_expr(),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let mut args = flat_single_state_payload(expr, "InternalContinuationFlatSingleChoiceState");
        let state = Self {
            flat_head: args.next().expect("flat-single choice stores its head"),
            one_identity: bool::from_continuation_expr(
                args.next()
                    .expect("flat-single choice stores its OneIdentity flag"),
            ),
            pattern: matcher_node_from_continuation_expr(
                args.next().expect("flat-single choice stores its matcher"),
            ),
            rest_patterns: matcher_args_from_continuation_expr(
                args.next()
                    .expect("flat-single choice stores its remaining patterns"),
            ),
            value_args: Vec::<Expr>::from_continuation_expr(
                args.next().expect("flat-single choice stores its values"),
            ),
            bindings: binding_map_from_continuation_expr(
                args.next().expect("flat-single choice stores its bindings"),
            ),
            consume: usize_from_continuation_expr(
                args.next().expect("flat-single choice stores its cursor"),
            ),
            max_consume: usize_from_continuation_expr(
                args.next()
                    .expect("flat-single choice stores its maximum cursor"),
            ),
            wrapped: bool::from_continuation_expr(
                args.next()
                    .expect("flat-single choice stores its wrapped flag"),
            ),
        };
        debug_assert!(args.next().is_none());
        state
    }
}

struct FlatSingleCollectionState {
    flat_head: Expr,
    one_identity: bool,
    pattern: MatcherNode,
    rest_patterns: Vec<MatcherArg>,
    value_args: Vec<Expr>,
    bindings: SymbolExprMap,
    consume: usize,
    max_consume: usize,
    wrapped: bool,
    collected: Vec<SymbolExprMap>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedFlatSingleCollectionState {
    flat_head: Expr,
    one_identity: bool,
    pattern: FrozenMatcherValue<MatcherNode>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    bindings: FrozenMatchBindings,
    consume: usize,
    max_consume: usize,
    wrapped: bool,
    collected: PersistentMatchResults,
}

impl SuspendedFlatSingleCollectionState {
    fn freeze(state: FlatSingleCollectionState) -> Self {
        Self {
            flat_head: state.flat_head,
            one_identity: state.one_identity,
            pattern: FrozenMatcherValue::freeze(state.pattern),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            value_args: SharedArgs::new(state.value_args),
            bindings: FrozenMatchBindings::freeze(state.bindings),
            consume: state.consume,
            max_consume: state.max_consume,
            wrapped: state.wrapped,
            collected: PersistentMatchResults::freeze(state.collected),
        }
    }

    fn into_owned(self) -> FlatSingleCollectionState {
        FlatSingleCollectionState {
            flat_head: self.flat_head,
            one_identity: self.one_identity,
            pattern: self.pattern.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            value_args: self.value_args.into_vec(),
            bindings: self.bindings.into_owned(),
            consume: self.consume,
            max_consume: self.max_consume,
            wrapped: self.wrapped,
            collected: self.collected.into_vec(),
        }
    }
}

impl FlatSingleCollectionState {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationFlatSingleCollectionState"),
            vec![
                self.flat_head,
                self.one_identity.into_continuation_expr(),
                matcher_node_into_continuation_expr(self.pattern),
                matcher_args_into_continuation_expr(self.rest_patterns),
                self.value_args.into_continuation_expr(),
                binding_map_into_continuation_expr(self.bindings),
                usize_into_continuation_expr(self.consume),
                usize_into_continuation_expr(self.max_consume),
                self.wrapped.into_continuation_expr(),
                self.collected.into_continuation_expr(),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let mut args =
            flat_single_state_payload(expr, "InternalContinuationFlatSingleCollectionState");
        let state = Self {
            flat_head: args.next().expect("flat-single collection stores its head"),
            one_identity: bool::from_continuation_expr(
                args.next()
                    .expect("flat-single collection stores its OneIdentity flag"),
            ),
            pattern: matcher_node_from_continuation_expr(
                args.next()
                    .expect("flat-single collection stores its matcher"),
            ),
            rest_patterns: matcher_args_from_continuation_expr(
                args.next()
                    .expect("flat-single collection stores its remaining patterns"),
            ),
            value_args: Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("flat-single collection stores its values"),
            ),
            bindings: binding_map_from_continuation_expr(
                args.next()
                    .expect("flat-single collection stores its bindings"),
            ),
            consume: usize_from_continuation_expr(
                args.next()
                    .expect("flat-single collection stores its cursor"),
            ),
            max_consume: usize_from_continuation_expr(
                args.next()
                    .expect("flat-single collection stores its maximum cursor"),
            ),
            wrapped: bool::from_continuation_expr(
                args.next()
                    .expect("flat-single collection stores its wrapped flag"),
            ),
            collected: Vec::<SymbolExprMap>::from_continuation_expr(
                args.next()
                    .expect("flat-single collection stores its results"),
            ),
        };
        debug_assert!(args.next().is_none());
        state
    }
}

fn flat_single_state_payload(expr: Expr, tag: &'static str) -> std::vec::IntoIter<Expr> {
    let Expr::Call { head, args, .. } = expr else {
        panic!("unexpected {tag} continuation payload")
    };
    assert!(
        is_system_symbol_name(&head, tag),
        "unexpected {tag} continuation tag: {head:?}"
    );
    args.into_iter()
}

mod entry;
mod optional_sequence;
mod ordered;
mod orderless;

fn optional_default_values(expr: Expr) -> Vec<Expr> {
    sequence_items(&expr)
        .map(|items| items.to_vec())
        .unwrap_or_else(|| vec![expr])
}

impl Session {
    pub(super) fn match_compiled_arg_and_rest_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Sequence(spec) => {
                let Some(start) = initial_sequence_consume(&spec, &rest_patterns, &value_args)
                else {
                    return Ok(match_outcome(false, bindings));
                };

                self.match_compiled_sequence_choices(
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    start,
                )
            }
            MatcherArg::Optional(spec) => {
                self.match_compiled_optional_choices(spec, rest_patterns, value_args, bindings)
            }
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                let mut expanded = items;
                expanded.extend(rest_patterns);
                self.match_compiled_args_owned(expanded, value_args, bindings)
            }
            MatcherArg::Single(pattern) => {
                let mut value_iter = value_args.into_iter();
                let Some(value) = value_iter.next() else {
                    return Ok(match_outcome(false, bindings));
                };
                let rest_values = value_iter.collect::<Vec<_>>();

                let pattern_result = self.match_compiled_owned(pattern, value, bindings);
                match pattern_result {
                    Ok(outcome) => {
                        if !outcome.matched {
                            Ok(outcome)
                        } else {
                            self.match_compiled_args_owned(
                                rest_patterns,
                                rest_values,
                                outcome.bindings,
                            )
                        }
                    }
                    Err(EvalError::InputRequired { request }) => {
                        let rest_patterns = FrozenMatcherSequence::freeze(rest_patterns);
                        let rest_values = SharedArgs::new(rest_values);
                        crate::resume_or_suspend!(insert self, MatchCompiledArgAndRestOwnedSuspensionFrame { rest_patterns, rest_values });
                        crate::resume_or_suspend!(insert self, ContinueMatchCompiledArgAndRestFrame);
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }

    fn match_compiled_flat_arg_and_rest_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Sequence(spec) => {
                let Some(start) = initial_sequence_consume(&spec, &rest_patterns, &value_args)
                else {
                    return Ok(match_outcome(false, bindings));
                };

                self.match_compiled_flat_sequence_choices(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    start,
                )
            }
            MatcherArg::Optional(spec) => self.match_compiled_flat_optional_choices(
                flat_head,
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                let mut expanded = items;
                expanded.extend(rest_patterns);
                self.match_compiled_args_with_flat_head_owned(
                    expanded, value_args, bindings, flat_head,
                )
            }
            MatcherArg::Single(pattern) => {
                let min_rest = min_required_matcher_arg_count(&rest_patterns);
                let Some(max_consume) = value_args.len().checked_sub(min_rest) else {
                    return Ok(match_outcome(false, bindings));
                };
                if max_consume == 0 {
                    return Ok(match_outcome(false, bindings));
                }

                let one_identity = matches!(&flat_head, Expr::Atom(Atom::Symbol(s))
                    if self.attribute_set_for_symbol(s).contains(Attribute::OneIdentity));
                self.match_compiled_flat_single_choices_owned(
                    flat_head,
                    one_identity,
                    pattern,
                    rest_patterns,
                    value_args,
                    bindings,
                    1,
                    max_consume,
                )
            }
        }
    }

    fn match_compiled_flat_single_choices_owned(
        &mut self,
        flat_head: Expr,
        one_identity: bool,
        pattern: MatcherNode,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        consume: usize,
        max_consume: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.match_flat_single_choice(FlatSingleChoiceState {
            flat_head,
            one_identity,
            pattern,
            rest_patterns,
            value_args,
            bindings,
            consume,
            max_consume,
            wrapped: false,
        })
    }

    fn match_flat_single_choice(
        &mut self,
        state: FlatSingleChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let selected_value = if state.wrapped {
            Expr::call(state.flat_head.clone(), vec![state.value_args[0].clone()])
        } else {
            flat_match_value(
                &state.flat_head,
                state
                    .value_args
                    .iter()
                    .take(state.consume)
                    .cloned()
                    .collect(),
            )
        };
        let rest_values = state
            .value_args
            .iter()
            .skip(state.consume)
            .cloned()
            .collect::<Vec<_>>();
        let pattern_result = self.match_compiled_owned(
            state.pattern.clone(),
            selected_value,
            state.bindings.clone(),
        );
        match pattern_result {
            Ok(outcome) => self.finish_flat_single_pattern(state, rest_values, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedFlatSingleChoiceState::freeze(state);
                let rest_values = SharedArgs::new(rest_values);
                crate::resume_or_suspend!(insert self, MatchFlatSingleChoiceSuspensionFrame { state, rest_values });
                crate::resume_or_suspend!(insert self, ContinueFlatSinglePatternFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_flat_single_pattern(
        &mut self,
        mut state: FlatSingleChoiceState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            state.bindings = outcome.bindings;
            return self.advance_flat_single_choice(state);
        }

        let rest_result = self.match_compiled_args_with_flat_head_owned(
            state.rest_patterns.clone(),
            rest_values,
            outcome.bindings,
            state.flat_head.clone(),
        );
        match rest_result {
            Ok(outcome) => self.finish_flat_single_rest(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedFlatSingleChoiceState::freeze(state);
                crate::resume_or_suspend!(insert self, FinishFlatSinglePatternSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueFlatSingleRestFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_flat_single_rest(
        &mut self,
        state: FlatSingleChoiceState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.advance_flat_single_choice(state)
        }
    }

    fn advance_flat_single_choice(
        &mut self,
        mut state: FlatSingleChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !state.wrapped && state.consume == 1 && !state.one_identity {
            state.wrapped = true;
            return self.match_flat_single_choice(state);
        }

        state.wrapped = false;
        state.consume += 1;
        if state.consume <= state.max_consume {
            self.match_flat_single_choice(state)
        } else {
            Ok(match_outcome(false, state.bindings))
        }
    }

    fn collect_compiled_arg_and_rest_matches_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        match pattern {
            MatcherArg::Sequence(spec) => {
                let min_rest = min_required_matcher_arg_count(&rest_patterns);
                let Some(max_len) = max_sequence_consume(&spec, min_rest, value_args.len()) else {
                    return Ok(Vec::new());
                };
                let consumes = sequence_candidate_lengths(spec.min_len, max_len, spec.greediness);

                self.collect_compiled_sequence_choice_matches_owned(
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    consumes,
                    0,
                    Vec::new(),
                )
            }
            MatcherArg::Optional(spec) => self.collect_compiled_optional_matches_owned(
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                let mut expanded = items;
                expanded.extend(rest_patterns);
                self.collect_compiled_args_matches_owned(expanded, value_args, bindings)
            }
            MatcherArg::Single(pattern) => {
                let mut value_iter = value_args.into_iter();
                let Some(value) = value_iter.next() else {
                    return Ok(Vec::new());
                };
                let rest_values = value_iter.collect::<Vec<_>>();

                let pattern_result = self.match_compiled_owned(pattern, value, bindings);
                match pattern_result {
                    Ok(outcome) => {
                        if !outcome.matched {
                            Ok(Vec::new())
                        } else {
                            self.collect_compiled_args_matches_owned(
                                rest_patterns,
                                rest_values,
                                outcome.bindings,
                            )
                        }
                    }
                    Err(EvalError::InputRequired { request }) => {
                        let rest_patterns = FrozenMatcherSequence::freeze(rest_patterns);
                        let rest_values = SharedArgs::new(rest_values);
                        crate::resume_or_suspend!(insert self, CollectCompiledArgAndRestMatchesOwnedSuspensionFrame { rest_patterns, rest_values });
                        crate::resume_or_suspend!(insert self, ContinueCollectCompiledArgAndRestFrame);
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }

    fn collect_compiled_flat_arg_and_rest_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        match pattern {
            MatcherArg::Sequence(spec) => {
                let min_rest = min_required_matcher_arg_count(&rest_patterns);
                let Some(max_len) = max_sequence_consume(&spec, min_rest, value_args.len()) else {
                    return Ok(Vec::new());
                };
                let consumes = sequence_candidate_lengths(spec.min_len, max_len, spec.greediness);

                self.collect_compiled_flat_sequence_choice_matches_owned(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                    consumes,
                    0,
                    Vec::new(),
                )
            }
            MatcherArg::Optional(spec) => self.collect_compiled_flat_optional_matches_owned(
                flat_head,
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                let mut expanded = items;
                expanded.extend(rest_patterns);
                self.collect_compiled_args_matches_with_flat_head_owned(
                    expanded, value_args, bindings, flat_head,
                )
            }
            MatcherArg::Single(pattern) => {
                let min_rest = min_required_matcher_arg_count(&rest_patterns);
                let Some(max_consume) = value_args.len().checked_sub(min_rest) else {
                    return Ok(Vec::new());
                };
                if max_consume == 0 {
                    return Ok(Vec::new());
                }

                let one_identity = matches!(&flat_head, Expr::Atom(Atom::Symbol(s))
                    if self.attribute_set_for_symbol(s).contains(Attribute::OneIdentity));
                self.collect_compiled_flat_single_choice_matches_owned(
                    flat_head,
                    one_identity,
                    pattern,
                    rest_patterns,
                    value_args,
                    bindings,
                    1,
                    max_consume,
                    Vec::new(),
                )
            }
        }
    }

    fn collect_compiled_flat_single_choice_matches_owned(
        &mut self,
        flat_head: Expr,
        one_identity: bool,
        pattern: MatcherNode,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        consume: usize,
        max_consume: usize,
        collected: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.collect_flat_single_choice(FlatSingleCollectionState {
            flat_head,
            one_identity,
            pattern,
            rest_patterns,
            value_args,
            bindings,
            consume,
            max_consume,
            wrapped: false,
            collected,
        })
    }

    fn collect_flat_single_choice(
        &mut self,
        state: FlatSingleCollectionState,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if state.consume > state.max_consume {
            return Ok(state.collected);
        }

        let selected_value = if state.wrapped {
            Expr::call(state.flat_head.clone(), vec![state.value_args[0].clone()])
        } else {
            flat_match_value(
                &state.flat_head,
                state
                    .value_args
                    .iter()
                    .take(state.consume)
                    .cloned()
                    .collect(),
            )
        };
        let rest_values = state
            .value_args
            .iter()
            .skip(state.consume)
            .cloned()
            .collect::<Vec<_>>();
        let pattern_result = self.match_compiled_owned(
            state.pattern.clone(),
            selected_value,
            state.bindings.clone(),
        );
        match pattern_result {
            Ok(outcome) => self.finish_flat_single_collection_pattern(state, rest_values, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedFlatSingleCollectionState::freeze(state);
                let rest_values = SharedArgs::new(rest_values);
                crate::resume_or_suspend!(insert self, CollectFlatSingleChoiceSuspensionFrame { state, rest_values });
                crate::resume_or_suspend!(insert self, ContinueFlatSingleCollectionPatternFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_flat_single_collection_pattern(
        &mut self,
        state: FlatSingleCollectionState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if !outcome.matched {
            return self.advance_flat_single_collection(state);
        }

        let rest_result = self.collect_compiled_args_matches_with_flat_head_owned(
            state.rest_patterns.clone(),
            rest_values,
            outcome.bindings,
            state.flat_head.clone(),
        );
        match rest_result {
            Ok(matches) => self.finish_flat_single_collection_rest(state, matches),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedFlatSingleCollectionState::freeze(state);
                crate::resume_or_suspend!(insert self, FinishFlatSingleCollectionPatternSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueFlatSingleCollectionRestFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_flat_single_collection_rest(
        &mut self,
        mut state: FlatSingleCollectionState,
        mut matches: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        state.collected.append(&mut matches);
        self.advance_flat_single_collection(state)
    }

    fn advance_flat_single_collection(
        &mut self,
        mut state: FlatSingleCollectionState,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if !state.wrapped && state.consume == 1 && !state.one_identity {
            state.wrapped = true;
        } else {
            state.wrapped = false;
            state.consume += 1;
        }
        self.collect_flat_single_choice(state)
    }
}

fn initial_sequence_consume(
    spec: &CompiledSequencePattern,
    rest_patterns: &[MatcherArg],
    value_args: &[Expr],
) -> Option<usize> {
    let min_rest = min_required_matcher_arg_count(rest_patterns);
    let max_len = max_sequence_consume(spec, min_rest, value_args.len())?;

    Some(match spec.greediness {
        SequenceGreediness::Shortest => spec.min_len,
        SequenceGreediness::Longest => max_len,
    })
}

fn max_sequence_consume(
    spec: &CompiledSequencePattern,
    min_rest: usize,
    value_len: usize,
) -> Option<usize> {
    if value_len < spec.min_len + min_rest {
        return None;
    }

    Some(
        spec.max_len
            .map(|limit| limit.min(value_len.saturating_sub(min_rest)))
            .unwrap_or_else(|| value_len.saturating_sub(min_rest)),
    )
}

fn next_sequence_consume(
    greediness: SequenceGreediness,
    current: usize,
    min_len: usize,
    max_len: usize,
) -> Option<usize> {
    match greediness {
        SequenceGreediness::Shortest => (current < max_len).then_some(current + 1),
        SequenceGreediness::Longest => (current > min_len).then_some(current - 1),
    }
}

fn sequence_candidate_lengths(
    min_len: usize,
    max_len: usize,
    greediness: SequenceGreediness,
) -> Vec<usize> {
    match greediness {
        SequenceGreediness::Shortest => (min_len..=max_len).collect(),
        SequenceGreediness::Longest => (min_len..=max_len).rev().collect(),
    }
}
fn flat_match_value(head: &Expr, values: Vec<Expr>) -> Expr {
    match values.as_slice() {
        [single] => single.clone(),
        _ => Expr::call(head.clone(), values),
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedFlatSingleChoiceState>();
    assert_traces::<SuspendedFlatSingleCollectionState>();
};
