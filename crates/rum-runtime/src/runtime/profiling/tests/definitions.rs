use crate::runtime::profiling::logical::test_support;
use crate::runtime::{EvalError, Session};

fn row(rows: &[(String, [u64; 10])], field: &str, name: &str) -> [u64; 10] {
    rows.iter()
        .find(|(key, _)| key.contains(&format!("{field}: \"{name}\"")))
        .map(|(_, counts)| *counts)
        .unwrap_or_else(|| panic!("missing {field} {name}: {rows:?}"))
}

fn evaluate(session: &mut Session, source: &str) -> String {
    match session.eval_text(source) {
        Ok(result) => result.value.to_string(),
        Err(error) => panic!("{source}: {error:?}"),
    }
}

fn assert_closed(rows: &[(String, [u64; 10])]) {
    for (key, counts) in rows {
        assert_eq!(counts[0], counts[1] + counts[2], "{key}: {counts:?}");
        assert_eq!(counts[8], 0, "{key}: {counts:?}");
        assert_eq!(counts[9], 0, "{key}: {counts:?}");
    }
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(test_support::active(), 0);
}

fn assert_rule_row(
    rows: &[(String, [u64; 10])],
    kind: &'static str,
    symbol: &str,
    expected: [u64; 10],
) {
    let key = format!(
        "{:?}",
        super::AggregateKey::Definition {
            kind,
            symbol: crate::runtime::Symbol::new("Global`", symbol),
            index: 0,
        }
    );
    let Some((_, counts)) = rows.iter().find(|(actual, _)| *actual == key) else {
        panic!("missing {key}: {rows:?}");
    };
    assert_eq!(*counts, expected, "{key}");
}

#[test]
fn lp04_rhs_completed_down_value() {
    let mut session = Session::new();
    evaluate(&mut session, "f[x_] := {x}");
    let _enabled = test_support::enabled();
    assert_eq!(evaluate(&mut session, "f[1]"), "List[1]");
    let rows = Session::logical_profile_test_rows();
    for phase in ["definition.rhs.total", "definition.rhs.evaluate"] {
        assert_eq!(row(&rows, "phase", phase), [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    }
    assert_rule_row(
        &rows,
        "down_value_rhs_eval",
        "f",
        [1, 1, 0, 1, 0, 0, 0, 0, 0, 0],
    );
    assert_closed(&rows);
}

#[test]
fn lp04_rhs_paused_and_resumed_down_value() {
    let mut session = Session::new();
    evaluate(&mut session, "f[x_] := InputString[\"rhs\"]");
    let _enabled = test_support::enabled();
    assert!(matches!(
        session.eval_text("f[1]"),
        Err(EvalError::InputRequired { .. })
    ));
    match session.resume_suspended_evaluation("reply") {
        Ok(value) => assert_eq!(value.to_string(), "\"reply\""),
        Err(error) => panic!("resume: {error:?}"),
    }
    let rows = Session::logical_profile_test_rows();
    for phase in ["definition.rhs.total", "definition.rhs.evaluate"] {
        assert_eq!(row(&rows, "phase", phase), [1, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
    }
    assert_rule_row(
        &rows,
        "down_value_rhs_eval",
        "f",
        [1, 0, 1, 0, 0, 0, 0, 0, 0, 0],
    );
    assert_closed(&rows);
}

#[test]
fn lp04_direct_default_preserves_some_and_none() {
    let mut session = Session::new();
    evaluate(&mut session, "Default[f] = 3");
    let _enabled = test_support::enabled();
    assert_eq!(evaluate(&mut session, "Default[f]"), "3");
    assert_eq!(evaluate(&mut session, "Default[g]"), "Default[g]");
    let rows = Session::logical_profile_test_rows();
    assert_eq!(
        row(&rows, "phase", "down_values.direct_default_lookup"),
        [2, 2, 0, 1, 1, 0, 0, 0, 0, 0]
    );
    assert_closed(&rows);
}

#[test]
fn lp04_down_value_compiled_matcher_closes_before_rhs() {
    let mut session = Session::new();
    evaluate(&mut session, "f[x_ /; True] := {x}");
    let _enabled = test_support::enabled();
    assert_eq!(evaluate(&mut session, "f[1]"), "List[1]");
    let rows = Session::logical_profile_test_rows();
    assert_eq!(
        row(&rows, "phase", "down_values.compiled_matcher"),
        [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_rule_row(
        &rows,
        "down_value_rhs_eval",
        "f",
        [1, 1, 0, 1, 0, 0, 0, 0, 0, 0],
    );
    assert_closed(&rows);
}

#[test]
fn lp04_sub_value_rule_and_compiled_matcher_close() {
    let mut session = Session::new();
    evaluate(&mut session, "f[x_][y_] /; True := {x, y}");
    let _enabled = test_support::enabled();
    assert_eq!(evaluate(&mut session, "f[1][2]"), "List[1, 2]");
    let rows = Session::logical_profile_test_rows();
    assert_eq!(
        row(&rows, "phase", "sub_values.compiled_matcher"),
        [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_rule_row(
        &rows,
        "sub_value_rhs_eval",
        "f",
        [1, 1, 0, 1, 0, 0, 0, 0, 0, 0],
    );
    assert_closed(&rows);
}

#[test]
fn lp04_up_value_rule_closes() {
    let mut session = Session::new();
    evaluate(&mut session, "tag /: h[tag] := 7");
    let _enabled = test_support::enabled();
    assert_eq!(evaluate(&mut session, "h[tag]"), "7");
    let rows = Session::logical_profile_test_rows();
    assert_rule_row(
        &rows,
        "up_value_rhs_eval",
        "tag",
        [1, 1, 0, 1, 0, 0, 0, 0, 0, 0],
    );
    assert_closed(&rows);
}
