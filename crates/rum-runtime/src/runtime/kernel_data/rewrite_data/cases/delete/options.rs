use super::*;
use crate::runtime::kernel_api::{FrozenOptionReadContinuation, OptionReadContinuation};

pub(super) enum DeleteCasesOptionWorking {
    Owned {
        target: Expr,
        pattern: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    },
    Suspended {
        target: Expr,
        pattern: Expr,
        call_args: SharedArgs,
    },
}
#[derive(FrameRootTrace)]
pub(super) struct DeleteCasesOptionRead {
    target: Expr,
    pattern: Expr,
    call_args: SharedArgs,
}

impl OptionReadContinuation<bool> for DeleteCasesOptionWorking {
    type Frozen = DeleteCasesOptionRead;
    fn finish(mut self, session: &mut Session, include_heads: bool) -> Result<Expr, EvalError> {
        let defaults = CasesTraversalConfig {
            include_heads,
            ..CasesTraversalConfig::default_cases()
        };
        let traversal_result = match &mut self {
            Self::Owned { option_exprs, .. } => {
                session.resolve_cases_traversal_owned(std::mem::take(option_exprs), defaults)
            }
            Self::Suspended { call_args, .. } => {
                session.resolve_cases_traversal_shared_from(call_args.clone(), 2, defaults)
            }
        };
        match traversal_result {
            Ok(traversal) => match self {
                Self::Owned {
                    target,
                    pattern,
                    call_args,
                    ..
                } => session.finish_delete_cases_after_resolved_traversal(
                    target, pattern, traversal, call_args,
                ),
                Self::Suspended {
                    target,
                    pattern,
                    call_args,
                } => session.finish_delete_cases_after_resolved_traversal_suspended(
                    target, pattern, traversal, call_args,
                ),
            },
            Err(EvalError::InputRequired { request }) => {
                let DeleteCasesOptionRead {
                    target,
                    pattern,
                    call_args,
                } = self.freeze();
                crate::resume_or_suspend!(insert session, ResumeDeleteCasesAfterPatternSuspensionFrame { target, pattern, call_args });
                crate::resume_or_suspend!(insert session, ContinueDeleteCasesFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }
    fn freeze(self) -> Self::Frozen {
        match self {
            Self::Owned {
                target,
                pattern,
                call_args,
                ..
            } => DeleteCasesOptionRead {
                target,
                pattern,
                call_args: SharedArgs::new(call_args),
            },
            Self::Suspended {
                target,
                pattern,
                call_args,
            } => DeleteCasesOptionRead {
                target,
                pattern,
                call_args,
            },
        }
    }
}
impl FrozenOptionReadContinuation<bool> for DeleteCasesOptionRead {
    fn finish(&self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        DeleteCasesOptionWorking::Suspended {
            target: self.target.clone(),
            pattern: self.pattern.clone(),
            call_args: self.call_args.clone(),
        }
        .finish(session, value)
    }
}
