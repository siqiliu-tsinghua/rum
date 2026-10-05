//! rum's queued versus resumed same-request schema validation contract.
//! Contracts: docs/guide/evaluation.md (input and schema validation),
//! docs/guide/structured-data.md, iteration/cancel contract.
use rum_core::Expr;
use rum_runtime::{
    RuntimeMessage, Session, SessionEvalResponse, SessionRequestError, SessionRequestFailure,
};

fn session(limit: Option<usize>) -> Session {
    let mut s = Session::new();
    rum_stdlib::install_core(&mut s);
    s.eval_request("log={};probe::tag=\"schema message\"")
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
        assert_eq!(
            s.pending_input_request(),
            Some(request),
            "exact pending owner"
        );
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
fn compare(label: &str, source: &str, inputs: &[&str], expected_prompts: &[&str]) {
    let limit = None;
    let resumed = run(source, inputs, false, limit);
    let queued = run(source, inputs, true, limit);
    record(label, &resumed, &queued, inputs.len());
    assert_eq!(resumed.2, expected_prompts, "must reach every suspension");
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

#[test]
fn oracle_conformschema_leaf() {
    compare(
        "ConformSchema-leaf",
        r###"ConformSchema[{1,2,3},{_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]}]"###,
        &["two"],
        &["p: "],
    );
}

#[test]
fn oracle_conformschema_association() {
    compare(
        "ConformSchema-association",
        r###"ConformSchema[<|"a"->1,"b"->2,"c"->3|>,<|"a"->_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])],"b"->_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])],"c"->_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]|>]"###,
        &["two"],
        &["p: "],
    );
}

#[test]
fn oracle_conformschema_default() {
    compare(
        "ConformSchema-default",
        r###"ConformSchema[<||>,<|"a"->Optional[_Integer,(AppendTo[log,1];1)],"b"->Optional[_Integer,(AppendTo[log,2];Input["p: "])],"c"->Optional[_Integer,(AppendTo[log,3];3)]|>]"###,
        &["2"],
        &["p: "],
    );
}

#[test]
fn oracle_schemaerrors_leaf() {
    compare(
        "SchemaErrors-leaf",
        r###"SchemaErrors[{1,2,3},{_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]}]"###,
        &["two"],
        &["p: "],
    );
}

#[test]
fn oracle_schemaerrors_association() {
    compare(
        "SchemaErrors-association",
        r###"SchemaErrors[<|"a"->1,"b"->2,"c"->3|>,<|"a"->_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])],"b"->_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])],"c"->_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]|>]"###,
        &["two"],
        &["p: "],
    );
}

#[test]
fn oracle_schemaerrors_default() {
    compare(
        "SchemaErrors-default",
        r###"SchemaErrors[<||>,<|"a"->Optional[_Integer,(AppendTo[log,1];1)],"b"->Optional[_Integer,(AppendTo[log,2];Input["p: "])],"c"->Optional[_Integer,(AppendTo[log,3];3)]|>]"###,
        &["2"],
        &["p: "],
    );
}

#[test]
fn oracle_schema_twice() {
    compare(
        "Schema-twice",
        r###"ConformSchema[{1,2,3,4,5},{_?Function[x,(AppendTo[log,x];If[Or[SameQ[x,2],SameQ[x,4]],InputString["p: "],Null];True)]}]"###,
        &["two", "four"],
        &["p: ", "p: "],
    );
}

#[test]
fn oracle_conformschema_default_predicate() {
    compare(
        "ConformSchema-default-predicate",
        r###"ConformSchema[<||>,<|"a"->Optional[_Integer,1],"b"->Optional[_?Function[x,(AppendTo[log,2];InputString["p: "];True)],2],"c"->Optional[_Integer,3]|>]"###,
        &["two"],
        &["p: "],
    );
}

#[test]
fn oracle_schema_defaults_twice() {
    compare(
        "Schema-defaults-twice",
        r###"ConformSchema[<||>,<|"a"->Optional[_Integer,(AppendTo[log,1];1)],"b"->Optional[_Integer,(AppendTo[log,2];Input["p: "])],"c"->Optional[_Integer,(AppendTo[log,3];3)],"d"->Optional[_Integer,(AppendTo[log,4];Input["p: "])],"e"->Optional[_Integer,(AppendTo[log,5];5)]|>]"###,
        &["2", "4"],
        &["p: ", "p: "],
    );
}

#[test]
fn oracle_positions_distinct() {
    compare(
        "positions_distinct",
        r###"ConformSchema[{1,2,3},{_?Function[x,(AppendTo[log,x];If[x==1,InputString["first: "]];If[x==3,InputString["third: "]];True)]}]"###,
        &["a", "c"],
        &["first: ", "third: "],
    );
}

#[test]
fn oracle_error_after_resume() {
    compare(
        "error_after_resume",
        r###"ConformSchema[{1,2,3},{_?Function[x,(AppendTo[log,x];If[x==1,InputString["p: "]];x<3)]}]"###,
        &["a"],
        &["p: "],
    );
}

#[test]
fn oracle_error_prefix() {
    compare(
        "error_prefix",
        r###"SchemaErrors[<|"bad"->"x","ask"->2,"late"->"y"|>,<|"bad"->_Integer,"ask"->_?Function[x,(AppendTo[log,x];Print["predicate"];Message[probe::tag];InputString["p: "];True)],"late"->_Integer|>]"###,
        &["a"],
        &["p: "],
    );
}

#[test]
fn oracle_eval_error_after_resume() {
    compare(
        "eval_error_after_resume",
        r###"ConformSchema[<||>,<|"a"->Optional[_Integer,(AppendTo[log,1];Input["p: "])],"b"->Optional[_Integer,(AppendTo[log,2];Throw["schema boom"])]|>]"###,
        &["1"],
        &["p: "],
    );
}

#[test]
fn oracle_malformed_after_resume() {
    compare(
        "malformed_after_resume",
        r###"SchemaErrors[<|"a"->1,"b"->{}|>,<|"a"->_?Function[x,(AppendTo[log,x];InputString["p: "];True)],"b"->{_Integer,_String}|>]"###,
        &["a"],
        &["p: "],
    );
}

#[test]
fn oracle_strict_extra() {
    compare(
        "strict_extra",
        r###"SchemaErrors[<|"a"->1,"extra"->9|>,<|"a"->_?Function[x,(AppendTo[log,x];InputString["p: "];True)]|>]"###,
        &["a"],
        &["p: "],
    );
}

#[test]
fn oracle_allow_extra() {
    compare(
        "allow_extra",
        r###"ConformSchema[<|"extra"->9,"a"->1|>,<|"a"->_?Function[x,(AppendTo[log,x];InputString["p: "];True)],"default"->Optional[_Integer,(AppendTo[log,2];2)]|>,"AllowExtraKeys"->True]"###,
        &["a"],
        &["p: "],
    );
}

#[test]
fn oracle_nested_assoc_list_assoc() {
    compare(
        "nested_assoc_list_assoc",
        r###"ConformSchema[<|"rows"->{<|"a"->1|>,<|"a"->2|>}|>,<|"rows"->{<|"a"->_?Function[x,(AppendTo[log,x];If[x==1,InputString["p: "]];True)],"d"->Optional[_Integer,(AppendTo[log,9];9)]|>}|>]"###,
        &["a"],
        &["p: "],
    );
}

#[test]
fn oracle_default_then_predicate() {
    compare(
        "default_then_predicate",
        r###"ConformSchema[<||>,<|"a"->Optional[_?Function[x,(AppendTo[log,2];InputString["predicate: "];True)],(AppendTo[log,1];Input["default: "])],"b"->Optional[_Integer,(AppendTo[log,3];3)]|>]"###,
        &["7", "yes"],
        &["default: ", "predicate: "],
    );
}

#[test]
fn oracle_predicate_twice() {
    compare(
        "predicate_twice",
        r###"ConformSchema[{1,2},{_?Function[x,(AppendTo[log,x];If[x==1,(InputString["one: "];InputString["two: "])];True)]}]"###,
        &["a", "b"],
        &["one: ", "two: "],
    );
}

#[test]
fn oracle_absent_optional_control() {
    compare(
        "absent_optional_control",
        r###"ConformSchema[<|"a"->1|>,<|"a"->_Integer,"b"->Optional[_Integer]|>] "###,
        &[],
        &[],
    );
}

#[test]
fn oracle_present_default_control() {
    compare(
        "present_default_control",
        r###"ConformSchema[<|"a"->1|>,<|"a"->Optional[_Integer,(AppendTo[log,99];Input["forbidden: "])]|>] "###,
        &[],
        &[],
    );
}

#[test]
fn oracle_capped_errors_continue() {
    compare(
        "capped_errors_continue",
        r###"SchemaErrors[Range[67],{_?Function[x,(AppendTo[log,x];If[x==2,InputString["cap: "]];False)]}]"###,
        &["go"],
        &["cap: "],
    );
}

fn cancel_case(default: bool) {
    let source = if default {
        r#"ConformSchema[<||>,<|"a"->Optional[_Integer,(AppendTo[log,1];Print["before"];Input["default: "];AppendTo[log,2];2)],"b"->Optional[_Integer,(AppendTo[log,3];3)]|>]"#
    } else {
        r#"ConformSchema[{1,2},{_?Function[x,(AppendTo[log,x];Print["before"];InputString["predicate: "];AppendTo[log,9];True)]}]"#
    };
    let mut s = session(None);
    let cancel = s.request_cancel_handle();
    let out = s.eval_request(source);
    let Err(f) = out else {
        panic!("must suspend");
    };
    let SessionRequestError::InputRequired { request } = f.error else {
        panic!("must request input");
    };
    assert_eq!(
        request.prompt,
        if default { "default: " } else { "predicate: " }
    );
    assert_eq!(f.prints, vec!["before"]);
    assert!(s.pending_input_request().is_some());
    cancel.cancel();
    let out = s.resume_eval_request_with_input("unused");
    let result = finish(&mut s, out, vec![request.prompt]);
    println!("CANCEL default={default} {result:?}");
    assert_eq!(result.0.termination, "Some(Cancelled)");
    assert_eq!(
        result.0.value.unwrap().to_string(),
        "TerminatedEvaluation[\"Cancelled\"]"
    );
    assert_eq!(result.1, "List[1]");
}
#[test]
fn cancel_predicate() {
    cancel_case(false);
}
#[test]
fn cancel_default() {
    cancel_case(true);
}

#[path = "schema_continuations/depth.rs"]
mod depth;
