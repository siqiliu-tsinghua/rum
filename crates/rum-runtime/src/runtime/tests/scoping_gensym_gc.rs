use super::*;

mod reachability;
mod suspension_frames;

fn session_with_test_first() -> Session {
    let mut session = session_with_test_control_builtins();
    crate::runtime::test_builtins::register_test_first_builtin(&mut session);
    session
}

fn session_with_test_gc_probe_builtins() -> Session {
    let mut session = session_with_test_control_builtins();
    crate::runtime::test_builtins::register_test_sameq_builtin(&mut session);
    session
}

#[test]
fn all_live_modules_run_one_sweep_before_the_next_watermark() {
    const SUBCASE: &str = "all-live-counter";
    if !in_gensym_gc_watermark_subprocess(SUBCASE) {
        run_gensym_gc_watermark_subprocess(
            "all_live_modules_run_one_sweep_before_the_next_watermark",
            SUBCASE,
            3,
        );
        return;
    }

    assert_eq!(gensym_gc_threshold(), 3);
    let mut session = session_with_test_control_builtins();
    let candidates = (0..5)
        .map(|index| {
            allocate_rooted_candidate(
                &mut session,
                &format!("gcWatermarkCounterRoot{index}"),
                index,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(session.gensym_gc_sweep_count, 1);
    assert_eq!(session.gc_candidate_gensym_count(), 5);
    assert_eq!(session.gensym_gc_watermark, 6);
    assert_candidate_definition_count(&session, &candidates, 5);
}

#[test]
fn unreachable_candidates_are_reclaimed_at_the_next_watermark_boundary() {
    const SUBCASE: &str = "reclamation-bound";
    if !in_gensym_gc_watermark_subprocess(SUBCASE) {
        run_gensym_gc_watermark_subprocess(
            "unreachable_candidates_are_reclaimed_at_the_next_watermark_boundary",
            SUBCASE,
            3,
        );
        return;
    }

    let mut session = session_with_test_control_builtins();
    let old_candidates = (0..3)
        .map(|index| {
            allocate_rooted_candidate(&mut session, &format!("gcBoundOldRoot{index}"), index)
        })
        .collect::<Vec<_>>();
    assert_eq!(session.gensym_gc_sweep_count, 1);
    assert_eq!(session.gensym_gc_watermark, 6);

    session
        .eval_text("ClearAll[gcBoundOldRoot0, gcBoundOldRoot1, gcBoundOldRoot2]")
        .unwrap();
    let new_candidates_before_boundary = (0..2)
        .map(|index| {
            allocate_rooted_candidate(&mut session, &format!("gcBoundNewRoot{index}"), 10 + index)
        })
        .collect::<Vec<_>>();

    assert_eq!(session.gc_candidate_gensym_count(), 5);
    assert_eq!(session.gensym_gc_sweep_count, 1);
    assert_candidate_definition_count(&session, &old_candidates, 3);

    let boundary_candidate = allocate_rooted_candidate(&mut session, "gcBoundNewRoot2", 12);

    assert_eq!(session.gensym_gc_sweep_count, 2);
    assert_candidate_definition_count(&session, &old_candidates, 0);
    assert_candidate_definition_count(&session, &new_candidates_before_boundary, 2);
    assert_candidate_definition_count(&session, &[boundary_candidate], 1);
    assert!(
        old_candidates
            .iter()
            .all(|candidate| !session.gc_candidate_gensyms.contains(candidate))
    );
}

#[test]
fn partial_sweep_rearms_from_the_post_sweep_candidate_count() {
    const SUBCASE: &str = "partial-sweep";
    if !in_gensym_gc_watermark_subprocess(SUBCASE) {
        run_gensym_gc_watermark_subprocess(
            "partial_sweep_rearms_from_the_post_sweep_candidate_count",
            SUBCASE,
            3,
        );
        return;
    }

    let mut session = session_with_test_control_builtins();
    let dead = allocate_rooted_candidate(&mut session, "gcPartialDeadRoot", 1);
    let live = allocate_rooted_candidate(&mut session, "gcPartialLiveRoot", 2);
    session.eval_text("ClearAll[gcPartialDeadRoot]").unwrap();
    let boundary = allocate_rooted_candidate(&mut session, "gcPartialBoundaryRoot", 3);

    assert_eq!(session.gensym_gc_sweep_count, 1);
    assert_eq!(session.gc_candidate_gensym_count(), 2);
    assert_eq!(session.gensym_gc_watermark, 5);
    assert_candidate_definition_count(&session, &[dead], 0);
    assert_candidate_definition_count(&session, &[live, boundary], 2);

    allocate_rooted_candidate(&mut session, "gcPartialNextRoot0", 4);
    allocate_rooted_candidate(&mut session, "gcPartialNextRoot1", 5);
    assert_eq!(session.gc_candidate_gensym_count(), 4);
    assert_eq!(session.gensym_gc_sweep_count, 1);

    allocate_rooted_candidate(&mut session, "gcPartialNextRoot2", 6);
    assert_eq!(session.gensym_gc_sweep_count, 2);
}

#[test]
fn clear_all_candidate_shrink_rearms_the_watermark() {
    const SUBCASE: &str = "clear-all-shrink";
    if !in_gensym_gc_watermark_subprocess(SUBCASE) {
        run_gensym_gc_watermark_subprocess(
            "clear_all_candidate_shrink_rearms_the_watermark",
            SUBCASE,
            3,
        );
        return;
    }

    let mut session = session_with_test_control_builtins();
    let candidates = (0..3)
        .map(|index| {
            allocate_rooted_candidate(&mut session, &format!("gcClearAllRoot{index}"), index)
        })
        .collect::<Vec<_>>();
    let cleared = candidates[0];
    session.eval_text(&format!("ClearAll[{cleared}]")).unwrap();

    assert_eq!(session.gc_candidate_gensym_count(), 2);
    assert_eq!(session.gensym_gc_watermark, 5);
    assert!(!session.definitions.contains_key(&cleared));

    allocate_rooted_candidate(&mut session, "gcClearAllNextRoot0", 10);
    allocate_rooted_candidate(&mut session, "gcClearAllNextRoot1", 11);
    assert_eq!(session.gensym_gc_sweep_count, 1);
    allocate_rooted_candidate(&mut session, "gcClearAllNextRoot2", 12);
    assert_eq!(session.gensym_gc_sweep_count, 2);
}

#[test]
fn remove_candidate_shrink_rearms_the_watermark() {
    const SUBCASE: &str = "remove-shrink";
    if !in_gensym_gc_watermark_subprocess(SUBCASE) {
        run_gensym_gc_watermark_subprocess(
            "remove_candidate_shrink_rearms_the_watermark",
            SUBCASE,
            3,
        );
        return;
    }

    let mut session = session_with_test_control_builtins();
    let candidates = (0..3)
        .map(|index| {
            allocate_rooted_candidate(&mut session, &format!("gcRemoveRoot{index}"), index)
        })
        .collect::<Vec<_>>();
    let removed = candidates[0];
    session.eval_text(&format!("Remove[{removed}]")).unwrap();

    assert_eq!(session.gc_candidate_gensym_count(), 2);
    assert_eq!(session.gensym_gc_watermark, 5);
    assert!(!session.definitions.contains_key(&removed));
    assert!(!session.gc_candidate_gensyms.contains(&removed));

    allocate_rooted_candidate(&mut session, "gcRemoveNextRoot0", 10);
    allocate_rooted_candidate(&mut session, "gcRemoveNextRoot1", 11);
    assert_eq!(session.gensym_gc_sweep_count, 1);
    allocate_rooted_candidate(&mut session, "gcRemoveNextRoot2", 12);
    assert_eq!(session.gensym_gc_sweep_count, 2);
}

#[test]
fn loader_snapshot_candidate_shrink_rearms_the_watermark() {
    const SUBCASE: &str = "snapshot-shrink";
    if !in_gensym_gc_watermark_subprocess(SUBCASE) {
        run_gensym_gc_watermark_subprocess(
            "loader_snapshot_candidate_shrink_rearms_the_watermark",
            SUBCASE,
            3,
        );
        return;
    }

    let mut session = session_with_test_control_builtins();
    let restored_candidates = vec![
        allocate_rooted_candidate(&mut session, "gcSnapshotRoot0", 1),
        allocate_rooted_candidate(&mut session, "gcSnapshotRoot1", 2),
    ];
    let snapshot = session.capture_loader_snapshot();
    let rolled_back_candidates = vec![
        allocate_rooted_candidate(&mut session, "gcSnapshotRolledBackRoot0", 3),
        allocate_rooted_candidate(&mut session, "gcSnapshotRolledBackRoot1", 4),
    ];
    assert_eq!(session.gensym_gc_sweep_count, 1);
    assert_eq!(session.gc_candidate_gensym_count(), 4);

    session.restore_loader_snapshot(snapshot);

    assert_eq!(session.gc_candidate_gensym_count(), 2);
    assert_eq!(session.gensym_gc_watermark, 5);
    assert_candidate_definition_count(&session, &restored_candidates, 2);
    assert_candidate_definition_count(&session, &rolled_back_candidates, 0);
    assert!(
        rolled_back_candidates
            .iter()
            .all(|candidate| !session.gc_candidate_gensyms.contains(candidate))
    );

    allocate_rooted_candidate(&mut session, "gcSnapshotNextRoot0", 10);
    allocate_rooted_candidate(&mut session, "gcSnapshotNextRoot1", 11);
    assert_eq!(session.gensym_gc_sweep_count, 1);
    allocate_rooted_candidate(&mut session, "gcSnapshotNextRoot2", 12);
    assert_eq!(session.gensym_gc_sweep_count, 2);
}

#[test]
fn immediate_module_reclaim_candidate_shrink_rearms_the_watermark() {
    const SUBCASE: &str = "module-reclaim-shrink";
    if !in_gensym_gc_watermark_subprocess(SUBCASE) {
        run_gensym_gc_watermark_subprocess(
            "immediate_module_reclaim_candidate_shrink_rearms_the_watermark",
            SUBCASE,
            3,
        );
        return;
    }

    let mut session = session_with_test_control_builtins();
    let stale = Symbol::from_name("gcImmediateModule$1");
    session.gc_candidate_gensyms.insert(stale);
    session.gensym_gc_watermark = usize::MAX;

    assert_eq!(
        eval_string(&mut session, "Module[{gcImmediateModule}, 0]"),
        "0"
    );

    assert!(session.gc_candidate_gensyms.is_empty());
    assert_eq!(session.gensym_gc_watermark, 3);
    assert_eq!(session.gensym_gc_sweep_count, 0);
}

#[test]
fn gensym_gc_watermark_arithmetic_saturates() {
    assert_eq!(gensym_gc_watermark(0, usize::MAX), usize::MAX);
    assert_eq!(gensym_gc_watermark(usize::MAX - 1, 2), usize::MAX);
    assert_eq!(gensym_gc_watermark(usize::MAX, usize::MAX), usize::MAX);
}

#[test]
fn gensym_gc_threshold_parses_env_in_fresh_processes() {
    if let Ok(subcase) = std::env::var("RUM_GENSYM_GC_THRESHOLD_SUBCASE") {
        match subcase.as_str() {
            "valid" => assert_eq!(gensym_gc_threshold(), 3),
            "max" => assert_eq!(gensym_gc_threshold(), usize::MAX),
            "invalid" | "zero" | "default" => assert_eq!(gensym_gc_threshold(), 200_000),
            other => panic!("unknown threshold subcase {other}"),
        }
        return;
    }

    let current_exe = std::env::current_exe().unwrap();
    let test_name = format!(
        "{}::gensym_gc_threshold_parses_env_in_fresh_processes",
        module_path!()
            .strip_prefix("rum_runtime::")
            .unwrap_or(module_path!())
    );
    let max_threshold = usize::MAX.to_string();
    for (subcase, threshold) in [
        ("valid", Some("3")),
        ("max", Some(max_threshold.as_str())),
        ("invalid", Some("not-a-number")),
        ("zero", Some("0")),
        ("default", None),
    ] {
        let mut command = std::process::Command::new(&current_exe);
        command
            .arg("--exact")
            .arg(&test_name)
            .arg("--test-threads=1")
            .env("RUM_GENSYM_GC_THRESHOLD_SUBCASE", subcase)
            .env_remove("RUM_GENSYM_GC_THRESHOLD");
        if let Some(threshold) = threshold {
            command.env("RUM_GENSYM_GC_THRESHOLD", threshold);
        }

        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "threshold subcase {subcase} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

const GENSYM_GC_WATERMARK_SUBCASE_ENV: &str = "RUM_GENSYM_GC_WATERMARK_SUBCASE";

fn in_gensym_gc_watermark_subprocess(expected_subcase: &str) -> bool {
    let Ok(actual_subcase) = std::env::var(GENSYM_GC_WATERMARK_SUBCASE_ENV) else {
        return false;
    };
    assert_eq!(actual_subcase, expected_subcase);
    true
}

fn run_gensym_gc_watermark_subprocess(test_function: &str, subcase: &str, threshold: usize) {
    let current_exe = std::env::current_exe().unwrap();
    let test_name = format!(
        "{}::{test_function}",
        module_path!()
            .strip_prefix("rum_runtime::")
            .unwrap_or(module_path!())
    );
    let output = std::process::Command::new(current_exe)
        .arg("--exact")
        .arg(&test_name)
        .arg("--test-threads=1")
        .env(GENSYM_GC_WATERMARK_SUBCASE_ENV, subcase)
        .env("RUM_GENSYM_GC_THRESHOLD", threshold.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "watermark subcase {subcase} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn eval_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn assert_definition_escape_kept_then_reclaimed(
    label: &str,
    setup_source: &str,
    verify_source: &str,
    clear_source: &str,
) {
    let mut session = session_with_test_first();

    assert_eq!(eval_string(&mut session, setup_source), "0");
    let candidates = retained_candidates(&session);
    assert_eq!(candidates.len(), 1, "{label} should retain one candidate");

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, verify_source), "7");
    assert_candidate_definition_count(&session, &candidates, 1);

    session.eval_text(clear_source).unwrap();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &candidates, 0);
    assert!(
        candidates
            .iter()
            .all(|candidate| !session.gc_candidate_gensyms.contains(candidate)),
        "{label} candidate should be removed from the candidate set after sweep"
    );
}

fn assert_outcome_roots_keep_then_drop_candidates<F>(label: &str, count: usize, make_outcome: F)
where
    F: FnOnce(&[Expr]) -> Result<Expr, EvalError>,
{
    let mut session = session_with_test_control_builtins();
    let candidates = (0..count)
        .map(|index| allocate_retained_candidate(&mut session, index as i64))
        .collect::<Vec<_>>();
    let exprs = candidates
        .iter()
        .copied()
        .map(symbol_expr)
        .collect::<Vec<_>>();
    let outcome = make_outcome(&exprs);
    let roots = gensym_gc_extra_roots_for_outcome(&outcome);

    assert_eq!(
        roots.len(),
        count,
        "{label} should expose every outcome value as an extra root"
    );
    assert_candidate_definition_count(&session, &candidates, count);

    session.collect_and_sweep_dead_gensyms(&roots);

    assert_candidate_definition_count(&session, &candidates, count);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &candidates, 0);
    assert!(
        candidates
            .iter()
            .all(|candidate| !session.gc_candidate_gensyms.contains(candidate)),
        "{label} candidates should be removed from the candidate set after the extra roots drop"
    );
}

fn allocate_retained_candidate(session: &mut Session, value: i64) -> Symbol {
    let before = session.gc_candidate_gensyms.clone();
    let source = format!("Module[{{x}}, x = {value}; Hold[x]]");
    session.eval_text(&source).unwrap();
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .filter(|candidate| !before.contains(candidate))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);
    assert!(session.definitions.contains_key(&candidates[0]));
    candidates[0]
}

fn allocate_rooted_candidate(session: &mut Session, root_name: &str, value: i64) -> Symbol {
    let before = session.gc_candidate_gensyms.clone();
    let source =
        format!("ClearAll[{root_name}]; Module[{{x}}, x = {value}; {root_name} = Hold[x]; 0]");
    assert_eq!(eval_string(session, &source), "0");
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .filter(|candidate| !before.contains(candidate))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);
    assert!(session.definitions.contains_key(&candidates[0]));
    candidates[0]
}

fn retained_candidates(session: &Session) -> Vec<Symbol> {
    session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>()
}

fn symbol_expr(symbol: Symbol) -> Expr {
    Expr::Atom(Atom::Symbol(symbol))
}

fn assert_candidate_definition_count(session: &Session, candidates: &[Symbol], count: usize) {
    assert_eq!(
        candidates
            .iter()
            .filter(|candidate| session.definitions.contains_key(candidate))
            .count(),
        count
    );
}

fn assert_module_gensym_reclaimed(session: &Session, name: &str) {
    let gensym = Symbol::from_name(name);
    assert!(!session.definitions.contains_key(&gensym));
    assert!(!session.gc_candidate_gensyms.contains(&gensym));
}

fn assert_module_gensym_deferred(session: &Session, name: &str) {
    let gensym = Symbol::from_name(name);
    assert!(session.definitions.contains_key(&gensym));
    assert!(session.gc_candidate_gensyms.contains(&gensym));
}
