use crate::runtime::profiling::logical::test_support;
use crate::runtime::{EvalError, Session};

fn row(rows: &[(String, [u64; 10])], phase: &str) -> [u64; 10] {
    let Some((_, counts)) = rows
        .iter()
        .find(|(key, _)| key.contains(&format!("phase: \"{phase}\"")))
    else {
        panic!("missing {phase}: {rows:?}");
    };
    *counts
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

fn completed_set(source: &str, expected: &str, phase: &str) {
    let _enabled = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    match session.eval_text(source) {
        Ok(result) => assert_eq!(result.value.to_string(), expected),
        Err(error) => panic!("{source}: {error:?}"),
    }
    let rows = Session::logical_profile_test_rows();
    assert_eq!(row(&rows, phase), [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_closed(&rows);
    eprintln!("LP05 completed {phase}: {:?}", row(&rows, phase));
}

fn resumed_set(phase: &'static str) {
    let _enabled = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    assert!(matches!(
        session.eval_text("x = InputString[\"rhs\"]; x"),
        Err(EvalError::InputRequired { .. })
    ));
    let paused = Session::logical_profile_test_rows();
    match session.resume_suspended_evaluation("reply") {
        Ok(value) => assert_eq!(value.to_string(), "\"reply\""),
        Err(error) => panic!("resume: {error:?}"),
    }
    let rows = Session::logical_profile_test_rows();
    assert_eq!(row(&rows, phase), [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    if phase == "set.finish" {
        assert!(!paused.iter().any(|(key, _)| key.contains(phase)));
    } else {
        assert_eq!(row(&paused, phase), [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert!(Session::logical_profile_test_pauses_for(phase) > 0);
    }
    assert_closed(&rows);
    eprintln!("LP05 resumed {phase}: {:?}", row(&rows, phase));
}

macro_rules! assignment_rows {
    ($ordinary:ident, $atom:ident, $resumed:ident, $phase:literal) => {
        #[test]
        fn $ordinary() {
            completed_set("x = f[1]; x", "f[1]", $phase);
        }

        #[test]
        fn $atom() {
            completed_set("x = 1; x", "1", $phase);
        }

        #[test]
        fn $resumed() {
            resumed_set($phase);
        }
    };
}

assignment_rows!(
    lp05_ordinary_rhs_evaluate,
    lp05_atom_rhs_evaluate,
    lp05_resumed_rhs_evaluate,
    "set.rhs.evaluate"
);
assignment_rows!(
    lp05_ordinary_rhs_suspend,
    lp05_atom_rhs_suspend,
    lp05_resumed_rhs_suspend,
    "set.rhs.suspend"
);
assignment_rows!(
    lp05_ordinary_finish,
    lp05_atom_finish,
    lp05_resumed_finish,
    "set.finish"
);

#[test]
fn lp05_rhs_error_does_not_enter_finish() {
    let _enabled = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    assert!(matches!(
        session.eval_text("x = Throw[7]"),
        Err(EvalError::Thrown { .. })
    ));
    let rows = Session::logical_profile_test_rows();
    for phase in ["set.rhs.evaluate", "set.rhs.suspend"] {
        assert_eq!(row(&rows, phase), [1, 0, 1, 0, 0, 0, 1, 0, 0, 0]);
    }
    assert!(!rows.iter().any(|(key, _)| key.contains("set.finish")));
    assert_closed(&rows);
}

fn assignment_route(
    source: &str,
    responses: &[(&str, &str)],
    expected_value: Option<&str>,
    phase: &'static str,
    expected_counts: [u64; 10],
) {
    let _enabled = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    let mut result = session.eval_text(source).map(|result| result.value);
    for (prompt, answer) in responses {
        match &result {
            Err(EvalError::InputRequired { request }) => assert_eq!(&request.prompt, prompt),
            other => panic!("expected input {prompt}: {other:?}"),
        }
        result = session.resume_suspended_evaluation(*answer);
    }
    match (expected_value, result) {
        (Some(expected), Ok(value)) => assert_eq!(value.to_string(), expected),
        (None, Err(EvalError::Thrown { value, .. })) => assert_eq!(value.to_string(), "7"),
        (expected, actual) => panic!("expected {expected:?}: {actual:?}"),
    }
    let rows = Session::logical_profile_test_rows();
    assert_eq!(row(&rows, phase), expected_counts);
    if !responses.is_empty() {
        assert!(Session::logical_profile_test_pauses_for(phase) > 0);
    }
    assert_closed(&rows);
    eprintln!(
        "LP05 A2 {phase}: {:?}; responses={}",
        row(&rows, phase),
        responses.len()
    );
}

#[test]
fn lp05_a2_declined_list_is_none() {
    assignment_route(
        "f[1] = 7; f[1]",
        &[],
        Some("7"),
        "set.try_list_destructuring",
        [1, 1, 0, 0, 1, 0, 0, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_declined_association_is_none() {
    assignment_route(
        "f[1] = 7; f[1]",
        &[],
        Some("7"),
        "set.try_association_lvalue",
        [1, 1, 0, 0, 1, 0, 0, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_completed_lhs_includes_store() {
    assignment_route(
        "f[1] = 7; f[1]",
        &[],
        Some("7"),
        "set.prepare_lhs",
        [1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_completed_list_is_some() {
    assignment_route(
        "{a, b} = {1, 2}; {a, b}",
        &[],
        Some("List[1, 2]"),
        "set.try_list_destructuring",
        [1, 1, 0, 1, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_list_spans_two_leaf_pauses() {
    assignment_route(
        r#"{f[InputString["first"]], g[InputString["second"]]} = {1, 2}; {f["a"], g["b"]}"#,
        &[("first", "a"), ("second", "b")],
        Some("List[1, 2]"),
        "set.try_list_destructuring",
        [3, 3, 0, 1, 2, 0, 0, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_association_spans_two_selector_pauses() {
    assignment_route(
        r#"assoc = <|"a" -> <|"b" -> 0|>|>; assoc[InputString["first"], InputString["second"]] = 9; assoc["a", "b"]"#,
        &[("first", "a"), ("second", "b")],
        Some("9"),
        "set.try_association_lvalue",
        [1, 1, 0, 1, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_association_part_completes_after_typed_selector_consumer() {
    assignment_route(
        r#"assoc = <|"a" -> 0|>; assoc[[System`Key[InputString["key"]]]] = 9; assoc["a"]"#,
        &[("key", "a")],
        Some("9"),
        "set.try_association_lvalue",
        [1, 1, 0, 1, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_list_resumed_error_is_not_some() {
    assignment_route(
        r#"{f[(InputString["lhs"]; Throw[7])]} = {1}"#,
        &[("lhs", "a")],
        None,
        "set.try_list_destructuring",
        [2, 1, 1, 0, 1, 0, 1, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_association_resumed_error_is_not_some() {
    assignment_route(
        r#"assoc = <|"a" -> 0|>; assoc[(InputString["key"]; Throw[7])] = 9"#,
        &[("key", "a")],
        None,
        "set.try_association_lvalue",
        [1, 0, 1, 0, 0, 0, 1, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_lhs_resumed_error_closes_once() {
    assignment_route(
        r#"f[(InputString["lhs"]; Throw[7])] = 9"#,
        &[("lhs", "a")],
        None,
        "set.prepare_lhs",
        [1, 0, 1, 0, 0, 0, 1, 0, 0, 0],
    );
}

#[test]
fn lp05_a2_rhs_then_lhs_pause_preserves_nested_boundaries() {
    let _enabled = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    match session.eval_text(r#"f[InputString["lhs"]] = InputString["rhs"]; f["key"]"#) {
        Err(EvalError::InputRequired { request }) => assert_eq!(request.prompt, "rhs"),
        other => panic!("expected RHS input: {other:?}"),
    }
    let rhs_pause = Session::logical_profile_test_rows();
    assert!(
        !rhs_pause
            .iter()
            .any(|(key, _)| key.contains("set.prepare_lhs"))
    );
    match session.resume_suspended_evaluation("value") {
        Err(EvalError::InputRequired { request }) => assert_eq!(request.prompt, "lhs"),
        other => panic!("expected LHS input: {other:?}"),
    }
    let lhs_pause = Session::logical_profile_test_rows();
    for phase in ["set.rhs.evaluate", "set.rhs.suspend"] {
        assert_eq!(row(&lhs_pause, phase), [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    }
    for phase in ["set.prepare_lhs", "set.finish"] {
        assert_eq!(row(&lhs_pause, phase), [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    }
    match session.resume_suspended_evaluation("key") {
        Ok(value) => assert_eq!(value.to_string(), "\"value\""),
        Err(error) => panic!("assignment completion: {error:?}"),
    }
    let rows = Session::logical_profile_test_rows();
    for phase in [
        "set.rhs.evaluate",
        "set.rhs.suspend",
        "set.prepare_lhs",
        "set.finish",
    ] {
        assert_eq!(row(&rows, phase), [1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(Session::logical_profile_test_pauses_for(phase) > 0);
    }
    assert_closed(&rows);
    eprintln!(
        "LP05 A2 double pause: RHS closed before LHS; finish/LHS each retained once; final open=0 abandoned=0"
    );
}
