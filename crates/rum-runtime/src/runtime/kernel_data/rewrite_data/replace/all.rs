use super::all_application::{ReplaceAllApplicationFrame, ReplaceAllUnheldApplicationFrame};
use super::*;
use crate::runtime::kernel_api::{SlotWitness, slot_witness};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_replace_all(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() == 1 {
            return Ok(self.invalid_call("ReplaceAll", args));
        }
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplaceAll", args, 2));
        }

        let target_result = self.evaluate(args[0].clone());
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let rules_expr = args[1].clone();
        EvalReplaceAllSuspensionFrame { rules_expr }
        },
        )?;

        self.resume_replace_all_after_target(target, args[1].clone())
    }

    #[allow(dead_code)]
    fn resume_replace_all_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rules_owned(
            rules_expr.clone(),
            CompiledRewriteRuleContinuation::ReplaceAll {
                target,
                rules_input: rules_expr,
            },
        )
    }

    pub(in crate::runtime) fn resume_replace_all_after_rules(
        &mut self,
        target: Expr,
        rules_input: Expr,
        normalized_rules: Option<Vec<RewriteRule>>,
    ) -> Result<Expr, EvalError> {
        let Some(rules) = normalized_rules else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAll",
                &[target, rules_input],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let replaced_result =
            self.apply_replace_all(target, &rules, RuleApplicationSource::ReplaceAll);
        let witness = slot_witness(&replaced_result);
        match replaced_result {
            Ok(replaced) => self.finish_replace_all_application(replaced),
            Err(EvalError::InputRequired { request }) => {
                self.suspend_replace_all_application(witness, request)
            }
            Err(error) => Err(error),
        }
    }

    #[inline(always)]
    pub(super) fn finish_replace_all_application(
        &mut self,
        replaced: Expr,
    ) -> Result<Expr, EvalError> {
        if expr_is_self_evaluating_atom(&replaced) {
            return Ok(replaced);
        }
        let replaced_result = self.evaluate(replaced);
        crate::resume_or_suspend!(
            self,
            replaced_result,
            ok(value) => Ok(value),
            suspend => ResumeReplaceAllAfterRulesSuspensionFrame,
        )
    }

    pub(in crate::runtime) fn eval_replace_all_unheld(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() == 1 {
            return Ok(self.invalid_call("ReplaceAllUnheld", args));
        }
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplaceAllUnheld", args, 2));
        }

        let target_result = self.evaluate(args[0].clone());
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let rules_expr = args[1].clone();
        EvalReplaceAllUnheldSuspensionFrame { rules_expr }
        },
        )?;

        self.resume_replace_all_unheld_after_target(target, args[1].clone())
    }

    #[allow(dead_code)]
    fn resume_replace_all_unheld_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rules_owned(
            rules_expr.clone(),
            CompiledRewriteRuleContinuation::ReplaceAllUnheld {
                target,
                rules_input: rules_expr,
            },
        )
    }

    pub(in crate::runtime) fn resume_replace_all_unheld_after_rules(
        &mut self,
        target: Expr,
        rules_input: Expr,
        normalized_rules: Option<Vec<RewriteRule>>,
    ) -> Result<Expr, EvalError> {
        let Some(rules) = normalized_rules else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAllUnheld",
                &[target, rules_input],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let replaced_result =
            self.apply_replace_all(target, &rules, RuleApplicationSource::ReplaceAllUnheld);
        let witness = slot_witness(&replaced_result);
        match replaced_result {
            Ok(replaced) => self.finish_replace_all_unheld_application(replaced),
            Err(EvalError::InputRequired { request }) => {
                self.suspend_replace_all_unheld_application(witness, request)
            }
            Err(error) => Err(error),
        }
    }

    #[inline(always)]
    pub(super) fn finish_replace_all_unheld_application(
        &mut self,
        replaced: Expr,
    ) -> Result<Expr, EvalError> {
        if expr_is_self_evaluating_atom(&replaced) {
            return Ok(replaced);
        }
        let replaced_result = self.evaluate(replaced);
        crate::resume_or_suspend!(
            self,
            replaced_result,
            ok(value) => Ok(value),
            suspend => ResumeReplaceAllUnheldAfterRulesSuspensionFrame,
        )
    }

    #[cold]
    #[inline(never)]
    pub(super) fn suspend_replace_all_application(
        &mut self,
        witness: SlotWitness<Expr>,
        request: RuntimeInputRequest,
    ) -> Result<Expr, EvalError> {
        witness.suspend(self, request, ReplaceAllApplicationFrame)
    }

    #[cold]
    #[inline(never)]
    pub(super) fn suspend_replace_all_unheld_application(
        &mut self,
        witness: SlotWitness<Expr>,
        request: RuntimeInputRequest,
    ) -> Result<Expr, EvalError> {
        witness.suspend(self, request, ReplaceAllUnheldApplicationFrame)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalReplaceAllSuspensionFrame {
    rules_expr: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalReplaceAllSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut rules_expr = self.rules_expr.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_replace_all_after_target(target, rules_expr))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReplaceAllAfterRulesSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for ResumeReplaceAllAfterRulesSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalReplaceAllUnheldSuspensionFrame {
    rules_expr: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalReplaceAllUnheldSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut rules_expr = self.rules_expr.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_replace_all_unheld_after_target(target, rules_expr))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReplaceAllUnheldAfterRulesSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame
    for ResumeReplaceAllUnheldAfterRulesSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
