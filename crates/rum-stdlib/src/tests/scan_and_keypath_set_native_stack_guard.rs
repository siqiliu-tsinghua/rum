#[cfg(not(debug_assertions))]
use std::thread;

#[cfg(not(debug_assertions))]
use rum_core::Atom;
use rum_core::Expr;
use rum_runtime::Session;
use rum_runtime::kernel_api::system_expr;

use crate::install_core;

#[cfg(not(debug_assertions))]
const ASSOCIATION_WRAPPER_COUNT: usize = 2_048;
#[cfg(not(debug_assertions))]
const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
#[cfg(not(debug_assertions))]
const SCAN_DEEPEST_SENTINEL: &str = "scanDeepestSentinel";

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn assert_stack_limit_exceeded(outcome: &rum_runtime::EvalOutcome) {
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"StackLimit\"]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "General"
            && message.tag == "reslim"
            && message.text == "evaluation exceeded native stack limit (0 bytes)"
    }));
}

#[cfg(not(debug_assertions))]
fn run_on_certificate_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("the 2 MiB certificate thread should start")
        .join();
    assert!(joined.is_ok(), "certificate thread {name:?} panicked");
}

#[cfg(not(debug_assertions))]
fn assert_scan_fixture_physical_height(expr: &Expr) {
    let mut work = vec![(expr, 1_usize)];
    let mut max_depth = 0;
    let mut sentinel_depths = Vec::new();
    while let Some((current, depth)) = work.pop() {
        max_depth = max_depth.max(depth);
        match current {
            Expr::Call { head, args } => {
                work.push((head, depth + 1));
                work.extend(args.iter().map(|arg| (arg, depth + 1)));
            }
            Expr::Atom(Atom::Symbol(symbol)) if symbol.name() == SCAN_DEEPEST_SENTINEL => {
                sentinel_depths.push(depth);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_depth, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(
        sentinel_depths,
        vec![TARGET_PHYSICAL_HEIGHT],
        "the Scan fixture must contain one sentinel at its unique deepest value"
    );
}

#[cfg(not(debug_assertions))]
#[test]
fn association_scan_visits_unique_deepest_value_at_physical_height_4097() {
    run_on_certificate_thread("scan-native-stack-height-4097", || {
        let mut session = core_session();
        let source = format!(
            "scanInput = {SCAN_DEEPEST_SENTINEL}; \
             Do[scanInput = Association[\"k\" -> scanInput], {{i, {ASSOCIATION_WRAPPER_COUNT}}}]; \
             scanInput"
        );
        let input = session
            .eval_text(&source)
            .expect("the deep Scan fixture should be constructed")
            .value;
        assert_scan_fixture_physical_height(&input);

        session
            .eval_text("scanVisitCount = 0; scanDeepestVisited = False")
            .expect("the Scan observables should initialize");
        let outcome = session
            .eval_text(
                "Scan[Function[scanVisitCount = scanVisitCount + 1; \
                 If[SameQ[Slot[1], scanDeepestSentinel], scanDeepestVisited = True]], \
                 scanInput, {0, Infinity}]",
            )
            .expect("the Scan certificate should evaluate");

        assert_eq!(outcome.value, system_expr("Null"));
        assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
        assert_eq!(
            session
                .eval_text("scanVisitCount")
                .expect("the Scan visit count should be readable")
                .value,
            Expr::small_integer((ASSOCIATION_WRAPPER_COUNT + 1) as i64)
        );
        assert_eq!(
            session
                .eval_text("scanDeepestVisited")
                .expect("the deepest-visit flag should be readable")
                .value,
            system_expr("True")
        );
    });
}

#[cfg(not(debug_assertions))]
#[test]
fn association_scan_shallow_companion_pins_levels_position_and_visit_order() {
    run_on_certificate_thread("scan-native-stack-height-3", || {
        let mut session = core_session();
        let outcome = session
            .eval_text(
                "seen = {}; \
                 Scan[Function[seen = Append[seen, Slot[1]]], \
                 Association[\"outer\" -> Association[\"middle\" -> \
                 Association[\"inner\" -> scanDeepestSentinel]]], {0, Infinity}]; \
                 seen",
            )
            .expect("the shallow Scan companion should evaluate");

        assert_eq!(
            outcome.value.to_string(),
            "List[scanDeepestSentinel, Association[Rule[\"inner\", scanDeepestSentinel]], \
             Association[Rule[\"middle\", Association[Rule[\"inner\", scanDeepestSentinel]]]], \
             Association[Rule[\"outer\", Association[Rule[\"middle\", \
             Association[Rule[\"inner\", scanDeepestSentinel]]]]]]]"
        );
        assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
    });
}

#[test]
#[ignore = "requires rum-runtime/test-native-stack-guard-zero"]
fn scan_zero_native_stack_boundary_covers_each_edge_without_callbacks() {
    for (setup, scan) in [
        (
            "scanInput = scanAssociationSentinel; \
             Do[scanInput = Association[\"k\" -> scanInput], {i, 12}]",
            "Scan[Function[scanVisits = scanVisits + 1], scanInput, {0, Infinity}]",
        ),
        (
            "scanInput = scanHeadSentinel; Do[scanInput = scanInput[], {i, 12}]",
            "Scan[Function[scanVisits = scanVisits + 1], scanInput, {0, Infinity}, Heads -> True]",
        ),
        (
            "scanInput = scanArgumentSentinel; Do[scanInput = f[scanInput], {i, 12}]",
            "Scan[Function[scanVisits = scanVisits + 1], scanInput, {0, Infinity}]",
        ),
    ] {
        let mut session = core_session();
        session
            .eval_text(setup)
            .expect("the modest-depth Scan fixture should be constructed");
        session
            .eval_text("scanVisits = 0")
            .expect("the Scan callback counter should initialize");

        let exhausted = session
            .eval_text(scan)
            .expect("Scan guard exhaustion should be returned in band");
        assert_stack_limit_exceeded(&exhausted);
        assert_eq!(
            session
                .eval_text("scanVisits")
                .expect("the callback counter should remain readable")
                .value,
            Expr::small_integer(0),
            "an exhausted Scan collection must not run any callback"
        );
        assert_eq!(
            session
                .eval_text("2 + 5")
                .expect("the session should remain usable after Scan exhaustion")
                .value,
            Expr::small_integer(7)
        );
    }
}

#[test]
#[ignore = "requires rum-runtime/test-native-stack-guard-zero"]
fn keypath_set_zero_native_stack_boundary_preserves_both_lvalue_roots() {
    let bare_keys = std::iter::repeat_n("\"k\"", 12)
        .collect::<Vec<_>>()
        .join(", ");
    let explicit_keys = std::iter::repeat_n("Key[\"k\"]", 12)
        .collect::<Vec<_>>()
        .join(", ");

    for assignment in [
        format!("b[{bare_keys}] = 8"),
        format!("b[[{explicit_keys}]] = 9"),
    ] {
        let mut session = core_session();
        session
            .eval_text(
                "b = setDeepestSentinel; \
                 Do[b = Association[\"k\" -> b], {i, 12}]; before = b",
            )
            .expect("the modest-depth Set fixture should be constructed");

        let exhausted = session
            .eval_text(&assignment)
            .expect("Set guard exhaustion should be returned in band");
        assert_stack_limit_exceeded(&exhausted);
        assert!(
            exhausted
                .messages
                .iter()
                .all(|message| message.symbol != "Set" || message.tag != "kval"),
            "guard exhaustion must not emit a missing-key diagnostic"
        );
        assert_eq!(
            session
                .eval_text("SameQ[b, before]")
                .expect("the Set target should remain readable")
                .value,
            system_expr("True"),
            "an exhausted key-path rebuild must not change the ownvalue"
        );
        assert_eq!(
            session
                .eval_text("2 + 5")
                .expect("the session should remain usable after Set exhaustion")
                .value,
            Expr::small_integer(7)
        );
    }
}
