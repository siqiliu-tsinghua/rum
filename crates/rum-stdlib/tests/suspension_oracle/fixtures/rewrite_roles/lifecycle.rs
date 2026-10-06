use super::super::super::{descriptors::Fixture, support::lifecycle};

pub fn exercise(f: &Fixture) -> usize {
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
    lifecycle::exercise_observed(
        || f.session(),
        &f.source,
        &plans,
        |s, stop, reset| {
            let (log, state) = if reset {
                ("List[]", "0")
            } else if stop == 0 {
                ("List[1, 2]", "2")
            } else {
                ("List[1, 2, 3, 4]", "4")
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
        },
    )
}
