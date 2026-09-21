use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use std::sync::Arc;

/// A binding table captured only after matching has actually suspended.
///
/// Ordinary matching keeps using `SymbolExprMap` directly, so its lookup stays
/// one `HashMap` probe. Suspended continuations only need an immutable snapshot:
/// any bindings produced by the suspended matcher arrive in its owned outcome.
#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) struct FrozenMatchBindings(Arc<SymbolExprMap>);

impl FrozenMatchBindings {
    pub(in crate::runtime) fn freeze(bindings: SymbolExprMap) -> Self {
        Self(Arc::new(bindings))
    }

    pub(in crate::runtime) fn as_map(&self) -> &SymbolExprMap {
        self.0.as_ref()
    }

    pub(in crate::runtime) fn into_owned(self) -> SymbolExprMap {
        Arc::try_unwrap(self.0)
            .unwrap_or_else(|_| panic!("completed matcher bindings must be uniquely owned"))
    }
}

/// A matcher-owned bulk sequence moved behind one immutable handle only when
/// the operation suspends.
#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) struct FrozenMatcherSequence<T: FrameRootTrace>(Arc<Vec<T>>);

impl<T: FrameRootTrace> FrozenMatcherSequence<T> {
    pub(in crate::runtime) fn freeze(values: Vec<T>) -> Self {
        Self(Arc::new(values))
    }

    pub(in crate::runtime) fn into_vec(self) -> Vec<T> {
        Arc::try_unwrap(self.0)
            .unwrap_or_else(|_| panic!("completed matcher sequence must be uniquely owned"))
    }

    pub(in crate::runtime) fn as_slice(&self) -> &[T] {
        self.0.as_ref()
    }
}

/// A recursively owned matcher value moved behind an immutable handle at a
/// suspension boundary. This avoids cloning nested matcher nodes merely to
/// make continuation state shareable.
#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) struct FrozenMatcherValue<T: FrameRootTrace>(Arc<T>);

impl<T: FrameRootTrace> FrozenMatcherValue<T> {
    pub(in crate::runtime) fn freeze(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub(in crate::runtime) fn into_inner(self) -> T {
        Arc::try_unwrap(self.0)
            .unwrap_or_else(|_| panic!("completed matcher value must be uniquely owned"))
    }
}

/// Append-only binding-result storage for suspended all-match searches.
///
/// The prefix and every appended result batch are moved into immutable chunks.
/// Appending is O(1); the current closure implementation consumes the unique
/// chain once at completion. The named-frame cutover can move that consumption
/// into its Session-free finalizer without changing this representation.
#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) struct PersistentMatchResults {
    prefix: Arc<Vec<SymbolExprMap>>,
    tail: AccumulatorChain<PersistentMatchResultsNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentMatchResultsNode {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentMatchResultsNode>,
    values: Arc<Vec<SymbolExprMap>>,
}

impl PersistentMatchResults {
    pub(in crate::runtime) fn freeze(values: Vec<SymbolExprMap>) -> Self {
        let len = values.len();
        Self {
            prefix: Arc::new(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    pub(in crate::runtime) fn into_vec(self) -> Vec<SymbolExprMap> {
        let mut suffix = Vec::new();
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            let current = Arc::try_unwrap(current).unwrap_or_else(|_| {
                panic!("completed matcher result chain must be uniquely owned")
            });
            let values = Arc::try_unwrap(current.values).unwrap_or_else(|_| {
                panic!("completed matcher result chunk must be uniquely owned")
            });
            suffix.push(values);
            node = current.previous.into_inner();
        }
        suffix.reverse();

        let mut values = Arc::try_unwrap(self.prefix)
            .unwrap_or_else(|_| panic!("completed matcher result prefix must be uniquely owned"));
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
    assert_traces::<FrozenMatchBindings>();
    assert_traces::<FrozenMatcherSequence<Expr>>();
    assert_traces::<FrozenMatcherValue<Expr>>();
    assert_traces::<PersistentMatchResults>();
    assert_traces::<PersistentMatchResultsNode>();
};
