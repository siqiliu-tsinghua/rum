//! Workspace source and lexical resolution orchestration.
use super::{
    classify::{self, Report},
    receipts::{RECEIPTS, rows},
    scan::{self, Facts},
    source,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(super) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}
pub(super) fn feature() -> &'static str {
    if cfg!(feature = "hash") {
        "default"
    } else {
        "no-default"
    }
}
pub(super) fn discover(root: &Path) -> (BTreeMap<String, String>, Facts, Report) {
    let files = scan::files(root);
    let facts = scan::scan(&files);
    eprintln!(
        "census walk: {} functions, {} uses, {} module visits",
        facts.functions.len(),
        facts.uses.len(),
        facts.visited.len()
    );
    let projected = scan::resolution_files(&files, &facts);
    let source_files: Vec<_> = projected
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    let roots: Vec<_> = files
        .keys()
        .filter(|p| p.ends_with("/src/lib.rs"))
        .map(|p| p.split('/').nth(1).unwrap())
        .collect();
    let source = source::fixture_with_roots(&source_files, &roots);
    eprintln!("census resolver: {} scopes", source.scopes.len());
    let mut report = classify::classify(source, &facts);
    let decoders = rows(RECEIPTS)
        .into_iter()
        .filter(|r| r[0] == "Decoder")
        .map(|r| r[4].to_owned())
        .collect();
    classify::wire_graph(&mut report, &facts, &decoders);
    (files, facts, report)
}
