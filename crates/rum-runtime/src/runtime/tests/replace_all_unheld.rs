use super::*;
use crate::runtime::test_builtins::register_test_arithmetic_builtins;

fn eval_replace_all_unheld(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_rewrite_data_builtin(RewriteDataBuiltin::ReplaceAllUnheld, head, args)
}

fn session_with_replace_all_unheld() -> Session {
    let mut session = session_with_test_control_builtins();
    register_test_arithmetic_builtins(&mut session);
    session.register_builtin(
        BuiltinSpec::special(
            "ReplaceAllUnheld",
            "Test-local ReplaceAllUnheld builtin.",
            eval_replace_all_unheld,
            MemoryGrowthContract::Incremental,
        )
        .protected(),
    );
    session
}

fn assert_value(session: &mut Session, source: &str, expected: &str) {
    let outcome = session.eval_text(source).unwrap();
    assert!(
        outcome.messages.is_empty(),
        "{source}: {:?}",
        outcome.messages
    );
    assert_eq!(outcome.value.to_string(), expected, "{source}");
}

#[test]
fn replace_all_unheld_masks_only_held_argument_positions() {
    let mut session = session_with_replace_all_unheld();
    session.eval_text("SetAttributes[hr, HoldRest]").unwrap();

    for (replace_all, replace_all_unheld, expected_all, expected_unheld) in [
        (
            "ReplaceAll[Hold[1+1] + 2, Plus -> Times]",
            "ReplaceAllUnheld[Hold[1+1] + 2, Plus -> Times]",
            "Times[2, Hold[Times[1, 1]]]",
            "Times[2, Hold[Plus[1, 1]]]",
        ),
        (
            "ReplaceAll[hr[1+1, 2+2], Plus -> Times]",
            "ReplaceAllUnheld[hr[1+1, 2+2], Plus -> Times]",
            "hr[2, Times[2, 2]]",
            "hr[2, Plus[2, 2]]",
        ),
        (
            "ReplaceAll[Hold[a[1+1]], Plus -> Times]",
            "ReplaceAllUnheld[Hold[a[1+1]], Plus -> Times]",
            "Hold[a[Times[1, 1]]]",
            "Hold[a[Plus[1, 1]]]",
        ),
        (
            "ReplaceAll[HoldComplete[1+1], Plus -> Times]",
            "ReplaceAllUnheld[HoldComplete[1+1], Plus -> Times]",
            "HoldComplete[Times[1, 1]]",
            "HoldComplete[Plus[1, 1]]",
        ),
        (
            "ReplaceAll[1+1, Plus -> Times]",
            "ReplaceAllUnheld[1+1, Plus -> Times]",
            "2",
            "2",
        ),
    ] {
        assert_value(&mut session, replace_all, expected_all);
        assert_value(&mut session, replace_all_unheld, expected_unheld);
    }
}

#[test]
fn replace_all_unheld_uses_original_and_compound_head_attributes() {
    let mut session = session_with_replace_all_unheld();
    session
        .eval_text("SetAttributes[op, SubValuesHoldAll]")
        .unwrap();

    assert_value(
        &mut session,
        "ReplaceAll[op[0][1+1], Plus -> Times]",
        "op[0][Times[1, 1]]",
    );
    assert_value(
        &mut session,
        "ReplaceAllUnheld[op[0][1+1], Plus -> Times]",
        "op[0][Plus[1, 1]]",
    );
    assert_value(
        &mut session,
        "ReplaceAllUnheld[Hold[1+1], Hold -> List]",
        "List[2]",
    );
}

#[test]
fn replace_all_unheld_operator_form_applies_to_multiple_inputs() {
    let mut session = session_with_replace_all_unheld();

    assert_value(
        &mut session,
        "ReplaceAllUnheld[Plus -> Times][Hold[1+1] + 2]",
        "Times[2, Hold[Plus[1, 1]]]",
    );
    assert_value(
        &mut session,
        "ReplaceAllUnheld[Plus -> Times][Hold[a[1+1]]]",
        "Hold[a[Plus[1, 1]]]",
    );
}

#[test]
fn replace_all_unheld_rule_shape_messages_match_replace_all() {
    let mut session = session_with_replace_all_unheld();

    let replace_all = session
        .eval_text("ReplaceAll[Hold[1+1], notarule]")
        .unwrap();
    let replace_all_unheld = session
        .eval_text("ReplaceAllUnheld[Hold[1+1], notarule]")
        .unwrap();

    assert_eq!(
        replace_all.value.to_string(),
        "ReplaceAll[Hold[Plus[1, 1]], notarule]"
    );
    assert_eq!(
        replace_all_unheld.value.to_string(),
        "ReplaceAllUnheld[Hold[Plus[1, 1]], notarule]"
    );
    assert_eq!(replace_all.messages.len(), 1);
    assert_eq!(replace_all_unheld.messages.len(), 1);
    assert_eq!(replace_all.messages[0].symbol, "ReplaceAll");
    assert_eq!(replace_all_unheld.messages[0].symbol, "ReplaceAllUnheld");
    assert_eq!(replace_all.messages[0].tag, "argform");
    assert_eq!(
        replace_all.messages[0].tag,
        replace_all_unheld.messages[0].tag
    );
    assert_eq!(
        replace_all.messages[0].text,
        replace_all_unheld.messages[0].text
    );

    for (replace_all_source, replace_all_unheld_source) in [
        ("ReplaceAll[]", "ReplaceAllUnheld[]"),
        ("ReplaceAll[a, b, c]", "ReplaceAllUnheld[a, b, c]"),
    ] {
        let replace_all = session.eval_text(replace_all_source).unwrap();
        let replace_all_unheld = session.eval_text(replace_all_unheld_source).unwrap();
        assert_eq!(replace_all.messages.len(), 1, "{replace_all_source}");
        assert_eq!(
            replace_all_unheld.messages.len(),
            1,
            "{replace_all_unheld_source}"
        );
        assert_eq!(replace_all.messages[0].tag, "argx");
        assert_eq!(
            replace_all.messages[0].tag,
            replace_all_unheld.messages[0].tag
        );
        assert_eq!(
            replace_all.messages[0].text,
            replace_all_unheld.messages[0].text
        );
    }
}
