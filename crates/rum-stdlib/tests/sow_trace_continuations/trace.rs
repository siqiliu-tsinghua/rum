use super::support::{compare, record, session};

#[test]
fn case_trace_pattern() {
    compare(
        r###"Trace-pattern"###,
        r###"Trace[1+2, _?Function[x,(AppendTo[log,x];If[SameQ[Length[log],2],InputString["p: "],Null];True)]]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn case_tracedetail_nested_pattern() {
    compare(
        r###"TraceDetail-nested-pattern"###,
        r###"TraceDetail[f[1+2,3+4],_?Function[x,(AppendTo[log,x];If[SameQ[Length[log],2],InputString["p: "],Null];True)]]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn case_traceprint_nested_pattern() {
    compare(
        r###"TracePrint-nested-pattern"###,
        r###"TracePrint[f[1+2,3+4],_?Function[x,(AppendTo[log,x];If[SameQ[Length[log],2],InputString["p: "],Null];True)]]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn case_trace_filter_levels() {
    compare(
        r###"Trace-filter-levels"###,
        r###"Trace[f[1+2,3+4],_?Function[x,(AppendTo[log,x];If[SameQ[Length[log],2],InputString["p: "],Null];True)],Infinity]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn case_trace_filter_heads() {
    compare(
        r###"Trace-filter-heads"###,
        r###"Trace[f[1+2,3+4],_?Function[x,(AppendTo[log,x];If[SameQ[Length[log],2],InputString["p: "],Null];True)],Heads->True]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn case_tracedetail_filter_levels() {
    compare(
        r###"TraceDetail-filter-levels"###,
        r###"TraceDetail[f[1+2,3+4],_?Function[x,(AppendTo[log,x];If[SameQ[Length[log],2],InputString["p: "],Null];True)],Infinity]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn case_tracedetail_filter_heads() {
    compare(
        r###"TraceDetail-filter-heads"###,
        r###"TraceDetail[f[1+2,3+4],_?Function[x,(AppendTo[log,x];If[SameQ[Length[log],2],InputString["p: "],Null];True)],Heads->True]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn case_trace_twice() {
    compare(
        r###"Trace-twice"###,
        r###"Trace[f[1+2,3+4],_?Function[x,(AppendTo[log,x];If[Or[SameQ[Length[log],2],SameQ[Length[log],4]],InputString["p: "],Null];True)]]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"p: "###],
    );
}

#[test]
fn case_tracedetail_twice() {
    compare(
        r###"TraceDetail-twice"###,
        r###"TraceDetail[f[1+2,3+4,5+6],_?Function[x,(AppendTo[log,x];If[Or[SameQ[Length[log],2],SameQ[Length[log],4]],InputString["p: "],Null];False)]]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"p: "###],
    );
}

#[test]
fn case_trace_pattern_alternatives() {
    compare(
        r###"Trace-pattern-alternatives"###,
        r###"Trace[f[1+2,3+4],{_?Function[t,(AppendTo[log,{1,t}];If[SameQ[Length[log],1],InputString["decline: "]];False)],_?Function[t,(AppendTo[log,{2,t}];If[SameQ[Length[log],2],InputString["accept: "]];True)]}]"###,
        &[r###"a"###, r###"b"###],
        &[r###"decline: "###, r###"accept: "###],
    );
}

#[test]
fn case_trace_effects_twice() {
    compare(
        r###"Trace-effects-twice"###,
        r###"Trace[f[1+2,3+4],_?Function[t,(AppendTo[log,t];If[SameQ[Length[log],2],Print["before"];Message[probe::tag];InputString["first: "]];If[SameQ[Length[log],4],Print["after"];Message[probe::tag];InputString["second: "]];True)]]"###,
        &[r###"a"###, r###"b"###],
        &[r###"first: "###, r###"second: "###],
    );
}

#[test]
fn case_trace_ordinary() {
    compare(
        r###"Trace-ordinary"###,
        r###"Trace[f[1+2,3+4]]"###,
        &[],
        &[],
    );
}

#[test]
fn case_trace_ordinary_filter() {
    compare(
        r###"Trace-ordinary-filter"###,
        r###"Trace[f[1+2,3+4],_?Function[t,(AppendTo[log,t];True)],Infinity,Heads->True]"###,
        &[],
        &[],
    );
}

#[test]
fn case_tracedetail_pattern_alternatives() {
    compare(
        r###"TraceDetail-pattern-alternatives"###,
        r###"TraceDetail[f[1+2,3+4],{_?Function[t,(AppendTo[log,{1,t}];If[SameQ[Length[log],1],InputString["decline: "]];False)],_?Function[t,(AppendTo[log,{2,t}];If[SameQ[Length[log],2],InputString["accept: "]];True)]}]"###,
        &[r###"a"###, r###"b"###],
        &[r###"decline: "###, r###"accept: "###],
    );
}

#[test]
fn case_tracedetail_effects_twice() {
    compare(
        r###"TraceDetail-effects-twice"###,
        r###"TraceDetail[f[1+2,3+4,5+6],_?Function[t,(AppendTo[log,t];If[SameQ[Length[log],2],Print["before"];Message[probe::tag];InputString["first: "]];If[SameQ[Length[log],4],Print["after"];Message[probe::tag];InputString["second: "]];True)]]"###,
        &[r###"a"###, r###"b"###],
        &[r###"first: "###, r###"second: "###],
    );
}

#[test]
fn case_tracedetail_ordinary() {
    compare(
        r###"TraceDetail-ordinary"###,
        r###"TraceDetail[f[1+2,3+4]]"###,
        &[],
        &[],
    );
}

#[test]
fn case_tracedetail_ordinary_filter() {
    compare(
        r###"TraceDetail-ordinary-filter"###,
        r###"TraceDetail[f[1+2,3+4],_?Function[t,(AppendTo[log,t];True)],Infinity,Heads->True]"###,
        &[],
        &[],
    );
}

#[test]
fn case_traceprint_pattern_alternatives() {
    compare(
        r###"TracePrint-pattern-alternatives"###,
        r###"TracePrint[f[1+2,3+4],{_?Function[t,(AppendTo[log,{1,t}];If[SameQ[Length[log],1],InputString["decline: "]];False)],_?Function[t,(AppendTo[log,{2,t}];If[SameQ[Length[log],2],InputString["accept: "]];True)]}]"###,
        &[r###"a"###, r###"b"###],
        &[r###"decline: "###, r###"accept: "###],
    );
}

#[test]
fn case_traceprint_effects_twice() {
    compare(
        r###"TracePrint-effects-twice"###,
        r###"TracePrint[f[1+2,3+4],_?Function[t,(AppendTo[log,t];If[SameQ[Length[log],2],Print["before"];Message[probe::tag];InputString["first: "]];If[SameQ[Length[log],4],Print["after"];Message[probe::tag];InputString["second: "]];True)]]"###,
        &[r###"a"###, r###"b"###],
        &[r###"first: "###, r###"second: "###],
    );
}

#[test]
fn case_traceprint_ordinary() {
    compare(
        r###"TracePrint-ordinary"###,
        r###"TracePrint[f[1+2,3+4]]"###,
        &[],
        &[],
    );
}

#[test]
fn case_traceprint_ordinary_filter() {
    compare(
        r###"TracePrint-ordinary-filter"###,
        r###"TracePrint[f[1+2,3+4],_?Function[t,(AppendTo[log,t];True)],Infinity,Heads->True]"###,
        &[],
        &[],
    );
}

#[test]
fn case_traceprint_error_after_resume() {
    compare(
        r###"TracePrint-error-after-resume"###,
        r###"TracePrint[f[1+2,3+4],_?Function[t,(AppendTo[log,t];Print["before"];Message[probe::tag];InputString["error: "];Print["after"];Message[probe::tag];Throw["g7 boom"])]]"###,
        &[r###"a"###],
        &[r###"error: "###],
    );
}

#[test]
fn case_trace_definition_nesting() {
    compare(
        r###"Trace-definition-nesting"###,
        r###"g[z_]:=z+1; f[z_]:=g[z]+1; Trace[f[3],_?Function[t,(AppendTo[log,t];If[SameQ[Length[log],2],InputString["tree: "]];True)]]"###,
        &["a"],
        &["tree: "],
    );
}

#[test]
fn case_tracedetail_definition_nesting() {
    compare(
        r###"TraceDetail-definition-nesting"###,
        r###"g[z_]:=z+1; f[z_]:=g[z]+1; TraceDetail[f[3],_?Function[t,(AppendTo[log,t];If[SameQ[Length[log],2],InputString["tree: "]];True)]]"###,
        &["a"],
        &["tree: "],
    );
}

#[test]
fn case_traceprint_definition_nesting() {
    compare(
        r###"TracePrint-definition-nesting"###,
        r###"g[z_]:=z+1; f[z_]:=g[z]+1; TracePrint[f[3],_?Function[t,(AppendTo[log,t];If[SameQ[Length[log],2],InputString["tree: "]];True)]]"###,
        &["a"],
        &["tree: "],
    );
}

#[test]
fn captured_held_expression_exceeds_evaluation_depth() {
    let mut s = session();
    let out = s.eval_request(r###"Block[{$RecursionLimit=32},{Length[Trace[Hold[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[x]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]],Trace[Hold[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[f[x]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]],_Symbol,{97}]}]"###).unwrap();
    record("depth.tsv", &format!("held96_recursion32\t{}", out.value));
    assert_eq!(out.value.to_string(), "List[1, List[Hold[x]]]");
    assert!(out.termination.is_none());
    assert!(out.messages.is_empty());
    assert!(out.prints.is_empty());
    assert!(s.reject_overlapping_evaluation_request().is_ok());
    assert_eq!(
        s.eval_request("Global`log").unwrap().value.to_string(),
        "List[]"
    );
    assert!(s.pending_input_request().is_none());
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
}
