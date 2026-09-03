use super::*;

impl ContinuationValue for Expr {
    fn into_continuation_expr(self) -> Expr {
        self
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        expr
    }
}

impl ContinuationValue for bool {
    fn into_continuation_expr(self) -> Expr {
        system_expr(if self { "True" } else { "False" })
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        is_system_symbol_name(&expr, "True")
    }
}

impl ContinuationValue for String {
    fn into_continuation_expr(self) -> Expr {
        Expr::string(self)
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Atom(Atom::String(text)) => text,
            other => other.to_string(),
        }
    }
}

impl ContinuationValue for Option<Expr> {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Some(expr) => Expr::call(system_expr("InternalContinuationSome"), vec![expr]),
            None => Expr::call(system_expr("InternalContinuationNone"), vec![]),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationSome") =>
            {
                let mut items = args.into_iter();
                let first = items.next();
                debug_assert!(
                    items.next().is_none(),
                    "option continuation payloads use exactly one item for Some"
                );
                first
            }
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationNone") =>
            {
                debug_assert!(args.is_empty(), "none continuation payloads are empty");
                None
            }
            other => Some(other),
        }
    }
}

impl ContinuationValue for Vec<Expr> {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(system_expr("InternalContinuationVec"), self)
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationVec") =>
            {
                args.into_vec()
            }
            other => vec![other],
        }
    }
}

impl ContinuationValue for Vec<SymbolExprMap> {
    fn into_continuation_expr(self) -> Expr {
        let items = self
            .into_iter()
            .map(|bindings| {
                Expr::call(
                    system_expr("InternalContinuationBindingMap"),
                    bindings
                        .into_iter()
                        .map(|(symbol, value)| {
                            Expr::call(
                                system_expr("Rule"),
                                vec![Expr::Atom(Atom::Symbol(symbol)), value],
                            )
                        })
                        .collect(),
                )
            })
            .collect();

        Expr::call(system_expr("InternalContinuationBindingVec"), items)
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationBindingVec") =>
            {
                args.into_iter()
                    .map(|item| match item {
                        Expr::Call { head, args, .. }
                            if is_system_symbol_name(&head, "InternalContinuationBindingMap") =>
                        {
                            args.into_iter()
                                .map(|entry| match entry {
                                    Expr::Call { head, args, .. }
                                        if is_system_symbol_name(&head, "Rule")
                                            && args.len() == 2 =>
                                    {
                                        let mut parts = args.into_iter();
                                        let symbol_expr =
                                            parts.next().expect("rule stores lhs symbol");
                                        let value =
                                            parts.next().expect("rule stores rhs value");
                                        let Expr::Atom(Atom::Symbol(symbol)) = symbol_expr else {
                                            panic!(
                                                "binding continuation lhs should be a symbol: {symbol_expr:?}"
                                            );
                                        };
                                        (symbol, value)
                                    }
                                    other => {
                                        panic!(
                                            "unexpected binding continuation entry payload: {other:?}"
                                        )
                                    }
                                })
                                .collect()
                        }
                        other => panic!(
                            "unexpected binding continuation item payload: {other:?}"
                        ),
                    })
                    .collect()
            }
            other => panic!("unexpected continuation payload for binding vec: {other:?}"),
        }
    }
}

impl ContinuationValue for Option<Vec<Expr>> {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Some(values) => Expr::call(
                system_expr("InternalContinuationSomeVec"),
                vec![Expr::call(system_expr("List"), values)],
            ),
            None => Expr::call(system_expr("InternalContinuationNone"), vec![]),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationSomeVec") =>
            {
                let mut items = args.into_iter();
                let Some(list_expr) = items.next() else {
                    return Some(Vec::new());
                };
                debug_assert!(
                    items.next().is_none(),
                    "some-vec continuation payloads use exactly one list slot"
                );

                match list_expr {
                    Expr::Call { head, args, .. } if is_system_symbol_name(&head, "List") => {
                        Some(args.into_vec())
                    }
                    other => Some(vec![other]),
                }
            }
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationNone") =>
            {
                debug_assert!(args.is_empty(), "none continuation payloads are empty");
                None
            }
            Expr::Call { head, args, .. } if is_system_symbol_name(&head, "List") => {
                Some(args.into_vec())
            }
            other => Some(vec![other]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_vec_continuation_round_trips_multiple_symbol_maps() {
        let mut first = SymbolExprMap::default();
        first.insert(Symbol::from_name("x"), Expr::integer(BigInt::from(1)));
        first.insert(Symbol::from_name("y"), Expr::string("two"));

        let mut second = SymbolExprMap::default();
        second.insert(
            Symbol::from_name("z"),
            Expr::call(system_expr("f"), vec![Expr::integer(BigInt::from(3))]),
        );

        let original = vec![first, second];
        let encoded = original.clone().into_continuation_expr();
        let decoded = Vec::<SymbolExprMap>::from_continuation_expr(encoded);

        assert_eq!(decoded, original);
    }

    #[test]
    fn option_vec_expr_continuation_round_trips_some_none_and_fallback_payloads() {
        let values = vec![Expr::integer(BigInt::from(1)), Expr::symbol("x")];
        let encoded = Some(values.clone()).into_continuation_expr();
        assert_eq!(
            Option::<Vec<Expr>>::from_continuation_expr(encoded),
            Some(values.clone())
        );

        let none = Option::<Vec<Expr>>::None.into_continuation_expr();
        assert_eq!(Option::<Vec<Expr>>::from_continuation_expr(none), None);

        let legacy_list = Expr::call(system_expr("List"), values.clone());
        assert_eq!(
            Option::<Vec<Expr>>::from_continuation_expr(legacy_list),
            Some(values)
        );

        let singleton = Expr::symbol("fallback");
        assert_eq!(
            Option::<Vec<Expr>>::from_continuation_expr(singleton.clone()),
            Some(vec![singleton])
        );
    }
}
