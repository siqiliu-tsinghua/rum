use super::*;

pub(super) enum DefinitionLhsSyntaxCallPlan {
    Opaque,
    Structural {
        selection: DefinitionLhsStructuralSelection,
        child_region: DefinitionLhsRegion,
    },
    Ordinary,
}

pub(super) fn definition_lhs_syntax_call_plan(
    head: &Expr,
    arg_count: usize,
    region: DefinitionLhsRegion,
) -> DefinitionLhsSyntaxCallPlan {
    let Some(symbol) = expr_as_symbol(head).filter(|symbol| symbol.context() == SYSTEM_CONTEXT)
    else {
        return DefinitionLhsSyntaxCallPlan::Ordinary;
    };

    match symbol.name() {
        "MessageName"
        | heads::patterns::DEFAULT
        | "Attributes"
        | "Options"
        | "DefaultValues"
        | "OwnValues"
        | "DownValues"
        | "UpValues"
        | "SubValues"
        | "SyntaxInformation" => return DefinitionLhsSyntaxCallPlan::Opaque,
        // Unset[TagSet[tag, lhs]] is the explicit spelling retained for
        // tagged removal. Keep the tag opaque and prepare only the embedded
        // definition lhs before prepared_unset_target classifies it.
        heads::rules::TAG_SET if arg_count == 2 => {
            return structural_one(1, DefinitionLhsRegion::ActivePattern);
        }
        _ => {}
    }

    let Some(pattern_head) = matcher_pattern_head_name(symbol.name()) else {
        return DefinitionLhsSyntaxCallPlan::Ordinary;
    };

    match pattern_head {
        MatcherPatternHead::HoldPattern
        | MatcherPatternHead::OptionsPattern
        | MatcherPatternHead::Verbatim => DefinitionLhsSyntaxCallPlan::Opaque,
        MatcherPatternHead::Pattern if arg_count == 2 => {
            structural_one(1, DefinitionLhsRegion::ActivePattern)
        }
        MatcherPatternHead::Pattern => DefinitionLhsSyntaxCallPlan::Ordinary,
        MatcherPatternHead::Blank
        | MatcherPatternHead::BlankSequence
        | MatcherPatternHead::BlankNullSequence => {
            structural_all(DefinitionLhsRegion::ActivePattern)
        }
        MatcherPatternHead::PatternTest | MatcherPatternHead::Condition if arg_count == 2 => {
            structural_one(0, DefinitionLhsRegion::ActivePattern)
        }
        MatcherPatternHead::PatternTest | MatcherPatternHead::Condition => {
            DefinitionLhsSyntaxCallPlan::Ordinary
        }
        MatcherPatternHead::Alternatives
        | MatcherPatternHead::Except
        | MatcherPatternHead::Optional
        | MatcherPatternHead::Repeated
        | MatcherPatternHead::RepeatedNull => structural_all(recursive_pattern_region(region)),
        MatcherPatternHead::KeyValuePattern
        | MatcherPatternHead::Longest
        | MatcherPatternHead::OrderlessPatternSequence
        | MatcherPatternHead::PatternSequence
        | MatcherPatternHead::Shortest => raw_pattern_tree(),
    }
}

fn structural_all(child_region: DefinitionLhsRegion) -> DefinitionLhsSyntaxCallPlan {
    DefinitionLhsSyntaxCallPlan::Structural {
        selection: DefinitionLhsStructuralSelection::All { cursor: 0 },
        child_region,
    }
}

fn structural_one(index: usize, child_region: DefinitionLhsRegion) -> DefinitionLhsSyntaxCallPlan {
    DefinitionLhsSyntaxCallPlan::Structural {
        selection: DefinitionLhsStructuralSelection::One { index: Some(index) },
        child_region,
    }
}

fn raw_pattern_tree() -> DefinitionLhsSyntaxCallPlan {
    DefinitionLhsSyntaxCallPlan::Structural {
        selection: DefinitionLhsStructuralSelection::RawTree {
            head_pending: false,
            cursor: 0,
        },
        child_region: DefinitionLhsRegion::RawPattern,
    }
}

fn recursive_pattern_region(region: DefinitionLhsRegion) -> DefinitionLhsRegion {
    match region {
        DefinitionLhsRegion::RawPattern => DefinitionLhsRegion::RawPattern,
        DefinitionLhsRegion::Root | DefinitionLhsRegion::ActivePattern => {
            DefinitionLhsRegion::ActivePattern
        }
    }
}

pub(in crate::runtime::definitions) fn definition_lhs_contains_pattern_syntax(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            matcher_pattern_head(head).is_some()
                || definition_lhs_contains_pattern_syntax(head)
                || args.iter().any(definition_lhs_contains_pattern_syntax)
        }
    }
}

pub(in crate::runtime::definitions) fn definition_lhs_blank_head(head: &Expr) -> bool {
    matches!(
        matcher_pattern_head(head),
        Some(
            MatcherPatternHead::Blank
                | MatcherPatternHead::BlankSequence
                | MatcherPatternHead::BlankNullSequence
        )
    )
}

pub(in crate::runtime::definitions) fn definition_lhs_pattern_wrapper_head(head: &Expr) -> bool {
    matches!(
        matcher_pattern_head(head),
        Some(
            MatcherPatternHead::Alternatives
                | MatcherPatternHead::Optional
                | MatcherPatternHead::Except
                | MatcherPatternHead::Repeated
                | MatcherPatternHead::RepeatedNull
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_matcher_head_has_an_explicit_lhs_policy() {
        for pattern_head in MatcherPatternHead::ALL {
            let arg_count = match pattern_head {
                MatcherPatternHead::Pattern
                | MatcherPatternHead::PatternTest
                | MatcherPatternHead::Condition => 2,
                MatcherPatternHead::KeyValuePattern
                | MatcherPatternHead::Longest
                | MatcherPatternHead::OptionsPattern
                | MatcherPatternHead::Shortest
                | MatcherPatternHead::Verbatim => 1,
                MatcherPatternHead::Alternatives
                | MatcherPatternHead::Blank
                | MatcherPatternHead::BlankSequence
                | MatcherPatternHead::BlankNullSequence
                | MatcherPatternHead::Except
                | MatcherPatternHead::HoldPattern
                | MatcherPatternHead::Optional
                | MatcherPatternHead::OrderlessPatternSequence
                | MatcherPatternHead::PatternSequence
                | MatcherPatternHead::Repeated
                | MatcherPatternHead::RepeatedNull => 0,
            };
            let expr = Expr::call(
                system_expr(pattern_head.name()),
                vec![Expr::symbol("arg"); arg_count],
            );

            assert!(definition_lhs_contains_pattern_syntax(&expr));
            let Expr::Call { head, args } = expr else {
                unreachable!();
            };
            assert!(
                !matches!(
                    definition_lhs_syntax_call_plan(&head, args.len(), DefinitionLhsRegion::Root),
                    DefinitionLhsSyntaxCallPlan::Ordinary
                ),
                "{} lacks an lhs-preparation policy",
                pattern_head.name()
            );
        }
    }
}
