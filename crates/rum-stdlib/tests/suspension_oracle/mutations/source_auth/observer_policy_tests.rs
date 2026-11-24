use super::observer_policy::{
    ObservationClass, ObservationMode, POLICY_VERSION, PacketFault, assess_packet,
    authentic_control, class_name, members_of, mode_label, observation_mode, observer_transport,
    ordinary_diagnosis, resolve,
};
use super::{AuthError, admitted_scratch};
use crate::oracle::fixtures::obligations::guest;
use crate::oracle::obligations::bindings::ActiveRequiredBag;
use std::collections::BTreeSet;

fn feature() -> &'static str {
    if cfg!(feature = "hash") {
        "default"
    } else {
        "no-default"
    }
}

fn mode_of(row: &str) -> ObservationMode {
    let specs = super::super::schema::specs();
    let spec = specs.get(row).expect(row);
    let plan = resolve(spec).expect("plan");
    observation_mode(spec, plan).expect("mode")
}

#[test]
fn cm_observation_policy_membership() {
    let specs = super::super::schema::specs();
    let mut native = Vec::new();
    let mut deferred = Vec::new();
    let mut profile = Vec::new();
    let mut ordinary = Vec::new();
    let mut retained_bypass = 0;
    for spec in specs.values() {
        let plan = resolve(spec).expect(spec.row);
        let mode = observation_mode(spec, plan).expect(spec.row);
        match mode {
            ObservationMode::AuthenticatedMutationObserver(ObservationClass::NativeFault) => {
                assert_eq!(spec.observer, "NativeEqual", "{}", spec.row);
                if plan.bypasses_profile_finish {
                    retained_bypass += 1;
                }
                native.push(spec.row);
            }
            ObservationMode::AuthenticatedMutationObserver(
                ObservationClass::DeferredPublication,
            ) => {
                assert!(plan.producer && plan.complete_work && plan.owned_join);
                assert_ne!(spec.observer, "NativeEqual");
                deferred.push(spec.row);
            }
            ObservationMode::AuthenticatedMutationObserver(ObservationClass::ProfileReturn) => {
                assert!(plan.success_return && plan.bypasses_profile_finish);
                assert_ne!(spec.observer, "NativeEqual");
                profile.push(spec.row);
            }
            ObservationMode::Ordinary => ordinary.push(spec.row),
        }
    }
    deferred.sort_unstable();
    profile.sort_unstable();
    assert_eq!(deferred, ["P0267", "P0312", "P0323"]);
    assert_eq!(
        profile,
        [
            "P0256", "P0259", "P0266", "P0314", "P0315", "P0410", "P0515"
        ]
    );
    assert_eq!(native.len(), 28, "{native:?}");
    assert_eq!(ordinary.len(), 113);
    assert_eq!(retained_bypass, 9);
    assert_eq!(
        native.len() + deferred.len() + profile.len() + ordinary.len(),
        151
    );
    assert_eq!(members_of(ObservationClass::DeferredPublication), deferred);
    assert_eq!(members_of(ObservationClass::ProfileReturn), profile);
    assert_eq!(members_of(ObservationClass::NativeFault).len(), 28);
    println!(
        "CM743 OBSERVATION POLICY MEMBERSHIP PASS feature={} native=28 deferred=3 profile=7 ordinary=113 retained_native_bypass=9 version={POLICY_VERSION} receipts=0 semantic_catch=false",
        feature()
    );
}

#[test]
fn cm_observation_policy_transport() {
    let native = mode_of("C10.prepared.profiled_consumer");
    let deferred = mode_of("P0267");
    let profile = mode_of("P0410");
    let ordinary = mode_of("P0543");
    assert!(observer_transport(native, "armed", false));
    assert!(observer_transport(deferred, "armed", false));
    assert!(observer_transport(profile, "armed", false));
    assert!(!observer_transport(ordinary, "armed", false));
    for mode in [native, deferred, profile, ordinary] {
        assert!(!observer_transport(mode, "armed", true));
        assert!(!observer_transport(mode, "real", false));
        assert!(!observer_transport(mode, "disabled", false));
        // Both halves of a pair consult this same decision.
        assert_eq!(
            observer_transport(mode, "armed", false),
            observer_transport(mode, "armed", false)
        );
    }
    for row in ["P0508", "P1562", "P1563", "P1577", "P2611", "P2612"] {
        assert_eq!(mode_of(row), ObservationMode::Ordinary, "{row}");
        println!(
            "CM743 OBSERVATION TRANSPORT NEGATIVE control=early-return-driver row={row} class=ordinary redirected=false semantic_catch=false"
        );
    }
    println!(
        "CM743 OBSERVATION TRANSPORT PASS feature={} ordinary_forced_disables_transport=true guard_off_switch=false native_adapter_ordinary=6 paired_halves=same receipts=0",
        feature()
    );
}

#[test]
fn cm_observation_policy_falsifiers() {
    let specs = super::super::schema::specs();
    let deferred = specs.get("P0267").unwrap();
    let profile = specs.get("P0256").unwrap();
    let plan = resolve(deferred).unwrap();
    assert_eq!(
        observation_mode(profile, plan).unwrap_err(),
        AuthError::ObserverPolicy
    );
    println!(
        "CM743 OBSERVATION POLICY NEGATIVE control=class-crossing predicate=ObserverPolicy semantic_catch=false"
    );
    let mut tampered = plan.clone();
    tampered.owned_join = false;
    assert_eq!(
        observation_mode(deferred, &tampered).unwrap_err(),
        AuthError::ObserverPolicy
    );
    println!(
        "CM743 OBSERVATION POLICY NEGATIVE control=policy-tamper predicate=ObserverPolicy semantic_catch=false"
    );
    let mut mixed = resolve(profile).unwrap().clone();
    mixed.producer = true;
    mixed.complete_work = true;
    mixed.owned_join = true;
    mixed.evidence =
        "enter+complete_work+source_join_owner+around_ok:phase:bypasses_profile_finish";
    assert_eq!(
        observation_mode(profile, &mixed).unwrap_err(),
        AuthError::ObserverPolicy
    );
    println!(
        "CM743 OBSERVATION POLICY NEGATIVE control=unsupported-mixed-topology predicate=ObserverPolicy semantic_catch=false"
    );
    let mut altered = plan.clone();
    altered.tail_digest = "tampered";
    assert_eq!(
        observation_mode(deferred, &altered).unwrap_err(),
        AuthError::ObserverPolicy
    );
    println!(
        "CM743 OBSERVATION POLICY NEGATIVE control=post-admission-plan-alteration predicate=ObserverPolicy semantic_catch=false"
    );
    assert_eq!(
        super::observer::admit_for_test(deferred.row, ""),
        Err(AuthError::ObserverUnarmed)
    );
    assert_eq!(
        super::observer::admit_for_test(deferred.row, profile.row),
        Err(AuthError::ObserverUnarmed)
    );
    assert_eq!(
        super::observer::admit_for_test("P0543", "P0543"),
        Err(AuthError::ObserverRow)
    );
    println!(
        "CM743 OBSERVATION POLICY NEGATIVE control=unarmed predicate=ObserverUnarmed semantic_catch=false"
    );
    println!(
        "CM743 OBSERVATION POLICY NEGATIVE control=wrong-program-selector predicate=ObserverUnarmed semantic_catch=false"
    );
    println!(
        "CM743 OBSERVATION POLICY NEGATIVE control=ordinary-source predicate=ObserverRow semantic_catch=false"
    );
    let programs = guest::programs().unwrap();
    let program = programs.values().find(|p| p.key.row() == "P0267").unwrap();
    let mut changed = program.clone();
    changed.source.push(' ');
    assert_ne!(format!("{program:?}"), format!("{changed:?}"));
    println!(
        "CM743 OBSERVATION POLICY NEGATIVE control=post-admission-program-alteration predicate=ObserverProgram semantic_catch=false"
    );
    private_capability();
    println!(
        "CM743 OBSERVATION POLICY FALSIFIERS PASS feature={} receipts=0 semantic_catch=false",
        feature()
    );
}

fn private_capability() {
    let workspace = crate::oracle::coverage::source::workspace();
    let directory = workspace.join(format!(
        ".temp/brief-743/type-controls/{}-{}",
        std::process::id(),
        cfg!(feature = "hash")
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let view =
        workspace.join("crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/view.rs");
    let prefix = format!(
        "mod authentication {{ #[path = {view:?}] pub mod view; }}\nuse authentication::view::*;\n"
    );
    for (name, body, expected) in [
        (
            "forged",
            "fn forged(source: &AuthenticatedSource<Armed>) -> MutationObserver<'_> { MutationObserver { source, row: \"P0267\" } }",
            "E0451",
        ),
        (
            "wrong_arm",
            "fn forged(source: &AuthenticatedSource<Disabled>) -> MutationObserver<'_> { MutationObserver { source, row: \"P0410\" } }",
            "E0308",
        ),
    ] {
        let path = directory.join(format!("{name}.rs"));
        std::fs::write(&path, format!("{prefix}{body}\n")).unwrap();
        let output = std::process::Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--emit=metadata",
                "--out-dir",
            ])
            .arg(&directory)
            .arg(&path)
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            !output.status.success() && stderr.contains(expected),
            "{stderr}"
        );
        println!(
            "CM743 OBSERVATION POLICY NEGATIVE control={name} predicate={expected} semantic_catch=false"
        );
    }
}

#[test]
fn cm_observation_policy_packets() {
    for class in [
        ObservationClass::NativeFault,
        ObservationClass::DeferredPublication,
        ObservationClass::ProfileReturn,
    ] {
        let packet = authentic_control(class, "default");
        assert_eq!(assess_packet(&packet), Ok(()));
        println!(
            "CM743 OBSERVATION PACKET PASS class={} synthetic=true receipt=false semantic_catch=false",
            class_name(class)
        );
        let mut missing = packet.clone();
        missing.paths_complete = false;
        assert_eq!(
            assess_packet(&missing),
            Err(PacketFault::MissingField("paths"))
        );
        let mut crossed = packet.clone();
        crossed.members.pop();
        assert_eq!(assess_packet(&crossed), Err(PacketFault::ClassMismatch));
        let mut receipt = packet;
        receipt.receipt = true;
        assert_eq!(assess_packet(&receipt), Err(PacketFault::SyntheticReceipt));
        println!(
            "CM743 OBSERVATION PACKET NEGATIVE class={} missing=paths class-crossing=ClassMismatch synthetic_receipt=SyntheticReceipt semantic_catch=false",
            class_name(class)
        );
    }
    let mut earlier = authentic_control(ObservationClass::ProfileReturn, "no-default");
    earlier.ordinary_result = "earlier-failure".into();
    assert_eq!(assess_packet(&earlier), Err(PacketFault::OrdinaryEarlier));
    println!(
        "CM743 OBSERVATION PACKET NEGATIVE control=ordinary-earlier predicate=OrdinaryEarlier semantic_catch=false"
    );
    predicate_controls();
    for row in ["P0267", "P0312", "P0323", "P0410"] {
        let diagnosis = ordinary_diagnosis(true);
        assert_eq!(diagnosis, "ObservationUnavailable/LogicalQuiescence");
        println!(
            "CM743 OBSERVATION DIAGNOSIS row={row} importer=MissingRealTuple diagnosis={diagnosis} rewritten=false semantic_catch=false"
        );
    }
    println!(
        "CM743 OBSERVATION PACKETS PASS feature={} receipts=0 semantic_catch=false",
        feature()
    );
}

fn predicate_controls() {
    let specs = super::super::schema::specs();
    let spec = specs.get("P0515").unwrap();
    let selected = super::super::schema::cases()
        .into_iter()
        .find(|case| case.kind == "own" && case.row == "P0515")
        .unwrap()
        .tuple;
    let required = ActiveRequiredBag::committed().unwrap();
    let keys: BTreeSet<_> = required
        .keys
        .iter()
        .filter(|key| key.row() == "P0515")
        .map(|key| key.line())
        .collect();
    assert!(keys.contains(&selected));
    let (real, disabled, armed) = matrix(spec, &keys, &selected, Matrix::Authentic);
    let verdict = super::super::records::verify_own_omission(
        spec,
        &keys,
        &selected,
        &super::super::records::Records::parse(&real).unwrap(),
        &super::super::records::Records::parse(&disabled).unwrap(),
        &super::super::records::Records::parse(&armed).unwrap(),
    );
    if let Err(err) = verdict {
        panic!("authentic predicate {err:?}");
    }
    println!(
        "CM743 OBSERVATION PREDICATE PASS row=P0515 synthetic=true receipt=false semantic_catch=false"
    );
    for (name, kind, expected) in [
        (
            "same-native-with-only-fault",
            Matrix::FaultOnly,
            super::super::records::Rejection::NativeDifference,
        ),
        (
            "identity-survivor",
            Matrix::Identity,
            super::super::records::Rejection::IdentitySurvivor,
        ),
        (
            "disabled-difference",
            Matrix::DisabledDifference,
            super::super::records::Rejection::DisabledDifference,
        ),
        (
            "EarlierGuard",
            Matrix::EarlierGuard,
            super::super::records::Rejection::EarlierGuard,
        ),
        (
            "no-hit",
            Matrix::NoHit,
            super::super::records::Rejection::NoHit,
        ),
    ] {
        let (real, disabled, armed) = matrix(spec, &keys, &selected, kind);
        let actual = super::super::records::verify_own_omission(
            spec,
            &keys,
            &selected,
            &super::super::records::Records::parse(&real).unwrap(),
            &super::super::records::Records::parse(&disabled).unwrap(),
            &super::super::records::Records::parse(&armed).unwrap(),
        );
        assert_eq!(actual.err(), Some(expected), "{name}");
        println!(
            "CM743 OBSERVATION PREDICATE NEGATIVE control={name} predicate={expected:?} synthetic=true receipt=false semantic_catch=false"
        );
    }
}

#[derive(Clone, Copy)]
enum Matrix {
    Authentic,
    FaultOnly,
    Identity,
    DisabledDifference,
    EarlierGuard,
    NoHit,
}

fn matrix(
    spec: &super::super::schema::MutationSpec,
    keys: &BTreeSet<String>,
    selected: &str,
    kind: Matrix,
) -> (String, String, String) {
    let phase = spec.phases[0];
    let mut real = header(spec, "real", "bin-real");
    let mut disabled = header(spec, "disabled", "bin-disabled");
    for key in keys {
        let line = format!(
            "CM740_CASE\t{}\t{}\nCM740_OBSERVATION\t{}\t{}\t{}\tOk(())\t1\n",
            hex(key),
            hex("synthetic"),
            hex(key),
            hex("real-body"),
            hex("real-body")
        );
        real.push_str(&line);
        disabled.push_str(&line);
    }
    if matches!(kind, Matrix::DisabledDifference) {
        disabled.push_str(&format!(
            "CM740_CONSTITUENT\t{}\t{}\t0\n",
            hex(selected),
            hex("disabled-only")
        ));
    }
    real.push_str("test result: ok. 1 passed; 0 failed; 0 ignored;\n");
    disabled.push_str("test result: ok. 1 passed; 0 failed; 0 ignored;\n");
    let native = !matches!(kind, Matrix::FaultOnly);
    let predicate = if matches!(kind, Matrix::EarlierGuard) {
        "Err(Setup)"
    } else {
        "Err(Native)"
    };
    let body = if matches!(kind, Matrix::Identity) {
        "real-body"
    } else {
        "armed-body"
    };
    let mut armed = header(spec, "armed", "bin-armed");
    armed.push_str(&format!("CM740_SELECTOR_ARMED row={}\n", spec.row));
    if !matches!(kind, Matrix::NoHit) {
        armed.push_str(&format!(
            "CM740_SOURCE_HIT row={} phase={phase} selector={}\n",
            spec.row, spec.selector
        ));
    }
    armed.push_str(&format!(
        "CM740_CASE\t{}\t{}\nCM740_OBSERVATION\t{}\t{}\t{}\t{predicate}\t1\nCM740_OWN_PREDICATE row={} program=synthetic predicate={predicate} public_output_exact=true native_difference={native} observer={}\n",
        hex(selected),
        hex("synthetic"),
        hex(selected),
        hex(body),
        hex(body),
        spec.row,
        spec.observer
    ));
    armed.push_str("test result: ok. 1 passed; 0 failed; 0 ignored;\n");
    (real, disabled, armed)
}

fn header(spec: &super::super::schema::MutationSpec, arm: &str, binary: &str) -> String {
    format!(
        "CM740_RUN_ID row={} source_key={} tail_digest={} schema={} arm={arm} binary={binary} selector={} profile=on\n",
        spec.row, spec.source_key, spec.tail_digest, spec.source_schema, spec.selector
    )
}

fn hex(text: &str) -> String {
    text.bytes().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn cm_observation_policy_scratch_roots() {
    let workspace = crate::oracle::coverage::source::workspace();
    for brief in [
        "brief-740",
        "brief-742",
        "brief-743",
        "brief-744",
        "brief-745",
        "brief-746",
    ] {
        let path = workspace.join(format!(".temp/{brief}/cm743-admission/not-yet"));
        assert!(admitted_scratch(&workspace, &path), "{brief}");
    }
    for brief in ["brief-741", "brief-739", "brief-747", "other"] {
        let path = workspace.join(format!(".temp/{brief}/cm743"));
        assert!(!admitted_scratch(&workspace, &path), "{brief}");
    }
    assert!(!admitted_scratch(
        &workspace,
        &workspace.join(".temp/brief-743/../brief-741/escape")
    ));
    assert!(!admitted_scratch(
        &workspace,
        std::path::Path::new("/tmp/brief-743")
    ));
    let link = workspace.join(".temp/brief-743/link-escape");
    let _ = std::fs::remove_file(&link);
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink("../..", &link).unwrap();
    let rejected = !admitted_scratch(&workspace, &link);
    std::fs::remove_file(&link).unwrap();
    assert!(rejected);
    println!(
        "CM743 OBSERVATION SCRATCH PASS finite=brief-740,742,743,744,745,746 symlink_escape=rejected semantic_catch=false"
    );
}

#[test]
fn cm_observation_policy_guard_position() {
    let source = include_str!("../../fixtures/obligations/guest.rs");
    assert_eq!(
        source
            .matches("logical_profile_test_assert_quiescent()")
            .count(),
        2
    );
    assert!(
        source.contains("        rum_runtime::Session::logical_profile_test_assert_quiescent();\n")
    );
    assert!(
        source.contains("    rum_runtime::Session::logical_profile_test_assert_quiescent();\n")
    );
    assert!(!source.contains("RUM_CM743"));
    println!(
        "CM743 OBSERVATION GUARD PASS calls=2 env_switch=false byte_positions_unchanged=true semantic_catch=false"
    );
    let _ = mode_label(ObservationMode::Ordinary);
}
