//! LP05 falsifier: the adopted assignment phases are present, closed exactly
//! once and quiescent, in completed runs and in a run that pauses twice (the
//! RHS pauses first, then the LHS preparation pauses, in the same operation).
//!
//! Calibrated RED on the base tree (2026-11-03, HEAD `ab3c1c89`): LP04 records
//! the definition phases but no assignment phase, so every `set.*` row below
//! is missing. It must be GREEN after LP05 and is never weakened to reach
//! green.
use rum_host::{HostEvalOutcome, HostSession};
use rum_runtime::kernel_api::Session;

/// Every case reaches each listed `set.*` phase at HEAD with the legacy
/// recorders (`phase-sites.tsv`); the pause cases additionally prove the
/// operation observes the stated number of `InputRequired` outcomes.
const CASES: &[Case] = &[
    Case {
        program: "x = 1; x",
        responses: &[],
        phases: &["set.rhs.evaluate", "set.rhs.suspend", "set.finish"],
    },
    Case {
        program: "{a, b} = {1, 2}; {a, b}",
        responses: &[],
        phases: &[
            "set.rhs.evaluate",
            "set.rhs.suspend",
            "set.finish",
            "set.try_list_destructuring",
        ],
    },
    Case {
        program: "assoc = <|\"k\" -> 1|>; assoc[\"k\"] = 2; assoc",
        responses: &[],
        phases: &[
            "set.rhs.evaluate",
            "set.rhs.suspend",
            "set.finish",
            "set.try_association_lvalue",
        ],
    },
    Case {
        program: "f[g_] = g + 1; f[2]",
        responses: &[],
        phases: &[
            "set.rhs.evaluate",
            "set.rhs.suspend",
            "set.finish",
            "set.prepare_lhs",
        ],
    },
    Case {
        program: "x = StringLength[InputString[\"r\"]]; x",
        responses: &["3"],
        phases: &["set.rhs.evaluate", "set.rhs.suspend", "set.finish"],
    },
    Case {
        program: "f[StringLength[InputString[\"l\"]]] = StringLength[InputString[\"r\"]]; f[3]",
        responses: &["2", "3"],
        phases: &[
            "set.rhs.evaluate",
            "set.rhs.suspend",
            "set.finish",
            "set.prepare_lhs",
        ],
    },
];

struct Case {
    program: &'static str,
    /// Inputs supplied in order; each one must be demanded by an
    /// `InputRequired` outcome of this same operation.
    responses: &'static [&'static str],
    /// Rows that must exist, be closed exactly once and be quiescent.
    phases: &'static [&'static str],
}

fn run(program: &str, responses: &[&str]) -> Vec<(String, [u64; 10])> {
    let lease = Session::logical_profile_test_acquire();
    Session::logical_profile_test_reset(&lease);
    let mut session = HostSession::builder().build();
    let mut outcome = session.eval(program);
    for (index, input) in responses.iter().enumerate() {
        assert!(
            matches!(outcome, HostEvalOutcome::InputRequired(_)),
            "{program}: pause {} was not demanded: {outcome:?}",
            index + 1
        );
        outcome = session.resume_with_input(*input);
    }
    assert!(
        !matches!(outcome, HostEvalOutcome::InputRequired(_)),
        "{program}: the operation paused after its last response: {outcome:?}"
    );
    Session::logical_profile_test_assert_quiescent();
    let rows = Session::logical_profile_test_rows();
    if !responses.is_empty() {
        assert!(
            Session::logical_profile_test_pauses() > 0,
            "{program}: no pause was recorded"
        );
    }
    rows
}

fn row(rows: &[(String, [u64; 10])], phase: &str) -> Option<[u64; 10]> {
    rows.iter()
        .find(|(key, _)| key.contains(&format!("phase: \"{phase}\"")))
        .map(|(_, counts)| *counts)
}

fn assert_closed(label: &str, counts: [u64; 10]) {
    // calls, ok, err, some, none, return_err, throw_err, cancelled,
    // abandoned, open.
    assert_eq!(counts[0], counts[1] + counts[2], "{label}: {counts:?}");
    assert!(counts[0] > 0, "{label} never called: {counts:?}");
    assert_eq!(counts[8], 0, "{label} abandoned: {counts:?}");
    assert_eq!(counts[9], 0, "{label} left open: {counts:?}");
}

#[test]
fn lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs() {
    for case in CASES {
        let program = case.program;
        let rows = run(program, case.responses);
        for phase in case.phases {
            let counts = row(&rows, phase)
                .unwrap_or_else(|| panic!("{phase} missing for `{program}`: {rows:?}"));
            assert_closed(phase, counts);
        }
    }
}
