use super::*;

impl Session {
    pub(in crate::runtime) fn match_compiled_args_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        if pattern_args.len() > 1 && matches!(pattern_args.first(), Some(MatcherArg::Single(_))) {
            return self.match_compiled_ordered_args_iter_owned(
                pattern_args.into_iter(),
                value_args.into_iter(),
                bindings,
            );
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_arg_and_rest_owned(first, rest_patterns, value_args, bindings)
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime) fn match_compiled_args_owned_cm740_driven(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        if pattern_args.len() > 1 && matches!(pattern_args.first(), Some(MatcherArg::Single(_))) {
            return self.match_compiled_ordered_args_iter_owned_cm740_driven(
                pattern_args.into_iter(),
                value_args.into_iter(),
                bindings,
            );
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_arg_and_rest_owned_cm740_driven(
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn match_compiled_args_with_flat_head_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        flat_head: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_flat_arg_and_rest_owned(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime) fn match_compiled_args_with_flat_head_owned_cm740_driven(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        flat_head: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_flat_arg_and_rest_owned_cm740_driven(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn match_compiled_orderless_args_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_orderless_arg_choices_owned(first, rest_patterns, value_args, bindings)
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime) fn match_compiled_orderless_args_owned_cm740_driven(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_orderless_arg_choices_owned_cm740_driven(
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn match_compiled_orderless_args_with_flat_head_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        flat_head: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_orderless_flat_arg_choices_owned(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    pub(in crate::runtime) fn match_compiled_orderless_args_with_flat_head_owned_cm740_driven(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        flat_head: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_orderless_flat_arg_choices_owned_cm740_driven(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn collect_compiled_args_matches_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if pattern_args.is_empty() {
            return Ok(if value_args.is_empty() {
                vec![bindings]
            } else {
                Vec::new()
            });
        }

        if pattern_args.len() > 1 && matches!(pattern_args.first(), Some(MatcherArg::Single(_))) {
            return self.collect_compiled_ordered_args_iter_owned(
                pattern_args.into_iter(),
                value_args.into_iter(),
                bindings,
            );
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.collect_compiled_arg_and_rest_matches_owned(first, rest_patterns, value_args, bindings)
    }

    pub(in crate::runtime) fn collect_compiled_args_matches_with_flat_head_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        flat_head: Expr,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if pattern_args.is_empty() {
            return Ok(if value_args.is_empty() {
                vec![bindings]
            } else {
                Vec::new()
            });
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.collect_compiled_flat_arg_and_rest_matches_owned(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn collect_compiled_orderless_arg_matches_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if pattern_args.is_empty() {
            return Ok(if value_args.is_empty() {
                vec![bindings]
            } else {
                Vec::new()
            });
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.collect_compiled_orderless_arg_choice_matches_owned(
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn collect_compiled_orderless_arg_matches_with_flat_head_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        flat_head: Expr,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if pattern_args.is_empty() {
            return Ok(if value_args.is_empty() {
                vec![bindings]
            } else {
                Vec::new()
            });
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.collect_compiled_orderless_flat_arg_choice_matches_owned(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }
}
