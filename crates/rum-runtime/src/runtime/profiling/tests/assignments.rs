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
