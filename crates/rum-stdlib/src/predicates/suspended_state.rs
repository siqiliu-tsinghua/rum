use std::sync::Arc;

use rum_runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};

#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenPredicateSequence<T: FrameRootTrace>(Arc<[T]>);

impl<T: FrameRootTrace> FrozenPredicateSequence<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        Self(Arc::from(values))
    }

    pub(super) fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentPredicateMatches {
    base: Arc<[Vec<bool>]>,
    tail: AccumulatorChain<PersistentPredicateMatchNode>,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentPredicateMatchNode {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentPredicateMatchNode>,
    lhs_index: usize,
    rhs_index: usize,
}

impl PersistentPredicateMatches {
    pub(super) fn freeze(values: Vec<Vec<bool>>) -> Self {
        Self {
            base: Arc::from(values),
            tail: AccumulatorChain::default(),
        }
    }

    pub(super) fn matched(&self, lhs_index: usize, rhs_index: usize) -> Self {
        Self {
            base: self.base.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PersistentPredicateMatchNode {
                previous: self.tail.clone(),
                lhs_index,
                rhs_index,
            }))),
        }
    }

    pub(super) fn to_vec(&self) -> Vec<Vec<bool>> {
        let mut updates = Vec::new();
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            updates.push((current.lhs_index, current.rhs_index));
            node = current.previous.as_deref();
        }
        let mut values = self.base.iter().cloned().collect::<Vec<_>>();
        for (lhs_index, rhs_index) in updates.into_iter().rev() {
            values[lhs_index][rhs_index] = true;
        }
        values
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FrozenPredicateSequence<rum_core::Expr>>();
    assert_traces::<PersistentPredicateMatches>();
    assert_traces::<PersistentPredicateMatchNode>();
};
