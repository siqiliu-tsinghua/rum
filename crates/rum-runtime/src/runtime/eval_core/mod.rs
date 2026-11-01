use super::*;
use crate::runtime::accounting_mode::{AccountingMode, Unarmed};
use crate::runtime::native_stack_guard::native_stack_exhausted;

use std::time::Instant;

pub(in crate::runtime) mod call;

mod stability;

mod symbol;

const INTERRUPT_POLL_MASK: u64 = 1023;

impl Session {
    pub(super) fn evaluate_once_unstable_after_input_check_with_cycle_tracking(
        &mut self,
        expr: Expr,
        track_cycle: bool,
    ) -> Result<Expr, EvalError> {
        if self.current_memory_limit.is_some() {
            let mode = self.ensure_accounting_shadow();
            self.evaluate_once_in_accounting_mode(expr, track_cycle, mode)
        } else {
            self.evaluate_once_in_accounting_mode(expr, track_cycle, Unarmed)
        }
    }

    fn evaluate_once_in_accounting_mode<M: AccountingMode>(
        &mut self,
        expr: Expr,
        track_cycle: bool,
        mode: M,
    ) -> Result<Expr, EvalError> {
        let evaluation_depth = self.evaluation_depth.get();
        if self.interrupt_armed {
            self.poll_host_interrupt_if_due()?;
        }
        if track_cycle
            && self
                .evaluation_stack
                .iter()
                .any(|active| active.epoch == self.evaluation_epoch && active.expr == expr)
        {
            return Err(EvalError::EvaluationCycleDetected {
                expr: expr.to_string(),
            });
        }

        if evaluation_depth >= self.resource_limits.recursion_limit {
            return Err(EvalError::RecursionLimitExceeded {
                limit: self.resource_limits.recursion_limit,
            });
        }
        if self.evaluation_depth.native_check_due(evaluation_depth) {
            self.poll_native_stack_limit()?;
        }

        let stack_payload = if track_cycle {
            let entry = EvaluationStackEntry {
                expr: expr.clone(),
                epoch: self.evaluation_epoch,
            };
            if M::ARMED {
                self.push_accounted_evaluation_entry(entry)
            } else {
                self.evaluation_stack.push(entry);
                0
            }
        } else {
            0
        };
        self.evaluation_depth.enter_level(evaluation_depth);
        let shadow_stack = mode.stack_push(track_cycle);
        let result = self.evaluate_inner_unstable(expr);
        self.evaluation_depth.leave_level(evaluation_depth);
        if track_cycle {
            if M::ARMED {
                self.pop_accounted_evaluation_entry(stack_payload);
            } else if matches!(&result, Err(EvalError::InputRequired { .. }))
                && self.current_memory_limit.is_some()
            {
                // This frame predates a local arming that survived suspension.
                self.pop_cold_armed_evaluation_entry();
            } else {
                self.evaluation_stack.pop();
            }
        }
        drop(shadow_stack);
        match result {
            Ok(value) => {
                self.enforce_active_resource_limits_for_expr(&value)?;
                Ok(value)
            }
            Err(error) => Err(error),
        }
    }

    #[cold]
    #[inline(never)]
    fn poll_native_stack_limit(&mut self) -> Result<(), EvalError> {
        let stack_marker = 0_u8;
        let current_addr = (&stack_marker as *const u8) as usize;
        if native_stack_exhausted(current_addr, self.resource_limits.native_stack_limit) {
            return Err(EvalError::StackLimitExceeded {
                limit: self.resource_limits.native_stack_limit,
            });
        }
        Ok(())
    }

    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn native_stack_budget_exhausted(&self) -> bool {
        let stack_marker = 0_u8;
        let current_addr = (&stack_marker as *const u8) as usize;
        native_stack_exhausted(current_addr, self.resource_limits.native_stack_limit)
    }

    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn poll_host_interrupt_if_due(&mut self) -> Result<(), EvalError> {
        self.interrupt_poll_counter = self.interrupt_poll_counter.wrapping_add(1);
        if self.interrupt_poll_counter & INTERRUPT_POLL_MASK != 0 {
            return Ok(());
        }
        if self.active_accepted_request_is_cancelled() {
            return Err(EvalError::Cancelled);
        }
        if let Some(deadline) = self.eval_deadline {
            if Instant::now() >= deadline {
                return Err(EvalError::TimeConstraintExceeded);
            }
        }
        Ok(())
    }

    pub(super) fn evaluate_inner_unstable(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        let runtime_profile_total =
            runtime_path_timing_start_adopted(self, "evaluate_inner_unstable.total");
        let result = match expr {
            Expr::Atom(Atom::Symbol(symbol)) => {
                let runtime_profile =
                    runtime_path_timing_start_adopted(self, "evaluate_inner_unstable.symbol");
                let result = self.evaluate_symbol(symbol);
                record_profile_runtime_path_timing_adopted(self, runtime_profile, &result);
                result
            }
            Expr::Call { head, args } => {
                let runtime_profile =
                    runtime_path_timing_start_adopted(self, "evaluate_inner_unstable.call");
                let result = self.evaluate_call(head.clone(), args);
                record_profile_runtime_path_timing_adopted(self, runtime_profile, &result);
                result
            }
            other => {
                let runtime_profile =
                    runtime_path_timing_start_adopted(self, "evaluate_inner_unstable.atom");
                let result = Ok(other);
                record_profile_runtime_path_timing_adopted(self, runtime_profile, &result);
                result
            }
        };
        record_profile_runtime_path_timing_adopted(self, runtime_profile_total, &result);
        result
    }

    pub(super) fn evaluate_atom_fast_path(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        let kind = EvaluateAtomFastPathKind::classify(self, &expr);
        let runtime_profile_total =
            runtime_path_timing_start_adopted(self, "evaluate_atom_fast_path.total");
        let runtime_profile_kind = runtime_path_timing_start_adopted(self, kind.profile_name());
        let result = match kind {
            EvaluateAtomFastPathKind::StableSymbol | EvaluateAtomFastPathKind::Atom => Ok(expr),
            EvaluateAtomFastPathKind::UnstableSymbol | EvaluateAtomFastPathKind::Call => {
                self.evaluate(expr)
            }
        };
        record_profile_runtime_path_timing_adopted(self, runtime_profile_kind, &result);
        record_profile_runtime_path_timing_adopted(self, runtime_profile_total, &result);
        result
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvaluateAtomFastPathKind {
    StableSymbol,
    UnstableSymbol,
    Atom,
    Call,
}

impl EvaluateAtomFastPathKind {
    fn classify(session: &mut Session, expr: &Expr) -> Self {
        match expr {
            Expr::Atom(Atom::Symbol(symbol))
                if session.symbol_is_atom_evaluation_stable(symbol) =>
            {
                Self::StableSymbol
            }
            Expr::Atom(Atom::Symbol(_)) => Self::UnstableSymbol,
            Expr::Atom(_) => Self::Atom,
            Expr::Call { .. } => Self::Call,
        }
    }

    fn profile_name(self) -> &'static str {
        match self {
            Self::StableSymbol => "evaluate_atom_fast_path.stable_symbol",
            Self::UnstableSymbol => "evaluate_atom_fast_path.unstable_symbol",
            Self::Atom => "evaluate_atom_fast_path.atom",
            Self::Call => "evaluate_atom_fast_path.call",
        }
    }
}

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    runtime_profile_outcome_for_result(result)
}

fn runtime_profile_outcome_for_result<T>(result: &Result<T, EvalError>) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}
