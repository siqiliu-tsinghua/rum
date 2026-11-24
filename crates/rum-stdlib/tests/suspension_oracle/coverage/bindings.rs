//! Frozen requirements precede fixture availability. Blocked declarations do
//! not shrink the active bag, and no declared program is an execution receipt.
use std::collections::{BTreeMap, BTreeSet};

use super::{
    content_digest, imports::AcceptedInputs, partition::ApprovedLedger, partition::EnforcementMode,
};

pub const LEDGER_PIN: &str = "73ec2ff5132fefb684c5069dd99460715b2361f804b0ce00250597b3203c8ac8";
pub const ABSENT_ROWS: [&str; 9] = [
    "P0018", "P0580", "P0709", "P0790", "P0795", "P0798", "P0801", "P2287", "P2288",
];
pub const ABSENT_PACKETS: &str =
    "continuation2/equal-decisions.tsv;continuation2/equal-real-receipts.tsv";
pub const UNRESOLVED_ROWS: [&str; 13] = [
    "P0020", "P0028", "P0327", "P0479", "P1310", "P1331", "P1333", "P1335", "P2596", "P2602",
    "P2603", "P2604", "P2642",
];
pub const REPL_ROWS: [&str; 2] = ["P2647", "P2648"];
pub const REPL_BLOCKER: &str = "crates/rum-repl/src/lib.rs:113:Repl<R,W>::run@152;completeness_tests.rs:private-crate::resolve_completeness;E0433:rustix;E0432:signal_hook";
const HEADER: &str = "record\trow\tvariant\tcontext\tconsumer_route\tdescriptor\tprogram\tevidence";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TupleKey(pub [String; 5]);

impl TupleKey {
    pub fn row(&self) -> &str {
        &self.0[0]
    }
    pub fn variant(&self) -> &str {
        &self.0[1]
    }
    pub fn context(&self) -> &str {
        &self.0[2]
    }
    pub fn line(&self) -> String {
        self.0.join("\t")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingError {
    Inventory,
    DuplicateRequirement,
    DuplicateBinding,
    MissingBinding,
    ExtraBinding,
    Encoding,
    Disposition,
    FrozenSeal,
    ProgramIdentity,
    SourceIdentity,
    BlockedEvidence,
}

pub type BindingResult<T> = Result<T, BindingError>;

#[derive(Clone, Debug)]
pub struct ActiveRequiredBag {
    pub keys: Vec<TupleKey>,
    pub port_blocked: BTreeMap<TupleKey, String>,
}

pub type RequiredBag = ActiveRequiredBag;

#[derive(Clone, Debug)]
pub struct Binding {
    pub key: TupleKey,
    pub program: Option<String>,
    pub evidence: String,
}

#[derive(Clone, Debug)]
pub struct FrozenBindings {
    pub entries: Vec<Binding>,
}

#[derive(Clone, Debug)]
pub struct BoundPlan {
    pub key: TupleKey,
    pub program: String,
    pub source_identity: String,
    pub evidence: String,
}

#[derive(Clone, Debug, Default)]
pub struct BoundBag {
    pub plans: Vec<BoundPlan>,
    pub blocked: BTreeMap<TupleKey, String>,
}

fn rows() -> BindingResult<Vec<Vec<&'static str>>> {
    super::policy::table(
        include_str!("../inventory/obligations/bindings.tsv"),
        HEADER,
    )
    .map_err(|_| BindingError::Inventory)
}

fn key(row: &[&str]) -> TupleKey {
    TupleKey(std::array::from_fn(|index| row[index + 1].to_string()))
}

pub fn provenance() -> BindingResult<Vec<Vec<&'static str>>> {
    super::policy::table(
        include_str!("../inventory/obligations/provenance.tsv"),
        "kind\tid\tpath\tsha256\tcount\tdependency",
    )
    .map_err(|_| BindingError::Inventory)
}

pub fn class_provenance() -> BindingResult<Vec<Vec<&'static str>>> {
    super::policy::table(
        include_str!("../inventory/obligations/class-provenance.tsv"),
        "kind\tid\tpath\tsha256\tcount\tdependency",
    )
    .map_err(|_| BindingError::Inventory)
}

impl ActiveRequiredBag {
    pub fn committed() -> BindingResult<Self> {
        let ledger = ApprovedLedger::committed().map_err(|_| BindingError::Inventory)?;
        let mut seen = BTreeSet::new();
        let mut grouped: BTreeMap<String, Vec<TupleKey>> = BTreeMap::new();
        let mut keys = Vec::new();
        let mut port_blocked = BTreeMap::new();
        for row in rows()?.into_iter().filter(|row| row[0] == "required") {
            let key = key(&row);
            if !seen.insert(key.clone()) {
                return Err(BindingError::DuplicateRequirement);
            }
            if key.context().starts_with("guest_heads.") {
                return Err(BindingError::Encoding);
            }
            let approval = ledger
                .rows
                .get(key.row())
                .ok_or(BindingError::Disposition)?;
            if approval.mode != EnforcementMode::Execute {
                return Err(BindingError::Disposition);
            }
            if row[6] != "-" || row[7] != LEDGER_PIN {
                return Err(BindingError::FrozenSeal);
            }
            grouped
                .entry(key.row().to_string())
                .or_default()
                .push(key.clone());
            if ABSENT_ROWS.contains(&key.row()) {
                port_blocked.insert(key, ABSENT_PACKETS.to_string());
            } else if UNRESOLVED_ROWS.contains(&key.row()) {
                let missing = format!("CM1:aliases.tsv:{}:unresolved-target", key.row());
                port_blocked.insert(key, missing);
            } else if REPL_ROWS.contains(&key.row()) {
                port_blocked.insert(key, REPL_BLOCKER.to_string());
            } else {
                keys.push(key);
            }
        }
        let seals = provenance()?;
        for (row, approval) in ledger
            .rows
            .iter()
            .filter(|(_, a)| a.mode == EnforcementMode::Execute)
        {
            let row_keys = grouped.get_mut(row).ok_or(BindingError::FrozenSeal)?;
            row_keys.sort();
            let text = row_keys
                .iter()
                .map(TupleKey::line)
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            let matching: Vec<_> = seals
                .iter()
                .filter(|r| r[0] == "required-keys" && r[1] == row)
                .collect();
            if matching.len() != 1
                || matching[0][3] != content_digest(text.as_bytes())
                || matching[0][4] != row_keys.len().to_string()
                || matching[0][5] != LEDGER_PIN
                || row_keys.len() != approval.nonencoding
            {
                return Err(BindingError::FrozenSeal);
            }
        }
        // The CM2 leaf projections remain independent of fixture factories.
        let routes = super::routes::committed().map_err(|_| BindingError::Inventory)?;
        for route in routes
            .bindings
            .iter()
            .filter(|route| grouped.contains_key(&route.row))
        {
            let mut actual = BTreeMap::<String, usize>::new();
            for key in &grouped[&route.row] {
                if key.0[3] == route.consumer {
                    *actual.entry(key.0[2].clone()).or_default() += 1;
                }
            }
            let expected: BTreeMap<_, _> = route
                .leaves
                .iter()
                .filter(|(leaf, _)| !leaf.starts_with("guest_heads."))
                .map(|(leaf, count)| (leaf.clone(), *count))
                .collect();
            if actual != expected {
                return Err(BindingError::FrozenSeal);
            }
        }
        if grouped.len() != 175 || keys.len() != 9060 || port_blocked.len() != 1445 {
            return Err(BindingError::FrozenSeal);
        }
        keys.sort();
        Ok(Self { keys, port_blocked })
    }

    /// A complete named-row cohort is only a shape control, never the campaign.
    pub fn cohort(&self, rows: &[&str]) -> Self {
        Self {
            keys: self
                .keys
                .iter()
                .filter(|key| rows.contains(&key.row()))
                .cloned()
                .collect(),
            port_blocked: self
                .port_blocked
                .iter()
                .filter(|(key, _)| rows.contains(&key.row()))
                .map(|(key, evidence)| (key.clone(), evidence.clone()))
                .collect(),
        }
    }
}

impl FrozenBindings {
    pub fn committed() -> BindingResult<Self> {
        let mut entries = Vec::new();
        for row in rows()? {
            match row[0] {
                "required" => continue,
                "bound" | "blocked" => (),
                _ => return Err(BindingError::Inventory),
            }
            if row[7].is_empty() || row[7] == "-" {
                return Err(BindingError::BlockedEvidence);
            }
            let program = if row[0] == "bound" {
                if row[6] == "-" {
                    return Err(BindingError::ProgramIdentity);
                }
                Some(row[6].to_string())
            } else {
                if row[6] != "-" {
                    return Err(BindingError::BlockedEvidence);
                }
                None
            };
            entries.push(Binding {
                key: key(&row),
                program,
                evidence: row[7].to_string(),
            });
        }
        Ok(Self { entries })
    }

    pub fn cohort(&self, rows: &[&str]) -> Self {
        Self {
            entries: self
                .entries
                .iter()
                .filter(|binding| rows.contains(&binding.key.row()))
                .cloned()
                .collect(),
        }
    }
}

pub fn bind(required: &ActiveRequiredBag, frozen: &FrozenBindings) -> BindingResult<BoundBag> {
    let mut required_keys = BTreeSet::new();
    for key in required.keys.iter().chain(required.port_blocked.keys()) {
        if !required_keys.insert(key.clone()) {
            return Err(BindingError::DuplicateRequirement);
        }
    }
    let mut seen = BTreeSet::new();
    for binding in &frozen.entries {
        if !seen.insert(binding.key.clone()) {
            return Err(BindingError::DuplicateBinding);
        }
    }
    if !seen.is_subset(&required_keys) {
        return Err(BindingError::ExtraBinding);
    }
    if seen != required_keys {
        return Err(BindingError::MissingBinding);
    }
    let inputs = AcceptedInputs::committed().map_err(|_| BindingError::Inventory)?;
    let programs = super::super::fixtures::obligations::guest::programs()?;
    let mut result = BoundBag::default();
    for binding in &frozen.entries {
        let key = &binding.key;
        if key.context().starts_with("guest_heads.") {
            return Err(BindingError::Encoding);
        }
        let alias = inputs
            .population
            .aliases
            .get(key.row())
            .ok_or(BindingError::Disposition)?;
        if alias.verdict != "Equal" {
            return Err(BindingError::Disposition);
        }
        if UNRESOLVED_ROWS.contains(&key.row()) && alias.target.is_some() {
            return Err(BindingError::SourceIdentity);
        }
        if let Some(missing) = required.port_blocked.get(key) {
            if binding.program.is_some() || &binding.evidence != missing {
                return Err(BindingError::BlockedEvidence);
            }
        }
        match &binding.program {
            Some(program) => {
                let plan = programs.get(program).ok_or(BindingError::ProgramIdentity)?;
                if &plan.key != key {
                    return Err(BindingError::ProgramIdentity);
                }
                let source_identity = alias.target.clone().ok_or(BindingError::SourceIdentity)?;
                current_target(&inputs, &source_identity)?;
                result.plans.push(BoundPlan {
                    key: key.clone(),
                    program: program.clone(),
                    source_identity,
                    evidence: binding.evidence.clone(),
                });
            }
            None => {
                result.blocked.insert(key.clone(), binding.evidence.clone());
            }
        }
    }
    Ok(result)
}

pub(super) fn current_target(inputs: &AcceptedInputs, target: &str) -> BindingResult<()> {
    static GRAPH: std::sync::OnceLock<super::identity::DiscoveredGraph> =
        std::sync::OnceLock::new();
    let graph = GRAPH.get_or_init(|| {
        super::discovery::discover(
            &super::discovery::SourceTree::workspace(&super::super::coverage::source::workspace()),
            super::identity::FeatureMode::current(),
        )
    });
    let current = graph
        .rows
        .get(target)
        .or_else(|| graph.source_rows.get(target))
        .ok_or(BindingError::SourceIdentity)?;
    let accepted = inputs
        .population
        .rows
        .get(target)
        .ok_or(BindingError::SourceIdentity)?;
    if current.tokens == accepted.tokens
        && current.tail == accepted.tail
        && current.destructors == accepted.destructors
    {
        return Ok(());
    }
    let digest = content_digest(
        format!(
            "{}\0{}\0{}",
            current.tokens, current.tail, current.destructors
        )
        .as_bytes(),
    );
    let seals = class_provenance()?;
    if seals.iter().any(|r| {
        r[0] == "class-target"
            && r[1] == target
            && r[2] == accepted.span.file
            && r[3] == digest
            && r[5] == LEDGER_PIN
    }) {
        return Ok(());
    }
    Err(BindingError::SourceIdentity)
}
