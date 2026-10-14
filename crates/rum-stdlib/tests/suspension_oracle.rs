// The host implementation uses crate-root paths; these aliases preserve those
// paths while compiling its unchanged source for the restore boundary witness.
#[path = "suspension_oracle/host_facade.rs"]
mod host_facade;
pub use host_facade::*;

#[path = "suspension_oracle/mod.rs"]
mod oracle;

// The prescribed oracle command runs tests serially. Keep this last so its
// retained probes also cover sessions replaced during lifecycle checks.
#[test]
fn zz_continuation_fault_occurrences() {
    oracle::support::faults::assert_all();
}

#[test]
fn runner_scan_middle_and_twice() {
    use oracle::descriptors::{Checks, Fixture, Pause, accept};
    use rum_runtime::RuntimeInputKind;
    for twice in [false, true] {
        let mut f = Fixture::new(
            "Scan.callback",
            &format!(
                "Scan[Function[x,(AppendTo[Global`log,x];If[Or[x==2,{}],InputString[\"Scan.callback\"]];x)],{{1,2,3,4,5}}]",
                if twice { "x==4" } else { "False" }
            ),
        );
        f.pauses = (0..if twice { 2 } else { 1 })
            .map(|_| Pause {
                prompt: "Scan.callback".into(),
                kind: RuntimeInputKind::InputString,
                answer: "answer".into(),
            })
            .collect();
        f.expected_value = Some("Null".into());
        f.expected_log = Some("List[1, 2, 3, 4, 5]".into());
        accept(&f, Checks::ALL).unwrap();
    }
}
