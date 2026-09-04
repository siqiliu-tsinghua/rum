use super::*;
use rum_runtime::SessionRequestError;

#[test]
fn key_group_by_groups_association_entries_by_transformed_keys() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("KeyGroupBy[<|\"aa\" -> 1, \"b\" -> 2|>, StringLength]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[2, Association[Rule[\"aa\", 1]]], Rule[1, Association[Rule[\"b\", 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("KeyGroupBy[<|\"aa\" -> 1, \"bb\" -> 2|>, StringLength, HoldComplete]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[2, HoldComplete[Association[Rule[\"aa\", 1], Rule[\"bb\", 2]]]]]"
    );
}

#[test]
fn association_key_flatten_and_deflatten_transform_path_keys() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("AssociationKeyDeflatten[<|{\"a\", \"b\"} -> 1, {\"a\", \"c\"} -> 2|>]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"a\", Association[Rule[\"b\", 1], Rule[\"c\", 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationKeyFlatten[<|\"a\" -> <|\"b\" -> 1|>|>]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[List[\"a\", \"b\"], 1]]"
    );
}

#[test]
fn association_key_deflatten_preserves_wide_branch_order_after_indexing() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                "Keys[AssociationKeyDeflatten[AssociationThread[Table[{i}, {i, 1, 20}], Range[20]]]] === Range[20]",
            )
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn association_key_flatten_and_deflatten_round_trip_well_formed_inputs() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("AssociationKeyDeflatten[AssociationKeyFlatten[<|\"a\" -> <|\"b\" -> 1, \"c\" -> 2|>|>]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"a\", Association[Rule[\"b\", 1], Rule[\"c\", 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationKeyFlatten[AssociationKeyDeflatten[<|{\"a\", \"b\"} -> 1, {\"a\", \"c\"} -> 2|>]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[List[\"a\", \"b\"], 1], Rule[List[\"a\", \"c\"], 2]]"
    );
}

#[test]
fn merge_nested_recursively_merges_associations_and_combines_leaf_values() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("MergeNested[{<|\"db\" -> <|\"host\" -> \"a\"|>|>, <|\"db\" -> <|\"port\" -> 5432|>|>}, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"db\", Association[Rule[\"host\", \"a\"], Rule[\"port\", 5432]]]]"
    );
    assert_eq!(
        session
            .eval_text("MergeNested[{<|\"a\" -> 1|>, <|\"a\" -> 2|>}, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"a\", 2]]"
    );
}

#[test]
fn merge_nested_three_level_companion_pins_branch_order_and_combiner_schedule() {
    let mut session = core_session();

    let outcome = session
        .eval_text(
            r#"calls = {};
               result = MergeNested[
                 {
                   <|
                     "outer" -> <|
                       "nested" -> <|"first" -> 1, "mixed" -> <|"left" -> 10|>|>,
                       "leftSibling" -> 30
                     |>,
                     "topLeaf" -> 40
                   |>,
                   <|
                     "outer" -> <|
                       "nested" -> <|"second" -> 2, "first" -> 3, "mixed" -> 50|>,
                       "rightSibling" -> 31,
                       "leftSibling" -> 32
                     |>,
                     "topSecond" -> 41,
                     "topLeaf" -> 42
                   |>
                 },
                 Function[values, calls = Append[calls, values]; Last[values]]
               ];
               {result, calls}"#,
        )
        .expect("three-level MergeNested companion should evaluate");

    assert_eq!(
        outcome.value.to_string(),
        "List[Association[Rule[\"outer\", Association[Rule[\"nested\", Association[Rule[\"first\", 3], Rule[\"mixed\", 50], Rule[\"second\", 2]]], Rule[\"leftSibling\", 32], Rule[\"rightSibling\", 31]]], Rule[\"topLeaf\", 42], Rule[\"topSecond\", 41]], List[List[1, 3], List[Association[Rule[\"left\", 10]], 50], List[2], List[30, 32], List[31], List[40, 42], List[41]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn merge_nested_suspended_combiner_resumes_without_repeating_calls() {
    let mut session = core_session();
    let source = r#"calls = {};
        result = MergeNested[
          {
            <|"outer" -> <|"a" -> 1, "b" -> 2|>, "tail" -> 3|>,
            <|"outer" -> <|"a" -> 4, "c" -> 5|>, "tail" -> 6|>
          },
          Function[values,
            calls = Append[calls, values];
            InputString["merge-nested: "];
            Last[values]
          ]
        ];
        {result, calls}"#;

    let mut outcome = session.eval_request(source);
    let mut suspensions = 0;
    loop {
        match outcome {
            Ok(response) => {
                assert_eq!(
                    response.value.to_string(),
                    "List[Association[Rule[\"outer\", Association[Rule[\"a\", 4], Rule[\"b\", 2], Rule[\"c\", 5]]], Rule[\"tail\", 6]], List[List[1, 4], List[2], List[5], List[3, 6]]]"
                );
                assert!(response.messages.is_empty());
                break;
            }
            Err(failure) => {
                match &failure.error {
                    SessionRequestError::InputRequired { request } => {
                        assert_eq!(request.prompt, "merge-nested: ");
                    }
                    other => panic!("expected input-required failure, got {other:?}"),
                }
                suspensions += 1;
                assert!(suspensions <= 4, "a combiner call was replayed");
                outcome = session.resume_eval_request_with_input("ignored");
            }
        }
    }

    assert_eq!(suspensions, 4);
    assert!(session.pending_input_request().is_none());
}

#[test]
fn merge_nested_suspension_freezes_the_iterative_tail_without_replay() {
    let mut session = core_session();
    let source = r#"left = Association["a" -> 1, "leftOnly" -> 2];
        right = Association["a" -> 3, "rightOnly" -> 4];
        Do[
          left = Association["k" -> left];
          right = Association["k" -> right],
          {i, 20}
        ];
        calls = {};
        result = MergeNested[
          {left, right},
          Function[values,
            calls = Append[calls, values];
            InputString["merge-nested-tail: "];
            Total[values]
          ]
        ];
        expected = Association["a" -> 4, "leftOnly" -> 2, "rightOnly" -> 4];
        Do[expected = Association["k" -> expected], {i, 20}];
        {SameQ[result, expected], calls}"#;

    let mut outcome = session.eval_request(source);
    for _ in 0..3 {
        let failure = outcome
            .expect_err("each combiner call in the iterative MergeNested tail should suspend");
        match &failure.error {
            SessionRequestError::InputRequired { request } => {
                assert_eq!(request.prompt, "merge-nested-tail: ");
            }
            other => panic!("expected input-required failure, got {other:?}"),
        }
        outcome = session.resume_eval_request_with_input("ignored");
    }

    let response = outcome.expect("the frozen MergeNested work stack should finish after input");
    assert_eq!(
        response.value.to_string(),
        "List[True, List[List[1, 3], List[2], List[4]]]"
    );
    assert!(response.messages.is_empty());
    assert!(session.pending_input_request().is_none());
}

#[test]
fn nested_association_additions_support_operator_forms() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("KeyGroupBy[f]")
            .unwrap()
            .value
            .to_string(),
        "Function[rum$keyedArg, KeyGroupBy[rum$keyedArg, f]]"
    );
    assert_eq!(
        session
            .eval_text("MergeNested[Last]")
            .unwrap()
            .value
            .to_string(),
        "Function[rum$keyedArg, MergeNested[rum$keyedArg, Last]]"
    );

    let outcome = session
        .eval_text(
            "{KeyGroupBy[StringLength][<|\"aa\" -> 1|>], MergeNested[Last][{<|\"a\" -> 1|>, <|\"a\" -> 2|>}]}",
        )
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[Association[Rule[2, Association[Rule[\"aa\", 1]]]], Association[Rule[\"a\", 2]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn nested_association_additions_leave_malformed_inputs_inert() {
    let mut session = core_session();

    let outcome = session
        .eval_text(
            "{KeyGroupBy[x, f], AssociationKeyDeflatten[x], AssociationKeyFlatten[x], MergeNested[y, Last]}",
        )
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[KeyGroupBy[x, f], AssociationKeyDeflatten[x], AssociationKeyFlatten[x], MergeNested[y, Last]]"
    );
    assert!(outcome.messages.is_empty());
}
