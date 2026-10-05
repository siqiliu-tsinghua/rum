// Queued and resumed held patterns have identical values and ordered effects.
use super::support::compare;

#[test]
fn matchq_pattern_args() {
    compare(
        r###"MatchQ-pattern-args"###,
        r###"MatchQ[h[1,2,3],h[_,(AppendTo[log,2];Input["p: "]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn matchq_pattern_head() {
    compare(
        r###"MatchQ-pattern-head"###,
        r###"MatchQ[h[1],(AppendTo[log,2];Input["p: "])[_]]"###,
        &[r###"h"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn matchq_pattern_twice() {
    compare(
        r###"MatchQ-pattern-twice"###,
        r###"MatchQ[h[1,2,3,4,5],h[_,(AppendTo[log,2];Input["p: "]),3,(AppendTo[log,4];Input["p: "]),5]]"###,
        &[r###"2"###, r###"4"###],
        &[r###"p: "###, r###"p: "###],
        r###""###,
    );
}

#[test]
fn matchq_normalize_head() {
    compare(
        r###"MatchQ-normalize-head"###,
        r###"MatchQ[f[1,2,3],(AppendTo[log,1];Input["p: "])[_,_,_]]"###,
        &[r###"f"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn cases_pattern() {
    compare(
        r###"Cases-pattern"###,
        r###"Cases[{h[1,2,3],h[4,5,6]},h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn deletecases_pattern() {
    compare(
        r###"DeleteCases-pattern"###,
        r###"DeleteCases[{h[1,2,3],h[4,5,6]},h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn position_pattern() {
    compare(
        r###"Position-pattern"###,
        r###"Position[{h[1,2,3],h[4,5,6]},h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn count_pattern() {
    compare(
        r###"Count-pattern"###,
        r###"Count[{h[1,2,3],h[4,5,6]},h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn memberq_pattern() {
    compare(
        r###"MemberQ-pattern"###,
        r###"MemberQ[{h[1,2,3],h[4,5,6]},h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn freeq_pattern() {
    compare(
        r###"FreeQ-pattern"###,
        r###"FreeQ[{h[1,2,3],h[4,5,6]},h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn catch_pattern() {
    compare(
        r###"Catch-pattern"###,
        r###"Catch[Throw[99,h[1,2,3]],h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn reap_pattern() {
    compare(
        r###"Reap-pattern"###,
        r###"Reap[Sow[99,h[1,2,3]],h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn matchq_distinct_twice() {
    compare(
        r###"MatchQ-distinct-twice"###,
        r###"MatchQ[h[1,2,4,5],h[x_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,4];Input["second: "]),(AppendTo[log,5];5)]]"###,
        &[r###"2"###, r###"4"###],
        &[r###"first: "###, r###"second: "###],
        r###""###,
    );
}

#[test]
fn cases_distinct_twice() {
    compare(
        r###"Cases-distinct-twice"###,
        r###"Cases[{h[1,2,4,5]},h[x_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,4];Input["second: "]),(AppendTo[log,5];5)]]"###,
        &[r###"2"###, r###"4"###],
        &[r###"first: "###, r###"second: "###],
        r###""###,
    );
}

#[test]
fn matchq_nested_pattern() {
    compare(
        r###"MatchQ-nested-pattern"###,
        r###"MatchQ[h[g[1,2],7,3],h[g[x_,(AppendTo[log,2];Input["first: "])],y_,(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn cases_nested_pattern() {
    compare(
        r###"Cases-nested-pattern"###,
        r###"Cases[{h[g[1,2],7,3]},h[g[x_,(AppendTo[log,2];Input["first: "])],y_,(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn matchq_prepared_call_suspends() {
    compare(
        r###"MatchQ-prepared-call-suspends"###,
        r###"MatchQ[2,h[_,(AppendTo[log,1];1)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###"h[a_,b_]:=(AppendTo[log,2];Input["first: "]);"###,
    );
}

#[test]
fn matchq_error_after_resume() {
    compare(
        r###"MatchQ-error-after-resume"###,
        r###"MatchQ[h[1,2,3,4],h[x_,(AppendTo[log,1];Print["prefix"];Message[probe::tag];Input["first: "]),(AppendTo[log,2];Print["suffix"];Message[probe::tag];Throw["g4 boom"]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn cases_error_after_resume() {
    compare(
        r###"Cases-error-after-resume"###,
        r###"Cases[{h[1,2,3,4]},h[x_,(AppendTo[log,1];Print["prefix"];Message[probe::tag];Input["first: "]),(AppendTo[log,2];Print["suffix"];Message[probe::tag];Throw["g4 boom"]),(AppendTo[log,3];3)]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn matchq_condition_held() {
    compare(
        r###"MatchQ-Condition-held"###,
        r###"MatchQ[z,h[Condition[x_,(AppendTo[log,99];InputString["held: "];True)],(AppendTo[log,2];Input["first: "])]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn matchq_patterntest_held() {
    compare(
        r###"MatchQ-PatternTest-held"###,
        r###"MatchQ[z,h[PatternTest[x_,(AppendTo[log,99];InputString["held: "];True)],(AppendTo[log,2];Input["first: "])]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn control_cases() {
    compare(
        r###"control-Cases"###,
        r###"Cases[{1,2,3},_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]]"###,
        &[r###"two"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn control_cases_target() {
    compare(
        r###"control-Cases-target"###,
        r###"Cases[(AppendTo[log,1];InputString["p: "];{1,2,3}),2]"###,
        &[r###"two"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn control_cases_spec() {
    compare(
        r###"control-Cases-spec"###,
        r###"Cases[{1,2,3},(AppendTo[log,2];InputString["p: "];2)]"###,
        &[r###"two"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn control_cases_rule() {
    compare(
        r###"control-Cases-rule"###,
        r###"Cases[{1,2,3},z_Integer:>(AppendTo[log,z];If[SameQ[z,2],InputString["p: "],z])]"###,
        &[r###"two"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn control_matchq_normalize_twice() {
    compare(
        r###"control-MatchQ-normalize-twice"###,
        r###"MatchQ[f[1,2,3,4,5],f[1,(AppendTo[log,2];Input["p: "]),3,(AppendTo[log,4];Input["p: "]),5]]"###,
        &[r###"2"###, r###"4"###],
        &[r###"p: "###, r###"p: "###],
        r###""###,
    );
}

#[test]
fn replacepart_resolver() {
    compare(
        r###"ReplacePart-resolver"###,
        r###"ReplacePart[{{1,2},{3,4}}, HoldPattern[{_,(AppendTo[log,2];Input["first: "])}]:>9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn replacepart_guarded_resolver() {
    compare(
        r###"ReplacePart-guarded-resolver"###,
        r###"ReplacePart[{{1,2},{3,4}}, HoldPattern[{_,(AppendTo[log,2];Input["first: "])}]:>(9/;True)]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn matchq_nested_prepared_call() {
    compare(
        r###"MatchQ-nested-prepared-call"###,
        r###"MatchQ[h[2,_],h[g[_,(AppendTo[log,1];1)],_]]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###"g[a_,b_]:=(AppendTo[log,2];Input["first: "]);"###,
    );
}
