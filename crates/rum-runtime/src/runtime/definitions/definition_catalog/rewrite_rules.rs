use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) enum RewriteRuleExprContinuation {
    NormalizeRules {
        original: Expr,
    },
    Replace {
        target: Expr,
        levels_expr: Option<Expr>,
        heads_option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    },
    ReplaceAt {
        target: Expr,
        paths: Vec<Vec<isize>>,
        call_args: Vec<Expr>,
    },
    ReplaceRepeated {
        current: Expr,
        rules_input: Expr,
        max_iterations_expr: Option<Expr>,
        call_args: Vec<Expr>,
    },
    ReplaceList {
        target: Expr,
        max_results: Option<usize>,
        call_args: Vec<Expr>,
    },
}

impl RewriteRuleExprContinuation {
    fn resume(
        self,
        session: &mut Session,
        rule_exprs: Option<Vec<Expr>>,
    ) -> Result<Expr, EvalError> {
        match self {
            Self::NormalizeRules { original } => {
                session.resume_normalize_rules_after_rule_exprs(original, rule_exprs)
            }
            Self::Replace {
                target,
                levels_expr,
                heads_option_exprs,
                call_args,
            } => session.resume_replace_after_rule_exprs(
                target,
                levels_expr,
                heads_option_exprs,
                rule_exprs,
                call_args,
            ),
            Self::ReplaceAt {
                target,
                paths,
                call_args,
            } => session.resume_replace_at_after_rule_exprs(target, paths, rule_exprs, call_args),
            Self::ReplaceRepeated {
                current,
                rules_input,
                max_iterations_expr,
                call_args,
            } => session.resume_replace_repeated_after_rule_exprs(
                current,
                rules_input,
                max_iterations_expr,
                rule_exprs,
                call_args,
            ),
            Self::ReplaceList {
                target,
                max_results,
                call_args,
            } => session.resume_replace_list_after_rule_exprs(
                target,
                rule_exprs,
                max_results,
                call_args,
            ),
        }
    }
}

#[derive(FrameRootTrace)]
pub(in crate::runtime) enum RewriteRuleInfoContinuation {
    InspectRewriteRules,
}

impl RewriteRuleInfoContinuation {
    fn resume(
        self,
        session: &mut Session,
        rule_exprs: Option<Vec<Expr>>,
    ) -> Result<Option<Vec<RewriteRuleInfo>>, EvalError> {
        match self {
            Self::InspectRewriteRules => Ok(rule_exprs
                .and_then(|rule_exprs| {
                    session.compile_rewrite_rules_from_normalized_exprs(rule_exprs)
                })
                .map(|rules| {
                    rules
                        .into_iter()
                        .map(|rule| RewriteRuleInfo {
                            lhs: rule.lhs,
                            rhs: rebuild_definition_rhs_guards(rule.rhs, &rule.guards),
                            delayed: rule.delayed,
                        })
                        .collect()
                })),
        }
    }
}

#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) enum CompiledRewriteRuleContinuation {
    ReplaceAll { target: Expr, rules_input: Expr },
    ReplaceAllUnheld { target: Expr, rules_input: Expr },
}

impl CompiledRewriteRuleContinuation {
    fn resume(
        self,
        session: &mut Session,
        rules: Option<Vec<RewriteRule>>,
    ) -> Result<Expr, EvalError> {
        match self {
            Self::ReplaceAll {
                target,
                rules_input,
            } => session.resume_replace_all_after_rules(target, rules_input, rules),
            Self::ReplaceAllUnheld {
                target,
                rules_input,
            } => session.resume_replace_all_unheld_after_rules(target, rules_input, rules),
        }
    }
}

impl Session {
    pub(in crate::runtime) fn resolve_cases_spec_expr_for_matching_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Expr, EvalError> {
        let candidate = strip_hold_pattern(&expr).clone();

        if should_evaluate_single_rule_candidate(&candidate) {
            let resolved_result = self.evaluate(candidate);
            return crate::resume_or_suspend!(
                self,
                resolved_result,
                ok(resolved_expr) => { let session = self; let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resolved_expr); (|| {
                            if session
                                .compile_rewrite_rule_for_matching(&resolved)
                                .is_some()
                            {
                                Ok(resolved)
                            } else {
                                Ok(session.normalize_match_expr(&resolved))
                            }
                        })() },
                suspend => ResolveCasesSpecExprForMatchingOwnedSuspensionFrame,
            );
        }

        self.resolve_matcher_pattern_expr_owned(candidate)
    }

    pub(in crate::runtime) fn compile_cases_spec_from_resolved_expr(
        &self,
        resolved: Expr,
    ) -> CasesSpec {
        match self.compile_rewrite_rule_for_matching(&resolved) {
            Some(rule) => CasesSpec::Rule(rule),
            None => CasesSpec::Pattern(resolved),
        }
    }

    pub(in crate::runtime) fn resolve_rewrite_rule_exprs_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Option<Vec<Expr>>, EvalError> {
        let resolved_result = self.evaluate(expr);
        crate::resume_or_suspend!(
            self,
            resolved_result,
            ok(resolved_expr) => { let session = self; let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resolved_expr); (|| {
                    Ok(session.normalize_rewrite_rule_exprs_from_resolved(&resolved))
                })() },
            suspend => ResolveRewriteRuleExprsOwnedSuspensionFrame,
        )
    }

    pub(in crate::runtime) fn drive_rewrite_rule_exprs_owned(
        &mut self,
        expr: Expr,
        continuation: RewriteRuleExprContinuation,
    ) -> Result<Expr, EvalError> {
        let resolved_rule_exprs_result = self.resolve_rewrite_rule_exprs_owned(expr);
        crate::resume_or_suspend!(
            self,
            resolved_rule_exprs_result,
            ok(rule_exprs) => { let session = self; (|| {
                    continuation.resume(session, rule_exprs)
                })() },
            suspend => DriveRewriteRuleExprsOwnedSuspensionFrame { continuation },
        )
    }

    pub(in crate::runtime) fn drive_rewrite_rule_infos_owned(
        &mut self,
        expr: Expr,
        continuation: RewriteRuleInfoContinuation,
    ) -> Result<Option<Vec<RewriteRuleInfo>>, EvalError> {
        let resolved_rule_exprs_result = self.resolve_rewrite_rule_exprs_owned(expr);
        crate::resume_or_suspend!(
            self,
            resolved_rule_exprs_result,
            ok(rule_exprs) => { let session = self; (|| {
                    continuation.resume(session, rule_exprs)
                })() },
            suspend => DriveRewriteRuleInfosOwnedSuspensionFrame,
        )
    }

    pub(in crate::runtime) fn drive_rewrite_rules_owned(
        &mut self,
        expr: Expr,
        continuation: CompiledRewriteRuleContinuation,
    ) -> Result<Expr, EvalError> {
        match self.evaluate(expr) {
            Ok(resolved) => {
                continuation.resume(self, self.compile_rewrite_rules_for_matching(&resolved))
            }
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, DriveRewriteRulesOwnedSuspensionFrame { continuation });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime) fn normalize_rewrite_rule_exprs_from_resolved(
        &self,
        resolved: &Expr,
    ) -> Option<Vec<Expr>> {
        normalize_rewrite_rules(resolved).map(|rules| {
            rules
                .into_iter()
                .map(|rule| Self::rewrite_rule_expr(&rule))
                .collect()
        })
    }

    pub(in crate::runtime) fn compile_rewrite_rules_from_normalized_exprs(
        &self,
        rule_exprs: Vec<Expr>,
    ) -> Option<Vec<RewriteRule>> {
        self.compile_rewrite_rules_for_matching(&Expr::call(system_expr("List"), rule_exprs))
    }

    pub(in crate::runtime) fn rewrite_rule_expr(rule: &RewriteRule) -> Expr {
        Expr::call(
            system_expr(if rule.delayed { "RuleDelayed" } else { "Rule" }),
            vec![
                rule.lhs.clone(),
                rebuild_definition_rhs_guards(rule.rhs.clone(), &rule.guards),
            ],
        )
    }

    pub(in crate::runtime) fn normalized_rules_expr(rules: &[RewriteRule]) -> Expr {
        Expr::call(
            system_expr("List"),
            rules.iter().map(Self::rewrite_rule_expr).collect(),
        )
    }

    pub(in crate::runtime) fn compile_rewrite_rules_for_matching(
        &self,
        expr: &Expr,
    ) -> Option<Vec<RewriteRule>> {
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => args
                .iter()
                .map(|item| self.compile_rewrite_rule_for_matching(item))
                .collect(),
            _ => self
                .compile_rewrite_rule_for_matching(expr)
                .map(|rule| vec![rule]),
        }
    }

    pub(in crate::runtime) fn compile_rewrite_rule_for_matching(
        &self,
        expr: &Expr,
    ) -> Option<RewriteRule> {
        let mut rule = normalize_single_rewrite_rule(expr)?;
        rule.matcher = self.compile_down_value_matcher(&rule.lhs, &rule.guards);
        Some(rule)
    }

    pub(in crate::runtime) fn compile_matcher_for_expr(&self, expr: &Expr) -> MatcherNode {
        compile_matcher(&self.normalize_match_expr(expr))
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<CompiledRewriteRuleContinuation>();
    assert_traces::<RewriteRuleExprContinuation>();
    assert_traces::<RewriteRuleInfoContinuation>();
};

fn should_evaluate_single_rule_candidate(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(_)) => true,
        Expr::Call { head, .. } => {
            is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed")
        }
        _ => false,
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResolveCasesSpecExprForMatchingOwnedSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame
    for ResolveCasesSpecExprForMatchingOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                if session
                    .compile_rewrite_rule_for_matching(&resolved)
                    .is_some()
                {
                    Ok(resolved)
                } else {
                    Ok(session.normalize_match_expr(&resolved))
                }
            })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResolveRewriteRuleExprsOwnedSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for ResolveRewriteRuleExprsOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Option<Vec<rum_core::Expr>>, EvalError> {
            Ok(session.normalize_rewrite_rule_exprs_from_resolved(&resolved))
        })()
                    .map(|value| <Option<Vec<rum_core::Expr>> as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct DriveRewriteRuleExprsOwnedSuspensionFrame {
    continuation:
        crate::runtime::definitions::definition_catalog::rewrite_rules::RewriteRuleExprContinuation,
}

impl crate::runtime::suspension::SuspensionFrame for DriveRewriteRuleExprsOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut continuation = self.continuation.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let rule_exprs = <Option<Vec<rum_core::Expr>> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            continuation.resume(session, rule_exprs)
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct DriveRewriteRuleInfosOwnedSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for DriveRewriteRuleInfosOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let rule_exprs = <Option<Vec<rum_core::Expr>> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Option<Vec<crate::runtime::RewriteRuleInfo>>, EvalError> {
            RewriteRuleInfoContinuation::InspectRewriteRules.resume(session, rule_exprs)
        })()
                    .map(|value| <Option<Vec<crate::runtime::RewriteRuleInfo>> as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct DriveRewriteRulesOwnedSuspensionFrame {
    continuation: crate::runtime::definitions::definition_catalog::rewrite_rules::CompiledRewriteRuleContinuation,
}

impl crate::runtime::suspension::SuspensionFrame for DriveRewriteRulesOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut continuation = self.continuation.clone();
        let resumed = (|| match outcome {
            Ok(resolved) => {
                let rules = session.compile_rewrite_rules_for_matching(&resolved);
                let resumed = continuation.resume(session, rules)?;
                Ok(resumed.into_continuation_expr())
            }
            Err(error) => Err(error),
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
