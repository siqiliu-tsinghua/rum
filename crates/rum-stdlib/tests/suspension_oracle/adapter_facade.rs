//! Q21 source reuse: the observation driver is inside the worker's own module.
#![allow(dead_code, unused_imports)]
include!("../../../rum-jupyter/src/adapter/mod.rs");

pub(crate) mod observed_worker {
    include!("../../../rum-jupyter/src/adapter/worker.rs");

    // Q22/Q23: the historical c2_probe begin/end calls are provenance only
    // (sealed witness 512785d2e13d77db194406f6212fef330c2a673c964410cfb44ad358a63ebdba).
    // The caller holds the live ledger lease and resets before the subject.
    pub(crate) fn observe_segment(
        session: &mut HostSession,
        code: Option<&str>,
        input: &str,
        cancel: &HostCancelHandle,
    ) -> (KernelExecuteOutcome, Option<Vec<(String, [u64; 10])>>) {
        let segment = match code {
            Some(code) => ExecuteSegment::Eval {
                code: code.into(),
                allow_stdin: true,
            },
            None => ExecuteSegment::Resume {
                input: input.into(),
            },
        };
        let outcome = run_segment(session, segment, cancel);
        let native = if matches!(&outcome, KernelExecuteOutcome::InputRequired(_)) {
            None
        } else {
            Some(rum_runtime::Session::logical_profile_test_rows())
        };
        (outcome, native)
    }
}
