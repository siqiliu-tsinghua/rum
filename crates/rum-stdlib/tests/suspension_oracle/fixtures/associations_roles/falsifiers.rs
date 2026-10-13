use super::super::super::{
    descriptors::{Checks, Variant, accept},
    support::observe::Rejection,
};

#[test]
fn every_owner_rejects_missing_pause_and_lost_log() {
    for d in super::descriptors() {
        let good = (d.factory)(d.entry, Variant::Middle);
        let mut no_pause = good.clone();
        for p in &good.pauses {
            let head = if p.kind == rum_runtime::RuntimeInputKind::Input {
                "Input"
            } else {
                "InputString"
            };
            let input = format!("{head}[\"{}\"]", p.prompt);
            let encoded = format!("{head}[\\\"{}\\\"]", p.prompt);
            assert!(
                no_pause.source.contains(&input)
                    || no_pause.setup.contains(&input)
                    || no_pause.source.contains(&encoded),
                "{}",
                d.id
            );
            let answer = if head == "InputString" {
                format!("\"{}\"", p.answer)
            } else {
                p.answer.clone()
            };
            no_pause.source = no_pause
                .source
                .replace(&input, &answer)
                .replace(&encoded, &answer);
            no_pause.setup = no_pause.setup.replace(&input, &answer);
        }
        assert_eq!(
            accept(&no_pause, Checks::ALL).unwrap_err(),
            Rejection::PauseCount {
                expected: good.pauses.len(),
                actual: 0
            },
            "{}",
            d.id
        );
        accept(
            &no_pause,
            Checks {
                transcript: false,
                queue: false,
                ..Checks::ALL
            },
        )
        .unwrap();
        let mut lost = good.clone();
        lost.source = format!(
            "Global`oracleResult=({});Global`log={{}};Global`oracleResult",
            good.source
        );
        assert_eq!(
            accept(&lost, Checks::ALL).unwrap_err(),
            Rejection::ExpectedLog,
            "{}",
            d.id
        );
        accept(
            &lost,
            Checks {
                expectations: false,
                ..Checks::ALL
            },
        )
        .unwrap();
        accept(&good, Checks::ALL).unwrap();
        println!(
            "ASSOCIATION_SCHEMA_FALSIFIERS {} missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS",
            d.id
        );
    }
}
