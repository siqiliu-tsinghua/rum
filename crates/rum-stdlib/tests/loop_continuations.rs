//! Loop callbacks obey guide/evaluation.md's same-request input contract.
//! Values follow the generated Scan, OptionValue, Differences, Dot and
//! ReplaceRepeated reference entries; queued rum is the oracle.

use rum_core::Expr;
use rum_runtime::{
    RuntimeMessage, Session, SessionEvalResponse, SessionRequestError, SessionRequestFailure,
};

fn session() -> Session {
    let mut s = Session::new();
    rum_stdlib::install_core(&mut s);
    s.eval_request("log={}; probe::tag=\"loop message\"")
        .unwrap();
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

fn run(source: &str, inputs: &[&str], queued: bool) -> (View, String, Vec<String>) {
    let mut s = session();
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
        assert!(!queued, "queued control must finish");
        prompts.push(request.prompt.clone());
        let input = inputs
            .get(prompts.len() - 1)
            .expect("unexpected extra suspension");
        out = s.resume_eval_request_with_input(*input);
    }
    let final_view = view(out);
    assert!(s.pending_input_request().is_none());
    let log = s.eval_request("log").unwrap().value.to_string();
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
    (final_view, log, prompts)
}

fn compare(label: &str, source: &str, inputs: &[&str]) -> View {
    let resumed = run(source, inputs, false);
    let queued = run(source, inputs, true);
    println!("CASE\t{label}\tresumed={resumed:?}\tqueued={queued:?}");
    assert_eq!(
        resumed.2,
        vec!["p: "; inputs.len()],
        "must reach every suspension, in order"
    );
    assert!(queued.2.is_empty());
    assert_eq!(
        resumed.0, queued.0,
        "value, error, termination, messages and prints"
    );
    assert_eq!(
        resumed.1, queued.1,
        "completed effects must execute exactly once"
    );
    queued.0
}

fn witness(label: &str, source: &str, inputs: &[&str], value: &str, log: &str) {
    let queued = run(source, inputs, true);
    assert_eq!(queued.0.value.as_ref().unwrap().to_string(), value);
    assert_eq!(queued.1, log);
    compare(label, source, inputs);
}

fn cancel_at_both_inputs(label: &str, source: &str, first_input: &str, prefixes: [&str; 2]) {
    for stop in 0..2 {
        let mut s = session();
        let cancel = s.request_cancel_handle();
        let mut out = s.eval_request(source);
        for index in 0..=stop {
            assert!(
                matches!(&out, Err(f) if matches!(&f.error, SessionRequestError::InputRequired { request } if request.prompt == "p: ")),
                "must suspend at input {index}"
            );
            if index < stop {
                out = s.resume_eval_request_with_input(first_input);
            }
        }
        cancel.cancel();
        let out = s.resume_eval_request_with_input("unused").unwrap();
        assert_eq!(format!("{:?}", out.termination), "Some(Cancelled)");
        assert!(s.pending_input_request().is_none());
        let log = s.eval_request("log").unwrap().value.to_string();
        println!(
            "CANCEL\t{label}\tstop={stop}\tlog={log}\ttermination={:?}",
            out.termination
        );
        assert_eq!(
            log, prefixes[stop],
            "no remaining callback may run on cancellation"
        );
        assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
    }
}

const SCAN: &str =
    "Scan[Function[x, (AppendTo[log,x]; If[SameQ[x,2], InputString[\"p: \"], x])],{1,2,3}]";
#[test]
fn scan() {
    witness("Scan", SCAN, &["two"], "Null", "List[1, 2, 3]");
}

const SCAN_TWICE: &str = "Scan[Function[x,(AppendTo[log,x];If[Or[SameQ[x,2],SameQ[x,4]],InputString[\"p: \"],x])],{1,2,3,4,5}]";
#[test]
fn scan_twice() {
    witness(
        "Scan-twice",
        SCAN_TWICE,
        &["two", "four"],
        "Null",
        "List[1, 2, 3, 4, 5]",
    );
}

const OPTIONVALUE_MANY: &str = "Options[f]={a:>(AppendTo[log,1];1),b:>(AppendTo[log,2];InputString[\"p: \"]),c:>(AppendTo[log,3];3)};OptionValue[f,{a,b,c}]";
#[test]
fn optionvalue_many() {
    witness(
        "OptionValue-many",
        OPTIONVALUE_MANY,
        &["two"],
        "List[1, \"two\", 3]",
        "List[1, 2, 3]",
    );
}

const OPTIONVALUE_TWICE: &str = "Options[f]={a:>(AppendTo[log,1];1),b:>(AppendTo[log,2];InputString[\"p: \"]),c:>(AppendTo[log,3];3),d:>(AppendTo[log,4];InputString[\"p: \"]),e:>(AppendTo[log,5];5)};OptionValue[f,{a,b,c,d,e}]";
#[test]
fn optionvalue_twice() {
    witness(
        "OptionValue-twice",
        OPTIONVALUE_TWICE,
        &["two", "four"],
        "List[1, \"two\", 3, \"four\", 5]",
        "List[1, 2, 3, 4, 5]",
    );
}

const REPLACEREPEATED_PREDICATE: &str = "ReplaceRepeated[{1,2,3},z_Integer /;(AppendTo[log,z];If[SameQ[z,2],InputString[\"p: \"],Null];True):>h[z],MaxIterations->1]";
#[test]
fn replacerepeated_predicate() {
    witness(
        "ReplaceRepeated-predicate",
        REPLACEREPEATED_PREDICATE,
        &["two"],
        "TerminatedEvaluation[\"IterationLimit\"]",
        "List[1, 2, 3]",
    );
}

const REPLACEREPEATED_TWICE: &str = "ReplaceRepeated[{1,2,3,4,5},z_Integer /;(AppendTo[log,z];If[Or[SameQ[z,2],SameQ[z,4]],InputString[\"p: \"],Null];True):>h[z],MaxIterations->1]";
#[test]
fn replacerepeated_twice() {
    witness(
        "ReplaceRepeated-twice",
        REPLACEREPEATED_TWICE,
        &["two", "four"],
        "TerminatedEvaluation[\"IterationLimit\"]",
        "List[1, 2, 3, 4, 5]",
    );
}

const DOT_CHAIN: &str = "a /: Times[a,x_] := (AppendTo[log,x];If[SameQ[Length[log],2],InputString[\"p: \"],Null];x);Dot[{{a,a},{a,a}},{{1,2},{3,4}},{{3,0},{0,3}}]";
#[test]
fn dot_chain() {
    witness(
        "Dot-chain",
        DOT_CHAIN,
        &["two"],
        "List[List[12, 18], List[12, 18]]",
        "List[1, 3, 2, 4, 1, 3, 2, 4]",
    );
}

const DOT_TWICE: &str = "a /: Times[a,x_] := (AppendTo[log,x];If[Or[SameQ[Length[log],2],SameQ[Length[log],4]],InputString[\"p: \"],Null];x);Dot[{{a,a},{a,a}},{{1,2},{3,4}},{{3,0},{0,3}}]";
#[test]
fn dot_twice() {
    witness(
        "Dot-twice",
        DOT_TWICE,
        &["two", "four"],
        "List[List[12, 18], List[12, 18]]",
        "List[1, 3, 2, 4, 1, 3, 2, 4]",
    );
}

const DIFFERENCES: &str =
    "a /: Times[-1,a] := (AppendTo[log,2];Input[\"p: \"]);Differences[{1,a,3,4}]";
#[test]
fn differences() {
    witness(
        "Differences",
        DIFFERENCES,
        &["-2"],
        "List[Plus[-1, a], 1, 1]",
        "List[2]",
    );
}

const DIFFERENCES_TWICE: &str = "a /:Times[-1,a]:=(AppendTo[log,2];Input[\"p: \"]);b /:Times[-1,b]:=(AppendTo[log,4];Input[\"p: \"]);Differences[{1,a,3,b,5}]";
#[test]
fn differences_twice() {
    witness(
        "Differences-twice",
        DIFFERENCES_TWICE,
        &["-2", "-4"],
        "List[Plus[-1, a], 1, Plus[-3, b], 1]",
        "List[2, 4]",
    );
}

const DIFFERENCES_ORDER_TWO: &str =
    "a /:Times[-1,a]:=(AppendTo[log,2];Input[\"p: \"]);Differences[{1,a,3,4},2]";
#[test]
fn differences_order_two() {
    witness(
        "Differences-order-two",
        DIFFERENCES_ORDER_TWO,
        &["-2"],
        "List[Plus[1, Times[-1, Plus[-1, a]]], 0]",
        "List[2]",
    );
}

#[test]
fn scan_error_after_resume() {
    error_witness(
        "scan-error",
        "Message[probe::tag];Print[\"before\"];Scan[Function[x,(AppendTo[log,x];If[x==2,InputString[\"p: \"]];If[x==3,Message[probe::tag];Print[\"after\"];Throw[9]];x)],{1,2,3,4}]",
        &["two"],
    );
}

#[test]
fn option_value_error_after_resume() {
    error_witness(
        "option_value-error",
        "Message[probe::tag];Print[\"before\"];Options[f]={a:>(AppendTo[log,1];1),b:>(AppendTo[log,2];InputString[\"p: \"]),c:>(AppendTo[log,3];Message[probe::tag];Print[\"after\"];Throw[9]),d:>(AppendTo[log,4];4)};OptionValue[f,{a,b,c,d}]",
        &["two"],
    );
}

#[test]
fn replace_repeated_error_after_resume() {
    error_witness(
        "replace_repeated-error",
        "Message[probe::tag];Print[\"before\"];ReplaceRepeated[{1,2,3},{z_Integer /;(AppendTo[log,z];If[z==2,InputString[\"p: \"]];True):>h[z],h[z_Integer]:>(AppendTo[log,99];Message[probe::tag];Print[\"after\"];Throw[9])},MaxIterations->4]",
        &["two"],
    );
}

#[test]
fn dot_error_after_resume() {
    error_witness(
        "dot-error",
        "Message[probe::tag];Print[\"before\"];a /: Times[a,x_] := (AppendTo[log,x];If[Length[log]==2,InputString[\"p: \"]];x);b /: Times[b,x_] := (AppendTo[log,99];Message[probe::tag];Print[\"after\"];Throw[9]);Dot[{{a,a},{a,a}},{{1,2},{3,4}},{{b,b},{b,b}}]",
        &["two"],
    );
}

#[test]
fn differences_error_after_resume() {
    error_witness(
        "differences-error",
        "Message[probe::tag];Print[\"before\"];a /:Times[-1,a]:=(AppendTo[log,2];Input[\"p: \"]);b /:Times[-1,b]:=(AppendTo[log,4];Message[probe::tag];Print[\"after\"];Throw[9]);Differences[{1,a,3,b,5}]",
        &["-2"],
    );
}

#[test]
fn scan_cancel() {
    cancel_at_both_inputs(
        "scan",
        SCAN_TWICE,
        "two",
        ["List[1, 2]", "List[1, 2, 3, 4]"],
    );
}

#[test]
fn option_value_cancel() {
    cancel_at_both_inputs(
        "option_value",
        OPTIONVALUE_TWICE,
        "two",
        ["List[1, 2]", "List[1, 2, 3, 4]"],
    );
}

#[test]
fn replace_repeated_cancel() {
    cancel_at_both_inputs(
        "replace_repeated",
        REPLACEREPEATED_TWICE,
        "two",
        ["List[1, 2]", "List[1, 2, 3, 4]"],
    );
}

#[test]
fn dot_cancel() {
    cancel_at_both_inputs("dot", DOT_TWICE, "two", ["List[1, 3]", "List[1, 3, 2, 4]"]);
}

#[test]
fn differences_cancel() {
    cancel_at_both_inputs(
        "differences",
        DIFFERENCES_TWICE,
        "-2",
        ["List[2]", "List[2, 4]"],
    );
}

fn error_witness(label: &str, source: &str, inputs: &[&str]) {
    let out = run(source, inputs, true).0;
    assert!(out.status.contains("uncaught throw: 9"), "{:?}", out.status);
    assert_eq!(
        out.messages
            .iter()
            .map(|m| (m.symbol.as_str(), m.tag.as_str()))
            .collect::<Vec<_>>(),
        [("probe", "tag"), ("probe", "tag"), ("Throw", "nocatch")]
    );
    assert_eq!(out.prints, ["before", "after"]);
    compare(label, source, inputs);
}
