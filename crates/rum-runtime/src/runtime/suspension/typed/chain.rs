use std::marker::PhantomData;

use super::adapter::{ErasedFrame, erase};
use super::{Arc, EvalError, Session, SlotValue, TypedFrame};
use crate::runtime::RuntimeInputRequest;

/// A witnessed, nonempty sequence with invariant input and output types.
pub struct FrameChain<I: SlotValue, O: SlotValue> {
    frames: Vec<Arc<dyn ErasedFrame>>,
    ends: PhantomData<fn(I, O) -> (I, O)>,
}

pub(super) fn start<I: SlotValue, F: TypedFrame<Input = I>>(first: F) -> FrameChain<I, F::Output> {
    FrameChain {
        frames: vec![erase(first)],
        ends: PhantomData,
    }
}

impl<I: SlotValue, O: SlotValue> FrameChain<I, O> {
    pub fn suspend(
        self,
        session: &mut Session,
        request: RuntimeInputRequest,
    ) -> Result<O, EvalError> {
        for frame in self.frames {
            session.insert_typed_erased_frame(frame);
        }
        Err(EvalError::InputRequired { request })
    }

    pub fn then<F: TypedFrame<Input = O>>(mut self, next: F) -> FrameChain<I, F::Output> {
        self.frames.push(erase(next));
        FrameChain {
            frames: self.frames,
            ends: PhantomData,
        }
    }
}
