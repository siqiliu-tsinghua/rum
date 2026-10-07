//! Shared result tails must complete while their resuming frame is still rooted.
use super::super::descriptors::{Checks, Fixture, Pause, Variant, accept};
use super::super::support::lifecycle;
use super::primary::Descriptor;
use rum_runtime::{RuntimeInputKind, RuntimeInputRequest, SessionRequestError};

const VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
];
const MATCH_VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
    Variant::Decline,
];
const ASSOCIATION: &str =
    r#"Association[Rule["a", Association[Rule["b", 0], Rule["keep", 7]]], Rule["keep", 11]]"#;

pub fn descriptors() -> Vec<Descriptor> {
    let rows: &[(&str, &str, fn(&str, Variant) -> Fixture)] = &[
        (
            "TestReport.explicitOption.sharedInput",
            "TestReport.explicitOption",
            |e, v| report(e, v, "input"),
        ),
        (
            "TestReport.explicitOption.sharedExpectedOutput",
            "TestReport.explicitOption",
            |e, v| report(e, v, "expected"),
        ),
        (
            "TestReport.explicitOption.sharedSameTest",
            "TestReport.explicitOption",
            |e, v| report(e, v, "same"),
        ),
        ("Set.target.associationSelectors", "Set.target", association),
        (
            "DeleteCases.patternTest.sharedKeep",
            "DeleteCases.patternTest",
            |e, v| deletion(e, v, "keep"),
        ),
        (
            "DeleteCases.patternTest.sharedMixed",
            "DeleteCases.patternTest",
            |e, v| deletion(e, v, "mixed"),
        ),
        (
            "DeleteCases.patternTest.sharedAssociation",
            "DeleteCases.patternTest",
            |e, v| deletion(e, v, "association"),
        ),
        (
            "DeleteCases.patternTest.sharedLimit",
            "DeleteCases.patternTest",
            |e, v| deletion(e, v, "limit"),
        ),
        (
            "DeleteCases.guard.sharedKeep",
            "DeleteCases.guard",
            |e, v| deletion(e, v, "keep"),
        ),
        (
            "DeleteCases.guard.sharedMixed",
            "DeleteCases.guard",
            |e, v| deletion(e, v, "mixed"),
        ),
        (
            "DeleteCases.guard.sharedAssociation",
            "DeleteCases.guard",
            |e, v| deletion(e, v, "association"),
        ),
        (
            "DeleteCases.guard.sharedLimit",
            "DeleteCases.guard",
            |e, v| deletion(e, v, "limit"),
        ),
    ];
    rows.iter()
        .map(|&(id, entry, factory)| Descriptor {
            id,
            entry,
            factory,
            variants: if entry.starts_with("DeleteCases.") {
                MATCH_VARIANTS
            } else {
                VARIANTS
            },
            boundary: false,
            lifecycle: exercise_lifecycle,
        })
        .collect()
}

fn pause(prompt: String, answer: &str) -> Pause {
    Pause {
        prompt,
        kind: RuntimeInputKind::InputString,
        answer: answer.into(),
    }
}

fn base(entry: &str, v: Variant, phase: &str) -> Fixture {
    let mut f = Fixture::new(entry, "");
    f.setup = "Global`state=0;Global`scope=17".into();
    f.expected_prints = Some(vec![]);
    f.projections = vec!["Global`state".into(), "Global`scope".into()];
    for i in 1..=if v == Variant::Middle { 1 } else { 2 } {
        f.pauses.push(pause(format!("{phase}:{i}"), &i.to_string()));
    }
    f
}

fn step(f: &Fixture, i: usize, value: &str) -> String {
    let input = f
        .pauses
        .get(i - 1)
        .map(|p| format!("InputString[\"{}\"];", p.prompt))
        .unwrap_or_default();
    format!("(AppendTo[Global`log,{i}];Global`state={i};{input}{value})")
}

fn finish(mut f: Fixture, v: Variant, expected: &str, visited: usize) -> Fixture {
    f.expected_value = Some(expected.into());
    f.expected_log = Some(format!(
        "List[{}]",
        (1..=visited)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    f.expected_projections = vec![visited.to_string(), "17".into()];
    if v == Variant::Error {
        // An error after completion must not leave the completed owner pending.
        f.source = format!(
            "({};Throw[\"completion boom\",\"completion tag\"])",
            f.source
        );
        f.expected_value = None;
        f.expected_error = Some(SessionRequestError::Other {
            message: "uncaught throw: \"completion boom\" with tag \"completion tag\"".into(),
        });
    }
    if v == Variant::AlreadyResumed {
        f.source = format!("InputString[\"completion caller\"];{}", f.source);
        f.pauses
            .insert(0, pause("completion caller".into(), "caller"));
        f.lifecycle_pauses = [1, 2];
    }
    if v == Variant::Nested {
        f.source = format!("Block[{{Global`scope=73}},{}]", f.source);
    }
    f
}

fn report(entry: &str, v: Variant, phase: &str) -> Fixture {
    let mut f = base(entry, v, &format!("report {phase}"));
    let tests = (1..=2)
        .map(|i| {
            let body = step(&f, i, &i.to_string());
            match phase {
                "input" => format!("TestCreate[{body},{i}]"),
                "expected" => format!("TestCreate[{i},{body}]"),
                "same" => format!("TestCreate[{i},{i}]"),
                _ => unreachable!(),
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    let same_test = if phase == "same" {
        format!(
            "Function[{{a,b}},If[a==1,{},{}]]",
            step(&f, 1, "SameQ[a,b]"),
            step(&f, 2, "SameQ[a,b]")
        )
    } else {
        "SameQ".into()
    };
    // Project both retained result payloads, not just the independent counters.
    // Timing leaves never enter the observable value compared by the oracle.
    f.source = format!(
        "Map[Function[test,{{test[\"ActualOutput\"],test[\"Success\"]}}],TestReport[{{{tests}}},\"TestResults\",SameTest->{same_test}]]"
    );
    finish(f, v, "List[List[1, True], List[2, True]]", 2)
}

fn association(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, "association selector");
    f.setup.push_str(&format!(";Global`assoc={ASSOCIATION}"));
    f.source = format!(
        "Global`assoc[{},{}]=9",
        step(&f, 1, r#""a""#),
        step(&f, 2, r#""b""#)
    );
    let mut f = finish(f, v, "9", 2);
    f.projections.push("Global`assoc".into());
    f.expected_projections
        .push(ASSOCIATION.replace("Rule[\"b\", 0]", "Rule[\"b\", 9]"));
    f
}

fn deletion(entry: &str, v: Variant, shape: &str) -> Fixture {
    let mut f = base(entry, v, &format!("{entry} {shape}"));
    let decline = v == Variant::Decline;
    let result = if !decline && matches!(shape, "mixed" | "limit") {
        "x==2"
    } else {
        "False"
    };
    let inputs = f
        .pauses
        .iter()
        .enumerate()
        .map(|(i, p)| format!("If[x=={},InputString[\"{}\"]];", i + 1, p.prompt))
        .collect::<String>();
    let body = format!("(AppendTo[Global`log,x];Global`state=x;{inputs}{result})");
    let pattern = if entry.ends_with(".guard") {
        format!("x_Integer/;{body}")
    } else {
        format!("_Integer?Function[x,{body}]")
    };
    let (target, limit, expected, visited) = match shape {
        "keep" => ("{1,2}", "", "List[1, 2]", 2),
        "mixed" => (
            "{1,2,3}",
            "",
            if decline {
                "List[1, 2, 3]"
            } else {
                "List[1, 3]"
            },
            3,
        ),
        "association" => (
            r#"<|"a"->1,"b"->2,"c"->3|>"#,
            "",
            r#"Association[Rule["a", 1], Rule["b", 2], Rule["c", 3]]"#,
            3,
        ),
        "limit" => (
            "{1,2,3,4}",
            ",1",
            if decline {
                "List[1, 2, 3, 4]"
            } else {
                "List[1, 3, 4]"
            },
            if decline { 4 } else { 2 },
        ),
        _ => unreachable!(),
    };
    f.source = format!("DeleteCases[{target},{pattern},{{1}}{limit},Heads->False]");
    finish(f, v, expected, visited)
}

fn exercise_lifecycle(f: &Fixture) -> usize {
    let plans = f
        .pauses
        .iter()
        .map(|p| {
            (
                RuntimeInputRequest {
                    kind: p.kind,
                    prompt: p.prompt.clone(),
                    password: false,
                },
                p.answer.clone(),
            )
        })
        .collect::<Vec<_>>();
    lifecycle::exercise_observed_at(
        || f.session(),
        &f.source,
        &plans,
        f.lifecycle_pauses,
        |s, stop, reset| {
            let (log, state) = if reset {
                ("List[]", "0")
            } else if stop == 0 {
                ("List[1]", "1")
            } else {
                ("List[1, 2]", "2")
            };
            for (source, expected) in [
                ("Global`log", log),
                ("Global`state", state),
                ("Global`scope", "17"),
            ] {
                assert_eq!(
                    s.eval_request(source).unwrap().value.to_string(),
                    expected,
                    "{} lifecycle {source}",
                    f.id
                );
            }
            if f.id == "Set.target" {
                assert_eq!(
                    s.eval_request("Global`assoc").unwrap().value.to_string(),
                    ASSOCIATION,
                    "cancelled selectors must not assign"
                );
            }
        },
    )
}

#[test]
fn shared_completion_variants_and_lifecycle() {
    for d in descriptors() {
        for &v in d.variants {
            accept(&(d.factory)(d.entry, v), Checks::ALL)
                .unwrap_or_else(|e| panic!("{} {v:?}: {e:?}", d.id));
            println!("SHARED_COMPLETION {} {v:?} PASS", d.id);
        }
        assert_eq!((d.lifecycle)(&(d.factory)(d.entry, Variant::Twice)), 6);
        println!(
            "SHARED_COMPLETION {} lifecycle first=PASS second=PASS",
            d.id
        );
    }
}

#[test]
fn original_panic_witnesses_complete() {
    let mut association = super::make(
        "brief-594.association",
        r#"Global`assoc[InputString["assoc:1"],InputString["assoc:2"]]=9"#,
        &[
            ("InputString", "assoc:1", "a"),
            ("InputString", "assoc:2", "b"),
        ],
        Some("9"),
        "List[]",
    );
    association.setup = r#"Global`assoc=<|"a"-><|"b"->0|>|>"#.into();
    association.projections = vec![r#"Global`assoc["a","b"]"#.into()];
    association.expected_projections = vec!["9".into()];
    for f in [
        super::make(
            "brief-594.report",
            r#"TestReport[{TestCreate[InputString["report:1"],"a"],TestCreate[InputString["report:2"],"b"]},"TestsSucceededCount"]"#,
            &[
                ("InputString", "report:1", "a"),
                ("InputString", "report:2", "b"),
            ],
            Some("2"),
            "List[]",
        ),
        association,
        super::make(
            "brief-594.delete-cases",
            r#"DeleteCases[{1,2},_?Function[x,(InputString["delete"];False)]]"#,
            &[
                ("InputString", "delete", "a"),
                ("InputString", "delete", "b"),
            ],
            Some("List[1, 2]"),
            "List[]",
        ),
    ] {
        let out = accept(&f, Checks::ALL).unwrap();
        println!(
            "SHARED_WITNESS {} value={} PASS",
            f.id,
            out.observation.value.unwrap()
        );
    }
}
