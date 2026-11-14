//! Versioned Open owners and independent source-provenance duties.
use std::collections::BTreeMap;

use super::{
    imports::AcceptedInputs,
    policy::*,
    requirements::{OWNED, OWNED_HEADER, RowKey},
};

pub type OwnerId = String;
pub type PromotionId = String;
pub type DutyKey = String;
pub type SourceKey = String;
pub type NamedBar = String;
pub const TRANSPORT: &str = "G13-transport";
pub const TRANSPORT_BAR: &str = "transport-program-not-opened";
pub const TRANSPORT_BAR_TEXT: &str =
    "transport encoding program not opened: no encoding execution, receipt, scheduling or credit";
pub const SOURCE_BAR: &str = "source-provenance-design-first";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnerState {
    Open,
    Done(PromotionId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Owner {
    pub id: OwnerId,
    pub kind: String,
    pub state: OwnerState,
    pub subject: String,
    pub bar: NamedBar,
    pub provenance: String,
    pub count: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnedObligation {
    pub row: RowKey,
    pub owner: OwnerId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDuty {
    pub id: DutyKey,
    pub subjects: Vec<SourceKey>,
    pub dependencies: Vec<RowKey>,
    pub owner: OwnerId,
    pub bar: NamedBar,
    pub pinned_subject: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Default)]
pub struct OwnerStates {
    pub registry: BTreeMap<OwnerId, Owner>,
    pub duties: BTreeMap<DutyKey, SourceDuty>,
}

pub fn committed() -> Checked<OwnerStates> {
    let mut out = OwnerStates::default();
    for r in table(OWNED, OWNED_HEADER)? {
        if r[0] == "state" {
            if r[4] != "Open" {
                return reject(Predicate::OwnerDone, r[1], "no-stage-promotion");
            }
            let owner = Owner {
                id: r[1].into(),
                kind: r[3].into(),
                state: OwnerState::Open,
                subject: r[5].into(),
                bar: r[8].into(),
                provenance: r[9].into(),
                count: if r[7] == "-" {
                    None
                } else {
                    Some(number(r[7])?)
                },
            };
            if r[1] != r[2] || out.registry.insert(r[1].into(), owner).is_some() {
                return reject(
                    Predicate::UnknownOwner,
                    r[1],
                    "duplicate-or-redirected-state",
                );
            }
        }
    }
    for r in table(
        include_str!("../inventory/obligations/source-duties.tsv"),
        "duty\towner\tsubjects\tdependencies\tbar\tpinned_subject\tsha256",
    )? {
        let duty = SourceDuty {
            id: r[0].into(),
            owner: r[1].into(),
            subjects: members(r[2])?,
            dependencies: members(r[3])?,
            bar: r[4].into(),
            pinned_subject: r[5].into(),
            sha256: r[6].into(),
        };
        if out.duties.insert(r[0].into(), duty).is_some() {
            return reject(Predicate::MissingDuty, r[0], "duplicate-duty");
        }
    }
    Ok(out)
}

// Exact function identities from the pinned source packets. No file-wide,
// suffix or nearest-position assignment, even for unresolved P1560/P1571.
pub fn subjects(id: &str) -> Checked<Vec<String>> {
    let subjects: &[&str] = match id {
        "Session::lower_text" => {
            &["crates/rum-runtime/src/runtime/session/session_api.rs:Session::lower_text"]
        }
        "ToExpression" => &["crates/rum-stdlib/src/strings/text/builtins.rs:::eval_to_expression"],
        "outer-request lowering" => &[
            "crates/rum-runtime/src/runtime/session/session_requests.rs:Session::eval_request_admitted",
            "crates/rum-runtime/src/runtime/session/session_requests.rs:Session::lower_request",
        ],
        "loader" => &[
            "crates/rum-runtime/src/runtime/session/session_eval.rs:Session::eval_file",
            "crates/rum-runtime/src/runtime/session/session_eval.rs:Session::eval_file_retaining_output",
        ],
        "file-reread provenance" => &[
            "crates/rum-runtime/src/runtime/session/session_requests.rs:Session::eval_file_request",
            "crates/rum-runtime/src/runtime/session/session_requests.rs:Session::eval_file_request_admitted",
        ],
        unknown => return reject(Predicate::MissingDuty, unknown, "undeclared-source-duty"),
    };
    Ok(subjects.iter().map(|s| (*s).into()).collect())
}

pub fn owner_integrity(inputs: &AcceptedInputs, states: &OwnerStates) -> Checked<()> {
    for (id, owner) in &states.registry {
        if id != TRANSPORT && !inputs.owners.contains_key(id) {
            return reject(Predicate::UnknownOwner, id, "registry");
        }
        if owner.state != OwnerState::Open {
            return reject(Predicate::OwnerDone, id, "no-promotion-in-CM2");
        }
    }
    for (id, r) in &inputs.owners {
        let owner = states.registry.get(id).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::UnknownOwner,
                id,
                "missing-state",
            )]
        })?;
        let (subject, bar) = match r[0].as_str() {
            "campaign" => ("c2-rows", "c2-residual-evidence"),
            "g13" => ("g13-rows", "g13-step-promotion"),
            "source" => (r[5].as_str(), SOURCE_BAR),
            kind => return reject(Predicate::UnknownOwner, id, kind),
        };
        if owner.id != *id
            || owner.kind != r[0]
            || owner.subject != subject
            || owner.bar != bar
            || owner.count.is_some()
            || owner.provenance != r[6]
        {
            return reject(Predicate::OwnerReassignment, id, "CM1-owner-state");
        }
    }
    let transport = states.registry.get(TRANSPORT).ok_or_else(|| {
        vec![Diagnostic::new(
            Predicate::UnknownOwner,
            TRANSPORT,
            "missing-state",
        )]
    })?;
    if transport.id != TRANSPORT
        || transport.kind != "g13-transport"
        || transport.subject != "c2-encoding-keys"
        || transport.count != Some(18840)
        || transport.bar != TRANSPORT_BAR
        || transport.provenance != "738:continuation-1"
    {
        return reject(
            Predicate::EncodingOwner,
            TRANSPORT,
            "identity-kind-subject-count-bar",
        );
    }
    for (id, r) in inputs.owners.iter().filter(|(_, r)| r[0] == "source") {
        let duty = states.duties.get(id).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::MissingDuty,
                id,
                "source-provenance-duty",
            )]
        })?;
        if duty.owner != *id {
            return reject(Predicate::DutyReassignment, id, &duty.owner);
        }
        let expected = subjects(id)?;
        let dependencies: Vec<_> = inputs
            .population
            .aliases
            .values()
            .filter(|a| expected.contains(&a.function))
            .map(|a| a.alias.clone())
            .collect();
        if duty.id != *id
            || duty.subjects != expected
            || duty.dependencies != dependencies
            || duty.bar != SOURCE_BAR
            || duty.pinned_subject != r[5]
            || duty.sha256 != r[6]
        {
            return reject(
                Predicate::DutySubjectMismatch,
                id,
                "subject-dependencies-bar-pin",
            );
        }
    }
    if states.duties.len() != 5 || states.registry.len() != 51 {
        return reject(Predicate::MissingDuty, "registry", "51-states-five-duties");
    }
    Ok(())
}
