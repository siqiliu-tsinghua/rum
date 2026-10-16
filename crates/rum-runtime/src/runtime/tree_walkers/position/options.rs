use super::*;
use crate::runtime::kernel_api::{FrozenOptionReadContinuation, OptionReadContinuation};
pub(super) struct PositionOptionWorking<'a> {
    pub(super) target: Expr,
    pub(super) pattern: Expr,
    pub(super) args: &'a SharedArgs,
}
#[derive(FrameRootTrace)]
pub(super) struct PositionOptionRead {
    target: Expr,
    pattern: Expr,
    args: SharedArgs,
}
impl OptionReadContinuation<bool> for PositionOptionWorking<'_> {
    type Frozen = PositionOptionRead;
    fn finish(self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        session.finish_position_option(self.target, self.pattern, self.args, value)
    }
    fn freeze(self) -> Self::Frozen {
        PositionOptionRead {
            target: self.target,
            pattern: self.pattern,
            args: self.args.clone(),
        }
    }
}
impl FrozenOptionReadContinuation<bool> for PositionOptionRead {
    fn finish(&self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        session.finish_position_option(self.target.clone(), self.pattern.clone(), &self.args, value)
    }
}
impl Session {
    fn finish_position_option(
        &mut self,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
        include_heads: bool,
    ) -> Result<Expr, EvalError> {
        let mut defaults = CasesTraversalConfig::default_position();
        defaults.include_heads = include_heads;
        let traversal_result = self.resolve_position_traversal_owned(args[2..].to_vec(), defaults);
        let witness = slot_witness(&traversal_result);
        match traversal_result {
            Ok(traversal) => {
                self.finish_position_after_resolved_traversal(target, pattern, traversal, args)
            }
            Err(EvalError::InputRequired { request }) => {
                suspend_position_pattern(self, witness, request, target, pattern, args)
            }
            Err(error) => Err(error),
        }
    }
}

#[cold]
#[inline(never)]
fn suspend_position_pattern(
    session: &mut Session,
    witness: SlotWitness<CasesTraversalResolution>,
    request: RuntimeInputRequest,
    target: Expr,
    pattern: Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.clone();
    witness.suspend(
        session,
        request,
        ResumePositionAfterPatternSuspensionFrame {
            target,
            pattern,
            args,
        },
    )
}
