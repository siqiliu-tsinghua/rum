//! LP03 falsifier: the adopted evaluator/dispatch/argument phases are present,
//! closed exactly once and quiescent, in a completed run and in a run that
//! pauses on an argument and resumes.
//!
//! Calibrated RED on the base tree (2026-11-01): the logical engine records no
//! production phase, so every `ADOPTED` row is missing. It must be GREEN after
//! LP03 and is never weakened to reach green.
use rum_host::{HostEvalOutcome, HostSession};
use rum_runtime::kernel_api::Session;

/// Rows every completed evaluation of the probe programs must produce.
const ADOPTED: &[&str] = &[
    "evaluate.total",
    "evaluate.once_unstable",
    "evaluate_inner_unstable.total",
    "evaluate_inner_unstable.symbol",
    "evaluate_call.total",
    "evaluate_call.head_eval",
    "evaluate_call.resume_after_head",
    "evaluate_call.dispatch",
    "prepare_arguments.total",
    "prepare_arguments.evaluate_arg",
];

fn run(program: &str, expect_pause: bool) -> Vec<(String, [u64; 10])> {
    let lease = Session::logical_profile_test_acquire();
    Session::logical_profile_test_reset(&lease);
    let mut session = HostSession::builder().build();
    let outcome = session.eval(program);
    if expect_pause {
        assert!(
            matches!(outcome, HostEvalOutcome::InputRequired(_)),
            "{program}: {outcome:?}"
        );
        let done = session.resume_with_input("reply");
        assert!(
            !matches!(done, HostEvalOutcome::InputRequired(_)),
            "{program}: second pause {done:?}"
        );
    } else {
        assert!(
            !matches!(outcome, HostEvalOutcome::InputRequired(_)),
            "{program}: {outcome:?}"
        );
    }
    Session::logical_profile_test_assert_quiescent();
    let rows = Session::logical_profile_test_rows();
    if expect_pause {
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

fn has_prefix(rows: &[(String, [u64; 10])], prefix: &str) -> bool {
    rows.iter()
        .any(|(key, _)| key.contains(&format!("phase: \"{prefix}")))
}

#[test]
fn lp03_adopted_phases_are_exact_in_completed_and_resumed_runs() {
    for (program, pause) in [
        ("x = 1; f[y_] := y + x; f[2]", false),
        ("x = 1; f[y_] := y + x; f[InputString[\"p\"]]", true),
    ] {
        let rows = run(program, pause);
        for phase in ADOPTED {
            let counts = row(&rows, phase)
                .unwrap_or_else(|| panic!("{phase} missing for `{program}`: {rows:?}"));
            // calls, ok, err, some, none, return_err, throw_err, cancelled,
            // abandoned, open.
            assert_eq!(counts[0], counts[1] + counts[2], "{phase}: {counts:?}");
            assert!(counts[1] > 0, "{phase} never completed ok: {counts:?}");
            assert_eq!(counts[8], 0, "{phase} abandoned: {counts:?}");
            assert_eq!(counts[9], 0, "{phase} left open: {counts:?}");
        }
        assert!(
            has_prefix(&rows, "call.stage."),
            "no call stage recorded for `{program}`"
        );
        assert!(
            has_prefix(&rows, "evaluate_atom_fast_path."),
            "no atom fast path recorded for `{program}`"
        );
    }
}
