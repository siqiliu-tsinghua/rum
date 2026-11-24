use super::candidates::orderless_candidate_choices;
use super::*;

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_arg_choices_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self.match_compiled_orderless_optional_choices(
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, false, None);
                self.match_compiled_orderless_choices_owned(
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                )
            }
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_arg_choices_owned_cm740_driven(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self
                .match_compiled_orderless_optional_choices_cm740_driven(
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, false, None);
                self.match_compiled_orderless_choices_owned(
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                )
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_arg_choices_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self.match_compiled_orderless_flat_optional_choices(
                flat_head,
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            other => {
                let one_identity = matches!(&flat_head, Expr::Atom(Atom::Symbol(s))
                    if self.attribute_set_for_symbol(s).contains(Attribute::OneIdentity));
                let choices = orderless_candidate_choices(
                    &other,
                    &rest_patterns,
                    &value_args,
                    true,
                    (!one_identity).then_some(&flat_head),
                );
                self.match_compiled_orderless_flat_choices_owned(
                    flat_head,
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                )
            }
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_arg_choices_owned_cm740_driven(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self
                .match_compiled_orderless_flat_optional_choices_cm740_driven(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                ),
            other => {
                let one_identity = matches!(&flat_head, Expr::Atom(Atom::Symbol(s))
                    if self.attribute_set_for_symbol(s).contains(Attribute::OneIdentity));
                let choices = orderless_candidate_choices(
                    &other,
                    &rest_patterns,
                    &value_args,
                    true,
                    (!one_identity).then_some(&flat_head),
                );
                self.match_compiled_orderless_flat_choices_owned(
                    flat_head,
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                )
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_arg_choice_matches_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self.collect_compiled_orderless_optional_matches_owned(
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, false, None);
                self.collect_compiled_orderless_choices_matches_owned(
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                    Vec::new(),
                )
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_flat_arg_choice_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self
                .collect_compiled_orderless_flat_optional_matches_owned(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                ),
            other => {
                let one_identity = matches!(&flat_head, Expr::Atom(Atom::Symbol(s))
                    if self.attribute_set_for_symbol(s).contains(Attribute::OneIdentity));
                let choices = orderless_candidate_choices(
                    &other,
                    &rest_patterns,
                    &value_args,
                    true,
                    (!one_identity).then_some(&flat_head),
                );
                self.collect_compiled_orderless_flat_choices_matches_owned(
                    flat_head,
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                    Vec::new(),
                )
            }
        }
    }
}
