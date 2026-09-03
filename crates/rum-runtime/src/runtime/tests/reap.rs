use super::*;

#[test]
fn reap_buckets_stay_in_first_appearance_order() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(r#"Reap[Sow[1, "b"]; Sow[2, "a"]; Sow[3, "b"]]"#)
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[3, List[List[1, 3], List[2]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn nested_reap_scopes_keep_their_own_buckets() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("Reap[Sow[1, x]; Reap[Sow[2, y]]]")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[2, List[List[2]]], List[List[1], List[2]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn reap_patterns_select_which_sows_are_captured() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(r#"Reap[Sow[1, "aa"]; Sow[2, "ab"], _String]"#)
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[2, List[List[1], List[2]]]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn reap_index_preserves_wide_order_and_appends_to_existing_buckets() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(
            "Reap[Do[Sow[i, i], {i, 1, 64}]; \
             Sow[1001, 1]; Sow[1032, 32]; Sow[1064, 64]]",
        )
        .unwrap();
    assert!(outcome.messages.is_empty());

    let Expr::Call { head, args, .. } = outcome.value else {
        panic!("Reap should return a list pair, got {}", outcome.value);
    };
    assert!(is_system_symbol_name(&head, "List"));
    assert_eq!(args.len(), 2);

    let Expr::Call {
        head: buckets_head,
        args: buckets,
        ..
    } = &args[1]
    else {
        panic!("Reap harvest should be a list, got {}", args[1]);
    };
    assert!(is_system_symbol_name(buckets_head, "List"));
    assert_eq!(buckets.len(), 64);
    assert_eq!(buckets[0].to_string(), "List[1, 1001]");
    assert_eq!(buckets[31].to_string(), "List[32, 1032]");
    assert_eq!(buckets[63].to_string(), "List[64, 1064]");
}
