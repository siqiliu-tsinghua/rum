use super::super::descriptors::{Fixture, Pause, Variant};
use rum_runtime::RuntimeInputKind;

pub fn scan(variant: Variant) -> Fixture {
    let twice = matches!(
        variant,
        Variant::Twice | Variant::AlreadyResumed | Variant::Nested
    );
    let source = format!(
        r#"Scan[Function[x,(AppendTo[Global`log,x];Print[x];If[x==2,AppendTo[Global`log,InputString["Scan.callback:2"]]];{}{}x)],{{1,2,3,4,5}}]"#,
        if twice {
            r#"If[x==4,AppendTo[Global`log,InputString["Scan.callback:4"]]];"#
        } else {
            ""
        },
        if variant == Variant::Error {
            "If[x==5,Throw[9]];"
        } else {
            ""
        }
    );
    let mut f = Fixture::new("Scan.callback", &source);
    f.pauses = vec![Pause {
        prompt: "Scan.callback:2".into(),
        kind: RuntimeInputKind::InputString,
        answer: "two".into(),
    }];
    if twice {
        f.pauses.push(Pause {
            prompt: "Scan.callback:4".into(),
            kind: RuntimeInputKind::InputString,
            answer: "four".into(),
        });
    }
    if variant != Variant::Error {
        f.expected_value = Some("Null".into());
    }
    f.expected_log = Some(
        if twice {
            r#"List[1, 2, "two", 3, 4, "four", 5]"#
        } else {
            r#"List[1, 2, "two", 3, 4, 5]"#
        }
        .into(),
    );
    if variant == Variant::AlreadyResumed {
        f.source = format!("InputString[\"caller\"];{}", f.source);
        f.pauses.insert(
            0,
            Pause {
                prompt: "caller".into(),
                kind: RuntimeInputKind::InputString,
                answer: "caller".into(),
            },
        );
    }
    if variant == Variant::Nested {
        f.source = format!("Block[{{Global`local=7}},{}]", f.source);
    }
    f
}

pub struct Descriptor {
    pub id: &'static str,
    pub entry: &'static str,
    pub variants: &'static [Variant],
    pub factory: fn(&str, Variant) -> Fixture,
    pub boundary: bool,
    pub lifecycle: fn(&Fixture) -> usize,
}
const SCAN_VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
];
const RETIRED_VARIANTS: &[Variant] = &[Variant::Middle, Variant::Twice];
fn valueq(v: Variant) -> Fixture {
    retired("ValueQ.trial", v)
}
fn monitor(v: Variant) -> Fixture {
    retired("Monitor.display", v)
}
fn monitor_eval(v: Variant) -> Fixture {
    retired("Monitor.evaluationMonitor", v)
}
fn monitor_step(v: Variant) -> Fixture {
    retired("Monitor.stepMonitor", v)
}
fn retired(entry: &str, v: Variant) -> Fixture {
    super::boundaries::fixture(entry, v)
}
pub fn descriptors() -> Vec<Descriptor> {
    let mut rows = vec![
        Descriptor {
            id: "primary.preparation",
            entry: "Evaluator.callPreparation",
            variants: &[
                Variant::Middle,
                Variant::Twice,
                Variant::Decline,
                Variant::Error,
                Variant::AlreadyResumed,
                Variant::Nested,
            ],
            factory: |_, v| super::preparation::fixture(v),
            boundary: false,
            lifecycle: exercise_lifecycle,
        },
        Descriptor {
            id: "primary.scan",
            entry: "Scan.callback",
            variants: SCAN_VARIANTS,
            factory: |_, v| scan(v),
            boundary: false,
            lifecycle: exercise_lifecycle,
        },
        Descriptor {
            id: "primary.valueq",
            entry: "ValueQ.trial",
            variants: RETIRED_VARIANTS,
            factory: |_, v| valueq(v),
            boundary: true,
            lifecycle: exercise_lifecycle,
        },
        Descriptor {
            id: "primary.monitor-display",
            entry: "Monitor.display",
            variants: RETIRED_VARIANTS,
            factory: |_, v| monitor(v),
            boundary: true,
            lifecycle: exercise_lifecycle,
        },
        Descriptor {
            id: "primary.monitor-evaluation",
            entry: "Monitor.evaluationMonitor",
            variants: RETIRED_VARIANTS,
            factory: |_, v| monitor_eval(v),
            boundary: true,
            lifecycle: exercise_lifecycle,
        },
        Descriptor {
            id: "primary.monitor-step",
            entry: "Monitor.stepMonitor",
            variants: RETIRED_VARIANTS,
            factory: |_, v| monitor_step(v),
            boundary: true,
            lifecycle: exercise_lifecycle,
        },
    ];
    rows.extend(super::rules::descriptors());
    rows.extend(super::rewrite_roles::descriptors());
    rows
}
pub fn execute(descriptor: &Descriptor) -> usize {
    use super::super::descriptors::{Checks, accept};
    for &variant in descriptor.variants {
        let f = (descriptor.factory)(descriptor.entry, variant);
        assert_eq!(f.id, descriptor.entry);
        accept(&f, Checks::ALL).unwrap_or_else(|e| panic!("{} {variant:?}: {e:?}", descriptor.id));
        println!("VARIANT {} {variant:?} PASS", descriptor.entry);
    }
    if descriptor.boundary {
        return 0;
    }
    let f = (descriptor.factory)(descriptor.entry, Variant::Twice);
    let obligations = (descriptor.lifecycle)(&f);
    println!(
        "LIFECYCLE {} first=PASS second=PASS completion/cancel/reset/pending",
        descriptor.entry
    );
    obligations
}

fn exercise_lifecycle(f: &Fixture) -> usize {
    let plans = f
        .pauses
        .iter()
        .map(|p| {
            (
                rum_runtime::RuntimeInputRequest {
                    kind: p.kind,
                    prompt: p.prompt.clone(),
                    password: false,
                },
                p.answer.clone(),
            )
        })
        .collect::<Vec<_>>();
    super::super::support::lifecycle::exercise(|| f.session(), &f.source, &plans)
}
