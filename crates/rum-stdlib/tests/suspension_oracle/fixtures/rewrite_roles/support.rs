use super::super::super::descriptors::{Fixture, Pause, Variant};
use rum_runtime::{RuntimeInputKind, SessionRequestError};

pub const VARIANTS: &[Variant] = &[
    Variant::Middle,
    Variant::Twice,
    Variant::Error,
    Variant::AlreadyResumed,
    Variant::Nested,
];
pub const MATCH_VARIANTS: &[Variant] = &[
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

pub fn base(entry: &str, v: Variant) -> Fixture {
    let mut f = Fixture::new(entry, "");
    f.setup = "Global`state=0;Global`scope=17".into();
    f.expected_log = Some("List[1, 2, 3, 4, 5]".into());
    f.expected_prints = Some((1..=5).map(|i| i.to_string()).collect());
    f.projections = vec!["Global`state".into(), "Global`scope".into()];
    f.expected_projections = vec!["5".into(), "17".into()];
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

// Place each step inside its owned role: pattern argument, option, held input,
// matcher callback or delayed rule body.
pub fn step(f: &mut Fixture, v: Variant, i: usize, value: &str) -> String {
    let tail = if i == 5 && v == Variant::Error {
        r#"Throw["rewrite boom","rewrite tag"]"#.into()
    } else if i == 2 || (i == 4 && twice(v)) {
        let prompt = format!("{}:{i}", f.id);
        f.pauses
            .iter_mut()
            .find(|p| p.prompt == prompt)
            .unwrap()
            .answer = value.into();
        format!("Input[\"{prompt}\"]")
    } else {
        value.into()
    };
    format!("(AppendTo[Global`log,{i}];Print[{i}];Global`state={i};{tail})")
}

pub fn sequence(f: &mut Fixture, v: Variant, result: &str) -> String {
    let steps = (1..=5)
        .map(|i| step(f, v, i, &i.to_string()))
        .collect::<Vec<_>>();
    format!("({};{result})", steps.join(";"))
}

// Five traversal callbacks, with suspension in callbacks two and four. The
// predicate result is independent of the response, while the exact response
// plan is checked by the driver. Decline still visits all five candidates.
pub fn callback(f: &Fixture, v: Variant, result: &str) -> String {
    format!(
        "(AppendTo[Global`log,x];Print[x];Global`state=x;If[x==2,Input[\"{}:2\"]];{}{}{})",
        f.id,
        if twice(v) {
            format!("If[x==4,Input[\"{}:4\"]];", f.id)
        } else {
            String::new()
        },
        if v == Variant::Error {
            r#"If[x==5,Throw["rewrite boom","rewrite tag"]];"#
        } else {
            ""
        },
        result,
    )
}

pub fn finish(mut f: Fixture, v: Variant, value: &str) -> Fixture {
    if v == Variant::Error {
        f.expected_error = Some(SessionRequestError::Other {
            message: "uncaught throw: \"rewrite boom\" with tag \"rewrite tag\"".into(),
        });
    } else {
        f.expected_value = Some(value.into());
    }
    if v == Variant::AlreadyResumed {
        f.source = format!("InputString[\"rewrite caller\"];{}", f.source);
        f.pauses.insert(
            0,
            Pause {
                prompt: "rewrite caller".into(),
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
