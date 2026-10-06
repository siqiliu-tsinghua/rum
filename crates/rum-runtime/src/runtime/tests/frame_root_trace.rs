use std::fs;
use std::path::{Path, PathBuf};

use crate::runtime::suspension::trace::MANUAL_FRAME_ROOT_TRACE_IMPLS;

mod derived_inventory;
mod root_visitation;
use derived_inventory::{DERIVED_FRAME_ROOT_TRACE_TYPES, PURE_FRAME_ABANDON_TYPES};

const STATEFUL_FRAME_ABANDON_TYPES: &[&str] = &[
    "crates/rum-runtime/src/runtime/builtins_context_loader/resume.rs:BeginPackageDependencyFrame",
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
    "crates/rum-runtime/src/runtime/session/batch_file.rs:PublicFileRollbackFrame",
    "crates/rum-runtime/src/runtime/session/guest_loading/file.rs:GuestFileCloseFrame",
    "crates/rum-runtime/src/runtime/session/guest_loading/script.rs:GuestScriptFormFrame",
    "crates/rum-runtime/src/runtime/session/session_api/resume_frames.rs:EvaluateWithRandomStateRestoredSuspensionFrame",
];

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

    let mut expected = MANUAL_FRAME_ROOT_TRACE_IMPLS.to_vec();
    expected.push("AccumulatorChain<N>");
    expected.sort_unstable();
    assert_eq!(implementations, expected);
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
    let mut repository_files = Vec::new();
    collect_repository_files(&workspace.join("crates"), &mut repository_files);
    let deleted_cancellation_names = [
        ["interrupt", "_flag"].concat(),
        ["interrupt", "_handle"].concat(),
        ["legacy", "_interrupt"].concat(),
        ["legacy", "_handle"].concat(),
        ["synchronize", "_legacy_", "interrupt"].concat(),
        ["suppress", "_legacy_", "interrupt"].concat(),
    ];
    let mut cancellation_lane_violations = Vec::new();
    for path in &repository_files {
        let source = fs::read_to_string(path).expect("source gate should read Rust source");
        for name in &deleted_cancellation_names {
            if source.contains(name) {
                cancellation_lane_violations.push(format!("{}:{name}", path.display()));
            }
        }
    }
    assert!(
        cancellation_lane_violations.is_empty(),
        "deleted cancellation lane names returned:\n{}",
        cancellation_lane_violations.join("\n")
    );

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
    let raw_clear_name = ["clear_pending_", "suspension_state"].concat();
    let raw_owner_option = ["pending_suspension:Option<", "SuspendedEvaluation>"].concat();
    let raw_owner_replacements = [
        ["self.pending_", "suspension="].concat(),
        [".pending_", "suspension.take()"].concat(),
        ["mem::take(&mutself.pending_", "suspension)"].concat(),
        ["mem::replace(&mutself.pending_", "suspension"].concat(),
    ];
    let mut frame_count = 0usize;
    let mut finalizer_count = 0usize;
    let mut pure_abandon_frames = Vec::new();
    let mut stateful_abandon_frames = Vec::new();
    let mut ownership_violations = Vec::new();
    let mut raw_live_owner_clear_hits = 0usize;
    let mut raw_live_owner_replacement_hits = 0usize;

    for path in &files {
        let source = fs::read_to_string(path).expect("source gate should read Rust source");
        let compact = remove_ascii_whitespace(&source);

        raw_live_owner_clear_hits += source.matches(raw_clear_name.as_str()).count();
        if path != &core_path {
            raw_live_owner_replacement_hits += compact.matches(raw_owner_option.as_str()).count();
            for replacement in &raw_owner_replacements {
                raw_live_owner_replacement_hits += compact.matches(replacement.as_str()).count();
            }
        }

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
    assert_eq!(
        raw_live_owner_clear_hits, 0,
        "the deleted raw live-owner clear API returned"
    );
    assert_eq!(
        raw_live_owner_replacement_hits, 0,
        "raw live-owner replacement escaped suspension/core.rs"
    );
    println!("raw_live_owner_clears={raw_live_owner_clear_hits}");
    println!("raw_live_owner_replacements={raw_live_owner_replacement_hits}");
    println!(
        "removed_cancellation_lane_hits={}",
        cancellation_lane_violations.len()
    );
    assert!(
        ownership_violations.is_empty(),
        "suspension ownership violations:\n{}",
        ownership_violations.join("\n")
    );
    println!("legacy_api_hits=0 direct_queue_mutation_hits=0 ownership_hits=0");
    // The option loop is now continuation data inside the existing retained
    // frame; its roots remain in the derived inventory above.
    assert_eq!(frame_count, 561);
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

fn collect_repository_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("source gate should read source directories") {
            let entry = entry.expect("source gate should read source entries");
            collect_repository_files(&entry.path(), files);
        }
        return;
    }

    files.push(path.to_path_buf());
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
