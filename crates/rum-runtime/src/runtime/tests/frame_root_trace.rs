use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rum_core::{Expr, SharedArgs, SharedExpr, Symbol};

use crate::runtime::suspension::trace::{
    FrameRootTrace, FrameRootVisitor, MANUAL_FRAME_ROOT_TRACE_IMPLS,
};
use crate::runtime::{
    AssociationEntryForm, BlockSymbolState, FileIoError, MatcherArg, MatcherNode, SymbolExprMap,
};

#[derive(Default)]
struct RecordedRoots {
    exprs: Vec<*const Expr>,
    symbols: Vec<*const Symbol>,
}

impl FrameRootVisitor for RecordedRoots {
    fn visit_expr(&mut self, expr: &Expr) {
        self.exprs.push(expr as *const Expr);
    }

    fn visit_symbol(&mut self, symbol: &Symbol) {
        self.symbols.push(symbol as *const Symbol);
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct NestedRoots {
    expression: Expr,
    symbol: Symbol,
}

#[derive(rum_derive::FrameRootTrace)]
struct NestedFrame {
    direct: Expr,
    nested: Option<Box<NestedRoots>>,
}

#[derive(rum_derive::FrameRootTrace)]
struct ContainerRoots {
    vector: Vec<Expr>,
    queue: VecDeque<Symbol>,
    optional: Option<Box<Expr>>,
    arc_slice: Arc<[Expr]>,
    hash_map: HashMap<Symbol, Expr>,
    tree_map: BTreeMap<Symbol, Expr>,
    tuple: (Expr, Symbol),
}

#[derive(rum_derive::FrameRootTrace)]
enum VariantRoots {
    Named { expression: Expr, symbol: Symbol },
    Tuple(Expr, Symbol),
    Unit,
}

#[derive(rum_derive::FrameRootTrace)]
struct GenericRoots<T, const N: usize> {
    values: Vec<T>,
    fixed: [Expr; N],
}

mod derived_inventory;
use derived_inventory::{DERIVED_FRAME_ROOT_TRACE_TYPES, PURE_FRAME_ABANDON_TYPES};

const STATEFUL_FRAME_ABANDON_TYPES: &[&str] = &[
    "crates/rum-runtime/src/runtime/debug_tools/evaluation_data.rs:EvalEvaluationDataSuspensionFrame",
    "crates/rum-runtime/src/runtime/debug_tools/monitor.rs:EvalMonitorSuspensionFrame",
    "crates/rum-runtime/src/runtime/debug_tools/reap.rs:EvalReapSuspensionFrame",
    "crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:ResumeMemoryConstrainedWithLimitSuspensionFrame",
    "crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:ResumeTimeConstrainedWithLimitSuspensionFrame",
    "crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:EvalExprWithStreamCaptureSuspensionFrame",
    "crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:EvalStructuredTraceCaptureExprSuspensionFrame",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:EvalAbortProtectSuspensionFrame",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortAfterPrimarySuspensionFrame1",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortAfterPrimarySuspensionFrame2",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup/resume_frames.rs:ResumeCheckAbortWithPropagationSuspensionFrame",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:EvaluateLoopIteratorBodySuspensionFrame",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs:EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrame",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:WithIterationProductScopeSuspensionFrame",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:EvalCheckSuspensionFrame",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:EvalQuietSuspensionFrame",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:DriveWithCleanupPrimarySuspensionFrame",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:EvalWithCleanupSuspensionFrame",
    "crates/rum-runtime/src/runtime/options/patterns.rs:EvaluateWithOptionValueScopeSuspensionFrame",
    "crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:EvalSingleSymbolBlockSuspensionFrame",
    "crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeBlockBodySuspensionFrame",
    "crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeBlockInitializersSuspensionFrame",
    "crates/rum-runtime/src/runtime/scoping/lexical/block/resume_frames.rs:ResumeSingleSymbolBlockBodySuspensionFrame",
    "crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeModuleInitializersSuspensionFrame1",
    "crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeModuleInitializersSuspensionFrame2",
    "crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeSuspendedModuleInitializersSuspensionFrame1",
    "crates/rum-runtime/src/runtime/scoping/lexical/module.rs:ResumeSuspendedModuleInitializersSuspensionFrame2",
    "crates/rum-runtime/src/runtime/scoping/packages/resume_frames.rs:ResumePackageInstallSuspensionFrame",
    "crates/rum-runtime/src/runtime/session/session_api/resume_frames.rs:EvaluateWithRandomStateRestoredSuspensionFrame",
];

#[test]
fn expr_is_one_whole_root_and_bare_symbol_is_an_independent_root() {
    let expression = Expr::call(Expr::symbol("head"), vec![Expr::symbol("argument")]);
    let symbol = Symbol::from_name("bare");
    let mut roots = RecordedRoots::default();

    expression.trace_roots(&mut roots);
    symbol.trace_roots(&mut roots);

    assert_eq!(roots.exprs, vec![&expression as *const Expr]);
    assert_eq!(roots.symbols, vec![&symbol as *const Symbol]);
}

#[test]
fn shared_expression_types_borrow_their_expression_roots() {
    let shared_expr = SharedExpr::new(Expr::symbol("shared"));
    let shared_args = SharedArgs::new(vec![Expr::symbol("first"), Expr::symbol("second")]);
    let mut roots = RecordedRoots::default();

    shared_expr.trace_roots(&mut roots);
    shared_args.trace_roots(&mut roots);

    assert_eq!(
        roots.exprs,
        vec![
            shared_expr.as_ref() as *const Expr,
            &shared_args[0] as *const Expr,
            &shared_args[1] as *const Expr,
        ]
    );
    assert!(roots.symbols.is_empty());
}

#[test]
fn nested_derived_fields_are_all_traversed() {
    let frame = NestedFrame {
        direct: Expr::symbol("direct"),
        nested: Some(Box::new(NestedRoots {
            expression: Expr::symbol("nested"),
            symbol: Symbol::from_name("bare"),
        })),
    };
    let nested = frame.nested.as_ref().unwrap();
    let mut roots = RecordedRoots::default();

    frame.trace_roots(&mut roots);

    assert_eq!(
        roots.exprs,
        vec![
            &frame.direct as *const Expr,
            &nested.expression as *const Expr,
        ]
    );
    assert_eq!(
        roots.symbols.len(),
        1,
        "the nested Symbol field must be traversed"
    );
    assert_eq!(roots.symbols, vec![&nested.symbol as *const Symbol]);
}

#[test]
fn container_implementations_visit_keys_values_and_elements() {
    let map_symbol = Symbol::from_name("hash-key");
    let tree_symbol = Symbol::from_name("tree-key");
    let mut hash_map = HashMap::new();
    hash_map.insert(map_symbol, Expr::symbol("hash-value"));
    let mut tree_map = BTreeMap::new();
    tree_map.insert(tree_symbol, Expr::symbol("tree-value"));
    let frame = ContainerRoots {
        vector: vec![Expr::symbol("vector")],
        queue: VecDeque::from([Symbol::from_name("queue")]),
        optional: Some(Box::new(Expr::symbol("optional"))),
        arc_slice: Arc::from([Expr::symbol("arc")]),
        hash_map,
        tree_map,
        tuple: (Expr::symbol("tuple"), Symbol::from_name("tuple-symbol")),
    };
    let mut roots = RecordedRoots::default();

    frame.trace_roots(&mut roots);

    assert_eq!(roots.exprs.len(), 6);
    assert_eq!(roots.symbols.len(), 4);
}

#[test]
fn enum_shapes_and_generic_arrays_trace_without_special_cases() {
    let variants = [
        VariantRoots::Named {
            expression: Expr::symbol("named"),
            symbol: Symbol::from_name("named-symbol"),
        },
        VariantRoots::Tuple(Expr::symbol("tuple"), Symbol::from_name("tuple-symbol")),
        VariantRoots::Unit,
    ];
    let generic = GenericRoots {
        values: vec![Expr::symbol("generic")],
        fixed: [Expr::symbol("array")],
    };
    let mut roots = RecordedRoots::default();

    for variant in variants {
        variant.trace_roots(&mut roots);
    }
    generic.trace_roots(&mut roots);

    assert_eq!(roots.exprs.len(), 4);
    assert_eq!(roots.symbols.len(), 2);
}

#[test]
fn matcher_binding_table_visits_the_exact_symbol_and_expression_roots() {
    let mut bindings = SymbolExprMap::default();
    bindings.insert(Symbol::from_name("binding"), Expr::symbol("binding-value"));
    let (symbol, expression) = bindings.iter().next().expect("binding should exist");
    let mut roots = RecordedRoots::default();

    bindings.trace_roots(&mut roots);

    assert_eq!(roots.exprs, vec![expression as *const Expr]);
    assert_eq!(roots.symbols, vec![symbol as *const Symbol]);
}

#[test]
fn recursive_matcher_arg_visits_every_nested_root_exactly_once() {
    let matcher = MatcherArg::PatternSequenceGroup(vec![
        MatcherArg::Single(MatcherNode::Named {
            name: Symbol::from_name("named-pattern"),
            inner: Box::new(MatcherNode::Exact(Expr::symbol("exact-pattern"))),
        }),
        MatcherArg::PatternSequenceGroup(vec![MatcherArg::Single(MatcherNode::Blank {
            head_constraint: Some(Expr::symbol("head-constraint")),
        })]),
    ]);
    let MatcherArg::PatternSequenceGroup(arguments) = &matcher else {
        unreachable!("test matcher should be a pattern-sequence group");
    };
    let MatcherArg::Single(MatcherNode::Named { name, inner }) = &arguments[0] else {
        unreachable!("first test matcher should be named");
    };
    let MatcherNode::Exact(exact) = inner.as_ref() else {
        unreachable!("named matcher should wrap an exact matcher");
    };
    let MatcherArg::PatternSequenceGroup(nested) = &arguments[1] else {
        unreachable!("second test matcher should be recursive");
    };
    let MatcherArg::Single(MatcherNode::Blank {
        head_constraint: Some(head_constraint),
    }) = &nested[0]
    else {
        unreachable!("recursive matcher should hold a head constraint");
    };
    let mut roots = RecordedRoots::default();

    matcher.trace_roots(&mut roots);

    assert_eq!(
        roots.exprs,
        vec![exact as *const Expr, head_constraint as *const Expr]
    );
    assert_eq!(roots.symbols, vec![name as *const Symbol]);
}

#[test]
fn association_entry_form_visits_key_and_value_roots_in_field_order() {
    let entry = AssociationEntryForm {
        key: Expr::symbol("association-key"),
        value: Expr::symbol("association-value"),
        delayed: true,
    };
    let mut roots = RecordedRoots::default();

    entry.trace_roots(&mut roots);

    assert_eq!(
        roots.exprs,
        vec![&entry.key as *const Expr, &entry.value as *const Expr]
    );
    assert!(roots.symbols.is_empty());
}

#[test]
fn block_symbol_state_map_visits_the_bare_symbol_key() {
    let mut states = BTreeMap::new();
    states.insert(Symbol::from_name("blocked"), BlockSymbolState::default());
    let symbol = states
        .first_key_value()
        .expect("block state should contain one symbol")
        .0;
    let mut roots = RecordedRoots::default();

    states.trace_roots(&mut roots);

    assert!(roots.exprs.is_empty());
    assert_eq!(roots.symbols, vec![symbol as *const Symbol]);
}

#[test]
fn file_io_error_is_the_zero_root_negative_calibration_target() {
    let error = FileIoError::new("calibration");
    let mut roots = RecordedRoots::default();

    error.trace_roots(&mut roots);

    println!("file_io_error_expr_roots={}", roots.exprs.len());
    assert_eq!(roots.exprs.len(), 0);
    assert!(roots.symbols.is_empty());
}

#[test]
fn derived_frame_root_trace_inventory_matches_the_closed_capture_graph() {
    assert!(
        DERIVED_FRAME_ROOT_TRACE_TYPES
            .windows(2)
            .all(|entries| entries[0] < entries[1]),
        "the derived FrameRootTrace inventory must stay sorted"
    );

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runtime crate should live under the workspace crates directory");
    let mut files = Vec::new();
    collect_rust_sources(&workspace.join("crates/rum-runtime/src"), &mut files);
    collect_rust_sources(&workspace.join("crates/rum-stdlib/src"), &mut files);

    let mut derived = files
        .iter()
        .flat_map(|path| derived_frame_root_trace_types(workspace, path))
        .collect::<Vec<_>>();
    derived.sort();
    let expected = DERIVED_FRAME_ROOT_TRACE_TYPES
        .iter()
        .map(|entry| (*entry).to_string())
        .collect::<Vec<_>>();

    assert_eq!(derived, expected);
}

#[test]
fn manual_frame_root_trace_implementations_match_sorted_allowlist() {
    assert!(
        MANUAL_FRAME_ROOT_TRACE_IMPLS
            .windows(2)
            .all(|entries| entries[0] < entries[1]),
        "the manual FrameRootTrace implementation allowlist must stay sorted"
    );

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runtime crate should live under the workspace crates directory");
    let mut files = Vec::new();
    collect_rust_sources(&workspace.join("crates/rum-runtime/src"), &mut files);
    collect_rust_sources(&workspace.join("crates/rum-stdlib/src"), &mut files);

    let mut implementations = files
        .iter()
        .flat_map(|path| manual_frame_root_trace_impls(path))
        .collect::<Vec<_>>();
    implementations.sort();

    assert_eq!(implementations, MANUAL_FRAME_ROOT_TRACE_IMPLS);
}

#[test]
fn suspension_protocol_repository_invariants_hold() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runtime crate should live under the workspace crates directory");
    let mut files = Vec::new();
    collect_rust_sources(&workspace.join("crates/rum-runtime/src"), &mut files);
    collect_rust_sources(&workspace.join("crates/rum-stdlib/src"), &mut files);

    let core_path = workspace.join("crates/rum-runtime/src/runtime/suspension/core.rs");
    let cleanup_path =
        workspace.join("crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs");
    let banned_legacy_names = [
        "ClosureFrame",
        "suspend_with_frame",
        "suspend_with_result_frame",
        "drive_evaluated_value",
        "drive_internal_value",
        "push_suspension_frame",
    ];
    let queue_mutations = [
        ".frames.push_back(",
        ".frames.push_front(",
        ".frames.pop_front(",
        ".frames.append(",
        ".frames.extend(",
        ".frames.insert(",
    ];
    let ownership_operations = [
        ".into_owned(",
        ".into_vec(",
        ".to_vec(",
        "Arc::try_unwrap(",
        "Arc::make_mut(",
    ];
    let mut frame_count = 0usize;
    let mut finalizer_count = 0usize;
    let mut pure_abandon_frames = Vec::new();
    let mut stateful_abandon_frames = Vec::new();
    let mut ownership_violations = Vec::new();

    for path in &files {
        let source = fs::read_to_string(path).expect("source gate should read Rust source");
        let compact = remove_ascii_whitespace(&source);

        for banned in banned_legacy_names {
            assert!(
                !source.contains(banned),
                "legacy suspension API name {banned} remains in {}",
                path.display()
            );
        }
        if path != &core_path {
            for mutation in queue_mutations {
                assert!(
                    !compact.contains(mutation),
                    "direct suspension queue mutation {mutation} escaped core in {}",
                    path.display()
                );
            }
            assert!(
                !compact.contains("insert_suspension_frame("),
                "named frames must be inserted only through the data-only macro in {}",
                path.display()
            );
            assert!(
                !compact.contains("FrameAbandonAction::EvaluateCleanup("),
                "EvaluateCleanup capability payload escaped core in {}",
                path.display()
            );
        }
        let cleanup_constructor_hits = compact
            .matches("FrameAbandonAction::evaluate_cleanup(")
            .count();
        if path == &cleanup_path {
            assert_eq!(
                cleanup_constructor_hits, 2,
                "only the two WithCleanup owner frames may request cleanup evaluation"
            );
        } else {
            assert_eq!(
                cleanup_constructor_hits,
                0,
                "cleanup evaluation construction escaped the WithCleanup allowlist in {}",
                path.display()
            );
        }

        for block in trait_impl_blocks(&source, "SuspensionFrame") {
            if block.header.contains("PureFrameFinalizer") {
                continue;
            }
            frame_count += 1;
            let type_name = implemented_type_name(block.header, "SuspensionFrame");
            let declaration = format!("struct {type_name}");
            let declaration_offset = source.find(&declaration).unwrap_or_else(|| {
                panic!(
                    "SuspensionFrame {type_name} must be a named struct in {}",
                    path.display()
                )
            });
            let derive_start = source[..declaration_offset]
                .rfind("#[derive(")
                .expect("named suspension frames should have a derive attribute");
            let derive_end = source[derive_start..declaration_offset]
                .find(")]")
                .map(|offset| derive_start + offset + 2)
                .expect("frame derive attribute should close before its declaration");
            let derive = &source[derive_start..derive_end];
            assert!(
                derive.contains("FrameRootTrace"),
                "SuspensionFrame {type_name} must derive FrameRootTrace in {}",
                path.display()
            );
            let relative_path = path
                .strip_prefix(workspace)
                .expect("suspension frame path should be inside the workspace")
                .display()
                .to_string()
                .replace('\\', "/");
            let inventory_row = format!("{relative_path}:{type_name}");
            let manual_abandon =
                trait_impl_blocks(&source, "FrameAbandon")
                    .iter()
                    .any(|implementation| {
                        implemented_type_name(implementation.header, "FrameAbandon") == type_name
                    });
            if derive.contains("PureFrameAbandon") {
                assert!(
                    !manual_abandon,
                    "pure frame {inventory_row} must not also hand-write FrameAbandon"
                );
                pure_abandon_frames.push(inventory_row);
            } else {
                assert!(
                    manual_abandon,
                    "stateful frame {inventory_row} must hand-write FrameAbandon"
                );
                stateful_abandon_frames.push(inventory_row);
            }
            let body = declared_struct_body(&source, declaration_offset)
                .expect("named frame declarations should have a body or be unit structs");
            for owned in directly_owned_frame_fields(body) {
                ownership_violations.push(format!(
                    "{}: SuspensionFrame {type_name} directly owns {owned}",
                    path.display()
                ));
            }
            let compact_block = remove_ascii_whitespace(block.body);
            for operation in ownership_operations {
                if compact_block.contains(operation) {
                    ownership_violations.push(format!(
                        "{}: SuspensionFrame {type_name} performs {operation} before drop",
                        path.display()
                    ));
                }
            }
        }

        for block in trait_impl_blocks(&source, "PureFrameFinalizer") {
            finalizer_count += 1;
            let type_name = implemented_type_name(block.header, "PureFrameFinalizer");
            let declaration = format!("struct {type_name}");
            let declaration_offset = source.find(&declaration).unwrap_or_else(|| {
                panic!(
                    "PureFrameFinalizer {type_name} must be a named struct in {}",
                    path.display()
                )
            });
            let line_start = source[..declaration_offset]
                .rfind('\n')
                .map_or(0, |offset| offset + 1);
            assert!(
                !source[line_start..declaration_offset].contains("pub"),
                "PureFrameFinalizer {type_name} must be internal in {}",
                path.display()
            );
            let compact_block = remove_ascii_whitespace(block.body);
            assert!(
                compact_block.contains("fnfinish(self:Box<Self>)->Result<Expr,EvalError>"),
                "PureFrameFinalizer {type_name} must use the closed Session-free signature in {}",
                path.display()
            );
            for forbidden in [
                "Session",
                ".evaluate(",
                "resume_or_suspend!",
                "insert_suspension_frame",
                "FrameAction",
                "InputRequired",
            ] {
                assert!(
                    !compact_block.contains(forbidden),
                    "PureFrameFinalizer {type_name} contains forbidden capability {forbidden} in {}",
                    path.display()
                );
            }
        }
    }

    let core = fs::read_to_string(&core_path).expect("source gate should read suspension core");
    let compact_core = remove_ascii_whitespace(&core);
    assert!(
        !compact_core.contains("Box<dynFn") && !compact_core.contains("Arc<dynFn"),
        "the suspension protocol must not contain a runtime closure factory"
    );
    assert!(compact_core.contains(
        "pubtraitSuspensionFrame:FrameRootTrace+FrameAbandon+Send+Sync+'static{fnresume(self:Arc<Self>,session:&mutSession,outcome:Result<Expr,EvalError>,)->FrameAction;"
    ));
    assert!(compact_core.contains(
        "pubtraitFrameAbandon{fnabandon(self:Arc<Self>,context:&mutAbandonmentContext<'_>)->FrameAbandonAction;"
    ));
    assert!(compact_core.contains(
        "pubtraitPureFrameFinalizer:Send+'static{fnfinish(self:Box<Self>)->Result<Expr,EvalError>;"
    ));
    let action_declaration = compact_core
        .find("pubenumFrameAction{")
        .expect("the frame transition protocol should be a closed enum");
    let action_open = action_declaration
        + compact_core[action_declaration..]
            .find('{')
            .expect("FrameAction should have a body");
    let action_close =
        matching_brace(&compact_core, action_open).expect("FrameAction should have a closed body");
    assert_eq!(
        &compact_core[action_open + 1..action_close],
        "Complete(Result<Expr,EvalError>),Evaluate{expr:Expr,next_frame:Arc<DynSuspensionFrame>,},Finalize(Box<dynPureFrameFinalizer>),"
    );
    let abandon_action_declaration = compact_core
        .find("pubenumFrameAbandonAction{")
        .expect("the abandonment transition protocol should be a closed enum");
    let abandon_action_open = abandon_action_declaration
        + compact_core[abandon_action_declaration..]
            .find('{')
            .expect("FrameAbandonAction should have a body");
    let abandon_action_close = matching_brace(&compact_core, abandon_action_open)
        .expect("FrameAbandonAction should have a closed body");
    assert_eq!(
        &compact_core[abandon_action_open + 1..abandon_action_close],
        "Discard,EvaluateCleanup(AbandonCleanup),"
    );
    let resume_index = compact_core
        .find("letaction=dispatch_frame.resume(self,outcome);")
        .expect("driver should invoke the still-rooted front frame");
    let drop_index = compact_core[resume_index..]
        .find("drop(completed_frame);")
        .map(|offset| offset + resume_index)
        .expect("driver should explicitly drop the rooted completed frame");
    let finalize_index = compact_core[drop_index..]
        .find("FrameAction::Finalize(finalizer)=>finalizer.finish()")
        .map(|offset| offset + drop_index)
        .expect("driver should invoke pure finalizers");
    assert!(resume_index < drop_index && drop_index < finalize_index);

    let abandon_driver = compact_core
        .find("fnabandon_pending_suspension(")
        .expect("the closed abandonment driver should exist");
    let abandon_driver_open = abandon_driver
        + compact_core[abandon_driver..]
            .find('{')
            .expect("abandonment driver should have a body");
    let abandon_driver_close = matching_brace(&compact_core, abandon_driver_open)
        .expect("abandonment driver should have a closed body");
    let abandon_driver_body = &compact_core[abandon_driver_open + 1..abandon_driver_close];
    assert!(abandon_driver_body.contains("frame.abandon("));
    assert!(
        !abandon_driver_body.contains(".resume("),
        "abandonment must never invoke ordinary frame resumption"
    );

    pure_abandon_frames.sort();
    stateful_abandon_frames.sort();
    assert!(
        PURE_FRAME_ABANDON_TYPES
            .windows(2)
            .all(|entries| entries[0] < entries[1]),
        "the pure abandonment inventory must stay sorted"
    );
    assert!(
        STATEFUL_FRAME_ABANDON_TYPES
            .windows(2)
            .all(|entries| entries[0] < entries[1]),
        "the stateful abandonment inventory must stay sorted"
    );
    assert_eq!(
        pure_abandon_frames,
        PURE_FRAME_ABANDON_TYPES
            .iter()
            .map(|entry| (*entry).to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        stateful_abandon_frames,
        STATEFUL_FRAME_ABANDON_TYPES
            .iter()
            .map(|entry| (*entry).to_string())
            .collect::<Vec<_>>()
    );

    println!("named_suspension_frames={frame_count}");
    println!("named_pure_finalizers={finalizer_count}");
    println!(
        "pure_frame_abandon_derives={}",
        PURE_FRAME_ABANDON_TYPES.len()
    );
    println!(
        "handwritten_stateful_frame_abandons={}",
        STATEFUL_FRAME_ABANDON_TYPES.len()
    );
    for entry in STATEFUL_FRAME_ABANDON_TYPES {
        println!("stateful_frame_abandon={entry}");
    }
    println!("evaluate_cleanup_allowlist=WithCleanup owner frames (2)");
    assert!(
        ownership_violations.is_empty(),
        "suspension ownership violations:\n{}",
        ownership_violations.join("\n")
    );
    println!("legacy_api_hits=0 direct_queue_mutation_hits=0 ownership_hits=0");
    assert_eq!(frame_count, 545);
    assert_eq!(
        frame_count,
        PURE_FRAME_ABANDON_TYPES.len() + STATEFUL_FRAME_ABANDON_TYPES.len()
    );
    assert!(finalizer_count > 0);
}

struct TraitImplBlock<'a> {
    header: &'a str,
    body: &'a str,
}

fn trait_impl_blocks<'a>(source: &'a str, trait_name: &str) -> Vec<TraitImplBlock<'a>> {
    let mut blocks = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative_impl) = source[cursor..].find("impl") {
        let impl_offset = cursor + relative_impl;
        let Some(relative_open) = source[impl_offset..].find('{') else {
            break;
        };
        let open = impl_offset + relative_open;
        let header = &source[impl_offset..open];
        let compact_header = remove_ascii_whitespace(header);
        if !compact_header.contains(&format!("{trait_name}for")) {
            cursor = open + 1;
            continue;
        }
        let close = matching_brace(source, open).unwrap_or_else(|| {
            panic!("trait implementation beginning at byte {impl_offset} must close")
        });
        blocks.push(TraitImplBlock {
            header,
            body: &source[open + 1..close],
        });
        cursor = close + 1;
    }
    blocks
}

fn implemented_type_name(header: &str, trait_name: &str) -> String {
    let compact = remove_ascii_whitespace(header);
    let marker = format!("{trait_name}for");
    let suffix = compact
        .split_once(&marker)
        .unwrap_or_else(|| panic!("implementation header must contain {marker}"))
        .1;
    suffix
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect()
}

fn declared_struct_body(source: &str, declaration_offset: usize) -> Option<&str> {
    let tail = &source[declaration_offset..];
    let semicolon = tail.find(';');
    let open = tail.find('{');
    if semicolon.is_some_and(|semicolon| open.is_none_or(|open| semicolon < open)) {
        return Some("");
    }
    let open = declaration_offset + open?;
    let close = matching_brace(source, open)?;
    Some(&source[open + 1..close])
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn remove_ascii_whitespace(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect()
}

fn directly_owned_frame_fields(body: &str) -> Vec<String> {
    let compact = remove_ascii_whitespace(body);
    compact
        .split(',')
        .filter_map(|field| {
            let bytes = field.as_bytes();
            let delimiter = bytes.iter().enumerate().find_map(|(index, byte)| {
                if *byte == b':'
                    && bytes.get(index.wrapping_sub(1)) != Some(&b':')
                    && bytes.get(index + 1) != Some(&b':')
                {
                    Some(index)
                } else {
                    None
                }
            })?;
            let field_type = &field[delimiter + 1..];
            [
                "Vec<",
                "std::vec::Vec<",
                "VecDeque<",
                "std::collections::VecDeque<",
                "String",
                "std::string::String",
                "Box<",
                "std::boxed::Box<",
            ]
            .iter()
            .any(|owned| field_type.starts_with(owned))
            .then(|| field_type.to_string())
        })
        .collect()
}

fn collect_rust_sources(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("source gate should read source directories") {
            let entry = entry.expect("source gate should read source entries");
            collect_rust_sources(&entry.path(), files);
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

fn manual_frame_root_trace_impls(path: &Path) -> Vec<String> {
    let source = fs::read_to_string(path).expect("source gate should read Rust source");
    let mut implementations = Vec::new();
    let mut header = None::<String>;

    for line in source.lines() {
        let trimmed = line.trim_start();
        if header.is_none() && trimmed.starts_with("impl") {
            header = Some(trimmed.to_string());
        } else if let Some(header) = &mut header {
            header.push(' ');
            header.push_str(trimmed);
        }

        let Some(current) = &header else {
            continue;
        };
        if !current.contains('{') {
            continue;
        }

        let normalized = current
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        if let Some((_, implementation)) = normalized.split_once("FrameRootTracefor") {
            let end = implementation
                .find("where")
                .or_else(|| implementation.find('{'))
                .expect("an impl header should end in a body");
            implementations.push(implementation[..end].to_string());
        }
        header = None;
    }

    implementations
}

fn derived_frame_root_trace_types(workspace: &Path, path: &Path) -> Vec<String> {
    let source = fs::read_to_string(path).expect("source gate should read Rust source");
    let relative_path = path
        .strip_prefix(workspace)
        .expect("source path should be inside the workspace")
        .display()
        .to_string()
        .replace('\\', "/");
    let mut types = Vec::new();
    let mut derive_attribute = String::new();
    let mut reading_derive = false;
    let mut derives_frame_root_trace = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if !reading_derive && trimmed.starts_with("#[derive(") {
            derive_attribute.clear();
            derive_attribute.push_str(trimmed);
            reading_derive = !trimmed.ends_with(")]");
            if !reading_derive {
                derives_frame_root_trace = derive_attribute.contains("FrameRootTrace");
            }
            continue;
        }

        if reading_derive {
            derive_attribute.push_str(trimmed);
            if trimmed.ends_with(")]") {
                reading_derive = false;
                derives_frame_root_trace = derive_attribute.contains("FrameRootTrace");
            }
            continue;
        }

        if !derives_frame_root_trace {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("#[") {
            continue;
        }

        if let Some(type_name) = declared_type_name(trimmed) {
            types.push(format!("{relative_path}:{type_name}"));
        }
        derives_frame_root_trace = false;
    }

    types
}

fn declared_type_name(declaration: &str) -> Option<&str> {
    let (_, name) = ["struct ", "enum "]
        .into_iter()
        .filter_map(|keyword| {
            declaration
                .find(keyword)
                .map(|index| (index, &declaration[index + keyword.len()..]))
        })
        .min_by_key(|(index, _)| *index)?;

    let name_len = name
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .map(char::len_utf8)
        .sum::<usize>();
    (name_len > 0).then_some(&name[..name_len])
}
