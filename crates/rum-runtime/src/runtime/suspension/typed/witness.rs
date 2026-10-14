use std::marker::PhantomData;

use super::{EvalError, FrameChain, Session, SlotValue, TypedFrame, chain};
use crate::runtime::RuntimeInputRequest;

/// Zero-sized, invariant evidence of the input type of an actual result.
pub struct SlotWitness<I: SlotValue> {
    proof: PhantomData<fn(I) -> I>,
}

pub fn slot_witness<I: SlotValue>(_: &Result<I, EvalError>) -> SlotWitness<I> {
    SlotWitness { proof: PhantomData }
}

impl<I: SlotValue> SlotWitness<I> {
    pub fn suspend<F: TypedFrame<Input = I>>(
        self,
        session: &mut Session,
        request: RuntimeInputRequest,
        frame: F,
    ) -> Result<F::Output, EvalError> {
        session.insert_typed_suspension_frame(frame);
        Err(EvalError::InputRequired { request })
    }

    pub fn chain<F: TypedFrame<Input = I>>(self, first: F) -> FrameChain<I, F::Output> {
        chain::start(first)
    }
}

/// Captures the actual producer once and constructs a frame only on suspension.
#[macro_export]
macro_rules! resume_typed_or_suspend {
    (
        $session:expr, $result:expr,
        ok($value:pat) => $continue:expr,
        suspend => $frame:expr $(,)?
    ) => {{
        let produced = $result;
        let witness = $crate::kernel_api::slot_witness(&produced);
        match produced {
            Ok($value) => $continue,
            Err($crate::kernel_api::EvalError::InputRequired { request }) => {
                return witness.suspend($session, request, $frame);
            }
            Err(error) => Err(error),
        }
    }};
}
