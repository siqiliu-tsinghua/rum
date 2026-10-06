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
    base_at(entry, v, count, [2, 4])
}

pub fn base_at(entry: &str, v: Variant, count: usize, positions: [usize; 2]) -> Fixture {
    let mut f = Fixture::new(entry, "");
    f.setup =
        "Global`state=0;Global`scope=17;Global`answers=0;Global`selected=0;Global`calls=0;Global`before=0;Global`seen={}".into();
    f.expected_log = Some(list(count));
    f.expected_prints = Some((1..=count).map(|i| i.to_string()).collect());
    project(&mut f, "Global`state", &count.to_string());
    project(&mut f, "Global`scope", "17");
    project(
        &mut f,
        "Global`answers",
        &(positions[0] + if twice(v) { positions[1] } else { 0 }).to_string(),
    );
    for i in positions {
        if i == positions[0] || twice(v) {
            f.pauses.push(Pause {
                prompt: format!("{entry}:{i}"),
                kind: RuntimeInputKind::Input,
                answer: i.to_string(),
            });
        }
    }
    f
}

pub fn step(f: &Fixture, v: Variant, i: usize, result: &str, last: bool) -> String {
    let input = if f.pauses.iter().any(|p| p.prompt == format!("{}:{i}", f.id)) {
        format!("Global`answers+=Input[\"{}:{i}\"];", f.id)
    } else {
        String::new()
    };
    let tail = if last && v == Variant::Error {
        "Throw[\"functional boom\",\"functional tag\"]"
    } else {
        result
    };
    format!("(AppendTo[Global`log,{i}];Print[{i}];Global`state={i};{input}{tail})")
}

pub fn sequence(f: &Fixture, v: Variant, result: &str) -> String {
    format!(
        "({})",
        (1..=5)
            .map(|i| step(f, v, i, result, i == 5))
            .collect::<Vec<_>>()
            .join(";")
    )
}

// Each arm is a separate visit by the builtin's loop. The state is updated
// before input; a replay or skipped visit changes the log and answer sum.
pub fn visits(f: &Fixture, v: Variant, count: usize, result: &str) -> String {
    let arms = (1..=count)
        .map(|i| format!("{i},{}", step(f, v, i, result, i == count)))
        .collect::<Vec<_>>();
    format!(
        "Switch[Global`state+1,{},_,(AppendTo[Global`log,999];Print[999];Global`state++;{result})]",
        arms.join(",")
    )
}

pub fn finish(mut f: Fixture, v: Variant, expected: &str) -> Fixture {
    if v == Variant::Error {
        f.expected_error = Some(SessionRequestError::Other {
            message: "uncaught throw: \"functional boom\" with tag \"functional tag\"".into(),
        });
    } else {
        f.expected_value = Some(expected.into());
    }
    if v == Variant::AlreadyResumed {
        f.source = format!("InputString[\"functional caller\"];{}", f.source);
        f.pauses.insert(
            0,
            Pause {
                prompt: "functional caller".into(),
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
