//! rum's queued versus resumed same-request lazy realization contract.
//! Contracts: docs/guide/evaluation.md (input and lazy sequences),
//! generated lazy/Normal/Take/First/Part references, iteration/cancel contract.
use rum_core::Expr;
use rum_runtime::{
    ClosurePrintSink, RuntimeMessage, Session, SessionEvalResponse, SessionRequestError,
    SessionRequestFailure,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

fn session(limit: Option<usize>) -> Session {
    let mut s = Session::new();
    rum_stdlib::install_core(&mut s);
    s.eval_request("log={};probe::tag=\"lazy message\"")
        .unwrap();
    if let Some(limit) = limit {
        s.set_iteration_work_limit(limit);
    }
    s
}
#[derive(PartialEq, Eq)]
struct View {
    status: String,
    value: Option<Expr>,
    termination: String,
    messages: Vec<RuntimeMessage>,
    prints: Vec<String>,
}
impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("status", &self.status)
            .field("value", &self.value.as_ref().map(ToString::to_string))
            .field("termination", &self.termination)
            .field("messages", &self.messages)
            .field("prints", &self.prints)
            .finish()
    }
}
fn view(out: Result<SessionEvalResponse, SessionRequestFailure>) -> View {
    match out {
        Ok(r) => View {
            status: "OK".into(),
            value: Some(r.value),
            termination: format!("{:?}", r.termination),
            messages: r.messages,
            prints: r.prints,
        },
        Err(f) => View {
            status: format!("{:?}", f.error),
            value: None,
            termination: "None".into(),
            messages: f.messages,
            prints: f.prints,
        },
    }
}
fn finish(
    s: &mut Session,
    out: Result<SessionEvalResponse, SessionRequestFailure>,
    prompts: Vec<String>,
) -> (View, String, Vec<String>) {
    let v = view(out);
    assert!(
        s.pending_input_request().is_none(),
        "pending owner leaked: {v:?}"
    );
    let log = s.eval_request("log").unwrap().value.to_string();
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
    (v, log, prompts)
}
fn run(
    source: &str,
    inputs: &[&str],
    queued: bool,
    limit: Option<usize>,
) -> (View, String, Vec<String>) {
    let mut s = session(limit);
    if queued {
        for input in inputs {
            s.queue_input_response(*input);
        }
    }
    let mut out = s.eval_request(source);
    let mut prompts = Vec::new();
    while let Err(f) = &out {
        let SessionRequestError::InputRequired { request } = &f.error else {
            break;
        };
        assert!(!queued, "queued control exhausted answers");
        prompts.push(request.prompt.clone());
        let input = inputs
            .get(prompts.len() - 1)
            .expect("unexpected extra suspension");
        out = s.resume_eval_request_with_input(*input);
    }
    finish(&mut s, out, prompts)
}
fn record(
    label: &str,
    resumed: &(View, String, Vec<String>),
    queued: &(View, String, Vec<String>),
    expected_prompts: usize,
) {
    println!("CASE\t{label}\tresumed={resumed:?}\tqueued={queued:?}");
    println!(
        "ROW\t{label}\t{:?}\t{:?}\t{:?}\t{:?}\t{}\t{expected_prompts}\t{}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}",
        resumed
            .0
            .value
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "<error>".into()),
        resumed.1,
        queued
            .0
            .value
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "<error>".into()),
        queued.1,
        resumed.2.len(),
        resumed.0 == queued.0 && resumed.1 == queued.1,
        resumed.0.status,
        queued.0.status,
        resumed.0.termination,
        queued.0.termination,
        resumed.0.messages,
        queued.0.messages,
        resumed.0.prints,
        queued.0.prints
    );
}
fn compare(label: &str, source: &str, inputs: &[&str], limit: Option<usize>) {
    let resumed = run(source, inputs, false, limit);
    let queued = run(source, inputs, true, limit);
    record(label, &resumed, &queued, inputs.len());
    assert_eq!(
        resumed.2,
        vec!["p: "; inputs.len()],
        "must reach every suspension"
    );
    assert!(queued.2.is_empty());
    assert_eq!(
        resumed.0, queued.0,
        "value, error, termination, messages, prints"
    );
    assert_eq!(
        resumed.1, queued.1,
        "each completed effect runs exactly once"
    );
}
macro_rules! probe {
    ($name:ident, $label:expr, $source:expr, $inputs:expr, $limit:expr) => {
        #[test]
        fn $name() {
            compare($label, $source, $inputs, $limit);
        }
    };
}

probe!(
    lazymap,
    "LazyMap",
    r###"Normal[LazyTake[LazyMap[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString["p: "], x])],LazyRange[1,1]],3]]"###,
    &["two"],
    None
);
probe!(
    lazynestlist,
    "LazyNestList",
    r###"Normal[LazyTake[LazyNestList[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString["p: "], Null]; x+1)],1],4]]"###,
    &["two"],
    None
);
probe!(
    lazyselect,
    "LazySelect",
    r###"Normal[LazyTake[LazySelect[Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])],LazyRange[1,1]],3]]"###,
    &["two"],
    None
);
probe!(
    lazyfoldlist,
    "LazyFoldList",
    r###"Normal[LazyTake[LazyFoldList[Function[{a,x},(AppendTo[log,x];If[SameQ[x,2],InputString["p: "],Null];a+x)],0,LazyRange[1,1]],4]]"###,
    &["two"],
    None
);
probe!(
    lazydrop_inner,
    "LazyDrop-inner",
    r###"Normal[LazyTake[LazyDrop[LazyMap[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString["p: "], x])],LazyRange[1,1]],2],2]]"###,
    &["two"],
    None
);
probe!(
    lazymap_inner,
    "LazyMap-inner",
    r###"Normal[LazyTake[LazyMap[Function[x,{x}],LazyMap[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString["p: "], x])],LazyRange[1,1]]],3]]"###,
    &["two"],
    None
);
probe!(
    lazyselect_inner,
    "LazySelect-inner",
    r###"Normal[LazyTake[LazySelect[Function[x,True],LazyMap[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString["p: "], x])],LazyRange[1,1]]],3]]"###,
    &["two"],
    None
);
probe!(
    lazyfoldlist_inner,
    "LazyFoldList-inner",
    r###"Normal[LazyTake[LazyFoldList[Function[{a,x},Append[a,x]],{},LazyMap[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString["p: "], x])],LazyRange[1,1]]],4]]"###,
    &["two"],
    None
);
probe!(
    lazytake_consumer,
    "LazyTake-consumer",
    r###"Take[LazyMap[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString["p: "], x])],LazyRange[1,1]],3]"###,
    &["two"],
    None
);
probe!(
    lazyfirst_consumer,
    "LazyFirst-consumer",
    r###"First[LazyDrop[LazyMap[Function[x,{x}],LazyMap[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString["p: "], x])],LazyRange[1,1]]],1]]"###,
    &["two"],
    None
);
probe!(
    lazypart_consumer,
    "LazyPart-consumer",
    r###"LazyMap[Function[x,{x}],LazyMap[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString["p: "], x])],LazyRange[1,1]]][[2]]"###,
    &["two"],
    None
);
probe!(
    lazytable_body,
    "LazyTable-body",
    r###"Normal[LazyTable[(AppendTo[log,i];If[SameQ[i,2],InputString["p: "],i]),{i,1,3}]]"###,
    &["two"],
    None
);
probe!(
    lazymap_twice,
    "LazyMap-twice",
    r###"Normal[LazyTake[LazyMap[Function[x,(AppendTo[log,x];If[Or[SameQ[x,2],SameQ[x,4]],InputString["p: "],x])],LazyRange[1,1]],5]]"###,
    &["two", "four"],
    None
);
probe!(
    lazynestlist_twice,
    "LazyNestList-twice",
    r###"Normal[LazyTake[LazyNestList[Function[x,(AppendTo[log,x];If[Or[SameQ[x,2],SameQ[x,4]],InputString["p: "],Null];x+1)],1],5]]"###,
    &["two", "four"],
    None
);
probe!(
    lazyselect_twice,
    "LazySelect-twice",
    r###"Normal[LazyTake[LazySelect[Function[x,(AppendTo[log,x];If[Or[SameQ[x,2],SameQ[x,4]],InputString["p: "],Null];True)],LazyRange[1,1]],5]]"###,
    &["two", "four"],
    None
);
probe!(
    lazyfoldlist_twice,
    "LazyFoldList-twice",
    r###"Normal[LazyTake[LazyFoldList[Function[{a,x},(AppendTo[log,x];If[Or[SameQ[x,2],SameQ[x,4]],InputString["p: "],Null];a+x)],0,LazyRange[1,1]],6]]"###,
    &["two", "four"],
    None
);
probe!(
    lazydrop_twice,
    "LazyDrop-twice",
    r###"First[LazyDrop[LazyMap[Function[x,(AppendTo[log,x];If[Or[SameQ[x,2],SameQ[x,4]],InputString["p: "],x])],LazyRange[1,1]],4]]"###,
    &["two", "four"],
    None
);
probe!(
    lazymap_error_next,
    "LazyMap-error-next",
    r###"Message[probe::tag];Print["before"];Normal[LazyTake[LazyMap[Function[x,(AppendTo[log,x];If[x==2,InputString["p: "]];If[x==3,Message[probe::tag];Print["after"];Throw[9]];x)],LazyRange[1,1]],4]]"###,
    &["two"],
    None
);
probe!(
    lazymap_error_in_callback,
    "LazyMap-error-in-callback",
    r###"Normal[LazyTake[LazyMap[Function[x,(AppendTo[log,x];If[x==2,InputString["p: "];Message[probe::tag];Throw[9]];x)],LazyRange[1,1]],4]]"###,
    &["two"],
    None
);
probe!(
    lazy_infinite_after_resume,
    "Lazy-infinite-after-resume",
    r###"Normal[LazyMap[Function[x,(AppendTo[log,x];If[x==2,InputString["p: "]];x)],LazyRange[1,1]]]"###,
    &["two"],
    Some(4)
);
probe!(
    lazyselect_never_after_resume,
    "LazySelect-never-after-resume",
    r###"First[LazySelect[Function[x,(AppendTo[log,x];If[x==2,InputString["p: "]];False)],LazyRange[1,1]]]"###,
    &["two"],
    Some(4)
);
probe!(
    lazy_source_budget_exact,
    "Lazy-source-budget-exact",
    r###"Take[LazyDrop[LazyMap[Function[x,(AppendTo[log,x];If[x==2,InputString["p: "]];x)],LazyRange[1,1]],2],2]"###,
    &["two"],
    Some(4)
);
probe!(
    lazyfirst_empty_default,
    "LazyFirst-empty-default",
    r###"First[LazySelect[Function[x,(AppendTo[log,x];If[x==2,InputString["p: "]];False)],LazyTake[LazyRange[1,1],3]],99]"###,
    &["two"],
    None
);
probe!(
    lazyfirst_empty_message,
    "LazyFirst-empty-message",
    r###"First[LazySelect[Function[x,(AppendTo[log,x];If[x==2,InputString["p: "]];False)],LazyTake[LazyRange[1,1],3]]]"###,
    &["two"],
    None
);
probe!(
    lazynestlist_lookahead_first,
    "LazyNestList-lookahead-first",
    r###"First[LazyNestList[Function[x,(AppendTo[log,x];InputString["p: "];x+1)],1]]"###,
    &["two"],
    None
);
probe!(
    lazy_adapter_unwind_twice,
    "Lazy-adapter-unwind-twice",
    r###"Take[LazyMap[Function[x,(AppendTo[log,{outer,x}];If[x==2,InputString["p: "]];{x})],LazyMap[Function[x,(AppendTo[log,{inner,x}];If[x==2,InputString["p: "]];x)],LazyRange[1,1]]],3]"###,
    &["inner", "outer"],
    None
);
probe!(
    lazyselect_reject_resumed_candidate,
    "LazySelect-reject-resumed-candidate",
    r###"Take[LazySelect[Function[x,(AppendTo[log,x];If[x==2,Input["p: "],True])],LazyRange[1,1]],3]"###,
    &["False"],
    None
);
probe!(
    lazy_rerederive,
    "Lazy-rerederive",
    r###"saved=LazyTake[LazyMap[Function[x,(AppendTo[log,x];If[x==2,InputString["p: "],x])],LazyRange[1,1]],3];{Normal[saved],Normal[saved]}"###,
    &["two", "two"],
    None
);
probe!(
    lazypermutations_cursor,
    "LazyPermutations-cursor",
    r###"Take[LazyMap[Function[x,(AppendTo[log,x];If[Length[log]==2,InputString["p: "]];x)],LazyPermutations[{1,2,3}]],3]"###,
    &["two"],
    None
);
probe!(
    lazysubsets_cursor,
    "LazySubsets-cursor",
    r###"Take[LazyMap[Function[x,(AppendTo[log,x];If[Length[log]==2,InputString["p: "]];x)],LazySubsets[{1,2}]],3]"###,
    &["two"],
    None
);
probe!(
    lazytuples_cursor,
    "LazyTuples-cursor",
    r###"Take[LazyMap[Function[x,(AppendTo[log,x];If[Length[log]==2,InputString["p: "]];x)],LazyTuples[{{1,2},{3,4}}]],3]"###,
    &["two"],
    None
);

const CANCEL_SOURCE: &str = r###"Normal[LazyTake[LazyMap[Function[x,(AppendTo[log,x];If[Or[x==2,x==4],Print["boundary"];InputString["p: "]];x)],LazyRange[1,1]],5]]"###;
fn cancel_run(stop: usize, queued: bool) -> (View, String, Vec<String>) {
    let mut s = session(None);
    let cancel = s.request_cancel_handle();
    let mut prompts = Vec::new();
    // Use the same host print sink on both paths: a sink consumes Print output,
    // so comparing a sink's queued response with buffered resumed output is invalid.
    let prints = Arc::new(Mutex::new(Vec::<String>::new()));
    let capture = prints.clone();
    let hits = Arc::new(AtomicUsize::new(0));
    let on_print = cancel.clone();
    s.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |line| {
            capture.lock().unwrap().push(line.to_owned());
            if queued && line == "boundary" && hits.fetch_add(1, Ordering::SeqCst) == stop {
                on_print.cancel();
            }
        },
    )));
    if queued {
        s.queue_input_response("two");
        s.queue_input_response("four");
    }
    let mut out = s.eval_request(CANCEL_SOURCE);
    if !queued {
        while let Err(f) = &out {
            let SessionRequestError::InputRequired { request } = &f.error else {
                break;
            };
            prompts.push(request.prompt.clone());
            if prompts.len() - 1 == stop {
                cancel.cancel();
                out = s.resume_eval_request_with_input("unused");
                break;
            }
            out = s.resume_eval_request_with_input("two");
        }
    }
    s.clear_print_sink();
    let mut result = finish(&mut s, out, prompts);
    assert!(
        result.0.prints.is_empty(),
        "both paths deliver prints through the sink"
    );
    result.0.prints = prints.lock().unwrap().clone();
    result
}
fn cancel_compare(stop: usize) {
    let resumed = cancel_run(stop, false);
    let queued = cancel_run(stop, true);
    record(
        &format!("Lazy-cancel-at-{}", stop + 1),
        &resumed,
        &queued,
        stop + 1,
    );
    assert_eq!(queued.0.termination, "Some(Cancelled)");
    assert_eq!(
        queued.1,
        if stop == 0 {
            "List[1, 2]"
        } else {
            "List[1, 2, 3, 4]"
        }
    );
    assert_eq!(
        resumed.2,
        vec!["p: "; stop + 1],
        "must reach cancellation boundary"
    );
    assert_eq!(resumed.0, queued.0, "cancelled outcome, messages, prints");
    assert_eq!(resumed.1, queued.1, "cancellation runs no further pull");
}
#[test]
fn cancel_first_suspension() {
    cancel_compare(0);
}
#[test]
fn cancel_second_suspension() {
    cancel_compare(1);
}

#[path = "lazy_pull_continuations/depth.rs"]
mod depth;
#[path = "lazy_pull_continuations/structural.rs"]
mod structural;

#[path = "lazy_pull_continuations/bounds.rs"]
mod bounds;
