//! Participation coverage for the structurally discovered accumulator family.

use std::collections::BTreeSet;

use super::{graph, model::Discovery, source};

fn uncovered_chains(discovered: &Discovery) -> BTreeSet<String> {
    let mut uncovered = BTreeSet::new();
    for cycle in &discovered.cycles {
        // Also inspect every direct self-edge: one wrapped link must not hide
        // another raw link behind discovery's single canonical cycle witness.
        let witnesses = std::iter::once(cycle.witness.clone()).chain(
            discovered.edges[&cycle.id]
                .iter()
                .filter(|edge| edge.target == cycle.id)
                .map(|edge| format!("{} -> {}", edge.witness, edge.target)),
        );
        for witness in witnesses {
            if witness.contains("Option[Own]/arg0/std::sync::Arc")
                && !witness.contains("/rum_runtime::runtime::accumulator_chain::AccumulatorChain<")
            {
                uncovered.insert(format!(
                    "{} at {}:{}: ownership path: {witness}",
                    cycle.id, cycle.location.file, cycle.location.line,
                ));
            }
        }
    }
    uncovered
}

#[test]
fn every_self_reaching_arc_chain_participates() {
    // Participation only: the derive proves the link's type, the helper tests
    // prove the iterative drain mechanism, and this gate proves family coverage.
    let discovered = graph::discover(source::production());
    let uncovered = uncovered_chains(&discovered);
    println!("uncovered accumulator ownership paths: {}", uncovered.len());
    assert!(
        uncovered.is_empty(),
        "self-reaching Option<Arc<_>> chains must carry AccumulatorChain:\n{}",
        uncovered.into_iter().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn fresh_names_and_additional_self_edges_are_checked() {
    let discovered = graph::discover(source::fixture(&[(
        "crates/fixture/src/lib.rs",
        "use std::sync::Arc;\n\
         struct Fresh { first: Box<Self>, second: Option<Arc<Self>> }\n\
         struct AccumulatorChainPretender { next: Option<Arc<Self>> }\n\
         struct OtherShape { next: Option<Box<Self>> }\n",
    )]));
    let uncovered = uncovered_chains(&discovered);
    assert_eq!(uncovered.len(), 2, "{uncovered:#?}");
    assert!(uncovered.iter().any(|row| row.starts_with(
        "fixture::Fresh at crates/fixture/src/lib.rs:2: ownership path: fixture::Fresh.second/"
    )));
    assert!(uncovered.iter().any(|row| row.starts_with(
        "fixture::AccumulatorChainPretender at crates/fixture/src/lib.rs:3: ownership path:"
    )));
}
