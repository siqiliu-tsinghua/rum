//! Exhaustive policy vocabulary; no factory or execution outcome is an input.
use std::collections::{BTreeMap, BTreeSet};

use super::{identity::Population, imports};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Predicate {
    PortBlocked,
    UnclassifiedPolicy,
    DuplicatePolicy,
    ForbiddenPolicy,
    MissingPolicy,
    InvalidPolicy,
    DuplicateRequirement,
    MissingApplicability,
    ApplicabilityMismatch,
    MissingLeaf,
    MissingRoute,
    RouteSealMismatch,
    RequirementSealMismatch,
    LostFrozenKey,
    PartitionMultiplicity,
    EncodingExecution,
    EncodingOwner,
    UnknownOwner,
    OwnerDone,
    PrematureClosure,
    OwnerReassignment,
    MissingDuty,
    DutyReassignment,
    DutySubjectMismatch,
    MissingProof,
    ProofMismatch,
    LedgerMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Diagnostic {
    pub predicate: Predicate,
    pub subject: String,
    pub datum: String,
}
pub type Diagnostics = Vec<Diagnostic>;
pub type Checked<T> = Result<T, Diagnostics>;

impl Diagnostic {
    pub fn new(predicate: Predicate, subject: impl Into<String>, datum: impl Into<String>) -> Self {
        Self {
            predicate,
            subject: subject.into(),
            datum: datum.into(),
        }
    }
    pub fn line(&self) -> String {
        format!(
            "CM DIAGNOSTIC {:?}\t{}\t{}",
            self.predicate, self.subject, self.datum
        )
    }
}

pub fn reject<T>(predicate: Predicate, subject: &str, datum: &str) -> Checked<T> {
    Err(vec![Diagnostic::new(predicate, subject, datum)])
}

pub fn table<'a>(text: &'a str, header: &str) -> Checked<Vec<Vec<&'a str>>> {
    imports::table(text, header).map_err(|ds| {
        ds.into_iter()
            .map(|d| Diagnostic::new(Predicate::PortBlocked, d.context, d.detail))
            .collect()
    })
}

pub fn members(cell: &str) -> Checked<Vec<String>> {
    imports::members(cell).map_err(|ds| {
        ds.into_iter()
            .map(|d| Diagnostic::new(Predicate::DuplicateRequirement, d.context, d.detail))
            .collect()
    })
}

pub fn number(cell: &str) -> Checked<usize> {
    cell.parse().map_err(|_| {
        vec![Diagnostic::new(
            Predicate::PortBlocked,
            cell,
            "unsigned-count",
        )]
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PolicyId {
    CaptureOutput,
    ConditionalBranch,
    DefinitionLhsPreparation,
    DefinitionStore,
    ErrorTransform,
    ForwardResult,
    FunctionResult,
    GuestLoading,
    HostBatchBoundary,
    HostRequestDriver,
    MatchDecision,
    MatchingSymbolCycleRecovery,
    NativeCallbackBridge,
    NonSuspending,
    OptionReader,
    OptionalStage,
    PhaseOwnership,
    PrefixLoop,
    PrepareArgumentsRestore,
    RebuildTraversal,
    RetiredBoundary,
    ReturnBoundary,
    ScalarFinish,
    ScopeCleanup,
    ShapeDecode,
}

pub const ALL: [PolicyId; 25] = [
    PolicyId::CaptureOutput,
    PolicyId::ConditionalBranch,
    PolicyId::DefinitionLhsPreparation,
    PolicyId::DefinitionStore,
    PolicyId::ErrorTransform,
    PolicyId::ForwardResult,
    PolicyId::FunctionResult,
    PolicyId::GuestLoading,
    PolicyId::HostBatchBoundary,
    PolicyId::HostRequestDriver,
    PolicyId::MatchDecision,
    PolicyId::MatchingSymbolCycleRecovery,
    PolicyId::NativeCallbackBridge,
    PolicyId::NonSuspending,
    PolicyId::OptionReader,
    PolicyId::OptionalStage,
    PolicyId::PhaseOwnership,
    PolicyId::PrefixLoop,
    PolicyId::PrepareArgumentsRestore,
    PolicyId::RebuildTraversal,
    PolicyId::RetiredBoundary,
    PolicyId::ReturnBoundary,
    PolicyId::ScalarFinish,
    PolicyId::ScopeCleanup,
    PolicyId::ShapeDecode,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyKind {
    Tail,
    Carrier,
    Route,
    Applicability,
}

impl PolicyKind {
    pub fn token(self) -> &'static str {
        match self {
            Self::Tail => "tail",
            Self::Carrier => "carrier",
            Self::Route => "route",
            Self::Applicability => "applicability",
        }
    }
}

impl PolicyId {
    pub fn parse(token: &str) -> Checked<Self> {
        if ["*", "_", "Other", "default"].contains(&token) {
            return reject(Predicate::ForbiddenPolicy, token, "no-default-arm");
        }
        ALL.into_iter()
            .find(|id| format!("{id:?}") == token)
            .ok_or_else(|| {
                vec![Diagnostic::new(
                    Predicate::UnclassifiedPolicy,
                    token,
                    "policy-token",
                )]
            })
    }
    pub fn specification(self) -> (PolicyKind, &'static str) {
        use PolicyKind::{Applicability, Carrier, Route, Tail};
        match self {
            Self::CaptureOutput => (Tail, "captured-output"),
            Self::ConditionalBranch => (Tail, "selected-branch"),
            Self::DefinitionLhsPreparation => (Tail, "lhs-preparation"),
            Self::DefinitionStore => (Tail, "definition-store"),
            Self::ErrorTransform => (Carrier, "error-transform"),
            Self::ForwardResult => (Route, "indexed-callers-or-concrete-root"),
            Self::FunctionResult => (Carrier, "function-result-finish"),
            Self::GuestLoading => (Route, "loader-lifecycle"),
            Self::HostBatchBoundary => (Applicability, "source-boundary-certificate"),
            Self::HostRequestDriver => (Route, "request-lifecycle"),
            Self::MatchDecision => (Carrier, "match-decision"),
            Self::MatchingSymbolCycleRecovery => (Tail, "matching-cycle-recovery"),
            Self::NativeCallbackBridge => (Route, "concrete-native-callback"),
            Self::NonSuspending => (Applicability, "source-pure-certificate"),
            Self::OptionReader => (Route, "concrete-option-continuation"),
            Self::OptionalStage => (Carrier, "optional-stage-finish"),
            Self::PhaseOwnership => (Tail, "phase-ownership"),
            Self::PrefixLoop => (Tail, "prefix-cursor"),
            Self::PrepareArgumentsRestore => (Tail, "arguments-restore"),
            Self::RebuildTraversal => (Tail, "traversal-rebuild"),
            Self::RetiredBoundary => (Applicability, "retirement-certificate"),
            Self::ReturnBoundary => (Carrier, "return-finish"),
            Self::ScalarFinish => (Carrier, "scalar-finish"),
            Self::ScopeCleanup => (Tail, "scope-cleanup"),
            Self::ShapeDecode => (Carrier, "shape-decode"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PolicyEntry {
    pub id: PolicyId,
    pub kind: PolicyKind,
    pub predicate: String,
    pub contexts: Vec<String>,
    pub variants: Vec<String>,
    pub historical_rows: usize,
    pub accepted_rows: usize,
}
pub type PolicyTable = BTreeMap<PolicyId, PolicyEntry>;

pub fn declarations(entries: Vec<PolicyEntry>) -> Checked<PolicyTable> {
    let mut result = BTreeMap::new();
    for entry in entries {
        let id = entry.id;
        if result.insert(id, entry).is_some() {
            return reject(
                Predicate::DuplicatePolicy,
                &format!("{id:?}"),
                "declaration",
            );
        }
    }
    Ok(result)
}

pub fn committed() -> Checked<PolicyTable> {
    let mut entries = Vec::new();
    for r in table(
        include_str!("../inventory/obligations/policies.tsv"),
        "policy\tkind\tpredicate\tcontexts\tvariants\thistorical_rows\taccepted_rows",
    )? {
        let id = PolicyId::parse(r[0])?;
        let (kind, predicate) = id.specification();
        if r[1] != kind.token() || r[2] != predicate {
            return reject(Predicate::InvalidPolicy, r[0], "kind-or-predicate");
        }
        entries.push(PolicyEntry {
            id,
            kind,
            predicate: r[2].into(),
            contexts: members(r[3])?,
            variants: members(r[4])?,
            historical_rows: number(r[5])?,
            accepted_rows: number(r[6])?,
        });
    }
    declarations(entries)
}

pub fn validate(population: &Population, policies: &PolicyTable) -> Checked<()> {
    let mut counts = BTreeMap::<PolicyId, usize>::new();
    for row in population.aliases.values() {
        let mut seen = BTreeSet::new();
        for token in &row.policies {
            let parsed = PolicyId::parse(token)?;
            if !seen.insert(parsed) {
                return reject(Predicate::DuplicateRequirement, &row.alias, token);
            }
            *counts.entry(parsed).or_default() += 1;
        }
        if !row.policies.windows(2).all(|p| p[0] < p[1]) {
            return reject(
                Predicate::InvalidPolicy,
                &row.alias,
                "unordered-policy-reasons",
            );
        }
    }
    for id in ALL {
        let e = policies.get(&id).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::MissingPolicy,
                format!("{id:?}"),
                "declaration",
            )]
        })?;
        if e.id != id
            || (e.kind, e.predicate.as_str()) != id.specification()
            || e.historical_rows == 0
        {
            return reject(
                Predicate::InvalidPolicy,
                &format!("{id:?}"),
                "specification",
            );
        }
        if counts.get(&id) != Some(&e.accepted_rows) {
            return reject(
                Predicate::InvalidPolicy,
                &format!("{id:?}"),
                "accepted-occurrences",
            );
        }
    }
    Ok(())
}
