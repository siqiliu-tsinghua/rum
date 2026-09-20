//! Constant-space destruction of persistent accumulator links.

use std::sync::Arc;

use super::suspension::trace::{FrameRootTrace, FrameRootVisitor};

/// Access to the single owning link of an accumulator node.
///
/// Derive this trait with exactly one `#[accumulator_link]` field of type
/// `AccumulatorChain<Self>`. Taking the link must empty that field without
/// cloning or manufacturing a link. Nodes remain free of a production `Drop`
/// implementation so their payload fields can still be moved after acquisition.
pub trait AccumulatorChainNode: Sized {
    fn take_accumulator_link(&mut self) -> Option<Arc<Self>>;
}

/// An optional owning link whose exclusive suffix is destroyed iteratively.
///
/// The private field and node bound keep destruction attached to every link.
/// This bounds link destruction only; it makes no promise about payloads and
/// does not certify suspension frames as having bounded destruction.
#[repr(transparent)]
pub struct AccumulatorChain<N: AccumulatorChainNode>(Option<Arc<N>>);

impl<N: AccumulatorChainNode> AccumulatorChain<N> {
    #[inline]
    pub fn new(link: Option<Arc<N>>) -> Self {
        Self(link)
    }

    #[inline]
    pub fn as_deref(&self) -> Option<&N> {
        self.0.as_deref()
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Detaches the link, leaving this wrapper empty.
    #[inline]
    pub fn take(&mut self) -> Self {
        Self(self.0.take())
    }

    /// Consumes the wrapper without releasing the extracted link.
    #[inline]
    pub fn into_inner(mut self) -> Option<Arc<N>> {
        self.0.take()
    }
}

impl<N: AccumulatorChainNode> Default for AccumulatorChain<N> {
    #[inline]
    fn default() -> Self {
        Self(None)
    }
}

impl<N: AccumulatorChainNode> Clone for AccumulatorChain<N> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<N: AccumulatorChainNode + FrameRootTrace> FrameRootTrace for AccumulatorChain<N> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.0.trace_roots(visitor);
    }
}

impl<N: AccumulatorChainNode> Drop for AccumulatorChain<N> {
    fn drop(&mut self) {
        drain(self.0.take());
    }
}

/// Acquires and releases one detached node at a time, with O(1) auxiliary space.
///
/// Immediate release is sound because no production payload in this accumulator
/// family has an observable destructor: payload destruction is pure memory
/// reclamation. Adding a payload with an observable destructor requires revisiting
/// this policy; recursive payload release order is deliberately not preserved.
/// A shared node ends this drain, leaving its surviving strong holder responsible
/// for the tail. Weak handles do not prevent acquisition.
#[cold]
#[inline(never)]
fn drain<N: AccumulatorChainNode>(mut link: Option<Arc<N>>) {
    while let Some(shared) = link {
        let Some(mut node) = Arc::into_inner(shared) else {
            break;
        };
        link = node.take_accumulator_link();
        drop(node);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::*;

    #[derive(Default)]
    struct Observations {
        active: Cell<usize>,
        maximum: Cell<usize>,
        nodes: RefCell<Vec<usize>>,
        payloads: RefCell<Vec<usize>>,
    }

    struct Payload {
        id: usize,
        observations: Rc<Observations>,
        remaining: AccumulatorChain<Node>,
    }

    impl Drop for Payload {
        fn drop(&mut self) {
            let observations = &self.observations;
            let depth = observations.active.get() + 1;
            observations.active.set(depth);
            observations
                .maximum
                .set(observations.maximum.get().max(depth));
            observations.payloads.borrow_mut().push(self.id);
            drop(self.remaining.take());
            observations.active.set(depth - 1);
        }
    }

    #[derive(rum_derive::AccumulatorChainNode)]
    struct Node {
        #[accumulator_link]
        previous: AccumulatorChain<Self>,
        payload: Payload,
    }

    impl Drop for Node {
        fn drop(&mut self) {
            self.payload
                .observations
                .nodes
                .borrow_mut()
                .push(self.payload.id);
            // Test-only instrumentation: release any still-attached tail inside
            // Payload::drop, so recursive release nests the payload observers.
            // The real drain has already detached this link before dropping us.
            self.payload.remaining = self.previous.take();
        }
    }

    fn prefix(
        observations: &Rc<Observations>,
        ids: std::ops::Range<usize>,
        mut tail: AccumulatorChain<Node>,
    ) -> AccumulatorChain<Node> {
        for id in ids {
            tail = AccumulatorChain::new(Some(Arc::new(Node {
                previous: tail,
                payload: Payload {
                    id,
                    observations: observations.clone(),
                    remaining: AccumulatorChain::default(),
                },
            })));
        }
        tail
    }

    fn assert_released(observations: &Observations, expected: std::ops::Range<usize>) {
        let expected: Vec<_> = expected.collect();
        let mut nodes = observations.nodes.borrow().clone();
        let mut payloads = observations.payloads.borrow().clone();
        nodes.sort_unstable();
        payloads.sort_unstable();
        assert_eq!(nodes, expected, "each node must be released exactly once");
        assert_eq!(
            payloads, expected,
            "each payload must be released exactly once"
        );
        assert_eq!(observations.active.get(), 0);
    }

    fn exclusive_chain(length: usize) {
        let observations = Rc::new(Observations::default());
        drop(prefix(
            &observations,
            0..length,
            AccumulatorChain::default(),
        ));
        assert_released(&observations, 0..length);
        eprintln!(
            "exclusive length={length}: maximum payload drop depth={}",
            observations.maximum.get()
        );
        assert_eq!(observations.maximum.get(), 1);
    }

    #[test]
    fn exclusive_depth_16() {
        exclusive_chain(16);
    }

    #[test]
    fn exclusive_depth_64() {
        exclusive_chain(64);
    }

    #[test]
    fn shared_tail_stops_then_drains() {
        let observations = Rc::new(Observations::default());
        let tail = prefix(&observations, 0..16, AccumulatorChain::default());
        let first = prefix(&observations, 16..24, tail.clone());
        let second = prefix(&observations, 24..32, tail);
        drop(first);
        assert_released(&observations, 16..24);
        eprintln!(
            "shared tail first prefix: 8 nodes/payloads, maximum payload drop depth={}",
            observations.maximum.get()
        );
        assert_eq!(observations.maximum.get(), 1);
        observations.maximum.set(0);
        drop(second);
        assert_released(&observations, 0..32);
        eprintln!(
            "shared tail second prefix + tail: 24 nodes/payloads, maximum payload drop depth={}",
            observations.maximum.get()
        );
        assert_eq!(observations.maximum.get(), 1);
    }

    #[test]
    fn live_weak_does_not_prevent_drain() {
        let observations = Rc::new(Observations::default());
        let tail = prefix(&observations, 0..16, AccumulatorChain::default());
        let weak = Arc::downgrade(tail.0.as_ref().unwrap());
        drop(tail);
        assert_released(&observations, 0..16);
        assert!(weak.upgrade().is_none());
        eprintln!(
            "live Weak length=16: maximum payload drop depth={}",
            observations.maximum.get()
        );
        assert_eq!(observations.maximum.get(), 1);
    }

    #[test]
    fn wrapper_is_transparent_and_extraction_preserves_ownership() {
        assert_eq!(
            size_of::<AccumulatorChain<Node>>(),
            size_of::<Option<Arc<Node>>>()
        );
        assert_eq!(
            align_of::<AccumulatorChain<Node>>(),
            align_of::<Option<Arc<Node>>>()
        );
        let observations = Rc::new(Observations::default());
        let mut chain = prefix(&observations, 0..1, AccumulatorChain::default());
        let extracted = chain.take().into_inner();
        assert!(chain.is_none());
        assert_released(&observations, 0..0);
        drop(AccumulatorChain::new(extracted));
        assert_released(&observations, 0..1);
    }
}
