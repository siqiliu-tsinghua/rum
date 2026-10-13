use std::marker::PhantomData;

use super::adapter::{ErasedFrame, erase};
use super::{Arc, SlotValue, TypedFrame};

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
    pub fn then<F: TypedFrame<Input = O>>(mut self, next: F) -> FrameChain<I, F::Output> {
        self.frames.push(erase(next));
        FrameChain {
            frames: self.frames,
            ends: PhantomData,
        }
    }
}
