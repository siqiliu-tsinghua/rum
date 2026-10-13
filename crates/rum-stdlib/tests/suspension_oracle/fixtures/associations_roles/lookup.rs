use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;
pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let data = data();
    let (source, expected) = match entry {
        "Lookup.heldTarget" => (
            format!(
                "Lookup[{},{{1,2,3,4,5,6}},99,Function[x,10+x]]",
                sequence(&f, v, &data)
            ),
            "List[20, 30, 40, 50, 60, 99]",
        ),
        "Lookup.key" => (
            format!(
                "Lookup[{data},{},99,Function[x,10+x]]",
                sequence(&f, v, "{1,2,3,4,5,6}")
            ),
            "List[20, 30, 40, 50, 60, 99]",
        ),
        "Lookup.default" => (
            format!(
                "Lookup[{data},{{1,6,2,7,3,8}},{},Function[x,10+x]]",
                sequence(&f, v, "99")
            ),
            "List[20, 99, 30, 99, 40, 99]",
        ),
        "Lookup.wrapper" => (
            format!(
                "Lookup[{data},{{1,6,2,7,3,8}},99,{}]",
                sequence(&f, v, "Function[x,10+x]")
            ),
            "List[20, 99, 30, 99, 40, 99]",
        ),
        _ => unreachable!("{entry}"),
    };
    f.source = source;
    finish(f, v, expected)
}

// The wrapper has a separate owner when entered after a default suspension.
pub fn wrapper_after_default(entry: &str, v: Variant) -> Fixture {
    use super::super::super::descriptors::Pause;
    let mut f = fixture(entry, v);
    f.source = f
        .source
        .replacen(",99,", ",(InputString[\"lookup default\"];99),", 1);
    let index = usize::from(v == Variant::AlreadyResumed);
    f.pauses.insert(
        index,
        Pause {
            prompt: "lookup default".into(),
            kind: rum_runtime::RuntimeInputKind::InputString,
            answer: "default resumed".into(),
        },
    );
    for pause in &mut f.lifecycle_pauses {
        *pause += 1;
    }
    f.pause_prints.as_mut().unwrap().insert(index, vec![]);
    f.pause_messages.as_mut().unwrap().insert(index, vec![]);
    f.pause_trace.as_mut().unwrap().insert(index, vec![]);
    f
}
