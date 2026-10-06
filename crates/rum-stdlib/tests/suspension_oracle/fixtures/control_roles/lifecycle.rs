use super::super::super::{descriptors::Fixture, support::lifecycle};
use rum_runtime::Session;

fn check(s: &mut Session, f: &Fixture, source: &str, expected: &str) {
    assert_eq!(
        s.eval_request(source).unwrap().value.to_string(),
        expected,
        "{} lifecycle {source}",
        f.id
    );
}

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
            } else if stop == 0 {
                2
            } else {
                4
            };
            check(
                s,
                f,
                "Global`log",
                if reset {
                    "List[]"
                } else if stop == 0 {
                    "List[1, 2]"
                } else {
                    "List[1, 2, 3, 4]"
                },
            );
            check(s, f, "Global`state", &n.to_string());
            check(s, f, "Global`scope", "17");
            check(
                s,
                f,
                "Global`cleanupCount",
                if !reset && f.id == "WithCleanup.body" {
                    "1"
                } else {
                    "0"
                },
            );
            check(
                s,
                f,
                "Global`selected",
                if !reset && f.id == "WithCleanup.cleanup" {
                    "1"
                } else {
                    "0"
                },
            );
            let (name, role) = f.id.split_once('.').unwrap();
            if matches!(name, "Do" | "Table" | "Sum" | "Product") {
                check(s, f, "{Global`i,Global`j}", "List[91, 92]");
                let count = if reset {
                    0
                } else if role == "bounds" {
                    n - 1
                } else {
                    n
                };
                check(s, f, "Global`count", &count.to_string());
            }
            if matches!(name, "For" | "While" | "Until") {
                let index = if reset || role == "initialization" {
                    0
                } else {
                    n
                };
                let count = if reset || role == "initialization" {
                    0
                } else if role == "test" && name != "Until" {
                    n - 1
                } else {
                    n
                };
                check(s, f, "Global`n", &index.to_string());
                check(s, f, "Global`count", &count.to_string());
            }
            if matches!(f.id.as_str(), "AbortProtect.body" | "CheckAbort.body") {
                check(
                    s,
                    f,
                    "(Global`abortSuffix=0;CheckAbort[(Abort[];Global`abortSuffix=1),73];Global`abortSuffix)",
                    "0",
                );
            }
            if name == "BlockRandom" {
                check(
                    s,
                    f,
                    "Table[RandomInteger[1000000],{5}]==Global`expectedRandom",
                    "True",
                );
            }
        },
    )
}
