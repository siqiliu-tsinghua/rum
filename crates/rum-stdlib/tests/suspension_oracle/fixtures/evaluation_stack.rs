//! Resumption preserves each logical evaluation level and its completion checks.
//! Contract: guide/evaluation.md and guide/resource-limits-and-sandbox.md.
use super::super::descriptors::{Checks, Fixture, Pause, accept};
use rum_runtime::RuntimeInputKind;

#[path = "evaluation_stack_support.rs"]
mod support;

fn list(value: &str, count: usize) -> String {
    format!("List[{}]", vec![value; count].join(", "))
}

fn fixture(id: &str, setup: &str, source: &str, prompts: &[&str], value: &str) -> Fixture {
    let mut f = Fixture::new(&format!("g14.{id}"), source);
    f.setup = setup.into();
    f.pauses = prompts
        .iter()
        .map(|prompt| Pause {
            kind: RuntimeInputKind::InputString,
            prompt: (*prompt).into(),
            answer: "x".into(),
        })
        .collect();
    f.expected_value = Some(value.into());
    f.expected_log = Some("List[]".into());
    f.expected_prints = Some(vec![]);
    f.expected_messages = Some(vec![]);
    f.projections.push("{Global`log,Global`t}".into());
    f.expected_projections.push("List[List[], t]".into());
    f
}

fn seeds() -> Vec<Fixture> {
    let f = r#"f[x_]:=(InputString["rhs"];7)"#;
    let mut rows = vec![
        fixture("false-cycle-plus", f, "f[1]+f[1]", &["rhs"; 2], "14"),
        fixture(
            "false-cycle-table",
            f,
            "Table[f[1],{3}]",
            &["rhs"; 3],
            &list("7", 3),
        ),
        fixture(
            "false-cycle-do",
            f,
            "Do[f[1],{1100}]",
            &["rhs"; 1100],
            "Null",
        ),
        fixture(
            "subvalue",
            r#"k[x_][y_]:=(InputString["x"];7)"#,
            "Table[k[1][2],{3}]",
            &["x"; 3],
            &list("7", 3),
        ),
        fixture(
            "ownvalue",
            r#"v:=(InputString["x"];7)"#,
            "Table[v,{3}]",
            &["x"; 3],
            &list("7", 3),
        ),
        fixture(
            "upvalue",
            r#"a/:h[a]:=(InputString["x"];7)"#,
            "Table[h[a],{3}]",
            &["x"; 3],
            &list("7", 3),
        ),
        fixture(
            "nested-lists",
            "",
            r#"Table[{{InputString["x"]}},{3}]"#,
            &["x"; 3],
            &list("List[List[\"x\"]]", 3),
        ),
        fixture(
            "head-list",
            "",
            r#"Table[h[{InputString["x"]}],{3}]"#,
            &["x"; 3],
            &list("h[List[\"x\"]]", 3),
        ),
        fixture(
            "nested-heads",
            "",
            r#"Table[h[k[InputString["x"]]],{3}]"#,
            &["x"; 3],
            &list("h[k[\"x\"]]", 3),
        ),
        fixture(
            "arithmetic",
            "",
            r#"Table[Plus[1,Length[{InputString["x"]}]],{3}]"#,
            &["x"; 3],
            &list("2", 3),
        ),
        fixture(
            "nested-identities",
            "g[x_]:=x",
            r#"Table[g[g[InputString["x"]]],{3}]"#,
            &["x"; 3],
            &list("\"x\"", 3),
        ),
        fixture(
            "depth-leak-1100",
            f,
            "Table[(t=i;f[1]),{i,1100}]",
            &["rhs"; 1100],
            &list("7", 1100),
        ),
        fixture(
            "pass-direct-1100",
            "",
            r#"Table[InputString["x"],{1100}]"#,
            &["x"; 1100],
            &list("\"x\"", 1100),
        ),
        fixture(
            "pass-if",
            "",
            r#"Table[If[True,InputString["x"],0],{3}]"#,
            &["x"; 3],
            &list("\"x\"", 3),
        ),
        fixture(
            "pass-module",
            "",
            r#"Table[Module[{},InputString["x"]],{3}]"#,
            &["x"; 3],
            &list("\"x\"", 3),
        ),
        fixture("pass-map", f, "Map[f,Range[3]]", &["rhs"; 3], &list("7", 3)),
        fixture(
            "pass-set",
            "",
            r#"Table[(v=(InputString["x"];7)),{3}]"#,
            &["x"; 3],
            &list("7", 3),
        ),
        fixture(
            "mid-loop-twice",
            "",
            r#"Table[If[Or[i==2,i==4],h[k[InputString["x"]]],i],{i,5}]"#,
            &["x"; 2],
            "List[1, h[k[\"x\"]], 3, h[k[\"x\"]], 5]",
        ),
        fixture(
            "twice-one-body",
            "",
            r#"Table[{h[k[InputString["a"]]],h[k[InputString["b"]]]},{3}]"#,
            &["a", "b", "a", "b", "a", "b"],
            &list("List[h[k[\"x\"]], h[k[\"x\"]]]", 3),
        ),
        fixture(
            "matcher-backtrack",
            "",
            r#"MatchQ[{1,2},{x___,y_/;(InputString["guard"];False),z___}]"#,
            &["guard"; 2],
            "False",
        ),
        fixture(
            "stack-pass",
            "",
            r#"Table[(InputString["x"];Stack[]),{3}]"#,
            &["x"; 3],
            &list(
                "List[Hold[CompoundExpression[InputString[\"x\"], Stack[]]]]",
                3,
            ),
        ),
    ];
    for depth in 0..=6 {
        let mut body = "InputString[\"x\"]".to_owned();
        let mut value = "\"x\"".to_owned();
        for _ in 0..depth {
            body = format!("h[{body}]");
            value = format!("h[{value}]");
        }
        rows.push(fixture(
            &format!("wrapper-depth-{depth}"),
            "",
            &format!("Table[{body},{{3}}]"),
            &["x"; 3],
            &list(&value, 3),
        ));
    }
    let mut low = fixture(
        "low-limit-changing",
        f,
        "Table[(t=i;f[1]),{i,100}]",
        &["rhs"; 100],
        &list("7", 100),
    );
    low.configure = |s| s.set_recursion_limit(32);
    rows.push(low);
    rows.push(fixture(
        "map-identical",
        f,
        "Map[f,{1,1,1}]",
        &["rhs"; 3],
        &list("7", 3),
    ));
    rows.push(fixture(
        "mid-loop-twice-changing",
        "",
        r#"Table[(t=i;If[Or[i==2,i==4],h[k[InputString["x"]]],i]),{i,5}]"#,
        &["x"; 2],
        "List[1, h[k[\"x\"]], 3, h[k[\"x\"]], 5]",
    ));
    let mut prints = fixture(
        "false-cycle-side-effects",
        f,
        r#"Print["before"];Table[f[1],{3}];Print["after"]"#,
        &["rhs"; 3],
        "Null",
    );
    prints.expected_prints = Some(vec!["before".into(), "after".into()]);
    prints.pause_prints = Some(vec![vec!["before".into()]; 3]);
    rows.push(prints);
    for (id, source) in [
        ("memory-epilogue", "MemoryConstrained[G14Grow[],5000000,99]"),
        ("time-epilogue", "TimeConstrained[G14Same[],1/10,99]"),
    ] {
        let mut native = fixture(id, "", source, &["epilogue"], "99");
        native.configure = support::configure;
        rows.push(native);
    }
    for row in &mut rows {
        let t = match row.id.as_str() {
            "g14.depth-leak-1100" => Some(1100),
            "g14.low-limit-changing" => Some(100),
            "g14.mid-loop-twice-changing" => Some(5),
            _ => None,
        };
        if let Some(t) = t {
            row.expected_projections = vec![format!("List[List[], {t}]")];
        }
    }
    rows
}

fn verify(rows: &[Fixture]) {
    let mut failures = Vec::new();
    for f in rows {
        let result = accept(f, Checks::ALL);
        println!(
            "G14 {} planned={} verdict={:?}",
            f.id,
            f.pauses.len(),
            result.as_ref().map(|_| "PASS")
        );
        support::diagnose(f);
        if let Err(error) = result {
            failures.push(format!("{}: {error:?}", f.id));
        }
    }
    super::super::support::faults::assert_all();
    println!(
        "G14 TOTAL attempted={} failed={} passed={}",
        rows.len(),
        failures.len(),
        rows.len() - failures.len()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn g14_seed_ledger() {
    let rows = seeds();
    assert_eq!(rows.len(), 34);
    verify(&rows);
}

fn dimensions() -> Vec<Fixture> {
    let mut rows = Vec::new();
    for (kind, definition, call) in [
        ("down", r#"f[x_]:=(InputString["rhs"];7)"#, "f[1]"),
        ("sub", r#"k[x_][y_]:=(InputString["rhs"];7)"#, "k[1][2]"),
        ("up", r#"a/:h[a]:=(InputString["rhs"];7)"#, "h[a]"),
        ("own", r#"v:=(InputString["rhs"];7)"#, "v"),
    ] {
        for depth in [0, 1, 2, 3, 4, 6] {
            let mut body = call.to_owned();
            let mut value = "7".to_owned();
            for _ in 0..depth {
                body = format!("wrap[{body}]");
                value = format!("wrap[{value}]");
            }
            rows.push(fixture(
                &format!("{kind}-depth-{depth}"),
                definition,
                &format!("Table[{body},{{3}}]"),
                &["rhs"; 3],
                &list(&value, 3),
            ));
        }
    }
    for count in [3, 4] {
        let tail = (1..=count)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(fixture(
            &format!("tracked-nodes-{}", count + 5),
            "",
            &format!("Table[h[InputString[\"x\"],{tail}],{{3}}]"),
            &["x"; 3],
            &list(&format!("h[\"x\", {tail}]"), 3),
        ));
    }
    for index in [1, 3, 5] {
        let expected = (1..=5)
            .map(|i| {
                if i == index {
                    "h[k[\"x\"]]".into()
                } else {
                    i.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(fixture(
            &format!("iteration-{index}"),
            "",
            &format!("Table[If[i=={index},h[k[InputString[\"x\"]]],i],{{i,5}}]"),
            &["x"],
            &format!("List[{expected}]"),
        ));
    }
    let mut long = fixture(
        "depth-leak-2000",
        r#"f[x_]:=(InputString["rhs"];7)"#,
        "Table[(t=i;f[1]),{i,2000}]",
        &["rhs"; 2000],
        &list("7", 2000),
    );
    long.expected_projections = vec!["List[List[], 2000]".into()];
    rows.push(long);
    let mut epoch = fixture(
        "epoch-mutation",
        r#"t=0;f[x_]:=(InputString["rhs"];If[t==0,t=1;f[x],7])"#,
        "f[1]",
        &["rhs"; 2],
        "7",
    );
    epoch.expected_projections = vec!["List[List[], 1]".into()];
    rows.push(epoch);
    rows.push(fixture(
        "no-input-control",
        "",
        "Table[h[k[7]],{3}]",
        &[],
        &list("h[k[7]]", 3),
    ));
    let mut cycle = fixture(
        "true-active-cycle",
        r#"f[]:=(InputString["cycle"];f[])"#,
        "f[]",
        &["cycle"],
        "TerminatedEvaluation[\"IterationLimit\"]",
    );
    cycle.expected_messages = None;
    rows.push(cycle);
    let mut body = r#"(InputString["depth"];r[3])"#.to_owned();
    for _ in 0..28 {
        body = format!("wrap[{body}]");
    }
    let mut pressure = fixture(
        "untracked-depth-pressure",
        "r[0]=7;r[n_]:=r[n-1]",
        &body,
        &["depth"],
        "TerminatedEvaluation[\"RecursionLimit\"]",
    );
    pressure.configure = |s| s.set_recursion_limit(32);
    pressure.expected_messages = None;
    rows.push(pressure);
    let mut body = r#"finish[(InputString["a"];InputString["b"];7)]"#.to_owned();
    for _ in 0..28 {
        body = format!("wrap[{body}]");
    }
    let mut append = fixture(
        "nested-append-depth-pressure",
        "r[0]=7;r[n_]:=r[n-1];finish[x_]:=r[3]",
        &body,
        &["a", "b"],
        "TerminatedEvaluation[\"RecursionLimit\"]",
    );
    append.configure = |s| s.set_recursion_limit(32);
    append.expected_messages = None;
    rows.push(append);
    let mut widen = fixture(
        "legacy-widen",
        "",
        "Table[G14LegacyFirst[],{3}]",
        &["widen"; 3],
        &list("7", 3),
    );
    widen.configure = support::configure_legacy_first;
    rows.push(widen);
    let mut returned_epoch = fixture(
        "epoch-after-child-return",
        r#"t=0;g[]:=(InputString["epoch"];t=1;0);f[x_]:=If[t==0,{g[],f[x]},7]"#,
        "f[1]",
        &["epoch"],
        "List[0, 7]",
    );
    returned_epoch.expected_projections = vec!["List[List[], 1]".into()];
    rows.push(returned_epoch);
    rows
}

#[test]
fn g14_dimension_ledger() {
    verify(&dimensions());
}

#[test]
fn g14_mutation_panel() {
    let mut rows = seeds();
    rows.extend(dimensions());
    // The ordinary control must execute no Input, including no oracle queue
    // sentinel: that sentinel deliberately suspends and exercises abandonment.
    let ordinary = rows
        .iter()
        .find(|f| f.id == "g14.no-input-control")
        .unwrap();
    let mut observed = Vec::new();
    for _ in 0..2 {
        let mut session = ordinary.session();
        let result = session.eval_request(&ordinary.source);
        let observation = super::super::support::observe::Observation::from_output(&result);
        assert_eq!(
            observation.value.as_ref().map(ToString::to_string),
            ordinary.expected_value
        );
        assert!(observation.error.is_none() && observation.termination.is_none());
        assert!(
            observation.messages.is_empty()
                && observation.prints.is_empty()
                && observation.trace.is_empty()
        );
        assert!(session.pending_input_request().is_none());
        super::super::support::session::fresh(&mut session).unwrap();
        observed.push(observation);
    }
    assert_eq!(observed[0], observed[1]);
    println!("G14 MUTATION g14.no-input-control passed=true (no sentinel input)");
    let assigned = [
        "g14.stack-pass",
        "g14.false-cycle-table",
        "g14.twice-one-body",
        "g14.epoch-mutation",
        "g14.epoch-after-child-return",
        "g14.untracked-depth-pressure",
        "g14.nested-append-depth-pressure",
        "g14.legacy-widen",
        "g14.memory-epilogue",
        "g14.time-epilogue",
    ];
    let mut failures = Vec::new();
    for id in assigned {
        let f = rows.iter().find(|f| f.id == id).unwrap();
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| accept(f, Checks::ALL)));
        let passed = matches!(&result, Ok(Ok(_)));
        println!("G14 MUTATION {id} passed={passed} result={result:?}");
        if !passed {
            failures.push(id);
        }
    }
    super::super::support::faults::assert_all();
    assert!(failures.is_empty(), "mutation panel rejects {failures:?}");
}
