use super::support::{finish, session};
use rum_runtime::SessionRequestError;
fn cancel(stop: usize, without_answer: bool) {
    let source = r#"Unprotect[Count];SetOptions[Count,Heads:>(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];False)];Count[{1,2,3},_Integer]"#;
    let mut s = session();
    let handle = s.request_cancel_handle();
    let mut out = s.eval_request(source);
    for i in 0..=stop {
        let Err(f) = &out else {
            panic!("must suspend at {i}");
        };
        let SessionRequestError::InputRequired { request } = &f.error else {
            panic!("must ask for input {f:?}");
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
    println!(
        "lifecycle\t{}",
        format!("cancel-stop-{stop}-without-answer-{without_answer}\t{v:?}\t{log}")
    );
    assert_eq!(v.termination, "Some(Cancelled)");
    assert_eq!(v.value, "TerminatedEvaluation[\"Cancelled\"]");
    assert_eq!(log, if stop == 0 { "List[1]" } else { "List[1, 2]" });
    assert_eq!(v.prints, vec!["before"]);
    assert_eq!(
        v.messages
            .iter()
            .map(|m| (m.symbol.as_str(), m.tag.as_str()))
            .collect::<Vec<_>>(),
        vec![("probe", "tag"), ("General", "cancel")]
    );
}
#[test]
fn cancel_first() {
    cancel(0, false);
}
#[test]
fn cancel_second() {
    cancel(1, false);
}
#[test]
fn abandon_first_without_answer() {
    cancel(0, true);
}
#[test]
fn abandon_second_without_answer() {
    cancel(1, true);
}

#[test]
fn phased_read_cancellation_and_abandonment_run_no_pending_work() {
    for (name, source) in [
        (
            "Monitor",
            r#"Unprotect[Monitor];SetOptions[Monitor,EvaluationMonitor->v,StepMonitor->w];v:=(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];None);w:=(AppendTo[log,2];InputString["second: "];AppendTo[log,3];None);Monitor[(AppendTo[log,99];77),(AppendTo[log,98];0)]"#,
        ),
        (
            "Names",
            r#"Unprotect[Names];SetOptions[Names,SpellingCorrection->v,ResolveContextAliases->w,IgnoreCase->z];v:=(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];False);w:=(AppendTo[log,2];InputString["second: "];AppendTo[log,3];True);z:=(AppendTo[log,99];False);Names["Global`never578"]"#,
        ),
        (
            "TestReport",
            r#"Unprotect[TestReport];SetOptions[TestReport,SameTest->v,TimeConstraint->w,MemoryConstraint->z];v:=(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];SameQ);w:=(AppendTo[log,2];InputString["second: "];AppendTo[log,3];Infinity);z:=(AppendTo[log,99];Infinity);TestReport[{TestCreate[(AppendTo[log,98];1),1]},"TestsSucceededCount",{}]"#,
        ),
        (
            "DumpSave",
            r#"Unprotect[DumpSave];SetOptions[DumpSave,CompressionLevel->v];v:=(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];1);DumpSave[".temp/brief-578/mock.rum",{}]"#,
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
                    assert_eq!(
                        f.prints,
                        ["before"],
                        "no display/body/report stream before reads finish"
                    );
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
