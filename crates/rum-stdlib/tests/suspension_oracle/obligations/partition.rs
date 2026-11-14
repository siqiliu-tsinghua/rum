//! Disjoint counted enforcement bags. Execute is a requirement mode, never an
//! execution receipt. Proof IDs address imported dispositions, not new proofs.
use std::collections::{BTreeMap, BTreeSet};

use super::{
    imports::AcceptedInputs,
    owners::{self, NamedBar, OwnedObligation, OwnerId, OwnerState, OwnerStates},
    policy::*,
    requirements::{self, Count, OWNED, OWNED_HEADER, RequiredBag, RowKey},
};

pub type ProofId = String;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnforcementMode {
    Execute,
    Deferred(OwnerId, NamedBar),
    Proof(ProofId),
}

impl EnforcementMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Execute => "Execute",
            Self::Deferred(_, _) => "Deferred",
            Self::Proof(_) => "Proof",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Scope {
    Nonencoding,
    Encoding,
}

#[derive(Clone, Debug)]
pub struct Enforced {
    pub row: RowKey,
    pub scope: Scope,
    pub keys: usize,
    // The full CM1 row digest binds both classified subsets; this is not a
    // fabricated hash of a subset or an ordinal range of unknown tuples.
    pub row_digest: String,
    pub mode: EnforcementMode,
}

#[derive(Clone, Debug)]
pub struct Approval {
    pub mode: EnforcementMode,
    pub nonencoding: usize,
    pub encoding: usize,
    pub key_digest: String,
}

#[derive(Clone, Debug)]
pub struct ProofMode {
    pub predicate: String,
    pub namespace: String,
    pub ledger_sha256: String,
    pub rows: usize,
    pub keys: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ApprovedLedger {
    pub rows: BTreeMap<RowKey, Approval>,
    pub row_owners: BTreeMap<RowKey, OwnerId>,
    pub dispositions: BTreeMap<RowKey, String>,
    pub owned: Vec<OwnedObligation>,
    pub proofs: BTreeMap<String, ProofMode>,
}

impl ApprovedLedger {
    pub fn committed() -> Checked<Self> {
        let mut result = Self::default();
        for r in table(
            include_str!("../inventory/obligations/enforcement.tsv"),
            "row\tmode\towner\tbar\tproof\tnonencoding\tencoding\tkey_digest",
        )? {
            let mode = match r[1] {
                "Execute" => EnforcementMode::Execute,
                "Deferred" => EnforcementMode::Deferred(r[2].into(), r[3].into()),
                "Proof" => EnforcementMode::Proof(r[4].into()),
                mode => return reject(Predicate::LedgerMismatch, r[0], mode),
            };
            if (r[1] != "Deferred" && (r[2] != "-" || r[3] != "-"))
                || (r[1] != "Proof" && r[4] != "-")
            {
                return reject(Predicate::LedgerMismatch, r[0], "mode-payload");
            }
            if result
                .rows
                .insert(
                    r[0].into(),
                    Approval {
                        mode,
                        nonencoding: number(r[5])?,
                        encoding: number(r[6])?,
                        key_digest: r[7].into(),
                    },
                )
                .is_some()
            {
                return reject(Predicate::DuplicateRequirement, r[0], "approval");
            }
        }
        for r in table(OWNED, OWNED_HEADER)? {
            if r[0] == "row" {
                if result.row_owners.insert(r[1].into(), r[2].into()).is_some() {
                    return reject(Predicate::DuplicateRequirement, r[1], "owned-row");
                }
                result.dispositions.insert(r[1].into(), r[6].into());
                if r[3] == "g13" || r[6].ends_with("/Residual") {
                    result.owned.push(OwnedObligation {
                        row: r[1].into(),
                        owner: r[2].into(),
                    });
                }
            }
        }
        for r in table(
            include_str!("../inventory/obligations/proof-modes.tsv"),
            "mode\tpredicate\tnamespace\tledger_sha256\trows\tkeys",
        )? {
            if result
                .proofs
                .insert(
                    r[0].into(),
                    ProofMode {
                        predicate: r[1].into(),
                        namespace: r[2].into(),
                        ledger_sha256: r[3].into(),
                        rows: number(r[4])?,
                        keys: number(r[5])?,
                    },
                )
                .is_some()
            {
                return reject(Predicate::DuplicateRequirement, r[0], "proof-mode");
            }
        }
        Ok(result)
    }
}

#[derive(Clone)]
pub struct EnforcementPlan {
    pub required: RequiredBag,
    pub ledger: ApprovedLedger,
    pub owners: OwnerStates,
    pub bags: Vec<Enforced>,
}
pub type Plan = EnforcementPlan;

fn proof_id(inputs: &AcceptedInputs, row: &str, verdict: &str) -> Checked<ProofId> {
    let pin = inputs
        .pins
        .iter()
        .find(|(p, _, _)| p == ".temp/g15-c2/c68/settled.tsv")
        .ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::MissingProof,
                row,
                "CM1-ledger-pin",
            )]
        })?;
    Ok(format!("c68:{}:{row}:{verdict}", pin.2))
}

fn expected_mode(inputs: &AcceptedInputs, row: &str, verdict: &str) -> Checked<EnforcementMode> {
    match verdict {
        "Equal" => Ok(EnforcementMode::Execute),
        "Residual" => Ok(EnforcementMode::Deferred(
            "G15-C2".into(),
            "c2-residual-evidence".into(),
        )),
        "Boundary" | "Identity" | "NoObservableTail" | "Transferred" | "Unreachable" => {
            Ok(EnforcementMode::Proof(proof_id(inputs, row, verdict)?))
        }
        value => reject(Predicate::LedgerMismatch, row, value),
    }
}

pub fn validate_ledger(
    bag: &RequiredBag,
    ledger: &ApprovedLedger,
    states: &OwnerStates,
) -> Checked<()> {
    let inputs = AcceptedInputs::committed().map_err(|ds| {
        vec![Diagnostic::new(
            Predicate::PortBlocked,
            "CM1",
            format!("{ds:?}"),
        )]
    })?;
    owners::owner_integrity(&inputs, states)?;
    if ledger.row_owners.len() != 2204
        || ledger.dispositions.len() != 2204
        || ledger.rows.len() != 1042
    {
        return reject(Predicate::LedgerMismatch, "ledger", "population");
    }
    for (row, req) in &bag.rows {
        let accepted = &inputs.population.aliases[row];
        if ledger.row_owners.get(row) != Some(&accepted.owner) {
            let owner = ledger
                .row_owners
                .get(row)
                .map(String::as_str)
                .unwrap_or("missing");
            return reject(
                if states.registry.contains_key(owner) {
                    Predicate::OwnerReassignment
                } else {
                    Predicate::UnknownOwner
                },
                row,
                owner,
            );
        }
        if ledger.dispositions.get(row)
            != Some(&format!("{}/{}", accepted.disposition, accepted.verdict))
        {
            return reject(Predicate::LedgerMismatch, row, "accepted-disposition");
        }
        if let Count::Frozen {
            keys,
            encoding,
            digest,
        } = &req.count
        {
            let a = ledger
                .rows
                .get(row)
                .ok_or_else(|| vec![Diagnostic::new(Predicate::LostFrozenKey, row, "approval")])?;
            if (a.nonencoding + a.encoding, a.encoding, &a.key_digest) != (*keys, *encoding, digest)
            {
                return reject(Predicate::LostFrozenKey, row, "approval-seal");
            }
            if a.mode != expected_mode(&inputs, row, &accepted.verdict)? {
                return reject(Predicate::LedgerMismatch, row, "accepted-enforcement");
            }
        }
    }
    let mut seen = BTreeSet::new();
    for entry in &ledger.owned {
        if !seen.insert(&entry.row) {
            return reject(Predicate::PartitionMultiplicity, &entry.row, "owned-row");
        }
        let state = states.registry.get(&entry.owner).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::UnknownOwner,
                &entry.row,
                &entry.owner,
            )]
        })?;
        if state.state != OwnerState::Open {
            return reject(Predicate::OwnerDone, &entry.row, &entry.owner);
        }
        let alias = inputs.population.aliases.get(&entry.row).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::OwnerReassignment,
                &entry.row,
                "unaccepted-row",
            )]
        })?;
        if alias.owner != entry.owner || !(alias.cohort == "g13" || alias.verdict == "Residual") {
            return reject(Predicate::OwnerReassignment, &entry.row, &entry.owner);
        }
    }
    let expected: BTreeSet<_> = inputs
        .population
        .aliases
        .values()
        .filter(|a| a.cohort == "g13" || a.verdict == "Residual")
        .map(|a| &a.alias)
        .collect();
    if seen != expected {
        return reject(
            Predicate::LostFrozenKey,
            "owned-rows",
            "964-G13-plus-118-residual",
        );
    }
    let pin = inputs
        .pins
        .iter()
        .find(|(p, _, _)| p == ".temp/g15-c2/c68/settled.tsv")
        .unwrap();
    for verdict in [
        "Boundary",
        "Identity",
        "NoObservableTail",
        "Transferred",
        "Unreachable",
    ] {
        let proof = ledger
            .proofs
            .get(verdict)
            .ok_or_else(|| vec![Diagnostic::new(Predicate::MissingProof, verdict, "mode")])?;
        let rows: Vec<_> = bag
            .rows
            .values()
            .filter(|r| matches!(r.count, Count::Frozen { .. }) && r.alias.verdict == verdict)
            .collect();
        let keys: usize = rows.iter().map(|r| r.alias.keys - r.alias.encoding).sum();
        if proof.namespace != "c68"
            || proof.predicate != format!("accepted-{verdict}")
            || proof.ledger_sha256 != pin.2
            || proof.rows != rows.len()
            || proof.keys != keys
        {
            return reject(Predicate::ProofMismatch, verdict, "certificate-population");
        }
    }
    if ledger.proofs.len() != 5 {
        return reject(Predicate::ProofMismatch, "proof-modes", "extra-mode");
    }
    Ok(())
}

pub fn partition(
    required: &RequiredBag,
    ledger: &ApprovedLedger,
    owners: &OwnerStates,
) -> Checked<EnforcementPlan> {
    requirements::validate_bag(required)?;
    validate_ledger(required, ledger, owners)?;
    let mut bags = Vec::new();
    for (row, approval) in &ledger.rows {
        bags.push(Enforced {
            row: row.clone(),
            scope: Scope::Nonencoding,
            keys: approval.nonencoding,
            row_digest: approval.key_digest.clone(),
            mode: approval.mode.clone(),
        });
        if approval.encoding > 0 {
            bags.push(Enforced {
                row: row.clone(),
                scope: Scope::Encoding,
                keys: approval.encoding,
                row_digest: approval.key_digest.clone(),
                mode: EnforcementMode::Deferred(
                    owners::TRANSPORT.into(),
                    owners::TRANSPORT_BAR.into(),
                ),
            });
        }
    }
    let plan = EnforcementPlan {
        required: required.clone(),
        ledger: ledger.clone(),
        owners: owners.clone(),
        bags,
    };
    validate_partition(&plan)?;
    Ok(plan)
}

pub fn validate_partition(plan: &Plan) -> Checked<()> {
    let mut seen = BTreeMap::new();
    let mut totals = [0_usize; 3];
    for part in &plan.bags {
        if part.scope == Scope::Encoding {
            match &part.mode {
                EnforcementMode::Execute | EnforcementMode::Proof(_) => {
                    return reject(
                        Predicate::EncodingExecution,
                        &part.row,
                        "encoding-not-executable-or-creditable",
                    );
                }
                EnforcementMode::Deferred(owner, bar) => {
                    if owner != owners::TRANSPORT || bar != owners::TRANSPORT_BAR {
                        return reject(Predicate::EncodingOwner, &part.row, owner);
                    }
                }
            }
        }
        if seen.insert((&part.row, part.scope), part.keys).is_some() {
            return reject(
                Predicate::PartitionMultiplicity,
                &part.row,
                "duplicate-scope",
            );
        }
        let approval = plan.ledger.rows.get(&part.row).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::LostFrozenKey,
                &part.row,
                "unaccepted-row",
            )]
        })?;
        let expected = match part.scope {
            Scope::Nonencoding => approval.nonencoding,
            Scope::Encoding => approval.encoding,
        };
        if part.keys != expected || part.row_digest != approval.key_digest {
            return reject(Predicate::LostFrozenKey, &part.row, "count-or-digest");
        }
        if part.scope == Scope::Nonencoding && part.mode != approval.mode {
            return reject(
                if matches!(approval.mode, EnforcementMode::Proof(_)) {
                    Predicate::ProofMismatch
                } else {
                    Predicate::LedgerMismatch
                },
                &part.row,
                "nonencoding-mode",
            );
        }
        totals[match &part.mode {
            EnforcementMode::Execute => 0,
            EnforcementMode::Deferred(_, _) => 1,
            EnforcementMode::Proof(_) => 2,
        }] += part.keys;
    }
    for (row, approval) in &plan.ledger.rows {
        for (scope, n) in [
            (Scope::Nonencoding, approval.nonencoding),
            (Scope::Encoding, approval.encoding),
        ] {
            if n > 0 && seen.get(&(row, scope)) != Some(&n) {
                return reject(Predicate::LostFrozenKey, row, "missing-scope");
            }
        }
    }
    if totals != [10505, 24435, 43390] {
        return reject(
            Predicate::LostFrozenKey,
            "partition",
            "Execute-Deferred-Proof-totals",
        );
    }
    Ok(())
}

pub fn closure_check(plan: &Plan, owner: &str) -> Checked<()> {
    let state = plan
        .owners
        .registry
        .get(owner)
        .ok_or_else(|| vec![Diagnostic::new(Predicate::UnknownOwner, owner, "closure")])?;
    let outstanding = plan.ledger.owned.iter().any(|r| r.owner == owner)
        || plan.owners.duties.values().any(|d| d.owner == owner)
        || plan
            .bags
            .iter()
            .any(|p| matches!(&p.mode, EnforcementMode::Deferred(id, _) if id == owner));
    if matches!(state.state, OwnerState::Done(_)) && outstanding {
        return reject(
            Predicate::PrematureClosure,
            owner,
            "owned-row-or-duty-remains",
        );
    }
    Ok(())
}

pub fn manifest_integrity(plan: &Plan) -> bool {
    requirements::validate_bag(&plan.required).is_ok()
        && validate_ledger(&plan.required, &plan.ledger, &plan.owners).is_ok()
        && validate_partition(plan).is_ok()
        && plan
            .owners
            .registry
            .keys()
            .all(|id| closure_check(plan, id).is_ok())
}

pub fn owner_closure(plan: &Plan, owner: &str) -> bool {
    plan.owners
        .registry
        .get(owner)
        .is_some_and(|o| matches!(o.state, OwnerState::Done(_)))
        && closure_check(plan, owner).is_ok()
        && manifest_integrity(plan)
}
