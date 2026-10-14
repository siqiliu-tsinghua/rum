use std::marker::PhantomData;

use super::action::{Action, ErasedFinalizer};
use super::channel::{CodecKey, Slot, decode, encode};
use super::*;

/// Keeps a rejected input intact until the driver records a structural fault.
pub(in crate::runtime::suspension) struct DecodeFailure {
    pub expected: &'static str,
    pub actual: Slot,
}

pub(in crate::runtime::suspension) trait ErasedFrame:
    FrameRootTrace + FrameAbandon + Send + Sync
{
    /// Consumes input, leaving the resource-free Abort even when decoding fails.
    fn resume(&self, session: &mut Session, input: &mut Result<Slot, EvalError>) -> Action;
    fn output_boundary(&self) -> OutputBoundary;
    fn inner_frame(&self) -> &dyn FrameRootTrace;
}

#[derive(rum_derive::FrameRootTrace)]
struct TypedAdapter<F: TypedFrame> {
    inner: Arc<F>,
}

impl<F: TypedFrame> FrameAbandon for TypedAdapter<F> {
    fn abandon(self: Arc<Self>, context: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        self.inner.clone().abandon(context)
    }
}

impl<F: TypedFrame> ErasedFrame for TypedAdapter<F> {
    fn resume(&self, session: &mut Session, input: &mut Result<Slot, EvalError>) -> Action {
        let decoded = match std::mem::replace(input, Err(EvalError::Abort)) {
            Ok(slot) => match decode::<F::Input>(slot, CodecKey::new()) {
                Ok(value) => Ok(value),
                Err(actual) => {
                    return Action::DecodeFailure(DecodeFailure {
                        expected: std::any::type_name::<F::Input>(),
                        actual,
                    });
                }
            },
            Err(error) => Err(error),
        };
        // This is the sole token mint. It stores neither this adapter nor F.
        let token = ContinueToken { proof: PhantomData };
        self.inner.clone().resume(session, decoded, token).inner
    }

    fn output_boundary(&self) -> OutputBoundary {
        self.inner.output_boundary()
    }

    fn inner_frame(&self) -> &dyn FrameRootTrace {
        self.inner.as_ref()
    }
}

pub(in crate::runtime::suspension) fn erase<F: TypedFrame>(frame: F) -> Arc<dyn ErasedFrame> {
    Arc::new(TypedAdapter {
        inner: Arc::new(frame),
    })
}

pub(in crate::runtime::suspension) fn allocation_bytes<F: TypedFrame>() -> usize {
    4 * std::mem::size_of::<usize>()
        + std::mem::size_of::<TypedAdapter<F>>()
        + std::mem::size_of::<F>()
}

impl<F: SlotFinalizer> ErasedFinalizer for F {
    fn finish(self: Box<Self>) -> Result<Slot, EvalError> {
        SlotFinalizer::finish(self).map(|value| encode(value, CodecKey::new()))
    }
}
