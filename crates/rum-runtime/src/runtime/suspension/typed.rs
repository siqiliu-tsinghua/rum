//! Typed continuation construction and private driver entries.

use std::sync::Arc;

use super::trace::{FrameRootTrace, FrameRootVisitor};
use super::{AbandonmentContext, FrameAbandon, FrameAbandonAction, OutputBoundary};
use crate::runtime::{EvalError, Session};

mod action;
mod adapter;
mod chain;
mod channel;
mod entry;
mod legacy;
mod registry;
mod resume_token;
mod witness;

#[cfg(test)]
mod tests;

pub use action::{ResumeStop, TypedAction};
pub use chain::FrameChain;
pub use resume_token::ContinueToken;
pub use witness::{SlotWitness, slot_witness};

pub(super) use action::Action;
pub(super) use adapter::{
    DecodeFailure, ErasedFrame, allocation_bytes as typed_allocation_bytes, erase,
};
pub(super) use channel::{Slot, decode_completion};
pub(super) use entry::{Entry, EntryRef};

/// A runtime-registered continuation payload. External implementations are sealed.
pub trait SlotValue: channel::Codec + Sized + Send + 'static {}

/// A continuation whose input and output have fixed slot types.
pub trait TypedFrame: FrameRootTrace + FrameAbandon + Send + Sync + Sized + 'static {
    type Input: SlotValue;
    type Output: SlotValue;

    fn resume<'resume>(
        self: Arc<Self>,
        session: &mut Session,
        input: Result<Self::Input, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Self::Output>;

    /// Buffered output owned until this frame's capture settles. Metadata only.
    fn output_boundary(&self) -> OutputBoundary {
        OutputBoundary::default()
    }
}

/// A Session-free completion producing the enclosing continuation's output type.
pub trait SlotFinalizer: Send + 'static {
    type Output: SlotValue;
    fn finish(self: Box<Self>) -> Result<Self::Output, EvalError>;
}

// Private sibling tests enter through the real adapter, which alone mints the token.
#[cfg(test)]
pub(in crate::runtime) fn resume_typed_completion_for_test<F: TypedFrame>(
    frame: F,
    session: &mut Session,
    outcome: Result<F::Input, EvalError>,
) -> Result<F::Output, EvalError> {
    let mut input = outcome.map(|value| channel::encode(value, channel::CodecKey::new()));
    match erase(frame).resume(session, &mut input) {
        Action::Complete(outcome) => {
            decode_completion(outcome).unwrap_or_else(|_| panic!("test completion slot mismatch"))
        }
        _ => panic!("test expected a typed completion"),
    }
}
