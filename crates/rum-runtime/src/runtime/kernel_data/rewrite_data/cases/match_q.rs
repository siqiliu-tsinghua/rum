use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_match_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() == 1 {
            return Ok(self.invalid_call("MatchQ", args));
        }
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("MatchQ", args, 2));
        }
        if let Some(error) = first_invalid_repeated_bounds(&args[1]) {
            self.push_message(
                error.symbol,
                "range",
                format!(
                    "{} is not a valid repetition bound specification; expected n, {{n}}, or {{min, max}} with 0 <= min <= max, where max may be Infinity.",
                    error.spec
                ),
            );
            return Ok(Expr::call(system_expr("MatchQ"), args.to_vec()));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = crate::resume_or_suspend!(
            self,
            value_result,
            ok(value) => Ok(value),
            suspend => {
        let pattern_expr = args[1].clone();
        EvalMatchQSuspensionFrame { pattern_expr }
        },
        )?;

        self.resume_match_q_after_value(args[1].clone(), value)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn normalize_pattern_expr(
        &mut self,
        expr: &Expr,
    ) -> Result<Expr, EvalError> {
        self.resolve_matcher_pattern_expr_owned(expr.clone())
    }

    pub(in crate::runtime) fn resolve_matcher_pattern_expr_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Expr, EvalError> {
        let resolved = self.resolve_match_pattern_subexprs(strip_hold_pattern(&expr))?;
        Ok(self.normalize_match_expr(&resolved))
    }

    /// Evaluate the ordinary subexpressions of a held matcher pattern while keeping
    /// genuine pattern constructs held.
    fn resolve_match_pattern_subexprs(&mut self, expr: &Expr) -> Result<Expr, EvalError> {
        if !contains_pattern_syntax(expr) {
            return self.evaluate(expr.clone());
        }

        match expr {
            Expr::Atom(_) => Ok(expr.clone()),
            Expr::Call { head, args } => {
                if matches_pattern_head(head) {
                    return Ok(expr.clone());
                }

                let resolved_head = self.resolve_match_pattern_subexprs(head)?;
                let mut resolved_args = Vec::with_capacity(args.len());
                for arg in args.iter() {
                    resolved_args.push(self.resolve_match_pattern_subexprs(arg)?);
                }
                self.resume_call_from_prepared_args(resolved_head, resolved_args)
            }
        }
    }

    #[allow(dead_code)]
    pub(in crate::runtime) fn normalize_cases_spec(
        &mut self,
        expr: &Expr,
    ) -> Result<CasesSpec, EvalError> {
        let resolved = self.resolve_cases_spec_expr_for_matching_owned(expr.clone())?;
        Ok(self.compile_cases_spec_from_resolved_expr(resolved))
    }

    fn resume_match_q_after_value(
        &mut self,
        pattern_expr: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        let pattern = crate::resume_or_suspend!(
            self,
            pattern_result,
            ok(value) => Ok(value),
            suspend => {
        let value = value.clone();
        ResumeMatchQAfterValueSuspensionFrame { value }
        },
        )?;

        self.finish_match_q_with_pattern(pattern, value)
    }

    fn finish_match_q_with_pattern(
        &mut self,
        pattern: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        if let Some(matched) = self.fast_match_q_resolved_pattern(&pattern, &value) {
            return Ok(system_expr(if matched { "True" } else { "False" }));
        }

        let match_result = self.match_expr_owned(pattern, value, SymbolExprMap::default());
        crate::resume_or_suspend!(
            self,
            match_result,
            ok(outcome) => { (|| {
                    Ok(system_expr(if outcome.matched { "True" } else { "False" }))
                })() },
            suspend => FinishMatchQWithPatternSuspensionFrame,
        )
    }
    fn fast_match_q_resolved_pattern(&self, pattern: &Expr, value: &Expr) -> Option<bool> {
        if let Some(head_constraint) = pattern_blank_head(pattern) {
            return Some(match_blank(head_constraint, value));
        }

        self.fast_fixed_head_blank_args_match_q(pattern, value)
    }

    fn fast_fixed_head_blank_args_match_q(&self, pattern: &Expr, value: &Expr) -> Option<bool> {
        let Expr::Call {
            head: pattern_head,
            args: pattern_args,
        } = pattern
        else {
            return None;
        };

        if pattern_args.is_empty() || !pattern_args.iter().all(is_plain_blank_pattern) {
            return None;
        }

        if matches_pattern_head(pattern_head) {
            return None;
        }

        let pattern_symbol = expr_as_symbol(pattern_head)?;
        let attributes = self.attribute_set_for_symbol(pattern_symbol);
        if fixed_head_blank_args_fast_path_needs_general_matcher(attributes) {
            return None;
        }

        let Expr::Call { head, args } = value else {
            return Some(false);
        };

        Some(expr_as_symbol(head) == Some(pattern_symbol) && args.len() == pattern_args.len())
    }
}

fn is_plain_blank_pattern(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args } if MatcherPatternHead::Blank.matches(head) && args.is_empty()
    )
}

fn fixed_head_blank_args_fast_path_needs_general_matcher(attributes: AttributeSet) -> bool {
    attributes.contains(Attribute::Flat)
        || attributes.contains(Attribute::Orderless)
        || attributes.contains(Attribute::OneIdentity)
}

fn contains_pattern_syntax(expr: &Expr) -> bool {
    let expr = strip_hold_pattern(expr);

    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            if matches_pattern_head(head) {
                true
            } else {
                contains_pattern_syntax(head) || args.iter().any(contains_pattern_syntax)
            }
        }
    }
}

fn matches_pattern_head(head: &Expr) -> bool {
    // OptionsPattern is recognized by the sequence matcher, but the existing
    // MatchQ resolution policy does not treat it as an opaque whole-pattern
    // wrapper. Keep that matcher behavior unchanged while sharing the registry.
    matcher_pattern_head(head)
        .is_some_and(|pattern_head| pattern_head != MatcherPatternHead::OptionsPattern)
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalMatchQSuspensionFrame {
    pattern_expr: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalMatchQSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut pattern_expr = self.pattern_expr.clone();
        let resumed = match outcome {
            Ok(value) => (|| session.resume_match_q_after_value(pattern_expr, value))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeMatchQAfterValueSuspensionFrame {
    value: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeMatchQAfterValueSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        let resumed = match outcome {
            Ok(resolved) => (|| session.finish_match_q_with_pattern(resolved, value))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishMatchQWithPatternSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for FinishMatchQWithPatternSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            Ok(system_expr(if outcome.matched { "True" } else { "False" }))
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
