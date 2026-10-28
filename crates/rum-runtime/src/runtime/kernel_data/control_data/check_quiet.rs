use super::*;
use crate::runtime::kernel_api::{
    ContinueToken, SlotWitness, TypedAction, TypedFrame, slot_witness,
};
use crate::runtime::model::OutputMark;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

impl Session {
    pub(in crate::runtime) fn eval_check(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Check",
                args,
                "argx",
                format!("expected 2 or 3 argument(s), got {}", args.len()),
            ));
        }

        let selectors = match self.normalize_optional_message_selectors(args.get(2)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Check", args, error.message()));
            }
        };

        let fallback = args[1].clone();
        let mark = self.output_mark();
        let primary_result = self.evaluate(args[0].clone());
        let witness = slot_witness(&primary_result);
        match primary_result {
            Err(EvalError::InputRequired { request }) => {
                suspend_check(self, selectors, fallback, mark, witness, request)
            }
            Ok(value) => {
                self.resume_check_after_primary(selectors.as_deref(), fallback, mark, Ok(value))
            }
            Err(error) => {
                self.resume_check_after_primary(selectors.as_deref(), fallback, mark, Err(error))
            }
        }
    }

    pub(in crate::runtime) fn eval_quiet(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Quiet",
                args,
                "argx",
                format!("expected 1 or 2 argument(s), got {}", args.len()),
            ));
        }

        let selectors = match self.normalize_optional_message_selectors(args.get(1)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Quiet", args, error.message()));
            }
        };

        let mark = self.output_mark();
        let primary_result = self.evaluate(args[0].clone());
        let witness = slot_witness(&primary_result);
        match primary_result {
            Err(EvalError::InputRequired { request }) => {
                suspend_quiet(self, selectors, mark, witness, request)
            }
            Ok(value) => self.resume_quiet_after_primary(selectors.as_deref(), mark, Ok(value)),
            Err(error) => self.resume_quiet_after_primary(selectors.as_deref(), mark, Err(error)),
        }
    }

    fn resume_check_after_primary(
        &mut self,
        selectors: Option<&[MessageSelector]>,
        fallback: Expr,
        mark: OutputMark,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                let matched = selectors.map_or_else(
                    || self.messages.has_after(mark),
                    |selectors| self.messages_since_match(mark, selectors),
                );

                self.handled_non_loader_eval_errors.truncate_after(mark);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
                );

                if matched {
                    self.evaluate(fallback)
                } else {
                    Ok(value)
                }
            }
            Err(err) => {
                self.emit_non_loader_eval_message(mark, &err);

                let matched = selectors.map_or_else(
                    || self.messages.has_after(mark),
                    |selectors| self.messages_since_match(mark, selectors),
                );

                if matched {
                    self.evaluate(fallback)
                } else {
                    Err(err)
                }
            }
        }
    }

    fn resume_quiet_after_primary(
        &mut self,
        selectors: Option<&[MessageSelector]>,
        mark: OutputMark,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let outcome = match outcome {
            Ok(value) => Ok(value),
            Err(err) => {
                self.emit_non_loader_eval_message(mark, &err);
                Err(err)
            }
        };

        if let Some(selectors) = selectors {
            self.suppress_messages_since(mark, selectors);
        } else {
            self.messages.truncate_after(mark);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::Messages,
            );
        }

        if outcome.is_ok() {
            self.handled_non_loader_eval_errors.truncate_after(mark);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
            );
        }

        outcome
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct EvalCheckSuspensionFrame {
    selectors: Option<Arc<[MessageSelector]>>,
    fallback: Expr,
    mark: OutputMark,
}

impl TypedFrame for EvalCheckSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn output_boundary(&self) -> crate::runtime::suspension::OutputBoundary {
        crate::runtime::suspension::OutputBoundary::messages(self.mark)
    }

    fn resume<'resume>(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        TypedAction::complete(session.resume_check_after_primary(
            self.selectors.as_deref(),
            self.fallback.clone(),
            self.mark,
            outcome,
        ))
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalCheckSuspensionFrame {
    fn abandon(
        self: Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_message_capture(self.mark);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct EvalQuietSuspensionFrame {
    selectors: Option<Arc<[MessageSelector]>>,
    mark: OutputMark,
}

impl TypedFrame for EvalQuietSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn output_boundary(&self) -> crate::runtime::suspension::OutputBoundary {
        crate::runtime::suspension::OutputBoundary::messages(self.mark)
    }

    fn resume<'resume>(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        TypedAction::complete(session.resume_quiet_after_primary(
            self.selectors.as_deref(),
            self.mark,
            outcome,
        ))
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalQuietSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_message_capture(self.mark);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[cold]
#[inline(never)]
fn suspend_check(
    session: &mut Session,
    selectors: Option<Vec<MessageSelector>>,
    fallback: Expr,
    mark: OutputMark,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        EvalCheckSuspensionFrame {
            selectors: selectors.map(Arc::<[MessageSelector]>::from),
            fallback,
            mark,
        },
    )
}

#[cold]
#[inline(never)]
fn suspend_quiet(
    session: &mut Session,
    selectors: Option<Vec<MessageSelector>>,
    mark: OutputMark,
    witness: SlotWitness<Expr>,
    request: RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        EvalQuietSuspensionFrame {
            selectors: selectors.map(Arc::<[MessageSelector]>::from),
            mark,
        },
    )
}
