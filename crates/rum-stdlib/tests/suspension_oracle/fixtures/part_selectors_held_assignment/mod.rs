//! S7 Part selectors and held updates: queued input equals resumed input.
//! Contract: guide assignments, Part, and held mutation; Q5 remains unspecified.
use super::super::descriptors::{Checks, accept};
mod director;
mod held;
mod markers;

fn equality(id: &str, setup: &str, source: &str, prompts: &[&str], value: Option<&str>, log: &str) {
    let pauses: Vec<_> = prompts.iter().map(|p| ("InputString", *p, "1")).collect();
    let mut fixture = super::make(id, source, &pauses, value, log);
    fixture.setup = setup.into();
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
        "S7 {id} pauses={} strict equality PASS: {:?}; log={}",
        prompts.len(),
        accepted.observation,
        accepted.log
    );
}
