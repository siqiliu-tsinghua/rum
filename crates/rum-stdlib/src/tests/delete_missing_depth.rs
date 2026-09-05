use super::*;
use rum_core::Atom;
use rum_runtime::kernel_api::{
    AssociationEntryForm, association_entry_forms_from_expr, list_items, system_expr,
};

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
const DELETE_MISSING_FIXTURE_N: usize = 2_048;
const RECURSIVE_ASSOCIATION_LEVELS: usize = DELETE_MISSING_FIXTURE_N - 1;
const RECURSIVE_OUTER_ASSOCIATIONS: usize = RECURSIVE_ASSOCIATION_LEVELS - 1;
const SCAN_ASSOCIATION_LEVELS: usize = DELETE_MISSING_FIXTURE_N - 1;
const SCAN_OUTER_ASSOCIATIONS: usize = SCAN_ASSOCIATION_LEVELS - 1;

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB DeleteMissing certificate thread should spawn")
        .join();
    assert!(
        joined.is_ok(),
        "DeleteMissing certificate {name:?} panicked"
    );
}

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn construct_recursive_fixture(
    session: &mut Session,
    target: &str,
    sentinel: &str,
    include_delayed: bool,
) -> Expr {
    let survivor_rule = if include_delayed { ":>" } else { "->" };
    let source = format!(
        "{target} = Association[\"drop\" -> Missing[], \
         \"survivor\" {survivor_rule} \"{sentinel}\"]; \
         Do[{target} = Association[\"k\" -> {target}], \
            {{i, {RECURSIVE_OUTER_ASSOCIATIONS}}}]; \
         {target} = {{{target}}}"
    );
    session
        .eval_text(&source)
        .expect("deep recursive DeleteMissing fixture should construct")
        .value
}

fn expected_recursive_innermost(
    session: &mut Session,
    sentinel: &str,
    include_delayed: bool,
) -> Vec<AssociationEntryForm> {
    let survivor_rule = if include_delayed { ":>" } else { "->" };
    let source = format!(
        "DeleteMissing[Association[\"drop\" -> Missing[], \
         \"survivor\" {survivor_rule} \"{sentinel}\"], Infinity]"
    );
    let value = session
        .eval_text(&source)
        .expect("shallow expected DeleteMissing centre should evaluate")
        .value;
    association_entry_forms_from_expr(&value)
        .expect("shallow expected DeleteMissing centre should be an association")
}

fn assert_raw_recursive_fixture(expr: &Expr, sentinel: &str) {
    let mut work = vec![(expr, 1usize)];
    let mut max_depth = 0usize;
    let mut sentinel_depths = Vec::new();
    let mut deepest_missing_heads = Vec::new();

    while let Some((current, depth)) = work.pop() {
        max_depth = max_depth.max(depth);
        match current {
            Expr::Call { head, args } => {
                work.push((head, depth + 1));
                work.extend(args.iter().map(|arg| (arg, depth + 1)));
            }
            Expr::Atom(Atom::Symbol(symbol))
                if symbol.context() == "System`" && symbol.name() == "Missing" =>
            {
                deepest_missing_heads.push(depth);
            }
            Expr::Atom(Atom::String(value)) if value == sentinel => {
                sentinel_depths.push(depth);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_depth, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(
        sentinel_depths,
        vec![TARGET_PHYSICAL_HEIGHT - 1],
        "the deepest association must have one distinguishing survivor"
    );
    assert_eq!(deepest_missing_heads, vec![TARGET_PHYSICAL_HEIGHT]);
}

fn assert_exact_recursive_result(mut current: Expr, expected_innermost: Vec<AssociationEntryForm>) {
    let items = list_items(&current).expect("recursive DeleteMissing result must remain a list");
    assert_eq!(items.len(), 1);
    current = items[0].clone();

    for level in 0..RECURSIVE_OUTER_ASSOCIATIONS {
        let mut entries = association_entry_forms_from_expr(&current).unwrap_or_else(|| {
            panic!("recursive DeleteMissing level {level} must be an association")
        });
        assert_eq!(
            entries.len(),
            1,
            "outer level {level} must retain one entry"
        );
        let entry = entries.pop().expect("one outer entry should exist");
        assert_eq!(entry.key, Expr::string("k"));
        assert!(!entry.delayed, "outer Rule head must remain immediate");
        current = entry.value;
    }

    let actual_innermost = association_entry_forms_from_expr(&current)
        .expect("recursive DeleteMissing must propagate its innermost association");
    assert_eq!(actual_innermost, expected_innermost);
}

fn construct_scan_fixture(session: &mut Session) -> Expr {
    let source = format!(
        "deleteMissingScanBranch = Association[\
             \"drop\" -> Missing[], \
             \"survivor\" -> \"deep-scan-survivor\"]; \
         Do[deleteMissingScanBranch = Association[\"k\" -> deleteMissingScanBranch], \
            {{i, {SCAN_OUTER_ASSOCIATIONS}}}]; \
         deleteMissingScanInput = {{deleteMissingScanBranch, \"scan-kept\"}}"
    );
    session
        .eval_text(&source)
        .expect("deep missing-scan fixture should construct")
        .value
}

fn assert_raw_scan_fixture(expr: &Expr) {
    let mut work = vec![(expr, 1usize)];
    let mut max_depth = 0usize;
    let mut deepest_missing_heads = Vec::new();
    let mut survivor_depths = Vec::new();

    while let Some((current, depth)) = work.pop() {
        max_depth = max_depth.max(depth);
        match current {
            Expr::Call { head, args } => {
                work.push((head, depth + 1));
                work.extend(args.iter().map(|arg| (arg, depth + 1)));
            }
            Expr::Atom(Atom::Symbol(symbol))
                if symbol.context() == "System`" && symbol.name() == "Missing" =>
            {
                deepest_missing_heads.push(depth);
            }
            Expr::Atom(Atom::String(value)) if value == "deep-scan-survivor" => {
                survivor_depths.push(depth);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_depth, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(deepest_missing_heads, vec![TARGET_PHYSICAL_HEIGHT]);
    assert_eq!(survivor_depths, vec![TARGET_PHYSICAL_HEIGHT - 1]);
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn delete_missing_recursive_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("delete-missing-recursive-depth-4097", || {
        let mut session = core_session();
        let sentinel = "deleteMissingRecursiveDeepSurvivor";
        let input = construct_recursive_fixture(
            &mut session,
            "deleteMissingRecursiveInput",
            sentinel,
            false,
        );
        assert_raw_recursive_fixture(&input, sentinel);
        let expected = expected_recursive_innermost(&mut session, sentinel, false);
        drop(input);

        let outcome = session
            .eval_text("DeleteMissing[deleteMissingRecursiveInput, Infinity]")
            .expect("deep recursive DeleteMissing should complete");
        assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
        assert_exact_recursive_result(outcome.value, expected);
    });
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn delete_missing_recursive_armed_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("delete-missing-recursive-armed-depth-4097", || {
        let mut session = core_session();
        let sentinel = "deleteMissingArmedDeepSurvivor";
        let input =
            construct_recursive_fixture(&mut session, "deleteMissingArmedInput", sentinel, true);
        assert_raw_recursive_fixture(&input, sentinel);
        let expected = expected_recursive_innermost(&mut session, sentinel, true);
        drop(input);

        let outcome = session
            .eval_text(
                "MemoryConstrained[\
                    DeleteMissing[deleteMissingArmedInput, Infinity], \
                    MemoryInUse[] + 1000000000]",
            )
            .expect("armed deep recursive DeleteMissing should complete");
        assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
        assert_exact_recursive_result(outcome.value, expected);
    });
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn delete_missing_scan_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("delete-missing-scan-depth-4097", || {
        let mut session = core_session();
        let input = construct_scan_fixture(&mut session);
        assert_raw_scan_fixture(&input);
        drop(input);

        let outcome = session
            .eval_text("DeleteMissing[deleteMissingScanInput, 1, All]")
            .expect("deep DeleteMissing scan should complete");
        assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
        assert_eq!(
            outcome.value,
            Expr::call(system_expr("List"), vec![Expr::string("scan-kept")])
        );
    });
}

#[test]
fn delete_missing_recursive_shallow_companion_pins_level_interpretation() {
    let mut session = core_session();
    let source = "x = Association[\"drop\" -> Missing[], \"survivor\" -> 7]; \
                  x = Association[\"k\" -> x]; \
                  x = Association[\"k\" -> x]; \
                  {DeleteMissing[x, 2], DeleteMissing[x, 3]}";
    assert_eq!(
        session.eval_text(source).unwrap().value.to_string(),
        "List[Association[Rule[\"k\", Association[Rule[\"k\", Association[Rule[\"drop\", Missing[]], Rule[\"survivor\", 7]]]]]], Association[Rule[\"k\", Association[Rule[\"k\", Association[Rule[\"survivor\", 7]]]]]]]"
    );
}

#[test]
fn delete_missing_recursive_armed_shallow_companion_pins_level_and_delayed_head() {
    let mut session = core_session();
    let source = "x = Association[\"drop\" -> Missing[], \"survivor\" :> 7]; \
                  x = Association[\"k\" -> x]; \
                  x = Association[\"k\" -> x]; \
                  MemoryConstrained[DeleteMissing[x, 3], MemoryInUse[] + 1000000000]";
    let value = session.eval_text(source).unwrap().value;
    let mut current = value;
    for _ in 0..2 {
        let mut entries = association_entry_forms_from_expr(&current).unwrap();
        assert_eq!(entries.len(), 1);
        current = entries.pop().unwrap().value;
    }
    let entries = association_entry_forms_from_expr(&current).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, Expr::string("survivor"));
    assert!(entries[0].delayed, "RuleDelayed head must be preserved");
}

#[test]
fn delete_missing_scan_shallow_companion_pins_scan_depth() {
    let mut session = core_session();
    let source = "{\
        DeleteMissing[{f[g[Missing[]]], f[g[scanSurvivor]]}, 1, 1], \
        DeleteMissing[{f[g[Missing[]]], f[g[scanSurvivor]]}, 1, 2]\
    }";
    assert_eq!(
        session.eval_text(source).unwrap().value.to_string(),
        "List[List[f[g[Missing[]]], f[g[scanSurvivor]]], List[f[g[scanSurvivor]]]]"
    );
}
