//! Q7 predicate-class falsifiers. These fixtures cannot mint mutation receipts.
use super::{
    records::{Records, Rejection, verify_own_omission},
    schema,
};
use crate::oracle::fixtures::obligations::guest;

#[test]
fn cm_mutant_observer_class_controls() {
    let specs = schema::specs();
    assert_eq!(
        specs.values().filter(|s| s.observer == "Strict").count(),
        97
    );
    let spec = &specs["P0813"];
    let cases = schema::cases();
    let case = cases
        .iter()
        .find(|c| c.kind == "own" && c.row == spec.row)
        .unwrap();
    let programs = guest::programs().unwrap();
    let program = &programs[case.program];
    assert_eq!(program.key.line(), case.tuple);
    let baseline = guest::run(program, false).unwrap();
    let queued = guest::run(program, true).unwrap();
    guest::validate(program, &baseline, &queued).unwrap();
    let mut differing = baseline.clone();
    let crate::HostEvalOutcome::Success(out) = &mut differing.output else {
        panic!("frozen Strict cell is a real success")
    };
    out.value.text = "14".into();
    out.value.typed = crate::HostTyped::Integer(14);
    for item in &mut out.outputs {
        if item.kind == crate::HostStructuredOutputKind::Result {
            item.text = "14".into();
        }
    }
    let predicate = guest::validate(program, &differing, &queued);
    assert_eq!(predicate, Err(guest::RunFailure::Output));
    let hex = |s: &str| s.bytes().map(|b| format!("{b:02x}")).collect::<String>();
    let packet = |arm: &str, observation: &guest::RunObservation| {
        let armed = arm == "armed";
        let predicate = guest::validate(program, observation, &queued);
        let raw = format!(
            "CM740_RUN_ID row={} source_key={} tail_digest={} schema={} arm={arm} binary=synthetic-{arm} selector={} profile=on\nCM740_CASE\t{}\t{}\nCM740_OBSERVATION\t{}\t{}\t{}\t{predicate:?}\t{}\n{}test result: ok. 1 passed; 0 failed;\n",
            spec.row,
            spec.source_key,
            spec.tail_digest,
            spec.source_schema,
            spec.selector,
            hex(&case.tuple),
            hex(case.program),
            hex(&case.tuple),
            hex(&format!("{observation:?}")),
            hex(&format!("{queued:?}")),
            baseline.pauses.len(),
            if armed {
                format!(
                    "CM740_SELECTOR_ARMED row={}\nCM740_SOURCE_HIT row={} phase={} selector={}\nCM740_OWN_PREDICATE row={} predicate={predicate:?} public_output_exact=false native_difference=true\n",
                    spec.row, spec.row, spec.phases[0], spec.selector, spec.row
                )
            } else {
                String::new()
            },
        );
        Records::parse(&raw).unwrap()
    };
    let real = packet("real", &baseline);
    let disabled = packet("disabled", &baseline);
    let armed = packet("armed", &differing);
    let required = [case.tuple.clone()].into();
    let verify =
        |s, a: &Records| verify_own_omission(s, &required, &case.tuple, &real, &disabled, a);
    assert!(verify(spec, &armed).is_ok());
    let mut native_spec = spec.clone();
    native_spec.observer = "NativeEqual";
    let mut native = armed.clone();
    native.observations[0].3 = "Err(Native)".into();
    native.predicates[0].1 = "Err(Native)".into();
    assert_eq!(
        verify(&native_spec, &native).err(),
        Some(Rejection::PublicOutput)
    );
    let mut earlier = armed.clone();
    earlier.observations[0].3 = "Err(Phase)".into();
    earlier.predicates[0].1 = "Err(Phase)".into();
    assert_eq!(verify(spec, &earlier).err(), Some(Rejection::EarlierGuard));
    let mut survivor = armed.clone();
    survivor.observations[0] = real.observations[0].clone();
    assert_eq!(
        verify(spec, &survivor).err(),
        Some(Rejection::IdentitySurvivor)
    );
    let mut wrong = armed.clone();
    wrong.cases[0].push('!');
    assert_eq!(
        verify(spec, &wrong).err(),
        Some(Rejection::MissingRealTuple)
    );
    println!(
        "CM4 OBSERVER CLASS CONTROLS PASS Strict=97 nonStrict=54 strict_changed_output=own_Output nonStrict_changed_output=PublicOutput out_of_class=EarlierGuard identity=IdentitySurvivor wrong_tuple=MissingRealTuple synthetic=true mutation_credit=false"
    );
}
