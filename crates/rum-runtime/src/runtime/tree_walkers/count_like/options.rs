use super::*;
use crate::runtime::kernel_api::{FrozenOptionReadContinuation, OptionReadContinuation};
pub(super) struct CountLikeOptionWorking<'a> {
    pub(super) target: Expr,
    pub(super) pattern: Expr,
    pub(super) args: &'a SharedArgs,
    pub(super) finish: CountLikeFinish,
}

#[derive(FrameRootTrace)]
pub(super) struct CountLikeOptionRead {
    target: Expr,
    pattern: Expr,
    args: SharedArgs,
    finish: CountLikeFinish,
}

impl OptionReadContinuation<bool> for CountLikeOptionWorking<'_> {
    type Frozen = CountLikeOptionRead;
    fn finish(self, session: &mut Session, include_heads: bool) -> Result<Expr, EvalError> {
        let (name, mut defaults) = match self.finish {
            CountLikeFinish::Count => ("Count", CasesTraversalConfig::default_cases()),
            CountLikeFinish::MemberQ => ("MemberQ", CasesTraversalConfig::default_member_q()),
            CountLikeFinish::FreeQ => ("FreeQ", CasesTraversalConfig::default_free_q()),
        };
        defaults.include_heads = include_heads;
        session.finish_count_like_after_pattern(
            name,
            self.target,
            self.pattern,
            self.args,
            defaults,
            self.finish,
        )
    }
    fn freeze(self) -> Self::Frozen {
        CountLikeOptionRead {
            target: self.target,
            pattern: self.pattern,
            args: self.args.clone(),
            finish: self.finish,
        }
    }
}
impl FrozenOptionReadContinuation<bool> for CountLikeOptionRead {
    fn finish(&self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        CountLikeOptionWorking {
            target: self.target.clone(),
            pattern: self.pattern.clone(),
            args: &self.args,
            finish: self.finish,
        }
        .finish(session, value)
    }
}
