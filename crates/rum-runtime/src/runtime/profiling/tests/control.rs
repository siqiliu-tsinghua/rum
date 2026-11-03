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
