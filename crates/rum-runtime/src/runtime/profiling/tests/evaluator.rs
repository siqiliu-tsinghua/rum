use crate::runtime::profiling::logical::test_support;
use crate::runtime::{EvalError, Session};

fn row(rows: &[(String, [u64; 10])], phase: &str) -> [u64; 10] {
    rows.iter()
        .find(|(key, _)| key.contains(&format!("phase: \"{phase}\"")))
        .map(|(_, counts)| *counts)
        .unwrap_or_else(|| panic!("missing {phase}: {rows:?}"))
}

fn assert_leaf_rows(rows: &[(String, [u64; 10])]) {
    for phase in [
        "evaluate_inner_unstable.total",
        "evaluate_inner_unstable.call",
        "evaluate_atom_fast_path.total",
        "evaluate_call.head_eval",
        "call.stage.prepare_arguments",
        "prepare_arguments.total",
        "prepare_arguments.evaluate_arg",
    ] {
        let counts = row(rows, phase);
        assert_eq!(
            counts[0],
            counts[1] + counts[2],
            "{phase}: {counts:?}; rows={rows:?}"
        );
        assert!(counts[1] > 0, "{phase}: {counts:?}; rows={rows:?}");
    }
    for (key, counts) in rows {
        assert_eq!(counts[8], 0, "{key}: {counts:?}");
        assert_eq!(counts[9], 0, "{key}: {counts:?}");
        if key.contains("phase: \"call.stage.") {
            assert_eq!(counts[3] + counts[4], counts[1], "{key}: {counts:?}");
        }
    }
    assert!(row(rows, "call.stage.prepare_arguments")[4] > 0);
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(test_support::active(), 0);
}

fn defined_session(definition: &str) -> Session {
    let mut session = Session::new();
    session.eval_text(definition).unwrap();
    session
}

#[test]
fn lp03_leaf_phases_close_on_completed_evaluation() {
    let mut session = defined_session("f[x_] := x");
    let _enabled = test_support::enabled();
    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "1");
    assert_leaf_rows(&Session::logical_profile_test_rows());
}

#[test]
fn lp03_inner_symbol_and_atom_close() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    assert!(
        session
            .evaluate_inner_unstable(crate::runtime::Expr::small_integer(1))
            .is_ok()
    );
    assert!(
        session
            .evaluate_inner_unstable(crate::runtime::Expr::contextual_symbol("Global`", "x"))
            .is_ok()
    );
    let rows = Session::logical_profile_test_rows();
    assert!(row(&rows, "evaluate_inner_unstable.symbol")[1] > 0);
    assert!(row(&rows, "evaluate_inner_unstable.atom")[1] > 0);
    Session::logical_profile_test_assert_quiescent();
}

#[test]
fn lp03_leaf_phases_close_after_resume() {
    let mut session = defined_session("f[x_] := x");
    let _enabled = test_support::enabled();
    assert_eq!(session.eval_text("f[0]").unwrap().value.to_string(), "0");
    assert!(matches!(
        session.eval_text("f[InputString[\"prompt\"]]"),
        Err(EvalError::InputRequired { .. })
    ));
    assert_eq!(
        session
            .resume_suspended_evaluation("reply")
            .unwrap()
            .to_string(),
        "\"reply\""
    );
    assert!(Session::logical_profile_test_pauses() > 0);
    assert_leaf_rows(&Session::logical_profile_test_rows());
}

#[test]
fn lp03_argument_total_closes_each_pause() {
    let mut session = defined_session("f[x_, y_] := x");
    let _enabled = test_support::enabled();
    assert_eq!(session.eval_text("f[0, 1]").unwrap().value.to_string(), "0");
    assert!(matches!(
        session.eval_text("f[InputString[\"first\"], InputString[\"second\"]]"),
        Err(EvalError::InputRequired { .. })
    ));
    assert!(matches!(
        session.resume_suspended_evaluation("one"),
        Err(EvalError::InputRequired { .. })
    ));
    assert_eq!(
        session
            .resume_suspended_evaluation("two")
            .unwrap()
            .to_string(),
        "\"one\""
    );
    let rows = Session::logical_profile_test_rows();
    assert_leaf_rows(&rows);
    assert_eq!(row(&rows, "prepare_arguments.total")[0], 4);
    assert_eq!(row(&rows, "prepare_arguments.evaluate_arg")[0], 6);
    assert_eq!(row(&rows, "call.stage.prepare_arguments")[0], 4);
}

#[test]
fn lp03_adopted_site_table_is_exact() {
    let expected = std::collections::BTreeSet::from([
        "evaluate.total",
        "evaluate.once_unstable",
        "evaluate_inner_unstable.total",
        "evaluate_inner_unstable.symbol",
        "evaluate_inner_unstable.call",
        "evaluate_inner_unstable.atom",
        "evaluate_atom_fast_path.total",
        "evaluate_atom_fast_path.stable_symbol",
        "evaluate_atom_fast_path.unstable_symbol",
        "evaluate_atom_fast_path.atom",
        "evaluate_atom_fast_path.call",
        "evaluate_call.total",
        "evaluate_call.head_eval",
        "evaluate_call.resume_after_head",
        "evaluate_call.dispatch",
        "call.stage.special_form",
        "call.stage.prepare_arguments",
        "call.stage.listable",
        "call.stage.up_values",
        "call.stage.down_values",
        "call.stage.sub_values",
        "call.stage.ordinary_builtin",
        "call.stage.function",
        "prepare_arguments.total",
        "prepare_arguments.evaluate_arg",
    ]);
    let actual = super::PhaseKey::ALL
        .iter()
        .map(|key| key.name())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(actual, expected);
    for key in super::PhaseKey::ALL {
        assert_eq!(super::PhaseKey::from_name(key.name()), Some(key));
    }
}
