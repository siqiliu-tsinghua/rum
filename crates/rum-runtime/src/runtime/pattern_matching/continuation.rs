use super::*;

fn tagged(name: &'static str, args: Vec<Expr>) -> Expr {
    Expr::call(system_expr(name), args)
}

fn tagged_args(expr: Expr, name: &'static str) -> Vec<Expr> {
    let Expr::Call { head, args } = expr else {
        panic!("unexpected {name} continuation payload")
    };
    assert!(
        is_system_symbol_name(&head, name),
        "unexpected {name} continuation tag: {head:?}"
    );
    args.into_vec()
}

fn usize_expr(value: usize) -> Expr {
    Expr::string(value.to_string())
}

fn usize_from_expr(expr: Expr) -> usize {
    let Expr::Atom(Atom::String(value)) = expr else {
        panic!("matcher continuation index should be a string: {expr:?}")
    };
    value
        .parse()
        .expect("matcher continuation index should be a usize")
}

fn symbol_expr(symbol: Symbol) -> Expr {
    Expr::Atom(Atom::Symbol(symbol))
}

fn symbol_from_expr(expr: Expr) -> Symbol {
    let Expr::Atom(Atom::Symbol(symbol)) = expr else {
        panic!("matcher continuation symbol should be a symbol: {expr:?}")
    };
    symbol
}

fn option_symbol_expr(symbol: Option<Symbol>) -> Expr {
    symbol.map(symbol_expr).into_continuation_expr()
}

fn option_symbol_from_expr(expr: Expr) -> Option<Symbol> {
    Option::<Expr>::from_continuation_expr(expr).map(symbol_from_expr)
}

pub(in crate::runtime) fn matcher_node_into_continuation_expr(matcher: MatcherNode) -> Expr {
    match matcher {
        MatcherNode::Exact(value) => tagged("InternalContinuationMatcherExact", vec![value]),
        MatcherNode::Blank { head_constraint } => tagged(
            "InternalContinuationMatcherBlank",
            vec![head_constraint.into_continuation_expr()],
        ),
        MatcherNode::KeyValuePattern(entries) => tagged(
            "InternalContinuationMatcherKeyValuePattern",
            entries
                .into_iter()
                .map(|(key, value, optional)| {
                    tagged(
                        "InternalContinuationMatcherKeyValueEntry",
                        vec![
                            matcher_node_into_continuation_expr(key),
                            matcher_node_into_continuation_expr(value),
                            optional.into_continuation_expr(),
                        ],
                    )
                })
                .collect(),
        ),
        MatcherNode::Named { name, inner } => tagged(
            "InternalContinuationMatcherNamed",
            vec![
                symbol_expr(name),
                matcher_node_into_continuation_expr(*inner),
            ],
        ),
        MatcherNode::Condition { inner, condition } => tagged(
            "InternalContinuationMatcherCondition",
            vec![matcher_node_into_continuation_expr(*inner), condition],
        ),
        MatcherNode::DefinitionCondition(condition) => tagged(
            "InternalContinuationMatcherDefinitionCondition",
            vec![
                matcher_node_into_continuation_expr(condition.inner),
                condition.lhs,
                tagged(
                    "InternalContinuationMatcherGuards",
                    condition.guards.into_vec(),
                ),
            ],
        ),
        MatcherNode::PatternTest { inner, test } => tagged(
            "InternalContinuationMatcherPatternTest",
            vec![matcher_node_into_continuation_expr(*inner), test],
        ),
        MatcherNode::Except { excluded, inner } => tagged(
            "InternalContinuationMatcherExcept",
            vec![
                matcher_node_into_continuation_expr(*excluded),
                matcher_node_into_continuation_expr(*inner),
            ],
        ),
        MatcherNode::Alternatives(alternatives) => tagged(
            "InternalContinuationMatcherAlternatives",
            alternatives
                .into_iter()
                .map(matcher_node_into_continuation_expr)
                .collect(),
        ),
        MatcherNode::Call { head, args } => tagged(
            "InternalContinuationMatcherCall",
            vec![
                matcher_node_into_continuation_expr(*head),
                matcher_args_into_continuation_expr(args),
            ],
        ),
    }
}

pub(in crate::runtime) fn matcher_node_from_continuation_expr(expr: Expr) -> MatcherNode {
    let Expr::Call { head, args } = expr else {
        panic!("unexpected matcher continuation payload")
    };
    let mut args = args.into_iter();
    let matcher = if is_system_symbol_name(&head, "InternalContinuationMatcherExact") {
        MatcherNode::Exact(args.next().expect("exact matcher stores its value"))
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherBlank") {
        MatcherNode::Blank {
            head_constraint: Option::<Expr>::from_continuation_expr(
                args.next().expect("blank matcher stores its constraint"),
            ),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherKeyValuePattern") {
        return MatcherNode::KeyValuePattern(
            args.map(|entry| {
                let mut fields =
                    tagged_args(entry, "InternalContinuationMatcherKeyValueEntry").into_iter();
                let key = matcher_node_from_continuation_expr(
                    fields.next().expect("key-value matcher stores its key"),
                );
                let value = matcher_node_from_continuation_expr(
                    fields.next().expect("key-value matcher stores its value"),
                );
                let optional = bool::from_continuation_expr(
                    fields
                        .next()
                        .expect("key-value matcher stores its optional flag"),
                );
                debug_assert!(fields.next().is_none());
                (key, value, optional)
            })
            .collect(),
        );
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherNamed") {
        MatcherNode::Named {
            name: symbol_from_expr(args.next().expect("named matcher stores its name")),
            inner: Box::new(matcher_node_from_continuation_expr(
                args.next().expect("named matcher stores its inner matcher"),
            )),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherCondition") {
        MatcherNode::Condition {
            inner: Box::new(matcher_node_from_continuation_expr(
                args.next()
                    .expect("condition matcher stores its inner matcher"),
            )),
            condition: args.next().expect("condition matcher stores its condition"),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherDefinitionCondition") {
        MatcherNode::DefinitionCondition(Box::new(DefinitionConditionMatcher {
            inner: matcher_node_from_continuation_expr(
                args.next()
                    .expect("definition condition stores its inner matcher"),
            ),
            lhs: args.next().expect("definition condition stores its lhs"),
            guards: tagged_args(
                args.next()
                    .expect("definition condition stores its guard sequence"),
                "InternalContinuationMatcherGuards",
            )
            .into_boxed_slice(),
        }))
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherPatternTest") {
        MatcherNode::PatternTest {
            inner: Box::new(matcher_node_from_continuation_expr(
                args.next().expect("pattern test stores its inner matcher"),
            )),
            test: args.next().expect("pattern test stores its test"),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherExcept") {
        MatcherNode::Except {
            excluded: Box::new(matcher_node_from_continuation_expr(
                args.next()
                    .expect("except matcher stores its excluded matcher"),
            )),
            inner: Box::new(matcher_node_from_continuation_expr(
                args.next()
                    .expect("except matcher stores its inner matcher"),
            )),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherAlternatives") {
        return MatcherNode::Alternatives(args.map(matcher_node_from_continuation_expr).collect());
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherCall") {
        MatcherNode::Call {
            head: Box::new(matcher_node_from_continuation_expr(
                args.next().expect("call matcher stores its head"),
            )),
            args: matcher_args_from_continuation_expr(
                args.next().expect("call matcher stores its arguments"),
            ),
        }
    } else {
        panic!("unexpected matcher continuation tag: {head:?}")
    };
    debug_assert!(args.next().is_none());
    matcher
}

pub(in crate::runtime) fn matcher_args_into_continuation_expr(args: Vec<MatcherArg>) -> Expr {
    tagged(
        "InternalContinuationMatcherArgs",
        args.into_iter()
            .map(matcher_arg_into_continuation_expr)
            .collect(),
    )
}

pub(in crate::runtime) fn matcher_args_from_continuation_expr(expr: Expr) -> Vec<MatcherArg> {
    tagged_args(expr, "InternalContinuationMatcherArgs")
        .into_iter()
        .map(matcher_arg_from_continuation_expr)
        .collect()
}

pub(in crate::runtime) fn matcher_arg_into_continuation_expr(arg: MatcherArg) -> Expr {
    match arg {
        MatcherArg::Single(matcher) => tagged(
            "InternalContinuationMatcherArgSingle",
            vec![matcher_node_into_continuation_expr(matcher)],
        ),
        MatcherArg::Sequence(spec) => tagged(
            "InternalContinuationMatcherArgSequence",
            vec![sequence_pattern_into_continuation_expr(spec)],
        ),
        MatcherArg::Optional(spec) => tagged(
            "InternalContinuationMatcherArgOptional",
            vec![optional_pattern_into_continuation_expr(spec)],
        ),
        MatcherArg::PatternSequenceGroup(args) => tagged(
            "InternalContinuationMatcherArgPatternSequenceGroup",
            vec![matcher_args_into_continuation_expr(args)],
        ),
        MatcherArg::OrderlessPatternSequenceGroup(args) => tagged(
            "InternalContinuationMatcherArgOrderlessPatternSequenceGroup",
            vec![matcher_args_into_continuation_expr(args)],
        ),
    }
}

pub(in crate::runtime) fn matcher_arg_from_continuation_expr(expr: Expr) -> MatcherArg {
    let Expr::Call { head, args } = expr else {
        panic!("unexpected matcher-argument continuation payload")
    };
    let mut args = args.into_iter();
    let arg = if is_system_symbol_name(&head, "InternalContinuationMatcherArgSingle") {
        MatcherArg::Single(matcher_node_from_continuation_expr(
            args.next()
                .expect("single matcher argument stores its matcher"),
        ))
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherArgSequence") {
        MatcherArg::Sequence(sequence_pattern_from_continuation_expr(
            args.next()
                .expect("sequence matcher argument stores its spec"),
        ))
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherArgOptional") {
        MatcherArg::Optional(optional_pattern_from_continuation_expr(
            args.next()
                .expect("optional matcher argument stores its spec"),
        ))
    } else if is_system_symbol_name(&head, "InternalContinuationMatcherArgPatternSequenceGroup") {
        MatcherArg::PatternSequenceGroup(matcher_args_from_continuation_expr(
            args.next()
                .expect("pattern sequence group stores its arguments"),
        ))
    } else if is_system_symbol_name(
        &head,
        "InternalContinuationMatcherArgOrderlessPatternSequenceGroup",
    ) {
        MatcherArg::OrderlessPatternSequenceGroup(matcher_args_from_continuation_expr(
            args.next()
                .expect("orderless pattern sequence group stores its arguments"),
        ))
    } else {
        panic!("unexpected matcher-argument continuation tag: {head:?}")
    };
    debug_assert!(args.next().is_none());
    arg
}

pub(in crate::runtime) fn sequence_pattern_into_continuation_expr(
    spec: CompiledSequencePattern,
) -> Expr {
    let max_len = spec.max_len.map(usize_expr).into_continuation_expr();
    let checks = spec
        .checks
        .into_iter()
        .map(|check| match check {
            SequencePatternCheck::Condition(condition) => {
                tagged("InternalContinuationSequenceCondition", vec![condition])
            }
            SequencePatternCheck::PatternTest(test) => {
                tagged("InternalContinuationSequencePatternTest", vec![test])
            }
        })
        .collect();
    tagged(
        "InternalContinuationSequencePattern",
        vec![
            usize_expr(spec.min_len),
            max_len,
            matcher_node_into_continuation_expr(spec.matcher),
            option_symbol_expr(spec.name),
            tagged("InternalContinuationSequenceChecks", checks),
            bool::from(matches!(spec.greediness, SequenceGreediness::Longest))
                .into_continuation_expr(),
        ],
    )
}

pub(in crate::runtime) fn sequence_pattern_from_continuation_expr(
    expr: Expr,
) -> CompiledSequencePattern {
    let mut args = tagged_args(expr, "InternalContinuationSequencePattern").into_iter();
    let min_len = usize_from_expr(args.next().expect("sequence pattern stores min length"));
    let max_len = Option::<Expr>::from_continuation_expr(
        args.next().expect("sequence pattern stores max length"),
    )
    .map(usize_from_expr);
    let matcher =
        matcher_node_from_continuation_expr(args.next().expect("sequence pattern stores matcher"));
    let name = option_symbol_from_expr(args.next().expect("sequence pattern stores name"));
    let checks = tagged_args(
        args.next().expect("sequence pattern stores checks"),
        "InternalContinuationSequenceChecks",
    )
    .into_iter()
    .map(|check| {
        let Expr::Call { head, args } = check else {
            panic!("unexpected sequence-check continuation payload")
        };
        let mut args = args.into_iter();
        let value = args.next().expect("sequence check stores its expression");
        debug_assert!(args.next().is_none());
        if is_system_symbol_name(&head, "InternalContinuationSequenceCondition") {
            SequencePatternCheck::Condition(value)
        } else if is_system_symbol_name(&head, "InternalContinuationSequencePatternTest") {
            SequencePatternCheck::PatternTest(value)
        } else {
            panic!("unexpected sequence-check continuation tag: {head:?}")
        }
    })
    .collect();
    let greediness = if bool::from_continuation_expr(
        args.next().expect("sequence pattern stores its greediness"),
    ) {
        SequenceGreediness::Longest
    } else {
        SequenceGreediness::Shortest
    };
    debug_assert!(args.next().is_none());
    CompiledSequencePattern {
        min_len,
        max_len,
        matcher,
        name,
        checks,
        greediness,
    }
}

pub(in crate::runtime) fn optional_pattern_into_continuation_expr(
    spec: CompiledOptionalPattern,
) -> Expr {
    let default = match spec.default {
        CompiledOptionalDefault::Explicit(value) => {
            tagged("InternalContinuationOptionalDefaultExplicit", vec![value])
        }
        CompiledOptionalDefault::Implicit { head, position } => tagged(
            "InternalContinuationOptionalDefaultImplicit",
            vec![head, usize_expr(position)],
        ),
        CompiledOptionalDefault::UnavailableImplicit => {
            tagged("InternalContinuationOptionalDefaultUnavailable", Vec::new())
        }
    };
    tagged(
        "InternalContinuationOptionalPattern",
        vec![matcher_arg_into_continuation_expr(*spec.inner), default],
    )
}

pub(in crate::runtime) fn optional_pattern_from_continuation_expr(
    expr: Expr,
) -> CompiledOptionalPattern {
    let mut args = tagged_args(expr, "InternalContinuationOptionalPattern").into_iter();
    let inner = Box::new(matcher_arg_from_continuation_expr(
        args.next()
            .expect("optional pattern stores its inner matcher"),
    ));
    let default = args.next().expect("optional pattern stores its default");
    debug_assert!(args.next().is_none());
    let Expr::Call { head, args } = default else {
        panic!("unexpected optional-default continuation payload")
    };
    let mut args = args.into_iter();
    let default = if is_system_symbol_name(&head, "InternalContinuationOptionalDefaultExplicit") {
        CompiledOptionalDefault::Explicit(
            args.next()
                .expect("explicit optional default stores its value"),
        )
    } else if is_system_symbol_name(&head, "InternalContinuationOptionalDefaultImplicit") {
        CompiledOptionalDefault::Implicit {
            head: args
                .next()
                .expect("implicit optional default stores its head"),
            position: usize_from_expr(
                args.next()
                    .expect("implicit optional default stores its position"),
            ),
        }
    } else if is_system_symbol_name(&head, "InternalContinuationOptionalDefaultUnavailable") {
        CompiledOptionalDefault::UnavailableImplicit
    } else {
        panic!("unexpected optional-default continuation tag: {head:?}")
    };
    debug_assert!(args.next().is_none());
    CompiledOptionalPattern { inner, default }
}

pub(in crate::runtime) fn binding_map_into_continuation_expr(bindings: SymbolExprMap) -> Expr {
    vec![bindings].into_continuation_expr()
}

pub(in crate::runtime) fn binding_map_from_continuation_expr(expr: Expr) -> SymbolExprMap {
    let mut bindings = Vec::<SymbolExprMap>::from_continuation_expr(expr).into_iter();
    let first = bindings
        .next()
        .expect("binding-map continuation stores one map");
    debug_assert!(bindings.next().is_none());
    first
}

pub(in crate::runtime) fn usize_into_continuation_expr(value: usize) -> Expr {
    usize_expr(value)
}

pub(in crate::runtime) fn usize_from_continuation_expr(expr: Expr) -> usize {
    usize_from_expr(expr)
}
