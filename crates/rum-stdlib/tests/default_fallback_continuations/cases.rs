use super::support::{Case, compare};

#[test]
fn direct_decline_continues_to_first_ordinary_candidate() {
    compare(Case {
        label: "witness-first-fallback",
        source: r#"log={};n=0;Default[d]:=7/;(n=n+1;AppendTo[log,n];If[n<=2,InputString["p: "]];n>=3);Default[d]"#,
        prompts: &["p: ", "p: "],
        value: Some("7"),
        log: "List[1, 2, 3]",
    });
}

#[test]
fn accepting_direct_control() {
    compare(Case {
        label: "accepting-control",
        source: r#"n=0;Default[d]:=7/;(n=n+1;AppendTo[log,n];If[n==1,InputString["p: "]];n>=2);Default[d]"#,
        prompts: &["p: "],
        value: Some("7"),
        log: "List[1, 2]",
    });
}

#[test]
fn optional_default_declining_control() {
    compare(Case {
        label: "optional-decline-control",
        source: r#"Default[d]:=7/;(AppendTo[log,1];InputString["p: "];False);d[x_.]:=x;d[]"#,
        prompts: &["p: ", "p: "],
        value: Some("d[]"),
        log: "List[1, 1]",
    });
}

#[test]
fn terminal_none_still_finishes_ordinary_scan() {
    compare(Case {
        label: "terminal-none",
        source: r#"n=0;Default[d]:=7/;(n=n+1;AppendTo[log,n];If[n<=2,InputString["p: "]];False);Default[d]"#,
        prompts: &["p: ", "p: "],
        value: Some("Default[d]"),
        log: "List[1, 2, 3, 4]",
    });
}

#[test]
fn ordinary_fallback_first_pattern_candidate() {
    compare(Case {
        label: "first-pattern-fallback",
        source: r#"n=0;Default[x_]:=(AppendTo[log,9];9);Default[d]:=7/;(n=n+1;AppendTo[log,n];If[n==1,InputString["p: "]];False);Default[d]"#,
        prompts: &["p: "],
        value: Some("9"),
        log: "List[1, 9]",
    });
}

#[test]
fn unguarded_direct_hit() {
    compare(Case {
        label: "unguarded-hit",
        source: "Default[d]=7;Default[d]",
        prompts: &[],
        value: Some("7"),
        log: "List[]",
    });
}

#[test]
fn unguarded_direct_miss() {
    compare(Case {
        label: "unguarded-miss",
        source: "Default[e]=7;Default[d]",
        prompts: &[],
        value: Some("Default[d]"),
        log: "List[]",
    });
}

#[test]
fn nested_caller_keeps_its_suffix() {
    compare(Case {
        label: "nested-caller",
        source: r#"n=0;Default[d]:=7/;(n=n+1;AppendTo[log,n];If[n<=2,InputString["p: "]];n>=3);{10+Default[d],(AppendTo[log,4];11)}"#,
        prompts: &["p: ", "p: "],
        value: Some("List[17, 11]"),
        log: "List[1, 2, 4, 3]",
    });
}

#[test]
fn fallback_suspends_again_with_distinct_effects() {
    let out = compare(Case {
        label: "fallback-second-pause",
        source: r#"n=0;Default[d]:=(AppendTo[log,40];Print["body"];7)/;(n=n+1;AppendTo[log,n];If[n==1,Print["direct"];Message[probe::tag];InputString["direct: "];AppendTo[log,10]];If[n==2,Print["fallback"];Message[probe::tag];InputString["fallback: "];AppendTo[log,30]];n>=3);Default[d]"#,
        prompts: &["direct: ", "fallback: "],
        value: Some("7"),
        log: "List[1, 10, 2, 30, 3, 40]",
    });
    assert_eq!(out.prints, ["direct", "fallback", "body"]);
    assert_eq!(
        out.messages
            .iter()
            .map(|m| (m.symbol.as_str(), m.tag.as_str()))
            .collect::<Vec<_>>(),
        [("probe", "tag"), ("probe", "tag")]
    );
}

#[test]
fn direct_guard_error_forwards_without_fallback() {
    let out = compare(Case {
        label: "direct-error",
        source: r#"Default[d]:=(AppendTo[log,99];7)/;(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["error: "];AppendTo[log,2];Print["after"];Message[probe::tag];Throw["default boom"]);Default[d]"#,
        prompts: &["error: "],
        value: None,
        log: "List[1, 2]",
    });
    assert!(out.status.contains("default boom"));
    assert_eq!(out.prints, ["before", "after"]);
}

#[test]
fn fallback_guard_error_forwards_without_body() {
    let out = compare(Case {
        label: "fallback-error",
        source: r#"n=0;Default[d]:=(AppendTo[log,99];7)/;(n=n+1;AppendTo[log,n];If[n==1,InputString["direct: "]];If[n==3,InputString["error: "];Throw["fallback boom"]];False);Default[d]"#,
        prompts: &["direct: ", "error: "],
        value: None,
        log: "List[1, 2, 3]",
    });
    assert!(out.status.contains("fallback boom"));
}

#[test]
fn accepted_body_can_suspend() {
    compare(Case {
        label: "accepted-body",
        source: r#"Default[d]:=(AppendTo[log,1];InputString["body: "];AppendTo[log,2];7);Default[d]"#,
        prompts: &["body: "],
        value: Some("7"),
        log: "List[1, 2]",
    });
}

#[test]
fn completed_guard_side_effects_are_not_replayed() {
    compare(Case {
        label: "guard-finishes",
        source: r#"Default[d]:=(AppendTo[log,3];7)/;(AppendTo[log,1];InputString["guard: "];AppendTo[log,2];True);Default[d]"#,
        prompts: &["guard: "],
        value: Some("7"),
        log: "List[1, 2, 3]",
    });
}

#[test]
fn fallback_reads_live_definitions_after_guard() {
    compare(Case {
        label: "live-fallback-definition",
        source: r#"n=0;Default[d]:=7/;(n=n+1;AppendTo[log,n];If[n==1,InputString["guard: "];Default[x_]:=(AppendTo[log,9];9)];False);Default[d]"#,
        prompts: &["guard: "],
        value: Some("9"),
        log: "List[1, 2, 9]",
    });
}

#[test]
fn ordinary_fallback_middle_candidate() {
    compare(Case {
        label: "middle-fallback",
        source: r#"n=0;Default[x_]:=8/;(AppendTo[log,8];False);Default[x_Symbol]:=(AppendTo[log,90];9)/;(AppendTo[log,9];True);Default[d]:=7/;(n=n+1;AppendTo[log,n];If[n==1,InputString["p: "]];False);Default[d]"#,
        prompts: &["p: "],
        value: Some("9"),
        log: "List[1, 8, 9, 90]",
    });
}
