use super::super::super::{descriptors::Fixture, support::lifecycle};
use super::support::list;

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
            let n = if reset {
                0
            } else {
                f.pauses[stop]
                    .prompt
                    .rsplit(':')
                    .next()
                    .unwrap()
                    .parse::<usize>()
                    .unwrap()
            };
            for (source, expected) in [
                ("Global`log", list(n)),
                ("Global`state", n.to_string()),
                ("Global`scope", "17".into()),
                (
                    "Global`answers",
                    if reset || stop == 0 {
                        "0".into()
                    } else {
                        f.pauses[0].answer.clone()
                    },
                ),
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
