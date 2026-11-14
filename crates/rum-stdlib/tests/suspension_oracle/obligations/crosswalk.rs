use std::collections::{BTreeMap, BTreeSet};

use super::{identity::*, imports::AcceptedInputs};

pub fn reconcile(
    graph: &DiscoveredGraph,
    accepted: &AcceptedInputs,
) -> Result<Population, Diagnostics> {
    let mut diagnostics = graph.diagnostics.clone();
    let population = &accepted.population;
    for (key, old) in &population.rows {
        match graph.source_rows.get(key) {
            None => diagnostics.push(Diagnostic::new(
                Kind::MissingAcceptedKey,
                key,
                old.aliases.join(";"),
            )),
            Some(now) => {
                if now.group_size != old.group_size
                    || (old.group_size > 1 && now.order != old.order)
                {
                    diagnostics.push(Diagnostic::new(
                        Kind::AmbiguousOccurrence,
                        key,
                        old.aliases.join(";"),
                    ));
                }
                if now.tokens != old.tokens {
                    diagnostics.push(Diagnostic::new(
                        Kind::PinMismatch,
                        key,
                        "token-digest-collision",
                    ));
                }
                if now.tail != old.tail {
                    diagnostics.push(Diagnostic::new(Kind::StaleTail, key, old.aliases.join(";")));
                }
                if now.destructors != old.destructors {
                    diagnostics.push(Diagnostic::new(
                        Kind::StaleDestructor,
                        key,
                        old.aliases.join(";"),
                    ));
                }
            }
        }
    }
    for (key, row) in &graph.rows {
        if !population.rows.contains_key(key) {
            diagnostics.push(Diagnostic::new(Kind::NewEdge, key, row.span.display()));
        }
    }
    for alias in population.aliases.values() {
        let Some(key) = &alias.target else {
            let ambiguous = accepted.crosswalks.iter().any(|r| {
                r[0] == "737-source-v1" && r[1] == alias.alias && r[3].starts_with("Ambiguous")
            });
            diagnostics.push(Diagnostic::new(
                if ambiguous {
                    Kind::AmbiguousOccurrence
                } else {
                    Kind::PortBlocked
                },
                &alias.alias,
                format!("unresolved-source:{}:{}", alias.function, alias.tokens),
            ));
            continue;
        };
        if let Some(row) = population.rows.get(key) {
            if !row.aliases.contains(&alias.alias)
                || !alias.policies.iter().all(|p| row.policies.contains(p))
            {
                diagnostics.push(Diagnostic::new(Kind::PolicyChange, &alias.alias, key));
            }
        } else {
            diagnostics.push(Diagnostic::new(Kind::MissingAcceptedKey, &alias.alias, key));
        }
    }
    let (_, transfer_errors) = transfer_fixed_point(population, graph);
    diagnostics.extend(transfer_errors);
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(population.clone())
    } else {
        Err(diagnostics)
    }
}

pub fn transfer_fixed_point(
    population: &Population,
    graph: &DiscoveredGraph,
) -> (BTreeMap<String, BTreeSet<String>>, Diagnostics) {
    let mut diagnostics = Vec::new();
    let mut closure = BTreeMap::<String, BTreeSet<String>>::new();
    let mut edges = BTreeMap::<String, BTreeSet<String>>::new();
    let mut source_aliases = BTreeMap::<String, BTreeSet<String>>::new();
    for alias in population.aliases.values() {
        if let Some(key) = &alias.target {
            if graph.source_rows.contains_key(key) && !alias.policies.is_empty() {
                closure.insert(alias.alias.clone(), BTreeSet::from([alias.alias.clone()]));
                source_aliases
                    .entry(key.clone())
                    .or_default()
                    .insert(alias.alias.clone());
            }
        }
    }
    for t in &population.transfers {
        if let Some(source) = &t.source {
            source_aliases
                .entry(source.clone())
                .or_default()
                .insert(t.alias.clone());
        }
        closure.entry(t.alias.clone()).or_default();
        if !t.retained && edges.insert(t.alias.clone(), t.callers.clone()).is_some() {
            diagnostics.push(Diagnostic::new(
                Kind::DuplicateInput,
                &t.alias,
                "transfer-node",
            ));
        }
    }
    for t in &population.transfers {
        let Some(row) = t.source.as_ref().and_then(|key| graph.source_rows.get(key)) else {
            diagnostics.push(Diagnostic::new(
                Kind::PortBlocked,
                &t.alias,
                "transfer-source-unresolved",
            ));
            continue;
        };
        if !t.retained {
            if !row.strict || row.tail != t.tail {
                diagnostics.push(Diagnostic::new(
                    Kind::StaleTail,
                    &t.alias,
                    "strict-transfer",
                ));
            }
            if row.destructors != t.destructors {
                diagnostics.push(Diagnostic::new(
                    Kind::StaleDestructor,
                    &t.alias,
                    "post-producer-drop",
                ));
            }
            let mut actual = BTreeSet::new();
            for (key, caller) in &graph.rows {
                if caller.edge.producer.key() == row.edge.consumer.function {
                    if let Some(aliases) = source_aliases.get(key) {
                        actual.extend(aliases.clone());
                    } else {
                        diagnostics.push(Diagnostic::new(Kind::ChangedCallerGraph, &t.alias, key));
                    }
                }
            }
            if actual != t.callers {
                diagnostics.push(Diagnostic::new(
                    Kind::ChangedCallerGraph,
                    &t.alias,
                    format!(
                        "expected={};actual={}",
                        t.callers.iter().cloned().collect::<Vec<_>>().join(";"),
                        actual.into_iter().collect::<Vec<_>>().join(";")
                    ),
                ));
            }
        }
        for caller in &t.callers {
            if !closure.contains_key(caller) {
                diagnostics.push(Diagnostic::new(
                    Kind::MissingTransferTarget,
                    &t.alias,
                    caller,
                ));
            }
        }
    }
    loop {
        let before = closure.clone();
        for (source, callers) in &edges {
            for caller in callers {
                if let Some(roots) = before.get(caller) {
                    closure
                        .entry(source.clone())
                        .or_default()
                        .extend(roots.clone());
                }
            }
        }
        if closure == before {
            break;
        }
    }
    for t in &population.transfers {
        let roots = &closure[&t.alias];
        if roots.is_empty() {
            fn cyclic(
                node: &str,
                edges: &BTreeMap<String, BTreeSet<String>>,
                path: &mut BTreeSet<String>,
            ) -> bool {
                if !path.insert(node.into()) {
                    return true;
                }
                let cycle = edges
                    .get(node)
                    .is_some_and(|next| next.iter().any(|n| cyclic(n, edges, path)));
                path.remove(node);
                cycle
            }
            if cyclic(&t.alias, &edges, &mut BTreeSet::new()) {
                diagnostics.push(Diagnostic::new(
                    Kind::RootlessCycle,
                    &t.alias,
                    "no-concrete-root-or-approved-cycle-proof",
                ));
            } else if t.callers.is_empty() && !t.retained {
                diagnostics.push(Diagnostic::new(
                    Kind::MissingTransferTarget,
                    &t.alias,
                    "empty-indexed-caller-set",
                ));
            }
        } else if roots != &t.roots {
            diagnostics.push(Diagnostic::new(
                Kind::ChangedCallerGraph,
                &t.alias,
                "root-fixed-point-changed",
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    (closure, diagnostics)
}
