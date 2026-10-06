use super::support::{finish, record, session};
use rum_runtime::SessionRequestError;

fn cancellation(label: &str, source: &str, stop: usize, without_answer: bool) {
    let mut s = session();
    let handle = s.request_cancel_handle();
    let mut out = s.eval_request(source);
    for i in 0..=stop {
        let Err(f) = &out else {
            panic!("{label}: must suspend at {i}");
        };
        let SessionRequestError::InputRequired { request } = &f.error else {
            panic!("{label}: {f:?}");
        };
        assert_eq!(request.prompt, if i == 0 { "first: " } else { "second: " });
        assert_eq!(s.pending_input_request(), Some(request));
        assert!(s.reject_overlapping_evaluation_request().is_err());
        assert_eq!(
            f.prints,
            ["before"],
            "no completed TracePrint stream during filtering"
        );
        if i < stop {
            out = s.resume_eval_request_with_input("a");
        }
    }
    handle.cancel();
    let cancelled = if without_answer {
        Ok(s.finish_cancelled_pending_evaluation_request()
            .unwrap()
            .expect("must abandon"))
    } else {
        s.resume_eval_request_with_input("unused")
    };
    let (v, log) = finish(&mut s, cancelled);
    record(
        "lifecycle.tsv",
        &format!("{label}\tstop={stop}\twithout_answer={without_answer}\t{v:?}\t{log}"),
    );
    assert_eq!(v.termination, "Some(Cancelled)");
    assert_eq!(v.value, "TerminatedEvaluation[\"Cancelled\"]");
    assert_eq!(v.prints, ["before"]);
    assert_eq!(log, if stop == 0 { "List[1]" } else { "List[1, 2]" });
    assert_eq!(
        v.messages
            .iter()
            .map(|m| (m.symbol.as_str(), m.tag.as_str()))
            .collect::<Vec<_>>(),
        [("probe", "tag"), ("General", "cancel")]
    );
    assert_eq!(
        s.eval_request("Reap[Sow[9]]").unwrap().value.to_string(),
        "List[9, List[List[9]]]"
    );
    assert_eq!(
        s.eval_request("Trace[1+1]").unwrap().value.to_string(),
        "List[Hold[Plus[1, 1]], Hold[2]]"
    );
}

#[test]
fn sow_cancel_0_false() {
    cancellation(
        "Sow-cancel-0-false",
        r###"Reap[Sow[7,2],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        0,
        false,
    );
}

#[test]
fn sow_cancel_0_true() {
    cancellation(
        "Sow-cancel-0-true",
        r###"Reap[Sow[7,2],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        0,
        true,
    );
}

#[test]
fn sow_cancel_1_false() {
    cancellation(
        "Sow-cancel-1-false",
        r###"Reap[Sow[7,2],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        1,
        false,
    );
}

#[test]
fn sow_cancel_1_true() {
    cancellation(
        "Sow-cancel-1-true",
        r###"Reap[Sow[7,2],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        1,
        true,
    );
}

#[test]
fn trace_cancel_0_false() {
    cancellation(
        "Trace-cancel-0-false",
        r###"Trace[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        0,
        false,
    );
}

#[test]
fn trace_cancel_0_true() {
    cancellation(
        "Trace-cancel-0-true",
        r###"Trace[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        0,
        true,
    );
}

#[test]
fn trace_cancel_1_false() {
    cancellation(
        "Trace-cancel-1-false",
        r###"Trace[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        1,
        false,
    );
}

#[test]
fn trace_cancel_1_true() {
    cancellation(
        "Trace-cancel-1-true",
        r###"Trace[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        1,
        true,
    );
}

#[test]
fn tracedetail_cancel_0_false() {
    cancellation(
        "TraceDetail-cancel-0-false",
        r###"TraceDetail[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        0,
        false,
    );
}

#[test]
fn tracedetail_cancel_0_true() {
    cancellation(
        "TraceDetail-cancel-0-true",
        r###"TraceDetail[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        0,
        true,
    );
}

#[test]
fn tracedetail_cancel_1_false() {
    cancellation(
        "TraceDetail-cancel-1-false",
        r###"TraceDetail[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        1,
        false,
    );
}

#[test]
fn tracedetail_cancel_1_true() {
    cancellation(
        "TraceDetail-cancel-1-true",
        r###"TraceDetail[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        1,
        true,
    );
}

#[test]
fn traceprint_cancel_0_false() {
    cancellation(
        "TracePrint-cancel-0-false",
        r###"TracePrint[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        0,
        false,
    );
}

#[test]
fn traceprint_cancel_0_true() {
    cancellation(
        "TracePrint-cancel-0-true",
        r###"TracePrint[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        0,
        true,
    );
}

#[test]
fn traceprint_cancel_1_false() {
    cancellation(
        "TracePrint-cancel-1-false",
        r###"TracePrint[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        1,
        false,
    );
}

#[test]
fn traceprint_cancel_1_true() {
    cancellation(
        "TracePrint-cancel-1-true",
        r###"TracePrint[f[1+2,3+4],_?Function[t,(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["first: "];AppendTo[log,2];InputString["second: "];AppendTo[log,3];True)]]"###,
        1,
        true,
    );
}
