use super::super::super::{
    descriptors::{Checks, Variant, accept},
    support::observe::Rejection,
};

#[test]
fn every_role_rejects_missing_pause_and_lost_log() {
    for d in super::descriptors() {
        let good = (d.factory)(d.entry, Variant::Middle);
        let mut missing = good.clone();
        for pause in &good.pauses {
            let kind = if pause.kind == rum_runtime::RuntimeInputKind::Input {
                "Input"
            } else {
                "InputString"
            };
            let input = format!("{kind}[\"{}\"]", pause.prompt);
            let answer = if kind == "Input" {
                pause.answer.clone()
            } else {
                format!("{:?}", pause.answer)
            };
            missing.source = missing.source.replace(&input, &answer);
            missing.setup = missing.setup.replace(&input, &answer);
            for p in &mut missing.pauses {
                p.answer = p.answer.replace(&input, &answer);
            }
            if let Some(files) = &mut missing.files {
                for (_, source) in files {
                    *source = source.replace(&input, &answer);
                }
            }
        }
        assert_eq!(
            accept(&missing, Checks::ALL).unwrap_err(),
            Rejection::PauseCount {
                expected: good.pauses.len(),
                actual: 0
            },
            "{} missing pause",
            d.id
        );
        // Removing Input legitimately changes detailed trace events, so the
        // deliberately weak control also disables literal value expectations.
        accept(
            &missing,
            Checks {
                transcript: false,
                queue: false,
                expectations: false,
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
            "{} lost log",
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
            "DEBUG_FALSIFIERS {} missing-pause=REJECT lost-log=REJECT weak-controls=PASS good=PASS",
            d.id
        );
    }
}

#[test]
fn duration_normalization_preserves_value_and_rejects_non_numeric_leaves() {
    let f = super::bodies::fixture("Timing.body", Variant::Middle);
    let mut wrong = f.clone();
    wrong.expected_value = Some("List[0, 91]".into());
    assert_eq!(
        accept(&wrong, Checks::ALL).unwrap_err(),
        Rejection::ExpectedValue
    );
    assert!(
        std::panic::catch_unwind(|| super::super::super::support::normalize::duration(
            &rum_core::Expr::string("0"),
            &[]
        ))
        .is_err()
    );
    accept(&f, Checks::ALL).unwrap();
}
