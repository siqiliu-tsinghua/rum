use super::*;
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

        let mut selectors = match self.normalize_optional_message_selectors(args.get(2)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Check", args, error.message()));
            }
        };

        let fallback = args[1].clone();
        let message_checkpoint = self.messages.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let primary_result = self.evaluate(args[0].clone());
        let outcome = crate::resume_or_suspend!(
            self,
            primary_result,
            ok(value) => Ok(value),
            suspend => {
        let selectors = selectors.take().map(Arc::<[MessageSelector]>::from);
                    let fallback = fallback.clone();
        EvalCheckSuspensionFrame { selectors, fallback, message_checkpoint, handled_error_checkpoint }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_check_after_primary(
                selectors.as_deref(),
                fallback,
                message_checkpoint,
                handled_error_checkpoint,
                outcome,
            ),
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

        let mut selectors = match self.normalize_optional_message_selectors(args.get(1)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Quiet", args, error.message()));
            }
        };

        let message_checkpoint = self.messages.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let primary_result = self.evaluate(args[0].clone());
        let result = crate::resume_or_suspend!(
            self,
            primary_result,
            ok(value) => Ok(value),
            suspend => {
        let selectors = selectors.take().map(Arc::<[MessageSelector]>::from);
        EvalQuietSuspensionFrame { selectors, message_checkpoint, handled_error_checkpoint }
        },
        );

        match result {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_quiet_after_primary(
                selectors.as_deref(),
                message_checkpoint,
                handled_error_checkpoint,
                outcome,
            ),
        }
    }

    fn resume_check_after_primary(
        &mut self,
        selectors: Option<&[MessageSelector]>,
        fallback: Expr,
        message_checkpoint: usize,
        handled_error_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                let matched = selectors.map_or_else(
                    || self.messages.len() > message_checkpoint,
                    |selectors| self.messages_since_match(message_checkpoint, selectors),
                );

                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);
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
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);

                let matched = selectors.map_or_else(
                    || self.messages.len() > message_checkpoint,
                    |selectors| self.messages_since_match(message_checkpoint, selectors),
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
        message_checkpoint: usize,
        handled_error_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let outcome = match outcome {
            Ok(value) => Ok(value),
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);
                Err(err)
            }
        };

        if let Some(selectors) = selectors {
            self.suppress_messages_since(message_checkpoint, selectors);
        } else {
            self.messages.truncate(message_checkpoint);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::Messages,
            );
        }

        if outcome.is_ok() {
            self.handled_non_loader_eval_errors
                .truncate(handled_error_checkpoint);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
            );
        }

        outcome
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct EvalCheckSuspensionFrame {
    selectors: std::option::Option<std::sync::Arc<[crate::runtime::MessageSelector]>>,
    fallback: rum_core::Expr,
    message_checkpoint: usize,
    handled_error_checkpoint: usize,
}

impl crate::runtime::suspension::SuspensionFrame for EvalCheckSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut selectors = self.selectors.clone();
        #[allow(unused_mut)]
        let mut fallback = self.fallback.clone();
        #[allow(unused_mut)]
        let mut message_checkpoint = self.message_checkpoint.clone();
        #[allow(unused_mut)]
        let mut handled_error_checkpoint = self.handled_error_checkpoint.clone();
        let resumed = (|| {
            session.resume_check_after_primary(
                selectors.as_deref(),
                fallback,
                message_checkpoint,
                handled_error_checkpoint,
                outcome,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalCheckSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_message_capture(self.message_checkpoint, self.handled_error_checkpoint);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct EvalQuietSuspensionFrame {
    selectors: std::option::Option<std::sync::Arc<[crate::runtime::MessageSelector]>>,
    message_checkpoint: usize,
    handled_error_checkpoint: usize,
}

impl crate::runtime::suspension::SuspensionFrame for EvalQuietSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut selectors = self.selectors.clone();
        #[allow(unused_mut)]
        let mut message_checkpoint = self.message_checkpoint.clone();
        #[allow(unused_mut)]
        let mut handled_error_checkpoint = self.handled_error_checkpoint.clone();
        let resumed = (|| {
            session.resume_quiet_after_primary(
                selectors.as_deref(),
                message_checkpoint,
                handled_error_checkpoint,
                outcome,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for EvalQuietSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_message_capture(self.message_checkpoint, self.handled_error_checkpoint);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}
