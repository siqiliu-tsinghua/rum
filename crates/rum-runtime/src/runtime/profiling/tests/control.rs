use super::{AggregateKey, test_support};
use crate::runtime::{
    BuiltinSpec, EvalError, Expr, MemoryGrowthContract, RuntimeInputKind, Session, SharedArgs,
    expr_as_exact_integer, system_expr,
};
use std::collections::BTreeMap;

const WHILE: &[&str] = &[
    "while.test.evaluate",
    "while.test.suspend",
    "while.body.evaluate",
    "while.body.suspend",
];
const COMPOUND: &[&str] = &["compound.item.evaluate", "compound.item.suspend"];
const FUNCTION: &[&str] = &["function.apply.evaluate"];
const OPEN: [u64; 10] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
const OK: [u64; 10] = [1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
const ERROR: [u64; 10] = [1, 0, 1, 0, 0, 0, 0, 0, 0, 0];
const RETURN: [u64; 10] = [1, 0, 1, 0, 0, 1, 0, 0, 0, 0];
const THROW: [u64; 10] = [1, 0, 1, 0, 0, 0, 1, 0, 0, 0];
const CATCH: &[&str] = &[
    "catch.eval.total",
    "catch.body.evaluate",
    "catch.resume_after_body",
];

struct BuiltinEnabled(u8);
impl BuiltinEnabled {
    fn new() -> Self {
        Self(
            super::super::state::BUILTIN_TIMING_ENABLED
                .swap(1, std::sync::atomic::Ordering::Relaxed),
        )
    }
}
impl Drop for BuiltinEnabled {
    fn drop(&mut self) {
        super::super::state::BUILTIN_TIMING_ENABLED
            .store(self.0, std::sync::atomic::Ordering::Relaxed);
    }
}

// Finite runtime doubles; the oracle and standalone package use the stdlib handlers.
fn integer_greater(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let [left, right] = args.as_ref() else {
        return Ok(Expr::call(head.clone(), args.as_ref().to_vec()));
    };
    let (Some(left), Some(right)) = (expr_as_exact_integer(left), expr_as_exact_integer(right))
    else {
        return Ok(Expr::call(head.clone(), args.as_ref().to_vec()));
    };
    Ok(system_expr(if left > right { "True" } else { "False" }))
}

fn call_length(_session: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    match args.as_ref() {
        [Expr::Call { args, .. }] => Ok(Expr::small_integer(args.len() as i64)),
        _ => Ok(Expr::call(head.clone(), args.as_ref().to_vec())),
    }
}

fn a2_snapshot(
    id: &str,
    at: &str,
    keys: &[AggregateKey],
    counts: &[[u64; 10]],
) -> BTreeMap<String, [u64; 10]> {
    let all = Session::logical_profile_test_rows()
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let actual = test_support::epoch()
        .rows()
        .into_iter()
        .filter(|(key, _)| {
            keys.iter().any(|selected| match (key, selected) {
                (
                    AggregateKey::Runtime { phase: a, .. },
                    AggregateKey::Runtime { phase: b, .. },
                ) => a == b,
                (
                    AggregateKey::Builtin { symbol: a, .. },
                    AggregateKey::Builtin { symbol: b, .. },
                ) => a == b,
                _ => false,
            })
        })
        .map(|(key, _)| {
            let key = format!("{key:?}");
            let row = all[&key];
            (key, row)
        })
        .collect::<BTreeMap<_, _>>();
    let expected = keys
        .iter()
        .zip(counts)
        .filter(|(_, row)| **row != [0; 10])
        .map(|(key, row)| (format!("{key:?}"), *row))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        actual, expected,
        "{id} {at}: FULL keys and calls/ok/err/some/none/return/throw/cancelled/abandoned/open"
    );
    assert_eq!(
        test_support::epoch()
            .faults
            .load(std::sync::atomic::Ordering::Relaxed),
        0,
        "{id} {at}: instrumentation fault"
    );
    eprintln!("LP06 A2 NATIVE {id} {at}: SAME {actual:?}; faults=0");
    actual
}

fn a2_cell(
    id: &str,
    source: &str,
    prompts: &[&str],
    value: Option<&str>,
    builtin: Option<(&'static str, u16)>,
    pauses: &[&[[u64; 10]]],
    terminal: &[[u64; 10]],
) {
    let enabled = test_support::enabled();
    let _builtin_enabled = BuiltinEnabled::new();
    let keys = match builtin {
        Some((name, arity)) => vec![AggregateKey::Builtin {
            symbol: crate::runtime::Symbol::new("System`", name),
            source: None,
            arity,
        }],
        None => CATCH
            .iter()
            .map(|phase| AggregateKey::Runtime {
                phase,
                source: None,
            })
            .collect(),
    };
    let mut completed = None;
    let mut completed_output = None;
    assert_eq!(prompts.len(), pauses.len());
    for queued in [true, false] {
        let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
        crate::runtime::test_builtins::register_test_arithmetic_builtins(&mut session);
        session.register_builtin(BuiltinSpec::ordinary(
            "Greater",
            "Finite integer comparison.",
            integer_greater,
            MemoryGrowthContract::NonAmplifying,
        ));
        session.register_builtin(BuiltinSpec::ordinary(
            "Length",
            "Call argument count.",
            call_length,
            MemoryGrowthContract::NonAmplifying,
        ));
        super::super::reset_lexical_profile_counters();
        super::epoch::reset_for_lease(enabled.lease());
        let probe = session.continuation_fault_probe();
        let faults = probe.occurrences();
        if queued {
            for answer in ["a", "b"].into_iter().take(prompts.len()) {
                session.queue_input_response(answer);
            }
        }
        let mut result = session.eval_request(source);
        if !queued {
            for (index, (prompt, expected)) in prompts.iter().zip(pauses).enumerate() {
                match &result {
                    Err(failure) => match &failure.error {
                        crate::runtime::SessionRequestError::InputRequired { request } => {
                            assert_eq!(request.prompt, *prompt, "{id} request {}", index + 1);
                            assert_eq!(request.kind, RuntimeInputKind::InputString);
                        }
                        other => panic!("{id} missed request {}: {other:?}", index + 1),
                    },
                    other => panic!("{id} missed request {}: {other:?}", index + 1),
                }
                a2_snapshot(id, prompt, &keys, expected);
                result = session.resume_eval_request_with_input(["a", "b"][index]);
            }
        }
        if let Some(value) = value {
            assert_eq!(result.as_ref().unwrap().value.to_string(), value, "{id}");
        } else {
            let error = &result.as_ref().unwrap_err().error;
            let expected = if id == "lp06.catch.bypass_miss" {
                "uncaught throw: 1 with tag t"
            } else {
                "uncaught Abort[]"
            };
            assert_eq!(
                error,
                &crate::runtime::SessionRequestError::Other {
                    message: expected.into()
                },
                "{id}"
            );
        }
        let rows = a2_snapshot(id, "terminal", &keys, terminal);
        Session::logical_profile_test_assert_quiescent();
        assert_eq!(test_support::active(), 0);
        assert_eq!(probe.occurrences(), faults, "{id}: continuation fault");
        if let Some(expected) = &completed {
            assert_eq!(expected, &rows, "{id}: queued/resumed full key map");
        } else {
            completed = Some(rows);
        }
        if let Some(expected) = &completed_output {
            assert_eq!(expected, &result, "{id}: queued/resumed guest outcome");
        } else {
            completed_output = Some(result);
        }
    }
}

#[test]
fn lp06_a2_builtin_no_handler() {
    let _enabled = test_support::enabled();
    let _builtin_enabled = BuiltinEnabled::new();
    let mut session = Session::new();
    let head = system_expr("Length");
    let args = SharedArgs::from(Vec::new());
    for spec in [
        None,
        Some(BuiltinSpec::new(
            "Length",
            "No handler.",
            MemoryGrowthContract::NonAmplifying,
        )),
    ] {
        assert!(matches!(
            session.try_dispatch_builtin_call(spec, &head, &args),
            Ok(None)
        ));
        assert!(
            test_support::epoch()
                .rows()
                .iter()
                .all(|(key, _)| !matches!(key, AggregateKey::Builtin { .. }))
        );
    }
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(
        test_support::epoch()
            .faults
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    eprintln!(
        "LP06 A2 NATIVE lp06.builtin.no_handler: SAME spec=None handler=None; builtin keys=0 faults=0"
    );
}

#[test]
fn lp06_c1_builtin_gate_transition() {
    use super::super::state::LEXICAL_PROFILE_ACTIVE;
    use std::sync::atomic::Ordering;

    let enabled = test_support::enabled();
    let _builtin_enabled = BuiltinEnabled::new();
    let mut session = Session::new();
    let symbol = crate::runtime::Symbol::new("System`", "Length");
    for (disable, expected) in [(false, (1, 0, 1, 0)), (true, (1, 0, 0, 1))] {
        super::epoch::reset_for_lease(enabled.lease());
        // The builtin starter charges its record to the executing parent epoch.
        let parent = session
            .logical_phase_start(
                super::PhaseKey::EvaluateCallDispatch.name(),
                super::Policy::Expr,
            )
            .unwrap();
        let token = session.builtin_timing_start_adopted(&symbol, 1).unwrap();
        let prior = if disable {
            Some(LEXICAL_PROFILE_ACTIVE.swap(2, Ordering::Relaxed))
        } else {
            None
        };
        session.record_profile_builtin_timing_adopted_boundary(
            Some(token),
            &Ok(Expr::small_integer(2)),
        );
        if let Some(prior) = prior {
            LEXICAL_PROFILE_ACTIVE.store(prior, Ordering::Relaxed);
        }
        parent.finish(super::Outcome::Ok);
        let rows = test_support::epoch()
            .rows()
            .into_iter()
            .filter(|(key, _)| matches!(key, AggregateKey::Builtin { .. }))
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].0,
            AggregateKey::Builtin {
                symbol,
                source: None,
                arity: 1,
            }
        );
        let counts = rows[0].1;
        assert_eq!(
            (counts.calls, counts.open, counts.ok, counts.abandoned),
            expected
        );
        Session::logical_profile_test_assert_quiescent();
        assert_eq!(test_support::active(), 0);
        eprintln!(
            "LP06 C1 NATIVE gate={} PASS calls={} open={} ok={} abandoned={} quiescent active=0",
            if disable { "disabled" } else { "enabled" },
            counts.calls,
            counts.open,
            counts.ok,
            counts.abandoned,
        );
    }
}

#[test]
fn lp06_a2_incoming_policy_cases() {
    use super::super::RuntimeProfileOutcome;
    let enabled = test_support::enabled();
    let mut session = Session::new();
    for (incoming, legacy, expected) in [
        (Ok(Expr::small_integer(1)), RuntimeProfileOutcome::Ok, OK),
        (Err(EvalError::Abort), RuntimeProfileOutcome::Error, ERROR),
        (
            Err(EvalError::Return {
                value: Expr::small_integer(1),
            }),
            RuntimeProfileOutcome::Return,
            RETURN,
        ),
        (
            Err(EvalError::Thrown {
                value: Expr::small_integer(1),
                tag: system_expr("None"),
                explicit_tag: false,
            }),
            RuntimeProfileOutcome::Throw,
            THROW,
        ),
        (
            Err(EvalError::Cancelled),
            RuntimeProfileOutcome::Error,
            [1, 0, 1, 0, 0, 0, 0, 1, 0, 0],
        ),
    ] {
        super::epoch::reset_for_lease(enabled.lease());
        let token = session.runtime_path_timing_start_adopted_incoming(CATCH[2], &incoming);
        session.record_profile_runtime_path_timing_adopted_incoming(
            token,
            legacy,
            &Ok(Expr::small_integer(9)),
        );
        a2_snapshot(
            "lp06.catch.policy",
            "terminal",
            &[AggregateKey::Runtime {
                phase: CATCH[2],
                source: None,
            }],
            &[expected],
        );
        Session::logical_profile_test_assert_quiescent();
    }
}

// The runtime test session uses the same finite integer comparison double as
// definitions_rhs_guards; guest cells exercise stdlib's actual Less handler.
fn integer_less(_session: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let [left, right] = args.as_ref() else {
        return Ok(Expr::call(head.clone(), args.as_ref().to_vec()));
    };
    let (Some(left), Some(right)) = (expr_as_exact_integer(left), expr_as_exact_integer(right))
    else {
        return Ok(Expr::call(head.clone(), args.as_ref().to_vec()));
    };
    Ok(system_expr(if left < right { "True" } else { "False" }))
}

fn snapshot(
    id: &str,
    at: &str,
    selected: &[&str],
    expected: &[[u64; 10]],
) -> BTreeMap<String, [u64; 10]> {
    assert_eq!(selected.len(), expected.len());
    let rows = Session::logical_profile_test_rows();
    let mut actual = BTreeMap::new();
    for (phase, counts) in selected.iter().zip(expected) {
        let matching = rows
            .iter()
            .filter(|(key, _)| key.contains(&format!("phase: \"{phase}\"")))
            .collect::<Vec<_>>();
        if *counts == [0; 10] {
            assert!(
                matching.is_empty(),
                "{id} {at}: unexpected {phase}: {matching:?}"
            );
        } else {
            assert_eq!(
                matching.len(),
                1,
                "{id} {at}: required full key {phase}: {matching:?}"
            );
            let (key, row) = matching[0];
            assert_eq!(
                row, counts,
                "{id} {at} {key}: calls/ok/err/some/none/return/throw/cancelled/abandoned/open"
            );
            actual.insert(key.clone(), *row);
        }
    }
    let legacy = crate::test_logical_profile_snapshot();
    for (key, _) in test_support::epoch().rows() {
        if let AggregateKey::Runtime { phase, source } = key {
            if selected.contains(&phase) {
                let source = source.map_or_else(|| "-".into(), |s| s.to_string());
                assert!(
                    legacy
                        .logical
                        .contains_key(&format!("runtime|{phase}|{source}")),
                    "{id} {at}: source differs from the legacy invocation {key:?}"
                );
            }
        }
    }
    assert_eq!(
        test_support::epoch()
            .faults
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    actual
}

fn cell(
    id: &str,
    setup: &str,
    source: &str,
    prompts: &[&str],
    value: &str,
    selected: &[&'static str],
    pauses: &[&[[u64; 10]]],
    terminal: &[[u64; 10]],
) {
    let enabled = test_support::enabled();
    let mut completed = None;
    assert_eq!(prompts.len(), pauses.len());
    for queued in [true, false] {
        let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
        crate::runtime::test_builtins::register_test_arithmetic_builtins(&mut session);
        session.register_builtin(BuiltinSpec::ordinary(
            "Less",
            "Test-local exact-integer comparison.",
            integer_less,
            MemoryGrowthContract::NonAmplifying,
        ));
        if !setup.is_empty() {
            session.eval_text(setup).unwrap();
        }
        super::super::reset_lexical_profile_counters();
        super::epoch::reset_for_lease(enabled.lease());
        let faults = session.continuation_fault_probe();
        let before = faults.occurrences();
        if queued {
            for answer in ["a", "b"].into_iter().take(prompts.len()) {
                session.queue_input_response(answer);
            }
        }
        let mut result = session.eval_text(source).map(|r| r.value);
        if !queued {
            for (index, (prompt, expected)) in prompts.iter().zip(pauses).enumerate() {
                match &result {
                    Err(EvalError::InputRequired { request }) => {
                        assert_eq!(&request.prompt, prompt, "{id} request {}", index + 1);
                        assert_eq!(request.kind, RuntimeInputKind::InputString);
                    }
                    other => panic!("{id} missed request {} ({prompt}): {other:?}", index + 1),
                }
                snapshot(id, prompt, selected, expected);
                eprintln!(
                    "LP06 NATIVE {id} request={}: SAME open/calls/terminal",
                    index + 1
                );
                result = session.resume_suspended_evaluation(["a", "b"][index]);
            }
        }
        match result {
            Ok(actual) => assert_eq!(actual.to_string(), value, "{id} queued={queued}"),
            other => panic!("{id} queued={queued}: {other:?}"),
        }
        let rows = snapshot(id, "terminal", selected, terminal);
        for (index, phase) in selected.iter().enumerate() {
            let observed = Session::logical_profile_test_pauses_for(phase);
            let spans_pause = pauses.iter().any(|p| p[index][9] > 0);
            if !queued && spans_pause {
                assert!(observed > 0, "{id} {phase}: missing pause telemetry");
            } else {
                assert_eq!(observed, 0, "{id} {phase}: unexpected pause telemetry");
            }
        }
        Session::logical_profile_test_assert_quiescent();
        assert_eq!(test_support::active(), 0);
        assert_eq!(faults.occurrences(), before, "{id}: continuation fault");
        if let Some(queued_rows) = &completed {
            assert_eq!(queued_rows, &rows, "{id}: complete selected key/source map");
        } else {
            completed = Some(rows);
        }
        if id == "lp06.while.body.continue" {
            assert_eq!(session.eval_text("n").unwrap().value.to_string(), "2");
        }
        eprintln!("LP06 NATIVE {id} queued={queued}: SAME terminal; open=0 abandoned=0 faults=0");
    }
}

#[test]
fn lp06_while_test_once() {
    cell(
        "lp06.while.test.once",
        r#""#,
        r#"While[(InputString["test"];False),Null]"#,
        &["test"],
        "Null",
        WHILE,
        &[&[OPEN, OPEN, [0; 10], [0; 10]]],
        &[OK, OK, [0; 10], [0; 10]],
    );
}

#[test]
fn lp06_while_test_twice() {
    cell(
        "lp06.while.test.twice",
        r#""#,
        r#"While[(InputString["test1"];InputString["test2"];False),Null]"#,
        &["test1", "test2"],
        "Null",
        WHILE,
        &[
            &[OPEN, OPEN, [0; 10], [0; 10]],
            &[OPEN, OPEN, [0; 10], [0; 10]],
        ],
        &[OK, OK, [0; 10], [0; 10]],
    );
}

#[test]
fn lp06_while_body_break() {
    cell(
        "lp06.while.body.break",
        r#""#,
        r#"While[True,(InputString["body"];Break[])]"#,
        &["body"],
        "Null",
        WHILE,
        &[&[OK, OK, OPEN, OPEN]],
        &[OK, OK, ERROR, ERROR],
    );
}

#[test]
fn lp06_while_body_continue() {
    cell(
        "lp06.while.body.continue",
        r#"n=0"#,
        r#"While[n<2,(n++;InputString["body"];Continue[])]"#,
        &["body", "body"],
        "Null",
        WHILE,
        &[
            &[OK, OK, OPEN, OPEN],
            &[
                [2, 2, 0, 0, 0, 0, 0, 0, 0, 0],
                [2, 2, 0, 0, 0, 0, 0, 0, 0, 0],
                [1, 0, 1, 0, 0, 0, 0, 0, 0, 1],
                [1, 0, 1, 0, 0, 0, 0, 0, 0, 1],
            ],
        ],
        &[
            [3, 3, 0, 0, 0, 0, 0, 0, 0, 0],
            [3, 3, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 0, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 0, 0, 0, 0],
        ],
    );
}

#[test]
fn lp06_compound_items() {
    cell(
        "lp06.compound.items",
        r#""#,
        r#"InputString["first"];InputString["second"]"#,
        &["first", "second"],
        "\"b\"",
        COMPOUND,
        &[
            &[OPEN, OPEN],
            &[
                [1, 1, 0, 0, 0, 0, 0, 0, 0, 1],
                [1, 1, 0, 0, 0, 0, 0, 0, 0, 1],
            ],
        ],
        &[
            [2, 2, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 2, 0, 0, 0, 0, 0, 0, 0, 0],
        ],
    );
}

#[test]
fn lp06_compound_next_throw() {
    cell(
        "lp06.compound.next_throw",
        r#""#,
        r#"Catch[(InputString["first"];Throw[7])]"#,
        &["first"],
        "7",
        COMPOUND,
        &[&[OPEN, OPEN]],
        &[
            [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
            [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
        ],
    );
}

#[test]
fn lp06_function_return_twice() {
    cell(
        "lp06.function.return.twice",
        r#""#,
        r#"Function[{},(InputString["one"];InputString["two"];Return[7])][]"#,
        &["one", "two"],
        "7",
        FUNCTION,
        &[&[OPEN], &[OPEN]],
        &[RETURN],
    );
}

#[test]
fn lp06_function_ok() {
    cell(
        "lp06.function.ok",
        r#""#,
        r#"Function[{},(InputString["value"];19)][]"#,
        &["value"],
        "19",
        FUNCTION,
        &[&[OPEN]],
        &[OK],
    );
}

#[test]
fn lp06_status_projection_value() {
    cell(
        "lp06.status.projection_value",
        r#""#,
        r#"Failure["t",<|"MessageTemplate":>(InputString["value"];19),"MessageParameters":>(InputString["projection"];{})|>]["Message"]"#,
        &["projection", "value"],
        "19",
        FUNCTION,
        &[&[[0; 10]], &[OPEN]],
        &[OK],
    );
}

#[test]
fn lp06_status_projection_return() {
    cell(
        "lp06.status.projection_return",
        r#"f[]:=(Failure["t",<|"MessageTemplate"->"x","MessageParameters":>(InputString["w"];Return[3])|>]["Message"];7)"#,
        r#"f[]"#,
        &["w"],
        "3",
        FUNCTION,
        &[&[[0; 10]]],
        &[[0; 10]],
    );
}

#[test]
fn lp06_status_value_return() {
    cell(
        "lp06.status.value_return",
        r#""#,
        r#"f[]:=(Failure["t",<|"MessageTemplate":>(InputString["value"];Return[3]),"MessageParameters":>(InputString["projection"];{})|>]["Message"];7);f[]"#,
        &["projection", "value"],
        "7",
        FUNCTION,
        &[&[[0; 10]], &[OPEN]],
        &[RETURN],
    );
}

#[test]
fn lp06_status_raw_failure() {
    cell(
        "lp06.status.raw_failure",
        r#""#,
        r#"Failure["t",<|"x":>(InputString["value"];17)|>]["x"]"#,
        &["value"],
        "17",
        FUNCTION,
        &[&[OPEN]],
        &[OK],
    );
}

#[test]
fn lp06_status_raw_success() {
    cell(
        "lp06.status.raw_success",
        r#""#,
        r#"Success["t",<|"x":>(InputString["value"];17)|>]["x"]"#,
        &["value"],
        "17",
        FUNCTION,
        &[&[OPEN]],
        &[OK],
    );
}

#[test]
fn lp06_a2_builtin_ok() {
    a2_cell(
        "lp06.builtin.ok",
        r###"Length[{1,2}]"###,
        &[],
        Some("2"),
        Some(("Length", 1)),
        &[],
        &[OK],
    );
}

#[test]
fn lp06_a2_builtin_cleanup_twice() {
    a2_cell(
        "lp06.builtin.cleanup.twice",
        r###"WithCleanup[(InputString["primary"];7),(InputString["cleanup"];Null)]"###,
        &["primary", "cleanup"],
        Some("7"),
        Some(("WithCleanup", 2)),
        &[&[OPEN], &[OPEN]],
        &[OK],
    );
}

#[test]
fn lp06_a2_builtin_cleanup_throw() {
    a2_cell(
        "lp06.builtin.cleanup.throw",
        r###"Catch[WithCleanup[Throw[7,t],(InputString["cleanup"];Null)],t]"###,
        &["cleanup"],
        Some("7"),
        Some(("WithCleanup", 2)),
        &[&[OPEN]],
        &[THROW],
    );
}

#[test]
fn lp06_a2_builtin_return() {
    a2_cell(
        "lp06.builtin.return",
        r###"Function[{},Return[7]][]"###,
        &[],
        Some("7"),
        Some(("Return", 1)),
        &[],
        &[RETURN],
    );
}

#[test]
fn lp06_a2_catch_body() {
    a2_cell(
        "lp06.catch.body",
        r###"Catch[(InputString["p"];Throw[1])]"###,
        &["p"],
        Some("1"),
        None,
        &[&[OPEN, OPEN, [0; 10]]],
        &[OK, THROW, THROW],
    );
}

#[test]
fn lp06_a2_catch_handler_twice() {
    a2_cell(
        "lp06.catch.handler.twice",
        r###"Catch[(InputString["body"];Throw[7,t]),t,Function[{v,k},(InputString["handler"];v+1)]]"###,
        &["body", "handler"],
        Some("8"),
        None,
        &[&[OPEN, OPEN, [0; 10]], &[OPEN, THROW, OPEN]],
        &[OK, THROW, THROW],
    );
}

#[test]
fn lp06_a2_catch_handler_error() {
    a2_cell(
        "lp06.catch.handler_error",
        r###"Catch[(InputString["body"];Throw[7,t]),t,Function[{v,k},(InputString["handler"];Abort[])]]"###,
        &["body", "handler"],
        Some("$Aborted"),
        None,
        &[&[OPEN, OPEN, [0; 10]], &[OPEN, THROW, OPEN]],
        &[ERROR, THROW, THROW],
    );
}

#[test]
fn lp06_a2_catch_form() {
    a2_cell(
        "lp06.catch.form",
        r###"Catch[Throw[1,t],(InputString["f"];t)]"###,
        &["f"],
        Some("1"),
        None,
        &[&[OPEN, THROW, OPEN]],
        &[OK, THROW, THROW],
    );
}

#[test]
fn lp06_a2_catch_match() {
    a2_cell(
        "lp06.catch.match",
        r###"Catch[Throw[1,t[2]],t[x_]/;(InputString["g"];x>1)]"###,
        &["g"],
        Some("1"),
        None,
        &[&[OPEN, THROW, OPEN]],
        &[OK, THROW, THROW],
    );
}

#[test]
fn lp06_a2_catch_match_miss() {
    a2_cell(
        "lp06.catch.match_miss",
        r###"Catch[Catch[Throw[1,t[2]],t[x_]/;(InputString["g"];x>5)],_]"###,
        &["g"],
        Some("1"),
        None,
        &[&[
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
            [1, 0, 1, 0, 0, 0, 1, 0, 0, 1],
            OPEN,
        ]],
        &[
            [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
        ],
    );
}

#[test]
fn lp06_a2_catch_bypass_form() {
    a2_cell(
        "lp06.catch.bypass_form",
        r###"Catch[Catch[WithCleanup[Throw[1,t],Throw[2,t]],(InputString["f"];t)],t]"###,
        &["f"],
        Some("1"),
        None,
        &[&[
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
            [1, 0, 1, 0, 0, 0, 1, 0, 0, 1],
            OPEN,
        ]],
        &[
            [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
        ],
    );
}

#[test]
fn lp06_a2_catch_bypass_match() {
    a2_cell(
        "lp06.catch.bypass_match",
        r###"Catch[Catch[WithCleanup[Throw[1,t[1]],Throw[2,t[1]]],t[x_]/;(InputString["g"];True)],_]"###,
        &["g"],
        Some("1"),
        None,
        &[&[
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
            [1, 0, 1, 0, 0, 0, 1, 0, 0, 1],
            OPEN,
        ]],
        &[
            [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
        ],
    );
}

#[test]
fn lp06_a2_catch_ok() {
    a2_cell(
        "lp06.catch.ok",
        r###"Catch[(InputString["body"];19)]"###,
        &["body"],
        Some("19"),
        None,
        &[&[OPEN, OPEN, [0; 10]]],
        &[OK, OK, OK],
    );
}

#[test]
fn lp06_a2_catch_invalid_arity() {
    a2_cell(
        "lp06.catch.invalid_arity",
        r###"Catch[]"###,
        &[],
        Some("Catch[]"),
        None,
        &[],
        &[OK, [0; 10], [0; 10]],
    );
}

#[test]
fn lp06_a2_catch_return() {
    a2_cell(
        "lp06.catch.return",
        r###"Function[{},Catch[(InputString["body"];Return[7])]][]"###,
        &["body"],
        Some("7"),
        None,
        &[&[OPEN, OPEN, [0; 10]]],
        &[RETURN, RETURN, RETURN],
    );
}

#[test]
fn lp06_a2_catch_bypass_no_form() {
    a2_cell(
        "lp06.catch.bypass_no_form",
        r###"Catch[Catch[WithCleanup[Throw[1,t],(InputString["cleanup"];Throw[2,t])]],t]"###,
        &["cleanup"],
        Some("1"),
        None,
        &[&[
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
            [0; 10],
        ]],
        &[
            [2, 1, 1, 0, 0, 0, 1, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
        ],
    );
}

#[test]
fn lp06_a2_catch_bypass_miss() {
    a2_cell(
        "lp06.catch.bypass_miss",
        r###"Catch[Catch[WithCleanup[Throw[1,t],Throw[2,t]],(InputString["f"];u)],t]"###,
        &["f"],
        None,
        None,
        &[&[
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
            [1, 0, 1, 0, 0, 0, 1, 0, 0, 1],
            OPEN,
        ]],
        &[
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
            [2, 0, 2, 0, 0, 0, 2, 0, 0, 0],
        ],
    );
}

#[test]
fn lp06_a2_catch_match_error() {
    a2_cell(
        "lp06.catch.match_error",
        r###"Catch[Throw[1,t[2]],t[x_]/;(InputString["g"];Abort[])]"###,
        &["g"],
        Some("$Aborted"),
        None,
        &[&[OPEN, THROW, OPEN]],
        &[ERROR, THROW, THROW],
    );
}
