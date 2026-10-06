use super::support::compare;

#[test]
fn case_sow_match() {
    compare(
        r###"Sow-match"###,
        r###"Reap[Do[Sow[x,x],{x,{1,2,3}}],_?Function[x, (AppendTo[log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]]"###,
        &[r###"two"###],
        &[r###"p: "###],
    );
}

#[test]
fn case_sow_twice() {
    compare(
        r###"Sow-twice"###,
        r###"Reap[Do[Sow[x,x],{x,{1,2,3,4,5}}],_?Function[x,(AppendTo[log,x];If[Or[SameQ[x,2],SameQ[x,4]],InputString["p: "],Null];True)]]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"p: "###],
    );
}

#[test]
fn case_sow_nested_patterns() {
    compare(
        r###"Sow-nested-patterns"###,
        r###"Reap[Reap[Do[Sow[x,x],{x,{1,2,3}}],{_?Function[t,(AppendTo[log,{"innerA",t}];SameQ[t,2])],_?Function[t,(AppendTo[log,{"innerB",t}];If[SameQ[t,2],InputString["inner: "]];True)]}],_?Function[t,(AppendTo[log,{"outer",t}];If[SameQ[t,2],InputString["outer: "]];True)]]"###,
        &[r###"a"###, r###"b"###],
        &[r###"outer: "###, r###"inner: "###],
    );
}

#[test]
fn case_sow_false_then_next_pattern() {
    compare(
        r###"Sow-false-then-next-pattern"###,
        r###"Reap[Sow[7,2],{_?Function[t,(AppendTo[log,1];InputString["decline: "];False)],_?Function[t,(AppendTo[log,2];InputString["accept: "];True)]}]"###,
        &[r###"no"###, r###"yes"###],
        &[r###"decline: "###, r###"accept: "###],
    );
}

#[test]
fn case_sow_same_tag_twice() {
    compare(
        r###"Sow-same-tag-twice"###,
        r###"Reap[Do[Sow[x,9],{x,{1,2,3}}],_?Function[t,(AppendTo[log,t];If[SameQ[Length[log],2],InputString["same: "]];True)]]"###,
        &[r###"a"###],
        &[r###"same: "###],
    );
}

#[test]
fn case_sow_error_after_resume() {
    compare(
        r###"Sow-error-after-resume"###,
        r###"Reap[Do[Sow[x,x],{x,{1,2,3}}],_?Function[t,(AppendTo[log,t];Print["before"];Message[probe::tag];InputString["error: "];AppendTo[log,8];Print["after"];Message[probe::tag];Throw["g7 boom"])]]"###,
        &[r###"a"###],
        &[r###"error: "###],
    );
}

#[test]
fn case_sow_later_pattern_error() {
    compare(
        r###"Sow-later-pattern-error"###,
        r###"Reap[Sow[7,2],{_?Function[t,(AppendTo[log,1];InputString["first: "];True)],_?Function[t,(AppendTo[log,2];Throw["g7 later"])],_?Function[t,(AppendTo[log,3];True)]}]"###,
        &[r###"a"###],
        &[r###"first: "###],
    );
}

#[test]
fn case_sow_direct_return() {
    compare(
        r###"Sow-direct-return"###,
        r###"Reap[Sow[77,2],_?Function[t,(AppendTo[log,t];InputString["value: "];True)]]"###,
        &[r###"a"###],
        &[r###"value: "###],
    );
}

#[test]
fn case_sow_ordinary() {
    compare(
        r###"Sow-ordinary"###,
        r###"Reap[Reap[Do[Sow[x,x],{x,{1,2,3}}],{_Integer,2}],_Integer]"###,
        &[],
        &[],
    );
}
