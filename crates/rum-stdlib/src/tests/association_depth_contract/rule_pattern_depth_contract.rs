//! Rule-LHS and held-pattern owned source boundaries. Evaluator reentry and shared Expr operations
//! are explicit outbound boundaries; this does not certify arbitrary Rust calls.
use super::recursion_inventory::{flat_walk_call_graph, flat_walk_recursive_edges};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};

const OWNER: &str = "crates/rum-runtime/src/runtime/";
const FILES: &[(&str, &[&str])] = &[
    (
        "definitions/rule_lhs.rs",
        &[
            "Session::evaluate_rule_lhs",
            "Session::drive_rule_lhs",
            "Session::suspend_rule_lhs",
            "rule_lhs_inert_pattern",
            "rule_lhs_policy",
            "rule_lhs_child",
            "accept_rule_lhs_value",
        ],
    ),
    (
        "definitions/rule_lhs/state.rs",
        &["RuleLhsState::destination", "RuleLhsState::pending_expr"],
    ),
    (
        "definitions/rule_lhs/continuation.rs",
        &[
            "RulePatternSuspensionFrame::resume",
            "resume_rule_pattern",
            "rule_lhs_working_copy",
            "RuleLhsCompletionFrame::resume",
        ],
    ),
    (
        "definitions/definition_lhs_preparation/syntax_policy.rs",
        &["definition_lhs_contains_pattern_syntax"],
    ),
    (
        "kernel_data/rewrite_data/cases/pattern_resolution.rs",
        &[
            "Session::resolve_held_matcher_pattern",
            "Session::drive_matcher_pattern",
            "Session::suspend_matcher_pattern",
            "accept_matcher_pattern_value",
            "contains_pattern_syntax",
            "matches_pattern_head",
        ],
    ),
    (
        "kernel_data/rewrite_data/cases/pattern_resolution/state.rs",
        &["MatcherPatternState::destination"],
    ),
    (
        "kernel_data/rewrite_data/cases/pattern_resolution/continuation.rs",
        &[
            "MatcherPatternSuspensionFrame::resume",
            "resume_matcher_pattern",
            "matcher_pattern_working_copy",
        ],
    ),
    (
        "substitution/function_and_subst_helpers.rs",
        &["strip_hold_pattern"],
    ),
];

fn sources() -> BTreeMap<String, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    FILES.iter().map(|(file, _)| {
        let raw = std::fs::read_to_string(root.join(OWNER).join(file)).expect("required G4 file");
        let mut ast = syn::parse_file(&raw).unwrap();
        if file.ends_with("syntax_policy.rs") {
            ast.items.retain(|i| matches!(i, syn::Item::Fn(f) if f.sig.ident == "definition_lhs_contains_pattern_syntax"));
        }
        if file.ends_with("function_and_subst_helpers.rs") {
            ast.items.retain(|i| matches!(i, syn::Item::Fn(f) if f.sig.ident == "strip_hold_pattern"));
        }
        // Concatenation uses item tokens; inner module doc attributes stay local.
        (file.to_string(), ast.items.iter().map(ToTokens::to_token_stream).map(|s| s.to_string()).collect::<Vec<_>>().join("\n"))
    }).collect()
}

fn assert_files_and_functions(sources: &BTreeMap<String, String>) {
    assert_eq!(
        sources.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        FILES.iter().map(|(f, _)| *f).collect(),
        "required G4 files"
    );
    for (file, expected) in FILES {
        let actual = flat_walk_call_graph(&sources[*file])
            .into_keys()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual,
            expected.iter().map(|s| s.to_string()).collect(),
            "required G4 functions in {file}"
        );
    }
}

// Every external call/macro is reviewed: ordinary evaluator reentry, existing
// completion (including the typed continuation API), syntax registry, shared
// constructors, scalar and Vec operations.
const OUTBOUND: &[&str] = &[
    "Session::evaluate",
    "resume_call_from_prepared_args",
    "normalize_match_expr",
    "MatcherPatternStep::Visit",
    "MatcherPatternStep::Done",
    "MatcherPatternDestination::Argument",
    "is_some_and",
    "matches",
    "evaluate",
    "finish_rule_lhs",
    "matcher_pattern_head",
    "is_system_symbol_name",
    "definition_lhs_blank_head",
    "definition_lhs_pattern_wrapper_head",
    "Expr::call",
    "Expr::call_shared",
    "FrameAction::complete",
    "TypedAction::complete",
    "TypedAction::complete_work",
    "ResumeStop::failure",
    "Route::failure",
    // ZST evidence borrows the actual producer; it cannot evaluate guest code.
    "slot_witness",
    // The route installs fresh typed state and forwards the request; no G4 reentry.
    "suspend",
    "std::slice::from_ref",
    "as_slice",
    "is_empty",
    "next",
    "Vec::new",
    "Vec::with_capacity",
    "RuleLhsStep::Visit",
    "RuleLhsStep::Done",
    "RuleLhsDestination::Argument",
    "Ok",
    "Err",
    "Some",
    "clone",
    "as_ref",
    "len",
    "push",
    "pop",
    "last",
    "last_mut",
    "expect",
    "is_none",
    "is_some",
    "iter",
    "rev",
    "extend",
    "unwrap_or",
    "matches!",
    "unreachable!",
    "debug_assert_eq!",
    "resume_or_suspend!",
];
fn assert_outbound(source: &str) {
    let graph = flat_walk_call_graph(source);
    let mut unexpected = BTreeSet::new();
    for (caller, calls) in &graph {
        for callee in calls {
            if !graph.contains_key(callee) && !OUTBOUND.contains(&callee.as_str()) {
                unexpected.insert(format!("{caller} -> {callee}"));
            }
        }
    }
    assert!(
        unexpected.is_empty(),
        "unaudited G4 outbound calls: {unexpected:?}"
    );
}

#[test]
fn structural_rule_pattern_depth_contract() {
    let sources = sources();
    assert_files_and_functions(&sources);
    for (file, source) in &sources {
        let edges = flat_walk_recursive_edges(source);
        assert!(edges.is_empty(), "recursive G4 edge in {file}: {edges:?}");
    }
    for owner in ["definitions/", "kernel_data/rewrite_data/cases/"] {
        let tree = sources
            .iter()
            .filter(|(file, _)| file.starts_with(owner))
            .map(|(_, source)| source.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!tree.is_empty());
        assert!(
            flat_walk_recursive_edges(&tree).is_empty(),
            "recursive owner tree {owner}"
        );
    }
    let combined = sources.values().cloned().collect::<Vec<_>>().join("\n");
    assert!(
        flat_walk_recursive_edges(&combined).is_empty(),
        "cross-module G4 recursion"
    );
    assert_outbound(&combined);
    println!(
        "G4_SOURCE_CONTRACT 8 files, exact function sets, per-file/combined recursion and outbound audit passed"
    );
}

#[test]
fn direct_cross_module_and_callback_recursion_falsifiers() {
    assert_eq!(
        flat_walk_recursive_edges("fn drive() { drive(); }").len(),
        1
    );
    let a = "fn drive() { child(); }";
    let b = "fn child() { drive(); }";
    assert!(flat_walk_recursive_edges(a).is_empty());
    assert!(flat_walk_recursive_edges(b).is_empty());
    assert_eq!(flat_walk_recursive_edges(&format!("{a} {b}")).len(), 2);
    for source in [
        "fn drive() { items.iter().for_each(drive); }",
        "fn drive() { let callback = drive; items.iter().map(callback); }",
        "fn drive() { items.iter().for_each(|_| drive()); }",
        "impl Walker { fn drive(&mut self) { items.iter().for_each(Self::drive); } }",
    ] {
        assert!(
            !flat_walk_recursive_edges(source).is_empty(),
            "hidden recursion escaped: {source}"
        );
    }
    println!(
        "G4_RECURSION_FALSIFIERS direct, cross-module, function-item, callback alias, closure, Self method fired"
    );
}
#[test]
#[should_panic(expected = "unaudited G4 outbound calls")]
fn unscanned_helper_falsifier() {
    assert_outbound("fn drive() { moved_to_unscanned_helper(); }");
}
#[test]
#[should_panic(expected = "required G4 files")]
fn missing_file_falsifier() {
    let mut sources = sources();
    sources.remove("definitions/rule_lhs/state.rs");
    assert_files_and_functions(&sources);
}
#[test]
#[should_panic(expected = "required G4 functions")]
fn missing_function_falsifier() {
    let mut sources = sources();
    sources.insert(
        "definitions/definition_lhs_preparation/syntax_policy.rs".into(),
        String::new(),
    );
    assert_files_and_functions(&sources);
}
fn assert_no_self_reaching(text: &str) {
    assert!(
        !text.lines().any(
            |line| line.contains("rum_runtime::runtime::definitions::rule_lhs::")
                || line.contains(
                    "rum_runtime::runtime::kernel_data::rewrite_data::cases::pattern_resolution::"
                )
        ),
        "self-reaching G4 state"
    );
}
#[test]
fn rule_lhs_ownership_is_not_self_reaching() {
    assert_no_self_reaching(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/tests/ownership_graph/inventory/self_reaching.tsv"
        ))
        .unwrap(),
    );
}
#[test]
#[should_panic(expected = "self-reaching G4 state")]
fn self_reaching_state_falsifier() {
    assert_no_self_reaching(
        "rum_runtime::runtime::definitions::rule_lhs::state::RuleLhsWork\tRuleLhsWork",
    );
}

fn root_fields(fields: &syn::Fields) -> BTreeSet<String> {
    fields
        .iter()
        .filter(|f| {
            let ty = f.ty.to_token_stream().to_string();
            ty.contains("Expr")
                || ty.contains("SharedArgs")
                || ty.contains("RuleLhs")
                || ty.contains("MatcherPattern")
        })
        .map(|f| {
            assert!(f.attrs.is_empty(), "root field cannot skip tracing");
            f.ident.as_ref().unwrap().to_string()
        })
        .collect()
}
fn assert_root_fields(sources: &BTreeMap<String, String>) {
    let mut actual = BTreeMap::new();
    for source in sources.values() {
        for item in syn::parse_file(source).unwrap().items {
            let (attrs, name) = match &item {
                syn::Item::Struct(item) => {
                    actual.insert(item.ident.to_string(), root_fields(&item.fields));
                    (&item.attrs, item.ident.to_string())
                }
                syn::Item::Enum(item) if item.ident == "MatcherPatternAwait" => {
                    for variant in &item.variants {
                        assert!(
                            variant.attrs.is_empty(),
                            "await variant cannot skip tracing"
                        );
                        actual.insert(
                            format!("{}::{}", item.ident, variant.ident),
                            root_fields(&variant.fields),
                        );
                    }
                    (&item.attrs, item.ident.to_string())
                }
                _ => continue,
            };
            assert!(
                attrs
                    .iter()
                    .any(|a| a.to_token_stream().to_string().contains("FrameRootTrace")),
                "{name} must derive root tracing"
            );
        }
    }
    let expected = [
        ("RuleLhsState", &["stack"][..]),
        (
            "RuleLhsWork",
            &[
                "original_head",
                "original_args",
                "resolved_head",
                "prefix",
                "policy",
            ][..],
        ),
        ("RuleLhsAwait", &["destination", "pending_expr"][..]),
        ("RulePatternSuspensionFrame", &["state", "pending"][..]),
        ("RuleLhsCompletionFrame", &["rhs"][..]),
        ("MatcherPatternState", &["stack"][..]),
        (
            "MatcherPatternWork",
            &["original_head", "original_args", "resolved_head", "prefix"][..],
        ),
        (
            "MatcherPatternAwait::Evaluate",
            &["destination", "pending_expr"][..],
        ),
        ("MatcherPatternAwait::PreparedCall", &["destination"][..]),
        ("MatcherPatternSuspensionFrame", &["state", "pending"][..]),
    ]
    .into_iter()
    .map(|(name, fields)| {
        (
            name.to_string(),
            fields
                .iter()
                .map(|s| s.to_string())
                .collect::<BTreeSet<_>>(),
        )
    })
    .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, expected, "required G4 root fields");
}
#[test]
fn rule_lhs_source_root_fields_are_complete() {
    assert_root_fields(&sources());
}
#[test]
#[should_panic(expected = "required G4 root fields")]
fn missing_root_field_falsifier() {
    let mut sources = sources();
    let source = sources.get_mut("definitions/rule_lhs/state.rs").unwrap();
    let mut ast = syn::parse_file(source).unwrap();
    for item in &mut ast.items {
        if let syn::Item::Struct(item) = item {
            if item.ident == "RuleLhsAwait" {
                if let syn::Fields::Named(fields) = &mut item.fields {
                    fields.named = fields
                        .named
                        .clone()
                        .into_iter()
                        .filter(|f| f.ident.as_ref().unwrap() != "pending_expr")
                        .collect();
                }
            }
        }
    }
    *source = ast.to_token_stream().to_string();
    assert_root_fields(&sources);
}

#[test]
#[should_panic(expected = "self-reaching G4 state")]
fn matcher_self_reaching_state_falsifier() {
    assert_no_self_reaching(
        "rum_runtime::runtime::kernel_data::rewrite_data::cases::pattern_resolution::state::MatcherPatternWork\tMatcherPatternWork",
    );
}
#[test]
#[should_panic(expected = "required G4 root fields")]
fn matcher_missing_await_root_falsifier() {
    let mut sources = sources();
    let source = sources
        .get_mut("kernel_data/rewrite_data/cases/pattern_resolution/state.rs")
        .unwrap();
    let mut ast = syn::parse_file(source).unwrap();
    for item in &mut ast.items {
        if let syn::Item::Enum(item) = item {
            if item.ident == "MatcherPatternAwait" {
                for variant in &mut item.variants {
                    if let syn::Fields::Named(fields) = &mut variant.fields {
                        fields.named = fields
                            .named
                            .clone()
                            .into_iter()
                            .filter(|f| f.ident.as_ref().unwrap() != "pending_expr")
                            .collect();
                    }
                }
            }
        }
    }
    *source = ast.to_token_stream().to_string();
    assert_root_fields(&sources);
}
#[test]
#[should_panic(expected = "required G4 root fields")]
fn matcher_missing_prepared_await_falsifier() {
    let mut sources = sources();
    let source = sources
        .get_mut("kernel_data/rewrite_data/cases/pattern_resolution/state.rs")
        .unwrap();
    let mut ast = syn::parse_file(source).unwrap();
    for item in &mut ast.items {
        if let syn::Item::Enum(item) = item {
            if item.ident == "MatcherPatternAwait" {
                item.variants = item
                    .variants
                    .clone()
                    .into_iter()
                    .filter(|v| v.ident != "PreparedCall")
                    .collect();
            }
        }
    }
    *source = ast.to_token_stream().to_string();
    assert_root_fields(&sources);
}
#[test]
fn ambiguous_method_recursion_cannot_escape_combined_scan() {
    let source = "impl A { fn drive(&self) { other.drive(); } } impl B { fn drive(&self) {} }";
    assert!(!flat_walk_recursive_edges(source).is_empty());
}
