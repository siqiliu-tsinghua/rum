use super::*;
use rum_core::Atom;
use rum_runtime::kernel_api::association_entries_from_expr;

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
const ASSOCIATION_WRAPPER_COUNT: usize = 2_048;
const ASSOCIATION_DEPTH_SENTINEL: &str = "association-depth-sentinel-455";

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() -> String + Send + 'static) -> String {
    std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB probe thread should spawn")
        .join()
        .unwrap_or_else(|_| panic!("2 MiB probe thread {name:?} panicked"))
}

fn evaluate_certificate(body: &str) -> String {
    evaluate_certificate_at_height(TARGET_PHYSICAL_HEIGHT, body)
}

fn evaluate_certificate_at_height(physical_height: usize, body: &str) -> String {
    evaluate_certificate_value_at_height(physical_height, body).to_string()
}

fn evaluate_certificate_value_at_height(physical_height: usize, body: &str) -> Expr {
    let mut session = Session::new();
    install_core(&mut session);
    let mut source = format!("x = 1; Do[x = {{x}}, {{i, {}}}]; ", physical_height - 1);
    source.push_str(body);
    session
        .eval_text(&source)
        .expect("depth certificate should evaluate")
        .value
}

fn evaluate_string_certificate(body: &str) -> String {
    let value = evaluate_certificate_value_at_height(TARGET_PHYSICAL_HEIGHT, body);
    let Expr::Atom(Atom::String(value)) = value else {
        panic!("string depth certificate should return an Atom::String")
    };
    value
}

fn nested_input_form(wrapper_count: usize, centre: &str) -> String {
    format!(
        "{}{centre}{}",
        "{".repeat(wrapper_count),
        "}".repeat(wrapper_count)
    )
}

fn nested_association_input_form(wrapper_count: usize, centre: &str) -> String {
    format!(
        "{}{centre}{}",
        "<|\"k\" -> ".repeat(wrapper_count),
        "|>".repeat(wrapper_count)
    )
}

fn mapped_nested_association_input_form(wrapper_count: usize) -> String {
    format!(
        "{}ff[\"{ASSOCIATION_DEPTH_SENTINEL}\"]{}",
        "ff[<|\"k\" -> ".repeat(wrapper_count),
        "|>]".repeat(wrapper_count)
    )
}

fn assert_raw_association_fixture_height(expr: &Expr) {
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
            Expr::Atom(Atom::String(value)) if value == ASSOCIATION_DEPTH_SENTINEL => {
                sentinel_depths.push(depth);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_depth, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(
        sentinel_depths,
        vec![TARGET_PHYSICAL_HEIGHT],
        "the fixture must contain one sentinel at its unique deepest value"
    );
}

fn assert_raw_height_with_unique_deepest_integer(expr: &Expr, expected_integer: i64) {
    let mut work = vec![(expr, 1_usize)];
    let mut max_depth = 0;
    let mut integer_depths = Vec::new();
    while let Some((current, depth)) = work.pop() {
        max_depth = max_depth.max(depth);
        match current {
            Expr::Call { head, args } => {
                work.push((head, depth + 1));
                work.extend(args.iter().map(|arg| (arg, depth + 1)));
            }
            Expr::Atom(Atom::Integer(value)) => {
                integer_depths.push((value.as_i64(), depth));
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_depth, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(
        integer_depths,
        vec![(Some(expected_integer), TARGET_PHYSICAL_HEIGHT)],
        "the input must contain one integer at its unique deepest value"
    );
}

fn assert_exact_merge_nested_chain(expr: &Expr) {
    let root_entries = association_entries_from_expr(expr)
        .expect("the complete MergeNested result should be an association");
    assert_eq!(root_entries.len(), 3);
    assert_eq!(root_entries[0].0, Expr::string("leftSentinel"));
    assert_eq!(root_entries[0].1.to_string(), "mergeLeftSentinel");
    assert_eq!(root_entries[1].0, Expr::string("k"));
    assert_eq!(root_entries[2].0, Expr::string("rightSentinel"));
    assert_eq!(root_entries[2].1.to_string(), "mergeRightSentinel");

    let mut current = root_entries[1].1.clone();
    for level in 1..ASSOCIATION_WRAPPER_COUNT {
        let entries = association_entries_from_expr(&current)
            .unwrap_or_else(|| panic!("merged level {level} should be an association"));
        assert_eq!(entries.len(), 1, "merged level {level} should have one key");
        let (key, value) = entries
            .into_iter()
            .next()
            .expect("a one-entry association should contain its entry");
        assert_eq!(key, Expr::string("k"));
        current = value;
    }
    assert_eq!(current, Expr::small_integer(42));
}

fn construct_association_depth_fixture(session: &mut Session, target: &str) -> Expr {
    let source = format!(
        "{target} = \"{ASSOCIATION_DEPTH_SENTINEL}\"; \
         Do[{target} = Association[\"k\" -> {target}], {{i, {ASSOCIATION_WRAPPER_COUNT}}}]; \
         {target}"
    );
    let value = session
        .eval_text(&source)
        .expect("deep association fixture should be constructed")
        .value;
    assert_raw_association_fixture_height(&value);
    value
}

fn evaluate_shallow_input_form(body: &str) -> String {
    let mut session = Session::new();
    install_core(&mut session);
    let source = format!("ToString[{body}, InputForm]");
    let value = session
        .eval_text(&source)
        .expect("shallow selector companion should evaluate")
        .value;
    let Expr::Atom(Atom::String(value)) = value else {
        panic!("shallow selector companion should return an Atom::String")
    };
    value
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn matchq_completes_at_physical_height_20001_on_two_mib() {
    let certificate = run_on_two_mib_thread("matchq-depth-20001", || {
        evaluate_certificate_at_height(20_001, "MatchQ[x, _?IntegerQ]")
    });
    assert_eq!(certificate, "False");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn construction_reaches_physical_height_20001_on_two_mib() {
    let certificate = run_on_two_mib_thread("construction-depth-20001", || {
        evaluate_certificate_at_height(20_001, "Depth[x]")
    });
    assert_eq!(certificate, "20001");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn tostring_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("tostring-depth-4097", || {
        evaluate_certificate("StringLength[ToString[x]]")
    });
    assert_eq!(certificate, "8193");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn take_multispec_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("take-multispec-depth-4097", || {
        let spec_count = TARGET_PHYSICAL_HEIGHT - 1;
        let body =
            format!("ToString[Take @@ Prepend[Table[{{1}}, {{{spec_count}}}], x], InputForm]");
        evaluate_string_certificate(&body)
    });
    assert_eq!(
        certificate,
        nested_input_form(TARGET_PHYSICAL_HEIGHT - 1, "1")
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn replacepart_deep_path_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("replacepart-deep-path-depth-4097", || {
        let path_length = TARGET_PHYSICAL_HEIGHT - 1;
        let body = format!("ToString[ReplacePart[x, Table[1, {{{path_length}}}] -> 9], InputForm]");
        evaluate_string_certificate(&body)
    });
    assert_eq!(
        certificate,
        nested_input_form(TARGET_PHYSICAL_HEIGHT - 1, "9")
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn map_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("map-depth-4097", || {
        evaluate_string_certificate("ToString[Map[deepMapProbe, x, {-1}], InputForm]")
    });
    assert_eq!(
        certificate,
        nested_input_form(TARGET_PHYSICAL_HEIGHT - 1, "deepMapProbe[1]")
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn association_map_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("association-map-depth-4097", || {
        let mut session = Session::new();
        install_core(&mut session);
        let input = construct_association_depth_fixture(&mut session, "associationMapInput");
        let value = session
            .eval_text("ToString[Map[ff, associationMapInput, {0, Infinity}], InputForm]")
            .expect("deep association Map should complete")
            .value;
        drop(input);
        let Expr::Atom(Atom::String(value)) = value else {
            panic!("deep association Map should return an Atom::String")
        };
        value
    });
    assert_eq!(
        certificate,
        mapped_nested_association_input_form(ASSOCIATION_WRAPPER_COUNT)
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn association_assignment_read_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("association-assignment-read-depth-4097", || {
        let mut session = Session::new();
        install_core(&mut session);
        let input = construct_association_depth_fixture(&mut session, "associationAssignmentInput");
        let selectors = std::iter::repeat_n("\"k\"", ASSOCIATION_WRAPPER_COUNT)
            .collect::<Vec<_>>()
            .join(", ");
        let source =
            format!("b = associationAssignmentInput; b[[{selectors}]] = 8; ToString[b, InputForm]");
        let value = session
            .eval_text(&source)
            .expect("deep association assignment and read should complete")
            .value;
        drop(input);
        let Expr::Atom(Atom::String(value)) = value else {
            panic!("deep association assignment read should return an Atom::String")
        };
        value
    });
    assert_eq!(
        certificate,
        nested_association_input_form(ASSOCIATION_WRAPPER_COUNT, "8")
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn merge_nested_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("merge-nested-depth-4097", || {
        let mut session = Session::new();
        install_core(&mut session);
        let source = format!(
            "mergeLeftInput = 17; mergeRightInput = 25; \
             Do[mergeLeftInput = Association[\"k\" -> mergeLeftInput]; \
                mergeRightInput = Association[\"k\" -> mergeRightInput], \
                {{i, {}}}]; \
             mergeLeftInput = Association[\"leftSentinel\" -> mergeLeftSentinel, \
                                          \"k\" -> mergeLeftInput]; \
             mergeRightInput = Association[\"k\" -> mergeRightInput, \
                                           \"rightSentinel\" -> mergeRightSentinel]; \
             Null",
            ASSOCIATION_WRAPPER_COUNT - 1
        );
        session
            .eval_text(&source)
            .expect("deep MergeNested inputs should be constructed");

        let left = session
            .eval_text("mergeLeftInput")
            .expect("left deep MergeNested input should be readable")
            .value;
        let right = session
            .eval_text("mergeRightInput")
            .expect("right deep MergeNested input should be readable")
            .value;
        assert_raw_height_with_unique_deepest_integer(&left, 17);
        assert_raw_height_with_unique_deepest_integer(&right, 25);

        let outcome = session
            .eval_text("MergeNested[{mergeLeftInput, mergeRightInput}, Total]")
            .expect("deep MergeNested should complete");
        assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
        assert_exact_merge_nested_chain(&outcome.value);
        "complete".to_string()
    });
    assert_eq!(certificate, "complete");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn apply_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("apply-depth-4097", || {
        evaluate_string_certificate("ToString[Apply[deepApplyProbe, x, {-2}], InputForm]")
    });
    assert_eq!(
        certificate,
        nested_input_form(TARGET_PHYSICAL_HEIGHT - 2, "deepApplyProbe[1]")
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn replaceall_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("replaceall-depth-4097", || {
        evaluate_string_certificate("ToString[ReplaceAll[x, 1 -> 2], InputForm]")
    });
    assert_eq!(
        certificate,
        nested_input_form(TARGET_PHYSICAL_HEIGHT - 1, "2")
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn replace_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("replace-depth-4097", || {
        evaluate_string_certificate("ToString[Replace[x, 1 -> 2, {0, Infinity}], InputForm]")
    });
    assert_eq!(
        certificate,
        nested_input_form(TARGET_PHYSICAL_HEIGHT - 1, "2")
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn with_substitution_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("with-substitution-depth-4097", || {
        let mut session = Session::new();
        install_core(&mut session);
        let source = format!(
            "deepBody = q; Do[deepBody = {{deepBody}}, {{i, {}}}]; \
             ToString[With @@ {{{{q -> 7}}, deepBody}}, InputForm]",
            TARGET_PHYSICAL_HEIGHT - 1
        );
        let value = session
            .eval_text(&source)
            .expect("deep With substitution should complete")
            .value;
        let Expr::Atom(Atom::String(value)) = value else {
            panic!("deep With substitution should return an Atom::String")
        };
        value
    });
    assert_eq!(
        certificate,
        nested_input_form(TARGET_PHYSICAL_HEIGHT - 1, "7")
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn rule_rhs_substitution_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("rule-rhs-substitution-depth-4097", || {
        let mut session = Session::new();
        install_core(&mut session);
        let source = format!(
            "deepTemplate = q; Do[deepTemplate = {{deepTemplate}}, {{i, {}}}]; \
             ToString[Replace[7, RuleDelayed @@ {{Pattern[q, Blank[]], deepTemplate}}], \
             InputForm]",
            TARGET_PHYSICAL_HEIGHT - 1
        );
        let value = session
            .eval_text(&source)
            .expect("deep rule RHS substitution should complete")
            .value;
        let Expr::Atom(Atom::String(value)) = value else {
            panic!("deep rule RHS substitution should return an Atom::String")
        };
        value
    });
    assert_eq!(
        certificate,
        nested_input_form(TARGET_PHYSICAL_HEIGHT - 1, "7")
    );
}

#[test]
#[ignore = "release-only active-spine safety probe; run explicitly with --release"]
fn replace_active_spine_returns_in_band_resource_error_on_two_mib() {
    let result = run_on_two_mib_thread("replace-active-spine", || {
        let mut session = Session::new();
        install_core(&mut session);
        let source = format!(
            "x = 1; Do[x = {{x}}, {{i, {}}}]; Replace[x, List -> Identity, {{0, Infinity}}, Heads -> True]",
            TARGET_PHYSICAL_HEIGHT - 1
        );
        let outcome = session
            .eval_text(&source)
            .expect("active Replace spine should fail in band");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "General" && message.tag == "reslim")
        );
        assert_eq!(
            session
                .eval_text("1 + 1")
                .expect("session should remain usable after the resource error")
                .value
                .to_string(),
            "2"
        );
        outcome.value.to_string()
    });
    assert_eq!(result, "TerminatedEvaluation[\"StackLimit\"]");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn deletecases_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("deletecases-depth-4097", || {
        evaluate_string_certificate("ToString[Apply[DeleteCases, {x, 1, Infinity}], InputForm]")
    });
    assert_eq!(
        certificate,
        nested_input_form(TARGET_PHYSICAL_HEIGHT - 1, "")
    );
}

#[test]
#[ignore = "release-only active-spine safety probe; run explicitly with --release"]
fn deletecases_active_spine_returns_in_band_resource_error_on_two_mib() {
    let result = run_on_two_mib_thread("deletecases-active-spine", || {
        let mut session = Session::new();
        install_core(&mut session);
        let source = format!(
            "x = 1; Do[x = f[x], {{i, {}}}]; held = Apply[HoldComplete, {{x}}]; f[y_] := y; Apply[DeleteCases, {{held, HoldComplete, All, Heads -> True}}]",
            TARGET_PHYSICAL_HEIGHT - 1
        );
        let outcome = session
            .eval_text(&source)
            .expect("active DeleteCases spine should fail in band");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "General" && message.tag == "reslim")
        );
        assert_eq!(
            session
                .eval_text("1 + 1")
                .expect("session should remain usable after the resource error")
                .value
                .to_string(),
            "2"
        );
        outcome.value.to_string()
    });
    assert_eq!(result, "TerminatedEvaluation[\"StackLimit\"]");
}

#[test]
#[ignore = "release-only active-spine safety probe; run explicitly with --release"]
fn d_nested_sin_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("d-nested-sin-depth-4097", || {
        let physical_height = TARGET_PHYSICAL_HEIGHT;
        let mut session = Session::new();
        install_core(&mut session);
        let source = format!(
            "e = q; Do[e = Sin[e], {{i, {}}}]; Depth[e]",
            physical_height - 1
        );
        assert_eq!(
            session
                .eval_text(&source)
                .expect("nested-Sin spine should be constructed")
                .value
                .to_string(),
            physical_height.to_string()
        );

        let outcome = session
            .eval_text("Length[D[e, q]]")
            .expect("nested-Sin derivative should complete");
        assert_eq!(outcome.value.to_string(), (physical_height - 1).to_string());
        assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
        assert_eq!(
            session
                .eval_text("1 + 1")
                .expect("session should remain usable after D")
                .value
                .to_string(),
            "2"
        );
        outcome.value.to_string()
    });
    assert_eq!(certificate, (TARGET_PHYSICAL_HEIGHT - 1).to_string());
}

#[test]
fn map_certificate_companion_pins_descending_selector() {
    assert_eq!(
        evaluate_shallow_input_form("Map[deepMapProbe, {{{1}}}, {-1}]"),
        "{{{deepMapProbe[1]}}}"
    );
}

#[test]
fn association_map_certificate_companion_pins_descending_selector() {
    assert_eq!(
        evaluate_shallow_input_form(
            "Map[ff, Association[\"k\" -> Association[\"k\" -> Association[\"k\" -> deepestAssociationSentinel]]], {0, Infinity}]",
        ),
        "ff[<|\"k\" -> ff[<|\"k\" -> ff[<|\"k\" -> ff[deepestAssociationSentinel]|>]|>]|>]"
    );
}

#[test]
fn association_assignment_certificate_companion_pins_full_key_selector_chain() {
    assert_eq!(
        evaluate_shallow_input_form(
            "b = Association[\"k\" -> Association[\"k\" -> Association[\"k\" -> deepestAssociationSentinel]]]; b[[\"k\", \"k\", \"k\"]] = 8; b",
        ),
        "<|\"k\" -> <|\"k\" -> <|\"k\" -> 8|>|>|>"
    );
}

#[test]
fn apply_certificate_companion_pins_descending_selector() {
    assert_eq!(
        evaluate_shallow_input_form("Apply[deepApplyProbe, {{{1}}}, {-2}]"),
        "{{deepApplyProbe[1]}}"
    );
}

#[test]
fn replaceall_certificate_companion_pins_matching_rule() {
    assert_eq!(
        evaluate_shallow_input_form("ReplaceAll[{{{1}}}, 1 -> 2]"),
        "{{{2}}}"
    );
}

#[test]
fn replace_certificate_companion_pins_descending_selector() {
    assert_eq!(
        evaluate_shallow_input_form("Replace[{{{1}}}, 1 -> 2, {0, Infinity}]"),
        "{{{2}}}"
    );
}

#[test]
fn deletecases_certificate_companion_pins_descending_selector() {
    assert_eq!(
        evaluate_shallow_input_form("DeleteCases[{{{1}}}, 1, Infinity]"),
        "{{{}}}"
    );
}

#[test]
fn deletecases_limit_pins_post_order_visit_order() {
    assert_eq!(
        evaluate_shallow_input_form("DeleteCases[{{{1}}, {2}, {3}}, _List, Infinity, 2]"),
        "{{2}, {3}}"
    );
}

#[test]
fn lexical_substitution_certificate_companion_pins_three_level_body() {
    assert_eq!(
        evaluate_shallow_input_form("With[{q = 7}, {{{q}}}]"),
        "{{{7}}}"
    );
}

#[test]
fn rule_rhs_substitution_certificate_companion_pins_three_level_body() {
    assert_eq!(
        evaluate_shallow_input_form("Replace[7, RuleDelayed @@ {Pattern[q, Blank[]], {{{q}}}}]"),
        "{{{7}}}"
    );
}

#[test]
fn lexical_substitution_keeps_nested_with_shadow_local_to_its_subtree() {
    assert_eq!(
        evaluate_shallow_input_form("With[{q = 7}, {{{With[{q = 11}, Hold[q]]}, q}}]"),
        "{{{Hold[11]}, 7}}"
    );
}

#[test]
fn lexical_substitution_iterative_tail_keeps_shadow_local_to_nested_with() {
    let body = nested_input_form(20, "{With[{q = 11}, Hold[q]], q}");
    let expected = nested_input_form(20, "{Hold[11], 7}");
    assert_eq!(
        evaluate_shallow_input_form(&format!("With[{{q = 7}}, {body}]")),
        expected
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn cases_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("cases-depth-4097", || {
        evaluate_certificate("Length[Cases[x, z, Infinity]]")
    });
    assert_eq!(certificate, "0");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn level_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("level-depth-4097", || {
        evaluate_certificate("Length[Level[x, Infinity]]")
    });
    assert_eq!(certificate, "4096");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn memberq_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("memberq-depth-4097", || {
        evaluate_certificate("MemberQ[x, z, Infinity]")
    });
    assert_eq!(certificate, "False");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn count_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("count-depth-4097", || {
        evaluate_certificate("Count[x, z, Infinity]")
    });
    assert_eq!(certificate, "0");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn position_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("position-depth-4097", || {
        evaluate_certificate("Length[Position[x, z, Infinity]]")
    });
    assert_eq!(certificate, "0");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn freeq_completes_at_physical_height_4097_on_two_mib() {
    let certificate =
        run_on_two_mib_thread("freeq-depth-4097", || evaluate_certificate("FreeQ[x, z]"));
    assert_eq!(certificate, "True");
}
