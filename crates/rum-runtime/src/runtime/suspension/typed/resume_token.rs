use std::marker::PhantomData;

use super::{EvalError, ResumeStop, Session, TypedFrame};
use crate::runtime::RuntimeInputRequest;

/// Adapter-minted evidence for one universally quantified resume call.
///
/// The token is invariant, has no Clone/Copy implementation, and owns no Arc.
pub struct ContinueToken<'resume, F: TypedFrame> {
    pub(super) proof: PhantomData<fn(&'resume mut (), F) -> (&'resume mut (), F)>,
}

impl<F: TypedFrame> ContinueToken<'_, F> {
    pub fn suspend(
        self,
        session: &mut Session,
        request: RuntimeInputRequest,
        fresh: F,
    ) -> ResumeStop<F::Output> {
        session.insert_typed_suspension_frame(fresh);
        ResumeStop::failure(EvalError::InputRequired { request })
    }
}
