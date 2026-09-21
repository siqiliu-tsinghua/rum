use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use std::sync::Arc;

/// Immutable queue storage created only after a rewrite traversal suspends.
/// The cursor lives in the family-specific state, so retrying never rebuilds
/// or copies the queue.
#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenRewriteQueue<T: FrameRootTrace>(Arc<[T]>);

impl<T: FrameRootTrace> FrozenRewriteQueue<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        Self(Arc::from(values))
    }

    pub(super) fn as_slice(&self) -> &[T] {
        self.0.as_ref()
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenRewriteValue<T: FrameRootTrace>(Arc<T>);

impl<T: FrameRootTrace> FrozenRewriteValue<T> {
    pub(super) fn freeze(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub(super) fn as_ref(&self) -> &T {
        self.0.as_ref()
    }

    pub(super) fn into_inner(self) -> T {
        Arc::try_unwrap(self.0)
            .unwrap_or_else(|_| panic!("completed rewrite value must be uniquely owned"))
    }
}

/// A move-only persistent queue for suspended paths that must consume owned
/// elements (notably binding maps) without cloning them out of an Arc slice.
#[derive(FrameRootTrace)]
pub(super) struct PersistentRewriteQueue<T: FrameRootTrace> {
    head: AccumulatorChain<PersistentRewriteQueueNode<T>>,
    len: usize,
}

impl<T: FrameRootTrace> Clone for PersistentRewriteQueue<T> {
    fn clone(&self) -> Self {
        Self {
            head: self.head.clone(),
            len: self.len,
        }
    }
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentRewriteQueueNode<T: FrameRootTrace> {
    value: T,
    #[accumulator_link]
    next: AccumulatorChain<PersistentRewriteQueueNode<T>>,
}

impl<T: FrameRootTrace> PersistentRewriteQueue<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        let len = values.len();
        let mut head = AccumulatorChain::default();
        for value in values.into_iter().rev() {
            head = AccumulatorChain::new(Some(Arc::new(PersistentRewriteQueueNode {
                value,
                next: head,
            })));
        }
        Self { head, len }
    }

    pub(super) fn pop_front(self) -> Option<(T, Self)> {
        let head = self.head.into_inner()?;
        let node = Arc::try_unwrap(head)
            .unwrap_or_else(|_| panic!("consumed rewrite queue must be uniquely owned"));
        Some((
            node.value,
            Self {
                head: node.next,
                len: self.len - 1,
            },
        ))
    }

    pub(super) fn into_vec(mut self) -> Vec<T> {
        let mut values = Vec::with_capacity(self.len);
        while let Some((value, remaining)) = self.pop_front() {
            values.push(value);
            self = remaining;
        }
        values
    }
}

/// Append-only output storage for suspended rewrite walks. The ordinary path
/// keeps its mutable Vec; only the first suspension freezes that prefix.
#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentRewriteExprs {
    prefix: SharedArgs,
    tail: AccumulatorChain<PersistentRewriteExprNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentRewriteExprNode {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentRewriteExprNode>,
    values: SharedArgs,
}

impl PersistentRewriteExprs {
    pub(super) fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    pub(super) fn appended(&self, value: Expr) -> Self {
        self.appended_values(vec![value])
    }

    pub(super) fn appended_values(&self, values: Vec<Expr>) -> Self {
        if values.is_empty() {
            return self.clone();
        }
        let added = values.len();
        Self {
            prefix: self.prefix.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PersistentRewriteExprNode {
                previous: self.tail.clone(),
                values: SharedArgs::new(values),
            }))),
            len: self.len + added,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::new();
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            let current = Arc::try_unwrap(current)
                .unwrap_or_else(|_| panic!("completed rewrite output must be uniquely owned"));
            suffix.push(current.values.into_vec());
            node = current.previous.into_inner();
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(self.len.saturating_sub(values.len()));
        for chunk in suffix {
            values.extend(chunk);
        }
        values
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FrozenRewriteQueue<Expr>>();
    assert_traces::<FrozenRewriteValue<Expr>>();
    assert_traces::<PersistentRewriteQueue<Expr>>();
    assert_traces::<PersistentRewriteQueueNode<Expr>>();
    assert_traces::<PersistentRewriteExprs>();
    assert_traces::<PersistentRewriteExprNode>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abandoning_shared_rewrite_queue_preserves_remaining_binding_order() {
        let key = Symbol::from_name("x");
        let expected: Vec<_> = (0..3)
            .map(|value| {
                let mut bindings = SymbolExprMap::default();
                bindings.insert(key, Expr::integer(value.into()));
                bindings
            })
            .collect();
        let queue = PersistentRewriteQueue::freeze(expected.clone());
        let (first, remaining) = queue.pop_front().unwrap();
        assert_eq!(first, expected[0]);
        let survivor = remaining.clone();
        drop(remaining);
        assert_eq!(survivor.into_vec(), expected[1..]);
    }
}
