//! LP07 completion ownership: actual native outcomes at every request boundary.
use super::engine_support as support;
use crate::runtime::profiling::{self, logical};
use crate::runtime::suspension::AbandonReason;
use crate::runtime::{RuntimeInputKind, Session, SessionRequestError, Symbol};
use logical::{AggregateKey, Counts, Retire};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

const PHASES: [&str; 4] = [
    "module.eval.total",
    "module.resume_initializers",
    "module.initializer.evaluate",
    "module.body.evaluate",
];
const OK: [u64; 10] = [1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
const ERROR: [u64; 10] = [1, 0, 1, 0, 0, 0, 0, 0, 0, 0];
const RETURN: [u64; 10] = [1, 0, 1, 0, 0, 1, 0, 0, 0, 0];
const THROW: [u64; 10] = [1, 0, 1, 0, 0, 0, 1, 0, 0, 0];

struct Case {
    name: &'static str,
    program: &'static str,
    prompts: &'static [&'static str],
    value: &'static str,
    // Initializers completed/open, ancestors open, bodies open at each pause.
    pauses: &'static [[u64; 4]],
    terminal: [[u64; 10]; 4],
    named: bool,
}

fn cases() -> Vec<Case> {
    let two = OK.map(|n| n * 2);
    vec![
        Case {
            name: "body",
            program: r#"Module[{x=3},(InputString["body"];x)]"#,
            prompts: &["body"],
            value: "3",
            pauses: &[[1, 0, 1, 1]],
            terminal: [OK; 4],
            named: false,
        },
        Case {
            name: "three",
            program: r#"Module[{x=(InputString["init1"];3),y=(InputString["init2"];4)},(InputString["body"];x+y)]"#,
            prompts: &["init1", "init2", "body"],
            value: "7",
            pauses: &[[0, 1, 1, 0], [1, 1, 1, 0], [2, 0, 1, 1]],
            terminal: [OK, OK, two, OK],
            named: false,
        },
        Case {
            name: "repeated",
            program: r#"Module[{x=(InputString["a"];InputString["b"];3),y=(InputString["c"];4)},(InputString["d"];InputString["e"];x+y)]"#,
            prompts: &["a", "b", "c", "d", "e"],
            value: "7",
            pauses: &[
                [0, 1, 1, 0],
                [0, 1, 1, 0],
                [1, 1, 1, 0],
                [2, 0, 1, 1],
                [2, 0, 1, 1],
            ],
            terminal: [OK, OK, two, OK],
            named: false,
        },
        Case {
            name: "source",
            program: r#"f[]:=Module[{x=(InputString["init1"];3),y=(InputString["init2"];4)},(InputString["body"];x+y)];f[]"#,
            prompts: &["init1", "init2", "body"],
            value: "7",
            pauses: &[[0, 1, 1, 0], [1, 1, 1, 0], [2, 0, 1, 1]],
            terminal: [OK, OK, two, OK],
            named: true,
        },
        Case {
            name: "pure-after-resume",
            program: r#"f[]:=(InputString["before"];Module[{x=3},x]);f[]"#,
            prompts: &["before"],
            value: "3",
            pauses: &[[0; 4]],
            terminal: [OK; 4],
            named: true,
        },
        Case {
            name: "nested",
            program: r#"Module[{x=(InputString["outer"];3)},Block[{b=4},Module[{y=(InputString["inner"];5)},(InputString["body"];x+y+b)]]]"#,
            prompts: &["outer", "inner", "body"],
            value: "12",
            pauses: &[[0, 1, 1, 0], [1, 1, 2, 1], [2, 0, 2, 2]],
            terminal: [two; 4],
            named: false,
        },
        Case {
            name: "initializer-abort",
            program: r#"Module[{x=(InputString["init"];Abort[])},9]"#,
            prompts: &["init"],
            value: "$Aborted",
            pauses: &[[0, 1, 1, 0]],
            terminal: [ERROR, ERROR, ERROR, [0; 10]],
            named: false,
        },
        Case {
            name: "function-return",
            program: r#"Function[{},Module[{x=3},(InputString["body"];Return[x])]][]"#,
            prompts: &["body"],
            value: "3",
            pauses: &[[1, 0, 1, 1]],
            terminal: [RETURN, RETURN, OK, RETURN],
            named: false,
        },
        Case {
            name: "throw",
            program: r#"Catch[Module[{x=3},(InputString["body"];Throw[x,t])],t]"#,
            prompts: &["body"],
            value: "3",
            pauses: &[[1, 0, 1, 1]],
            terminal: [THROW, THROW, OK, THROW],
            named: false,
        },
        Case {
            name: "cleanup-pauses",
            program: r#"Module[{x=3},WithCleanup[(InputString["body"];x),(InputString["cleanup"];Null)]]"#,
            prompts: &["body", "cleanup"],
            value: "3",
            pauses: &[[1, 0, 1, 1]; 2],
            terminal: [OK; 4],
            named: false,
        },
    ]
}

fn fields(c: Counts) -> [u64; 10] {
    [
        c.calls,
        c.ok,
        c.err,
        c.some,
        c.none,
        c.return_err,
        c.throw_err,
        c.cancelled,
        c.abandoned,
        c.open,
    ]
}

fn snapshot(case: &Case, at: &str, wanted: [[u64; 10]; 4], aliases: u64) {
    let snapshot = profiling::logical_profile_snapshot();
    let source = (case.named && snapshot.flags["EVALUATION_CACHE_PROFILE_ENABLED"] == 1)
        .then(|| Symbol::new("Global`", "f"));
    let actual = support::epoch()
        .rows()
        .into_iter()
        .filter_map(|(key, c)| {
            matches!(key, AggregateKey::Runtime { phase, .. } if PHASES.contains(&phase))
                .then_some((key, fields(c)))
        })
        .collect::<BTreeMap<_, _>>();
    let expected = PHASES
        .into_iter()
        .zip(wanted)
        .filter(|(_, c)| *c != [0; 10])
        .map(|(phase, c)| (AggregateKey::Runtime { phase, source }, c))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        actual, expected,
        "{} {at}: full native key union, ten fields",
        case.name
    );
    let projected = snapshot
        .logical
        .iter()
        .filter(|(k, _)| {
            PHASES
                .iter()
                .any(|p| k.starts_with(&format!("runtime|{p}|")))
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<BTreeMap<_, _>>();
    let expected_projection = actual
        .iter()
        .map(|(key, c)| {
            let AggregateKey::Runtime { phase, source } = key else {
                unreachable!()
            };
            (
                format!(
                    "runtime|{phase}|{}",
                    source.map(|s| s.to_string()).unwrap_or_else(|| "-".into())
                ),
                c[..7].to_vec(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        projected, expected_projection,
        "{} {at}: typed observer projection",
        case.name
    );
    assert!(snapshot.maps.contains_key("RUNTIME_PATH_TIMING_STATS"));
    assert_eq!(
        snapshot.logical["counter|eval_module"],
        vec![aliases],
        "{} {at}: scalar publication",
        case.name
    );
    if source.is_some() {
        let value = snapshot.logical.get("cache-source|eval_module_by_source|f");
        if aliases == 0 {
            assert!(value.is_none());
        } else {
            assert_eq!(value, Some(&vec![aliases]));
        }
    }
    assert_eq!(support::epoch().faults.load(Ordering::Relaxed), 0);
    eprintln!(
        "LP07 completion verdict: PASS {} {at} rows={actual:?} aliases={aliases}",
        case.name
    );
}

fn quiescent(session: &Session) {
    assert!(session.module_activations.is_empty());
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(Session::logical_profile_test_leases(), 0);
    assert_eq!(support::active(), 0);
    assert_eq!(support::epoch().faults.load(Ordering::Relaxed), 0);
}

#[test]
fn lp07_completion_rows_and_source_identity() {
    let enabled = support::enabled();
    for case in cases() {
        for queued in [true, false] {
            profiling::reset_lexical_profile_counters();
            crate::runtime::reset_evaluation_cache_profile_counters();
            logical::epoch::reset_for_lease(enabled.lease());
            let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
            crate::runtime::test_builtins::register_test_arithmetic_builtins(&mut session);
            if queued {
                for prompt in case.prompts {
                    session.queue_input_response(*prompt);
                }
            }
            let mut result = session.eval_request(case.program);
            if !queued {
                for (index, prompt) in case.prompts.iter().enumerate() {
                    assert!(matches!(&result.as_ref().unwrap_err().error,
                        SessionRequestError::InputRequired { request }
                        if request.kind == RuntimeInputKind::InputString && request.prompt == *prompt));
                    let [done, init_open, ancestors, body_open] = case.pauses[index];
                    let mut rows = [[0; 10]; 4];
                    rows[0][9] = ancestors;
                    rows[1][9] = ancestors;
                    rows[2][0] = done;
                    rows[2][1] = done;
                    rows[2][9] = init_open;
                    rows[3][9] = body_open;
                    snapshot(&case, &format!("pause{}", index + 1), rows, 0);
                    assert_eq!(session.module_activations.len() as u64, ancestors);
                    result = session.resume_eval_request_with_input(*prompt);
                }
            }
            assert_eq!(result.unwrap().value.to_string(), case.value);
            snapshot(
                &case,
                if queued { "queued" } else { "resumed" },
                case.terminal,
                case.terminal[0][0],
            );
            quiescent(&session);
        }
    }
}

#[test]
fn lp07_lifecycle_reasons_scopes_and_storage() {
    for mode in [
        "cancel",
        "discard",
        "drop",
        "reset",
        "cleanup-discard",
        "reset-body",
        "reset-init",
    ] {
        let enabled = support::enabled();
        profiling::reset_lexical_profile_counters();
        crate::runtime::reset_evaluation_cache_profile_counters();
        let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
        let program = match mode {
            "reset-init" => r#"f[]:=Module[{x=(InputString["init"];3)},x];f[]"#,
            "cleanup-discard" => {
                r#"f[]:=Module[{x=3},WithCleanup[(InputString["body"];x),(InputString["cleanup"];Null)]];f[]"#
            }
            _ => r#"f[]:=Module[{x=3},(InputString["body"];x)];f[]"#,
        };
        let first = session.eval_request(program).unwrap_err();
        assert!(
            matches!(first.error, SessionRequestError::InputRequired { request }
            if request.prompt == if mode == "reset-init" { "init" } else { "body" })
        );
        if mode == "cleanup-discard" {
            assert!(
                matches!(session.resume_eval_request_with_input("body").unwrap_err().error,
                SessionRequestError::InputRequired { request } if request.prompt == "cleanup")
            );
        }
        let gensyms = session
            .module_activations
            .iter()
            .flat_map(|a| a.gensyms.iter().copied())
            .collect::<Vec<_>>();
        assert!(!gensyms.is_empty());
        let old = support::epoch();
        let mut session = Some(session);
        let reset = mode.starts_with("reset");
        if reset {
            profiling::reset_lexical_profile_counters();
            crate::runtime::reset_evaluation_cache_profile_counters();
        }
        match mode {
            "drop" => drop(session.take()),
            "reset-body" | "reset-init" => {
                assert_eq!(
                    session
                        .as_mut()
                        .unwrap()
                        .resume_eval_request_with_input("answer")
                        .unwrap()
                        .value
                        .to_string(),
                    "3"
                );
            }
            _ => {
                let reason = if mode == "cancel" {
                    AbandonReason::CancelledRequest
                } else {
                    AbandonReason::HelperBestEffortDiscard
                };
                session.as_mut().unwrap().abandon_pending_suspension(reason);
                let once = old
                    .rows()
                    .into_iter()
                    .map(|(k, c)| (k, fields(c)))
                    .collect::<Vec<_>>();
                session.as_mut().unwrap().abandon_pending_suspension(reason);
                assert_eq!(
                    once,
                    old.rows()
                        .into_iter()
                        .map(|(k, c)| (k, fields(c)))
                        .collect::<Vec<_>>()
                );
            }
        }
        if let Some(session) = &session {
            for symbol in gensyms {
                assert!(!session.definitions.contains_key(&symbol));
            }
            quiescent(session);
        }
        old.assert_quiescent();
        assert_eq!(old.faults.load(Ordering::Relaxed), 0);
        assert_eq!(old.contexts.load(Ordering::Relaxed), 0);
        for (key, c) in old.rows() {
            if !matches!(key, AggregateKey::Runtime { phase, .. } if PHASES.contains(&phase)) {
                continue;
            }
            assert_eq!(c.calls, 1);
            assert_eq!(c.open, 0);
            if c.ok == 1 {
                continue;
            } // An initializer completed before the body pause.
            if mode == "cancel" {
                assert_eq!(fields(c), [1, 0, 1, 0, 0, 0, 0, 1, 0, 0]);
                assert_eq!(c.abandonment_reasons, [0; 11]);
            } else {
                let reason = if reset {
                    Retire::ProfileReset
                } else if mode == "drop" {
                    Retire::SessionDrop
                } else {
                    Retire::Structural(AbandonReason::HelperBestEffortDiscard)
                };
                let mut reasons = [0; 11];
                reasons[reason.index()] = 1;
                assert_eq!(c.abandonment_reasons, reasons, "{mode}: {key:?}");
                assert_eq!(fields(c), [1, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
            }
        }
        if reset {
            let snapshot = profiling::logical_profile_snapshot();
            assert_eq!(snapshot.logical["counter|eval_module"], vec![0]);
            assert!(
                !snapshot
                    .logical
                    .contains_key("cache-source|eval_module_by_source|f")
            );
        }
        drop(session);
        drop(old);
        drop(enabled);
        // The test lease releases its TLS header before the zero-byte drain.
        let _lease = logical::epoch::lease_drained();
        eprintln!(
            "LP07 lifecycle verdict: PASS {mode} open=0 leases=0 faults=0 retained_diagnostic_bytes=0"
        );
    }
}
