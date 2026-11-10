//! Direct Default lookup and fallback completion preserve their own outcomes.
//! The two recipes are brief 719's logical-profiler contract witnesses.
use super::{epoch, test_support};
use crate::runtime::suspension::AbandonReason;
use crate::runtime::{EvalError, Session};

const DIRECT: &str = "down_values.direct_default_lookup";
const NONE: [u64; 10] = [1, 1, 0, 0, 1, 0, 0, 0, 0, 0];
const SOME: [u64; 10] = [1, 1, 0, 1, 0, 0, 0, 0, 0, 0];
const OPEN: [u64; 10] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

fn direct(rows: &[(String, [u64; 10])]) -> [u64; 10] {
    rows.iter().find(|(key, _)| key.contains(DIRECT)).unwrap().1
}

fn fallback(rows: &[(String, [u64; 10])]) -> [u64; 10] {
    let found: Vec<_> = rows
        .iter()
        .filter(|(key, _)| key.contains("down_value_rhs_eval") && key.contains("name: \"Default\""))
        .collect();
    assert_eq!(found.len(), 1, "one fallback definition: {rows:?}");
    found[0].1
}

fn assert_closed(rows: &[(String, [u64; 10])]) {
    for (key, counts) in rows {
        assert_eq!(counts[0], counts[1] + counts[2], "{key}: {counts:?}");
        assert_eq!((counts[8], counts[9]), (0, 0), "{key}: {counts:?}");
    }
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(test_support::active(), 0);
}

fn witness(rhs_pauses: bool) {
    let enabled = test_support::enabled();
    let setup = if rhs_pauses {
        "Default[x_]:=(InputString[\"rhs\"];83);Default[d]:=73/;(InputString[\"guard\"];False)"
    } else {
        "Default[x_]:=83;Default[d]:=73/;(InputString[\"guard\"];False)"
    };
    let mut queued_rows = None;
    for queued in [true, false] {
        let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
        session.eval_text(setup).unwrap();
        epoch::reset_for_lease(enabled.lease());
        if queued {
            session.queue_input_response("answer");
            if rhs_pauses {
                session.queue_input_response("answer");
            }
        }
        let mut result = session.eval_text("Default[d]").map(|r| r.value);
        if !queued {
            assert!(
                matches!(&result, Err(EvalError::InputRequired { request }) if request.prompt == "guard")
            );
            assert_eq!(direct(&Session::logical_profile_test_rows()), OPEN);
            result = session.resume_suspended_evaluation("answer");
            if rhs_pauses {
                assert!(
                    matches!(&result, Err(EvalError::InputRequired { request }) if request.prompt == "rhs")
                );
                let paused = Session::logical_profile_test_rows();
                assert_eq!(
                    direct(&paused),
                    NONE,
                    "direct lookup closes before fallback RHS"
                );
                assert_eq!(
                    fallback(&paused),
                    OPEN,
                    "RHS remains open at its actual pause"
                );
                result = session.resume_suspended_evaluation("answer");
            }
        }
        assert_eq!(result.unwrap().to_string(), "83");
        let rows = Session::logical_profile_test_rows();
        assert_eq!(direct(&rows), NONE);
        if rhs_pauses {
            assert_eq!(fallback(&rows), SOME);
        }
        assert_closed(&rows);
        eprintln!(
            "BRIEF719 NATIVE rhs_pauses={rhs_pauses} queued={queued} direct={:?} terminal_open=0",
            direct(&rows)
        );
        if let Some(expected) = &queued_rows {
            assert_eq!(expected, &rows, "all ten fields for every logical key");
        } else {
            queued_rows = Some(rows);
        }
    }
}

#[test]
fn default_fallback_direct_decline_completes_before_fallback() {
    witness(false);
}

#[test]
fn default_fallback_paused_rhs_closes_definition_at_completion() {
    witness(true);
}

#[test]
fn default_fallback_error_return_repeat_and_lifecycle() {
    let enabled = test_support::enabled();
    for (setup, prompts, answers, expected) in [
        (
            "Default[x_]:=83;Default[d]:=73/;(Input[\"guard\"];False)",
            vec!["guard"],
            vec!["("],
            None,
        ),
        (
            "Default[x_]:=(InputString[\"rhs\"];Return[83]);Default[d]:=73/;(InputString[\"guard\"];False)",
            vec!["guard", "rhs"],
            vec!["a", "b"],
            Some("83"),
        ),
        (
            "Default[x_]:=83;Default[d]:=73/;(InputString[\"guard\"];Throw[17])",
            vec!["guard"],
            vec!["a"],
            None,
        ),
        (
            "Default[x_]:=83;Default[d]:=73/;(InputString[\"one\"];InputString[\"two\"];False)",
            vec!["one", "two"],
            vec!["a", "b"],
            Some("83"),
        ),
    ] {
        let mut queued_observation = None;
        for queued in [true, false] {
            let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
            session.eval_text(setup).unwrap();
            epoch::reset_for_lease(enabled.lease());
            if queued {
                for answer in &answers {
                    session.queue_input_response(*answer);
                }
            }
            let mut result = session.eval_text("Default[d]").map(|r| r.value);
            if !queued {
                for (prompt, answer) in prompts.iter().zip(&answers) {
                    assert!(
                        matches!(&result, Err(EvalError::InputRequired { request }) if request.prompt == *prompt)
                    );
                    result = session.resume_suspended_evaluation(*answer);
                }
            }
            let observation = result
                .map(|value| value.to_string())
                .map_err(|error| error.to_string());
            match expected {
                Some(value) => assert_eq!(observation.as_deref(), Ok(value)),
                None => assert!(observation.is_err()),
            }
            let rows = Session::logical_profile_test_rows();
            assert_closed(&rows);
            if let Some(expected) = &queued_observation {
                assert_eq!(expected, &(observation, rows));
            } else {
                queued_observation = Some((observation, rows));
            }
        }
    }
    for at_rhs in [false, true] {
        for reason in [
            AbandonReason::CancelledRequest,
            AbandonReason::HelperBestEffortDiscard,
        ] {
            let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
            session.eval_text("Default[x_]:=(InputString[\"rhs\"];83);Default[d]:=73/;(InputString[\"guard\"];False)").unwrap();
            epoch::reset_for_lease(enabled.lease());
            assert!(matches!(
                session.eval_text("Default[d]"),
                Err(EvalError::InputRequired { .. })
            ));
            if at_rhs {
                assert!(matches!(
                    session.resume_suspended_evaluation("answer"),
                    Err(EvalError::InputRequired { .. })
                ));
            }
            session.abandon_pending_suspension(reason);
            session.abandon_pending_suspension(reason);
            let rows = Session::logical_profile_test_rows();
            let counts = if at_rhs {
                assert_eq!(direct(&rows), NONE);
                fallback(&rows)
            } else {
                direct(&rows)
            };
            assert_eq!(counts[0], 1);
            assert_eq!(counts[9], 0);
            if reason == AbandonReason::CancelledRequest {
                assert_eq!((counts[2], counts[8]), (1, 0));
            } else {
                assert_eq!((counts[2], counts[8]), (0, 1));
            }
            Session::logical_profile_test_assert_quiescent();
            assert_eq!(test_support::active(), 0);
            eprintln!(
                "BRIEF719 LIFECYCLE {reason:?} at_rhs={at_rhs} target={counts:?} terminal_open=0"
            );
        }
    }
}

// Regression coverage for definition-resume cancellation and abandonment.
#[test]
fn brief720_definition_resume_lifecycle_probe() {
    let enabled = test_support::enabled();
    for (kind, setup, source) in [
        (
            "down_value_rhs_eval",
            "f[x_Integer]:=((InputString[\"r1\"];InputString[\"r2\"];83)/;(InputString[\"g\"];True))",
            "f[2]",
        ),
        (
            "sub_value_rhs_eval",
            "f[a_][x_Integer]:=((InputString[\"r1\"];InputString[\"r2\"];83)/;(InputString[\"g\"];True))",
            "f[1][2]",
        ),
        (
            "up_value_rhs_eval",
            "TagSetDelayed[a,f[a,x_Integer],((InputString[\"r1\"];InputString[\"r2\"];83)/;(InputString[\"g\"];True))]",
            "f[a,2]",
        ),
    ] {
        for answers in [0, 1, 2] {
            for reason in [
                AbandonReason::CancelledRequest,
                AbandonReason::HelperBestEffortDiscard,
            ] {
                let mut session =
                    crate::runtime::test_builtins::session_with_test_control_builtins();
                session.eval_text(setup).unwrap();
                epoch::reset_for_lease(enabled.lease());
                assert!(matches!(
                    session.eval_text(source),
                    Err(EvalError::InputRequired { .. })
                ));
                for _ in 0..answers {
                    assert!(matches!(
                        session.resume_suspended_evaluation("answer"),
                        Err(EvalError::InputRequired { .. })
                    ));
                }
                session.abandon_pending_suspension(reason);
                let rows = Session::logical_profile_test_rows();
                session.abandon_pending_suspension(reason);
                assert_eq!(rows, Session::logical_profile_test_rows());
                let found: Vec<_> = rows.iter().filter(|(key, _)| key.contains(kind)).collect();
                if answers == 0 {
                    assert!(found.is_empty());
                } else {
                    assert_eq!(found.len(), 1);
                    let counts = found[0].1;
                    assert_eq!(counts[0], 1);
                    assert_eq!(counts[9], 0);
                    if reason == AbandonReason::CancelledRequest {
                        assert_eq!((counts[2], counts[8]), (1, 0));
                    } else {
                        assert_eq!((counts[2], counts[8]), (0, 1));
                    }
                }
                Session::logical_profile_test_assert_quiescent();
                assert_eq!(test_support::active(), 0);
                eprintln!(
                    "BRIEF720 LIFECYCLE PASS kind={kind} answered={answers} reason={reason:?} definition={found:?} duplicate_abandon=unchanged terminal_open=0"
                );
            }
        }
    }
}
