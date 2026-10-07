use rum_host::{HostEvalOutcome, HostInputKind, HostSession};

fn assert_success(outcome: HostEvalOutcome, expected: &str) {
    let HostEvalOutcome::Success(success) = outcome else {
        panic!("expected success {expected}, got {outcome:?}");
    };
    assert_eq!(success.value.text, expected);
    assert!(success.termination.is_none());
    assert!(success.messages.is_empty());
    assert!(success.prints.is_empty());
}

#[test]
fn shared_accumulators_complete_and_leave_the_host_session_usable() {
    for (setup, source, prompts, answers, expected, projection) in [
        (
            "Null",
            r#"TestReport[{TestCreate[InputString["report:1"],"a"],TestCreate[InputString["report:2"],"b"]},"TestsSucceededCount"]"#,
            ["report:1", "report:2"],
            ["a", "b"],
            "2",
            "Null",
        ),
        (
            "Null",
            r#"TestReport[{TestCreate["a",InputString["expected:1"]],TestCreate["b",InputString["expected:2"]]},"TestsSucceededCount"]"#,
            ["expected:1", "expected:2"],
            ["a", "b"],
            "2",
            "Null",
        ),
        (
            "Null",
            r#"TestReport[{TestCreate[1,1],TestCreate[2,2]},"TestsSucceededCount",SameTest->Function[{a,b},(InputString["same"];SameQ[a,b])]]"#,
            ["same", "same"],
            ["a", "b"],
            "2",
            "Null",
        ),
        (
            r#"Global`assoc=<|"a"-><|"b"->0|>|>;Null"#,
            r#"Global`assoc[InputString["assoc:1"],InputString["assoc:2"]]=9"#,
            ["assoc:1", "assoc:2"],
            ["a", "b"],
            "9",
            r#"Global`assoc["a","b"]"#,
        ),
        (
            "Null",
            r#"DeleteCases[{1,2},_?Function[x,(InputString["delete"];False)]]"#,
            ["delete", "delete"],
            ["a", "b"],
            "List[1, 2]",
            "Null",
        ),
        (
            "Null",
            r#"DeleteCases[{1,2,3},_?Function[x,(If[x<=2,InputString["mixed"]];x==2)]]"#,
            ["mixed", "mixed"],
            ["a", "b"],
            "List[1, 3]",
            "Null",
        ),
        (
            "Null",
            r#"DeleteCases[<|"a"->1,"b"->2|>,_?Function[x,(InputString["children"];False)]]"#,
            ["children", "children"],
            ["a", "b"],
            r#"Association[Rule["a", 1], Rule["b", 2]]"#,
            "Null",
        ),
        (
            "Null",
            r#"DeleteCases[{1,2,3,4},_?Function[x,(InputString["limit"];x==2)],{1},1]"#,
            ["limit", "limit"],
            ["a", "b"],
            "List[1, 3, 4]",
            "Null",
        ),
    ] {
        let mut session = HostSession::new();
        assert_success(session.eval(setup), "Null");
        let mut outcome = session.eval(source);
        for (prompt, answer) in prompts.into_iter().zip(answers) {
            let HostEvalOutcome::InputRequired(suspension) = &outcome else {
                panic!("expected {prompt} from {source}, got {outcome:?}");
            };
            assert_eq!(suspension.request.kind, HostInputKind::InputString);
            assert_eq!(suspension.request.prompt, prompt);
            assert_eq!(
                session.pending_input_request(),
                Some(suspension.request.clone())
            );
            outcome = session.resume_with_input(answer);
        }
        assert_success(outcome, expected);
        assert!(session.pending_input_request().is_none());
        assert_success(
            session.eval(projection),
            if projection == "Null" { "Null" } else { "9" },
        );
        assert_success(session.eval("6*7"), "42");
        let sentinel = session.eval("InputString[\"fresh request\"]");
        assert!(
            matches!(sentinel, HostEvalOutcome::InputRequired(ref pause) if pause.request.prompt == "fresh request")
        );
        assert_success(
            session.resume_with_input("fresh answer"),
            "\"fresh answer\"",
        );
        assert!(session.pending_input_request().is_none());
        println!("HOST_SHARED_COMPLETION {source} => {expected} PASS");
    }
}
