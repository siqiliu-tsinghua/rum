use super::*;
use crate::runtime::options::{FrozenOptionReadContinuation, OptionReadContinuation};

pub(super) struct CheckAbortOptionWorking<'a> {
    pub(super) args: &'a SharedArgs,
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct CheckAbortOptionRead {
    args: SharedArgs,
}

impl OptionReadContinuation<Option<Expr>> for CheckAbortOptionWorking<'_> {
    type Frozen = CheckAbortOptionRead;
    fn finish(self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        session.finish_check_abort_default(self.args, value)
    }
    fn freeze(self) -> Self::Frozen {
        CheckAbortOptionRead {
            args: self.args.clone(),
        }
    }
}

impl FrozenOptionReadContinuation<Option<Expr>> for CheckAbortOptionRead {
    fn finish(&self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        session.finish_check_abort_default(&self.args, value)
    }
}

impl Session {
    fn finish_check_abort_default(
        &mut self,
        args: &SharedArgs,
        default_option: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        let default_option_exprs: Vec<Expr> = default_option.into_iter().collect();
        let default_propagation =
            match parse_check_abort_propagation_source(&default_option_exprs, false) {
                Ok(Some(source)) => check_abort_propagation_from_value(source.value),
                Ok(None) => default_check_abort_propagation(),
                Err(error) => return Ok(self.invalid_check_abort_option_call(args_slice, error)),
            };

        let propagation_source = match parse_check_abort_propagation_source(&args_slice[2..], true)
        {
            Ok(source) => source,
            Err(error) => return Ok(self.invalid_check_abort_option_call(args_slice, error)),
        };
        let Some(source) = propagation_source else {
            return self.resume_check_abort_with_propagation(args, default_propagation);
        };
        if source.evaluate_rhs {
            let value_result = self.evaluate(source.value.clone());
            let value = crate::resume_or_suspend!(
                self,
                value_result,
                ok(value) => Ok(value),
                suspend => {
            let args = args.clone();
            EvalCheckAbortSuspensionFrame { args }
            },
            )?;

            return self.resume_check_abort_with_propagation(
                args,
                check_abort_propagation_from_value(value),
            );
        }

        self.resume_check_abort_with_propagation(
            args,
            check_abort_propagation_from_value(source.value),
        )
    }
}
