//! Class-level observation transport. The predicate on the row is not this mode.
use super::super::schema::MutationSpec;
use super::AuthError;
use std::{collections::BTreeMap, sync::OnceLock};

pub(crate) const POLICY_VERSION: &str = "cm743-1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObservationClass {
    NativeFault,
    DeferredPublication,
    ProfileReturn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObservationMode {
    Ordinary,
    AuthenticatedMutationObserver(ObservationClass),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedCarrier {
    pub row: &'static str,
    pub source_key: &'static str,
    pub tail_digest: &'static str,
    pub producer: bool,
    pub complete_work: bool,
    pub owned_join: bool,
    pub success_return: bool,
    pub bypasses_profile_finish: bool,
    pub evidence: &'static str,
}

fn plans() -> &'static BTreeMap<&'static str, ResolvedCarrier> {
    static PLANS: OnceLock<BTreeMap<&'static str, ResolvedCarrier>> = OnceLock::new();
    PLANS.get_or_init(|| {
        let mut lines = include_str!("observer-policy.tsv").lines();
        assert_eq!(
            lines.next(),
            Some(
                "row\tsource_key\ttail_digest\tproducer\tcomplete_work\towned_join\tsuccess_return\tbypasses_profile_finish\tevidence"
            )
        );
        let mut rows = BTreeMap::new();
        for line in lines {
            let c: Vec<_> = line.split('\t').collect();
            assert_eq!(c.len(), 9, "{line}");
            let plan = ResolvedCarrier {
                row: c[0],
                source_key: c[1],
                tail_digest: c[2],
                producer: flag(c[3]),
                complete_work: flag(c[4]),
                owned_join: flag(c[5]),
                success_return: flag(c[6]),
                bypasses_profile_finish: flag(c[7]),
                evidence: c[8],
            };
            assert!(evidence_agrees(&plan), "{}", plan.row);
            assert!(rows.insert(plan.row, plan).is_none());
        }
        assert_eq!(rows.len(), 151);
        rows
    })
}

fn flag(text: &str) -> bool {
    match text {
        "1" => true,
        "0" => false,
        _ => panic!("observation flag"),
    }
}

fn evidence_agrees(plan: &ResolvedCarrier) -> bool {
    let evidence = plan.evidence;
    let success = evidence.contains("around_ok");
    let producer = evidence.starts_with("enter+") || success;
    evidence.contains("source_join_owner") == plan.owned_join
        && evidence.contains("complete_work") == plan.complete_work
        && success == plan.success_return
        && evidence.contains("bypasses_profile_finish") == plan.bypasses_profile_finish
        && producer == plan.producer
        && (evidence == "ordinary" || success || evidence.starts_with("enter+"))
}

pub(super) fn resolve(spec: &MutationSpec) -> Result<&'static ResolvedCarrier, AuthError> {
    let plan = plans().get(spec.row).ok_or(AuthError::ObserverPolicy)?;
    if plan.source_key != spec.source_key || plan.tail_digest != spec.tail_digest {
        return Err(AuthError::ObserverPolicy);
    }
    Ok(plan)
}

pub(super) fn observation_mode(
    spec: &MutationSpec,
    plan: &ResolvedCarrier,
) -> Result<ObservationMode, AuthError> {
    if plan.row != spec.row
        || plan.source_key != spec.source_key
        || plan.tail_digest != spec.tail_digest
        || !evidence_agrees(plan)
    {
        return Err(AuthError::ObserverPolicy);
    }
    let join = plan.producer && plan.complete_work && plan.owned_join;
    let profile = plan.success_return && plan.bypasses_profile_finish;
    // A plan that is both reviewed topologies is not a fourth transport.
    if join && profile {
        return Err(AuthError::ObserverPolicy);
    }
    if spec.observer == "NativeEqual" {
        return Ok(ObservationMode::AuthenticatedMutationObserver(
            ObservationClass::NativeFault,
        ));
    }
    if join {
        return Ok(ObservationMode::AuthenticatedMutationObserver(
            ObservationClass::DeferredPublication,
        ));
    }
    if profile {
        return Ok(ObservationMode::AuthenticatedMutationObserver(
            ObservationClass::ProfileReturn,
        ));
    }
    Ok(ObservationMode::Ordinary)
}

pub(crate) fn mode_label(mode: ObservationMode) -> &'static str {
    match mode {
        ObservationMode::Ordinary => "ordinary",
        ObservationMode::AuthenticatedMutationObserver(ObservationClass::NativeFault) => {
            "native-fault"
        }
        ObservationMode::AuthenticatedMutationObserver(ObservationClass::DeferredPublication) => {
            "deferred-publication"
        }
        ObservationMode::AuthenticatedMutationObserver(ObservationClass::ProfileReturn) => {
            "profile-return"
        }
    }
}

pub(crate) fn class_name(class: ObservationClass) -> &'static str {
    mode_label(ObservationMode::AuthenticatedMutationObserver(class))
}

/// Armed observer transport. `ordinary_forced` only forces the guard on.
pub(crate) fn observer_transport(mode: ObservationMode, arm: &str, ordinary_forced: bool) -> bool {
    !ordinary_forced
        && arm == "armed"
        && matches!(mode, ObservationMode::AuthenticatedMutationObserver(_))
}

pub(crate) fn members_of(class: ObservationClass) -> Vec<&'static str> {
    let mut rows = Vec::new();
    for spec in super::super::schema::specs().values() {
        let plan = resolve(spec).expect("plan");
        if observation_mode(spec, plan).expect("mode")
            == ObservationMode::AuthenticatedMutationObserver(class)
        {
            rows.push(spec.row);
        }
    }
    rows.sort_unstable();
    rows
}

pub(crate) fn schema_digest() -> String {
    crate::oracle::obligations::content_digest(include_bytes!("overlays.tsv"))
}

pub(crate) fn policy_digest() -> String {
    crate::oracle::obligations::content_digest(include_bytes!("observer-policy.tsv"))
}

/// Importer reason stays put. Quiescence adds a diagnosis and does not clear it.
pub(crate) fn ordinary_diagnosis(quiescence_failed: bool) -> &'static str {
    if quiescence_failed {
        "ObservationUnavailable/LogicalQuiescence"
    } else {
        "-"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClassPacket {
    pub version: String,
    pub class_name: String,
    pub members: Vec<String>,
    pub schema_digest: String,
    pub policy_digest: String,
    pub configuration: String,
    pub paths_complete: bool,
    pub builds_authenticated: bool,
    pub spot_identity: bool,
    pub real_matrix_complete: bool,
    pub disabled_matrix_exact: bool,
    pub baseline_pauses: bool,
    pub arm_then_hit: bool,
    pub observations_present: bool,
    pub ordinary_result: String,
    pub synthetic: bool,
    pub receipt: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PacketFault {
    MissingField(&'static str),
    ClassMismatch,
    Digest,
    Configuration,
    OrdinaryEarlier,
    SyntheticReceipt,
}

pub(crate) fn assess_packet(packet: &ClassPacket) -> Result<(), PacketFault> {
    if packet.version != POLICY_VERSION {
        return Err(PacketFault::MissingField("version"));
    }
    let class = match packet.class_name.as_str() {
        "native-fault" => ObservationClass::NativeFault,
        "deferred-publication" => ObservationClass::DeferredPublication,
        "profile-return" => ObservationClass::ProfileReturn,
        _ => return Err(PacketFault::ClassMismatch),
    };
    if packet.members.is_empty() {
        return Err(PacketFault::MissingField("members"));
    }
    let expected: Vec<_> = members_of(class).into_iter().map(str::to_string).collect();
    if packet.members != expected {
        return Err(PacketFault::ClassMismatch);
    }
    if packet.schema_digest != schema_digest() || packet.policy_digest != policy_digest() {
        return Err(PacketFault::Digest);
    }
    if !matches!(
        packet.configuration.as_str(),
        "default" | "no-default" | "p0543-unchecked"
    ) {
        return Err(PacketFault::Configuration);
    }
    let required = [
        ("paths", packet.paths_complete),
        ("builds", packet.builds_authenticated),
        ("spot", packet.spot_identity),
        ("real-matrix", packet.real_matrix_complete),
        ("disabled-matrix", packet.disabled_matrix_exact),
        ("baseline-pauses", packet.baseline_pauses),
        ("arm-then-hit", packet.arm_then_hit),
        ("observations", packet.observations_present),
    ];
    for (name, present) in required {
        if !present {
            return Err(PacketFault::MissingField(name));
        }
    }
    match packet.ordinary_result.as_str() {
        "equal" | "quiescence" => {}
        "" => return Err(PacketFault::MissingField("ordinary-result")),
        _ => return Err(PacketFault::OrdinaryEarlier),
    }
    // Synthetic data and a private capture bit are not caught receipts.
    if packet.receipt || !packet.synthetic {
        return Err(PacketFault::SyntheticReceipt);
    }
    Ok(())
}

pub(crate) fn authentic_control(class: ObservationClass, configuration: &str) -> ClassPacket {
    ClassPacket {
        version: POLICY_VERSION.into(),
        class_name: class_name(class).into(),
        members: members_of(class).into_iter().map(str::to_string).collect(),
        schema_digest: schema_digest(),
        policy_digest: policy_digest(),
        configuration: configuration.into(),
        paths_complete: true,
        builds_authenticated: true,
        spot_identity: true,
        real_matrix_complete: true,
        disabled_matrix_exact: true,
        baseline_pauses: true,
        arm_then_hit: true,
        observations_present: true,
        ordinary_result: "quiescence".into(),
        synthetic: true,
        receipt: false,
    }
}
