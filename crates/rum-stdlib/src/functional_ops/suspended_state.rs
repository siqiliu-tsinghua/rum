use std::sync::Arc;

use rum_runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};

/// Immutable functional-family input storage created only after suspension.
#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenFunctionalSequence<T: FrameRootTrace>(Arc<[T]>);

impl<T: FrameRootTrace> FrozenFunctionalSequence<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        Self(Arc::from(values))
    }

    pub(super) fn as_slice(&self) -> &[T] {
        self.0.as_ref()
    }

    pub(super) fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
}

/// Append-only functional-family output/history storage for resumed work.
#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentFunctionalSequence<T: FrameRootTrace> {
    prefix: Arc<[T]>,
    tail: AccumulatorChain<PersistentFunctionalSequenceNode<T>>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentFunctionalSequenceNode<T: FrameRootTrace> {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentFunctionalSequenceNode<T>>,
    value: T,
}

impl<T: FrameRootTrace> PersistentFunctionalSequence<T> {
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
            tail: AccumulatorChain::new(Some(Arc::new(PersistentFunctionalSequenceNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn last(&self) -> Option<&T> {
        self.tail
            .as_deref()
            .map(|node| &node.value)
            .or_else(|| self.prefix.last())
    }

    pub(super) fn any(&self, mut predicate: impl FnMut(&T) -> bool) -> bool {
        if self.prefix.iter().any(&mut predicate) {
            return true;
        }
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            if predicate(&current.value) {
                return true;
            }
            node = current.previous.as_deref();
        }
        false
    }
}

impl<T: Clone + FrameRootTrace> PersistentFunctionalSequence<T> {
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

    pub(super) fn recent(&self, count: usize) -> Vec<T> {
        let count = count.min(self.len);
        let suffix_len = self.len.saturating_sub(self.prefix.len());
        let suffix_count = count.min(suffix_len);
        let prefix_count = count - suffix_count;

        let mut suffix = Vec::with_capacity(suffix_count);
        let mut node = self.tail.as_deref();
        while suffix.len() < suffix_count {
            let Some(current) = node else {
                break;
            };
            suffix.push(current.value.clone());
            node = current.previous.as_deref();
        }
        suffix.reverse();

        let mut values = Vec::with_capacity(count);
        values.extend(
            self.prefix[self.prefix.len().saturating_sub(prefix_count)..]
                .iter()
                .cloned(),
        );
        values.extend(suffix);
        values
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FrozenFunctionalSequence<rum_core::Expr>>();
    assert_traces::<PersistentFunctionalSequence<rum_core::Expr>>();
    assert_traces::<PersistentFunctionalSequenceNode<rum_core::Expr>>();
};
