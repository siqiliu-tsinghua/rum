use super::*;

fn value_q_session() -> Session {
    session_with_test_control_builtins()
}

#[test]
fn value_q_stability_shortcut_matches_the_trial_path_over_one_shared_corpus() {
    let _entry = crate::runtime::native_stack_guard::NativeStackEntry::enter();
    let mut session = value_q_session();
    session
        .eval_text(
            "valueQImmediate = 7; \
             valueQDelayed := 8; \
             valueQExact[1] = 11; \
             valueQPattern[x_] := x + 1; \
             valueQGuard[x_] := x + 1 /; MatchQ[x, 1]; \
             valueQPartData = {10, 20}",
        )
        .unwrap();

    let corpus = [
        ("undefined symbol", "valueQUndefined"),
        ("defined immediate own-value", "valueQImmediate"),
        ("defined delayed own-value", "valueQDelayed"),
        ("integer", "17"),
        ("string", "\"text\""),
        ("list", "{valueQUndefined, 1}"),
        ("matching exact down-value", "valueQExact[1]"),
        ("proven exact down-value miss", "valueQExact[2]"),
        ("pattern down-value", "valueQPattern[3]"),
        ("accepted Condition guard", "valueQGuard[1]"),
        ("declined Condition guard", "valueQGuard[-1]"),
        ("built-in call", "1 + 1"),
        ("nested call", "{valueQExact[1], {valueQUndefined}}"),
        ("stable association", "<|\"stable\" -> valueQUndefined|>"),
        ("changing association", "<|\"changed\" -> valueQExact[1]|>"),
        ("literal Part", "{10, 20}[[1]]"),
        ("own-value Part", "valueQPartData[[2]]"),
    ];
    let false_expr = system_expr("False");
    let mut stable_cases = Vec::new();

    for (description, source) in corpus {
        let target = session
            .lower_text(source)
            .unwrap_or_else(|error| panic!("{description} failed to lower: {error}"));
        let stable = session.cached_expr_is_evaluation_stable(&target);
        let trial_answer = session.eval_value_q_via_trial(&target);
        let shortcut_answer = session
            .eval_value_q(std::slice::from_ref(&target))
            .unwrap_or_else(|error| panic!("{description} shortcut failed: {error}"));

        assert_eq!(
            shortcut_answer, trial_answer,
            "ValueQ paths diverged for {description}: {source}"
        );
        if stable {
            assert_eq!(
                trial_answer, false_expr,
                "the stability proof disagreed with trial evaluation for {description}: {source}"
            );
            stable_cases.push(description);
        }
    }

    assert!(stable_cases.contains(&"undefined symbol"));
    assert!(stable_cases.contains(&"integer"));
    assert!(stable_cases.contains(&"string"));
    assert!(stable_cases.contains(&"list"));
    assert!(stable_cases.contains(&"proven exact down-value miss"));
}

#[test]
fn value_q_shortcut_skips_snapshot_and_suppression_even_after_attribute_revision() {
    let mut session = value_q_session();
    let target = session.lower_text("valueQRevisionTarget").unwrap();
    let target_symbol = *expr_as_symbol(&target).unwrap();

    take_value_q_trial_setup_counts();
    assert_eq!(
        session
            .eval_text("ValueQ[valueQRevisionTarget]")
            .unwrap()
            .value,
        system_expr("False")
    );
    assert_eq!(take_value_q_trial_setup_counts(), (0, 0));
    assert!(!session.value_q_effect_suppression_active);
    assert!(session.stable_symbol_cache.contains_key(&target_symbol));

    session
        .eval_text("SetAttributes[valueQRevisionTarget, HoldFirst]")
        .unwrap();
    assert!(!session.stable_symbol_cache.contains_key(&target_symbol));

    take_value_q_trial_setup_counts();
    assert_eq!(
        session
            .eval_text("ValueQ[valueQRevisionTarget]")
            .unwrap()
            .value,
        system_expr("False")
    );
    assert_eq!(take_value_q_trial_setup_counts(), (0, 0));
    assert!(!session.value_q_effect_suppression_active);

    assert_eq!(
        session.eval_text("ValueQ[1 + 1]").unwrap().value,
        system_expr("True")
    );
    assert_eq!(take_value_q_trial_setup_counts(), (1, 1));
    assert!(!session.value_q_effect_suppression_active);
}

#[test]
fn value_q_shortcut_populates_only_evaluation_caches() {
    let mut session = value_q_session();
    let target = session.lower_text("{valueQCacheOnlyUndefined, 1}").unwrap();
    let definitions_before = session.definitions.clone();
    let attributes_before = session.attributes.clone();
    let options_before = session.options.clone();
    let syntax_information_before = session.syntax_information.clone();
    let message_definitions_before = session.message_definitions.clone();
    let messages_before = session.messages.clone();
    let prints_before = session.prints.clone();
    let trace_before = session.trace_events.clone();
    let prng_before = session.prng_state;
    let capability_policy_before = session.capability_policy;
    let context_before = session.current_context.clone();
    let context_path_before = session.context_path.clone();
    let attribute_revision_before = session.attribute_revision;
    let evaluation_epoch_before = session.evaluation_epoch;
    let symbol_generation_before = session.symbol_cache_generation;
    let call_generation_before = session.call_stability_generation;
    let stable_symbol_entries_before = session.stable_symbol_cache.len();
    let stable_call_entries_before = session.call_stability_cache.len();

    assert_eq!(
        session.eval_value_q(std::slice::from_ref(&target)).unwrap(),
        system_expr("False")
    );

    assert!(session.stable_symbol_cache.len() > stable_symbol_entries_before);
    assert!(session.call_stability_cache.len() > stable_call_entries_before);
    assert_eq!(session.definitions, definitions_before);
    assert_eq!(session.attributes, attributes_before);
    assert_eq!(session.options, options_before);
    assert_eq!(session.syntax_information, syntax_information_before);
    assert_eq!(session.message_definitions, message_definitions_before);
    assert_eq!(session.messages, messages_before);
    assert_eq!(session.prints, prints_before);
    assert_eq!(session.trace_events, trace_before);
    assert_eq!(session.prng_state, prng_before);
    assert_eq!(session.capability_policy, capability_policy_before);
    assert_eq!(session.current_context, context_before);
    assert_eq!(session.context_path, context_path_before);
    assert_eq!(session.attribute_revision, attribute_revision_before);
    assert_eq!(session.evaluation_epoch, evaluation_epoch_before);
    assert_eq!(session.symbol_cache_generation, symbol_generation_before);
    assert_eq!(session.call_stability_generation, call_generation_before);
    assert!(!session.value_q_effect_suppression_active);
    assert!(session.pending_input_request.is_none());
    assert!(session.pending_suspension.is_none());
}
