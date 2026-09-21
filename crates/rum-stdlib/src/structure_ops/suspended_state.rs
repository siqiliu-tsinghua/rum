use std::sync::Arc;
use std::{collections::HashSet, hash::Hash};

use rum_runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};

const PERSISTENT_SEQUENCE_CHUNK_CAPACITY: usize = 32;

#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenStructureSequence<T: FrameRootTrace>(Arc<[T]>);

impl<T: FrameRootTrace> FrozenStructureSequence<T> {
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

#[derive(FrameRootTrace)]
pub(super) struct FrozenStructureValue<T: FrameRootTrace>(Arc<T>);

impl<T: FrameRootTrace> Clone for FrozenStructureValue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: FrameRootTrace> FrozenStructureValue<T> {
    pub(super) fn freeze(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub(super) fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentStructureSequence<T: FrameRootTrace> {
    prefix: Arc<[T]>,
    tail: AccumulatorChain<PersistentStructureSequenceNode<T>>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentStructureSequenceNode<T: FrameRootTrace> {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentStructureSequenceNode<T>>,
    values: Vec<T>,
}

impl<T: FrameRootTrace> PersistentStructureSequence<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        let len = values.len();
        Self {
            prefix: Arc::from(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    pub(super) fn push(&mut self, value: T) {
        let mut previous = self.tail.take().into_inner();
        if let Some(tail) = previous.as_mut().and_then(Arc::get_mut) {
            if tail.values.len() < PERSISTENT_SEQUENCE_CHUNK_CAPACITY {
                tail.values.push(value);
                self.tail = AccumulatorChain::new(previous);
                self.len += 1;
                return;
            }
        }

        let mut values = Vec::with_capacity(PERSISTENT_SEQUENCE_CHUNK_CAPACITY);
        values.push(value);
        self.tail = AccumulatorChain::new(Some(Arc::new(PersistentStructureSequenceNode {
            previous: AccumulatorChain::new(previous),
            values,
        })));
        self.len += 1;
    }

    pub(super) fn appended(mut self, value: T) -> Self {
        self.push(value);
        self
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn get(&self, index: usize) -> Option<&T> {
        if index < self.prefix.len() {
            return self.prefix.get(index);
        }
        if index >= self.len {
            return None;
        }
        let mut remaining = self.len - index - 1;
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            if remaining < current.values.len() {
                return current.values.get(current.values.len() - remaining - 1);
            }
            remaining -= current.values.len();
            node = current.previous.as_deref();
        }
        None
    }
}

impl<T: Clone + FrameRootTrace> PersistentStructureSequence<T> {
    pub(super) fn to_vec(&self) -> Vec<T> {
        let mut chunks = Vec::new();
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            chunks.push(current.values.as_slice());
            node = current.previous.as_deref();
        }

        let mut values = Vec::with_capacity(self.len);
        values.extend(self.prefix.iter().cloned());
        for chunk in chunks.into_iter().rev() {
            values.extend(chunk.iter().cloned());
        }
        values
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentStructureSet<T: FrameRootTrace> {
    base: Arc<HashSet<T>>,
    tail: AccumulatorChain<PersistentStructureSetNode<T>>,
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentStructureCounts {
    base: Arc<[(rum_core::Expr, usize)]>,
    tail: AccumulatorChain<PersistentStructureCountNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentStructureCountNode {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentStructureCountNode>,
    update: PersistentStructureCountUpdate,
}

#[derive(FrameRootTrace)]
enum PersistentStructureCountUpdate {
    Increment(usize),
    Append(rum_core::Expr),
}

impl PersistentStructureCounts {
    pub(super) fn freeze(values: Vec<(rum_core::Expr, usize)>) -> Self {
        let len = values.len();
        Self {
            base: Arc::from(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn item(&self, index: usize) -> Option<rum_core::Expr> {
        if let Some((item, _)) = self.base.get(index) {
            return Some(item.clone());
        }
        let mut appended = Vec::new();
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            if let PersistentStructureCountUpdate::Append(item) = &current.update {
                appended.push(item);
            }
            node = current.previous.as_deref();
        }
        let offset = index.checked_sub(self.base.len())?;
        appended
            .get(appended.len().checked_sub(offset + 1)?)
            .map(|item| (*item).clone())
    }

    pub(super) fn incremented(&self, index: usize) -> Self {
        Self {
            base: self.base.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PersistentStructureCountNode {
                previous: self.tail.clone(),
                update: PersistentStructureCountUpdate::Increment(index),
            }))),
            len: self.len,
        }
    }

    pub(super) fn appended(&self, item: rum_core::Expr) -> Self {
        Self {
            base: self.base.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PersistentStructureCountNode {
                previous: self.tail.clone(),
                update: PersistentStructureCountUpdate::Append(item),
            }))),
            len: self.len + 1,
        }
    }

    pub(super) fn to_vec(&self) -> Vec<(rum_core::Expr, usize)> {
        let mut updates = Vec::new();
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            updates.push(&current.update);
            node = current.previous.as_deref();
        }
        let mut values = self.base.iter().cloned().collect::<Vec<_>>();
        for update in updates.into_iter().rev() {
            match update {
                PersistentStructureCountUpdate::Increment(index) => values[*index].1 += 1,
                PersistentStructureCountUpdate::Append(item) => values.push((item.clone(), 1)),
            }
        }
        values
    }
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentStructureSetNode<T: FrameRootTrace> {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentStructureSetNode<T>>,
    value: T,
}

impl<T: Clone + Eq + Hash + FrameRootTrace> PersistentStructureSet<T> {
    pub(super) fn freeze(values: HashSet<T>) -> Self {
        Self {
            base: Arc::new(values),
            tail: AccumulatorChain::default(),
        }
    }

    pub(super) fn inserted(&self, value: T) -> (Self, bool) {
        if self.base.contains(&value) {
            return (self.clone(), false);
        }
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            if current.value == value {
                return (self.clone(), false);
            }
            node = current.previous.as_deref();
        }
        (
            Self {
                base: self.base.clone(),
                tail: AccumulatorChain::new(Some(Arc::new(PersistentStructureSetNode {
                    previous: self.tail.clone(),
                    value,
                }))),
            },
            true,
        )
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FrozenStructureSequence<rum_core::Expr>>();
    assert_traces::<FrozenStructureValue<rum_core::Expr>>();
    assert_traces::<PersistentStructureSequence<rum_core::Expr>>();
    assert_traces::<PersistentStructureSequenceNode<rum_core::Expr>>();
    assert_traces::<PersistentStructureSet<rum_core::Expr>>();
    assert_traces::<PersistentStructureSetNode<rum_core::Expr>>();
    assert_traces::<PersistentStructureCounts>();
    assert_traces::<PersistentStructureCountNode>();
    assert_traces::<PersistentStructureCountUpdate>();
};
