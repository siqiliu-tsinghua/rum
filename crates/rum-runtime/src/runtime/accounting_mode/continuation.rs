use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

struct AccountedCompoundFrame {
    args: SharedArgs,
    index: usize,
    shadow: AccountingShadow,
}

impl FrameRootTrace for AccountedCompoundFrame {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.args.trace_roots(visitor);
    }
}

impl FrameAbandon for AccountedCompoundFrame {
    fn abandon(self: Arc<Self>, _context: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        FrameAbandonAction::Discard
    }
}

impl SuspensionFrame for AccountedCompoundFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        self.shadow.0.resumes.fetch_add(1, Ordering::Relaxed);
        let result = match outcome {
            Ok(value) if self.index + 1 >= self.args.len() => Ok(value),
            Ok(_) => AccountingContext {
                session,
                shadow: self.shadow.clone(),
            }
            .compound_from(&self.args, self.index + 1),
            Err(error) => Err(error),
        };
        FrameAction::complete(result)
    }
}

impl AccountingContext<'_> {
    pub(super) fn compound_from(
        &mut self,
        args: &SharedArgs,
        start: usize,
    ) -> Result<Expr, EvalError> {
        let mut value = system_expr("Null");
        for index in start..args.len() {
            match self.evaluate(args[index].clone()) {
                Ok(next) => value = next,
                Err(EvalError::InputRequired { request }) => {
                    self.shadow.0.suspends.fetch_add(1, Ordering::Relaxed);
                    self.session
                        .insert_suspension_frame(AccountedCompoundFrame {
                            args: args.clone(),
                            index,
                            shadow: self.shadow.clone(),
                        });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
        Ok(value)
    }
}
