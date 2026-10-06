use super::super::super::{
    descriptors::{Fixture, Variant},
    support::lifecycle,
};

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
    // Error fixtures state explicitly which stores remain uncommitted. Except
    // for destructuring, cancellation at either pause has the same store state.
    let before_store = super::fixture(&f.id, Variant::Error);
    lifecycle::exercise_observed(
        || f.session(),
        &f.source,
        &plans,
        |s, stop, reset| {
            let log = if reset {
                "List[]"
            } else if stop == 0 {
                "List[1, 2]"
            } else {
                "List[1, 2, 3, 4]"
            };
            assert_eq!(
                s.eval_request("Global`log").unwrap().value.to_string(),
                log,
                "{} lifecycle log",
                f.id
            );
            for (i, source) in f.projections.iter().enumerate() {
                let expected = if i == 0 {
                    if reset {
                        "0"
                    } else if stop == 0 {
                        "2"
                    } else {
                        "4"
                    }
                } else if f.id == "Set.destructuring" && i == 2 {
                    if reset {
                        "List[0, 0, 0, 0, 0]"
                    } else if stop == 0 {
                        "List[11, 0, 0, 0, 0]"
                    } else {
                        "List[11, 12, 13, 0, 0]"
                    }
                } else {
                    &before_store.expected_projections[i]
                };
                assert_eq!(
                    s.eval_request(source).unwrap().value.to_string(),
                    expected,
                    "{} lifecycle projection {source}",
                    f.id
                );
            }
        },
    )
}
