use std::marker::PhantomData;

use super::action::{Action, ErasedFinalizer};
use super::channel::{CodecKey, Slot, decode, encode};
use super::*;

/// Keeps a rejected input intact; the live driver's fault policy belongs to S2.
pub(super) struct DecodeFailure {
    pub expected: &'static str,
    pub actual: Slot,
}

pub(super) trait ErasedFrame: FrameRootTrace + FrameAbandon + Send + Sync {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        input: Result<Slot, EvalError>,
    ) -> Result<Action, DecodeFailure>;
    fn output_boundary(&self) -> OutputBoundary;
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
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        input: Result<Slot, EvalError>,
    ) -> Result<Action, DecodeFailure> {
        let decoded = match input {
            Ok(slot) => match decode::<F::Input>(slot, CodecKey::new()) {
                Ok(value) => Ok(value),
                Err(actual) => {
                    return Err(DecodeFailure {
                        expected: std::any::type_name::<F::Input>(),
                        actual,
                    });
                }
            },
            Err(error) => Err(error),
        };
        // This is the sole token mint. It stores neither this adapter nor F.
        let token = ContinueToken { proof: PhantomData };
        Ok(self.inner.clone().resume(session, decoded, token).inner)
    }

    fn output_boundary(&self) -> OutputBoundary {
        self.inner.output_boundary()
    }
}

pub(super) fn erase<F: TypedFrame>(frame: F) -> Arc<dyn ErasedFrame> {
    Arc::new(TypedAdapter {
        inner: Arc::new(frame),
    })
}

pub(super) struct FinalizerAdapter<F: SlotFinalizer>(pub F);

impl<F: SlotFinalizer> ErasedFinalizer for FinalizerAdapter<F> {
    fn finish(self: Box<Self>) -> Result<Slot, EvalError> {
        Box::new(self.0)
            .finish()
            .map(|value| encode(value, CodecKey::new()))
    }
}
