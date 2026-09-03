use super::*;

#[test]
fn trace_returns_a_first_pass_structured_expression_list_without_leaking_session_trace_state() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();

    let traced = session.eval_text("Trace[f[1]]").unwrap();
    assert_eq!(
        traced.value.to_string(),
        "List[Hold[f[1]], List[Hold[Plus[1, 1]], Hold[2]], Hold[2]]"
    );
    assert!(session.take_trace().is_empty());

    let filtered = session.eval_text("Trace[f[1], _Plus]").unwrap();
    assert_eq!(filtered.value.to_string(), "List[List[Hold[Plus[1, 1]]]]");

    let multi = session.eval_text("Trace[f[1], {_Plus, _Integer}]").unwrap();
    assert_eq!(
        multi.value.to_string(),
        "List[List[Hold[Plus[1, 1]], Hold[2]], Hold[2]]"
    );

    let leveled = session.eval_text("Trace[f[1], _Integer, {-1}]").unwrap();
    assert_eq!(
        leveled.value.to_string(),
        "List[Hold[1], List[Hold[1], Hold[2]], Hold[2]]"
    );

    let top_level_only = session.eval_text("Trace[f[1], _Integer, 0]").unwrap();
    assert_eq!(
        top_level_only.value.to_string(),
        "List[List[Hold[2]], Hold[2]]"
    );

    let with_heads = session
        .eval_text("Trace[f[1], _Symbol, Heads -> True]")
        .unwrap();
    assert_eq!(
        with_heads.value.to_string(),
        "List[Hold[f], List[Hold[Plus]]]"
    );

    session.eval_text("Unprotect[Trace]").unwrap();
    session
        .eval_text("SetOptions[Trace, Heads -> True]")
        .unwrap();
    let default_heads = session.eval_text("Trace[f[1], _Symbol]").unwrap();
    assert_eq!(
        default_heads.value.to_string(),
        "List[Hold[f], List[Hold[Plus]]]"
    );

    let explicit_no_heads = session
        .eval_text("Trace[f[1], _Symbol, Heads -> False]")
        .unwrap();
    assert_eq!(explicit_no_heads.value.to_string(), "List[]");

    let explicit_no_heads_with_levels = session
        .eval_text("Trace[f[1], _Symbol, {0, Infinity}, {Heads -> False}]")
        .unwrap();
    assert_eq!(explicit_no_heads_with_levels.value.to_string(), "List[]");
}

#[test]
fn trace_uses_recorded_depths_to_keep_nested_branches_stable() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();
    session.eval_text("g[x_] := f[x] + 10").unwrap();

    let traced = session.eval_text("Trace[g[1]]").unwrap();
    assert_eq!(
        traced.value.to_string(),
        "List[Hold[g[1]], List[Hold[Plus[f[1], 10]], List[Hold[f[1]], List[Hold[Plus[1, 1]], Hold[2]], Hold[2]], Hold[12]], Hold[12]]"
    );

    let filtered = session.eval_text("Trace[g[1], _Plus]").unwrap();
    assert_eq!(
        filtered.value.to_string(),
        "List[List[Hold[Plus[f[1], 10]], List[Hold[Plus[1, 1]]]]]"
    );

    let via_eval_data = session
        .eval_text("EvaluationData[g[1], \"Trace\"]")
        .unwrap();
    assert_eq!(via_eval_data.value.to_string(), traced.value.to_string());
}

#[test]
fn trace_detail_returns_the_full_evaluation_pass_association_without_leaking_trace_state() {
    let mut session = Session::new();

    let traced = session.eval_text("TraceDetail[1 + 1]").unwrap();
    assert_eq!(
        traced.value.to_string(),
        "List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 1]]], Rule[\"After\", Hold[2]]]]"
    );
    assert!(session.take_trace().is_empty());
}

#[test]
fn trace_detail_identifies_the_downvalue_that_won_among_overlapping_definitions() {
    let mut session = Session::new();
    session.eval_text("f[x_] := general").unwrap();
    session.eval_text("f[x_Integer] := integer").unwrap();
    session.eval_text("f[2] := exact").unwrap();

    let traced = session.eval_text("TraceDetail[f[2]]").unwrap();
    assert_eq!(
        traced.value.to_string(),
        "List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[f[2]]], Rule[\"RuleLeft\", Hold[f[2]]], Rule[\"Result\", Hold[exact]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[2]]], Rule[\"After\", Hold[exact]]]]"
    );
}

#[test]
fn trace_detail_preserves_definition_symbol_kind_and_value() {
    let mut session = Session::new();

    let traced = session.eval_text("TraceDetail[g = 5]").unwrap();
    assert_eq!(
        traced.value.to_string(),
        "List[Association[Rule[\"Step\", \"DefinitionStored\"], Rule[\"Symbol\", \"Global`g\"], Rule[\"Kind\", \"OwnValue\"], Rule[\"Value\", Hold[5]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Set[g, 5]]], Rule[\"After\", Hold[5]]]]"
    );
}

#[test]
fn trace_detail_form_filter_matches_the_underlying_expression_not_the_association() {
    let mut session = Session::new();

    let filtered = session
        .eval_text("TraceDetail[(1 + 2) * 3, _Plus]")
        .unwrap();
    assert_eq!(
        filtered.value.to_string(),
        "List[List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]]]]"
    );

    let result_side = session.eval_text("TraceDetail[1 + 1, _Integer]").unwrap();
    assert_eq!(
        result_side.value.to_string(),
        "List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 1]]], Rule[\"After\", Hold[2]]]]"
    );

    let annotation_head = session
        .eval_text("TraceDetail[(1 + 2) * 3, _Association]")
        .unwrap();
    assert_eq!(annotation_head.value.to_string(), "List[]");
}

#[test]
fn trace_detail_supports_levelspec_and_heads_options() {
    let mut session = Session::new();

    let top_level_only = session
        .eval_text("TraceDetail[(1 + 2) * 3, _Plus, 0]")
        .unwrap();
    assert_eq!(
        top_level_only.value.to_string(),
        "List[List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]]]]"
    );

    let first_level_only = session
        .eval_text("TraceDetail[(1 + 2) * 3, _Plus, {1}]")
        .unwrap();
    assert_eq!(
        first_level_only.value.to_string(),
        "List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Times[Plus[1, 2], 3]]], Rule[\"After\", Hold[9]]]]"
    );

    let leaf_levels = session
        .eval_text("TraceDetail[(1 + 2) * 3, _Integer, {-1}, {Heads -> False}]")
        .unwrap();
    assert_eq!(
        leaf_levels.value.to_string(),
        "List[List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Times[Plus[1, 2], 3]]], Rule[\"After\", Hold[9]]]]"
    );

    let without_heads = session
        .eval_text("TraceDetail[1 + 1, _Symbol, Heads -> False]")
        .unwrap();
    assert_eq!(without_heads.value.to_string(), "List[]");

    let with_heads = session
        .eval_text("TraceDetail[1 + 1, _Symbol, Heads -> True]")
        .unwrap();
    assert_eq!(
        with_heads.value.to_string(),
        "List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 1]]], Rule[\"After\", Hold[2]]]]"
    );
}

#[test]
fn trace_detail_has_the_same_list_nesting_shape_as_trace() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();
    session.eval_text("g[x_] := f[x] + 10").unwrap();

    let trace = session.eval_text("Trace[g[1]]").unwrap().value;
    let detail = session.eval_text("TraceDetail[g[1]]").unwrap().value;
    let trace_shape = trace_list_nesting_shape(&trace);
    assert_eq!(trace_shape, "[[[[]]]]");
    assert_eq!(trace_list_nesting_shape(&detail), trace_shape);
}

#[test]
fn trace_print_emits_trace_lines_and_returns_the_underlying_value() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();

    let traced = session.eval_text("TracePrint[f[1]]").unwrap();
    assert_eq!(traced.value.to_string(), "2");
    assert_eq!(traced.prints, vec!["f[1]", "  Plus[1, 1]", "  2", "2"]);
    assert!(session.take_trace().is_empty());

    let filtered = session.eval_text("TracePrint[f[1], _Plus]").unwrap();
    assert_eq!(filtered.value.to_string(), "2");
    assert_eq!(filtered.prints, vec!["Plus[1, 1]"]);

    let multi = session
        .eval_text("TracePrint[f[1], {_Plus, _Integer}]")
        .unwrap();
    assert_eq!(multi.value.to_string(), "2");
    assert_eq!(multi.prints, vec!["  Plus[1, 1]", "  2", "2"]);

    let leveled = session
        .eval_text("TracePrint[f[1], _Integer, {-1}]")
        .unwrap();
    assert_eq!(leveled.value.to_string(), "2");
    assert_eq!(leveled.prints, vec!["1", "  1", "  2", "2"]);

    let with_heads = session
        .eval_text("TracePrint[f[1], _Symbol, Heads -> True]")
        .unwrap();
    assert_eq!(with_heads.value.to_string(), "2");
    assert_eq!(with_heads.prints, vec!["f", "  Plus"]);

    session.eval_text("Unprotect[TracePrint]").unwrap();
    session
        .eval_text("SetOptions[TracePrint, Heads -> True]")
        .unwrap();
    let default_heads = session.eval_text("TracePrint[f[1], _Symbol]").unwrap();
    assert_eq!(default_heads.value.to_string(), "2");
    assert_eq!(default_heads.prints, vec!["f", "  Plus"]);

    let explicit_no_heads = session
        .eval_text("TracePrint[f[1], _Symbol, Heads -> False]")
        .unwrap();
    assert_eq!(explicit_no_heads.value.to_string(), "2");
    assert!(explicit_no_heads.prints.is_empty());

    let explicit_no_heads_with_levels = session
        .eval_text("TracePrint[f[1], _Symbol, {0, Infinity}, {Heads -> False}]")
        .unwrap();
    assert_eq!(explicit_no_heads_with_levels.value.to_string(), "2");
    assert!(explicit_no_heads_with_levels.prints.is_empty());
}

#[test]
fn trace_print_uses_the_same_nested_structure_for_deeper_trace_branches() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x + 1").unwrap();
    session.eval_text("g[x_] := f[x] + 10").unwrap();

    let traced = session.eval_text("TracePrint[g[1], _Plus]").unwrap();
    assert_eq!(traced.value.to_string(), "12");
    assert_eq!(traced.prints, vec!["Plus[f[1], 10]", "  Plus[1, 1]"]);
}

#[test]
fn echo_family_prints_and_returns_values_in_the_current_first_pass_subset() {
    let mut session = session_with_test_control_builtins();

    let echoed = session.eval_text("Echo[1 + 1]").unwrap();
    assert_eq!(echoed.value.to_string(), "2");
    assert_eq!(echoed.prints, vec!["2"]);

    let labeled = session.eval_text("Echo[1 + 1, \"sum\"]").unwrap();
    assert_eq!(labeled.value.to_string(), "2");
    assert_eq!(labeled.prints, vec!["sum: 2"]);

    let via_function = session.eval_text("EchoFunction[][1 + 1]").unwrap();
    assert_eq!(via_function.value.to_string(), "2");
    assert_eq!(via_function.prints, vec!["2"]);

    let via_applied_function = session
        .eval_text("EchoFunction[Function[Plus[Slot[1], 1]]][1 + 1]")
        .unwrap();
    assert_eq!(via_applied_function.value.to_string(), "2");
    assert_eq!(via_applied_function.prints, vec!["3"]);

    let via_symbolic_function = session.eval_text("EchoFunction[f][1 + 2]").unwrap();
    assert_eq!(via_symbolic_function.value.to_string(), "3");
    assert_eq!(via_symbolic_function.prints, vec!["f[3]"]);

    let via_label = session.eval_text("EchoLabel[\"sum\"][1 + 1]").unwrap();
    assert_eq!(via_label.value.to_string(), "2");
    assert_eq!(via_label.prints, vec!["sum: 2"]);

    let via_symbolic_label = session.eval_text("EchoLabel[f][1 + 2]").unwrap();
    assert_eq!(via_symbolic_label.value.to_string(), "3");
    assert_eq!(via_symbolic_label.prints, vec!["f: 3"]);
    assert_ne!(via_symbolic_function.prints, via_symbolic_label.prints);
}

#[test]
fn short_uses_the_current_canonical_text_truncation_subset() {
    let mut session = Session::new();

    let shortened = session.eval_text("Short[\"abcdef\", 16]").unwrap();
    assert_eq!(shortened.value.to_string(), "\"abcdef\"");

    let long_form = session.eval_text("Short[Table[i, {i, 20}], 18]").unwrap();
    assert!(long_form.value.to_string().contains("..."));
}

#[test]
fn rum_profile_hooks_are_inert_without_profile_env() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("RumProfileReset[]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("RumProfileSnapshot[\"unit\"]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session
            .eval_text("RumProfileReset[extra]")
            .unwrap()
            .value
            .to_string(),
        "RumProfileReset[extra]"
    );
}

#[test]
fn on_and_off_gate_matching_messages_at_session_scope() {
    let mut session = Session::new();

    session.eval_text("Off[FindFile::notfound]").unwrap();
    let suppressed = session.eval_text("Message[FindFile::notfound]").unwrap();
    assert!(suppressed.messages.is_empty());

    session.eval_text("On[FindFile]").unwrap();
    let restored = session.eval_text("Message[FindFile::notfound]").unwrap();
    assert!(
        restored
            .messages
            .iter()
            .any(|message| { message.symbol == "FindFile" && message.tag == "notfound" })
    );
}

fn trace_list_nesting_shape(expr: &Expr) -> String {
    let Expr::Call { head, args } = expr else {
        return String::new();
    };
    if !is_system_symbol_name(head, "List") {
        return String::new();
    }

    let mut shape = String::from("[");
    for item in args {
        if matches!(item, Expr::Call { head, .. } if is_system_symbol_name(head, "List")) {
            shape.push_str(&trace_list_nesting_shape(item));
        }
    }
    shape.push(']');
    shape
}
