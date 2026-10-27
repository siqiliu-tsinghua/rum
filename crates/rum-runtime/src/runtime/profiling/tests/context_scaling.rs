//! Perf's acknowledged control channel isolates each context transition.
use super::super::state::{LEXICAL_CONTEXT_DEPTH, LEXICAL_CONTEXT_STACK};
use crate::runtime::Session;
use crate::runtime::profiling::logical::{
    AggregateKey, Context, Epoch, Owner, Policy, Retire, epoch, segments,
};
use std::fs::{File, OpenOptions};
use std::hint::black_box;
use std::io::{Read, Write};
use std::sync::Arc;

struct Counter {
    control: File,
    acknowledge: File,
}
impl Counter {
    fn open() -> Self {
        let pipe = |name| {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(std::env::var(name).unwrap())
                .unwrap()
        };
        Self {
            control: pipe("LP02_PERF_CONTROL"),
            acknowledge: pipe("LP02_PERF_ACK"),
        }
    }
    fn command(&mut self, command: &[u8]) {
        self.control.write_all(command).unwrap();
        // This perf build sends the terminating NUL with its acknowledgement.
        let mut ack = [0; 5];
        self.acknowledge.read_exact(&mut ack).unwrap();
        assert_eq!(&ack, b"ack\n\0");
    }
    fn measure<T>(&mut self, operation: impl FnOnce() -> T) -> T {
        self.command(b"enable\n");
        let result = black_box(operation());
        self.command(b"disable\n");
        result
    }
}

#[test]
#[ignore = "run explicitly with perf stat --delay=-1 and acknowledged control FIFOs"]
fn logical_context_instruction_probe() {
    let depth: usize = std::env::var("LP02_PROBE_SIZE").unwrap().parse().unwrap();
    let repeats: usize = std::env::var("LP02_PROBE_REPEATS")
        .unwrap()
        .parse()
        .unwrap();
    let operation = std::env::var("LP02_PROBE_OPERATION").unwrap();
    let session = Session::new();
    let control = Arc::clone(&session.request_interrupt_control.borrow());
    let owner = Owner {
        generation: control.logical_generation(),
        control,
        root: epoch::next_id(),
    };
    let epoch = Epoch::new(16 * 1024 * 1024).unwrap();
    let key = AggregateKey::Runtime {
        phase: "context-probe",
        source: None,
    };
    let mut records = Vec::with_capacity(depth);
    let mut active = Vec::with_capacity(depth);
    for _ in 0..depth {
        let id = epoch
            .open(owner.clone(), records.last().copied(), key, Policy::Expr)
            .unwrap();
        records.push(id);
        active.push(segments::enter(&epoch, id).unwrap());
        LEXICAL_CONTEXT_STACK.with_borrow_mut(|stack| stack.push("context-probe"));
    }
    LEXICAL_CONTEXT_DEPTH.with(|n| n.set(depth));
    let context = Context::capture(&epoch).unwrap();
    while let Some(segment) = active.pop() {
        drop(segment);
    }
    LEXICAL_CONTEXT_STACK.with_borrow_mut(|stack| stack.clear());
    LEXICAL_CONTEXT_DEPTH.with(|n| n.set(0));
    let mut counter = Counter::open();
    for _ in 0..repeats {
        match operation.as_str() {
            "context-enter" => {
                let guard = counter.measure(|| context.enter().unwrap());
                drop(guard);
            }
            "context-pause" => {
                let guard = context.enter().unwrap();
                counter.measure(|| guard.pause());
                drop(guard);
            }
            "context-drop" => {
                let guard = context.enter().unwrap();
                counter.measure(|| drop(guard));
            }
            "empty" => counter.measure(|| black_box(())),
            _ => panic!("unknown context probe operation"),
        }
    }
    assert!(epoch.is_valid());
    assert_eq!(segments::active_count(), 0);
    drop(context);
    assert_eq!(epoch.contexts.load(std::sync::atomic::Ordering::Relaxed), 0);
    epoch.retire(Retire::ProfileReset);
}
