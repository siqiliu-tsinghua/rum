use super::super::super::{
    descriptors::{Checks, Variant, accept},
    support::observe::Rejection,
};

#[test]
fn every_role_rejects_missing_pause_and_lost_log() {
    for d in super::descriptors() {
        let good = (d.factory)(d.entry, Variant::Middle);
        let mut no_pause = good.clone();
        for p in &good.pauses {
            let input = format!("Input[\"{}\"]", p.prompt);
            assert!(
                no_pause.source.contains(&input) || no_pause.setup.contains(&input),
                "{}",
                d.entry
            );
            no_pause.source = no_pause.source.replace(&input, &p.answer);
            no_pause.setup = no_pause.setup.replace(&input, &p.answer);
        }
        let rejected = accept(&no_pause, Checks::ALL).unwrap_err();
        assert_eq!(
            rejected,
            Rejection::PauseCount {
                expected: 1,
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
            "PREDICATE_FALSIFIERS {} missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS",
            d.id
        );
    }
}
