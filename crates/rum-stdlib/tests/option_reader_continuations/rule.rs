use super::support::compare;

#[test]
fn oracle_arrayfilter_option() {
    compare(
        r###"ArrayFilter-option"###,
        r###"Unprotect[ArrayFilter];SetOptions[ArrayFilter,Padding:>(AppendTo[log,2];InputString["p: "];0)];ArrayFilter[Total,{1,2,3},1]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_checkabort_option() {
    compare(
        r###"CheckAbort-option"###,
        r###"Unprotect[CheckAbort];SetOptions[CheckAbort,PropagateAborts:>(AppendTo[log,2];InputString["p: "];True)];CheckAbort[(AppendTo[log,3];7),9]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn oracle_arrayfilter_option_twice576() {
    compare(
        r###"ArrayFilter-option-twice576"###,
        r###"Unprotect[ArrayFilter];SetOptions[ArrayFilter,Padding:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];0)];ArrayFilter[Total,{1,2,3},1]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_checkabort_option_twice576() {
    compare(
        r###"CheckAbort-option-twice576"###,
        r###"Unprotect[CheckAbort];SetOptions[CheckAbort,PropagateAborts:>(AppendTo[log,2];InputString["p: "];AppendTo[log,20];InputString["second: "];AppendTo[log,21];True)];CheckAbort[(AppendTo[log,3];7),9]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"second: "###],
    );
}

#[test]
fn oracle_arrayfilter_unread_default() {
    compare(
        r###"ArrayFilter-unread-default"###,
        r###"Unprotect[ArrayFilter];SetOptions[ArrayFilter,Padding:>(AppendTo[log,99];InputString["unread: "];0)];ArrayFilter[Total,{1,2,3},1,Padding->0]"###,
        &[],
        &[],
    );
}

#[test]
fn checkabort_default_before_explicit_rhs_and_body() {
    compare(
        "CheckAbort-default-explicit-body",
        r#"Unprotect[CheckAbort];SetOptions[CheckAbort,PropagateAborts:>(AppendTo[log,1];InputString["default: "];AppendTo[log,2];True)];CheckAbort[(AppendTo[log,5];InputString["body: "];AppendTo[log,6];7),(AppendTo[log,99];9),PropagateAborts->(AppendTo[log,3];InputString["explicit: "];AppendTo[log,4];False)]"#,
        &["a", "b", "c"],
        &["default: ", "explicit: ", "body: "],
    );
}

#[test]
fn checkabort_default_before_primary_and_fallback_suspensions() {
    compare(
        "CheckAbort-default-primary-fallback",
        r#"Unprotect[CheckAbort];SetOptions[CheckAbort,PropagateAborts:>(AppendTo[log,1];InputString["default: "];False)];CheckAbort[(AppendTo[log,2];InputString["body: "];Abort[]),(AppendTo[log,3];InputString["fallback: "];AppendTo[log,4];9)]"#,
        &["a", "b", "c"],
        &["default: ", "body: ", "fallback: "],
    );
}

#[test]
fn arrayfilter_operator_suspends_after_default_without_replaying_arguments() {
    compare(
        "ArrayFilter-default-operator",
        r#"Unprotect[ArrayFilter];SetOptions[ArrayFilter,Padding:>(AppendTo[log,3];InputString["default: "];0)];op[x_]:=(AppendTo[log,4];InputString["operator: "];Total[x]);ArrayFilter[op,(AppendTo[log,1];{1,2,3}),(AppendTo[log,2];1)]"#,
        &["a", "b", "c", "d"],
        &["default: ", "operator: ", "operator: ", "operator: "],
    );
}

#[test]
fn rule_read_cancel_and_abandon_run_no_pending_guest_work() {
    use super::support::{finish, session};
    use rum_runtime::SessionRequestError;
    for (name, source) in [
        (
            "CheckAbort",
            r#"Unprotect[CheckAbort];SetOptions[CheckAbort,PropagateAborts:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)];CheckAbort[(AppendTo[log,99];7),(AppendTo[log,98];9),PropagateAborts->(AppendTo[log,97];False)]"#,
        ),
        (
            "ArrayFilter",
            r#"Unprotect[ArrayFilter];SetOptions[ArrayFilter,Padding:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];0)];op[x_]:=(AppendTo[log,99];Total[x]);ArrayFilter[op,{1,2,3},1]"#,
        ),
    ] {
        for stop in [0, 1] {
            for without_answer in [false, true] {
                let mut s = session();
                let handle = s.request_cancel_handle();
                let mut out = s.eval_request(source);
                for i in 0..=stop {
                    let Err(f) = &out else {
                        panic!("{name}: must suspend at {i}");
                    };
                    let SessionRequestError::InputRequired { request } = &f.error else {
                        panic!("{name}: {f:?}");
                    };
                    assert_eq!(request.prompt, if i == 0 { "first: " } else { "second: " });
                    assert_eq!(s.pending_input_request(), Some(request));
                    assert!(s.reject_overlapping_evaluation_request().is_err());
                    if i < stop {
                        out = s.resume_eval_request_with_input("a");
                    }
                }
                handle.cancel();
                out = if without_answer {
                    Ok(s.finish_cancelled_pending_evaluation_request()
                        .unwrap()
                        .expect("must abandon"))
                } else {
                    s.resume_eval_request_with_input("unused")
                };
                let (v, log) = finish(&mut s, out);
                assert_eq!(v.termination, "Some(Cancelled)");
                assert_eq!(v.value, "TerminatedEvaluation[\"Cancelled\"]");
                assert_eq!(log, if stop == 0 { "List[1]" } else { "List[1, 2]" });
                assert_eq!(v.prints, ["before"]);
                assert_eq!(
                    v.messages
                        .iter()
                        .map(|m| (m.symbol.as_str(), m.tag.as_str()))
                        .collect::<Vec<_>>(),
                    [("probe", "tag"), ("General", "cancel")]
                );
                println!(
                    "lifecycle\t{name}\tstop={stop}\twithout_answer={without_answer}\t{v:?}\t{log}"
                );
            }
        }
    }
}
