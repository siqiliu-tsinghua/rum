//! Independent of descriptor Checks: every driven session must remain fault-free.
use rum_runtime::{FaultProbe, Session};
use std::sync::Mutex;

static OBSERVATIONS: Mutex<Vec<(FaultProbe, u64)>> = Mutex::new(Vec::new());

pub struct Check {
    probe: FaultProbe,
    before: u64,
}

impl Check {
    pub fn new(session: &Session) -> Self {
        let probe = session.continuation_fault_probe();
        let before = probe.occurrences();
        OBSERVATIONS.lock().unwrap().push((probe.clone(), before));
        Self { probe, before }
    }

    pub fn assert_unchanged(&self) {
        assert_eq!(
            self.probe.occurrences(),
            self.before,
            "oracle continuation slot fault"
        );
    }
}

impl Drop for Check {
    fn drop(&mut self) {
        // An already failing test must not double-panic. The final independent
        // observation still includes its probe, even if the session was reset.
        if !std::thread::panicking() {
            self.assert_unchanged();
        }
    }
}

pub fn assert_all() {
    let observations = OBSERVATIONS.lock().unwrap();
    assert!(!observations.is_empty());
    let delta: u64 = observations
        .iter()
        .map(|(probe, before)| probe.occurrences() - before)
        .sum();
    println!(
        "continuation-slot fault-occurrence delta={delta}; observations={}",
        observations.len()
    );
    assert_eq!(delta, 0);
}
