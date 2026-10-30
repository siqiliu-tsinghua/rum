//! Native instruction probe: setup is identical at N and 2N operations.
use super::super::state::{LEXICAL_CONTEXT_DEPTH, LEXICAL_CONTEXT_STACK};
use crate::runtime::Session;
use crate::runtime::profiling::logical::{
    AggregateKey, Context, Epoch, Outcome, Owner, Policy, epoch, segments,
};
use std::hint::black_box;
use std::sync::Arc;

#[test]
#[ignore = "run explicitly under perf stat; sizes and repetitions supplied by the driver"]
fn logical_engine_instruction_probe() {
    let n: usize = std::env::var("LP02_PROBE_SIZE").unwrap().parse().unwrap();
    let repeats: usize = std::env::var("LP02_PROBE_REPEATS")
        .unwrap()
        .parse()
        .unwrap();
    let axis = std::env::var("LP02_PROBE_AXIS").unwrap();
    let operation = std::env::var("LP02_PROBE_OPERATION").unwrap();
    let session = Session::new();
    let control = Arc::clone(&session.request_interrupt_control.borrow());
    let owner = Owner {
        generation: control.logical_generation(),
        control,
        root: epoch::next_id(),
    };
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 16 * 1024 * 1024).unwrap();
    let symbol = crate::runtime::Symbol::new("System`", "LogicalProbe");
    let key = |i| AggregateKey::Definition {
        kind: "probe",
        symbol,
        index: i as u32,
    };
    let mut records = Vec::with_capacity(n);
    for i in 0..n {
        let parent = if operation == "segment" || operation == "context-capture" {
            records.last().copied()
        } else {
            None
        };
        let id = epoch
            .open(
                owner.clone(),
                parent,
                key(if axis == "keys" { i } else { 0 }),
                Policy::Expr,
            )
            .unwrap();
        if axis == "keys" {
            assert!(epoch.close(id, Ok(Outcome::Ok)));
        } else {
            records.push(id);
        }
    }
    let selected = key(if axis == "keys" { n - 1 } else { 0 });
    let target = if axis == "keys" {
        epoch
            .open(owner.clone(), None, selected, Policy::Expr)
            .unwrap()
    } else {
        records[n - 1]
    };
    let mut active = Vec::new();
    if operation == "segment" || operation == "context-capture" {
        if axis == "keys" {
            active.push(segments::enter(&epoch, target).unwrap());
        } else {
            for id in &records {
                active.push(segments::enter(&epoch, *id).unwrap());
            }
        }
    }
    if operation == "context-capture" {
        LEXICAL_CONTEXT_STACK.with_borrow_mut(|s| {
            for _ in 0..active.len() {
                s.push("probe");
            }
        });
        LEXICAL_CONTEXT_DEPTH.with(|d| d.set(active.len()));
    }
    for _ in 0..repeats {
        match operation.as_str() {
            "open-close" => {
                let id = epoch
                    .open(
                        owner.clone(),
                        Some(target),
                        black_box(selected),
                        Policy::Expr,
                    )
                    .unwrap();
                assert!(epoch.close(black_box(id), Ok(Outcome::Ok)));
            }
            "inspect" => {
                black_box(epoch.inspect(black_box(target)).unwrap());
            }
            "interval" => epoch.interval(black_box(target), 13, 4),
            "pause" => epoch.pause(black_box(target)),
            "segment" => {
                let id = epoch
                    .open(owner.clone(), Some(target), selected, Policy::Expr)
                    .unwrap();
                drop(segments::enter(&epoch, black_box(id)).unwrap());
                assert!(epoch.close(id, Ok(Outcome::Ok)));
            }
            "context-capture" => {
                drop(black_box(Context::capture(&epoch).unwrap()));
            }
            _ => panic!("unknown instruction probe operation"),
        }
    }
    if operation == "context-capture" {
        LEXICAL_CONTEXT_STACK.with_borrow_mut(|s| s.clear());
        LEXICAL_CONTEXT_DEPTH.with(|d| d.set(0));
    }
    while let Some(segment) = active.pop() {
        drop(segment);
    }
    assert!(epoch.is_valid());
    epoch.retire(crate::runtime::profiling::logical::Retire::ProfileReset);
}
