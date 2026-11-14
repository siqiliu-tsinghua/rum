//! Reviewed policy combinations are row kinds, independent of witness factories.
use std::collections::{BTreeMap, BTreeSet};

use super::{identity::AliasRecord, policy::*};

#[derive(Clone, Debug)]
pub struct Rule {
    pub policies: Vec<PolicyId>,
    pub contexts: Vec<String>,
    pub variants: Vec<String>,
    pub predicate: String,
}
pub type Applicability = BTreeMap<String, Rule>;

pub fn committed() -> Checked<Applicability> {
    let mut result = BTreeMap::new();
    for r in table(
        include_str!("../inventory/obligations/applicability.tsv"),
        "row_kind\tpolicies\tcontexts\tvariants\tpredicate",
    )? {
        let rule = Rule {
            policies: members(r[1])?
                .iter()
                .map(|p| PolicyId::parse(p))
                .collect::<Checked<_>>()?,
            contexts: members(r[2])?,
            variants: members(r[3])?,
            predicate: r[4].into(),
        };
        if result.insert(r[0].into(), rule).is_some() {
            return reject(Predicate::DuplicateRequirement, r[0], "applicability");
        }
    }
    Ok(result)
}

pub fn for_row<'a>(
    row: &AliasRecord,
    table: &'a Applicability,
    policies: &PolicyTable,
) -> Checked<&'a Rule> {
    let kind = row.policies.join(";");
    let rule = table.get(&kind).ok_or_else(|| {
        vec![Diagnostic::new(
            Predicate::MissingApplicability,
            &row.alias,
            &kind,
        )]
    })?;
    let required = row
        .policies
        .iter()
        .map(|p| PolicyId::parse(p))
        .collect::<Checked<Vec<_>>>()?;
    let mut contexts = BTreeSet::new();
    let mut variants = BTreeSet::new();
    for id in &required {
        let entry = policies.get(id).ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::MissingPolicy,
                &row.alias,
                format!("{id:?}"),
            )]
        })?;
        contexts.extend(entry.contexts.iter().cloned());
        variants.extend(entry.variants.iter().cloned());
    }
    let predicate = if contexts.is_empty() {
        "source-certificate"
    } else {
        "counted-pausing-plan"
    };
    if rule.policies != required
        || rule.contexts != contexts.into_iter().collect::<Vec<_>>()
        || rule.variants != variants.into_iter().collect::<Vec<_>>()
        || rule.predicate != predicate
    {
        return reject(Predicate::ApplicabilityMismatch, &row.alias, &kind);
    }
    Ok(rule)
}
