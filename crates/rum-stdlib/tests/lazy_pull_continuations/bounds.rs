//! Eager LazyTable bounds resume before validation and held-plan construction.
use super::*;

probe!(
    lazytable_bound_1,
    "LazyTable-bound-1",
    r###"LazyTable[i,{i,(AppendTo[log,1];Input["p: "]),(AppendTo[log,2];3),(AppendTo[log,3];1)}]"###,
    &["1"],
    None
);
probe!(
    lazytable_bound_2,
    "LazyTable-bound-2",
    r###"LazyTable[i,{i,(AppendTo[log,1];1),(AppendTo[log,2];Input["p: "]),(AppendTo[log,3];1)}]"###,
    &["3"],
    None
);
probe!(
    lazytable_bound_3,
    "LazyTable-bound-3",
    r###"LazyTable[i,{i,(AppendTo[log,1];1),(AppendTo[log,2];3),(AppendTo[log,3];Input["p: "])}]"###,
    &["1"],
    None
);
probe!(
    lazytable_bounds_twice,
    "LazyTable-bounds-twice",
    r###"LazyTable[(AppendTo[log,99];i),{i,(AppendTo[log,1];1),(AppendTo[log,2];Input["p: "]),(AppendTo[log,3];Input["p: "])}]"###,
    &["3", "1"],
    None
);
probe!(
    lazytable_invalid_step,
    "LazyTable-invalid-step",
    r###"LazyTable[i,{i,(AppendTo[log,1];1),(AppendTo[log,2];Input["p: "]),(AppendTo[log,3];0)}]"###,
    &["3"],
    None
);
probe!(
    lazytable_body_and_bounds,
    "LazyTable-body-and-bounds",
    r###"Normal[LazyTable[(AppendTo[log,i];If[i==2,InputString["p: "],i]),{i,1,Input["p: "]}]]"###,
    &["3", "two"],
    None
);

#[test]
fn lazytable_error_in_step_after_upper() {
    let source = r###"Message[probe::tag];Print["before"];LazyTable[(AppendTo[log,99];i),{i,(AppendTo[log,1];1),(AppendTo[log,2];Input["p: "]),(AppendTo[log,3];Message[probe::tag];Print["after"];Throw[9])}]"###;
    compare("LazyTable-error-in-step-after-upper", source, &["3"], None);
    let queued = run(source, &["3"], true, None);
    assert_eq!(queued.1, "List[1, 2, 3]");
    assert_ne!(queued.0.status, "OK");
    assert_eq!(queued.0.messages.len(), 3);
    assert_eq!(queued.0.prints, ["before", "after"]);
}

#[test]
fn lazytable_nonnumeric_lower_runs_later_bounds() {
    let source = r###"LazyTable[(AppendTo[log,99];i),{i,(AppendTo[log,1];Input["p: "]),(AppendTo[log,2];Print["upper"];3),(AppendTo[log,3];Print["step"];1)}]"###;
    compare("LazyTable-nonnumeric-lower", source, &["x"], None);
    let queued = run(source, &["x"], true, None);
    assert_eq!(queued.1, "List[1, 2, 3]");
    assert_eq!(queued.0.status, "OK");
    assert_eq!(queued.0.messages.len(), 1);
    assert_eq!(queued.0.messages[0].symbol, "LazyTable");
    assert_eq!(queued.0.messages[0].tag, "argform");
    assert_eq!(queued.0.prints, ["upper", "step"]);
}

const BOUNDS_CANCEL_SOURCE: &str = r###"LazyTable[(AppendTo[log,99];i),{i,(AppendTo[log,1];Print["boundary"];Input["lower: "]),(AppendTo[log,2];Print["boundary"];Input["upper: "]),(AppendTo[log,3];1)}]"###;

// The reference queues the completed prefix, then cancels at the same input
// suspension. Cancelling from Print would exercise a different ordinary-path
// checkpoint: bound evaluations do not promise a poll after every host print.
fn cancel_bounds_run(stop: usize, queued_prefix: bool) -> (View, String, Vec<String>) {
    let mut s = session(None);
    let cancel = s.request_cancel_handle();
    if queued_prefix && stop == 1 {
        s.queue_input_response("1");
    }
    let mut prompts = Vec::new();
    let mut out = s.eval_request(BOUNDS_CANCEL_SOURCE);
    while let Err(f) = &out {
        let SessionRequestError::InputRequired { request } = &f.error else {
            break;
        };
        prompts.push(request.prompt.clone());
        if queued_prefix || prompts.len() - 1 == stop {
            cancel.cancel();
            out = s.resume_eval_request_with_input("unused");
            break;
        }
        out = s.resume_eval_request_with_input("1");
    }
    finish(&mut s, out, prompts)
}

fn cancel_bounds_compare(stop: usize) {
    let resumed = cancel_bounds_run(stop, false);
    let queued = cancel_bounds_run(stop, true);
    record(
        &format!("LazyTable-cancel-at-{}", stop + 1),
        &resumed,
        &queued,
        stop + 1,
    );
    assert_eq!(queued.0.termination, "Some(Cancelled)");
    assert_eq!(queued.1, if stop == 0 { "List[1]" } else { "List[1, 2]" });
    assert_eq!(resumed.2, ["lower: ", "upper: "][..=stop]);
    assert_eq!(queued.2, [["lower: ", "upper: "][stop]]);
    assert_eq!(resumed.0, queued.0, "cancelled outcome, messages, prints");
    assert_eq!(resumed.1, queued.1, "later bounds and held body never run");
}

#[test]
fn lazytable_cancel_first_suspension() {
    cancel_bounds_compare(0);
}

#[test]
fn lazytable_cancel_second_suspension() {
    cancel_bounds_compare(1);
}
