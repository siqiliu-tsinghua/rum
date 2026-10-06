//! Sow control records only. Matcher, Expr internals and Reap rendering are outbound boundaries.
use super::recursion_inventory::{flat_walk_call_graph, flat_walk_recursive_edges};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};

const SOW: &str = "crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs";
fn sources() -> BTreeMap<String, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    [(SOW.into(), std::fs::read_to_string(root.join(SOW)).unwrap())].into()
}
fn assert_acyclic(sources: &BTreeMap<String, String>) {
    for (file, source) in sources {
        assert!(
            flat_walk_recursive_edges(source).is_empty(),
            "recursive Sow file: {file}"
        );
    }
    let combined = sources.values().cloned().collect::<Vec<_>>().join("\n");
    assert!(
        flat_walk_recursive_edges(&combined).is_empty(),
        "recursive combined Sow modules"
    );
}
#[test]
fn sow_recursion_scanner_per_file_and_combined() {
    let sources = sources();
    assert_eq!(
        sources.keys().map(String::as_str).collect::<Vec<_>>(),
        [SOW]
    );
    assert_acyclic(&sources);
    let graph = flat_walk_call_graph(&sources[SOW]);
    assert_eq!(
        graph.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        [
            "Session::eval_sow",
            "Session::drive_sow_matches",
            "SowMatchSuspensionFrame::resume",
            "suspend_sow_match",
            "resume_sow_match",
        ]
        .into()
    );
    let outbound = [
        "as_ref",
        "clone",
        "system_expr",
        "Ok",
        "Expr::call",
        "to_vec",
        "len",
        "Vec::new",
        "match_expr_owned",
        "SymbolExprMap::default",
        "Err",
        "resume_or_suspend!",
        "push",
        "record_sown_value",
        "is_some",
        "charge_reap_occurrence",
        "FrameAction::complete",
        "CompiledMatchOutcome::from_continuation_expr",
    ];
    for (caller, calls) in &graph {
        for callee in calls {
            assert!(
                graph.contains_key(callee) || outbound.contains(&callee.as_str()),
                "unaudited Sow call: {caller} -> {callee}"
            );
        }
    }
    println!(
        "G7_SOW_SOURCE per-file and combined recursion, closed functions and outbound calls passed"
    );
}
#[test]
#[should_panic(expected = "recursive Sow file")]
fn direct_recursion_falsifier() {
    let mut sources = sources();
    sources
        .get_mut(SOW)
        .unwrap()
        .push_str("fn cycle() { cycle(); }");
    assert_acyclic(&sources);
}
#[test]
#[should_panic(expected = "recursive combined Sow modules")]
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
fn assert_fields(source: &str) {
    let mut actual = BTreeMap::new();
    for item in syn::parse_file(source).unwrap().items {
        if let syn::Item::Struct(item) = item {
            assert!(
                item.attrs
                    .iter()
                    .any(|a| compact(a).contains("FrameRootTrace")),
                "Sow roots require derive"
            );
            let fields = item
                .fields
                .iter()
                .map(|f| {
                    assert!(f.attrs.is_empty(), "Sow root cannot skip tracing");
                    (f.ident.as_ref().unwrap().to_string(), compact(&f.ty))
                })
                .collect::<BTreeMap<_, _>>();
            actual.insert(item.ident.to_string(), fields);
        }
    }
    let expected = [
        (
            "SowMatchState",
            vec![
                ("value", "Expr"),
                ("tag", "Expr"),
                ("scope_limit", "usize"),
                ("scope_index", "usize"),
                ("pattern_index", "usize"),
                ("pattern_count", "usize"),
                ("matched", "Vec<(usize,usize)>"),
            ],
        ),
        ("SowMatchSuspensionFrame", vec![("state", "SowMatchState")]),
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
    assert_eq!(actual, expected, "complete flat Sow fields and roots");
}
fn assert_no_self_reaching(inventory: &str) {
    assert!(
        !inventory
            .lines()
            .any(|l| l.contains("rum_runtime::runtime::debug_tools::reap::sow::")),
        "self-reaching Sow state"
    );
}
#[test]
fn sow_flat_fields_roots_and_ownership_are_complete() {
    assert_fields(&sources()[SOW]);
    assert_no_self_reaching(include_str!(
        "../ownership_graph/inventory/self_reaching.tsv"
    ));
    println!(
        "G7_SOW_ROOTS value, tag and state roots; exact scalar/pair fields; no self-reaching Sow state"
    );
}
#[test]
#[should_panic(expected = "complete flat Sow fields and roots")]
fn missing_root_falsifier() {
    assert_fields(&sources()[SOW].replace("    tag: Expr,", ""));
}
#[test]
#[should_panic(expected = "Sow root cannot skip tracing")]
fn skipped_root_falsifier() {
    assert_fields(&sources()[SOW].replace("    tag: Expr,", "    #[frame_root(skip)] tag: Expr,"));
}
#[test]
#[should_panic(expected = "complete flat Sow fields and roots")]
fn recursive_field_falsifier() {
    assert_fields(&sources()[SOW].replace(
        "matched: Vec<(usize, usize)>",
        "matched: Vec<SowMatchState>",
    ));
}
#[test]
#[should_panic(expected = "self-reaching Sow state")]
fn self_reaching_inventory_falsifier() {
    assert_no_self_reaching("rum_runtime::runtime::debug_tools::reap::sow::SowMatchState\tself");
}

fn functions(source: &str) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for item in syn::parse_file(source).unwrap().items {
        match item {
            syn::Item::Fn(f) => {
                result.insert(f.sig.ident.to_string(), compact(f.block));
            }
            syn::Item::Impl(i) => {
                for item in i.items {
                    if let syn::ImplItem::Fn(f) = item {
                        result.insert(f.sig.ident.to_string(), compact(f.block));
                    }
                }
            }
            _ => {}
        }
    }
    result
}
fn assert_continuation_edges(source: &str) {
    let bodies = functions(source);
    let drive = &bodies["drive_sow_matches"];
    let begin = drive
        .find("Err(EvalError::InputRequired{request})=>{")
        .unwrap();
    let end = drive[begin..].find("Err(error)=>returnErr(error)").unwrap() + begin;
    let capture = &drive[begin..end];
    assert_eq!(
        drive.matches("SowMatchState{").count(),
        1,
        "one cold Sow capture"
    );
    assert!(capture.contains("letstate=SowMatchState{value,tag,scope_limit,scope_index,pattern_index,pattern_count,matched,}"), "one cold Sow capture");
    assert!(
        capture.contains("returnsuspend_sow_match(self,state,request);"),
        "named Sow frame owns the match"
    );
    assert!(
        bodies["suspend_sow_match"].contains(
            "resume_or_suspend!(insertsession,SowMatchSuspensionFrame{state});Err(EvalError::InputRequired{request})"
        ),
        "named Sow frame owns the match"
    );
    assert!(
        !bodies["eval_sow"].contains("SowMatchState{"),
        "one cold Sow capture"
    );
    let ordinary = "ifoutcome.matched{matched.push((scope_index,pattern_index));}pattern_index+=1;";
    assert!(
        drive.contains(ordinary),
        "ordinary Sow answer advances once"
    );
    // The only bucket call must occur in the top-level final loop, after the matching loop.
    let ast = syn::parse_file(source).unwrap();
    let mut commit_after_matching = false;
    for item in ast.items {
        if let syn::Item::Impl(i) = item {
            for item in i.items {
                if let syn::ImplItem::Fn(f) = item {
                    if f.sig.ident != "drive_sow_matches" {
                        continue;
                    }
                    let statements = &f.block.stmts;
                    if let (
                        syn::Stmt::Expr(syn::Expr::While(matching), _),
                        syn::Stmt::Expr(syn::Expr::ForLoop(commit), _),
                    ) = (&statements[0], &statements[1])
                    {
                        commit_after_matching = !compact(matching).contains("record_sown_value")
                            && compact(commit).contains("record_sown_value");
                    }
                }
            }
        }
    }
    assert!(
        commit_after_matching && drive.matches("record_sown_value").count() == 1,
        "Sow commit follows all matching"
    );
    let resume = &bodies["resume_sow_match"];
    assert!(
        resume.contains("CompiledMatchOutcome::from_continuation_expr(value)"),
        "decode matcher outcome"
    );
    assert!(resume.contains("ifoutcome.matched{state.matched.push((state.scope_index,state.pattern_index));}state.pattern_index+=1;"), "resumed Sow answer advances once");
    assert!(
        !resume.contains("match_expr_owned"),
        "no replay of the pending match"
    );
    assert!(
        bodies["resume"].contains(
            "Ok(value)=>resume_sow_match(session,&self.state,value),Err(error)=>Err(error)"
        ),
        "error resumes do not copy or commit"
    );
}
#[test]
fn sow_cold_capture_cursor_and_commit_contract() {
    assert_continuation_edges(&sources()[SOW]);
    println!(
        "G7_SOW_CONTINUATION cold capture, decode, one cursor advance and final commit passed"
    );
}
#[test]
#[should_panic(expected = "resumed Sow answer advances once")]
fn doubled_cursor_falsifier() {
    assert_continuation_edges(
        &sources()[SOW].replace("state.pattern_index += 1;", "state.pattern_index += 2;"),
    );
}
#[test]
#[should_panic(expected = "resumed Sow answer advances once")]
fn lost_pending_accept_falsifier() {
    assert_continuation_edges(&sources()[SOW].replace(
        "state.matched.push((state.scope_index, state.pattern_index));",
        "",
    ));
}
#[test]
#[should_panic(expected = "one cold Sow capture")]
fn eager_capture_falsifier() {
    assert_continuation_edges(&sources()[SOW].replace(
        "let scope_limit = self.reap_scopes.len();",
        "let state = SowMatchState {}; let scope_limit = self.reap_scopes.len();",
    ));
}
#[test]
#[should_panic(expected = "Sow commit follows all matching")]
fn early_bucket_commit_falsifier() {
    assert_continuation_edges(&sources()[SOW].replace(
        "while pattern_index < pattern_count {",
        "while pattern_index < pattern_count { record_sown_value();",
    ));
}
