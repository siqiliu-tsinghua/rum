//! Synthetic validator fixtures; these never constitute source-omission evidence.
use super::{
    records::{Records, Rejection, verify_own_omission},
    schema,
};
use std::collections::BTreeSet;

#[test]
fn cm_mutant_profile_group_controls() {
    fn packet(off: bool, success: bool) -> String {
        let key = format!(
            "P0248\tMiddle\tvalidator-only\troute{}\tdescriptor",
            if off { ";PROFILED=false" } else { "" }
        );
        let hex: String = key.bytes().map(|b| format!("{b:02x}")).collect();
        format!(
            "CM740_RUN_ID row=P0248 source_key=synthetic tail_digest=synthetic schema=synthetic arm=real binary=synthetic selector=P0248 profile={}\nCM740_CASE\t{hex}\t7072696d617279\ntest result: {}\n",
            if off { "off" } else { "on" },
            if success {
                "ok. 1 passed; 0 failed;"
            } else {
                "FAILED. 0 passed; 1 failed;"
            }
        )
    }
    let on = packet(false, true);
    let off = packet(true, true);
    let complete = Records::parse(&format!("{on}{off}")).unwrap();
    assert_eq!(complete.cases.len(), 2);
    assert!(complete.success);
    assert_eq!(
        Records::parse(&format!("{on}{on}")).err(),
        Some(Rejection::RawArtifact)
    );
    assert_eq!(
        Records::parse(&on.replace("profile=on", "profile=off")).err(),
        Some(Rejection::RawArtifact)
    );
    assert!(
        !Records::parse(&format!("{on}{}", packet(true, false)))
            .unwrap()
            .success
    );
    println!(
        "CM4 MUTANT PROFILE GROUP CONTROLS PASS duplicate_group=RawArtifact wrong_profile_route=RawArtifact failed_group=not_executed synthetic=true"
    );
}

#[test]
fn cm_mutant_negative_controls() {
    let specs = schema::specs();
    let spec = &specs[super::source_auth::observer::ROW];
    let key = format!(
        "{}\tMiddle\tvalidator-only\tactual-route\tdescriptor",
        spec.row
    );
    let required: BTreeSet<_> = [key.clone()].into();
    let real = Records {
        row: spec.row.into(),
        source_key: spec.source_key.into(),
        tail_digest: spec.tail_digest.into(),
        source_schema: spec.source_schema.into(),
        arm: "real".into(),
        binary: "synthetic-real-build".into(),
        selector: spec.row.into(),
        success: true,
        cases: vec![key.clone()],
        root_programs: vec![(key.clone(), "synthetic-primary".into())],
        constituents: vec![(key.clone(), "synthetic-primary".into(), 0)],
        exact: vec!["synthetic complete record".into()],
        disabled_parity: vec!["synthetic complete record".into()],
        observations: vec![(
            key.clone(),
            "actual resumed record".into(),
            "actual queued record".into(),
            "Ok(())".into(),
            1,
        )],
        hits: Vec::new(),
        armed: BTreeSet::new(),
        hit_before_arm: false,
        predicates: Vec::new(),
    };
    let mut disabled = real.clone();
    disabled.arm = "disabled".into();
    disabled.binary = "synthetic-disabled-build".into();
    let mut armed = real.clone();
    armed.arm = "armed".into();
    armed.binary = "synthetic-armed-build".into();
    armed.observations[0].1 = "validator difference".into();
    armed.observations[0].3 = "Err(Native)".into();
    armed.armed.insert(spec.row.into());
    armed
        .hits
        .push((spec.row.into(), spec.phases[0].into(), spec.selector.into()));
    armed
        .predicates
        .push((spec.row.into(), "Err(Native)".into(), true, true));
    assert!(verify_own_omission(spec, &required, &key, &real, &disabled, &armed).is_ok());
    for (name, expected) in [
        ("wrong-row", Rejection::WrongRow),
        ("wrong-source-key", Rejection::SourceIdentity),
        ("wrong-tail", Rejection::SourceIdentity),
        ("unarmed-switch", Rejection::Unarmed),
        ("no-hit-phase", Rejection::NoHit),
        ("another-row-hit", Rejection::NoHit),
        ("wrong-phase", Rejection::NoHit),
        ("hit-before-arm", Rejection::NoHit),
        ("surviving-identity", Rejection::IdentitySurvivor),
        ("absent-baseline-pause", Rejection::BaselinePause),
        ("missing-real-tuple", Rejection::MissingRealTuple),
        ("missing-real-observation", Rejection::MissingRealTuple),
        ("disabled-copy-DIFF", Rejection::DisabledDifference),
        ("compile-only-catch", Rejection::CompileOnly),
        ("earlier-guard", Rejection::EarlierGuard),
        ("absent-native-difference", Rejection::NativeDifference),
        ("public-output-difference", Rejection::PublicOutput),
        ("disabled-armed-swap", Rejection::BuildIdentity),
        ("borrowed-real-binary", Rejection::BuildIdentity),
    ] {
        let (mut r, mut d, mut a) = (real.clone(), disabled.clone(), armed.clone());
        match name {
            "wrong-row" => a.row = "P0543".into(),
            "wrong-source-key" => a.source_key.push('!'),
            "wrong-tail" => a.tail_digest.push('!'),
            "unarmed-switch" => a.selector.clear(),
            "no-hit-phase" => a.hits.clear(),
            "another-row-hit" => a.hits[0].0 = "P0543".into(),
            "wrong-phase" => a.hits[0].1 = "different-phase".into(),
            "hit-before-arm" => a.hit_before_arm = true,
            "surviving-identity" => a.observations[0].1 = r.observations[0].1.clone(),
            "absent-baseline-pause" => {
                r.observations[0].4 = 0;
                d.observations[0].4 = 0;
            }
            "missing-real-tuple" => {
                r.cases.clear();
                d.cases.clear();
            }
            "missing-real-observation" => {
                r.observations.clear();
                d.observations.clear();
            }
            "disabled-copy-DIFF" => d.exact[0].push('!'),
            "compile-only-catch" => a.success = false,
            "earlier-guard" => {
                a.observations[0].3 = "Err(Phase)".into();
                a.predicates[0].1 = "Err(Phase)".into();
            }
            "absent-native-difference" => a.predicates[0].3 = false,
            "public-output-difference" => a.predicates[0].2 = false,
            "disabled-armed-swap" => a.arm = "disabled".into(),
            "borrowed-real-binary" => a.binary = r.binary.clone(),
            _ => unreachable!(),
        }
        let rejection = verify_own_omission(spec, &required, &key, &r, &d, &a).err();
        assert_eq!(rejection, Some(expected), "{name}");
        println!(
            "CM4 MUTANT NEGATIVE control={name} predicate={expected:?} synthetic=true mutation_credit=false"
        );
    }
    println!(
        "CM4 MUTANT NEGATIVE CONTROLS PASS negatives=19 synthetic_positive=1 copied_receipts=0"
    );
    let mut paired = real.clone();
    paired
        .constituents
        .push((key.clone(), "synthetic-paired".into(), 1));
    paired.observations.push(paired.observations[0].clone());
    let programs = [(
        key,
        vec!["synthetic-primary".into(), "synthetic-paired".into()],
    )]
    .into();
    assert_eq!(
        super::records::validate_constituents(&paired, &programs),
        Ok(())
    );
    let mut missing = paired.clone();
    missing.constituents.pop();
    missing.observations.pop();
    assert_eq!(
        super::records::validate_constituents(&missing, &programs),
        Err(Rejection::MissingRealTuple)
    );
    let mut wrong = paired.clone();
    wrong.constituents[1].1 = "synthetic-primary".into();
    assert_eq!(
        super::records::validate_constituents(&wrong, &programs),
        Err(Rejection::MissingRealTuple)
    );
    let mut observation = paired;
    observation.observations.pop();
    assert_eq!(
        super::records::validate_constituents(&observation, &programs),
        Err(Rejection::MissingRealTuple)
    );
    println!(
        "CM4 MUTANT CONSTITUENT CONTROLS PASS missing_pair=MissingRealTuple duplicate_pair=MissingRealTuple missing_observation=MissingRealTuple synthetic=true"
    );
}
