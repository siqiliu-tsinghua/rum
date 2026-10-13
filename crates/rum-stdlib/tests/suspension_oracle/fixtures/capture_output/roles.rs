use super::super::super::descriptors::{Checks, Fixture, Pause, Variant, accept, exercise_fixture};
use super::super::primary::Descriptor;
use rum_runtime::{RuntimeInputKind, RuntimeMessage, SessionRequestError};

const VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
];
const SAME_VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
    Variant::Decline,
];
pub fn message() -> RuntimeMessage {
    RuntimeMessage {
        symbol: "probe".into(),
        tag: "tag".into(),
        text: "oracle message".into(),
    }
}
pub fn pause(prompt: &str) -> Pause {
    Pause {
        prompt: prompt.into(),
        kind: RuntimeInputKind::InputString,
        answer: "answer".into(),
    }
}
pub fn descriptors() -> Vec<Descriptor> {
    let rows: &[(&str, &str, fn(&str, Variant) -> Fixture)] = &[
        ("capture.evaluation-data", "EvaluationData.body", within),
        ("capture.report-input-within", "TestReport.input", within),
        ("capture.report-input-cases", "TestReport.input", cases),
        (
            "capture.report-expected-within",
            "TestReport.expectedOutput",
            within,
        ),
        (
            "capture.report-expected-cases",
            "TestReport.expectedOutput",
            cases,
        ),
        ("capture.report-same-within", "TestReport.sameTest", within),
        ("capture.report-same-cases", "TestReport.sameTest", cases),
    ];
    rows.iter()
        .map(|&(id, entry, factory)| Descriptor {
            id,
            entry,
            factory,
            variants: if entry == "TestReport.sameTest" {
                SAME_VARIANTS
            } else {
                VARIANTS
            },
            boundary: false,
            lifecycle: exercise,
        })
        .collect()
}
fn within(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, false)
}
fn cases(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, true)
}
fn fixture(entry: &str, v: Variant, separate: bool) -> Fixture {
    let mut f = Fixture::new(entry, "");
    let captured = matches!(entry, "EvaluationData.body" | "TestReport.input");
    let twice = v != Variant::Middle;
    f.pauses.push(pause("capture:1"));
    if twice {
        f.pauses.push(pause("capture:2"));
    }
    f.pause_prints = Some(
        (0..f.pauses.len())
            .map(|i| {
                let mut p = vec!["outside before".into()];
                if !captured {
                    p.extend((1..=if separate { i * 2 + 1 } else { i + 1 }).map(|j| j.to_string()));
                }
                p
            })
            .collect(),
    );
    f.pause_messages = Some(
        (0..f.pauses.len())
            .map(|i| {
                vec![
                    message();
                    if captured {
                        1
                    } else {
                        if separate { i * 2 + 2 } else { i + 2 }
                    }
                ]
            })
            .collect(),
    );
    f.pause_trace = Some(vec![vec![]; f.pauses.len()]);
    let steps = if separate { 4 } else { 3 };
    let step = |i: usize, stop: Option<usize>, result: &str| {
        let input = stop
            .filter(|n| *n == 1 || twice)
            .map(|n| format!("InputString[\"capture:{n}\"];"))
            .unwrap_or_default();
        let tail = if v == Variant::Error && i == steps {
            "Exit[7]"
        } else {
            result
        };
        format!("(AppendTo[Global`log,{i}];Print[{i}];Message[Global`probe::tag];{input}{tail})")
    };
    let body = if separate {
        String::new()
    } else {
        format!(
            "({};{};{})",
            step(1, Some(1), "7"),
            step(2, Some(2), "7"),
            step(3, None, if v == Variant::Decline { "False" } else { "7" })
        )
    };
    let expression = if entry == "EvaluationData.body" {
        format!("EvaluationData[{body},{{\"Result\",\"Success\",\"OutputLog\",\"Messages\"}}]")
    } else {
        let mut tests = Vec::new();
        for case in 0..if separate { 2 } else { 1 } {
            let value = if separate {
                format!(
                    "({};{})",
                    step(case * 2 + 1, Some(case + 1), "7"),
                    step(
                        case * 2 + 2,
                        None,
                        if v == Variant::Decline { "False" } else { "7" }
                    )
                )
            } else {
                body.clone()
            };
            let expected_messages = if separate { 2 } else { 3 };
            let messages = vec!["Global`probe::tag"; expected_messages].join(",");
            tests.push(match entry {
                "TestReport.input" => {
                    format!("TestCreate[{value},7,{{ExpectedMessages->{{{messages}}}}}]")
                }
                "TestReport.expectedOutput" => format!("TestCreate[7,{value}]"),
                "TestReport.sameTest" => format!(
                    "TestCreate[7,7,{{SameTest->Function[{{a,b}},({value};{})]}}]",
                    if v == Variant::Decline {
                        "False"
                    } else {
                        "SameQ[a,b]"
                    }
                ),
                _ => unreachable!(),
            });
        }
        format!(
            "Global`report=TestReport[{{{}}}];{{Global`report[\"TestsSucceededCount\"],Global`report[\"TestsFailedCount\"],Map[Function[t,{{t[\"ActualOutput\"],t[\"Success\"],t[\"OutputLog\"],t[\"ActualMessages\"]}}],Global`report[\"TestResults\"]]}}",
            tests.join(",")
        )
    };
    f.source = format!(
        "Print[\"outside before\"];Message[Global`probe::tag];Global`result=({expression});Print[\"outside after\"];Message[Global`probe::tag];Global`result"
    );
    f.expected_log = Some(format!(
        "List[{}]",
        (1..=steps)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    let mut prints = vec!["outside before".into()];
    if !captured || (v == Variant::Error && entry == "TestReport.input") {
        prints.extend(
            (if separate && captured {
                3..=steps
            } else {
                1..=steps
            })
            .map(|i| i.to_string()),
        );
    }
    if v != Variant::Error {
        prints.push("outside after".into());
    }
    f.expected_prints = Some(prints);
    let count = if captured && !(v == Variant::Error && entry == "TestReport.input") {
        0
    } else if separate && captured {
        2
    } else {
        steps
    };
    f.expected_messages = Some(vec![
        message();
        count + if v == Variant::Error { 1 } else { 2 }
    ]);
    if v == Variant::Error {
        f.expected_error = Some(SessionRequestError::Exit { code: 7 });
    } else if entry == "EvaluationData.body" {
        f.expected_value=Some(r#"Association[Rule["Result", 7], Rule["Success", True], Rule["OutputLog", List["1", "2", "3"]], Rule["Messages", List[probe::tag, probe::tag, probe::tag]]]"#.into());
    } else {
        let count = if separate { 2 } else { 1 };
        let results = (0..count)
            .map(|case| {
                let logs = if captured {
                    format!(
                        "List[{}]",
                        (if separate {
                            case * 2 + 1..=case * 2 + 2
                        } else {
                            1..=3
                        })
                        .map(|i| format!("\"{i}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                    )
                } else {
                    "List[]".into()
                };
                let messages = if captured {
                    format!(
                        "List[{}]",
                        vec!["probe::tag"; if separate { 2 } else { 3 }].join(", ")
                    )
                } else {
                    "List[]".into()
                };
                format!(
                    "List[7, {}, {logs}, {messages}]",
                    if v == Variant::Decline {
                        "False"
                    } else {
                        "True"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        f.expected_value = Some(format!(
            "List[{}, {}, List[{results}]]",
            if v == Variant::Decline { 0 } else { count },
            if v == Variant::Decline { count } else { 0 }
        ));
    }
    if v == Variant::AlreadyResumed {
        f.source = format!("InputString[\"caller\"];{}", f.source);
        f.pauses.insert(0, pause("caller"));
        f.pause_prints.as_mut().unwrap().insert(0, vec![]);
        f.pause_messages.as_mut().unwrap().insert(0, vec![]);
        f.pause_trace.as_mut().unwrap().insert(0, vec![]);
        f.lifecycle_pauses = [1, 2];
    }
    if v == Variant::Nested {
        // An inner completion and its released output still belong to this outer capture.
        f.source = format!("EvaluationData[({}),\"Result\"]", f.source);
        f.expected_prints = Some(vec![]);
        f.expected_messages = Some(vec![]);
        f.pause_prints = Some(vec![vec![]; f.pauses.len()]);
        f.pause_messages = Some(vec![vec![]; f.pauses.len()]);
    }
    f
}
pub fn exercise(f: &Fixture) -> usize {
    exercise_fixture(f, |s, stop, reset| {
        let expected = if reset {
            "List[]"
        } else if stop == 0 {
            "List[1]"
        } else if f.source.matches("TestCreate[").count() == 2 {
            "List[1, 2, 3]"
        } else {
            "List[1, 2]"
        };
        assert_eq!(
            s.eval_request("Global`log").unwrap().value.to_string(),
            expected,
            "{} lifecycle",
            f.id
        );
        assert!(!s.configuration().tracing_enabled);
    })
}
#[test]
fn g11_admission_matrix() {
    for d in descriptors() {
        for &v in d.variants {
            let f = (d.factory)(d.entry, v);
            accept(&f, Checks::ALL).unwrap_or_else(|e| panic!("{} {v:?}: {e:?}", d.id));
            println!("G11 {} {v:?} PASS", d.id);
        }
        (d.lifecycle)(&(d.factory)(d.entry, Variant::Twice));
        println!("G11 {} lifecycle first=PASS second=PASS", d.id);
    }
}
