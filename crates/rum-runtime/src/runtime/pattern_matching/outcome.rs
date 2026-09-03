use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct CompiledMatchOutcome {
    pub(in crate::runtime) matched: bool,
    pub(in crate::runtime) bindings: SymbolExprMap,
}

impl ContinuationValue for CompiledMatchOutcome {
    fn into_continuation_expr(self) -> Expr {
        let bindings = self
            .bindings
            .into_iter()
            .map(|(symbol, value)| {
                Expr::call(
                    system_expr("Rule"),
                    vec![Expr::Atom(Atom::Symbol(symbol)), value],
                )
            })
            .collect::<Vec<_>>();

        Expr::call(
            system_expr("List"),
            vec![
                system_expr(if self.matched { "True" } else { "False" }),
                Expr::call(system_expr("List"), bindings),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "List") && args.len() == 2 =>
            {
                let mut items = args.into_iter();
                let matched_expr = items.next().expect("match outcomes store a boolean slot");
                let bindings_expr = items
                    .next()
                    .expect("match outcomes store a binding-list slot");

                Self {
                    matched: is_system_symbol_name(&matched_expr, "True"),
                    bindings: bindings_from_continuation_expr(bindings_expr),
                }
            }
            other => panic!("unexpected continuation payload for compiled matcher: {other:?}"),
        }
    }
}

fn bindings_from_continuation_expr(expr: Expr) -> SymbolExprMap {
    let Expr::Call { head, args } = expr else {
        panic!("binding payload should be a List expression");
    };
    assert!(
        is_system_symbol_name(&head, "List"),
        "binding payload should use List"
    );

    args.into_iter()
        .map(|item| {
            let Expr::Call { head, args } = item else {
                panic!("binding payload items should be Rule expressions");
            };
            assert!(
                is_system_symbol_name(&head, "Rule") && args.len() == 2,
                "binding payload items should be Rule[symbol, value]"
            );
            let mut parts = args.into_iter();
            let name_expr = parts.next().expect("binding rules store a symbol");
            let value = parts.next().expect("binding rules store a value");
            let Expr::Atom(Atom::Symbol(symbol)) = name_expr else {
                panic!("binding rules should store a symbol on the lhs");
            };

            (symbol, value)
        })
        .collect()
}

pub(super) fn match_outcome(matched: bool, bindings: SymbolExprMap) -> CompiledMatchOutcome {
    CompiledMatchOutcome { matched, bindings }
}
