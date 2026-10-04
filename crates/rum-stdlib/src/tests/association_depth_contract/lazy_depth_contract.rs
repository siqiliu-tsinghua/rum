//! Structural enforcement complements empirical depth tests: cheap recursive
//! destruction can survive depths that the descriptor Expr cannot yet exceed.
use super::recursion_inventory::recursive_edges_from_source;
use std::path::{Path, PathBuf};

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn lazy_self_reaching_rows(inventory: &str) -> Vec<&str> {
    inventory
        .lines()
        .filter(|row| row.contains("rum_stdlib::lazy::"))
        .collect()
}

#[test]
fn lazy_source_has_no_recursive_edges() {
    let root = workspace();
    let mut pending = vec![
        root.join("crates/rum-stdlib/src/lazy.rs"),
        root.join("crates/rum-stdlib/src/lazy"),
    ];
    let mut edges = Vec::new();
    let mut combined = String::new();
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            for entry in std::fs::read_dir(&path).unwrap() {
                pending.push(entry.unwrap().path());
            }
        } else if path.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
            let source = std::fs::read_to_string(&path).unwrap();
            edges.extend(recursive_edges_from_source(path.to_str().unwrap(), &source));
            combined.push_str(&without_inner_attributes(&source));
            combined.push('\n');
        }
    }
    // The scanner resolves callees within one source, so a cycle split across
    // the lazy modules is only visible when they are scanned as one unit.
    edges.extend(recursive_edges_from_source("lazy (all modules)", &combined));
    assert!(edges.is_empty(), "lazy recursion: {edges:?}");
}

fn without_inner_attributes(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with("//!") && !line.starts_with("#![")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn lazy_cross_module_recursion_falsifier() {
    let machine = "//! machine\nfn drive_pull() { drive_items(); }";
    let realization = "//! realization\nfn drive_items() { drive_pull(); }";
    assert!(recursive_edges_from_source("lazy/machine.rs", machine).is_empty());
    let combined = format!(
        "{}\n{}",
        without_inner_attributes(machine),
        without_inner_attributes(realization)
    );
    let edges = recursive_edges_from_source("lazy (all modules)", &combined);
    assert_eq!(edges.len(), 2, "both halves of the cycle: {edges:?}");
}

#[test]
fn lazy_recursive_helper_falsifier() {
    let source =
        "struct PullMachine; fn drive_pull(machine: &mut PullMachine) { drive_pull(machine); }";
    let edges = recursive_edges_from_source("lazy/machine.rs", source);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].caller, "drive_pull");
    assert_eq!(edges[0].callee, "drive_pull");
    assert_eq!(edges[0].count, 1);
    println!("recursive helper falsifier: {edges:?}");
}

#[test]
fn lazy_ownership_is_not_self_reaching() {
    let inventory = std::fs::read_to_string(
        workspace().join("crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv"),
    )
    .unwrap();
    assert_no_lazy_self_reaching(&inventory);
}

#[test]
#[should_panic(expected = "self-reaching lazy ownership")]
fn lazy_self_reaching_falsifier() {
    assert_no_lazy_self_reaching(
        "rum_stdlib::lazy::state::PullMachine\tlazy/state.rs\t1\trecursive owned child",
    );
}

fn assert_no_lazy_self_reaching(inventory: &str) {
    let rows = lazy_self_reaching_rows(inventory);
    assert!(rows.is_empty(), "self-reaching lazy ownership: {rows:?}");
}
