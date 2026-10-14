use super::observe::{Output, Rejection};
use rum_core::Expr;
use rum_runtime::Session;

pub fn new() -> Session {
    let mut s = Session::new();
    let _fault_check = super::faults::Check::new(&s);
    rum_stdlib::install_core(&mut s);
    s.eval_request("Global`log={};Global`probe::tag=\"oracle message\"")
        .unwrap();
    s
}

pub fn retire(s: &mut Session) {
    let _fault_check = super::faults::Check::new(s);
    if s.pending_input_request().is_some() || s.reject_overlapping_evaluation_request().is_err() {
        s.request_cancel_handle().cancel();
        let _ = s.finish_cancelled_pending_evaluation_request();
    }
}

pub fn fresh(s: &mut Session) -> Result<(), Rejection> {
    let _fault_check = super::faults::Check::new(s);
    match s.eval_request("1+1") {
        Ok(r) if r.value == Expr::small_integer(2) && r.termination.is_none() => Ok(()),
        _ => Err(Rejection::FreshRequest),
    }
}

// A surfaced sentinel is evidence of an empty queue. It is cancelled before
// another request; no unplanned answer is consumed to make this check pass.
pub fn empty_queue(s: &mut Session) -> Result<(), Rejection> {
    let _fault_check = super::faults::Check::new(s);
    let out: Output = s.eval_request("InputString[\"oracle queue sentinel\"]");
    let empty = matches!(out, Err(ref f) if matches!(&f.error,
        rum_runtime::SessionRequestError::InputRequired { request }
            if request.prompt == "oracle queue sentinel" && request.kind == rum_runtime::RuntimeInputKind::InputString));
    retire(s);
    if !empty {
        return Err(Rejection::QueueNotEmpty);
    }
    fresh(s)
}
