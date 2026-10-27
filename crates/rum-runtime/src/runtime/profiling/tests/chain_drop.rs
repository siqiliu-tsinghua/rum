//! The ownership line's 66,000-node failure depth on the ruled 2 MiB stack.
use crate::runtime::RequestInterruptState;
use crate::runtime::profiling::logical::{
    Epoch, Owner, RecordPath, source_stack::SourceStack, test_support,
};
use std::sync::{Arc, Barrier};

const DEPTH: usize = 66_000;
const STACK_BYTES: usize = 2 * 1024 * 1024;

fn drop_on_small_stack<T: Send + 'static>(value: T) {
    std::thread::Builder::new()
        .name("logical-chain-drop".into())
        .stack_size(STACK_BYTES)
        .spawn(move || drop(value))
        .unwrap()
        .join()
        .unwrap();
}

fn concurrent_drop<T: Send + 'static>(first: T, second: T) {
    let barrier = Arc::new(Barrier::new(2));
    let workers: Vec<_> = [first, second]
        .into_iter()
        .map(|value| {
            let barrier = Arc::clone(&barrier);
            std::thread::Builder::new()
                .name("logical-shared-chain-drop".into())
                .stack_size(STACK_BYTES)
                .spawn(move || {
                    barrier.wait();
                    drop(value);
                })
                .unwrap()
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
}

fn record_chain(concurrent: bool) {
    let epoch = Epoch::new(16 * 1024 * 1024).unwrap();
    let control = Arc::new(RequestInterruptState::new());
    let owner = Owner {
        control: Arc::clone(&control),
        generation: 1,
        root: 1,
    };
    let chain = RecordPath::test_chain(&epoch, owner, DEPTH);
    let weak = Arc::downgrade(&chain);
    assert_eq!(Arc::strong_count(&control), DEPTH + 1);
    if concurrent {
        let other = Arc::clone(&chain);
        assert_eq!(Arc::strong_count(&chain), 2);
        concurrent_drop(chain, other);
    } else {
        drop_on_small_stack(chain);
    }
    assert!(weak.upgrade().is_none());
    assert_eq!(
        Arc::strong_count(&control),
        1,
        "every node released its owner"
    );
    assert!(
        control.is_idle(),
        "releasing an owner does not cancel a request"
    );
    assert!(epoch.is_valid());
    // The chain occupies over half the cap. This reservation cannot succeed
    // if its Storage claims remain charged after the nodes have been released.
    let bytes = DEPTH * (size_of::<RecordPath>() + 2 * size_of::<usize>());
    assert!(bytes > 8 * 1024 * 1024);
    assert!(epoch.storage(bytes).is_some());
    eprintln!(
        "RecordPath depth={DEPTH} stack={STACK_BYTES} concurrent={concurrent}: released all nodes and budget"
    );
}

fn source_chain(concurrent: bool) {
    let _enabled = test_support::enabled();
    let value = Arc::new(());
    let mut chain = SourceStack::new();
    for _ in 0..DEPTH {
        chain.push(Arc::clone(&value));
    }
    assert_eq!(chain.len(), DEPTH);
    assert!(
        chain.last().is_some(),
        "the whole chain must fit the diagnostic cap"
    );
    assert_eq!(Arc::strong_count(&value), DEPTH + 1);
    if concurrent {
        let other = chain.clone();
        // Cloning the persistent stack shares the nodes, not their payloads.
        assert_eq!(Arc::strong_count(&value), DEPTH + 1);
        concurrent_drop(chain, other);
    } else {
        drop_on_small_stack(chain);
    }
    assert_eq!(
        Arc::strong_count(&value),
        1,
        "every node released its payload"
    );
    assert!(test_support::epoch().is_valid());
    eprintln!(
        "SourceStack depth={DEPTH} stack={STACK_BYTES} concurrent={concurrent}: released all nodes"
    );
}

#[test]
fn logical_record_chain_drops_on_two_mib() {
    record_chain(false);
}

#[test]
fn logical_source_chain_drops_on_two_mib() {
    source_chain(false);
}

#[test]
fn logical_record_chain_concurrent_last_holders() {
    record_chain(true);
}

#[test]
fn logical_source_chain_concurrent_last_holders() {
    source_chain(true);
}
