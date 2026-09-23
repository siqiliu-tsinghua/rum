//! Rejection enumeration, never admission or a production coverage certificate.

mod report;
mod tests;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::model::*;
use super::{catalog, effects, obligations, regions, resolve};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Definition {
    pub id: Id,
    pub site: Site,
}

#[derive(Clone)]
pub(super) struct Root {
    pub name: String,
    pub site: Site,
    pub target: Result<Id, String>,
    pub spelling: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Row {
    pub root: String,
    pub definition: Id,
    pub diagnostic: Diagnostic,
}

// The key intentionally excludes both the root and its witness path. Incidence
// is a separate quantity: a helper reached by 50 roots is still one edge site.
pub(super) type Key = (Id, Site, Kind, Option<Id>, String);

impl Row {
    pub fn key(&self) -> Key {
        let d = &self.diagnostic;
        (
            self.definition.clone(),
            d.site.clone(),
            d.kind,
            d.target.clone(),
            d.reason.clone(),
        )
    }
}

pub(super) struct Census {
    pub population: &'static str,
    pub roots: BTreeMap<String, Root>,
    pub visited: BTreeMap<Definition, BTreeSet<String>>,
    pub rows: Vec<Row>,
    pub first: Vec<Row>,
    pub first_ok: BTreeSet<String>,
    pub resolved: BTreeSet<regions::ResolvedEdge>,
}

impl Census {
    fn new(population: &'static str, roots: Vec<Root>) -> Self {
        let count = roots.len();
        let roots: BTreeMap<_, _> = roots.into_iter().map(|r| (r.name.clone(), r)).collect();
        assert_eq!(roots.len(), count, "distinct root attachments");
        Self {
            population,
            roots,
            visited: BTreeMap::new(),
            rows: Vec::new(),
            first: Vec::new(),
            first_ok: BTreeSet::new(),
            resolved: BTreeSet::new(),
        }
    }

    fn visit(
        &mut self,
        source: &Source,
        f: &Function,
        root: &str,
        witness: &[Id],
    ) -> Vec<Function> {
        self.visited
            .entry(Definition {
                id: f.id.clone(),
                site: f.site.clone(),
            })
            .or_default()
            .insert(root.into());
        let edges = regions::definition_edges(source, f);
        for mut diagnostic in edges.rejected {
            diagnostic.witness = witness.to_vec();
            self.rows.push(Row {
                root: root.into(),
                definition: f.id.clone(),
                diagnostic,
            });
        }
        let targets = edges
            .resolved
            .iter()
            .filter_map(|e| {
                edges
                    .methods
                    .get(&e.site)
                    .map(|m| m.function.clone())
                    .or_else(|| resolve::function(source, &e.target).ok().cloned())
            })
            .collect();
        self.resolved.extend(edges.resolved);
        targets
    }

    fn first_failure(&mut self, source: &Source, root: &Root) {
        let result = match &root.target {
            Ok(id) => effects::check(source, id, &super::contracts()),
            Err(reason) => Err(root_error(root, reason)),
        };
        match result {
            Ok(_) => {
                self.first_ok.insert(root.name.clone());
            }
            Err(mut diagnostic) => {
                // effects::check uses <root> for failed root lookup. Locate the
                // reporting row without changing the checker's admission logic.
                if diagnostic.site.file == "<root>" {
                    diagnostic.site = root.site.clone();
                    diagnostic.witness = vec![root.name.clone()];
                }
                self.first.push(Row {
                    root: root.name.clone(),
                    definition: root
                        .target
                        .clone()
                        .unwrap_or_else(|_| "<unresolved-attachment>".into()),
                    diagnostic,
                });
            }
        }
    }
}

fn root_error(root: &Root, reason: &str) -> Diagnostic {
    Diagnostic {
        kind: Kind::RET002,
        site: root.site.clone(),
        target: Some(
            root.target
                .clone()
                .unwrap_or_else(|_| root.spelling.clone()),
        ),
        witness: vec![root.name.clone()],
        reason: reason.into(),
    }
}

pub(super) fn definitions(source: &Source) -> BTreeSet<Definition> {
    source
        .functions
        .values()
        .flatten()
        .map(|f| Definition {
            id: f.id.clone(),
            site: f.site.clone(),
        })
        .collect()
}

pub(super) fn all(source: &Source) -> Census {
    let roots: Vec<_> = source
        .functions
        .values()
        .flatten()
        .map(|f| Root {
            name: format!("{}@{}", f.id, f.site.display()),
            site: f.site.clone(),
            target: Ok(f.id.clone()),
            spelling: f.id.clone(),
        })
        .collect();
    let mut report = Census::new("EDGE_ALL", roots);
    for f in source.functions.values().flatten() {
        let name = format!("{}@{}", f.id, f.site.display());
        let root = report.roots[&name].clone();
        if let Err(reason) = resolve::function(source, &f.id) {
            report.rows.push(Row {
                root: name.clone(),
                definition: f.id.clone(),
                diagnostic: root_error(&root, &reason),
            });
        }
        // Even a rejected/conditional definition has a body to enumerate in
        // EDGE_ALL. This says nothing about its semantic reachability.
        report.visit(source, f, &name, &[f.id.clone()]);
        report.first_failure(source, &root);
    }
    report
}

pub(super) fn n_roots(source: &Source) -> Vec<Root> {
    let classes = obligations::census(source);
    let routes = catalog::attachments(source);
    let mut roots = Vec::new();
    for (name, class) in classes {
        if class != "N" {
            continue;
        }
        let attachments = routes.get(&name).expect("N has a live attachment");
        assert!(!attachments.is_empty(), "N:{name} has no attachments");
        for attachment in attachments {
            roots.push(Root {
                name: format!("N:{name}@{}", attachment.site.display()),
                site: attachment.site.clone(),
                target: attachment.target.clone(),
                spelling: attachment.spelling.clone(),
            });
        }
    }
    roots
}

pub(super) fn frontier(source: &Source, roots: Vec<Root>) -> Census {
    let mut report = Census::new("EDGE_N_FRONTIER", roots.clone());
    for root in roots {
        report.first_failure(source, &root);
        let id = match &root.target {
            Ok(id) => id,
            Err(reason) => {
                report.rows.push(Row {
                    root: root.name.clone(),
                    definition: "<unresolved-attachment>".into(),
                    diagnostic: root_error(&root, reason),
                });
                continue;
            }
        };
        let mut pending = VecDeque::from([(
            resolve::function(source, id).cloned(),
            vec![root.name.clone(), id.clone()],
        )]);
        let mut seen = BTreeSet::new();
        while let Some((function, witness)) = pending.pop_front() {
            match function {
                Ok(f) => {
                    if !seen.insert(super::methods::key(&f)) {
                        continue;
                    }
                    for target in report.visit(source, &f, &root.name, &witness) {
                        let mut witness = witness.clone();
                        witness.push(target.id.clone());
                        pending.push_back((Ok(target), witness));
                    }
                }
                Err(reason) => {
                    let mut diagnostic = root_error(&root, &reason);
                    diagnostic.target = Some(id.clone());
                    diagnostic.witness = witness;
                    report.rows.push(Row {
                        root: root.name.clone(),
                        definition: id.clone(),
                        diagnostic,
                    });
                }
            }
        }
    }
    report
}

pub(super) fn validate(
    report: &Census,
    expected: &BTreeSet<Definition>,
    roots: &[Root],
) -> Result<(), String> {
    let actual: BTreeSet<_> = report.visited.keys().cloned().collect();
    if &actual != expected {
        return Err("census omitted or invented a production definition".into());
    }
    let wanted: BTreeSet<_> = roots.iter().map(|r| r.name.clone()).collect();
    if report.roots.keys().cloned().collect::<BTreeSet<_>>() != wanted {
        return Err("census omitted or invented a root attachment".into());
    }
    let visited_roots: BTreeSet<_> = report.visited.values().flatten().collect();
    let rejected_roots: BTreeSet<_> = report.rows.iter().map(|r| &r.root).collect();
    for root in roots {
        let visited = visited_roots.contains(&root.name);
        let rejected = rejected_roots.contains(&root.name);
        if !visited && !rejected {
            return Err(format!("root vanished: {}", root.name));
        }
        if root.target.is_err()
            && !report.rows.iter().any(|r| {
                r.root == root.name
                    && r.diagnostic.site == root.site
                    && Some(&r.diagnostic.reason) == root.target.as_ref().err()
            })
        {
            return Err(format!(
                "unresolved root lost its located rejection: {}",
                root.name
            ));
        }
    }
    if report.first.len() + report.first_ok.len() != roots.len() {
        return Err("first-failure population differs from roots".into());
    }
    Ok(())
}
