use std::sync::Arc;

use rum_runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};

#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenStringSequence<T: FrameRootTrace>(Arc<[T]>);

impl<T: FrameRootTrace> FrozenStringSequence<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        Self(Arc::from(values))
    }

    pub(super) fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }
}

#[derive(FrameRootTrace)]
pub(super) struct FrozenStringValue<T: FrameRootTrace>(Arc<T>);

impl<T: FrameRootTrace> Clone for FrozenStringValue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: FrameRootTrace> FrozenStringValue<T> {
    pub(super) fn freeze(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub(super) fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentStringSequence<T: FrameRootTrace> {
    prefix: Arc<[T]>,
    tail: AccumulatorChain<PersistentStringSequenceNode<T>>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentStringSequenceNode<T: FrameRootTrace> {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentStringSequenceNode<T>>,
    value: T,
}

impl<T: FrameRootTrace> PersistentStringSequence<T> {
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
            tail: AccumulatorChain::new(Some(Arc::new(PersistentStringSequenceNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }
}

impl<T: Clone + FrameRootTrace> PersistentStringSequence<T> {
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

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentStringText {
    prefix: Arc<String>,
    tail: AccumulatorChain<PersistentStringTextNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentStringTextNode {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentStringTextNode>,
    fragment: String,
}

impl PersistentStringText {
    pub(super) fn freeze(value: String) -> Self {
        let len = value.len();
        Self {
            prefix: Arc::new(value),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    pub(super) fn appended(mut self, fragment: impl Into<String>) -> Self {
        let fragment = fragment.into();
        let len = self.len + fragment.len();
        self.tail = AccumulatorChain::new(Some(Arc::new(PersistentStringTextNode {
            previous: self.tail,
            fragment,
        })));
        self.len = len;
        self
    }

    pub(super) fn to_string(&self) -> String {
        let mut fragments = Vec::new();
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            fragments.push(current.fragment.as_str());
            node = current.previous.as_deref();
        }

        let mut value = String::with_capacity(self.len);
        value.push_str(self.prefix.as_str());
        for fragment in fragments.into_iter().rev() {
            value.push_str(fragment);
        }
        value
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FrozenStringSequence<rum_core::Expr>>();
    assert_traces::<FrozenStringValue<rum_core::Expr>>();
    assert_traces::<PersistentStringSequence<rum_core::Expr>>();
    assert_traces::<PersistentStringSequenceNode<rum_core::Expr>>();
    assert_traces::<PersistentStringText>();
    assert_traces::<PersistentStringTextNode>();
};
