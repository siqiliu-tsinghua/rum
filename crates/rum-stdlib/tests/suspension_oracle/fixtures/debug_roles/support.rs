use super::super::super::descriptors::{Fixture, Pause, Variant};
use rum_runtime::{RuntimeInputKind, SessionRequestError};

pub const VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
];
pub fn twice(v: Variant) -> bool {
    matches!(
        v,
        Variant::Twice | Variant::AlreadyResumed | Variant::Nested
    )
}
pub fn base(entry: &str, v: Variant) -> Fixture {
    let mut f = Fixture::new(entry, "");
    f.setup =
        "Global`state=0;Global`scope=17;Global`local=19;Global`other=23;Global`count=0".into();
    f.expected_log = Some("List[1, 2, 3, 4, 5]".into());
    f.expected_prints = Some((1..=5).map(|i| i.to_string()).collect());
    project(&mut f, "Global`state", "5");
    project(
        &mut f,
        "{Global`scope,Global`local,Global`other}",
        "List[17, 19, 23]",
    );
    for i in [2, 4] {
        if i == 2 || twice(v) {
            f.pauses.push(Pause {
                prompt: format!("{entry}:{i}"),
                kind: RuntimeInputKind::Input,
                answer: i.to_string(),
            });
        }
    }
    f
}
pub fn project(f: &mut Fixture, expr: &str, expected: &str) {
    f.projections.push(expr.into());
    f.expected_projections.push(expected.into());
}
pub fn step(f: &Fixture, v: Variant, i: usize, value: &str) -> String {
    let input = if i == 2 || (i == 4 && twice(v)) {
        format!("Input[\"{}:{i}\"];", f.id)
    } else {
        String::new()
    };
    let tail = if i == 5 && v == Variant::Error {
        "Throw[\"debug boom\",\"debug tag\"]"
    } else {
        value
    };
    format!("(AppendTo[Global`log,{i}];Print[{i}];Global`state={i};{input}{tail})")
}
pub fn sequence(f: &Fixture, v: Variant, value: &str) -> String {
    format!(
        "({})",
        (1..=5)
            .map(|i| step(f, v, i, value))
            .collect::<Vec<_>>()
            .join(";")
    )
}
pub fn finish(mut f: Fixture, v: Variant, expected: &str) -> Fixture {
    if v == Variant::Error {
        f.expected_error = Some(SessionRequestError::Other {
            message: "uncaught throw: \"debug boom\" with tag \"debug tag\"".into(),
        });
    } else {
        f.expected_value = Some(expected.into());
    }
    if v == Variant::AlreadyResumed {
        f.source = format!("InputString[\"debug caller\"];Identity[{}]", f.source);
        f.pauses.insert(
            0,
            Pause {
                prompt: "debug caller".into(),
                kind: RuntimeInputKind::InputString,
                answer: "caller resumed".into(),
            },
        );
    }
    if v == Variant::Nested {
        f.source = format!("Block[{{Global`scope=31}},{}]", f.source);
    }
    f
}
