use super::*;

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_selected_values_owned(
        &mut self,
        pattern: MatcherArg,
        selected_values: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Single(pattern) => match selected_values.as_slice() {
                [value] => self.match_compiled_owned(pattern, value.clone(), bindings),
                _ => Ok(match_outcome(false, bindings)),
            },
            MatcherArg::Sequence(spec) => {
                self.match_compiled_sequence_pattern_owned(spec, selected_values, bindings)
            }
            MatcherArg::Optional(spec) => {
                if selected_values.is_empty() {
                    self.match_compiled_orderless_optional_omission_owned(
                        spec,
                        Vec::new(),
                        Vec::new(),
                        bindings,
                    )
                } else {
                    self.match_compiled_orderless_selected_values_owned(
                        *spec.inner,
                        selected_values,
                        bindings,
                    )
                }
            }
            MatcherArg::PatternSequenceGroup(items) => {
                self.match_compiled_args_owned(items, selected_values, bindings)
            }
            MatcherArg::OrderlessPatternSequenceGroup(items) => {
                self.match_compiled_orderless_args_owned(items, selected_values, bindings)
            }
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_selected_values_owned_cm740_driven(
        &mut self,
        pattern: MatcherArg,
        selected_values: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Single(pattern) => match selected_values.as_slice() {
                [value] => self.match_compiled_owned_cm740_driven(pattern, value.clone(), bindings),
                _ => Ok(match_outcome(false, bindings)),
            },
            MatcherArg::Sequence(spec) => self.match_compiled_sequence_pattern_owned_cm740_driven(
                spec,
                selected_values,
                bindings,
            ),
            MatcherArg::Optional(spec) => {
                if selected_values.is_empty() {
                    self.match_compiled_orderless_optional_omission_owned_cm740_driven(
                        spec,
                        Vec::new(),
                        Vec::new(),
                        bindings,
                    )
                } else {
                    self.match_compiled_orderless_selected_values_owned_cm740_driven(
                        *spec.inner,
                        selected_values,
                        bindings,
                    )
                }
            }
            MatcherArg::PatternSequenceGroup(items) => {
                self.match_compiled_args_owned_cm740_driven(items, selected_values, bindings)
            }
            MatcherArg::OrderlessPatternSequenceGroup(items) => self
                .match_compiled_orderless_args_owned_cm740_driven(items, selected_values, bindings),
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_selected_values_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        selected_values: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Single(pattern) => match selected_values.as_slice() {
                [] => Ok(match_outcome(false, bindings)),
                [value] => self.match_compiled_owned(pattern, value.clone(), bindings),
                _ => self.match_compiled_owned(
                    pattern,
                    flat_match_value(&flat_head, selected_values),
                    bindings,
                ),
            },
            MatcherArg::Sequence(spec) => {
                self.match_compiled_sequence_pattern_owned(spec, selected_values, bindings)
            }
            MatcherArg::Optional(spec) => {
                if selected_values.is_empty() {
                    self.match_compiled_orderless_flat_optional_omission_owned(
                        flat_head,
                        spec,
                        Vec::new(),
                        Vec::new(),
                        bindings,
                    )
                } else {
                    self.match_compiled_orderless_flat_selected_values_owned(
                        flat_head,
                        *spec.inner,
                        selected_values,
                        bindings,
                    )
                }
            }
            MatcherArg::PatternSequenceGroup(items) => self
                .match_compiled_args_with_flat_head_owned(
                    items,
                    selected_values,
                    bindings,
                    flat_head,
                ),
            MatcherArg::OrderlessPatternSequenceGroup(items) => self
                .match_compiled_orderless_args_with_flat_head_owned(
                    items,
                    selected_values,
                    bindings,
                    flat_head,
                ),
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_selected_values_owned_cm740_driven(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        selected_values: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Single(pattern) => match selected_values.as_slice() {
                [] => Ok(match_outcome(false, bindings)),
                [value] => self.match_compiled_owned_cm740_driven(pattern, value.clone(), bindings),
                _ => self.match_compiled_owned_cm740_driven(
                    pattern,
                    flat_match_value(&flat_head, selected_values),
                    bindings,
                ),
            },
            MatcherArg::Sequence(spec) => self.match_compiled_sequence_pattern_owned_cm740_driven(
                spec,
                selected_values,
                bindings,
            ),
            MatcherArg::Optional(spec) => {
                if selected_values.is_empty() {
                    self.match_compiled_orderless_flat_optional_omission_owned_cm740_driven(
                        flat_head,
                        spec,
                        Vec::new(),
                        Vec::new(),
                        bindings,
                    )
                } else {
                    self.match_compiled_orderless_flat_selected_values_owned_cm740_driven(
                        flat_head,
                        *spec.inner,
                        selected_values,
                        bindings,
                    )
                }
            }
            MatcherArg::PatternSequenceGroup(items) => self
                .match_compiled_args_with_flat_head_owned_cm740_driven(
                    items,
                    selected_values,
                    bindings,
                    flat_head,
                ),
            MatcherArg::OrderlessPatternSequenceGroup(items) => self
                .match_compiled_orderless_args_with_flat_head_owned_cm740_driven(
                    items,
                    selected_values,
                    bindings,
                    flat_head,
                ),
        }
    }
}
