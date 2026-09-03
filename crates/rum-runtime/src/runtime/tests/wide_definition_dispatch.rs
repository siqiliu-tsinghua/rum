use super::*;

// The value dispatchers and the rule/bucket walks used to advance by calling
// themselves (directly or from a drive_internal_value continuation), so the
// native stack depth equaled the candidate/rule/bucket count and a wide input
// aborted the process. These tests pin the iterative walks at a width far
// past every measured abort threshold, plus the suspension-resume paths the
// loop conversions re-enter through their pending outcomes.

#[test]
fn replace_list_walks_thirty_thousand_rules() {
    let mut session = session_with_test_rewrite_builtins();

    let rules = (1..=30_000)
        .map(|i| format!("{i} -> 0"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!("ReplaceList[1, {{{rules}}}]");
    let outcome = session.eval_text(&source).unwrap();

    assert_eq!(outcome.value.to_string(), "List[0]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn reap_walks_thirty_thousand_buckets() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("Reap[Do[Sow[i, i], {i, 1, 30000}]]")
        .unwrap();
    let Expr::Call { head, args } = outcome.value else {
        panic!("Reap should return a list pair, got {}", outcome.value);
    };
    assert!(is_system_symbol_name(&head, "List"));
    assert_eq!(args.len(), 2);

    let Expr::Call {
        head: buckets_head,
        args: buckets,
    } = &args[1]
    else {
        panic!("Reap harvest should be a list, got {}", args[1]);
    };
    assert!(is_system_symbol_name(buckets_head, "List"));
    assert_eq!(buckets.len(), 30_000);
    assert_eq!(buckets[0].to_string(), "List[1]");
    assert_eq!(buckets[29_999].to_string(), "List[30000]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn up_value_dispatch_walks_thirty_thousand_definitions() {
    let mut session = session_with_test_control_builtins();

    // A single statement chain is capped at 256 statements, so the
    // definitions go in in chunks; session definitions persist across calls.
    let definitions: Vec<String> = (1..=30_000)
        .map(|i| format!("g /: h{i}[g] := {i};"))
        .collect();
    for chunk in definitions.chunks(250) {
        session.eval_text(&chunk.join(" ")).unwrap();
    }

    let outcome = session.eval_text("h15000[g]").unwrap();
    assert_eq!(outcome.value.to_string(), "15000");
    assert!(outcome.messages.is_empty());
}

#[test]
fn sub_value_dispatch_walks_thirty_thousand_definitions() {
    let mut session = session_with_test_control_builtins();

    let definitions: Vec<String> = (1..=30_000)
        .map(|i| format!("s[{i}][x_] := {i};"))
        .collect();
    for chunk in definitions.chunks(250) {
        session.eval_text(&chunk.join(" ")).unwrap();
    }

    let outcome = session.eval_text("s[15000][1]").unwrap();
    assert_eq!(outcome.value.to_string(), "15000");
    assert!(outcome.messages.is_empty());
}

#[test]
fn up_value_dispatch_resumes_mid_walk_with_pending_outcome() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "g /: f[g, x_ /; MatchQ[InputString[\"ucond: \"], \"yes\"]] := x; \
             g /: f[g, 0] := never;",
        )
        .unwrap();

    let failure = session
        .eval_request("f[g, 1]")
        .expect_err("conditioned up-value match should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "ucond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish the conditioned up-value match");

    assert_eq!(response.value.to_string(), "1");
}

#[test]
fn sub_value_dispatch_resumes_mid_walk_with_pending_outcome() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "s[1][x_ /; MatchQ[InputString[\"scond: \"], \"yes\"]] := x; \
             s[1][0] := never;",
        )
        .unwrap();

    let failure = session
        .eval_request("s[1][5]")
        .expect_err("conditioned sub-value match should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "scond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish the conditioned sub-value match");

    assert_eq!(response.value.to_string(), "5");
}

#[test]
fn up_value_dispatch_resumes_to_next_candidate_when_condition_fails() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text(
            "g /: f[g, x_ /; MatchQ[InputString[\"ucond2: \"], \"yes\"]] := x; \
             g /: f[g, 0] := never;",
        )
        .unwrap();

    let failure = session
        .eval_request("f[g, 1]")
        .expect_err("conditioned up-value match should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "ucond2: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("no")
        .expect("resume should walk past the failing up-value candidate");

    assert_eq!(response.value.to_string(), "f[g, 1]");
}
