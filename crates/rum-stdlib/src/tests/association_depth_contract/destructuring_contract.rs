//! G9's flat store continuation. Existing pre-store shape validation, leaf
//! assignment, and Expr/SharedArgs internals are outside this module boundary.
use super::recursion_inventory::flat_walk_recursive_edges;
use quote::ToTokens;
use std::collections::BTreeMap;

const DRIVER: &str = "crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs";
fn sources() -> BTreeMap<String, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    [(
        DRIVER.into(),
        std::fs::read_to_string(root.join(DRIVER)).unwrap(),
    )]
    .into()
}
fn assert_acyclic(sources: &BTreeMap<String, String>) {
    for (file, source) in sources {
        assert!(
            flat_walk_recursive_edges(source).is_empty(),
            "recursive destructuring file: {file}"
        );
    }
    let combined = sources.values().cloned().collect::<Vec<_>>().join("\n");
    assert!(
        flat_walk_recursive_edges(&combined).is_empty(),
        "recursive combined destructuring modules"
    );
}
#[test]
fn recursion_scanner_per_file_and_combined() {
    let sources = sources();
    assert_eq!(
        sources.keys().map(String::as_str).collect::<Vec<_>>(),
        [DRIVER]
    );
    assert_acyclic(&sources);
    println!(
        "G9_SOURCE per-file and combined recursion scan passed; shape validation and leaf assignment are boundaries"
    );
}
#[test]
#[should_panic(expected = "recursive destructuring file")]
fn direct_recursion_falsifier() {
    let mut sources = sources();
    sources
        .get_mut(DRIVER)
        .unwrap()
        .push_str("fn cycle() { cycle(); }");
    assert_acyclic(&sources);
}
#[test]
#[should_panic(expected = "recursive combined destructuring modules")]
fn cross_file_recursion_falsifier() {
    assert_acyclic(
        &[
            ("a.rs".into(), "fn drive() { leaf(); }".into()),
            ("b.rs".into(), "fn leaf() { drive(); }".into()),
        ]
        .into(),
    );
}
fn compact(tokens: impl ToTokens) -> String {
    tokens
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect()
}
fn assert_roots(source: &str) {
    let mut actual = BTreeMap::new();
    for item in syn::parse_file(source).unwrap().items {
        if let syn::Item::Struct(item) = item {
            assert!(
                item.attrs
                    .iter()
                    .any(|a| compact(a).contains("FrameRootTrace")),
                "destructuring roots require derive"
            );
            if item.ident == "DestructuringSuspensionFrame" {
                assert!(
                    item.attrs
                        .iter()
                        .any(|a| compact(a).contains("PureFrameAbandon")),
                    "destructuring abandonment must be pure"
                );
            }
            let fields = item
                .fields
                .iter()
                .map(|f| {
                    assert!(f.attrs.is_empty(), "destructuring root cannot skip tracing");
                    (f.ident.as_ref().unwrap().to_string(), compact(&f.ty))
                })
                .collect::<BTreeMap<_, _>>();
            actual.insert(item.ident.to_string(), fields);
        }
    }
    let expected = [
        (
            "DestructuringCursor",
            vec![
                ("lhs", "SharedArgs"),
                ("rhs", "SharedArgs"),
                ("next", "usize"),
            ],
        ),
        (
            "DestructuringWork",
            vec![
                ("current", "DestructuringCursor"),
                ("parent", "Option<DestructuringCursor>"),
                ("parents", "Vec<DestructuringCursor>"),
                ("pending_leaf", "bool"),
            ],
        ),
        (
            "DestructuringSuspensionFrame",
            vec![
                ("symbol", "Symbol"),
                ("original_rhs", "Expr"),
                ("work", "DestructuringWork"),
            ],
        ),
    ]
    .into_iter()
    .map(|(name, fields)| {
        (
            name.to_string(),
            fields
                .into_iter()
                .map(|(n, t)| (n.to_string(), t.to_string()))
                .collect(),
        )
    })
    .collect::<BTreeMap<_, BTreeMap<_, _>>>();
    assert_eq!(
        actual, expected,
        "complete flat destructuring fields and roots"
    );
}
fn assert_no_self_reaching(inventory: &str) {
    assert!(
        !inventory
            .lines()
            .any(|line| line.contains("definitions::definitions::destructuring::")),
        "self-reaching destructuring state"
    );
}
#[test]
fn complete_roots_flat_fields_and_no_self_reaching_state() {
    assert_roots(&sources()[DRIVER]);
    assert_no_self_reaching(include_str!(
        "../ownership_graph/inventory/self_reaching.tsv"
    ));
    println!(
        "G9_ROOTS original RHS, current/parent LHS and RHS arguments; exact fields, pure abandonment, no self-reaching state"
    );
}
#[test]
#[should_panic(expected = "complete flat destructuring fields and roots")]
fn missing_root_falsifier() {
    assert_roots(&sources()[DRIVER].replace("    original_rhs: Expr,", ""));
}
#[test]
#[should_panic(expected = "destructuring root cannot skip tracing")]
fn skipped_root_falsifier() {
    assert_roots(&sources()[DRIVER].replace(
        "    original_rhs: Expr,",
        "    #[frame_root(skip)] original_rhs: Expr,",
    ));
}
#[test]
#[should_panic(expected = "complete flat destructuring fields and roots")]
fn recursive_state_falsifier() {
    assert_roots(&sources()[DRIVER].replace(
        "parents: Vec<DestructuringCursor>",
        "parents: Vec<DestructuringWork>",
    ));
}
#[test]
#[should_panic(expected = "self-reaching destructuring state")]
fn self_reaching_inventory_falsifier() {
    assert_no_self_reaching(
        "rum_runtime::runtime::definitions::definitions::destructuring::DestructuringWork\tself",
    );
}
