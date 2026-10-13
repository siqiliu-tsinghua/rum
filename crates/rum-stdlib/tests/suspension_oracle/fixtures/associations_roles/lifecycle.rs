use super::super::super::descriptors::{Fixture, exercise_fixture};
use super::support::list;

pub fn exercise(f: &Fixture) -> usize {
    exercise_fixture(f, |s, stop, reset| {
        let n = if reset {
            0
        } else {
            f.pauses[f.lifecycle_pauses[stop]]
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
            (
                "Global`answers",
                if reset || stop == 0 { "0" } else { "2" }.into(),
            ),
            ("Global`scope", "17".into()),
            ("Global`seen", if reset { "0" } else { "17" }.into()),
            ("Global`suffix", "0".into()),
        ] {
            assert_eq!(
                s.eval_request(source).unwrap().value.to_string(),
                expected,
                "{} lifecycle {source}",
                f.id
            );
        }
        if f.id.ends_with(".target") {
            assert_eq!(
                s.eval_request("Global`unchanged[]")
                    .unwrap()
                    .value
                    .to_string(),
                "True",
                "{} target changed before completion",
                f.id
            );
        }
        assert_eq!(
            s.eval_request("ValueQ[Global`oracleOverlap]")
                .unwrap()
                .value
                .to_string(),
            "False"
        );
    })
}
