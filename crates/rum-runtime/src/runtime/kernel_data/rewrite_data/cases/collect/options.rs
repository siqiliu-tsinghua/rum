use super::*;
use crate::runtime::kernel_api::{FrozenOptionReadContinuation, OptionReadContinuation};

pub(super) enum CasesOptionWorking {
    Owned {
        target: Expr,
        spec: CasesSpec,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    },
    Suspended {
        target: Expr,
        spec: FrozenRewriteValue<CasesSpec>,
        call_args: SharedArgs,
    },
}

#[derive(FrameRootTrace)]
pub(super) struct CasesOptionRead {
    target: Expr,
    spec: FrozenRewriteValue<CasesSpec>,
    call_args: SharedArgs,
}

impl OptionReadContinuation<bool> for CasesOptionWorking {
    type Frozen = CasesOptionRead;
    #[inline(always)]
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
                    spec,
                    call_args,
                    ..
                } => session
                    .finish_cases_after_resolved_traversal(target, spec, traversal, call_args),
                Self::Suspended {
                    target,
                    spec,
                    call_args,
                } => session.finish_cases_after_resolved_traversal_suspended(
                    target, spec, traversal, call_args,
                ),
            },
            Err(EvalError::InputRequired { request }) => {
                let CasesOptionRead {
                    target,
                    spec,
                    call_args,
                } = self.freeze();
                crate::resume_or_suspend!(insert session, ResumeCasesAfterResolvedSpecSuspensionFrame { target, spec, call_args });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }
    fn freeze(self) -> Self::Frozen {
        match self {
            Self::Owned {
                target,
                spec,
                call_args,
                ..
            } => CasesOptionRead {
                target,
                spec: FrozenRewriteValue::freeze(spec),
                call_args: SharedArgs::new(call_args),
            },
            Self::Suspended {
                target,
                spec,
                call_args,
            } => CasesOptionRead {
                target,
                spec,
                call_args,
            },
        }
    }
}

impl FrozenOptionReadContinuation<bool> for CasesOptionRead {
    fn finish(&self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        CasesOptionWorking::Suspended {
            target: self.target.clone(),
            spec: self.spec.clone(),
            call_args: self.call_args.clone(),
        }
        .finish(session, value)
    }
}
