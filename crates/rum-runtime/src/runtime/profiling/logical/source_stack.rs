//! Persistent diagnostic scopes: snapshots share nodes and never copy a prefix.
use super::{Storage, epoch};
use crate::runtime::accumulator_chain::{AccumulatorChain, AccumulatorChainNode};
use std::{fmt, mem::size_of, sync::Arc};

#[derive(rum_derive::AccumulatorChainNode)]
struct Node<T> {
    value: T,
    #[accumulator_link]
    parent: AccumulatorChain<Self>,
    depth: usize,
    _storage: Storage,
}
pub(in crate::runtime) struct SourceStack<T> {
    tail: Option<Arc<Node<T>>>,
    // A failed diagnostic allocation still balances its caller's push/pop.
    // The epoch is invalid; no missing scope is attributed to a live parent.
    missing: usize,
}
impl<T> SourceStack<T> {
    pub(in crate::runtime) const fn new() -> Self {
        Self {
            tail: None,
            missing: 0,
        }
    }
    pub(in crate::runtime) fn len(&self) -> usize {
        self.tail
            .as_ref()
            .map_or(0, |n| n.depth)
            .saturating_add(self.missing)
    }
    pub(in crate::runtime) fn last(&self) -> Option<&T> {
        if self.missing != 0 {
            None
        } else {
            self.tail.as_ref().map(|n| &n.value)
        }
    }
    pub(in crate::runtime) fn clear(&mut self) {
        *self = Self::new();
    }
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn push(&mut self, value: T) {
        if self.missing != 0 {
            self.missing = self.missing.saturating_add(1);
            return;
        }
        let storage =
            epoch::current().and_then(|e| e.storage(size_of::<Node<T>>() + 2 * size_of::<usize>()));
        let Some(storage) = storage else {
            self.missing = 1;
            return;
        };
        let depth = self.len() + 1;
        self.tail = Some(Arc::new(Node {
            value,
            parent: AccumulatorChain::new(self.tail.take()),
            depth,
            _storage: storage,
        }));
    }
}
impl<T: Copy> SourceStack<T> {
    // Keep the enabled release work out of the inert source guards' drops.
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn pop(&mut self) -> Option<T> {
        if self.missing != 0 {
            self.missing -= 1;
            return None;
        }
        let tail = self.tail.take()?;
        self.tail = tail.parent.clone().into_inner();
        Some(tail.value)
    }
}
impl<T> Default for SourceStack<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Clone for SourceStack<T> {
    fn clone(&self) -> Self {
        Self {
            tail: self.tail.clone(),
            missing: self.missing,
        }
    }
}
impl<T: fmt::Debug> fmt::Debug for SourceStack<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.missing != 0 {
            return f.write_str("<invalid diagnostic scope>");
        }
        if self.tail.is_none() {
            return f.debug_list().finish();
        }
        let storage = epoch::current().and_then(|e| e.storage(self.len() * size_of::<&T>()));
        let Some(_storage) = storage else {
            return f.write_str("<invalid diagnostic scope>");
        };
        let mut values = Vec::with_capacity(self.len());
        let mut tail = self.tail.as_deref();
        while let Some(node) = tail {
            values.push(&node.value);
            tail = node.parent.as_deref();
        }
        f.debug_list().entries(values.into_iter().rev()).finish()
    }
}
