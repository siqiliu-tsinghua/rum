use super::*;

fn cases_level_spec_expr(spec: CasesLevelSpec) -> Expr {
    Expr::call(
        system_expr("InternalContinuationCasesLevelSpec"),
        vec![
            Expr::integer(BigInt::from(spec.min)),
            spec.max
                .map(|value| Expr::integer(BigInt::from(value)))
                .unwrap_or_else(|| system_expr("Infinity")),
        ],
    )
}

fn cases_level_spec_from_expr(expr: Expr) -> CasesLevelSpec {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(&head, "InternalContinuationCasesLevelSpec")
                && args.len() == 2 =>
        {
            let mut args = Expr::unwrap_shared_args(args);
            let max_expr = args.pop().expect("cases-level payload stores max");
            let min_expr = args.pop().expect("cases-level payload stores min");
            let min = expr_as_signed_isize(&min_expr)
                .expect("cases-level payload min should be a signed integer");
            let max = if is_system_symbol_name(&max_expr, "Infinity") {
                None
            } else {
                Some(
                    expr_as_signed_isize(&max_expr)
                        .expect("cases-level payload max should be a signed integer"),
                )
            };

            CasesLevelSpec::range(min, max)
        }
        other => panic!("unexpected continuation payload for CasesLevelSpec: {other:?}"),
    }
}

impl ContinuationValue for Option<CasesLevelSpec> {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Some(spec) => Expr::call(
                system_expr("InternalContinuationSomeCasesLevelSpec"),
                vec![cases_level_spec_expr(spec)],
            ),
            None => Expr::call(system_expr("InternalContinuationNone"), vec![]),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationSomeCasesLevelSpec")
                    && args.len() == 1 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let spec_expr = args
                    .pop()
                    .expect("some-cases-level-spec payload stores one item");
                Some(cases_level_spec_from_expr(spec_expr))
            }
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationNone") =>
            {
                debug_assert!(args.is_empty(), "none continuation payloads are empty");
                None
            }
            other => Some(cases_level_spec_from_expr(other)),
        }
    }
}

impl ContinuationValue for CasesTraversalConfig {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationCasesTraversalConfig"),
            vec![
                cases_level_spec_expr(self.levels),
                self.max_matches
                    .map(|value| Expr::integer(BigInt::from(value)))
                    .unwrap_or_else(|| system_expr("InternalContinuationNone")),
                system_expr(if self.include_heads { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationCasesTraversalConfig")
                    && args.len() == 3 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let include_heads_expr = args
                    .pop()
                    .expect("cases-traversal payload stores include-heads");
                let max_matches_expr = args
                    .pop()
                    .expect("cases-traversal payload stores max-matches");
                let levels_expr = args.pop().expect("cases-traversal payload stores levels");

                Self {
                    levels: cases_level_spec_from_expr(levels_expr),
                    max_matches: if is_system_symbol_name(
                        &max_matches_expr,
                        "InternalContinuationNone",
                    ) {
                        None
                    } else {
                        expr_as_non_negative_usize(&max_matches_expr)
                    },
                    include_heads: is_system_symbol_name(&include_heads_expr, "True"),
                }
            }
            other => panic!("unexpected continuation payload for CasesTraversalConfig: {other:?}"),
        }
    }
}

impl ContinuationValue for CasesTraversalResolution {
    fn into_continuation_expr(self) -> Expr {
        match self {
            CasesTraversalResolution::Valid(config) => Expr::call(
                system_expr("InternalContinuationCasesTraversalValid"),
                vec![config.into_continuation_expr()],
            ),
            CasesTraversalResolution::Invalid(error) => Expr::call(
                system_expr("InternalContinuationCasesTraversalInvalid"),
                vec![error.into_continuation_expr()],
            ),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationCasesTraversalValid")
                    && args.len() == 1 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let config_expr = args.pop().expect("valid cases-traversal stores config");
                Self::Valid(CasesTraversalConfig::from_continuation_expr(config_expr))
            }
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationCasesTraversalInvalid")
                    && args.len() == 1 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let error_expr = args.pop().expect("invalid cases-traversal stores error");
                Self::Invalid(CasesTraversalNormalizationError::from_continuation_expr(
                    error_expr,
                ))
            }
            other => {
                panic!("unexpected continuation payload for CasesTraversalResolution: {other:?}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_cases_level_spec_continuation_round_trips_some_none_and_bare_spec() {
        let spec = CasesLevelSpec::range(-2, Some(3));
        let encoded = Some(spec).into_continuation_expr();
        assert_eq!(
            Option::<CasesLevelSpec>::from_continuation_expr(encoded),
            Some(spec)
        );

        let none = Option::<CasesLevelSpec>::None.into_continuation_expr();
        assert_eq!(Option::<CasesLevelSpec>::from_continuation_expr(none), None);

        let bare = cases_level_spec_expr(CasesLevelSpec::range(-1, None));
        assert_eq!(
            Option::<CasesLevelSpec>::from_continuation_expr(bare),
            Some(CasesLevelSpec::range(-1, None))
        );
    }
}
