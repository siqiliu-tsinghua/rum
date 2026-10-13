//! Typed continuation construction. S1 deliberately has no queue integration.
// Private erased operations become driver entry points in S2.
#![allow(dead_code)]

use std::sync::Arc;

use super::trace::{FrameRootTrace, FrameRootVisitor};
use super::{AbandonmentContext, FrameAbandon, FrameAbandonAction, OutputBoundary};
use crate::runtime::{EvalError, Session};

mod action;
mod adapter;
mod chain;
mod channel;
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
