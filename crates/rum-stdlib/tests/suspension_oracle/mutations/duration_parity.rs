//! Disabled-arm comparison for rows with declared timing leaves. Raw records stay exact.
use crate::oracle::fixtures::obligations::guest::{self, Program, RunFailure, RunObservation};

pub(super) fn applies(row: &str) -> bool {
    matches!(row, "P0492" | "P0515")
}

// The loader verifies the fixture fingerprint and declared_duration before exposing a program.
pub(super) fn declared_p0515_leaf(key: &str, id: &str) -> bool {
    guest::programs().is_ok_and(|programs| {
        programs.get(id).is_some_and(|p| {
            p.key.row() == "P0515"
                && p.key.line() == key
                && p.canonical
                    .as_ref()
                    .is_some_and(|f| !f.0.duration_paths.is_empty())
        })
    })
}

pub(super) fn observation_record(
    program: &Program,
    resumed: &RunObservation,
    queued: &RunObservation,
    verdict: Result<(), RunFailure>,
    pauses: usize,
) -> Result<String, RunFailure> {
    let comparison = |raw: &RunObservation| {
        let mut view = raw.clone();
        view.output = guest::mutation_disabled_duration_comparison(program, &raw.output)?;
        Ok::<_, RunFailure>(view)
    };
    let hex = |s: String| s.bytes().map(|b| format!("{b:02x}")).collect::<String>();
    Ok(format!(
        "CM740_OBSERVATION\t{}\t{}\t{}\t{verdict:?}\t{pauses}",
        hex(program.key.line()),
        hex(format!("{:?}", comparison(resumed)?)),
        hex(format!("{:?}", comparison(queued)?)),
    ))
}

#[test]
fn cm_mutant_declared_duration_controls() {
    let programs = guest::programs().unwrap();
    let p = programs
        .values()
        .find(|p| {
            p.key.row() == "P0492"
                && p.canonical
                    .as_ref()
                    .is_some_and(|f| !f.0.duration_paths.is_empty())
        })
        .unwrap();
    let mut first = guest::run(p, false).unwrap();
    let queued = guest::run(p, true).unwrap();
    guest::validate(p, &first, &queued).unwrap();
    // Synthetic validator inputs, never a mutation packet or a real matrix.
    let mut session = guest::session(p);
    first.output = session.eval("{1/100, 7}");
    let raw = first.clone();
    let mut second = first.clone();
    second.output = session.eval("{2/100, 7}");
    let encode = |program: &Program, record: &RunObservation| {
        observation_record(program, record, &queued, Ok(()), record.pauses.len())
    };
    assert_ne!(first, second);
    assert_eq!(encode(p, &first).unwrap(), encode(p, &second).unwrap());
    assert_eq!(first, raw, "comparison must preserve raw duration values");
    second.output = session.eval("{2/100, 8}");
    assert_ne!(encode(p, &first).unwrap(), encode(p, &second).unwrap());
    second = first.clone();
    second
        .counters
        .push(("non-duration-control".into(), [1; 10]));
    assert_ne!(encode(p, &first).unwrap(), encode(p, &second).unwrap());
    let mut undeclared = p.clone();
    undeclared.canonical.as_mut().unwrap().0.duration_paths = vec![vec![1]];
    assert_eq!(encode(&undeclared, &first), Err(RunFailure::Program));
    undeclared = p.clone();
    undeclared.key.0[0] = "P0256".into();
    assert_eq!(encode(&undeclared, &first), Err(RunFailure::Program));
    println!(
        "CM4 DURATION PARITY CONTROLS PASS row=P0492 declared_leaf=0 sibling_difference=rejected native_difference=rejected undeclared_path=Program wrong_row=Program raw_unchanged=true mutation_credit=false"
    );
}

#[test]
fn cm_mutant_p0515_fourth_difference_is_disabled_difference() {
    use super::records::{Records, Rejection, verify_own_omission};
    use std::collections::BTreeSet;

    let specs = super::schema::specs();
    let spec = &specs["P0515"];
    let programs = guest::programs().unwrap();
    let row: Vec<_> = programs
        .values()
        .filter(|p| p.key.row() == spec.row)
        .collect();
    let selected = super::schema::cases()
        .into_iter()
        .find(|case| case.kind == "own" && case.row == spec.row)
        .unwrap()
        .tuple;
    let required: BTreeSet<_> = row.iter().map(|p| p.key.line()).collect();
    let mut first = guest::run(row[0], false).unwrap();
    let mut session = crate::HostSession::builder().build();
    first.output = session.eval("{1/100, 73}");
    let mut second = first.clone();
    second.output = session.eval("{2/100, 73}");
    let record = |arm: &str, fourth: bool| {
        let mut text = format!(
            "CM740_RUN_ID row=P0515 source_key={} tail_digest={} schema={} arm={arm} binary=synthetic-{arm} selector=P0515 profile=on\n",
            spec.source_key, spec.tail_digest, spec.source_schema
        );
        let mut declared = 0;
        let mut changed_extra = false;
        let hex = |s: &str| s.bytes().map(|b| format!("{b:02x}")).collect::<String>();
        for p in &row {
            if arm == "armed" && p.key.line() != selected {
                continue;
            }
            let timing = p
                .canonical
                .as_ref()
                .is_some_and(|f| !f.0.duration_paths.is_empty());
            declared += usize::from(timing);
            let extra = fourth && !timing && !changed_extra;
            changed_extra |= extra;
            let o = if arm == "disabled" && (timing || extra) {
                &second
            } else {
                &first
            };
            let key = hex(&p.key.line());
            let obs = hex(&format!("{o:?}"));
            let raw = format!("CM740_OBSERVATION\t{key}\t{obs}\t{obs}\tOk(())\t1");
            text.push_str(&format!(
                "CM740_CASE\t{key}\t{id}\nCM740_CONSTITUENT\t{key}\t{id}\t0\n{raw}\n",
                id = hex(&p.id)
            ));
            if arm != "armed" {
                let comparison = observation_record(p, o, o, Ok(()), 1).unwrap();
                assert_eq!(raw != comparison, timing);
                text.push_str(&format!(
                    "CM740_DURATION_PARITY\t{}\t{}\n",
                    crate::oracle::obligations::content_digest(raw.as_bytes()),
                    hex(&comparison)
                ));
            }
        }
        if arm != "armed" {
            assert_eq!(declared, 3);
        }
        assert_eq!(fourth, changed_extra);
        text.push_str("test result: ok. 1 passed; 0 failed;\n");
        Records::parse(&text).unwrap()
    };
    let real = record("real", false);
    let armed = record("armed", false);
    let validate = |fourth| {
        verify_own_omission(
            spec,
            &required,
            &selected,
            &real,
            &record("disabled", fourth),
            &armed,
        )
        .err()
    };
    // The three declared leaves clear parity. The synthetic armed control has no selector arm.
    assert_eq!(validate(false), Some(Rejection::Unarmed));
    assert_eq!(validate(true), Some(Rejection::DisabledDifference));
    println!(
        "CM4 P0515 DURATION FALSIFIER PASS declared=3 fourth=DisabledDifference raw_unchanged=true mutation_credit=false"
    );
}
