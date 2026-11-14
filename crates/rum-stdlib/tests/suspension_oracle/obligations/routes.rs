//! Frozen count projections, not tuple bindings or claims of concrete-route closure.
use std::collections::{BTreeMap, BTreeSet};

use super::policy::*;

pub const LEAVES: [&str; 22] = [
    "consumer_routes.propagate-Automatic",
    "consumer_routes.propagate-False",
    "consumer_routes.propagate-True",
    "consumer_routes.route-original",
    "error_before_tail.last-pause-throw",
    "guest_heads.tail-nested",
    "guest_heads.tail-none",
    "guest_heads.tail-some-empty",
    "guest_heads.tail-some5",
    "held_and_splice.held-evaluate",
    "held_and_splice.tail-hold",
    "held_and_splice.tail-nothing",
    "held_and_splice.tail-sequence",
    "held_and_splice.unevaluated",
    "phase_observation.iteration-budget",
    "phase_observation.match-depth",
    "phase_observation.trace",
    "prefix_and_tail.log",
    "return_boundary.last-pause-return",
    "return_boundary.loop-store-reset",
    "same_type_error_finish.explicit-cycle-controls-required",
    "value.base",
];
pub const VARIANTS: [&str; 5] = ["AlreadyResumed", "Error", "Middle", "Nested", "Twice"];
pub const PHASES: [&str; 2] = ["projection_then_value", "two_operands"];
pub const GENERIC_GAPS: [(&str, &str); 5] = [
    ("P0403", "DefinitionResumeRoute"),
    ("P0508", "AccountingMode"),
    ("P1147", "OptionReadContinuation<Option<Expr>>"),
    ("P1155", "OptionReadContinuation<Expr>"),
    ("P2604", "HostSession::guard<T>"),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Route {
    pub row: String,
    pub consumer: String,
    pub leaves: BTreeMap<String, usize>,
    pub variants: BTreeMap<String, usize>,
    pub key_digest: String,
    pub row_digest: String,
}

impl Route {
    pub fn keys(&self) -> usize {
        self.leaves.values().sum()
    }
    pub fn encoding(&self) -> usize {
        self.leaves
            .iter()
            .filter(|(leaf, _)| leaf.starts_with("guest_heads."))
            .map(|(_, n)| n)
            .sum()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Routes {
    pub leaves: BTreeMap<String, String>,
    pub bindings: Vec<Route>,
    pub phases: BTreeSet<String>,
    pub gaps: BTreeSet<(String, String)>,
}

fn counts(cell: &str) -> Checked<BTreeMap<String, usize>> {
    let mut out = BTreeMap::new();
    for pair in members(cell)? {
        let (key, value) = pair.rsplit_once('=').ok_or_else(|| {
            vec![Diagnostic::new(
                Predicate::PortBlocked,
                &pair,
                "count-projection",
            )]
        })?;
        if out.insert(key.into(), number(value)?).is_some() {
            return reject(Predicate::DuplicateRequirement, key, "count-projection");
        }
    }
    Ok(out)
}

pub fn committed() -> Checked<Routes> {
    let mut out = Routes::default();
    for r in table(
        include_str!("../inventory/obligations/routes.tsv"),
        "record\trow\troute\tclass\tleaves\tvariants\tkey_digest\trow_digest",
    )? {
        match r[0] {
            "leaf" => {
                if out.leaves.insert(r[2].into(), r[3].into()).is_some() {
                    return reject(Predicate::DuplicateRequirement, r[2], "leaf");
                }
            }
            "binding" => {
                if r[3] != "frozen-consumer-label" {
                    return reject(Predicate::RouteSealMismatch, r[1], "route-class");
                }
                out.bindings.push(Route {
                    row: r[1].into(),
                    consumer: r[2].into(),
                    leaves: counts(r[4])?,
                    variants: counts(r[5])?,
                    key_digest: r[6].into(),
                    row_digest: r[7].into(),
                });
            }
            "phase" => {
                if r[3] != "Twice" || r[6] != "PortBlocked" || !out.phases.insert(r[2].into()) {
                    return reject(Predicate::RouteSealMismatch, r[2], "unrecovered-phase-plan");
                }
            }
            "gap" => {
                if r[6] != "PortBlocked" || !out.gaps.insert((r[1].into(), r[2].into())) {
                    return reject(Predicate::RouteSealMismatch, r[1], "generic-route-gap");
                }
            }
            kind => return reject(Predicate::PortBlocked, kind, "route-record-kind"),
        }
    }
    Ok(out)
}

pub fn validate(routes: &Routes) -> Checked<()> {
    let expected: BTreeSet<_> = LEAVES.iter().map(|s| (*s).to_string()).collect();
    let actual: BTreeSet<_> = routes.leaves.keys().cloned().collect();
    if actual != expected {
        return reject(
            Predicate::MissingLeaf,
            "leaf-table",
            &format!(
                "{:?}",
                actual.symmetric_difference(&expected).collect::<Vec<_>>()
            ),
        );
    }
    if routes.phases != PHASES.iter().map(|s| (*s).to_string()).collect()
        || routes.gaps
            != GENERIC_GAPS
                .iter()
                .map(|(r, c)| (r.to_string(), c.to_string()))
                .collect()
    {
        return reject(
            Predicate::MissingRoute,
            "source-plan",
            "concrete-routes-or-Twice-phases",
        );
    }
    let mut seen = BTreeSet::new();
    for route in &routes.bindings {
        if !seen.insert((&route.row, &route.consumer)) {
            return reject(Predicate::DuplicateRequirement, &route.row, &route.consumer);
        }
        if route.consumer.is_empty()
            || route.keys() == 0
            || route.keys() != route.variants.values().sum::<usize>()
            || route.key_digest.len() != 64
            || route.row_digest.len() != 64
        {
            return reject(Predicate::RouteSealMismatch, &route.row, &route.consumer);
        }
        for leaf in route.leaves.keys() {
            let Some(family) = routes.leaves.get(leaf) else {
                return reject(Predicate::MissingLeaf, &route.row, leaf);
            };
            if leaf.split_once('.').map(|v| v.0) != Some(family.as_str()) {
                return reject(Predicate::RouteSealMismatch, &route.row, leaf);
            }
        }
        if route
            .variants
            .keys()
            .any(|v| !VARIANTS.contains(&v.as_str()))
        {
            return reject(Predicate::RouteSealMismatch, &route.row, "variant");
        }
    }
    Ok(())
}

// Standard leaf expansion is used only for recorded G13 counts. NotYetBound
// never takes this branch: a policy upper bound is not a missing requirement.
pub fn policy_count(contexts: &[String], variants: &[String]) -> Checked<usize> {
    let mut leaves = BTreeSet::<&&str>::new();
    for context in contexts {
        let selected: Vec<_> = LEAVES
            .iter()
            .filter(|leaf| leaf.starts_with(&format!("{context}.")))
            .filter(|leaf| {
                ![
                    "consumer_routes.propagate-Automatic",
                    "consumer_routes.propagate-False",
                    "consumer_routes.propagate-True",
                    "phase_observation.iteration-budget",
                    "phase_observation.match-depth",
                ]
                .contains(leaf)
            })
            .collect();
        if selected.is_empty() {
            return reject(Predicate::MissingLeaf, context, "policy-family");
        }
        leaves.extend(selected);
    }
    Ok(leaves.len() * variants.len())
}
