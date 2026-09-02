use crate::install_core;
use rum_runtime::{Capability, EvalError, Session};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_text(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn lazy_range_builds_canonical_infinite_descriptors() {
    let mut session = core_session();

    for (source, expected) in [
        ("LazyRange[]", "LazyList[LazyRange[1, 1]]"),
        ("LazyRange[4]", "LazyList[LazyRange[4, 1]]"),
        ("LazyRange[4, 3/2]", "LazyList[LazyRange[4, 3/2]]"),
        ("x = 2; LazyRange[x + 1, x]", "LazyList[LazyRange[3, 2]]"),
    ] {
        assert_eq!(eval_text(&mut session, source), expected, "source {source}");
    }

    assert_eq!(
        eval_text(&mut session, "Attributes[LazyList]"),
        "List[HoldAllComplete, Protected]"
    );
    assert_eq!(
        eval_text(&mut session, "Attributes[LazyRange]"),
        "List[Protected]"
    );
    assert!(
        crate::catalog::LAZY_BUILTINS
            .iter()
            .all(|spec| spec.capability == Capability::Pure)
    );
}

#[test]
fn lazy_range_rejects_invalid_arguments_without_partial_values() {
    let mut session = core_session();

    let invalid_value = session.eval_text("LazyRange[x]").unwrap();
    assert_eq!(invalid_value.value.to_string(), "LazyRange[x]");
    assert!(
        invalid_value
            .messages
            .iter()
            .any(|message| message.symbol == "LazyRange" && message.tag == "argform")
    );

    let invalid_arity = session.eval_text("LazyRange[1, 2, 3]").unwrap();
    assert_eq!(invalid_arity.value.to_string(), "LazyRange[1, 2, 3]");
    assert!(
        invalid_arity
            .messages
            .iter()
            .any(|message| message.symbol == "LazyRange" && message.tag == "argtu")
    );
}

#[test]
fn lazy_list_is_opaque_structural_and_inert() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyRange[1, 1], _List]"),
        "False"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyRange[1, 1], _LazyList]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "Total[LazyRange[1, 1]]"),
        "Total[LazyList[LazyRange[1, 1]]]"
    );
    assert_eq!(
        eval_text(&mut session, "LazyRange[1, 1]"),
        "LazyList[LazyRange[1, 1]]"
    );
    assert_eq!(
        eval_text(&mut session, "SameQ[LazyRange[1, 1], LazyRange[1, 1]]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "SameQ[LazyRange[1, 1], LazyRange[2, 1]]"),
        "False"
    );
    assert_eq!(
        eval_text(&mut session, "held = 0; LazyList[held = 1]; held"),
        "0"
    );
}

#[test]
fn lazy_pull_cancellation_does_not_stale_the_same_value_for_a_fresh_request() {
    let mut session = core_session();
    session
        .eval_request(
            "savedLazy405 = LazyTake[LazyMap[Function[x, Print[\"lazy-pull-started\"]; x], LazyRange[1, 1]], 3]",
        )
        .expect("the reusable lazy value should be installed");

    let cancel = session.request_cancel_handle();
    session.set_print_sink(Box::new(move |line| {
        assert_eq!(line, "lazy-pull-started");
        cancel.cancel();
    }));
    let cancelled = session
        .eval_request("Normal[savedLazy405]")
        .expect("lazy cancellation should surface as a terminated evaluation");
    assert_eq!(
        cancelled.termination,
        Some(rum_runtime::EvalTerminationReason::Cancelled)
    );

    session.clear_print_sink();
    let fresh = session
        .eval_request("Normal[savedLazy405]")
        .expect("the same lazy value should pull in a fresh request");
    assert_eq!(fresh.termination, None);
    assert_eq!(fresh.value.to_string(), "List[1, 2, 3]");
}

#[test]
fn bounded_realizers_pull_lazy_ranges() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "Take[LazyRange[10, 5], 3]"),
        "List[10, 15, 20]"
    );
    assert_eq!(
        eval_text(&mut session, "Take[LazyRange[10, 5], UpTo[3]]"),
        "List[10, 15, 20]"
    );
    assert_eq!(
        eval_text(&mut session, "Take[LazyRange[1/2, 1/3], 3]"),
        "List[1/2, 5/6, 7/6]"
    );
    assert_eq!(
        eval_text(&mut session, "Take[LazyRange[10, 5], 0]"),
        "List[]"
    );
    assert_eq!(eval_text(&mut session, "First[LazyRange[7, 2]]"), "7");
}

#[test]
fn normal_on_infinite_lazy_range_fails_closed_at_iteration_ceiling() {
    let mut session = core_session();
    session.set_iteration_work_limit(3);

    let outcome = session.eval_text("Normal[LazyRange[1, 1]]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"IterationWorkLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );
    assert_eq!(eval_text(&mut session, "1 + 1"), "2");
}

#[test]
fn lazy_transformers_build_canonical_opaque_descriptors() {
    let mut session = core_session();

    for (source, expected) in [
        (
            "LazyMap[f, LazyRange[1, 1]]",
            "LazyList[LazyMap[f, LazyRange[1, 1]]]",
        ),
        (
            "LazySelect[p, LazyRange[2, 3]]",
            "LazyList[LazySelect[p, LazyRange[2, 3]]]",
        ),
        (
            "LazyTake[LazyRange[1, 1], 4]",
            "LazyList[LazyTake[LazyRange[1, 1], 4]]",
        ),
        (
            "LazyTake[LazyRange[1, 1], UpTo[4]]",
            "LazyList[LazyTake[LazyRange[1, 1], 4]]",
        ),
        (
            "LazyDrop[LazyRange[1, 1], 4]",
            "LazyList[LazyDrop[LazyRange[1, 1], 4]]",
        ),
    ] {
        assert_eq!(eval_text(&mut session, source), expected, "source {source}");
    }

    for symbol in ["LazyDrop", "LazyMap", "LazySelect", "LazyTake"] {
        assert_eq!(
            eval_text(&mut session, &format!("Attributes[{symbol}]")),
            "List[Protected]"
        );
    }
    assert_eq!(
        eval_text(
            &mut session,
            "MatchQ[LazyTake[LazyRange[1, 1], 3], _LazyList]"
        ),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyTake[LazyRange[1, 1], 3], _List]"),
        "False"
    );
}

#[test]
fn lazy_transformers_reject_invalid_inputs_without_partial_values() {
    let mut session = core_session();

    for (source, expected, symbol, tag) in [
        (
            "LazyMap[f, {1, 2}]",
            "LazyMap[f, List[1, 2]]",
            "LazyMap",
            "argform",
        ),
        (
            "LazySelect[p, {1, 2}]",
            "LazySelect[p, List[1, 2]]",
            "LazySelect",
            "argform",
        ),
        (
            "LazyTake[LazyRange[1, 1], -1]",
            "LazyTake[LazyList[LazyRange[1, 1]], -1]",
            "LazyTake",
            "argform",
        ),
        (
            "LazyTake[LazyRange[1, 1], UpTo[-1]]",
            "LazyTake[LazyList[LazyRange[1, 1]], UpTo[-1]]",
            "LazyTake",
            "argform",
        ),
        (
            "LazyDrop[LazyRange[1, 1], UpTo[2]]",
            "LazyDrop[LazyList[LazyRange[1, 1]], UpTo[2]]",
            "LazyDrop",
            "argform",
        ),
        ("LazyMap[f]", "LazyMap[f]", "LazyMap", "argx"),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "source {source}");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == symbol && message.tag == tag),
            "source {source}"
        );
    }
}

#[test]
fn finite_lazy_transformer_compositions_realize_to_lists() {
    let mut session = core_session();

    for (source, expected) in [
        (
            "Normal[LazyTake[LazyRange[1, 1], 5]]",
            "List[1, 2, 3, 4, 5]",
        ),
        (
            "Normal[LazyTake[LazyMap[Function[x, x^2], LazyRange[1, 1]], 4]]",
            "List[1, 4, 9, 16]",
        ),
        (
            "Normal[LazyTake[LazyDrop[LazyRange[1, 1], 3], 3]]",
            "List[4, 5, 6]",
        ),
        (
            "Normal[LazySelect[Function[x, 1], LazyTake[LazyRange[1, 1], 3]]]",
            "List[]",
        ),
    ] {
        assert_eq!(eval_text(&mut session, source), expected, "source {source}");
    }
}

#[test]
fn lazy_select_filters_under_bounded_realization() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "Take[LazySelect[EvenQ, LazyRange[1, 1]], 3]"),
        "List[2, 4, 6]"
    );
}

#[test]
fn never_matching_lazy_select_fails_closed_at_source_pull_ceiling() {
    let mut session = core_session();
    session.set_iteration_work_limit(4);

    let outcome = session
        .eval_text("Normal[LazySelect[Function[x, False], LazyRange[1, 1]]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"IterationWorkLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );
}

#[test]
fn lazy_table_realizes_arithmetic_iterator_forms() {
    let mut session = core_session();

    for (source, expected) in [
        ("Normal[LazyTable[i^2, {i, 1, 5}]]", "List[1, 4, 9, 16, 25]"),
        (
            "Normal[LazyTable[i, {i, 2, 10, 2}]]",
            "List[2, 4, 6, 8, 10]",
        ),
        ("Normal[LazyTable[i, {i, 3}]]", "List[1, 2, 3]"),
        ("Normal[LazyTable[i, {i, 5, 1}]]", "List[]"),
        ("Normal[LazyTable[i, {i, 5, 1, -2}]]", "List[5, 3, 1]"),
        (
            "Normal[LazyTable[i, {i, 1/2, 2, 1/2}]]",
            "List[1/2, 1, 3/2, 2]",
        ),
    ] {
        assert_eq!(eval_text(&mut session, source), expected, "source {source}");
    }

    assert_eq!(
        eval_text(&mut session, "Attributes[LazyTable]"),
        "List[HoldAll, Protected]"
    );
}

#[test]
fn lazy_table_holds_its_body_and_binds_the_iterator_lexically() {
    let mut session = core_session();

    assert_eq!(
        eval_text(
            &mut session,
            "n = 0; i = 99; lz = LazyTable[(n = n + 1); i^2, {i, 1, 3}]; n"
        ),
        "0"
    );
    assert_eq!(eval_text(&mut session, "Normal[lz]"), "List[1, 4, 9]");
    assert_eq!(eval_text(&mut session, "n"), "3");
}

#[test]
fn lazy_table_rejects_invalid_specs_without_partial_values() {
    let mut session = core_session();

    for (source, expected, tag) in [
        ("LazyTable[i]", "LazyTable[i]", "argx"),
        ("LazyTable[i, bad]", "LazyTable[i, bad]", "argform"),
        (
            "LazyTable[i, {1, 1, 3}]",
            "LazyTable[i, List[1, 1, 3]]",
            "argform",
        ),
        (
            "LazyTable[i, {i, missing}]",
            "LazyTable[i, List[i, missing]]",
            "argform",
        ),
        (
            "LazyTable[i, {i, 1, 3, 0}]",
            "LazyTable[i, List[i, 1, 3, 0]]",
            "argform",
        ),
        (
            "LazyTable[i, {i, {1, 2}}]",
            "LazyTable[i, List[i, List[1, 2]]]",
            "argform",
        ),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "source {source}");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "LazyTable" && message.tag == tag),
            "source {source}"
        );
    }
}

#[test]
fn lazy_table_result_is_opaque_and_bounded_take_is_lazy() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyTable[i, {i, 1, 3}], _LazyList]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyTable[i, {i, 1, 3}], _List]"),
        "False"
    );
    assert_eq!(
        eval_text(&mut session, "Take[LazyTable[i^2, {i, 1, 1000000}], 3]"),
        "List[1, 4, 9]"
    );
}

#[test]
fn lazy_drop_and_rest_keep_the_tail_lazy() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "Rest[LazyRange[10, 5]]"),
        "LazyList[LazyDrop[LazyRange[10, 5], 1]]"
    );
    assert_eq!(
        eval_text(&mut session, "First[Rest[LazyRange[10, 5]]]"),
        "15"
    );
    assert_eq!(
        eval_text(&mut session, "Normal[LazyTake[Rest[LazyRange[10, 5]], 3]]"),
        "List[15, 20, 25]"
    );
}

#[test]
fn nested_lazy_map_and_select_eval_errors_propagate() {
    let mut session = core_session();

    for (source, expected_value, expected_tag) in [
        (
            "Normal[LazyTake[LazyMap[Function[x, Throw[x, mapTag]], LazyRange[1, 1]], 1]]",
            "1",
            "mapTag",
        ),
        (
            "Normal[LazySelect[Function[x, Throw[x, selectTag]], LazyTake[LazyRange[1, 1], 1]]]",
            "1",
            "selectTag",
        ),
    ] {
        match session
            .eval_text(source)
            .expect_err("nested Throw should propagate out of the lazy realizer")
        {
            EvalError::Thrown { value, tag, .. } => {
                assert_eq!(value.to_string(), expected_value, "source {source}");
                assert_eq!(tag.to_string(), expected_tag, "source {source}");
            }
            other => panic!("expected nested Throw from {source}, got {other:?}"),
        }
    }
}

#[test]
fn snapshot_roundtrip_rederives_composed_lazy_descriptor() {
    let mut source = core_session();
    assert_eq!(
        eval_text(
            &mut source,
            "f[x_] := x + 1; lz = LazyTake[LazyMap[f, LazyRange[3, 2]], 4]"
        ),
        "LazyList[LazyTake[LazyMap[f, LazyRange[3, 2]], 4]]"
    );
    let snapshot = source
        .snapshot_definitions_source()
        .expect("composed lazy-list snapshot should render");

    let mut restored = core_session();
    restored
        .eval_script(&snapshot)
        .expect("composed lazy-list snapshot should restore");

    assert_eq!(
        eval_text(&mut restored, "lz"),
        "LazyList[LazyTake[LazyMap[f, LazyRange[3, 2]], 4]]"
    );
    assert_eq!(eval_text(&mut restored, "Normal[lz]"), "List[4, 6, 8, 10]");
}

#[test]
fn snapshot_roundtrip_rederives_lazy_table_descriptor() {
    let mut source = core_session();
    assert_eq!(
        eval_text(&mut source, "lz = LazyTable[i^2, {i, 2, 6, 2}]"),
        "LazyList[LazyTake[LazyMap[Function[i, Power[i, 2]], LazyRange[2, 2]], 3]]"
    );
    let snapshot = source
        .snapshot_definitions_source()
        .expect("lazy-table snapshot should render");

    let mut restored = core_session();
    restored
        .eval_script(&snapshot)
        .expect("lazy-table snapshot should restore");

    assert_eq!(
        eval_text(&mut restored, "lz"),
        "LazyList[LazyTake[LazyMap[Function[i, Power[i, 2]], LazyRange[2, 2]], 3]]"
    );
    assert_eq!(eval_text(&mut restored, "Normal[lz]"), "List[4, 16, 36]");
}

#[test]
fn positive_integer_part_redispatches_through_lazy_realizers() {
    let mut session = core_session();

    assert_eq!(eval_text(&mut session, "LazyRange[10, 5][[3]]"), "20");
    assert_eq!(eval_text(&mut session, "LazyRange[1, 1][[1]]"), "1");

    let out_of_range = session
        .eval_text("LazyTake[LazyRange[1, 1], 2][[3]]")
        .unwrap();
    assert_eq!(out_of_range.value.to_string(), "First[List[]]");
    assert!(
        out_of_range
            .messages
            .iter()
            .any(|message| message.symbol == "First" && message.tag == "nofirst")
    );
}

#[test]
fn other_lazy_part_specs_follow_the_existing_part_path() {
    let mut session = core_session();

    let non_integer = session.eval_text("LazyRange[1, 1][[x]]").unwrap();
    assert_eq!(
        non_integer.value.to_string(),
        "Part[LazyList[LazyRange[1, 1]], x]"
    );
    assert!(
        non_integer
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "pkspec1")
    );

    assert_eq!(
        eval_text(&mut session, "LazyRange[1, 1][[-1]]"),
        "LazyList[LazyRange[1, 1]]"
    );
    assert_eq!(eval_text(&mut session, "LazyRange[1, 1][[0]]"), "LazyList");
    assert_eq!(eval_text(&mut session, "LazyRange[1, 1][[1, 1]]"), "1");

    let span = session.eval_text("LazyRange[1, 1][[1 ;; 2]]").unwrap();
    assert_eq!(
        span.value.to_string(),
        "Part[LazyList[LazyRange[1, 1]], Span[1, 2]]"
    );
    assert!(
        span.messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "take")
    );
}

#[test]
fn lazy_nest_list_builds_an_opaque_doubling_stream() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "LazyNestList[Function[x, 2 x], 1]"),
        "LazyList[LazyNestList[Function[x, Times[2, x]], 1]]"
    );
    assert_eq!(
        eval_text(
            &mut session,
            "Normal[LazyTake[LazyNestList[Function[x, 2 x], 1], 5]]"
        ),
        "List[1, 2, 4, 8, 16]"
    );
    assert_eq!(
        eval_text(
            &mut session,
            "MatchQ[LazyNestList[Function[x, x + 1], 0], _LazyList]"
        ),
        "True"
    );
    assert_eq!(
        eval_text(
            &mut session,
            "MatchQ[LazyNestList[Function[x, x + 1], 0], _List]"
        ),
        "False"
    );
    assert_eq!(
        eval_text(&mut session, "Attributes[LazyNestList]"),
        "List[Protected]"
    );
}

#[test]
fn normal_on_infinite_lazy_nest_list_fails_closed() {
    let mut session = core_session();
    session.set_iteration_work_limit(3);

    let outcome = session
        .eval_text("Normal[LazyNestList[Function[x, x + 1], 0]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"IterationWorkLimit\"]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );
    assert_eq!(eval_text(&mut session, "1 + 1"), "2");
}

#[test]
fn lazy_fold_list_emits_init_then_running_accumulators() {
    let mut session = core_session();

    assert_eq!(
        eval_text(
            &mut session,
            "LazyFoldList[Plus, 0, LazyTake[LazyRange[1, 1], 4]]"
        ),
        "LazyList[LazyFoldList[Plus, 0, LazyTake[LazyRange[1, 1], 4]]]"
    );
    assert_eq!(
        eval_text(
            &mut session,
            "Normal[LazyFoldList[Plus, 0, LazyTake[LazyRange[1, 1], 4]]]"
        ),
        "List[0, 1, 3, 6, 10]"
    );
    assert_eq!(
        eval_text(
            &mut session,
            "Normal[LazyFoldList[Plus, 10, LazyTake[LazyRange[2, 2], 3]]]"
        ),
        "List[10, 12, 16, 22]"
    );
    assert_eq!(
        eval_text(
            &mut session,
            "Normal[LazyFoldList[Plus, 10, LazyTake[LazyRange[1, 1], 0]]]"
        ),
        "List[10]"
    );
    assert_eq!(
        eval_text(
            &mut session,
            "MatchQ[LazyFoldList[Plus, 0, LazyRange[1, 1]], _LazyList]"
        ),
        "True"
    );
    assert_eq!(
        eval_text(
            &mut session,
            "MatchQ[LazyFoldList[Plus, 0, LazyRange[1, 1]], _List]"
        ),
        "False"
    );
    assert_eq!(
        eval_text(&mut session, "Attributes[LazyFoldList]"),
        "List[Protected]"
    );
}

#[test]
fn new_lazy_builders_reject_invalid_arity_and_fold_input() {
    let mut session = core_session();

    for (source, expected, symbol, tag) in [
        ("LazyNestList[f]", "LazyNestList[f]", "LazyNestList", "argx"),
        (
            "LazyFoldList[f, 0]",
            "LazyFoldList[f, 0]",
            "LazyFoldList",
            "argx",
        ),
        (
            "LazyFoldList[f, 0, {1, 2}]",
            "LazyFoldList[f, 0, List[1, 2]]",
            "LazyFoldList",
            "argform",
        ),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "source {source}");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == symbol && message.tag == tag),
            "source {source}"
        );
    }
}

#[test]
fn lazy_nest_and_fold_nested_errors_propagate() {
    let mut session = core_session();

    for (source, expected_value, expected_tag) in [
        (
            "Normal[LazyTake[LazyNestList[Function[x, Throw[x, nestTag]], 1], 1]]",
            "1",
            "nestTag",
        ),
        (
            "Normal[LazyFoldList[Function[{acc, x}, Throw[x, foldTag]], 0, LazyTake[LazyRange[1, 1], 1]]]",
            "1",
            "foldTag",
        ),
    ] {
        match session
            .eval_text(source)
            .expect_err("nested Throw should propagate out of the lazy realizer")
        {
            EvalError::Thrown { value, tag, .. } => {
                assert_eq!(value.to_string(), expected_value, "source {source}");
                assert_eq!(tag.to_string(), expected_tag, "source {source}");
            }
            other => panic!("expected nested Throw from {source}, got {other:?}"),
        }
    }
}

#[test]
fn snapshot_roundtrip_rederives_lazy_nest_and_fold_descriptors() {
    let mut source = core_session();
    assert_eq!(
        eval_text(
            &mut source,
            "step[x_] := x + 2; combine[x_, y_] := x + y; nest = LazyNestList[step, 1]; fold = LazyFoldList[combine, 10, LazyTake[LazyRange[1, 1], 3]]"
        ),
        "LazyList[LazyFoldList[combine, 10, LazyTake[LazyRange[1, 1], 3]]]"
    );
    let snapshot = source
        .snapshot_definitions_source()
        .expect("lazy NestList and FoldList snapshot should render");

    let mut restored = core_session();
    restored
        .eval_script(&snapshot)
        .expect("lazy NestList and FoldList snapshot should restore");

    assert_eq!(
        eval_text(&mut restored, "nest"),
        "LazyList[LazyNestList[step, 1]]"
    );
    assert_eq!(
        eval_text(&mut restored, "Normal[LazyTake[nest, 4]]"),
        "List[1, 3, 5, 7]"
    );
    assert_eq!(
        eval_text(&mut restored, "fold"),
        "LazyList[LazyFoldList[combine, 10, LazyTake[LazyRange[1, 1], 3]]]"
    );
    assert_eq!(
        eval_text(&mut restored, "Normal[fold]"),
        "List[10, 11, 13, 16]"
    );
}

#[test]
fn lazy_subsets_matches_eager_order_and_edges() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "LazySubsets[{a, b, c}]"),
        "LazyList[LazySubsets[List[a, b, c]]]"
    );
    assert_eq!(
        eval_text(&mut session, "Normal[LazySubsets[{a, b, c}]]"),
        "List[List[], List[a], List[b], List[c], List[a, b], List[a, c], List[b, c], List[a, b, c]]"
    );
    for source in [
        "Normal[LazySubsets[{a, b, c}]] === Subsets[{a, b, c}]",
        "Normal[LazySubsets[{}]] === Subsets[{}]",
        "Normal[LazySubsets[{a}]] === Subsets[{a}]",
    ] {
        assert_eq!(eval_text(&mut session, source), "True", "source {source}");
    }
    assert_eq!(
        eval_text(&mut session, "Attributes[LazySubsets]"),
        "List[Protected]"
    );
}

#[test]
fn lazy_permutations_matches_eager_canonical_order_and_edges() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "LazyPermutations[{c, a, b}]"),
        "LazyList[LazyPermutations[List[c, a, b]]]"
    );
    assert_eq!(
        eval_text(&mut session, "Normal[LazyPermutations[{c, a, b}]]"),
        "List[List[a, b, c], List[a, c, b], List[b, a, c], List[b, c, a], List[c, a, b], List[c, b, a]]"
    );
    for source in [
        "Normal[LazyPermutations[{a, b, c}]] === Permutations[{a, b, c}]",
        "Normal[LazyPermutations[{1, 1, 2}]] === Permutations[{1, 1, 2}]",
        "Normal[LazyPermutations[{}]] === Permutations[{}]",
        "Normal[LazyPermutations[{a}]] === Permutations[{a}]",
        "Normal[LazyPermutations[{c, a, b}]] === Permutations[{c, a, b}]",
    ] {
        assert_eq!(eval_text(&mut session, source), "True", "source {source}");
    }
    assert_eq!(
        eval_text(&mut session, "Attributes[LazyPermutations]"),
        "List[Protected]"
    );
}

#[test]
fn lazy_permutations_large_take_is_lazy_cheap_and_opaque() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "Take[LazyPermutations[Range[12]], 3]"),
        "List[List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 11], List[1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 10, 12]]"
    );
    assert_eq!(
        eval_text(&mut session, "LazyPermutations[Range[12]][[3]]"),
        "List[1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 10, 12]"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyPermutations[{1, 2}], _LazyList]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyPermutations[{1, 2}], _List]"),
        "False"
    );
}

#[test]
fn lazy_subsets_large_take_is_lazy_and_opaque() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "Take[LazySubsets[Range[40]], 3]"),
        "List[List[], List[1], List[2]]"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazySubsets[{1, 2}], _LazyList]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazySubsets[{1, 2}], _List]"),
        "False"
    );
}

#[test]
fn lazy_tuples_matches_eager_order_and_edges() {
    let mut session = core_session();

    assert_eq!(
        eval_text(&mut session, "LazyTuples[{a, b}, 2]"),
        "LazyList[LazyTuples[List[List[a, b], List[a, b]]]]"
    );
    assert_eq!(
        eval_text(&mut session, "Normal[LazyTuples[{a, b}, 2]]"),
        "List[List[a, a], List[a, b], List[b, a], List[b, b]]"
    );
    assert_eq!(
        eval_text(&mut session, "Normal[LazyTuples[{{1, 2}, {3, 4}}]]"),
        "List[List[1, 3], List[1, 4], List[2, 3], List[2, 4]]"
    );
    for source in [
        "Normal[LazyTuples[{a, b}, 2]] === Tuples[{a, b}, 2]",
        "Normal[LazyTuples[{{1, 2}, {3, 4}}]] === Tuples[{{1, 2}, {3, 4}}]",
        "Normal[LazyTuples[{a, b}, 0]] === Tuples[{a, b}, 0]",
        "Normal[LazyTuples[{}]] === Tuples[{}]",
        "Normal[LazyTuples[{{1, 2}, {}}]] === Tuples[{{1, 2}, {}}]",
    ] {
        assert_eq!(eval_text(&mut session, source), "True", "source {source}");
    }
    assert_eq!(
        eval_text(&mut session, "Normal[LazyTuples[{a, b}, 0]]"),
        "List[List[]]"
    );
    assert_eq!(
        eval_text(&mut session, "Normal[LazyTuples[{{1, 2}, {}}]]"),
        "List[]"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyTuples[{a, b}, 2], _LazyList]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut session, "MatchQ[LazyTuples[{a, b}, 2], _List]"),
        "False"
    );
    assert_eq!(
        eval_text(&mut session, "Attributes[LazyTuples]"),
        "List[Protected]"
    );
}

#[test]
fn huge_lazy_combinatorial_spaces_fail_closed_at_iteration_ceiling() {
    let mut session = core_session();
    session.set_iteration_work_limit(3);

    for source in [
        "Normal[LazyPermutations[Range[50]]]",
        "Normal[LazySubsets[Range[40]]]",
        "Normal[LazyTuples[{a, b}, 40]]",
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(
            outcome.value.to_string(),
            "TerminatedEvaluation[\"IterationWorkLimit\"]",
            "source {source}"
        );
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "General" && message.tag == "reslim"),
            "source {source}"
        );
    }
    assert_eq!(eval_text(&mut session, "1 + 1"), "2");
}

#[test]
fn snapshot_roundtrip_rederives_lazy_combinatorial_descriptors() {
    let mut source = core_session();
    assert_eq!(
        eval_text(
            &mut source,
            "subsets = LazySubsets[{a, b, c}]; tuples = LazyTuples[{{1, 2}, {3, 4}}]; permutations = LazyPermutations[{b, a, a}]"
        ),
        "LazyList[LazyPermutations[List[b, a, a]]]"
    );
    let snapshot = source
        .snapshot_definitions_source()
        .expect("lazy combinatorial snapshot should render");

    let mut restored = core_session();
    restored
        .eval_script(&snapshot)
        .expect("lazy combinatorial descriptors should restore");

    assert_eq!(
        eval_text(&mut restored, "subsets"),
        "LazyList[LazySubsets[List[a, b, c]]]"
    );
    assert_eq!(
        eval_text(&mut restored, "Normal[subsets] === Subsets[{a, b, c}]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut restored, "tuples"),
        "LazyList[LazyTuples[List[List[1, 2], List[3, 4]]]]"
    );
    assert_eq!(
        eval_text(&mut restored, "Normal[tuples] === Tuples[{{1, 2}, {3, 4}}]"),
        "True"
    );
    assert_eq!(
        eval_text(&mut restored, "permutations"),
        "LazyList[LazyPermutations[List[b, a, a]]]"
    );
    assert_eq!(
        eval_text(
            &mut restored,
            "Normal[permutations] === Permutations[{b, a, a}]"
        ),
        "True"
    );
}

#[test]
fn lazy_combinatorial_builders_reject_invalid_inputs_without_partial_values() {
    let mut session = core_session();

    for (source, expected, symbol, tag) in [
        (
            "LazyPermutations[x]",
            "LazyPermutations[x]",
            "LazyPermutations",
            "argform",
        ),
        (
            "LazyPermutations[f[a]]",
            "LazyPermutations[f[a]]",
            "LazyPermutations",
            "argform",
        ),
        (
            "LazyPermutations[{}, 1]",
            "LazyPermutations[List[], 1]",
            "LazyPermutations",
            "argx",
        ),
        ("LazySubsets[x]", "LazySubsets[x]", "LazySubsets", "argform"),
        (
            "LazySubsets[{}, 1]",
            "LazySubsets[List[], 1]",
            "LazySubsets",
            "argx",
        ),
        ("LazyTuples[x]", "LazyTuples[x]", "LazyTuples", "argform"),
        (
            "LazyTuples[{{1}, x}]",
            "LazyTuples[List[List[1], x]]",
            "LazyTuples",
            "argform",
        ),
        (
            "LazyTuples[x, 2]",
            "LazyTuples[x, 2]",
            "LazyTuples",
            "argform",
        ),
        (
            "LazyTuples[{a, b}, -1]",
            "LazyTuples[List[a, b], -1]",
            "LazyTuples",
            "argform",
        ),
        ("LazyTuples[]", "LazyTuples[]", "LazyTuples", "argx"),
        (
            "LazyTuples[{a}, 1, 2]",
            "LazyTuples[List[a], 1, 2]",
            "LazyTuples",
            "argx",
        ),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "source {source}");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == symbol && message.tag == tag),
            "source {source}"
        );
    }
}
