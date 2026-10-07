//! Move-or-clone completion while older snapshots may still own the spine.
use super::{AccumulatorChain, AccumulatorChainNode};
use std::sync::Arc;

pub(in crate::runtime) struct Completion<N: AccumulatorChainNode> {
    remaining: AccumulatorChain<N>,
}

impl<N: AccumulatorChainNode + Clone> AccumulatorChain<N> {
    /// Consumes nodes newest first, with each returned node's link detached.
    ///
    /// A shared node is shallow-cloned; retained snapshots are left intact.
    /// The cursor keeps its remaining spine wrapper-owned, including when
    /// completion stops early. No active suspension owner needs to retire first.
    #[inline]
    pub(in crate::runtime) fn into_completion(self) -> Completion<N> {
        Completion { remaining: self }
    }
}

impl<N: AccumulatorChainNode + Clone> Iterator for Completion<N> {
    type Item = N;

    #[inline]
    fn next(&mut self) -> Option<N> {
        let shared = self.remaining.0.take()?;
        let mut node = match Arc::try_unwrap(shared) {
            Ok(node) => node,
            Err(shared) => shared.as_ref().clone(),
        };
        self.remaining.0 = node.take_accumulator_link();
        Some(node)
    }
}
