//! Counted sealed rows. No ordinal or invented tuple substitutes for a frozen key.
//! A CM1 source-policy identity is retained when resolved; unresolved aliases
//! remain explicit identities awaiting a cross-walk. Counts are not receipts.
use std::collections::{BTreeMap, BTreeSet};

use super::{
    applicability::{self, Applicability},
    identity,
    policy::*,
    routes::{self, Route, Routes},
};

pub type RowKey = String;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ObligationKey {
    Source(identity::ObligationKey),
    PortBlocked { row: RowKey, policy: PolicyId },
}

#[derive(Clone, Debug)]
pub struct RequirementSet {
    pub row: RowKey,
    pub policy: PolicyId,
    pub contexts: Vec<String>,
    pub variants: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Count {
    Frozen {
        keys: usize,
        encoding: usize,
        digest: String,
    },
    Recorded(usize),
    PortBlocked,
    PriorAccepted(Option<usize>),
}

#[derive(Clone, Debug)]
pub struct RowRequirements {
    pub alias: identity::AliasRecord,
    pub policies: Vec<PolicyId>,
    pub count: Count,
    pub routes: Vec<Route>,
}

#[derive(Clone, Debug, Default)]
pub struct RequiredBag {
    pub entries: BTreeMap<ObligationKey, RequirementSet>,
    pub rows: BTreeMap<RowKey, RowRequirements>,
    pub diagnostics: Diagnostics,
}

pub const OWNED: &str = include_str!("../inventory/obligations/owned.tsv");
pub const OWNED_HEADER: &str =
    "record\tid\towner\tkind\tstate\tsubject\tdisposition\tcount\tbar\tprovenance";

fn recorded_counts() -> Checked<BTreeMap<&'static str, &'static str>> {
    let mut counts = BTreeMap::new();
    for r in table(OWNED, OWNED_HEADER)? {
        match r[0] {
            "row" => {
                if counts.insert(r[1], r[7]).is_some() {
                    return reject(Predicate::DuplicateRequirement, r[1], "recorded-count");
                }
            }
            "state" => (),
            kind => return reject(Predicate::LedgerMismatch, r[1], kind),
        }
    }
    Ok(counts)
}

pub fn counted(
    declarations: Vec<(ObligationKey, RequirementSet)>,
) -> Checked<BTreeMap<ObligationKey, RequirementSet>> {
    let mut result = BTreeMap::new();
    for (key, value) in declarations {
        // Check multiplicity at insertion, before any policy union/deduplication.
        if result.contains_key(&key) {
            return reject(
                Predicate::DuplicateRequirement,
                &value.row,
                &format!("{key:?}"),
            );
        }
        result.insert(key, value);
    }
    Ok(result)
}

pub fn requirements(
    population: &identity::Population,
    policies: &PolicyTable,
    applicability: &Applicability,
    routes: &Routes,
) -> Checked<RequiredBag> {
    super::policy::validate(population, policies)?;
    routes::validate(routes)?;
    let mut counts = recorded_counts()?;
    for r in table(OWNED, OWNED_HEADER)? {
        match r[0] {
            "row" => {
                let alias = population.aliases.get(r[1]).ok_or_else(|| {
                    vec![Diagnostic::new(
                        Predicate::RequirementSealMismatch,
                        r[1],
                        "unaccepted-row",
                    )]
                })?;
                let bar = if alias.verdict == "Residual" {
                    "c2-residual-evidence"
                } else if alias.cohort == "g13" {
                    "g13-step-promotion"
                } else {
                    "accepted-disposition"
                };
                if r[2] != alias.owner
                    || r[3] != alias.cohort
                    || r[4] != "-"
                    || r[5] != alias.target.as_deref().unwrap_or("-")
                    || r[6] != format!("{}/{}", alias.disposition, alias.verdict)
                    || r[8] != bar
                    || r[9] != alias.provenance
                {
                    return reject(Predicate::LedgerMismatch, r[1], "owned-record-CM1-identity");
                }
            }
            "state" => (),
            kind => return reject(Predicate::LedgerMismatch, r[1], kind),
        }
    }
    let mut bag = RequiredBag::default();
    let mut declarations = Vec::new();
    let mut used_routes = 0;
    for alias in population.aliases.values() {
        let rule = applicability::for_row(alias, applicability, policies)?;
        let row = &alias.alias;
        let recorded = counts.remove(row.as_str()).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::PortBlocked,
                row,
                "missing-recorded-count",
            )]
        })?;
        let bindings: Vec<_> = routes
            .bindings
            .iter()
            .filter(|r| &r.row == row)
            .cloned()
            .collect();
        used_routes += bindings.len();
        let count = match alias.cohort.as_str() {
            "c2-original" | "c2-added" => {
                if bindings.is_empty() {
                    return reject(Predicate::MissingRoute, row, "frozen-consumer-label");
                }
                let keys = bindings.iter().map(Route::keys).sum::<usize>();
                let encoding = bindings.iter().map(Route::encoding).sum::<usize>();
                if keys != alias.keys
                    || encoding != alias.encoding
                    || number(recorded)? != keys
                    || bindings.iter().any(|r| r.row_digest != alias.key_digest)
                {
                    return reject(
                        Predicate::RequirementSealMismatch,
                        row,
                        "CM1-count-encoding-digest",
                    );
                }
                let families: BTreeSet<_> = bindings
                    .iter()
                    .flat_map(|r| r.leaves.keys())
                    .map(|leaf| leaf.split_once('.').unwrap().0.to_string())
                    .collect();
                let generic: BTreeSet<_> = rule.contexts.iter().cloned().collect();
                for context in generic.symmetric_difference(&families) {
                    bag.diagnostics.push(Diagnostic::new(
                        Predicate::PortBlocked,
                        row,
                        format!("source-specific-policy-applicability:{context}"),
                    ));
                }
                // Frozen Twice does not encode which of these distinct source
                // phases applies. Preserve the gap; never double the key bag.
                for phase in &routes.phases {
                    bag.diagnostics.push(Diagnostic::new(
                        Predicate::PortBlocked,
                        row,
                        format!("Twice:{phase}:source-phase-applicability"),
                    ));
                }
                Count::Frozen {
                    keys,
                    encoding,
                    digest: alias.key_digest.clone(),
                }
            }
            "g13" => {
                if !bindings.is_empty() {
                    return reject(
                        Predicate::RequirementSealMismatch,
                        row,
                        "unexpected-C2-route",
                    );
                }
                if recorded == "NotYetBound" {
                    bag.diagnostics.push(Diagnostic::new(
                        Predicate::PortBlocked,
                        row,
                        ".temp/brief-621/continuation3:NotYetBound",
                    ));
                    Count::PortBlocked
                } else {
                    let derived = routes::policy_count(&rule.contexts, &rule.variants)?;
                    if derived != number(recorded)? {
                        return reject(
                            Predicate::RequirementSealMismatch,
                            row,
                            "G13-recorded-policy-expansion",
                        );
                    }
                    // Recover a count, not absent descriptor/phase bindings.
                    bag.diagnostics.push(Diagnostic::new(
                        Predicate::PortBlocked,
                        row,
                        ".temp/brief-621/continuation3:tuple-identities",
                    ));
                    Count::Recorded(derived)
                }
            }
            "prior" => Count::PriorAccepted(if recorded == "NotRecorded" {
                None
            } else {
                Some(number(recorded)?)
            }),
            kind => return reject(Predicate::MissingApplicability, row, kind),
        };
        for id in &rule.policies {
            let key = if let Some(source) = &alias.target {
                let source = population.rows.get(source).ok_or_else(|| {
                    vec![Diagnostic::new(
                        Predicate::PortBlocked,
                        row,
                        "CM1-source-row",
                    )]
                })?;
                ObligationKey::Source(identity::ObligationKey {
                    edge: source.edge.clone(),
                    policy: format!("{id:?}"),
                })
            } else {
                ObligationKey::PortBlocked {
                    row: row.clone(),
                    policy: *id,
                }
            };
            declarations.push((
                key,
                RequirementSet {
                    row: row.clone(),
                    policy: *id,
                    contexts: policies[id].contexts.clone(),
                    variants: policies[id].variants.clone(),
                },
            ));
        }
        if alias.target.is_none() {
            bag.diagnostics.push(Diagnostic::new(
                Predicate::PortBlocked,
                row,
                "CM1-unresolved-source-identity",
            ));
        }
        bag.rows.insert(
            row.clone(),
            RowRequirements {
                alias: alias.clone(),
                policies: rule.policies.clone(),
                count,
                routes: bindings,
            },
        );
    }
    if !counts.is_empty() || used_routes != routes.bindings.len() {
        return reject(
            Predicate::RequirementSealMismatch,
            "population",
            "extra-count-or-route-row",
        );
    }
    for (row, route) in &routes.gaps {
        bag.diagnostics.push(Diagnostic::new(
            Predicate::PortBlocked,
            row,
            format!("concrete-route-inventory:{route}"),
        ));
    }
    bag.entries = counted(declarations)?;
    validate_bag(&bag)?;
    bag.diagnostics.sort();
    Ok(bag)
}

pub fn validate_bag(bag: &RequiredBag) -> Checked<()> {
    let accepted: BTreeMap<_, _> = table(super::imports::ALIASES, super::imports::ALIAS_HEADER)?
        .into_iter()
        .map(|r| (r[0], r))
        .collect();
    let policies = super::policy::committed()?;
    let mut counts = recorded_counts()?;
    let routes = routes::committed()?;
    let mut projections = BTreeMap::<&str, Vec<&Route>>::new();
    for route in &routes.bindings {
        projections.entry(&route.row).or_default().push(route);
    }
    let mut reasons = BTreeMap::<(&str, PolicyId), usize>::new();
    for (key, entry) in &bag.entries {
        let row = bag.rows.get(&entry.row).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::RequirementSealMismatch,
                &entry.row,
                "orphaned-policy-obligation",
            )]
        })?;
        let id_matches = match key {
            ObligationKey::Source(source) => {
                row.alias.target.as_deref() == Some(source.edge.key().as_str())
                    && source.policy == format!("{:?}", entry.policy)
            }
            ObligationKey::PortBlocked { row: alias, policy } => {
                row.alias.target.is_none() && alias == &entry.row && policy == &entry.policy
            }
        };
        if !id_matches
            || !row.alias.policies.contains(&format!("{:?}", entry.policy))
            || entry.contexts != policies[&entry.policy].contexts
            || entry.variants != policies[&entry.policy].variants
        {
            return reject(
                Predicate::RequirementSealMismatch,
                &entry.row,
                "policy-identity-or-requirement",
            );
        }
        *reasons.entry((&entry.row, entry.policy)).or_default() += 1;
    }
    let mut rows = 0;
    let mut keys = 0;
    let mut encoding = 0;
    let mut g13 = 0;
    let mut recorded_rows = 0;
    let mut recorded_keys = 0;
    let mut blocked = 0;
    let mut prior = 0;
    for (row, req) in &bag.rows {
        let original = accepted.get(row.as_str()).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::RequirementSealMismatch,
                row,
                "unaccepted-row",
            )]
        })?;
        if row != &req.alias.alias
            || req.alias.target.as_deref().unwrap_or("-") != original[2]
            || req.alias.cohort != original[3]
            || req.alias.owner != original[4]
            || req.alias.disposition != original[5]
            || req.alias.verdict != original[6]
            || req.alias.policies.join(";") != original[7]
            || req.alias.keys != number(original[8])?
            || req.alias.encoding != number(original[9])?
            || req.alias.key_digest != original[15]
        {
            return reject(Predicate::RequirementSealMismatch, row, "CM1-alias-seal");
        }
        if req.routes.iter().collect::<Vec<_>>()
            != projections.remove(row.as_str()).unwrap_or_default()
        {
            return reject(Predicate::RouteSealMismatch, row, "frozen-route-projection");
        }
        let recorded = counts.remove(row.as_str()).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::RequirementSealMismatch,
                row,
                "missing-recorded-count",
            )]
        })?;
        let count_matches = match (req.alias.cohort.as_str(), &req.count) {
            ("c2-original" | "c2-added", Count::Frozen { keys, .. }) => *keys == number(recorded)?,
            ("g13", Count::Recorded(n)) => recorded != "NotYetBound" && *n == number(recorded)?,
            ("g13", Count::PortBlocked) => recorded == "NotYetBound",
            ("prior", Count::PriorAccepted(None)) => recorded == "NotRecorded",
            ("prior", Count::PriorAccepted(Some(n))) => {
                recorded != "NotRecorded" && *n == number(recorded)?
            }
            _ => false,
        };
        if !count_matches {
            return reject(
                Predicate::RequirementSealMismatch,
                row,
                "recorded-row-count",
            );
        }
        match &req.count {
            Count::Frozen {
                keys: n,
                encoding: e,
                digest,
            } => {
                if (*n, *e, digest) != (req.alias.keys, req.alias.encoding, &req.alias.key_digest)
                    || *n != req.routes.iter().map(Route::keys).sum::<usize>()
                    || *e != req.routes.iter().map(Route::encoding).sum::<usize>()
                {
                    return reject(Predicate::RequirementSealMismatch, row, "frozen-row-seal");
                }
                rows += 1;
                keys += n;
                encoding += e;
            }
            Count::Recorded(n) => {
                g13 += 1;
                recorded_rows += 1;
                recorded_keys += n;
            }
            Count::PortBlocked => {
                g13 += 1;
                blocked += 1;
            }
            Count::PriorAccepted(_) => prior += 1,
        }
        let expected_policies = req
            .alias
            .policies
            .iter()
            .map(|p| PolicyId::parse(p))
            .collect::<Checked<Vec<_>>>()?;
        if req.policies != expected_policies
            || req
                .policies
                .iter()
                .any(|id| reasons.get(&(row.as_str(), *id)) != Some(&1))
        {
            return reject(
                Predicate::DuplicateRequirement,
                row,
                "policy-obligation-multiplicity",
            );
        }
    }
    if !counts.is_empty() || !projections.is_empty() {
        return reject(
            Predicate::RequirementSealMismatch,
            "population",
            "unconsumed-count-or-route",
        );
    }
    if (
        bag.rows.len(),
        rows,
        keys,
        encoding,
        g13,
        recorded_rows,
        recorded_keys,
        blocked,
        prior,
    ) != (2204, 1042, 78330, 18840, 964, 44, 3025, 920, 198)
    {
        return reject(
            Predicate::RequirementSealMismatch,
            "population",
            "accepted-bag-totals",
        );
    }
    Ok(())
}
