use std::fs;
use std::path::{Path, PathBuf};

fn collect_production_rust_sources(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("sealing gate should read source directories") {
            collect_production_rust_sources(
                &entry.expect("sealing gate should read entries").path(),
                files,
            );
        }
        return;
    }

    let normalized = path.display().to_string().replace('\\', "/");
    if path.extension().and_then(|extension| extension.to_str()) == Some("rs")
        && !normalized.contains("/tests/")
    {
        files.push(path.to_path_buf());
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runtime crate should live under the workspace crates directory")
        .to_path_buf()
}

fn relative_source(workspace: &Path, path: &Path) -> String {
    path.strip_prefix(workspace)
        .expect("source should be inside the workspace")
        .display()
        .to_string()
        .replace('\\', "/")
}

fn call_inventory(workspace: &Path, files: &[PathBuf], marker: &str) -> Vec<(String, usize)> {
    let mut inventory = files
        .iter()
        .filter_map(|path| {
            let source = fs::read_to_string(path).expect("sealing gate should read Rust source");
            let count = source.matches(marker).count();
            (count != 0).then(|| (relative_source(workspace, path), count))
        })
        .collect::<Vec<_>>();
    inventory.sort();
    inventory
}

#[test]
fn definition_store_and_targeted_removal_are_sealed_behind_prepared_tokens() {
    let workspace = workspace_root();
    let mut files = Vec::new();
    collect_production_rust_sources(&workspace.join("crates/rum-runtime/src"), &mut files);
    collect_production_rust_sources(&workspace.join("crates/rum-stdlib/src"), &mut files);

    // The staged migration APIs are gone, so no source-language consumer can
    // independently supply a classified owner and an lhs to storage/removal.
    let removed_raw_apis = [
        "store_definition_target(",
        "store_definition_assignment_target_immediate(",
        "unset_tagged_definition(",
        "remove_matching_down_values(",
        "remove_matching_up_values(",
        "remove_matching_sub_values(",
    ];
    for path in &files {
        let source = fs::read_to_string(path).expect("sealing gate should read Rust source");
        let relative = relative_source(&workspace, path);
        for removed in removed_raw_apis {
            assert!(
                !source.contains(removed),
                "raw definition API {removed} escaped the sealed module in {relative}"
            );
        }
    }

    let sealed_path = workspace.join(
        "crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs",
    );
    let sealed_source =
        fs::read_to_string(&sealed_path).expect("sealing gate should read the token module");
    assert_eq!(
        sealed_source
            .matches("fn new(lhs: Expr, owner: TaggedDefinitionTarget)")
            .count(),
        2,
        "store and removal tokens should each have one private constructor"
    );
    assert!(!sealed_source.contains("pub fn new(lhs: Expr, owner: TaggedDefinitionTarget)"));
    assert!(!sealed_source.contains("pub(super) fn new(lhs: Expr, owner: TaggedDefinitionTarget)"));

    // Rust field privacy carries the compile-time guarantee: every token
    // literal is confined to the sealed child module.
    for path in &files {
        let source = fs::read_to_string(path).expect("sealing gate should read Rust source");
        if source.contains("PreparedDefinitionStore {")
            || source.contains("PreparedDefinitionRemoval {")
        {
            assert_eq!(path, &sealed_path);
        }
    }

    // The only raw tagged-entry construction is the sealed store itself plus
    // Block's whole-SymbolDefinition restoration.
    let raw_entry_allowlist = [
        "crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs",
        "crates/rum-runtime/src/runtime/scoping/lexical/block/continuation.rs",
    ];
    for path in &files {
        let source = fs::read_to_string(path).expect("sealing gate should read Rust source");
        if source.lines().any(|line| {
            let line = line.trim();
            ["DownValue {", "UpValue {", "SubValue {"]
                .iter()
                .any(|constructor| {
                    line == *constructor || line.ends_with(&format!("= {constructor}"))
                })
        }) {
            let relative = relative_source(&workspace, path);
            assert!(
                raw_entry_allowlist.contains(&relative.as_str()),
                "raw tagged-entry construction escaped its justified allowlist in {relative}"
            );
        }
    }

    // Static catalog installation is the other explicit non-preparing path.
    // The general token constructor is private to session_api; published
    // constructors expose only the two exact static-catalog identities.
    let trusted_inventory = call_inventory(&workspace, &files, "TrustedSystemDownValueLhs::new(");
    assert_eq!(
        trusted_inventory,
        vec![(
            "crates/rum-runtime/src/runtime/session/session_api.rs".to_string(),
            1
        )]
    );
    let session_api_source =
        fs::read_to_string(workspace.join("crates/rum-runtime/src/runtime/session/session_api.rs"))
            .expect("sealing gate should read the trusted catalog token module");
    assert!(!session_api_source.contains("pub fn new(name: &str, lhs: Expr)"));
    assert_eq!(
        call_inventory(&workspace, &files, "install_system_down_value("),
        vec![
            (
                "crates/rum-runtime/src/runtime/session/session_api.rs".to_string(),
                1,
            ),
            ("crates/rum-stdlib/src/lib.rs".to_string(), 2),
        ],
        "the typed trusted installer must remain confined to its API declaration and static catalog seeding"
    );
}

#[test]
fn prepared_store_and_removal_call_site_inventory_is_closed() {
    let workspace = workspace_root();
    let mut files = Vec::new();
    collect_production_rust_sources(&workspace.join("crates/rum-runtime/src"), &mut files);
    collect_production_rust_sources(&workspace.join("crates/rum-stdlib/src"), &mut files);

    assert_eq!(
        call_inventory(&workspace, &files, ".store_prepared_definition("),
        vec![
            (
                "crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs"
                    .to_string(),
                1,
            ),
            (
                "crates/rum-runtime/src/runtime/definitions/definition_tags.rs".to_string(),
                2,
            ),
            (
                "crates/rum-runtime/src/runtime/definitions/definitions.rs".to_string(),
                1,
            ),
            (
                "crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs".to_string(),
                1,
            ),
        ]
    );
    assert_eq!(
        call_inventory(&workspace, &files, ".remove_prepared_definition("),
        vec![
            (
                "crates/rum-runtime/src/runtime/definitions/definition_clear.rs".to_string(),
                1,
            ),
            (
                "crates/rum-runtime/src/runtime/definitions/definition_tags.rs".to_string(),
                1,
            ),
        ]
    );
    assert_eq!(
        call_inventory(
            &workspace,
            &files,
            ".store_prepared_definition_lhs_immediate("
        ),
        vec![(
            "crates/rum-stdlib/src/associations/mutation.rs".to_string(),
            1
        )]
    );
    assert_eq!(
        call_inventory(&workspace, &files, ".store_immediate_symbol_own_value("),
        vec![(
            "crates/rum-stdlib/src/associations/mutation.rs".to_string(),
            3
        )]
    );
    assert_eq!(
        call_inventory(&workspace, &files, ".store_trusted_system_down_value("),
        vec![(
            "crates/rum-runtime/src/runtime/session/session_api.rs".to_string(),
            1
        )]
    );
}
