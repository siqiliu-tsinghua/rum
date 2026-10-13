use std::marker::PhantomData;

use super::TypedFrame;

/// Adapter-minted evidence for one universally quantified resume call.
///
/// The token is invariant, has no Clone/Copy implementation, and owns no Arc.
/// Checked suspension is added with the driver; S1 exposes no suspension stub.
pub struct ContinueToken<'resume, F: TypedFrame> {
    pub(super) proof: PhantomData<fn(&'resume mut (), F) -> (&'resume mut (), F)>,
}
