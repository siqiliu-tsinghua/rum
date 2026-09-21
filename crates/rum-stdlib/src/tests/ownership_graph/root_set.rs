//! Exercise the real crate/module loader without editing production source.

use super::super::{evidence, graph};
use super::{ROOTS, production_files, scan};

#[test]
fn syntax_root_discovers_added_recursive_type_and_restores_source() {
    const OLD_ROOTS: &[&str] = &["rum-core", "rum-runtime", "rum-stdlib"];
    const AST: &str = "crates/rum-syntax/src/ast.rs";
    const PROBE: &str = "rum_syntax::ast::ScannerRootSetProbe";

    let mut files = production_files(ROOTS);
    let scratchpad = files[AST].clone();
    files
        .get_mut(AST)
        .unwrap()
        .push_str("\nstruct ScannerRootSetProbe { next: Option<Box<ScannerRootSetProbe>> }\n");

    // Before the root change, collection never even offers syntax to the loader.
    let old_files = files
        .iter()
        .filter(|(path, _)| {
            OLD_ROOTS
                .iter()
                .any(|root| path.starts_with(&format!("crates/{root}/src/")))
        })
        .map(|(path, text)| (path.clone(), text.clone()))
        .collect();
    let before = graph::discover(scan(old_files, OLD_ROOTS));
    assert!(!before.source.definitions.contains_key(PROBE));
    assert!(!before.cycles.iter().any(|cycle| cycle.id == PROBE));
    println!("OLD ROOTS: {PROBE} invisible (no definition or self-reaching row)");

    let added = graph::discover(scan(files.clone(), ROOTS));
    let cycle = added.cycles.iter().find(|cycle| cycle.id == PROBE).unwrap();
    assert_eq!(cycle.location.file, AST);
    assert!(
        cycle
            .witness
            .contains("Option[Own]/arg0/std::boxed::Box[Own]/arg0")
    );
    println!("NEW ROOTS: {}", cycle.row());

    // Restore the exact source snapshot, never the git index. The probe is only
    // Rust source text: no recursive value is compiled, constructed, or dropped.
    files.insert(AST.into(), scratchpad);
    let restored = graph::discover(scan(files, ROOTS));
    assert!(!restored.source.definitions.contains_key(PROBE));
    assert!(!restored.cycles.iter().any(|cycle| cycle.id == PROBE));
    let without_probe: Vec<_> = evidence::cycle_rows(&added)
        .into_iter()
        .filter(|row| !row.starts_with(&format!("{PROBE}\t")))
        .collect();
    assert_eq!(without_probe, evidence::cycle_rows(&restored));
    assert_eq!(added.unknowns, restored.unknowns);
    println!("RESTORED: {PROBE} absent; all other cycles and unknowns unchanged");

    let requests = "rum_runtime::runtime::model::requests";
    for (owner, field, target) in [
        (
            format!("{requests}::SessionParseResponse"),
            "parsed",
            "rum_syntax::document::ParsedSource",
        ),
        (
            format!("{requests}::SessionIsCompleteResponse"),
            "diagnostic",
            "rum_syntax::document::StructuredSourceDiagnostic",
        ),
        (
            format!("{requests}::SessionRequestError"),
            "Syntax.0",
            "rum_syntax::document::StructuredSourceDiagnostic",
        ),
        (
            "rum_runtime::runtime::session::session_eval::TopLevelEvalInput".into(),
            "Parsed.0",
            "rum_syntax::ast::SyntaxExpr",
        ),
        (
            "rum_runtime::runtime::session::session_state::EvalError".into(),
            "Syntax.0",
            "rum_syntax::error::SyntaxError",
        ),
        (
            "rum_runtime::runtime::lowering::binary::left_chain::LeftChainLink".into(),
            "op",
            "rum_syntax::ast::BinaryOp",
        ),
    ] {
        assert!(
            restored.edges[&owner].iter().any(|edge| {
                edge.target == target && edge.witness.starts_with(&format!("{owner}.{field}"))
            }),
            "missing resolved ownership edge: {owner}.{field} -> {target}"
        );
    }
    println!("RESOLVED: all six former rum-syntax foreign storage rows are owning edges");
}
