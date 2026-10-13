//! G7 control records. Matcher, Expr internals, capture/build/render and level collection are boundaries.
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

const FILTER: &str = "crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs";
const LEAF: &str = "crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs";
const STATE: &str = "crates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs";
const FRAMES: &str = "crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs";
fn trace_sources() -> BTreeMap<String, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    [FILTER, LEAF, STATE, FRAMES]
        .into_iter()
        .map(|file| {
            (
                file.into(),
                std::fs::read_to_string(root.join(file)).unwrap(),
            )
        })
        .collect()
}
fn trace_acyclic(sources: &BTreeMap<String, String>) {
    for source in sources.values() {
        assert!(
            flat_walk_recursive_edges(source).is_empty(),
            "recursive trace file"
        );
    }
    let combined = sources.values().cloned().collect::<Vec<_>>().join("\n");
    assert!(
        flat_walk_recursive_edges(&combined).is_empty(),
        "recursive combined trace modules"
    );
}
#[test]
fn trace_flat_call_graph_and_outbound_boundaries() {
    let sources = trace_sources();
    trace_acyclic(&sources);
    let combined = sources.values().cloned().collect::<Vec<_>>().join("\n");
    let graph = flat_walk_call_graph(&combined);
    let owned = "Session::filter_trace_result Session::drive_trace_filter Session::finish_trace_filter_root append_child suspend_trace_filter resume_trace_filter CandidateWork::current CandidateWork::take_candidate select_candidates start_leaf LeafWork::apply_match LeafWork::next_field PatternWork::pattern_count PatternWork::pattern_at Session::match_trace_leaf finish_leaf LevelWork::from_items LevelWork::freeze_level TraceFilterLevel::thaw_level FilterWork::freeze_filter TraceFilterState::thaw_filter EvalStructuredTraceCaptureExprSuspensionFrame::resume EvalStructuredTraceCaptureExprSuspensionFrame::abandon EvalStructuredTraceCaptureExprSuspensionFrame::output_boundary TraceFilterSuspensionFrame::resume";
    assert_eq!(
        graph.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        owned.split_whitespace().collect(),
        "closed trace functions"
    );
    let outbound = "Ok Err Some is_system_symbol_name Expr::call_shared Expr::call into_vec Vec::new take next push std::mem::replace pop render_structured_trace_lines push_print is_empty len Expr::call_with_shared_args system_expr clone resume_or_suspend! CompiledMatchOutcome::from_continuation_expr as_ref as_slice first CasesLevelSpec::range is_some collect_level_parts_owned into_iter trace_detail_underlying_exprs is_none unwrap_trace_hold push_unique_trace_item normalize_trace_patterns matches! expect vec! push_unique_trace_expr match_expr_owned SymbolExprMap::default collect map iter <indirect-call> finish_structured_trace_capture_expr crate::runtime::suspension::FrameAction::complete restore_trace_capture CandidateWork::Single CandidateWork::Parts TraceFilterCandidates::Single TraceFilterCandidates::Parts PatternWork::Parts OutputBoundary::trace";
    for (caller, calls) in graph {
        for callee in calls {
            assert!(
                owned.split_whitespace().any(|n| n == callee)
                    || outbound.split_whitespace().any(|n| n == callee),
                "unaudited trace call: {caller} -> {callee}"
            );
        }
    }
    println!(
        "G7_TRACE_SOURCE closed functions, outbound boundaries and per-file/combined acyclic walk passed"
    );
}
fn trace_fields(sources: &BTreeMap<String, String>) {
    let mut actual = Vec::new();
    for source in sources.values() {
        for item in syn::parse_file(source).unwrap().items {
            let (name, fields, attrs) = match item {
                syn::Item::Struct(s) => (s.ident.to_string(), s.fields, s.attrs),
                syn::Item::Enum(e) if e.ident == "TraceFilterCandidates" => {
                    assert_eq!(
                        compact(&e.variants),
                        "Single(Expr),Parts(Vec<Expr>),",
                        "closed candidate variants"
                    );
                    assert!(
                        e.attrs
                            .iter()
                            .any(|a| compact(a).contains("FrameRootTrace")),
                        "trace roots require derive"
                    );
                    continue;
                }
                _ => continue,
            };
            if !name.starts_with("TraceFilter") {
                continue;
            }
            assert!(
                attrs.iter().any(|a| compact(a).contains("FrameRootTrace")),
                "trace roots require derive"
            );
            for field in fields {
                assert!(field.attrs.is_empty(), "trace root cannot skip tracing");
                actual.push(format!(
                    "{}.{}:{}",
                    name,
                    field.ident.unwrap(),
                    compact(field.ty)
                ));
            }
        }
    }
    actual.sort();
    let expected = "TraceFilterState.mode:TraceOutput TraceFilterState.original_value:Expr TraceFilterState.pattern:Option<Expr> TraceFilterState.levels:Option<CasesLevelSpec> TraceFilterState.include_heads:bool TraceFilterState.current:TraceFilterLevel TraceFilterState.parents:Vec<TraceFilterLevel> TraceFilterState.pending:TraceFilterLeaf TraceFilterLevel.remaining:Vec<Expr> TraceFilterLevel.emitted:Vec<Expr> TraceFilterLeaf.original:Expr TraceFilterLeaf.underlying:Vec<Expr> TraceFilterLeaf.field_index:usize TraceFilterLeaf.patterns:Vec<Expr> TraceFilterLeaf.pattern_index:usize TraceFilterLeaf.candidates:TraceFilterCandidates TraceFilterLeaf.candidate_index:usize TraceFilterLeaf.selected:Vec<Expr> TraceFilterSuspensionFrame.state:filter::TraceFilterState";
    let mut expected = expected.split_whitespace().collect::<Vec<_>>();
    expected.sort();
    assert_eq!(actual, expected, "complete flat trace fields and roots");
}
fn trace_no_self_reaching(inventory: &str) {
    assert!(
        !inventory
            .lines()
            .any(|l| l.contains("debug_tools::trace::filter::")
                || l.contains("::TraceFilterSuspensionFrame")),
        "self-reaching trace state"
    );
}
#[test]
fn trace_complete_flat_fields_and_roots() {
    trace_fields(&trace_sources());
    trace_no_self_reaching(include_str!(
        "../ownership_graph/inventory/self_reaching.tsv"
    ));
    println!(
        "G7_TRACE_ROOTS all five derived types, closed fields and candidates, no self-reaching control state"
    );
}
fn trace_edges(sources: &BTreeMap<String, String>) {
    let driver = functions(&sources[FILTER]);
    let leaf = functions(&sources[LEAF]);
    let frames = functions(&sources[FRAMES]);
    let drive = &driver["drive_trace_filter"];
    assert_eq!(
        drive.matches("finish_trace_filter_root").count(),
        1,
        "print at completed root only"
    );
    assert!(drive.contains("Err(EvalError::InputRequired{request})=>{returnsuspend_trace_filter(self,work,leaf,request);}"), "trace match owns cold capture");
    assert!(
        drive.contains("Err(error)=>returnErr(error)"),
        "trace ordinary error propagates"
    );
    assert!(driver["suspend_trace_filter"].contains("letstate=work.freeze_filter(leaf);crate::resume_or_suspend!(insertsession,TraceFilterSuspensionFrame{state});Err(EvalError::InputRequired{request})"), "one named cold filter frame");
    assert_eq!(
        sources[FILTER].matches(".freeze_filter(").count(),
        1,
        "one named cold filter frame"
    );
    assert_eq!(
        sources[LEAF].matches("match_expr_owned(").count(),
        1,
        "one pending match edge"
    );
    assert!(leaf["match_trace_leaf"].contains("self.match_expr_owned(leaf.patterns.pattern_at(leaf.pattern_index,work.pattern.as_ref()).clone(),candidate.clone(),SymbolExprMap::default(),)?.matched"), "one pending match edge");
    let resume = &driver["resume_trace_filter"];
    assert!(
        resume.contains("CompiledMatchOutcome::from_continuation_expr(value)"),
        "decode trace matcher answer"
    );
    assert!(resume.contains("let(mutwork,mutleaf)=frozen.thaw_filter();letpending=ifleaf.apply_match(outcome.matched,work.mode==TraceOutput::Detail){finish_leaf(&mutwork,leaf,true);None}else{Some(leaf)};session.drive_trace_filter(work,pending)"), "apply trace answer once without replay");
    assert!(frames["resume"].contains("Ok(value)=>filter::resume_trace_filter(session,&self.state,value),Err(error)=>Err(error)"), "trace error resumes do not thaw");
    let apply = &leaf["apply_match"];
    assert!(
        apply.starts_with("{ifmatched&&(detail||matches!(self.candidates,CandidateWork::Single(_))){returntrue;}self.pattern_index+=1;"),
        "detail short circuit and pattern advance"
    );
    assert!(
        apply.contains("ifmatched||self.pattern_index>=self.patterns.pattern_count()"),
        "false continues next pattern then candidate"
    );
    assert!(
        apply.contains("self.candidate_index+=1;self.pattern_index=0;"),
        "candidate advances once"
    );
    assert!(
        leaf["next_field"].contains("self.underlying.next();self.field_index+=1;"),
        "detail fields advance once"
    );
    assert!(
        drive.contains("work.parents.push(std::mem::replace(&mutwork.current,child))"),
        "save parent prefixes"
    );
    assert!(
        drive
            .contains("append_child(&mutparent.emitted,work.current.emitted);work.current=parent;"),
        "restore parent prefixes"
    );
    assert!(drive.contains(r#"letSome(mutparent)=work.parents.pop()else{lettrace=Expr::call(system_expr("List"),work.current.emitted);returnself.finish_trace_filter_root(trace,work.mode,work.original_value,);}"#), "print at completed root only");
    for (name, body) in driver.iter().chain(leaf.iter()) {
        if name != "finish_trace_filter_root" {
            assert!(
                !body.contains("push_print") && !body.contains("render_structured_trace_lines"),
                "no streaming trace lines"
            );
        }
    }
    assert!(
        driver["finish_trace_filter_root"].contains("self.push_print(line);"),
        "root prints once"
    );
    assert!(
        driver["finish_trace_filter_root"].contains("Ok(original_value)"),
        "saved TracePrint result"
    );
}
#[test]
fn trace_cold_edges_cursors_and_root_print_contract() {
    trace_edges(&trace_sources());
    println!(
        "G7_TRACE_CONTINUATION one cold capture, exact answer/cursor, short circuit, parent restore and root print passed"
    );
}
fn rejected(check: impl FnOnce() + std::panic::UnwindSafe, label: &str) {
    assert!(
        std::panic::catch_unwind(check).is_err(),
        "falsifier escaped: {label}"
    );
    println!("G7_TRACE_FALSIFIER {label}: rejected");
}
#[test]
fn trace_source_and_root_falsifiers_are_observed() {
    for (file, before, after, label) in [
        (
            STATE,
            "    pub(super) original_value: Expr,",
            "",
            "missing saved root",
        ),
        (
            STATE,
            "    remaining: Vec<Expr>,",
            "    #[frame_root(skip)] remaining: Vec<Expr>,",
            "skipped root",
        ),
        (
            STATE,
            "parents: Vec<TraceFilterLevel>",
            "parents: Vec<TraceFilterState>",
            "recursive state",
        ),
        (
            STATE,
            "Single(Expr)",
            "Single(usize)",
            "candidate root variant",
        ),
    ] {
        let mut sources = trace_sources();
        assert!(sources[file].contains(before));
        sources.insert(file.into(), sources[file].replace(before, after));
        rejected(|| trace_fields(&sources), label);
    }
    let mut sources = trace_sources();
    sources
        .get_mut(FILTER)
        .unwrap()
        .push_str("fn bad() { bad(); }");
    rejected(|| trace_acyclic(&sources), "direct recursion");
    let mut sources = trace_sources();
    sources
        .get_mut(FILTER)
        .unwrap()
        .push_str("fn bad_a() { bad_b(); }");
    sources
        .get_mut(LEAF)
        .unwrap()
        .push_str("fn bad_b() { bad_a(); }");
    rejected(|| trace_acyclic(&sources), "cross-helper recursion");
    rejected(
        || trace_no_self_reaching("debug_tools::trace::filter::TraceFilterState"),
        "self-reaching inventory",
    );
}
#[test]
fn trace_edge_falsifiers_are_observed() {
    for (file, before, after, label) in [
        (
            FILTER,
            "return suspend_trace_filter(self, work, leaf, request);",
            "return Err(EvalError::InputRequired { request });",
            "lost pending match",
        ),
        (
            LEAF,
            "self.pattern_index += 1;",
            "self.pattern_index += 2;",
            "doubled cursor",
        ),
        (
            FILTER,
            "let (mut work, mut leaf) = frozen.thaw_filter();",
            "let (mut work, mut leaf) = frozen.thaw_filter(); leaf.pattern_index = 0;",
            "cursor reset replay",
        ),
        (
            FILTER,
            "append_child(&mut parent.emitted, work.current.emitted);",
            "parent.emitted.clear();",
            "omitted parent prefix",
        ),
        (
            LEAF,
            "if matched || self.pattern_index >= self.patterns.pattern_count()",
            "if true",
            "false ends search",
        ),
        (
            FILTER,
            "if let Some(leaf) = start_leaf(&mut work, item) {",
            "if let Some(leaf) = start_leaf(&mut work, item) { return self.finish_trace_filter_root(item, work.mode, work.original_value);",
            "early print completion",
        ),
        (
            LEAF,
            "loop {",
            "loop { self.push_print(String::new());",
            "stream before final success",
        ),
        (
            FILTER,
            "let state = work.freeze_filter(leaf);",
            "let state = work.freeze_filter(leaf); let second = work.freeze_filter(leaf);",
            "double freeze",
        ),
    ] {
        let mut sources = trace_sources();
        assert!(sources[file].contains(before), "mutation anchor {label}");
        sources.insert(file.into(), sources[file].replace(before, after));
        rejected(|| trace_edges(&sources), label);
    }
}
