use super::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Default)]
struct Observations {
    clones: Cell<usize>,
    active: Cell<usize>,
    maximum: Cell<usize>,
    released: RefCell<Vec<usize>>,
}

struct Payload {
    id: usize,
    observations: Rc<Observations>,
    remaining: AccumulatorChain<Node>,
}

impl Clone for Payload {
    fn clone(&self) -> Self {
        self.observations
            .clones
            .set(self.observations.clones.get() + 1);
        Self {
            id: self.id,
            observations: self.observations.clone(),
            remaining: AccumulatorChain::default(),
        }
    }
}

impl Drop for Payload {
    fn drop(&mut self) {
        let depth = self.observations.active.get() + 1;
        self.observations.active.set(depth);
        self.observations
            .maximum
            .set(self.observations.maximum.get().max(depth));
        self.observations.released.borrow_mut().push(self.id);
        drop(self.remaining.take());
        self.observations.active.set(depth - 1);
    }
}

#[derive(Clone, rum_derive::AccumulatorChainNode)]
struct Node {
    #[accumulator_link]
    previous: AccumulatorChain<Self>,
    payload: Payload,
}

impl Drop for Node {
    fn drop(&mut self) {
        // Test instrumentation exposes recursive link release as nested payload
        // drops. Production nodes still have no manual Drop implementation.
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

fn released(observations: &Observations) -> Vec<usize> {
    let mut ids = observations.released.borrow().clone();
    ids.sort_unstable();
    ids
}

#[test]
fn unique_completion_moves_nodes_and_detaches_every_link() {
    let observations = Rc::new(Observations::default());
    let chain = prefix(&observations, 0..4, AccumulatorChain::default());
    let mut cursor = chain.into_completion();
    for id in (0..4).rev() {
        let node = cursor.next().unwrap();
        assert_eq!(node.payload.id, id);
        assert!(node.previous.is_none());
        assert_eq!(observations.clones.get(), 0);
        drop(node);
    }
    assert!(cursor.next().is_none());
    assert!(cursor.next().is_none());
    assert_eq!(released(&observations), vec![0, 1, 2, 3]);
    assert_eq!(observations.maximum.get(), 1);
}

#[test]
fn shared_completion_clones_nodes_and_preserves_a_retained_snapshot() {
    let observations = Rc::new(Observations::default());
    let snapshot = prefix(&observations, 0..4, AccumulatorChain::default());
    let mut cursor = snapshot.clone().into_completion();
    for id in (0..4).rev() {
        let node = cursor.next().unwrap();
        assert_eq!(node.payload.id, id);
        assert!(node.previous.is_none());
    }
    assert!(cursor.next().is_none());
    assert_eq!(observations.clones.get(), 4);
    let mut remaining = snapshot.as_deref();
    for id in (0..4).rev() {
        let node = remaining.unwrap();
        assert_eq!(node.payload.id, id);
        remaining = node.previous.as_deref();
    }
    assert!(remaining.is_none());
    drop(snapshot);
    assert_eq!(released(&observations), vec![0, 0, 1, 1, 2, 2, 3, 3]);
    assert_eq!(observations.maximum.get(), 1);
}

#[test]
fn early_cursor_drop_drains_remaining_unique_links_iteratively() {
    let observations = Rc::new(Observations::default());
    let mut cursor = prefix(&observations, 0..16, AccumulatorChain::default()).into_completion();
    let node = cursor.next().unwrap();
    assert!(node.previous.is_none());
    drop(cursor);
    assert_eq!(released(&observations), (0..15).collect::<Vec<_>>());
    drop(node);
    assert_eq!(released(&observations), (0..16).collect::<Vec<_>>());
    assert_eq!(observations.clones.get(), 0);
    assert_eq!(observations.maximum.get(), 1);
    eprintln!("early completion cursor: 16 nodes, maximum payload drop depth=1");
}

#[test]
fn early_cursor_drop_stops_at_shared_tail_then_snapshot_drains() {
    let observations = Rc::new(Observations::default());
    let tail = prefix(&observations, 0..4, AccumulatorChain::default());
    let chain = prefix(&observations, 4..8, tail.clone());
    let mut cursor = chain.into_completion();
    drop(cursor.next().unwrap());
    drop(cursor);
    assert_eq!(released(&observations), vec![4, 5, 6, 7]);
    assert_eq!(tail.as_deref().unwrap().payload.id, 3);
    drop(tail);
    assert_eq!(released(&observations), (0..8).collect::<Vec<_>>());
    assert_eq!(observations.maximum.get(), 1);
    eprintln!("completion shared tail: 8 nodes, maximum payload drop depth=1");
}
