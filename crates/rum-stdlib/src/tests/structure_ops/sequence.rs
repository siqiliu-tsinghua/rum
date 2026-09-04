use super::*;

#[test]
fn range_constructs_plain_integer_lists_for_current_subset() {
    let mut session = core_session();

    assert_eq!(
        session.eval_text("Range[5]").unwrap().value.to_string(),
        "List[1, 2, 3, 4, 5]"
    );
    assert_eq!(
        session
            .eval_text("Range[2, 6, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 4, 6]"
    );
    assert_eq!(
        session
            .eval_text("Range[5, 1, -2]")
            .unwrap()
            .value
            .to_string(),
        "List[5, 3, 1]"
    );
    assert_eq!(
        session.eval_text("Range[5, 1]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Range[1, 5, 0]")
            .unwrap()
            .value
            .to_string(),
        "Range[1, 5, 0]"
    );
}

#[test]
fn range_respects_generated_element_limit_before_allocating() {
    let mut session = core_session();
    session.set_generated_element_limit(3);

    let limited = session.eval_text("Range[5]").unwrap();
    assert_eq!(
        limited.value.to_string(),
        "TerminatedEvaluation[\"GeneratedElementLimit\"]"
    );
    assert!(
        limited
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    assert_eq!(
        session
            .eval_text("MemoryConstrained[Range[100], 20, memfail]")
            .unwrap()
            .value
            .to_string(),
        "memfail"
    );
}

#[test]
fn take_multispec_shallow_companion_pins_three_level_fanout() {
    let mut session = core_session();

    let outcome = session
        .eval_text("Take[{{{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}}, All, All, 1]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[List[List[1], List[3]], List[List[5], List[7]]]"
    );
    assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
}

#[test]
fn take_multispec_failure_below_first_spec_preserves_message_channel() {
    let mut session = core_session();

    let outcome = session.eval_text("Take[{{1}, {2}}, All, 2]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Take[List[List[1], List[2]], All, 2]"
    );
    assert_eq!(outcome.messages.len(), 1, "{:?}", outcome.messages);
    assert_eq!(outcome.messages[0].symbol, "Take");
    assert_eq!(outcome.messages[0].tag, "take");
    assert_eq!(
        outcome.messages[0].text,
        "Cannot apply the requested sequence specification in Take[List[List[1], List[2]], All, 2]."
    );
}

#[test]
fn take_drop_insert_delete_and_take_list_cover_current_structural_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Take[f[a, b, c, d], 2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[a, b, c, d], -2]")
            .unwrap()
            .value
            .to_string(),
        "f[c, d]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[a, b, c, d], {2}]")
            .unwrap()
            .value
            .to_string(),
        "f[b]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[a, b, c, d], {2, -1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[g[a, b], h[c, d]], All, {2}]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
    );
    assert_eq!(
        session
            .eval_text("Take[f[a, b], None]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );

    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c, d], 2]")
            .unwrap()
            .value
            .to_string(),
        "f[c, d]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c, d], -2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c, d], {2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, d]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c, d, e], {1, 5, 2}]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c, d, e], {5, 1, -2}]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c], None]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[a, b, c], All]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Drop[f[g[a, b], h[c, d]], None, 1]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
    );
    assert_eq!(
        session
            .eval_text("TakeDrop[f[a, b, c, d], 2]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, b], f[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("Insert[f[a, b, c], x, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, x, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Insert[f[a, b, c], x, -1]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c, x]"
    );
    assert_eq!(
        session
            .eval_text("Insert[f[g[a]], x, {1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "f[g[x, a]]"
    );
    assert_eq!(
        session
            .eval_text("Insert[f[a, b], x, {{1}, {-1}}]")
            .unwrap()
            .value
            .to_string(),
        "f[x, a, b, x]"
    );
    assert_eq!(
        session
            .eval_text("Delete[f[a, b, c, d], {{2}, {-1}}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Delete[f[g[a]], {1, 0}]")
            .unwrap()
            .value
            .to_string(),
        "f[a]"
    );
    assert_eq!(
        session
            .eval_text("Delete[f[a], 0]")
            .unwrap()
            .value
            .to_string(),
        "a"
    );
    assert_eq!(
        session
            .eval_text("Delete[f[a, b], 0]")
            .unwrap()
            .value
            .to_string(),
        "Sequence[a, b]"
    );
    assert_eq!(
        session
            .eval_text("TakeList[f[a, b, c, d], {1, 2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a], f[b, c], f[d]]"
    );
}

#[test]
fn take_drop_accept_upto_sequence_specs() {
    let mut session = core_session();

    for (input, expected) in [
        ("Take[{1,2,3}, UpTo[5]]", "List[1, 2, 3]"),
        ("Take[{1,2,3}, UpTo[2]]", "List[1, 2]"),
        ("Take[{1,2,3}, UpTo[0]]", "List[]"),
        ("Drop[{1,2,3}, UpTo[5]]", "List[]"),
        ("TakeDrop[{1,2,3}, UpTo[2]]", "List[List[1, 2], List[3]]"),
        ("Take[{1,2,3}, UpTo[Infinity]]", "List[1, 2, 3]"),
        (
            "Take[{{1,2,3},{4,5}}, UpTo[3], UpTo[2]]",
            "List[List[1, 2], List[4, 5]]",
        ),
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), expected);
        assert!(outcome.messages.is_empty());
    }
}

#[test]
fn take_upto_reports_atom_and_malformed_spec_messages() {
    let mut session = core_session();

    let outcome = session.eval_text("Take[7, UpTo[2]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[7, UpTo[2]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Take"
            && message.tag == "take"
            && message.text == "Cannot take positions 1 through 2 in 7."
    }));

    let outcome = session.eval_text("Take[{1,2,3}, UpTo[-2]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[List[1, 2, 3], UpTo[-2]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "UpTo"
            && message.tag == "innf"
            && message.text
                == "Non-negative integer or Infinity expected at position 1 in UpTo[-2]."
    }));

    let outcome = session.eval_text("Take[{1,2,3}, UpTo[1/2]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[List[1, 2, 3], UpTo[1/2]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "UpTo"
            && message.tag == "innf"
            && message.text
                == "Non-negative integer or Infinity expected at position 1 in UpTo[1/2]."
    }));

    let outcome = session.eval_text("Take[{1,2,3}, UpTo[x]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[List[1, 2, 3], UpTo[x]]");
    assert!(outcome.messages.is_empty());

    let outcome = session.eval_text("Take[{1,2,3}, UpTo[1,2]]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[List[1, 2, 3], UpTo[1, 2]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "UpTo"
            && message.tag == "argx"
            && message.text == "UpTo called with 2 arguments; 1 argument is expected."
    }));
}

#[test]
fn take_existing_sequence_specs_are_unchanged() {
    let mut session = core_session();

    for (input, expected) in [
        ("Take[{1,2,3}, 2]", "List[1, 2]"),
        ("Take[{1,2,3}, {2,3}]", "List[2, 3]"),
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), expected);
        assert!(outcome.messages.is_empty());
    }
}

#[test]
fn sequence_operator_forms_rewrite_to_data_first_calls() {
    let mut session = core_session();

    let applied = session
        .eval_text(
            "{Take[2][{a, b, c}], Drop[1][{a, b, c}], Delete[1][{a, b}], \
             TakeList[{2, 1}][{a, b, c, d}], Insert[x, 2][{a, b}]}",
        )
        .unwrap();
    assert_eq!(
        applied.value.to_string(),
        "List[List[a, b], List[b, c], List[b], List[List[a, b], List[c]], List[a, x, b]]"
    );
    assert!(applied.messages.is_empty());

    let inert = session
        .eval_text("{Take[2], Drop[1], Delete[1], TakeList[{2, 1}], Insert[x, 2]}")
        .unwrap();
    assert_eq!(
        inert.value.to_string(),
        "List[Take[2], Drop[1], Delete[1], TakeList[List[2, 1]], Insert[x, 2]]"
    );
    assert!(inert.messages.is_empty());
}

#[test]
fn take_drop_append_and_prepend_report_current_invalid_boundaries() {
    let mut session = core_session();

    let outcome = session.eval_text("Take[f[a], 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[f[a], 2]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Take" && message.tag == "take")
    );

    let outcome = session.eval_text("Take[5, 1]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[5, 1]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Take"
            && message.tag == "take"
            && message.text == "Cannot take positions 1 through 1 in 5."
    }));

    let outcome = session.eval_text("Take[5, All]").unwrap();
    assert_eq!(outcome.value.to_string(), "Take[5, All]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Take"
            && message.tag == "take"
            && message.text == "Cannot take positions 1 through -1 in 5."
    }));

    let outcome = session.eval_text("Drop[f[a], 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "Drop[f[a], 2]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Drop" && message.tag == "drop")
    );

    let outcome = session.eval_text("Drop[5, 1]").unwrap();
    assert_eq!(outcome.value.to_string(), "Drop[5, 1]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Drop"
            && message.tag == "drop"
            && message.text == "Cannot drop positions 1 through 1 in 5."
    }));

    let outcome = session.eval_text("TakeDrop[5, 1]").unwrap();
    assert_eq!(outcome.value.to_string(), "TakeDrop[5, 1]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "TakeDrop"
            && message.tag == "iseq"
            && message.text == "Invalid sequence specification 1 for an expression of length 0."
    }));

    let outcome = session.eval_text("Append[5, x]").unwrap();
    assert_eq!(outcome.value.to_string(), "Append[5, x]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Append" && message.tag == "normal")
    );

    let outcome = session.eval_text("Prepend[5, x]").unwrap();
    assert_eq!(outcome.value.to_string(), "Prepend[5, x]");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Prepend" && message.tag == "normal")
    );

    let outcome = session.eval_text("Insert[f[a], x, 0]").unwrap();
    assert_eq!(outcome.value.to_string(), "Insert[f[a], x, 0]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Insert"
            && message.tag == "ins"
            && message.text == "Cannot insert at position {0} in f[a]"
    }));

    let outcome = session.eval_text("Insert[f[g[a]], x, {1, 0}]").unwrap();
    assert_eq!(outcome.value.to_string(), "Insert[f[g[a]], x, List[1, 0]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Insert"
            && message.tag == "ins"
            && message.text == "Cannot insert at position {1, 0} in f[g[a]]"
    }));

    let outcome = session.eval_text("Insert[5, x, 1]").unwrap();
    assert_eq!(outcome.value.to_string(), "Insert[5, x, 1]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Insert"
            && message.tag == "ins"
            && message.text == "Cannot insert at position {1} in 5"
    }));

    let outcome = session.eval_text("Delete[f[a], 2]").unwrap();
    assert_eq!(outcome.value.to_string(), "Delete[f[a], 2]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Delete"
            && message.tag == "partw"
            && message.text == "Part {2} of f[a] does not exist."
    }));

    let outcome = session.eval_text("Delete[f[a, b, c, d], {2, -1}]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Delete[f[a, b, c, d], List[2, -1]]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Delete"
            && message.tag == "partw"
            && message.text == "Part -1 of b does not exist."
    }));

    let outcome = session.eval_text("Delete[5, {1}]").unwrap();
    assert_eq!(outcome.value.to_string(), "Delete[5, List[1]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Delete"
            && message.tag == "partw"
            && message.text == "Part 1 of 5 does not exist."
    }));

    let outcome = session.eval_text("TakeList[f[a], {2}]").unwrap();
    assert_eq!(outcome.value.to_string(), "TakeList[f[a], List[2]]");

    let outcome = session.eval_text("TakeList[5, {1}]").unwrap();
    assert_eq!(outcome.value.to_string(), "TakeList[5, List[1]]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "TakeList"
            && message.tag == "normal"
            && message.text
                == "Nonatomic expression expected at position 1 in TakeList[5, List[1]]."
    }));
}
