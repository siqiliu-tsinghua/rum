use std::sync::Arc;

use rum_runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};

use super::{Expr, grouping::IndexedGroups};

/// Immutable association-family input storage created only after suspension.
#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenAssociationSequence<T: FrameRootTrace>(Arc<[T]>);

impl<T: FrameRootTrace> FrozenAssociationSequence<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        Self(Arc::from(values))
    }

    pub(super) fn as_slice(&self) -> &[T] {
        self.0.as_ref()
    }

    pub(super) fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }
}

#[derive(FrameRootTrace)]
pub(super) struct FrozenAssociationValue<T: FrameRootTrace>(Arc<T>);

impl<T: FrameRootTrace> Clone for FrozenAssociationValue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: FrameRootTrace> FrozenAssociationValue<T> {
    pub(super) fn freeze(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub(super) fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

/// Append-only association-family output storage for resumed work.
///
/// The ordinary path retains its mutable `Vec`. The first real suspension
/// moves that prefix here; every resumed append is O(1), and completion reads
/// through the chain to build the result without relying on Arc uniqueness.
#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentAssociationSequence<T: FrameRootTrace> {
    prefix: Arc<[T]>,
    tail: AccumulatorChain<PersistentAssociationSequenceNode<T>>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentAssociationSequenceNode<T: FrameRootTrace> {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentAssociationSequenceNode<T>>,
    value: T,
}

impl<T: FrameRootTrace> PersistentAssociationSequence<T> {
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
            tail: AccumulatorChain::new(Some(Arc::new(PersistentAssociationSequenceNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }
}

impl<T: Clone + FrameRootTrace> PersistentAssociationSequence<T> {
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

/// A frozen grouping prefix plus an append-only log of resumed `(key, value)`
/// contributions. Resumed updates do not mutate or clone the indexed base;
/// insertion-order grouping is rebuilt once, by borrowing, at completion.
#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentAssociationGroups<T: FrameRootTrace> {
    base: Arc<[(Expr, Vec<T>)]>,
    tail: AccumulatorChain<PersistentAssociationGroupNode<T>>,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentAssociationGroupNode<T: FrameRootTrace> {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentAssociationGroupNode<T>>,
    key: Expr,
    value: T,
}

impl<T: FrameRootTrace> PersistentAssociationGroups<T> {
    pub(super) fn freeze(groups: IndexedGroups<T>) -> Self {
        Self {
            base: Arc::from(groups.into_entries()),
            tail: AccumulatorChain::default(),
        }
    }

    pub(super) fn appended(&self, key: Expr, value: T) -> Self {
        Self {
            base: self.base.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PersistentAssociationGroupNode {
                previous: self.tail.clone(),
                key,
                value,
            }))),
        }
    }
}

impl<T: Clone + FrameRootTrace> PersistentAssociationGroups<T> {
    pub(super) fn to_entries(&self) -> Vec<(Expr, Vec<T>)> {
        let mut updates = Vec::new();
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            updates.push((current.key.clone(), current.value.clone()));
            node = current.previous.as_deref();
        }
        updates.reverse();

        let mut groups = IndexedGroups::from_entries(self.base.iter().cloned().collect());
        for (key, value) in updates {
            groups.push(key, value);
        }
        groups.into_entries()
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FrozenAssociationSequence<Expr>>();
    assert_traces::<FrozenAssociationValue<Expr>>();
    assert_traces::<PersistentAssociationSequence<Expr>>();
    assert_traces::<PersistentAssociationSequenceNode<Expr>>();
    assert_traces::<PersistentAssociationGroups<Expr>>();
    assert_traces::<PersistentAssociationGroupNode<Expr>>();
};
