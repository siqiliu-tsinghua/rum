use super::*;

impl Session {
    pub(super) fn normalize_definition_candidate_for_matching(
        &mut self,
        candidate: &Expr,
        attributes: AttributeSet,
    ) -> Expr {
        if matches!(attributes.hold_attribute(), HoldAttribute::None) {
            candidate.clone()
        } else {
            self.normalize_match_expr(candidate)
        }
    }

    pub(super) fn definition_lhs_could_match_candidate(
        &self,
        _lhs: &Expr,
        matcher: &MatcherNode,
        candidate: &Expr,
    ) -> bool {
        self.matcher_could_match_candidate_shape(matcher, candidate)
    }

    fn matcher_could_match_candidate_shape(&self, matcher: &MatcherNode, candidate: &Expr) -> bool {
        match matcher {
            MatcherNode::Exact(pattern) => pattern == candidate,
            MatcherNode::Blank { head_constraint } => {
                match_blank(head_constraint.as_ref(), candidate)
            }
            MatcherNode::KeyValuePattern(_) | MatcherNode::Except { .. } => true,
            MatcherNode::Named { inner, .. }
            | MatcherNode::Condition { inner, .. }
            | MatcherNode::PatternTest { inner, .. } => {
                self.matcher_could_match_candidate_shape(inner, candidate)
            }
            MatcherNode::DefinitionCondition(matcher) => {
                self.matcher_could_match_candidate_shape(&matcher.inner, candidate)
            }
            MatcherNode::Alternatives(alternatives) => alternatives.iter().any(|alternative| {
                self.matcher_could_match_candidate_shape(alternative, candidate)
            }),
            MatcherNode::Call { head, args } => {
                // Attribute lookup is not free on definition-heavy paths.  Keep
                // the conservative shape gate sound for attributed heads, but
                // fetch their attributes only once per call node.
                let head_attributes = match head.as_ref() {
                    MatcherNode::Exact(Expr::Atom(Atom::Symbol(symbol))) => {
                        Some(self.attribute_set_for_symbol(symbol))
                    }
                    _ => None,
                };

                if head_attributes
                    .is_some_and(|attributes| attributes.contains(Attribute::OneIdentity))
                {
                    return true;
                }

                let Expr::Call {
                    head: value_head,
                    args: value_args,
                    ..
                } = candidate
                else {
                    return true;
                };

                if !self.matcher_could_match_candidate_shape(head, value_head) {
                    return false;
                }

                if head_attributes.is_some_and(|attributes| {
                    attributes.contains(Attribute::Flat)
                        || attributes.contains(Attribute::Orderless)
                }) {
                    return true;
                }

                if args.iter().any(|arg| !matches!(arg, MatcherArg::Single(_))) {
                    return true;
                }

                args.len() == value_args.len()
                    && args.iter().zip(value_args).all(|(pattern_arg, value_arg)| {
                        let MatcherArg::Single(matcher) = pattern_arg else {
                            return true;
                        };
                        self.matcher_could_match_candidate_shape(matcher, value_arg)
                    })
            }
        }
    }

    fn pattern_args_may_have_variable_arity(args: &[Expr]) -> bool {
        args.iter().any(|arg| {
            let arg = strip_hold_pattern(arg);
            decompose_sequence_pattern(arg).is_some()
                || pattern_sequence_items(arg).is_some()
                || pattern_orderless_sequence_items(arg).is_some()
                || pattern_optional(arg).is_some()
        })
    }

    pub(super) fn try_fast_definition_match(
        &self,
        lhs: &Expr,
        guards: &[Expr],
        candidate: &Expr,
    ) -> Option<CompiledMatchOutcome> {
        // A stale guarded matcher must be rebuilt from both stored halves of
        // the definition.  The raw-lhs fast path cannot evaluate rhs guards.
        if !guards.is_empty() {
            return None;
        }

        let mut bindings = SymbolExprMap::default();
        let matched =
            self.fast_match_definition_pattern(strip_hold_pattern(lhs), candidate, &mut bindings)?;

        Some(CompiledMatchOutcome {
            matched,
            bindings: if matched {
                bindings
            } else {
                SymbolExprMap::default()
            },
        })
    }

    pub(super) fn try_fast_definition_match_compiled(
        &self,
        matcher: &MatcherNode,
        candidate: &Expr,
    ) -> Option<CompiledMatchOutcome> {
        let mut bindings = SymbolExprMap::default();
        let matched = self.fast_match_compiled(matcher, candidate, &mut bindings)?;

        Some(CompiledMatchOutcome {
            matched,
            bindings: if matched {
                bindings
            } else {
                SymbolExprMap::default()
            },
        })
    }

    fn fast_match_compiled(
        &self,
        matcher: &MatcherNode,
        value: &Expr,
        bindings: &mut SymbolExprMap,
    ) -> Option<bool> {
        match matcher {
            MatcherNode::Exact(pattern) => Some(pattern == value),
            MatcherNode::Blank { head_constraint } => {
                Some(match_blank(head_constraint.as_ref(), value))
            }
            MatcherNode::Named { name, inner } => {
                if !self.fast_match_compiled(inner, value, bindings)? {
                    return Some(false);
                }
                Some(bind_pattern_name(name, value.clone(), bindings))
            }
            MatcherNode::Call { head, args } => {
                let Expr::Call {
                    head: value_head,
                    args: value_args,
                    ..
                } = value
                else {
                    return Some(false);
                };

                if args.iter().any(|arg| !matches!(arg, MatcherArg::Single(_))) {
                    return None;
                }
                if args.len() != value_args.len() {
                    return Some(false);
                }

                let head_attributes = self.attribute_set_for_head(value_head);
                if head_attributes.contains(Attribute::Flat)
                    || head_attributes.contains(Attribute::Orderless)
                    || head_attributes.contains(Attribute::OneIdentity)
                {
                    return None;
                }

                if !self.fast_match_compiled(head, value_head, bindings)? {
                    return Some(false);
                }

                for (matcher_arg, value_arg) in args.iter().zip(value_args) {
                    let MatcherArg::Single(arg_matcher) = matcher_arg else {
                        return None;
                    };
                    if !self.fast_match_compiled(arg_matcher, value_arg, bindings)? {
                        return Some(false);
                    }
                }

                Some(true)
            }
            MatcherNode::Condition { .. }
            | MatcherNode::DefinitionCondition(_)
            | MatcherNode::PatternTest { .. }
            | MatcherNode::Except { .. }
            | MatcherNode::Alternatives(_)
            | MatcherNode::KeyValuePattern(_) => None,
        }
    }

    fn fast_match_definition_pattern(
        &self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut SymbolExprMap,
    ) -> Option<bool> {
        let pattern = strip_hold_pattern(pattern);

        if let Some(inner) = pattern_verbatim_inner(pattern) {
            return Some(inner == value);
        }

        if let Some((name, inner)) = pattern_named_pattern(pattern) {
            if !self.fast_match_definition_pattern(inner, value, bindings)? {
                return Some(false);
            }
            return Some(bind_pattern_name(name, value.clone(), bindings));
        }

        if pattern_condition(pattern).is_some()
            || pattern_test(pattern).is_some()
            || pattern_except(pattern).is_some()
            || pattern_optional(pattern).is_some()
            || pattern_alternatives(pattern).is_some()
            || pattern_key_value_pattern_entries(pattern).is_some()
            || decompose_sequence_pattern(pattern).is_some()
            || pattern_sequence_items(pattern).is_some()
            || pattern_orderless_sequence_items(pattern).is_some()
        {
            return None;
        }

        if let Some(head_constraint) = pattern_blank_head(pattern) {
            return Some(match_blank(head_constraint, value));
        }

        match (pattern, value) {
            (Expr::Atom(_), _) => Some(pattern == value),
            (
                Expr::Call {
                    head: pattern_head,
                    args: pattern_args,
                    ..
                },
                Expr::Call {
                    head: value_head,
                    args: value_args,
                    ..
                },
            ) => {
                if Self::pattern_args_may_have_variable_arity(pattern_args) {
                    return None;
                }
                if pattern_args.len() != value_args.len() {
                    return Some(false);
                }

                let head_attributes = self.attribute_set_for_head(pattern_head);
                if head_attributes.contains(Attribute::Flat)
                    || head_attributes.contains(Attribute::Orderless)
                    || head_attributes.contains(Attribute::OneIdentity)
                {
                    return None;
                }

                if !self.fast_match_definition_pattern(pattern_head, value_head, bindings)? {
                    return Some(false);
                }

                for (pattern_arg, value_arg) in pattern_args.iter().zip(value_args) {
                    if !self.fast_match_definition_pattern(pattern_arg, value_arg, bindings)? {
                        return Some(false);
                    }
                }

                Some(true)
            }
            (Expr::Call { .. }, Expr::Atom(_)) => None,
        }
    }
}
