use std::sync::Arc;

use rum_runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentLogicSequence<T: FrameRootTrace> {
    prefix: Arc<[T]>,
    tail: AccumulatorChain<PersistentLogicSequenceNode<T>>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentLogicSequenceNode<T: FrameRootTrace> {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentLogicSequenceNode<T>>,
    value: T,
}

impl<T: FrameRootTrace> PersistentLogicSequence<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        let len = values.len();
        Self {
            prefix: Arc::from(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    pub(super) fn appended(&self, value: T) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PersistentLogicSequenceNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }
}

impl<T: Clone + FrameRootTrace> PersistentLogicSequence<T> {
    pub(super) fn to_vec(&self) -> Vec<T> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            suffix.push(current.value.clone());
            node = current.previous.as_deref();
        }
        suffix.reverse();
        let mut values = Vec::with_capacity(self.len);
        values.extend(self.prefix.iter().cloned());
        values.extend(suffix);
        values
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<PersistentLogicSequence<rum_core::Expr>>();
    assert_traces::<PersistentLogicSequenceNode<rum_core::Expr>>();
};
