use std::collections::{BTreeMap, BTreeSet};

use super::super::{
    bindings::{self, LEDGER_PIN},
    content_digest,
    imports::AcceptedInputs,
    partition::{ApprovedLedger, EnforcementMode},
    policy::table,
};

const HEADER: &str = "row\tverdict\tledger_sha256\tcertificate\ttarget\tkeys\tkey_digest\tscope\tsource_dependency\tstatus\tmissing\tpacket\tpacket_sha256\tterminal_contract\tsurvivors\ttransfer_targets\tsource_files";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofError {
    Inventory,
    Duplicate,
    Missing,
    Extra,
    Identity,
    Count,
    Pin,
    Scope,
    Dependency,
    TerminalContract,
    UnarmedSurvivor,
    WrongRowSurvivor,
    MissingTransferTarget,
    RootlessCycle,
    ClosedCallers,
    PreparedSkip,
    Closure,
    Packet,
}

#[derive(Clone, Debug)]
pub struct ProofDuty {
    pub row: String,
    pub verdict: String,
    pub pin: String,
    pub identity: String,
    pub target: String,
    pub keys: usize,
    pub key_digest: String,
    pub scope: String,
    pub source_dependency: String,
}

#[derive(Clone, Debug)]
pub struct PinnedEvidence {
    pub imported_status: String,
    pub missing: String,
    pub packet: String,
    pub packet_hash: String,
    pub terminal: String,
    pub survivors: Vec<(String, String, usize)>,
    pub targets: Vec<String>,
    pub sources: Vec<(String, String)>,
}

pub struct CurrentSource {
    pub inputs: AcceptedInputs,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProofStatus {
    Verified,
    PortBlocked(String),
    Rejected(ProofError),
}

#[derive(Clone, Debug)]
pub struct ProofCertificate {
    duty: ProofDuty,
    status: ProofStatus,
    packet_hash: String,
}

impl ProofCertificate {
    pub fn row(&self) -> &str {
        &self.duty.row
    }
    pub fn status(&self) -> &ProofStatus {
        &self.status
    }
    pub fn keys(&self) -> usize {
        self.duty.keys
    }
    pub fn packet_hash(&self) -> &str {
        &self.packet_hash
    }
}

fn members(cell: &str) -> Vec<String> {
    if cell == "-" {
        Vec::new()
    } else {
        cell.split(';').map(str::to_string).collect()
    }
}

pub fn committed() -> Result<Vec<(ProofDuty, PinnedEvidence)>, ProofError> {
    let text = include_str!("../inventory/obligations/proofs.tsv");
    let seals = bindings::provenance().map_err(|_| ProofError::Inventory)?;
    let seal: Vec<_> = seals.iter().filter(|r| r[0] == "proof-table").collect();
    if seal.len() != 1 || seal[0][3] != content_digest(text.as_bytes()) || seal[0][5] != LEDGER_PIN
    {
        return Err(ProofError::Pin);
    }
    let rows = table(text, HEADER).map_err(|_| ProofError::Inventory)?;
    let mut result = Vec::new();
    for r in rows {
        let survivors = members(r[14])
            .into_iter()
            .map(|s| {
                let fields: Vec<_> = s.split('@').collect();
                if fields.len() != 3 {
                    return Err(ProofError::Inventory);
                }
                Ok((
                    fields[0].to_string(),
                    fields[1].to_string(),
                    fields[2].parse().map_err(|_| ProofError::Inventory)?,
                ))
            })
            .collect::<Result<_, _>>()?;
        let sources = members(r[16])
            .into_iter()
            .map(|s| {
                s.split_once('@')
                    .map(|(p, h)| (p.to_string(), h.to_string()))
                    .ok_or(ProofError::Inventory)
            })
            .collect::<Result<_, _>>()?;
        result.push((
            ProofDuty {
                row: r[0].into(),
                verdict: r[1].into(),
                pin: r[2].into(),
                identity: r[3].into(),
                target: r[4].into(),
                keys: r[5].parse().map_err(|_| ProofError::Count)?,
                key_digest: r[6].into(),
                scope: r[7].into(),
                source_dependency: r[8].into(),
            },
            PinnedEvidence {
                imported_status: r[9].into(),
                missing: r[10].into(),
                packet: r[11].into(),
                packet_hash: r[12].into(),
                terminal: r[13].into(),
                survivors,
                targets: members(r[15]),
                sources,
            },
        ));
    }
    Ok(result)
}

pub fn validate_duty(d: &ProofDuty, current: &CurrentSource) -> Result<(), ProofError> {
    let ledger = ApprovedLedger::committed().map_err(|_| ProofError::Inventory)?;
    let approved = ledger.rows.get(&d.row).ok_or(ProofError::Extra)?;
    let alias = current
        .inputs
        .population
        .aliases
        .get(&d.row)
        .ok_or(ProofError::Extra)?;
    if !matches!(approved.mode, EnforcementMode::Proof(_))
        || d.verdict != alias.verdict
        || d.target != alias.target.as_deref().unwrap_or("-")
    {
        return Err(ProofError::Identity);
    }
    if d.pin != LEDGER_PIN
        || d.identity != content_digest(format!("{}\t{}\t{}", d.pin, d.row, d.verdict).as_bytes())
    {
        return Err(ProofError::Pin);
    }
    if d.keys != approved.nonencoding || d.key_digest != approved.key_digest {
        return Err(ProofError::Count);
    }
    if d.scope != "nonencoding-accepted-disposition" {
        return Err(ProofError::Scope);
    }
    if d.source_dependency.len() != 64 {
        return Err(ProofError::Dependency);
    }
    Ok(())
}

/// Content checks are also used by non-crediting validator controls. Passing
/// them does not resolve an alias, authenticate history, or mint a certificate.
pub fn validate_content(
    d: &ProofDuty,
    e: &PinnedEvidence,
    current: &CurrentSource,
) -> Result<(), ProofError> {
    if e.packet == "-" || e.packet_hash.len() != 64 {
        return Err(ProofError::Packet);
    }
    match d.verdict.as_str() {
        "Boundary" => {
            let contract = match d.row.as_str() {
                "P1556" | "P1558" => {
                    "HostSession::eval_script:nonresumable-batch;cancel-retires;reset-retires"
                }
                "P1555" => {
                    "HostSession::eval:NestedEvaluationInputBoundary;retired;C39-whole-token-identity;native-byte-exact"
                }
                "P2605" | "P2606" => {
                    "KernelAdapter::execute:terminal-InputRequired;cleanup-retired;overlap-preserved"
                }
                "P2608" => "KernelAdapter::execute:retirement-cleanup-output-once",
                "P2623" => "rum-cli::retire_input_request:buffered-cleanup-stdout-once",
                "P2624" => "rum-cli::run:full-stderr-error-tail",
                "P2625" => "rum-cli::run:owned-session-drop-summary-once",
                "P2635" => "rum-cli::run_repl:terminal-restoration;descriptor-close",
                "P2645" => "rum-cli::apply_binding:setup-outcome-tail",
                "P2656" => "rum-repl::request:io-error-abandonment-result-tail",
                "P2657" => "rum-repl::request:eof-abandonment-result-tail",
                "P1570" | "P1572" | "P2597" | "P2598" | "P2628" | "P2634" => {
                    "C68:source-argument:script-file-nonresumable;prequeued-batch;blocked-until-reset"
                }
                "P2594" => {
                    "C68:source-argument:restore-Other;abandon-input;retain-partial-state;fresh-request"
                }
                _ => return Err(ProofError::TerminalContract),
            };
            if e.terminal != contract {
                return Err(ProofError::TerminalContract);
            }
        }
        "NoObservableTail" | "Identity" => {
            // C45 accepted these strict whole-Result tails with 54 actual
            // own-hit survivors and 11 named armed error carriers per row.
            // Error carriers are never represented as hit survivors.
            if matches!(d.row.as_str(), "P1564" | "P1565")
                && (d.keys != 65
                    || e.terminal != "C45:strict-whole-result;own-hit=54;armed-error=11"
                    || e.survivors.len() != 1
                    || e.survivors[0].2 != 54)
            {
                return Err(ProofError::Scope);
            }
            if e.survivors.is_empty()
                || e.survivors
                    .iter()
                    .any(|(_, raw, hits)| *hits == 0 || raw.len() != 64)
            {
                return Err(ProofError::UnarmedSurvivor);
            }
            if e.survivors.iter().any(|(row, _, _)| row != &d.row) {
                return Err(ProofError::WrongRowSurvivor);
            }
        }
        "Transferred" => {
            let mut graph = BTreeMap::new();
            graph.insert(d.row.clone(), e.targets.clone());
            let roots: BTreeSet<_> = current
                .inputs
                .population
                .aliases
                .iter()
                .filter(|(row, alias)| *row != &d.row && alias.target.is_some())
                .map(|(row, _)| row.clone())
                .collect();
            validate_transfer(&d.row, &graph, &roots)?;
            if d.row == "P0500" && e.targets != ["C10.prepared.profiled_consumer", "P0505"] {
                return Err(ProofError::MissingTransferTarget);
            }
        }
        "Unreachable" => match d.row.as_str() {
            "P0496" => {
                crate::oracle::fixtures::obligations::proof_controls::validate_closure(
                    &crate::oracle::fixtures::obligations::proof_controls::current_sources(),
                )?;
            }
            "618.optional-source-helpers.038" => {
                if e.terminal
                    != "C68:source-argument:decimal-lowering;closed-InvalidInteger-chain;no-Session-or-guest-call"
                    || d.source_dependency
                        != "af4049c8a3a4fb182a2403fd15017540a4619ec441c8dce5e66230afe337440e"
                    || e.packet_hash != LEDGER_PIN
                    || e.sources.is_empty()
                {
                    return Err(ProofError::Closure);
                }
            }
            "618.optional-source-helpers.054" | "618.optional-source-helpers.055" => {
                if e.terminal
                    != "C68:source-argument:effective-same-test;borrowed-option-data-only;no-Err-or-Session"
                    || d.source_dependency
                        != "5101c7c56562171a4f796c9f25cd407e6f5cfb67c31d5d487bd14d50e268613d"
                    || e.packet_hash != LEDGER_PIN
                    || e.sources.is_empty()
                {
                    return Err(ProofError::Closure);
                }
            }
            _ => return Err(ProofError::Dependency),
        },
        _ => return Err(ProofError::Identity),
    }
    Ok(())
}

pub fn validate_transfer(
    row: &str,
    edges: &BTreeMap<String, Vec<String>>,
    roots: &BTreeSet<String>,
) -> Result<(), ProofError> {
    fn visit(
        row: &str,
        edges: &BTreeMap<String, Vec<String>>,
        roots: &BTreeSet<String>,
        active: &mut BTreeSet<String>,
    ) -> Result<(), ProofError> {
        if roots.contains(row) {
            return Ok(());
        }
        if !active.insert(row.to_string()) {
            return Err(ProofError::RootlessCycle);
        }
        let next = edges
            .get(row)
            .filter(|e| !e.is_empty())
            .ok_or(ProofError::MissingTransferTarget)?;
        for target in next {
            visit(target, edges, roots, active)?;
        }
        active.remove(row);
        Ok(())
    }
    visit(row, edges, roots, &mut BTreeSet::new())
}

pub fn validate_proof(
    d: &ProofDuty,
    current: &CurrentSource,
    e: &PinnedEvidence,
) -> ProofCertificate {
    let status = match validate_duty(d, current) {
        Err(error) => ProofStatus::Rejected(error),
        Ok(()) => {
            if e.imported_status == "PortBlocked" {
                let expected = match d.verdict.as_str() {
                    "Identity" => {
                        "continuation2/identity-decisions.tsv;continuation2/identity-receipts.tsv"
                    }
                    "Transferred" => "continuation2/transfers.tsv",
                    _ => e.missing.as_str(),
                };
                if e.missing.is_empty()
                    || e.missing == "-"
                    || e.missing != expected
                    || e.packet != "-"
                {
                    ProofStatus::Rejected(ProofError::Packet)
                } else {
                    ProofStatus::PortBlocked(e.missing.clone())
                }
            } else if e.imported_status != "VerifiedPacket" {
                ProofStatus::Rejected(ProofError::Packet)
            } else if let Err(error) = validate_content(d, e, current) {
                ProofStatus::Rejected(error)
            } else if d.target == "-" {
                ProofStatus::PortBlocked(format!("CM1:aliases.tsv:{}:unresolved-target", d.row))
            } else if bindings::current_target(&current.inputs, &d.target).is_err() {
                ProofStatus::PortBlocked(format!("CM1:population.tsv:{}:changed-source", d.target))
            } else if e.targets.iter().any(|target| {
                current
                    .inputs
                    .population
                    .aliases
                    .get(target)
                    .and_then(|a| a.target.as_deref())
                    .is_none_or(|id| bindings::current_target(&current.inputs, id).is_err())
            }) {
                ProofStatus::PortBlocked(format!("transfer-source-dependency:{}", d.row))
            } else if e.sources.iter().any(|(p, h)| {
                let path = crate::oracle::coverage::source::workspace().join(p);
                std::fs::read(path)
                    .map(|b| content_digest(&b) != *h)
                    .unwrap_or(true)
            }) {
                ProofStatus::PortBlocked(format!("source-dependency:{}", d.row))
            } else {
                ProofStatus::Verified
            }
        }
    };
    ProofCertificate {
        duty: d.clone(),
        status,
        packet_hash: e.packet_hash.clone(),
    }
}

pub fn reconcile(certificates: &[ProofCertificate]) -> Result<(), ProofError> {
    let mut seen = BTreeSet::new();
    for certificate in certificates {
        if !seen.insert(certificate.row().to_string()) {
            return Err(ProofError::Duplicate);
        }
    }
    let ledger = ApprovedLedger::committed().map_err(|_| ProofError::Inventory)?;
    let required: BTreeSet<_> = ledger
        .rows
        .iter()
        .filter(|(_, a)| matches!(a.mode, EnforcementMode::Proof(_)))
        .map(|(row, _)| row.clone())
        .collect();
    if !seen.is_subset(&required) {
        return Err(ProofError::Extra);
    }
    if seen != required {
        return Err(ProofError::Missing);
    }
    for c in certificates {
        if let ProofStatus::Rejected(error) = c.status {
            return Err(error);
        }
    }
    Ok(())
}
