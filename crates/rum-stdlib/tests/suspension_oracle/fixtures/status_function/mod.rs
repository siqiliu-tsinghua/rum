//! G15-A: a status call completes its Function stage after every pause.
//! The contract is the same documented status projection and Return boundary
//! with queued input and host suspension (guide: evaluation and control flow).
use super::super::descriptors::{Checks, accept};

fn equality(id: &str, setup: &str, source: &str, prompts: &[&str], value: &str, log: &str) {
    let pauses: Vec<_> = prompts
        .iter()
        .map(|prompt| ("InputString", *prompt, "1"))
        .collect();
    let mut fixture = super::make(id, source, &pauses, Some(value), log);
    fixture.setup = format!("log = {{}};{setup}");
    // Checks::ALL asserts the exact phase prompts and pause count. The shared
    // driver checks pending ownership, overlap rejection, the empty queue, and
    // a fresh request; its FaultProbe check is unconditional.
    let result = accept(&fixture, Checks::ALL);
    if result.is_err() {
        println!(
            "{id} queued: {:?}",
            super::super::descriptors::run(&fixture, true, Checks::ALL)
        );
        println!(
            "{id} resumed: {:?}",
            super::super::descriptors::run(&fixture, false, Checks::ALL)
        );
    }
    super::super::support::faults::assert_all();
    let accepted = result.unwrap_or_else(|error| panic!("{id}: {error:?}"));
    println!(
        "G15-A {id} pauses={} strict equality PASS: {:?}; log={}",
        prompts.len(),
        accepted.observation,
        accepted.log
    );
}

macro_rules! case {
    ($name:ident, $setup:expr, $source:expr, $prompts:expr, $value:expr, $log:expr) => {
        #[test]
        fn $name() {
            super::equality(
                concat!("G15A.", stringify!($name)),
                $setup,
                $source,
                $prompts,
                $value,
                $log,
            );
        }
    };
}

mod controls;
mod guest_heads;
mod return_boundaries;
