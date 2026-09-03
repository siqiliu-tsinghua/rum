use super::*;

// The rewrite/traversal builtins used to walk their children by calling
// themselves from inside a drive_internal_value continuation, so the native
// stack depth equaled the element count and a wide flat list aborted the
// process. These tests pin the iterative walks at a width far past every
// measured abort threshold.

/// Builds `{1, 2, ..., count}` as source text; the stdlib list constructors
/// (Table, Range) are not available in the kernel test session.
fn wide_list_source(count: usize) -> String {
    let elements = (1..=count)
        .map(|k| k.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{elements}}}")
}

#[test]
fn cases_resumes_mid_walk_between_children() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("Cases[{1, 2, 3}, x_Integer /; MatchQ[InputString[\"el: \"], \"yes\"], {1}]")
        .expect_err("the first element's condition should suspend the walk");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "el: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("yes")
        .expect_err("the second element's condition should suspend the walk again");

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "el: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let third = session
        .resume_eval_request_with_input("no")
        .expect_err("the third element's condition should suspend the walk again");

    match &third.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "el: ");
        }
        other => panic!("expected third input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish the Cases walk after the last condition resolves");

    assert_eq!(response.value.to_string(), "List[1, 3]");
}

#[test]
fn replace_walks_thirty_thousand_children() {
    let mut session = session_with_test_rewrite_builtins();

    let source = format!(
        "Replace[{}, _Integer -> 0, {{1}}]",
        wide_list_source(30_000)
    );
    let outcome = session.eval_text(&source).unwrap();
    let Expr::Call { head, args, .. } = outcome.value else {
        panic!("Replace should return a list, got {}", outcome.value);
    };
    assert!(is_system_symbol_name(&head, "List"));
    assert_eq!(args.len(), 30_000);
    let zero = Expr::integer(BigInt::from(0_u8));
    assert!(args.iter().all(|arg| *arg == zero));
    assert!(outcome.messages.is_empty());
}

#[test]
fn cases_walks_thirty_thousand_children() {
    let mut session = session_with_test_rewrite_builtins();

    let source = format!("Cases[{}, _Integer]", wide_list_source(30_000));
    let outcome = session.eval_text(&source).unwrap();
    let Expr::Call { head, args, .. } = outcome.value else {
        panic!("Cases should return a list, got {}", outcome.value);
    };
    assert!(is_system_symbol_name(&head, "List"));
    assert_eq!(args.len(), 30_000);
    assert_eq!(args[0], Expr::integer(BigInt::from(1_u8)));
    assert_eq!(
        args[args.len() - 1],
        Expr::integer(BigInt::from(30_000_u32))
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn count_walks_thirty_thousand_children() {
    let mut session = session_with_test_rewrite_builtins();

    let source = format!("Count[{}, _Integer]", wide_list_source(30_000));
    let outcome = session.eval_text(&source).unwrap();
    assert_eq!(outcome.value.to_string(), "30000");
    assert!(outcome.messages.is_empty());
}

#[test]
fn position_walks_thirty_thousand_children() {
    let mut session = session_with_test_rewrite_builtins();

    let source = format!("Position[{}, 5]", wide_list_source(30_000));
    let outcome = session.eval_text(&source).unwrap();
    assert_eq!(outcome.value.to_string(), "List[List[5]]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn delete_cases_walks_thirty_thousand_children() {
    let mut session = session_with_test_rewrite_builtins();

    let source = format!("DeleteCases[{}, 5]", wide_list_source(30_000));
    let outcome = session.eval_text(&source).unwrap();
    let Expr::Call { head, args, .. } = outcome.value else {
        panic!("DeleteCases should return a list, got {}", outcome.value);
    };
    assert!(is_system_symbol_name(&head, "List"));
    assert_eq!(args.len(), 29_999);
    assert_eq!(args[3], Expr::integer(BigInt::from(4_u8)));
    assert_eq!(args[4], Expr::integer(BigInt::from(6_u8)));
    assert!(outcome.messages.is_empty());
}

#[test]
fn replace_all_walks_thirty_thousand_children() {
    let mut session = session_with_test_rewrite_builtins();

    let source = format!("ReplaceAll[{}, 1 -> 0]", wide_list_source(30_000));
    let outcome = session.eval_text(&source).unwrap();
    let Expr::Call { head, args, .. } = outcome.value else {
        panic!("ReplaceAll should return a list, got {}", outcome.value);
    };
    assert!(is_system_symbol_name(&head, "List"));
    assert_eq!(args.len(), 30_000);
    assert_eq!(args[0], Expr::integer(BigInt::from(0_u8)));
    assert_eq!(args[1], Expr::integer(BigInt::from(2_u8)));
    assert!(outcome.messages.is_empty());
}
