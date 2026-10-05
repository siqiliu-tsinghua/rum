//! Typed option reads complete their consumer on both ordinary and resumed paths.
use super::*;
use crate::runtime::suspension::{FrameAction, SuspensionFrame};
use std::sync::Arc;

/// The remaining builtin work after a typed option read.
///
/// Working data may borrow arguments and move already completed values, but must
/// not retain a Session borrow. `freeze` is called only for `InputRequired`.
/// `finish` returns the completed builtin expression, including its own framed
/// suspensions. Return the reader's result directly from the calling phase.
/// This protocol is independent of the certified merged `OptionReentry` domain.
pub trait OptionReadContinuation<V> {
    /// Immutable, rooted data retained only while a read is suspended.
    type Frozen: FrozenOptionReadContinuation<V>;
    /// Complete the builtin using the decoded option value.
    fn finish(self, session: &mut Session, value: V) -> Result<Expr, EvalError>;
    /// Freeze working data on the suspension edge, never on ordinary errors.
    fn freeze(self) -> Self::Frozen;
}

/// An option consumer held in the Session's rooted resumption queue.
///
/// `finish` borrows this immutable state while the generic read frame remains
/// rooted. Later suspension captures a fresh continuation exactly once. Pure
/// abandonment drops this data without running any pending guest work.
pub trait FrozenOptionReadContinuation<V>: FrameRootTrace + Send + Sync + 'static {
    /// Complete the builtin after one successful decode; errors bypass this method.
    fn finish(&self, session: &mut Session, value: V) -> Result<Expr, EvalError>;
}

// Private decoders seal the value/frame pairing. No evaluating raw getter is
// exported by this protocol.
trait OptionDecode: FrameRootTrace + Send + Sync + 'static {
    type Value;
    fn decode(&self, value: Expr) -> Self::Value;
}

#[derive(FrameRootTrace)]
pub(super) struct BoolOptionDecode;

impl OptionDecode for BoolOptionDecode {
    type Value = bool;
    fn decode(&self, value: Expr) -> bool {
        is_system_symbol_name(&value, "True")
    }
}

#[derive(FrameRootTrace, rum_derive::PureFrameAbandon)]
struct OptionReadSuspensionFrame<D, F> {
    decoder: D,
    next: F,
}

impl<D, F> SuspensionFrame for OptionReadSuspensionFrame<D, F>
where
    D: OptionDecode,
    F: FrozenOptionReadContinuation<D::Value>,
{
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let result = match outcome {
            Ok(value) => self.next.finish(session, self.decoder.decode(value)),
            Err(error) => Err(error),
        };
        FrameAction::complete(result)
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend_option_read<C: OptionReadContinuation<bool>>(
    session: &mut Session,
    error: EvalError,
    decoder: BoolOptionDecode,
    next: C,
) -> Result<Expr, EvalError> {
    match error {
        EvalError::InputRequired { request } => {
            crate::resume_or_suspend!(insert session, OptionReadSuspensionFrame {
                decoder,
                next: next.freeze(),
            });
            Err(EvalError::InputRequired { request })
        }
        error => Err(error),
    }
}
