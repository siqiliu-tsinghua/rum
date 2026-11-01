//! LP04 falsifier: the adopted definition RHS, definition-rule and dispatch
//! phases are present, closed exactly once and quiescent, in a completed run
//! and in a run whose definition RHS pauses and resumes.
//!
//! Calibrated RED on the base tree (2026-11-01, HEAD `8a7eaae3`): LP03 records
//! the evaluator routes but no definition phase, so every row below is
//! missing. It must be GREEN after LP04 and is never weakened to reach green.
use rum_host::{HostEvalOutcome, HostSession};
use rum_runtime::kernel_api::Session;

/// `runtime|` phases reached by each case, and `AggregateKey::Definition`
/// kinds. Both probe programs were verified to reach every entry below with
/// the legacy recorders at HEAD (`.temp/brief-638/falsifier/reach.log`).
const CASES: &[Case] = &[
    Case {
        program: "f[x_Integer /; x > 0] := x + 1; Default[f] = 3; f[2]; Default[f]",
        resume: None,
        // Each of these must complete at least once without error.
        phases: &[
            "definition.rhs.total",
            "definition.rhs.evaluate",
            "down_values.direct_default_lookup",
            "down_values.compiled_matcher",
        ],
        kinds: &["down_value_rhs_eval"],
        // The pause lands inside the RHS, so its rows close as errors; the
        // matcher runs before the RHS and must still complete ok.
        paused_phases: &[],
    },
    Case {
        program: "f[x_Integer /; x > 0] := x + StringLength[InputString[\"p\"]]; f[2]",
        resume: Some("reply"),
        phases: &[
            "definition.rhs.total",
            "definition.rhs.evaluate",
            "down_values.compiled_matcher",
        ],
        kinds: &["down_value_rhs_eval"],
        paused_phases: &[
            "definition.rhs.total",
            "definition.rhs.evaluate",
            "down_value_rhs_eval",
        ],
    },
];

struct Case {
    program: &'static str,
    resume: Option<&'static str>,
    phases: &'static [&'static str],
    kinds: &'static [&'static str],
    /// Rows that must close with at least one error: the pause is recorded on
    /// the definition record, not silently dropped.
    paused_phases: &'static [&'static str],
}

fn run(program: &str, resume: Option<&str>) -> Vec<(String, [u64; 10])> {
    let lease = Session::logical_profile_test_acquire();
    Session::logical_profile_test_reset(&lease);
    let mut session = HostSession::builder().build();
    let outcome = session.eval(program);
    match resume {
        None => assert!(
            !matches!(outcome, HostEvalOutcome::InputRequired(_)),
            "{program}: {outcome:?}"
        ),
        Some(input) => {
            assert!(
                matches!(outcome, HostEvalOutcome::InputRequired(_)),
                "{program}: {outcome:?}"
            );
            let done = session.resume_with_input(input);
            assert!(
                !matches!(done, HostEvalOutcome::InputRequired(_)),
                "{program}: second pause {done:?}"
            );
        }
    }
    Session::logical_profile_test_assert_quiescent();
    let rows = Session::logical_profile_test_rows();
    if resume.is_some() {
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

fn rule_row(rows: &[(String, [u64; 10])], kind: &str) -> Option<[u64; 10]> {
    rows.iter()
        .find(|(key, _)| key.contains(&format!("kind: \"{kind}\"")))
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
fn lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs() {
    for case in CASES {
        let program = case.program;
        let rows = run(program, case.resume);
        for phase in case.phases {
            let counts = row(&rows, phase)
                .unwrap_or_else(|| panic!("{phase} missing for `{program}`: {rows:?}"));
            assert_closed(phase, counts);
            if case.paused_phases.contains(phase) {
                assert!(counts[2] > 0, "{phase} recorded no paused call: {counts:?}");
            } else {
                assert!(counts[1] > 0, "{phase} never completed ok: {counts:?}");
            }
        }
        for kind in case.kinds {
            let counts = rule_row(&rows, kind).unwrap_or_else(|| {
                panic!("definition-rule {kind} missing for `{program}`: {rows:?}")
            });
            assert_closed(kind, counts);
            if case.paused_phases.contains(kind) {
                assert!(counts[2] > 0, "{kind} recorded no paused call: {counts:?}");
            } else {
                assert!(counts[1] > 0, "{kind} never completed ok: {counts:?}");
            }
        }
    }
}
