use std::marker::PhantomData;

use rum_core::Expr;

use super::adapter::{DecodeFailure, ErasedFrame, erase};
use super::channel::{CodecKey, Slot, encode};
use super::{Arc, EvalError, SlotFinalizer, SlotValue, TypedFrame};

pub(in crate::runtime::suspension) trait ErasedFinalizer:
    Send
{
    fn finish(self: Box<Self>) -> Result<Slot, EvalError>;
}

pub(in crate::runtime::suspension) enum Action {
    DecodeFailure(DecodeFailure),
    Complete(Result<Slot, EvalError>),
    Evaluate {
        expr: Expr,
        next: Arc<dyn ErasedFrame>,
    },
    Finalize(Box<dyn ErasedFinalizer>),
}

/// An opaque action whose completion always has type `O`.
pub struct TypedAction<O: SlotValue> {
    pub(super) inner: Action,
    output: PhantomData<fn(O) -> O>,
}

impl<O: SlotValue> TypedAction<O> {
    // Runtime unit tests only: deliberately violate the declared output mode.
    #[cfg(test)]
    pub(super) fn malformed_slot(slot: Slot) -> Self {
        Self {
            inner: Action::Complete(Ok(slot)),
            output: PhantomData,
        }
    }

    pub fn complete(result: Result<O, EvalError>) -> Self {
        Self {
            inner: Action::Complete(result.map(|value| encode(value, CodecKey::new()))),
            output: PhantomData,
        }
    }

    pub fn evaluate<F: TypedFrame<Input = Expr, Output = O>>(expr: Expr, next: F) -> Self {
        Self {
            inner: Action::Evaluate {
                expr,
                next: erase(next),
            },
            output: PhantomData,
        }
    }

    pub fn finalize<F: SlotFinalizer<Output = O>>(finalizer: F) -> Self {
        Self {
            inner: Action::Finalize(Box::new(finalizer)),
            output: PhantomData,
        }
    }

    pub fn finish_work<T, F: SlotFinalizer<Output = O>>(
        work: Result<T, ResumeStop<O>>,
        finish: impl FnOnce(T) -> F,
    ) -> Self {
        match work {
            Ok(value) => Self::finalize(finish(value)),
            Err(stop) => stop.action,
        }
    }
}

/// A stopped deferred helper, bound to its enclosing frame's output type.
pub struct ResumeStop<O: SlotValue> {
    action: TypedAction<O>,
}

impl<O: SlotValue> ResumeStop<O> {
    pub fn failure(error: EvalError) -> Self {
        Self {
            action: TypedAction::complete(Err(error)),
        }
    }
}
