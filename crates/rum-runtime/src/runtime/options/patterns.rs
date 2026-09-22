use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Debug, PartialEq, Eq)]
enum OptionsPatternTarget {
    Automatic,
    Explicit(Symbol),
}

impl Session {
    pub(in crate::runtime) fn evaluate_with_option_value_scope(
        &mut self,
        scope: Option<OptionValueScope>,
        expr: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(scope) = scope else {
            return self.evaluate(expr);
        };

        self.option_value_scopes.push(scope);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::OptionValueScopes,
        );

        match self.evaluate(expr) {
            Ok(value) => {
                let _ = self.option_value_scopes.pop();
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::OptionValueScopes,
                );
                Ok(value)
            }
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, EvaluateWithOptionValueScopeSuspensionFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => {
                let _ = self.option_value_scopes.pop();
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::OptionValueScopes,
                );
                Err(error)
            }
        }
    }

    pub(in crate::runtime) fn option_value_scope_for_definition(
        &self,
        lhs: &Expr,
        candidate: &Expr,
    ) -> Option<OptionValueScope> {
        let Expr::Call { args: lhs_args, .. } = lhs else {
            return None;
        };
        let Expr::Call {
            args: candidate_args,
            ..
        } = candidate
        else {
            return None;
        };

        let automatic_target = automatic_options_pattern_target(candidate)?;
        let mut matched = None;

        for (index, arg) in lhs_args.iter().enumerate() {
            let Some(target) = self.options_pattern_target_for_arg(arg) else {
                continue;
            };
            if matched.is_some() {
                return None;
            }
            matched = Some((index, target));
        }

        let (index, target) = matched?;
        if index + 1 != lhs_args.len() || index > candidate_args.len() {
            return None;
        }

        let target = match target {
            OptionsPatternTarget::Automatic => automatic_target,
            OptionsPatternTarget::Explicit(symbol) => symbol,
        };
        let explicit_expr = Expr::call(system_expr("List"), candidate_args[index..].to_vec());
        let explicit = self
            .compile_assigned_options_from_resolved(&explicit_expr)
            .ok()?;

        Some(OptionValueScope { target, explicit })
    }

    fn options_pattern_target_for_arg(&self, expr: &Expr) -> Option<OptionsPatternTarget> {
        let expr = strip_hold_pattern(expr);

        match expr {
            Expr::Call { head, args }
                if MatcherPatternHead::Pattern.matches(head) && args.len() == 2 =>
            {
                self.options_pattern_target_for_arg(&args[1])
            }
            Expr::Call { head, args }
                if MatcherPatternHead::Optional.matches(head) && !args.is_empty() =>
            {
                self.options_pattern_target_for_arg(&args[0])
            }
            Expr::Call { head, args } if MatcherPatternHead::OptionsPattern.matches(head) => {
                match args.as_slice() {
                    [] => Some(OptionsPatternTarget::Automatic),
                    [target] => self
                        .normalize_symbol_target_expr(target, true)
                        .map(OptionsPatternTarget::Explicit),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

pub(in crate::runtime) fn options_pattern_sequence_matcher() -> CompiledSequencePattern {
    let option_rule_matcher = |head_name: &str| MatcherNode::Call {
        head: Box::new(MatcherNode::Exact(system_expr(head_name))),
        args: vec![
            MatcherArg::Single(MatcherNode::Blank {
                head_constraint: None,
            }),
            MatcherArg::Single(MatcherNode::Blank {
                head_constraint: None,
            }),
        ],
    };

    CompiledSequencePattern {
        min_len: 0,
        max_len: None,
        matcher: MatcherNode::Alternatives(vec![
            option_rule_matcher("Rule"),
            option_rule_matcher("RuleDelayed"),
        ]),
        name: None,
        checks: Vec::new(),
        greediness: SequenceGreediness::Shortest,
    }
}

pub(in crate::runtime) fn is_options_pattern_expr(expr: &Expr) -> bool {
    let expr = strip_hold_pattern(expr);

    match expr {
        Expr::Call { head, args }
            if MatcherPatternHead::Pattern.matches(head) && args.len() == 2 =>
        {
            is_options_pattern_expr(&args[1])
        }
        Expr::Call { head, args }
            if MatcherPatternHead::Optional.matches(head) && !args.is_empty() =>
        {
            is_options_pattern_expr(&args[0])
        }
        Expr::Call { head, args } if MatcherPatternHead::OptionsPattern.matches(head) => {
            args.len() <= 1
        }
        _ => false,
    }
}

fn automatic_options_pattern_target(candidate: &Expr) -> Option<Symbol> {
    let Expr::Call { head, .. } = candidate else {
        return None;
    };

    base_head_symbol_of_nested_call(head)
        .cloned()
        .or_else(|| expr_as_symbol(head).cloned())
}

#[derive(rum_derive::FrameRootTrace)]
struct EvaluateWithOptionValueScopeSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for EvaluateWithOptionValueScopeSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = (|| {
            let _ = session.option_value_scopes.pop();
            session.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::OptionValueScopes,
            );
            outcome
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvaluateWithOptionValueScopeSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.pop_option_value_scope();
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}
