use crate::{associations, install_core};
use rum_core::{Expr, SharedArgs};
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn assert_value(session: &mut Session, source: &str, expected: &str) {
    assert_eq!(
        session.eval_text(source).unwrap().value.to_string(),
        expected
    );
}

fn assert_log(session: &mut Session, expected: &str, expected_len: usize) {
    assert_value(session, "seen", expected);
    assert_value(session, "Length[seen]", &expected_len.to_string());
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}

#[test]
fn target_mode_mixed_list_parent_pins_exact_scan_zero_and_all_shapes() {
    const SOURCE_PREFIX: &str = r#"seen = {};
        {DeleteMissing[
            {rootListBefore,
             Association[
                assocLevelBefore -> assocLevelSurvivor,
                targetList -> {
                    targetBefore,
                    dropImmediate -> Missing[],
                    dropDelayed :> Missing["syntactic"],
                    nestedScanOnly[f[Missing[]]],
                    keepImmediate -> immediateValue,
                    keepDelayed :> delayedValue,
                    targetAfter
                },
                assocLevelAfter -> assocLevelTail
             ],
             rootListAfter},
            3, "#;
    const SOURCE_SUFFIX: &str = "], seen}";

    for (scan, expected) in [
        (
            "0",
            concat!(
                "List[List[rootListBefore, Association[",
                "Rule[assocLevelBefore, assocLevelSurvivor], ",
                "Rule[targetList, List[targetBefore, nestedScanOnly[f[Missing[]]], ",
                "Rule[keepImmediate, immediateValue], ",
                "RuleDelayed[keepDelayed, delayedValue], targetAfter]], ",
                "Rule[assocLevelAfter, assocLevelTail]], rootListAfter], List[]]"
            ),
        ),
        (
            "All",
            concat!(
                "List[List[rootListBefore, Association[",
                "Rule[assocLevelBefore, assocLevelSurvivor], ",
                "Rule[targetList, List[targetBefore, ",
                "Rule[keepImmediate, immediateValue], ",
                "RuleDelayed[keepDelayed, delayedValue], targetAfter]], ",
                "Rule[assocLevelAfter, assocLevelTail]], rootListAfter], List[]]"
            ),
        ),
    ] {
        let mut session = core_session();
        let source = format!("{SOURCE_PREFIX}{scan}{SOURCE_SUFFIX}");
        assert_value(&mut session, &source, expected);
    }
}

#[test]
fn target_mode_mixed_association_parent_pins_exact_scan_zero_and_all_shapes() {
    const SOURCE_PREFIX: &str = r#"seen = {};
        {DeleteMissing[
            Association[
                rootAssocBefore -> rootSurvivor,
                listBranch -> {
                    listLevelBefore,
                    Association[
                        immediateKeep -> immediateValue,
                        delayedKeep :> (AppendTo[seen, delayedKeepRan]; delayedValue),
                        syntaxDrop :> Missing[AppendTo[seen, syntaxDropRan]],
                        scanDrop -> nested[Missing[]],
                        delayedScanDrop :> (
                            AppendTo[seen, delayedScanRan]; nested[Missing[]]
                        ),
                        targetAssocSurvivor -> finalValue
                    ],
                    listLevelAfter
                },
                rootAssocAfter -> rootTail
            ],
            3, "#;
    const SOURCE_SUFFIX: &str = "], seen}";

    for (scan, expected) in [
        (
            "0",
            concat!(
                "List[Association[Rule[rootAssocBefore, rootSurvivor], ",
                "Rule[listBranch, List[listLevelBefore, Association[",
                "Rule[immediateKeep, immediateValue], ",
                "RuleDelayed[delayedKeep, CompoundExpression[",
                "AppendTo[seen, delayedKeepRan], delayedValue]], ",
                "Rule[scanDrop, nested[Missing[]]], ",
                "RuleDelayed[delayedScanDrop, CompoundExpression[",
                "AppendTo[seen, delayedScanRan], nested[Missing[]]]], ",
                "Rule[targetAssocSurvivor, finalValue]], listLevelAfter]], ",
                "Rule[rootAssocAfter, rootTail]], ",
                "List[delayedKeepRan, delayedScanRan]]"
            ),
        ),
        (
            "All",
            concat!(
                "List[Association[Rule[rootAssocBefore, rootSurvivor], ",
                "Rule[listBranch, List[listLevelBefore, Association[",
                "Rule[immediateKeep, immediateValue], ",
                "RuleDelayed[delayedKeep, CompoundExpression[",
                "AppendTo[seen, delayedKeepRan], delayedValue]], ",
                "Rule[targetAssocSurvivor, finalValue]], listLevelAfter]], ",
                "Rule[rootAssocAfter, rootTail]], ",
                "List[delayedKeepRan, delayedScanRan]]"
            ),
        ),
    ] {
        let mut session = core_session();
        let source = format!("{SOURCE_PREFIX}{scan}{SOURCE_SUFFIX}");
        assert_value(&mut session, &source, expected);
    }
}

#[test]
fn target_association_evaluates_only_reached_delayed_rhs_once() {
    let mut session = core_session();
    assert_value(&mut session, "seen = {}", "List[]");

    // Call the builtin directly with a lowered, unevaluated Association so the
    // walker's zero evaluations of an immediate Rule RHS remain observable.
    let input = session
        .lower_text(
            r#"Association[
                immediate -> (AppendTo[seen, immediateRan]; immediateValue),
                delayed :> (AppendTo[seen, delayedRan]; delayedValue),
                delayedMissing :> (AppendTo[seen, delayedMissingRan]; Missing[])
            ]"#,
        )
        .unwrap();
    let head = session.lower_text("DeleteMissing").unwrap();
    let args = SharedArgs::new(vec![input, Expr::small_integer(1), Expr::small_integer(0)]);

    let result = associations::eval_delete_missing(&mut session, &head, &args).unwrap();
    assert_eq!(
        result.to_string(),
        concat!(
            "Association[Rule[immediate, CompoundExpression[",
            "AppendTo[seen, immediateRan], immediateValue]], ",
            "RuleDelayed[delayed, CompoundExpression[",
            "AppendTo[seen, delayedRan], delayedValue]]]"
        )
    );
    assert_log(&mut session, "List[delayedRan, delayedMissingRan]", 2);
}

#[test]
fn delayed_rhs_above_target_gets_one_level_minus_one_cleanup_wrapper() {
    let mut session = core_session();
    assert_value(
        &mut session,
        concat!(
            "seen = {}; ",
            "dm = DeleteMissing[Association[",
            "before -> beforeValue, ",
            "deferred :> (AppendTo[seen, deferredRan]; ",
            "{levelOne, {drop -> Missing[], keep -> kept}}), ",
            "after -> afterValue], 3, 0]; ",
            "{dm, seen, dm[deferred], seen}"
        ),
        concat!(
            "List[Association[Rule[before, beforeValue], ",
            "RuleDelayed[deferred, With[List[Rule[rum$deleteMissingValue, ",
            "CompoundExpression[AppendTo[seen, deferredRan], ",
            "List[levelOne, List[Rule[drop, Missing[]], Rule[keep, kept]]]]]], ",
            "If[Or[ListQ[rum$deleteMissingValue], ",
            "AssociationQ[rum$deleteMissingValue]], ",
            "DeleteMissing[rum$deleteMissingValue, 2, 0], ",
            "rum$deleteMissingValue]]], Rule[after, afterValue]], ",
            "List[], List[levelOne, List[Rule[keep, kept]]], ",
            "List[deferredRan]]"
        ),
    );
    assert_log(&mut session, "List[deferredRan]", 1);
}

fn assert_single_sibling_suspension(
    source: &str,
    prompt: &str,
    log_at_suspension: &str,
    log_len_at_suspension: usize,
    expected_value: &str,
    final_log: &str,
    final_log_len: usize,
) {
    let mut resumed_session = core_session();
    let failure = resumed_session
        .eval_request(source)
        .expect_err("the selected target sibling should request input");
    assert_input_prompt(&failure.error, prompt);
    assert_log(
        &mut resumed_session,
        log_at_suspension,
        log_len_at_suspension,
    );

    let resumed = resumed_session
        .resume_eval_request_with_input("ready")
        .expect("DeleteMissing should finish after the selected sibling resumes");
    assert_eq!(resumed.value.to_string(), expected_value);
    assert!(resumed.messages.is_empty());
    assert_log(&mut resumed_session, final_log, final_log_len);

    let mut queued_session = core_session();
    queued_session.queue_input_response("ready");
    let queued = queued_session
        .eval_request(source)
        .expect("queued input should follow the same target-sibling path");
    assert_eq!(queued.value, resumed.value);
    assert_eq!(queued.messages, resumed.messages);
    assert_log(&mut queued_session, final_log, final_log_len);
}

#[test]
fn target_list_parent_resumes_first_delayed_sibling_without_replay() {
    const SOURCE: &str = r#"seen = {};
        firstCallback := (
            AppendTo[seen, firstBefore]; InputString["first: "];
            AppendTo[seen, firstAfter]; Missing[]
        );
        middleCallback := (AppendTo[seen, middleRan]; middleValue);
        lastCallback := (AppendTo[seen, lastRan]; lastValue);
        DeleteMissing[
            {outerBefore,
             Association[
                first :> firstCallback,
                middle :> middleCallback,
                last :> lastCallback
             ],
             outerAfter},
            2, 0
        ]"#;

    assert_single_sibling_suspension(
        SOURCE,
        "first: ",
        "List[firstBefore]",
        1,
        concat!(
            "List[outerBefore, Association[",
            "RuleDelayed[middle, middleCallback], ",
            "RuleDelayed[last, lastCallback]], outerAfter]"
        ),
        "List[firstBefore, firstAfter, middleRan, lastRan]",
        4,
    );
}

#[test]
fn target_list_parent_resumes_middle_delayed_sibling_without_prefix_replay() {
    const SOURCE: &str = r#"seen = {};
        firstCallback := (AppendTo[seen, firstRan]; firstValue);
        middleCallback := (
            AppendTo[seen, middleBefore]; InputString["middle: "];
            AppendTo[seen, middleAfter]; Missing[]
        );
        lastCallback := (AppendTo[seen, lastRan]; lastValue);
        DeleteMissing[
            {outerBefore,
             Association[
                first :> firstCallback,
                middle :> middleCallback,
                last :> lastCallback
             ],
             outerAfter},
            2, 0
        ]"#;

    assert_single_sibling_suspension(
        SOURCE,
        "middle: ",
        "List[firstRan, middleBefore]",
        2,
        concat!(
            "List[outerBefore, Association[",
            "RuleDelayed[first, firstCallback], ",
            "RuleDelayed[last, lastCallback]], outerAfter]"
        ),
        "List[firstRan, middleBefore, middleAfter, lastRan]",
        4,
    );
}

#[test]
fn target_list_parent_resumes_last_delayed_sibling_without_prefix_replay() {
    const SOURCE: &str = r#"seen = {};
        firstCallback := (AppendTo[seen, firstRan]; firstValue);
        middleCallback := (AppendTo[seen, middleRan]; middleValue);
        lastCallback := (
            AppendTo[seen, lastBefore]; InputString["last: "];
            AppendTo[seen, lastAfter]; Missing[]
        );
        DeleteMissing[
            {outerBefore,
             Association[
                first :> firstCallback,
                middle :> middleCallback,
                last :> lastCallback
             ],
             outerAfter},
            2, 0
        ]"#;

    assert_single_sibling_suspension(
        SOURCE,
        "last: ",
        "List[firstRan, middleRan, lastBefore]",
        3,
        concat!(
            "List[outerBefore, Association[",
            "RuleDelayed[first, firstCallback], ",
            "RuleDelayed[middle, middleCallback]], outerAfter]"
        ),
        "List[firstRan, middleRan, lastBefore, lastAfter]",
        4,
    );
}

#[test]
fn target_association_parent_resumes_two_different_list_branches_in_order() {
    const SOURCE: &str = r#"seen = {};
        onePrefixCallback := (AppendTo[seen, onePrefixRan]; onePrefixValue);
        oneSuspendCallback := (
            AppendTo[seen, oneBefore]; InputString["branch-one: "];
            AppendTo[seen, oneAfter]; Missing[]
        );
        oneTailCallback := (AppendTo[seen, oneTailRan]; oneTailValue);
        twoPrefixCallback := (AppendTo[seen, twoPrefixRan]; twoPrefixValue);
        twoSuspendCallback := (
            AppendTo[seen, twoBefore]; InputString["branch-two: "];
            AppendTo[seen, twoAfter]; Missing[]
        );
        twoTailCallback := (AppendTo[seen, twoTailRan]; twoTailValue);
        DeleteMissing[
            Association[
                rootBefore -> rootBeforeValue,
                listBranch -> {
                    listBefore,
                    Association[
                        onePrefix :> onePrefixCallback,
                        oneSuspend :> oneSuspendCallback,
                        oneTail :> oneTailCallback
                    ],
                    listBetween,
                    Association[
                        twoPrefix :> twoPrefixCallback,
                        twoSuspend :> twoSuspendCallback,
                        twoTail :> twoTailCallback
                    ],
                    listAfter
                },
                rootAfter -> rootAfterValue
            ],
            3, All
        ]"#;
    const EXPECTED: &str = concat!(
        "Association[Rule[rootBefore, rootBeforeValue], ",
        "Rule[listBranch, List[listBefore, Association[",
        "RuleDelayed[onePrefix, onePrefixCallback], ",
        "RuleDelayed[oneTail, oneTailCallback]], listBetween, Association[",
        "RuleDelayed[twoPrefix, twoPrefixCallback], ",
        "RuleDelayed[twoTail, twoTailCallback]], listAfter]], ",
        "Rule[rootAfter, rootAfterValue]]"
    );
    const FINAL_LOG: &str = concat!(
        "List[onePrefixRan, oneBefore, oneAfter, oneTailRan, ",
        "twoPrefixRan, twoBefore, twoAfter, twoTailRan]"
    );

    let mut resumed_session = core_session();
    let first = resumed_session
        .eval_request(SOURCE)
        .expect_err("the first target branch should request input");
    assert_input_prompt(&first.error, "branch-one: ");
    assert_log(&mut resumed_session, "List[onePrefixRan, oneBefore]", 2);

    let second = resumed_session
        .resume_eval_request_with_input("one")
        .expect_err("the second target branch should request input next");
    assert_input_prompt(&second.error, "branch-two: ");
    assert_log(
        &mut resumed_session,
        concat!(
            "List[onePrefixRan, oneBefore, oneAfter, oneTailRan, ",
            "twoPrefixRan, twoBefore]"
        ),
        6,
    );

    let resumed = resumed_session
        .resume_eval_request_with_input("two")
        .expect("DeleteMissing should finish after both branches resume");
    assert_eq!(resumed.value.to_string(), EXPECTED);
    assert!(resumed.messages.is_empty());
    assert_log(&mut resumed_session, FINAL_LOG, 8);

    let mut queued_session = core_session();
    queued_session.queue_input_response("one");
    queued_session.queue_input_response("two");
    let queued = queued_session
        .eval_request(SOURCE)
        .expect("queued input should preserve the same two-branch schedule");
    assert_eq!(queued.value, resumed.value);
    assert_eq!(queued.messages, resumed.messages);
    assert_log(&mut queued_session, FINAL_LOG, 8);
}

#[test]
fn target_mode_propagates_non_input_errors_unchanged() {
    let mut session = core_session();
    let response = session
        .eval_request(
            r#"seen = {};
               beforeCallback := (AppendTo[seen, beforeRan]; beforeValue);
               throwingCallback := (
                   AppendTo[seen, throwRan]; Throw[thrownValue, deleteMissingTag]
               );
               afterCallback := (AppendTo[seen, afterRan]; afterValue);
               {Catch[
                    DeleteMissing[
                        {outerBefore,
                         Association[
                            before :> beforeCallback,
                            throwing :> throwingCallback,
                            after :> afterCallback
                         ],
                         outerAfter},
                        2, 0
                    ],
                    deleteMissingTag
                ], seen}"#,
        )
        .expect("Catch should receive DeleteMissing's unchanged throw");

    assert_eq!(
        response.value.to_string(),
        "List[thrownValue, List[beforeRan, throwRan]]"
    );
    assert!(response.messages.is_empty());
    assert_log(&mut session, "List[beforeRan, throwRan]", 2);
}

#[test]
fn cancelling_target_mode_suspension_discards_frames_without_callbacks() {
    const SOURCE: &str = r#"seen = {};
        beforeCallback := (AppendTo[seen, beforeRan]; beforeValue);
        suspendCallback := (
            AppendTo[seen, suspendBefore]; InputString["cancel: "];
            AppendTo[seen, suspendAfter]; suspendValue
        );
        afterCallback := (AppendTo[seen, afterRan]; afterValue);
        DeleteMissing[
            {outerBefore,
             Association[
                before :> beforeCallback,
                pending :> suspendCallback,
                after :> afterCallback
             ],
             outerAfter},
            2, 0
        ]"#;

    let mut session = core_session();
    let accepted = session
        .accept_cancellable_request()
        .expect("the idle session should accept a cancellable request");
    let cancel = accepted.cancel_handle();
    let failure = session
        .eval_accepted_request(accepted, SOURCE)
        .expect_err("the target callback should suspend before cancellation");
    assert_input_prompt(&failure.error, "cancel: ");

    cancel.cancel();
    let response = session
        .resume_eval_request_with_input("ignored")
        .expect("cancellation should abandon the suspended DeleteMissing request");
    assert_eq!(
        response.termination,
        Some(rum_runtime::EvalTerminationReason::Cancelled)
    );
    assert!(session.pending_input_request().is_none());
    assert_log(&mut session, "List[beforeRan, suspendBefore]", 2);

    let follow_up = session
        .eval_request("{seen, 1 + 1}")
        .expect("the session should remain usable after frame abandonment");
    assert_eq!(
        follow_up.value.to_string(),
        "List[List[beforeRan, suspendBefore], 2]"
    );
}
