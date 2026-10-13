use std::marker::PhantomData;

use super::{EvalError, FrameChain, SlotValue, TypedFrame, chain};

/// Zero-sized, invariant evidence of the input type of an actual result.
pub struct SlotWitness<I: SlotValue> {
    proof: PhantomData<fn(I) -> I>,
}

pub fn slot_witness<I: SlotValue>(_: &Result<I, EvalError>) -> SlotWitness<I> {
    SlotWitness { proof: PhantomData }
}

impl<I: SlotValue> SlotWitness<I> {
    pub fn chain<F: TypedFrame<Input = I>>(self, first: F) -> FrameChain<I, F::Output> {
        chain::start(first)
    }
}
