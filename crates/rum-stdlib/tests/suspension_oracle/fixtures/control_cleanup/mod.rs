//! S9 control and cleanup: queued input equals resumed input.
use super::super::descriptors::{Checks, accept};
mod director;
mod harness;

fn equality(id: &str, setup: &str, source: &str, prompts: &[&str], value: Option<&str>) {
    let pauses: Vec<_> = prompts.iter().map(|p| ("InputString", *p, "1")).collect();
    let mut fixture = super::make(id, source, &pauses, value, "List[]");
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
    if value.is_none() {
        assert!(accepted.observation.error.is_some(), "{id} expected error");
    }
    println!(
        "S9 {id} pauses={} strict equality PASS: {:?}; log={}",
        prompts.len(),
        accepted.observation,
        accepted.log
    );
}
