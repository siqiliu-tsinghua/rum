//! Flat preparation of rule LHS syntax. Rebuilt calls are never applied here.
use super::super::*;
use super::definition_lhs_preparation::{
    definition_lhs_blank_head, definition_lhs_contains_pattern_syntax,
    definition_lhs_pattern_wrapper_head,
};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

pub(super) mod continuation;
mod state;
#[cfg(test)]
mod tests;

use continuation::RulePatternSuspensionFrame;
use state::*;

// These visits are local control, never retained across guest evaluation.
enum RuleLhsVisit {
    Classify,
    Structural,
    EvaluateOnly,
}

enum RuleLhsStep {
    Visit(Expr, RuleLhsVisit),
    Done(Expr),
}

impl Session {
    pub(super) fn evaluate_rule_lhs(&mut self, lhs: Expr) -> Result<Expr, EvalError> {
        let Expr::Call { head, args } = &lhs else {
            return self.evaluate(lhs);
        };
        let pattern_head = matcher_pattern_head(head);
        if rule_lhs_inert_pattern(pattern_head, args)
            || (args.is_empty()
                && matches!(
                    rule_lhs_policy(pattern_head, 0),
                    RuleLhsPolicy::WrapperArguments
                ))
        {
            return Ok(lhs);
        }
        if pattern_head.is_none() && !definition_lhs_contains_pattern_syntax(&lhs) {
            return self.evaluate(lhs);
        }
        // Every remaining root must descend to a head or an argument.
        self.drive_rule_lhs(
            RuleLhsState {
                stack: Vec::with_capacity(4),
            },
            RuleLhsStep::Visit(lhs, RuleLhsVisit::Structural),
        )
    }

    #[inline(never)]
    fn drive_rule_lhs(
        &mut self,
        mut state: RuleLhsState,
        mut step: RuleLhsStep,
    ) -> Result<Expr, EvalError> {
        loop {
            let (expr, mode) = match step {
                RuleLhsStep::Done(value) => return Ok(value),
                RuleLhsStep::Visit(expr, mode) => (expr, mode),
            };
            if matches!(mode, RuleLhsVisit::EvaluateOnly)
                || matches!(expr, Expr::Atom(_))
                || (matches!(mode, RuleLhsVisit::Classify)
                    && !definition_lhs_contains_pattern_syntax(&expr))
            {
                let value = match self.evaluate(expr) {
                    Ok(value) => value,
                    Err(error @ EvalError::InputRequired { .. }) => {
                        return self.suspend_rule_lhs(state, error);
                    }
                    Err(error) => return Err(error),
                };
                let destination = state.destination();
                step = accept_rule_lhs_value(&mut state, destination, value);
                continue;
            }
            let Expr::Call { head, args } = expr else {
                unreachable!("atoms were evaluated above");
            };
            let pattern_head = matcher_pattern_head(&head);
            if rule_lhs_inert_pattern(pattern_head, &args) {
                let value = Expr::call_shared(head, args);
                let destination = state.destination();
                step = accept_rule_lhs_value(&mut state, destination, value);
                continue;
            }
            let policy = rule_lhs_policy(pattern_head, args.len());
            // Keep the cached dispatch consistent with the canonical wrapper set.
            debug_assert_eq!(
                matches!(policy, RuleLhsPolicy::WrapperArguments),
                definition_lhs_pattern_wrapper_head(&head),
            );
            let mut work = RuleLhsWork {
                original_head: head.as_ref().clone(),
                original_args: args,
                resolved_head: None,
                prefix: Vec::new(),
                next: 0,
                policy,
            };
            work.prefix = Vec::with_capacity(work.original_args.len());
            if matches!(policy, RuleLhsPolicy::NamedInner) {
                work.prefix.push(work.original_args[0].clone());
                work.next = 1;
            }
            if matches!(policy, RuleLhsPolicy::OrdinaryCall) {
                step = RuleLhsStep::Visit(work.original_head.clone(), RuleLhsVisit::Classify);
                state.stack.push(work);
            } else if work.next < work.original_args.len() {
                step = rule_lhs_child(&work);
                state.stack.push(work);
            } else {
                let value = Expr::call(work.original_head, work.prefix);
                let destination = state.destination();
                step = accept_rule_lhs_value(&mut state, destination, value);
            }
        }
    }

    #[cold]
    fn suspend_rule_lhs(
        &mut self,
        state: RuleLhsState,
        error: EvalError,
    ) -> Result<Expr, EvalError> {
        // A direct root evaluation already owns its evaluator continuation.
        if state.stack.is_empty() {
            return Err(error);
        }
        let destination = state.destination();
        let pending_expr = state.pending_expr(destination).clone();
        crate::resume_or_suspend!(insert self, RulePatternSuspensionFrame {
            state,
            pending: RuleLhsAwait { destination, pending_expr },
        });
        Err(error)
    }
}

// No guest position is reachable in these leaves. Keep their shared raw shape
// just as for HoldPattern, without constructing an empty child work record.
fn rule_lhs_inert_pattern(head: Option<MatcherPatternHead>, args: &SharedArgs) -> bool {
    match head {
        Some(MatcherPatternHead::HoldPattern) => true,
        Some(
            MatcherPatternHead::Blank
            | MatcherPatternHead::BlankSequence
            | MatcherPatternHead::BlankNullSequence,
        ) => args.is_empty(),
        Some(MatcherPatternHead::Pattern) if args.len() == 2 => {
            if let Expr::Call { head, args } = &args[1] {
                args.is_empty() && definition_lhs_blank_head(head)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn rule_lhs_policy(head: Option<MatcherPatternHead>, arity: usize) -> RuleLhsPolicy {
    match head {
        Some(MatcherPatternHead::Pattern) if arity == 2 => RuleLhsPolicy::NamedInner,
        Some(
            MatcherPatternHead::Blank
            | MatcherPatternHead::BlankSequence
            | MatcherPatternHead::BlankNullSequence,
        ) => RuleLhsPolicy::BlankArguments,
        Some(MatcherPatternHead::PatternTest | MatcherPatternHead::Condition) if arity == 2 => {
            RuleLhsPolicy::FirstOperand
        }
        Some(
            MatcherPatternHead::Alternatives
            | MatcherPatternHead::Optional
            | MatcherPatternHead::Except
            | MatcherPatternHead::Repeated
            | MatcherPatternHead::RepeatedNull,
        ) => RuleLhsPolicy::WrapperArguments,
        _ => RuleLhsPolicy::OrdinaryCall,
    }
}

fn rule_lhs_child(work: &RuleLhsWork) -> RuleLhsStep {
    let mode = match work.policy {
        RuleLhsPolicy::BlankArguments => RuleLhsVisit::EvaluateOnly,
        RuleLhsPolicy::OrdinaryCall => RuleLhsVisit::Classify,
        _ => RuleLhsVisit::Structural,
    };
    RuleLhsStep::Visit(work.original_args[work.next].clone(), mode)
}

#[inline(always)]
fn accept_rule_lhs_value(
    state: &mut RuleLhsState,
    mut destination: RuleLhsDestination,
    mut value: Expr,
) -> RuleLhsStep {
    loop {
        if matches!(destination, RuleLhsDestination::Root) {
            return RuleLhsStep::Done(value);
        }
        let work = state
            .stack
            .last_mut()
            .expect("child destination has a parent");
        match destination {
            RuleLhsDestination::Root => unreachable!(),
            RuleLhsDestination::Head => work.resolved_head = Some(value),
            RuleLhsDestination::Argument(index) => {
                debug_assert_eq!(work.next, index);
                work.prefix.push(value);
                work.next = index + 1;
                if matches!(work.policy, RuleLhsPolicy::FirstOperand) {
                    work.prefix.push(work.original_args[1].clone());
                    work.next = 2;
                }
            }
        }
        if work.next < work.original_args.len() {
            return rule_lhs_child(work);
        }
        let work = state.stack.pop().expect("completed parent");
        value = Expr::call(
            work.resolved_head.unwrap_or(work.original_head),
            work.prefix,
        );
        destination = state.destination();
    }
}
