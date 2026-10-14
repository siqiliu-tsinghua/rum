use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use crate::runtime::suspension::{SlotFinalizer, TypedAction};

// F0272/F0273 share one nominal frame: repeated pauses freeze fresh cursor and
// splice state through this frame's adapter-minted token, never its Arc.
#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame {
    pub(super) state: SuspendedCallArgumentPreparation,
    pub(super) allow_sequence_splice: bool,
}

impl PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame {
    #[cold]
    #[inline(never)]
    pub(super) fn freeze(
        head: Expr,
        attributes: AttributeSet,
        args: SharedArgs,
        index: usize,
        built: Option<Vec<Expr>>,
        restores: Option<Vec<(usize, Expr)>>,
        allow_sequence_splice: bool,
    ) -> Self {
        let prepared = built.unwrap_or_else(|| args[..index].to_vec());
        Self {
            state: SuspendedCallArgumentPreparation {
                head,
                attributes,
                args,
                cursor: index + 1,
                prepared: PreparedArgumentAccumulator::freeze(prepared),
                unevaluated_restores: restores.map(UnevaluatedRestoreAccumulator::freeze),
            },
            allow_sequence_splice,
        }
    }
}

impl TypedFrame for PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame {
    type Input = Expr;
    type Output = PreparedCallArguments;

    fn resume<'resume>(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<PreparedCallArguments> {
        let value = match outcome {
            Ok(value) => value,
            Err(error) => return TypedAction::complete(Err(error)),
        };
        TypedAction::finish_work(
            session.resume_suspended_call_after_argument(
                self.state.clone(),
                value,
                self.allow_sequence_splice,
                token,
            ),
            |state| FinishPreparedCallArgumentsFinalizer { state },
        )
    }
}

struct FinishPreparedCallArgumentsFinalizer {
    state: SuspendedPreparedCallArguments,
}

impl SlotFinalizer for FinishPreparedCallArgumentsFinalizer {
    type Output = PreparedCallArguments;

    fn finish(self: Box<Self>) -> Result<PreparedCallArguments, EvalError> {
        Ok(finish_suspended_prepared_call(self.state))
    }
}
