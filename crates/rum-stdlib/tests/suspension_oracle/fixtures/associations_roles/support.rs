use super::super::super::descriptors::{Fixture, Pause, Variant};
use rum_runtime::{RuntimeInputKind, SessionRequestError};

pub const VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
];
pub const GUARD_VARIANTS: &[Variant] = &[
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
pub fn project(f: &mut Fixture, source: &str, expected: &str) {
    f.projections.push(source.into());
    f.expected_projections.push(expected.into());
}
pub fn base(entry: &str, v: Variant) -> Fixture {
    let mut f = Fixture::new(entry, "");
    f.setup =
        "Global`state=0;Global`answers=0;Global`scope=17;Global`seen=0;Global`suffix=0".into();
    f.expected_log = Some(list(5));
    f.expected_prints = Some((1..=5).map(|i| i.to_string()).collect());
    f.expected_messages = Some(vec![]);
    project(&mut f, "Global`state", "5");
    project(&mut f, "Global`answers", if twice(v) { "6" } else { "2" });
    project(&mut f, "Global`scope", "17");
    project(
        &mut f,
        "Global`seen",
        if v == Variant::Nested { "23" } else { "17" },
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

// Executed in the role's delayed option value, callback, guard or parsed body.
// Effects before input distinguish a replay from a continued visit.
pub fn step(f: &Fixture, v: Variant, i: usize, result: &str) -> String {
    let input = if i == 2 || (i == 4 && twice(v)) {
        format!("Global`answers+=Input[\"{}:{i}\"];", f.id)
    } else {
        String::new()
    };
    let result = if i == 5 && v == Variant::Error {
        "Throw[\"associations boom\",\"associations tag\"]"
    } else {
        result
    };
    format!(
        "(AppendTo[Global`log,{i}];Print[{i}];Global`state={i};Global`seen=Global`scope;{input}{result})"
    )
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
// One arm for each actual invocation of the builtin's match/string loop.
pub fn visits(f: &Fixture, v: Variant, result: &str) -> String {
    let arms = (1..=5)
        .map(|i| format!("{i},{}", step(f, v, i, result)))
        .collect::<Vec<_>>();
    format!(
        "Switch[Global`state+1,{},_,(AppendTo[Global`log,999];Print[999];Global`state++;{result})]",
        arms.join(",")
    )
}
pub fn finish(mut f: Fixture, v: Variant, expected: &str) -> Fixture {
    if v == Variant::Error {
        f.expected_messages = Some(vec![rum_runtime::RuntimeMessage {
            symbol: "Throw".into(),
            tag: "nocatch".into(),
            text:
                "Uncaught Throw[\"associations boom\", \"associations tag\"] returned to top level."
                    .into(),
        }]);
        f.expected_error = Some(SessionRequestError::Other {
            message: "uncaught throw: \"associations boom\" with tag \"associations tag\"".into(),
        });
    } else {
        f.expected_value = Some(expected.into());
    }
    if v == Variant::AlreadyResumed {
        f.source = format!("InputString[\"associations caller\"];{}", f.source);
        f.pauses.insert(
            0,
            Pause {
                prompt: "associations caller".into(),
                kind: RuntimeInputKind::InputString,
                answer: "caller resumed".into(),
            },
        );
        f.lifecycle_pauses = [1, 2];
    }
    if v == Variant::Nested {
        f.source = format!("Block[{{Global`scope=23}},{}]", f.source);
    }
    f.pause_prints = Some(
        f.pauses
            .iter()
            .map(|p| {
                let n = p
                    .prompt
                    .rsplit(':')
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                (1..=n).map(|i| i.to_string()).collect()
            })
            .collect(),
    );
    f.pause_messages = Some(vec![vec![]; f.pauses.len()]);
    f.pause_trace = Some(vec![vec![]; f.pauses.len()]);
    f
}

pub fn association(items: impl IntoIterator<Item = (usize, String)>) -> String {
    format!(
        "Association[{}]",
        items
            .into_iter()
            .map(|(k, v)| format!("Rule[{k}, {v}]"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
pub fn data() -> String {
    association((1..=5).map(|i| (i, (i * 10).to_string())))
}
pub fn delayed(f: &Fixture, v: Variant, result: impl Fn(usize) -> String) -> String {
    format!(
        "Association[{}]",
        (1..=5)
            .map(|i| format!("RuleDelayed[{i},{}]", step(f, v, i, &result(i))))
            .collect::<Vec<_>>()
            .join(",")
    )
}
