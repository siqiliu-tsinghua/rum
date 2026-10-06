use super::super::super::descriptors::{Fixture, Pause, Variant};
use rum_runtime::{RuntimeInputKind, SessionRequestError};

pub const VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
];
pub const DECLINE_VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
    Variant::Decline,
];

pub fn twice(v: Variant) -> bool {
    matches!(
        v,
        Variant::Twice | Variant::AlreadyResumed | Variant::Nested
    )
}

pub fn list(n: usize) -> String {
    format!(
        "List[{}]",
        (1..=n)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn project(f: &mut Fixture, expression: &str, expected: &str) {
    f.projections.push(expression.into());
    f.expected_projections.push(expected.into());
}

pub fn base(entry: &str, v: Variant, count: usize) -> Fixture {
    let mut f = Fixture::new(entry, "");
    f.setup =
        "Global`state=0;Global`scope=17;Global`answers=0;Global`selected=0;Global`calls=0".into();
    let count = if v == Variant::Error { 5 } else { count };
    f.expected_log = Some(list(count));
    f.expected_prints = Some((1..=count).map(|i| i.to_string()).collect());
    project(&mut f, "Global`state", &count.to_string());
    project(&mut f, "Global`scope", "17");
    project(&mut f, "Global`answers", if twice(v) { "6" } else { "2" });
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

pub fn step(f: &Fixture, v: Variant, i: usize, result: &str) -> String {
    let input = if i == 2 || (i == 4 && twice(v)) {
        format!("Global`answers+=Input[\"{}:{i}\"];", f.id)
    } else {
        String::new()
    };
    let tail = if i == 5 && v == Variant::Error {
        "Throw[\"predicate boom\",\"predicate tag\"]"
    } else {
        result
    };
    format!("(AppendTo[Global`log,{i}];Print[{i}];Global`state={i};{input}{tail})")
}

pub fn sequence(f: &Fixture, v: Variant, result: &str) -> String {
    format!(
        "({})",
        (1..=5)
            .map(|i| step(f, v, i, result))
            .collect::<Vec<_>>()
            .join(";")
    )
}

// Each arm is a separate visit by the builtin's loop. The state is updated
// before input; a replay or skipped visit changes the log and answer sum.
pub fn visits(f: &Fixture, v: Variant, count: usize, result: &str) -> String {
    let arms = (1..=count)
        .map(|i| format!("{i},{}", step(f, v, i, result)))
        .collect::<Vec<_>>();
    format!(
        "Switch[Global`state+1,{},_,(AppendTo[Global`log,999];Print[999];Global`state++;{result})]",
        arms.join(",")
    )
}

pub fn finish(mut f: Fixture, v: Variant, expected: &str) -> Fixture {
    if v == Variant::Error {
        f.expected_error = Some(SessionRequestError::Other {
            message: "uncaught throw: \"predicate boom\" with tag \"predicate tag\"".into(),
        });
    } else {
        f.expected_value = Some(expected.into());
    }
    if v == Variant::AlreadyResumed {
        f.source = format!("InputString[\"predicate caller\"];{}", f.source);
        f.pauses.insert(
            0,
            Pause {
                prompt: "predicate caller".into(),
                kind: RuntimeInputKind::InputString,
                answer: "caller resumed".into(),
            },
        );
    }
    if v == Variant::Nested {
        f.source = format!("Block[{{Global`scope=23}},{}]", f.source);
    }
    f
}
