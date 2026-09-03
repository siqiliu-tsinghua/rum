use super::*;

fn rewrite_rule_info_expr(rule: RewriteRuleInfo) -> Expr {
    Expr::call(
        system_expr(if rule.delayed { "RuleDelayed" } else { "Rule" }),
        vec![rule.lhs, rule.rhs],
    )
}

fn rewrite_rule_info_from_expr(expr: Expr) -> RewriteRuleInfo {
    match expr {
        Expr::Call { head, args, .. }
            if (is_system_symbol_name(&head, "Rule")
                || is_system_symbol_name(&head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            let mut args = Expr::unwrap_shared_args(args);
            let rhs = args.pop().expect("rewrite-rule continuation rhs exists");
            let lhs = args.pop().expect("rewrite-rule continuation lhs exists");
            RewriteRuleInfo {
                lhs,
                rhs,
                delayed: is_system_symbol_name(&head, "RuleDelayed"),
            }
        }
        other => RewriteRuleInfo {
            lhs: other,
            rhs: system_expr("Null"),
            delayed: false,
        },
    }
}

impl ContinuationValue for Option<Vec<RewriteRuleInfo>> {
    fn into_continuation_expr(self) -> Expr {
        match self {
            Some(values) => Expr::call(
                system_expr("InternalContinuationSomeRewriteRuleInfos"),
                vec![Expr::call(
                    system_expr("List"),
                    values.into_iter().map(rewrite_rule_info_expr).collect(),
                )],
            ),
            None => Expr::call(system_expr("InternalContinuationNone"), vec![]),
        }
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationSomeRewriteRuleInfos") =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let Some(list_expr) = args.pop() else {
                    return Some(Vec::new());
                };
                debug_assert!(
                    args.is_empty(),
                    "rewrite-rule continuation payloads use exactly one list slot"
                );

                match list_expr {
                    Expr::Call { head, args, .. } if is_system_symbol_name(&head, "List") => {
                        Some(args.into_iter().map(rewrite_rule_info_from_expr).collect())
                    }
                    other => Some(vec![rewrite_rule_info_from_expr(other)]),
                }
            }
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationNone") =>
            {
                debug_assert!(args.is_empty(), "none continuation payloads are empty");
                None
            }
            Expr::Call { head, args, .. } if is_system_symbol_name(&head, "List") => {
                Some(args.into_iter().map(rewrite_rule_info_from_expr).collect())
            }
            other => Some(vec![rewrite_rule_info_from_expr(other)]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(lhs: Expr, rhs: Expr, delayed: bool) -> RewriteRuleInfo {
        RewriteRuleInfo { lhs, rhs, delayed }
    }

    #[test]
    fn option_rewrite_rule_info_continuation_round_trips_some_none_and_fallbacks() {
        let rules = vec![
            rule(Expr::symbol("x"), Expr::integer(BigInt::from(1)), false),
            rule(Expr::symbol("y"), Expr::string("rhs"), true),
        ];
        let encoded = Some(rules.clone()).into_continuation_expr();
        assert_eq!(
            Option::<Vec<RewriteRuleInfo>>::from_continuation_expr(encoded),
            Some(rules.clone())
        );

        let none = Option::<Vec<RewriteRuleInfo>>::None.into_continuation_expr();
        assert_eq!(
            Option::<Vec<RewriteRuleInfo>>::from_continuation_expr(none),
            None
        );

        let bare_list = Expr::call(
            system_expr("List"),
            rules
                .clone()
                .into_iter()
                .map(rewrite_rule_info_expr)
                .collect(),
        );
        assert_eq!(
            Option::<Vec<RewriteRuleInfo>>::from_continuation_expr(bare_list),
            Some(rules)
        );

        let fallback_lhs = Expr::symbol("fallback");
        assert_eq!(
            Option::<Vec<RewriteRuleInfo>>::from_continuation_expr(fallback_lhs.clone()),
            Some(vec![rule(fallback_lhs, system_expr("Null"), false)])
        );
    }
}
