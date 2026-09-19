//! Deterministic cycle witnesses over resolved structural ownership edges.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::effects::{self, Shape};
use super::model::*;
use super::resolve;

pub(super) fn discover(mut source: Source) -> Discovery {
    resolve::imports(&mut source);
    let shapes = effects::shapes(&source);
    let ports = effects::ports(&shapes);
    let mut edges: BTreeMap<Id, BTreeSet<Edge>> = BTreeMap::new();
    let mut unknowns = source.unknowns.clone();
    unknowns.extend(effects::opaque_boundaries(&source, &ports));
    for (owner, fields) in shapes {
        let outgoing = edges.entry(owner.clone()).or_default();
        for (field, shape) in fields {
            effects::walk(
                &shape,
                format!("{owner}.{}", field.path),
                &ports,
                &mut |shape, trail| match shape {
                    Shape::Nominal(target, _) => {
                        outgoing.insert(Edge {
                            target: target.clone(),
                            witness: trail.into(),
                        });
                    }
                    Shape::Unknown(kind, construct) => {
                        unknowns.insert(Unknown {
                            kind: *kind,
                            location: field.location.clone(),
                            construct: format!("{trail}: {construct}"),
                        });
                    }
                    _ => {}
                },
            );
        }
    }
    let cycles = edges
        .keys()
        .filter_map(|id| {
            cycle(&edges, id).map(|witness| Cycle {
                id: id.clone(),
                location: source.definitions[id].location.clone(),
                witness,
            })
        })
        .collect();
    let drops = resolve::drops(&source);
    Discovery {
        source,
        edges,
        cycles,
        drops,
        unknowns,
    }
}

fn cycle(edges: &BTreeMap<Id, BTreeSet<Edge>>, start: &str) -> Option<String> {
    let mut queue = VecDeque::from([(start.to_string(), Vec::<Edge>::new())]);
    let mut seen = BTreeSet::from([start.to_string()]);
    while let Some((owner, path)) = queue.pop_front() {
        for edge in edges.get(&owner).into_iter().flatten() {
            let mut next = path.clone();
            next.push(edge.clone());
            if edge.target == start {
                return Some(
                    next.iter()
                        .map(|e| format!("{} -> {}", e.witness, e.target))
                        .collect::<Vec<_>>()
                        .join(" ; "),
                );
            }
            if seen.insert(edge.target.clone()) {
                queue.push_back((edge.target.clone(), next));
            }
        }
    }
    None
}
