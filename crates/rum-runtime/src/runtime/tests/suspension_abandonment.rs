use super::*;
use crate::runtime::suspension::AbandonReason;
use std::time::Duration;

fn suspend(session: &mut Session, source: &str) {
    let expr = session.lower_text(source).unwrap();
    let error = session
        .evaluate(expr)
        .expect_err("the probe should suspend for host input");
    assert!(
        matches!(error, EvalError::InputRequired { .. }),
        "expected input-required suspension, got {error:?}"
    );
    assert!(session.pending_input_request.is_some());
    assert!(session.pending_suspension.is_some());
}

fn abandon(session: &mut Session, reason: AbandonReason) {
    assert_eq!(session.abandon_pending_suspension(reason), reason);
    assert!(session.pending_input_request.is_none());
    assert!(session.pending_request.is_none());
    assert!(session.pending_suspension.is_none());
    assert!(session.resuming_suspension.is_none());
    assert!(session.abandonment_worklist.is_empty());
    assert!(!session.abandonment_active);
}

fn value(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn abandonment_generations_are_monotonic_and_pure_frames_run_no_continuation() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("later = 0").unwrap();

    suspend(&mut session, "InputString[\"first: \"]; later = 1");
    let first = session.pending_suspension_generation().unwrap();
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(value(&mut session, "later"), "0");

    suspend(&mut session, "InputString[\"second: \"]");
    let second = session.pending_suspension_generation().unwrap();
    assert!(second > first);
    abandon(&mut session, AbandonReason::CancelledRequest);
}

#[test]
fn queued_input_abandons_the_older_generation_before_consuming_the_value() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("queuedBlock403 = outside").unwrap();

    suspend(
        &mut session,
        "Block[{queuedBlock403 = inside}, InputString[\"old\"]]",
    );
    session.queue_input_response("drain");

    let input = session.lower_text("InputString[\"new\"]").unwrap();
    assert_eq!(session.evaluate(input).unwrap(), Expr::string("drain"));
    assert!(session.queued_input_responses.is_empty());
    assert!(session.pending_suspension.is_none());
    assert_eq!(value(&mut session, "queuedBlock403"), "outside");
}

#[test]
fn b3_superseded_queued_input_does_not_report_the_cleanup_throw() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("queuedCleaned630 = 0").unwrap();

    suspend(
        &mut session,
        "WithCleanup[InputString[\"old\"], (queuedCleaned630 = 1; Throw[630])]",
    );
    session.queue_input_response("drain");

    let input = session.lower_text("InputString[\"new\"]").unwrap();
    assert_eq!(session.evaluate(input).unwrap(), Expr::string("drain"));
    assert!(session.queued_input_responses.is_empty());
    assert!(session.pending_suspension.is_none());
    assert_eq!(value(&mut session, "queuedCleaned630"), "1");
    let tags: Vec<_> = session
        .messages()
        .iter()
        .map(|message| format!("{}::{}", message.symbol, message.tag))
        .collect();
    eprintln!("B3QVERDICT tags={tags:?}");
    assert_eq!(tags, [] as [&str; 0]);
}

#[test]
fn checkpoint_rollback_abandons_a_trial_generation_before_restoring_state() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("rollbackBlock403 = outside").unwrap();
    let snapshot = session.capture_evaluation_rollback_snapshot();

    suspend(
        &mut session,
        "Block[{rollbackBlock403 = inside}, Table[InputString[\"trial\"], {rollbackIterator403, {7}}]]",
    );
    assert!(session.pending_suspension_generation().is_some());

    session.restore_evaluation_rollback_snapshot(snapshot);

    assert!(session.pending_suspension_generation().is_none());
    assert_eq!(
        value(&mut session, "{rollbackBlock403, rollbackIterator403}"),
        "List[outside, rollbackIterator403]"
    );
}

#[test]
fn checkpoint_rollback_preserves_its_captured_generation() {
    let mut session = session_with_test_control_builtins();
    suspend(&mut session, "InputString[\"captured\"]");
    let captured_generation = session.pending_suspension_generation().unwrap();
    let snapshot = session.capture_evaluation_rollback_snapshot();

    session.queue_input_response("temporary");
    assert!(session.pending_input_request().is_none());
    session.restore_evaluation_rollback_snapshot(snapshot);

    assert_eq!(
        session.pending_suspension_generation(),
        Some(captured_generation)
    );
    assert_eq!(
        session
            .pending_input_request()
            .map(|request| request.prompt.as_str()),
        Some("captured")
    );
    assert!(session.queued_input_responses.is_empty());
    abandon(&mut session, AbandonReason::CheckpointRollbackDiscard);
}

#[test]
fn abandonment_restores_block_initializers_and_bodies_inner_to_outer() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("x = outside; y = outsideY; blockLater = 0")
        .unwrap();

    suspend(
        &mut session,
        "Block[{x = inside, y = InputString[\"init: \"]}, blockLater = 1]",
    );
    assert!(!session.dynamic_own_values.is_empty());
    abandon(&mut session, AbandonReason::HelperBestEffortDiscard);
    assert_eq!(
        value(&mut session, "{x, y, blockLater}"),
        "List[outside, outsideY, 0]"
    );

    suspend(
        &mut session,
        "Block[{x = inside, y = insideY}, InputString[\"body: \"]; blockLater = 2]",
    );
    assert!(!session.dynamic_own_values.is_empty());
    abandon(&mut session, AbandonReason::HelperBestEffortDiscard);
    assert_eq!(
        value(&mut session, "{x, y, blockLater}"),
        "List[outside, outsideY, 0]"
    );
}

#[test]
fn abandonment_finishes_module_activation_without_initializers_or_body() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("moduleLater = 0").unwrap();
    let activation_depth = session.module_activations.len();

    suspend(
        &mut session,
        "Module[{a = 1, b = InputString[\"module: \"], c = (moduleLater = 1)}, moduleLater = 2]",
    );
    assert_eq!(session.module_activations.len(), activation_depth + 1);
    let gensyms = session.module_activations[activation_depth].gensyms.clone();
    assert!(!gensyms.is_empty());

    abandon(&mut session, AbandonReason::CheckpointRollbackDiscard);
    assert_eq!(session.module_activations.len(), activation_depth);
    assert!(
        gensyms
            .iter()
            .all(|symbol| !session.definitions.contains_key(symbol))
    );
    assert_eq!(value(&mut session, "moduleLater"), "0");
}

#[test]
fn abandonment_restores_both_loop_iterator_scope_forms_and_iteration_budget() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("cleanLoopLater = 0; fullLoopLater = 0; fullIterator = outside")
        .unwrap();
    let dynamic_depth = session.dynamic_own_values.len();
    session.iteration_product_budget.set((17, 99));

    suspend(
        &mut session,
        "Table[InputString[\"clean loop: \"]; cleanLoopLater = 1, {cleanIterator, 2}]",
    );
    assert!(session.dynamic_own_values.len() > dynamic_depth);
    assert_ne!(session.iteration_product_budget.get(), (17, 99));
    abandon(&mut session, AbandonReason::SupersededOrQueuedInputDiscard);
    assert_eq!(session.dynamic_own_values.len(), dynamic_depth);
    assert_eq!(session.iteration_product_budget.get(), (17, 99));
    assert_eq!(value(&mut session, "cleanLoopLater"), "0");

    suspend(
        &mut session,
        "Table[InputString[\"full loop: \"]; fullLoopLater = 1, {fullIterator, 2}]",
    );
    abandon(&mut session, AbandonReason::SupersededOrQueuedInputDiscard);
    assert_eq!(session.iteration_product_budget.get(), (17, 99));
    assert_eq!(
        value(&mut session, "{fullIterator, fullLoopLater}"),
        "List[outside, 0]"
    );
}

#[test]
fn abandonment_pops_option_value_scope_without_running_definition_suffix() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text(
            "Options[optionProbe] = {choice -> default}; optionLater = 0; \
             optionProbe[OptionsPattern[]] := (InputString[\"option: \"]; optionLater = 1; OptionValue[choice])",
        )
        .unwrap();
    let scope_depth = session.option_value_scopes.len();

    suspend(&mut session, "optionProbe[choice -> explicit]");
    assert_eq!(session.option_value_scopes.len(), scope_depth + 1);
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(session.option_value_scopes.len(), scope_depth);
    assert_eq!(value(&mut session, "optionLater"), "0");
}

#[test]
fn abandonment_pops_reap_without_rendering_or_running_body_suffix() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("reapRender = 0; reapLater = 0").unwrap();
    let scope_depth = session.reap_scopes.len();

    suspend(
        &mut session,
        "Reap[Sow[1]; InputString[\"reap: \"]; reapLater = 1, _, Function[reapRender = reapRender + 1]]",
    );
    assert_eq!(session.reap_scopes.len(), scope_depth + 1);
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(session.reap_scopes.len(), scope_depth);
    assert_eq!(value(&mut session, "{reapRender, reapLater}"), "List[0, 0]");
}

#[test]
fn abandonment_pops_monitor_without_a_final_refresh_or_callback() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("monitorRefresh = 0; monitorCallback = 0; monitorLater = 0")
        .unwrap();
    let scope_depth = session.monitor_scopes.len();

    suspend(
        &mut session,
        "Monitor[InputString[\"monitor: \"]; monitorLater = 1, \
         monitorRefresh = monitorRefresh + 1, StepMonitor :> (monitorCallback = monitorCallback + 1)]",
    );
    assert_eq!(session.monitor_scopes.len(), scope_depth + 1);
    let refresh_symbol = Symbol::new(GLOBAL_CONTEXT, "monitorRefresh");
    let callback_symbol = Symbol::new(GLOBAL_CONTEXT, "monitorCallback");
    let refresh_before = session.definitions.get(&refresh_symbol).cloned();
    let callback_before = session.definitions.get(&callback_symbol).cloned();
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(session.monitor_scopes.len(), scope_depth);
    assert_eq!(
        session.definitions.get(&refresh_symbol),
        refresh_before.as_ref()
    );
    assert_eq!(
        session.definitions.get(&callback_symbol),
        callback_before.as_ref()
    );
    assert_eq!(value(&mut session, "monitorLater"), "0");
}

#[test]
fn abandonment_restores_constraint_nesting_without_fallbacks() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("constraintLater = 0; timeFallback = 0; memoryFallback = 0")
        .unwrap();
    assert!(session.current_time_deadline.is_none());
    assert!(session.current_memory_limit.is_none());

    suspend(
        &mut session,
        "TimeConstrained[MemoryConstrained[InputString[\"constraint: \"]; constraintLater = 1, \
         1000000000, memoryFallback = 1], 1000, timeFallback = 1]",
    );
    assert!(session.current_time_deadline.is_some());
    assert_eq!(session.current_memory_limit, Some(1_000_000_000));
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert!(session.current_time_deadline.is_none());
    assert!(session.current_memory_limit.is_none());
    assert_eq!(
        value(
            &mut session,
            "{constraintLater, timeFallback, memoryFallback}"
        ),
        "List[0, 0, 0]"
    );
}

#[test]
fn abandonment_restores_trace_and_evaluation_data_captures_without_rendering() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text(
            "traceLater = 0; evalDataLater = 0; evalDataResult = marker; \
             captureProbe::captured = \"captured\"",
        )
        .unwrap();

    let trace_checkpoint = session.trace_events.len();
    suspend(
        &mut session,
        "Trace[InputString[\"trace: \"]; traceLater = 1]",
    );
    assert!(session.tracing_enabled);
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert!(!session.tracing_enabled);
    assert_eq!(session.trace_events.len(), trace_checkpoint);
    assert_eq!(value(&mut session, "traceLater"), "0");

    let message_checkpoint = session.messages.len();
    let print_checkpoint = session.prints.len();
    let trace_checkpoint = session.trace_events.len();
    let handled_checkpoint = session.handled_non_loader_eval_errors.len();
    suspend(
        &mut session,
        "evalDataResult = EvaluationData[Message[captureProbe::captured]; Print[\"captured\"]; \
         InputString[\"evaluation data: \"]; evalDataLater = 1]",
    );
    assert!(session.tracing_enabled);
    assert!(session.messages.len() > message_checkpoint);
    assert!(session.prints.len() > print_checkpoint);
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert!(!session.tracing_enabled);
    assert_eq!(session.messages.len(), message_checkpoint);
    assert_eq!(session.prints.len(), print_checkpoint);
    assert_eq!(session.trace_events.len(), trace_checkpoint);
    assert_eq!(
        session.handled_non_loader_eval_errors.len(),
        handled_checkpoint
    );
    assert_eq!(
        value(&mut session, "{evalDataResult, evalDataLater}"),
        "List[marker, 0]"
    );
}

#[test]
fn abandonment_restores_check_and_quiet_captures_without_check_fallback() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text(
            "checkFallback = 0; checkLater = 0; quietLater = 0; \
             captureProbe::check = \"check\"; captureProbe::quiet = \"quiet\"",
        )
        .unwrap();

    let message_checkpoint = session.messages.len();
    let handled_checkpoint = session.handled_non_loader_eval_errors.len();
    suspend(
        &mut session,
        "Check[Message[captureProbe::check]; InputString[\"check: \"]; checkLater = 1, \
         checkFallback = 1]",
    );
    assert!(session.messages.len() > message_checkpoint);
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(session.messages.len(), message_checkpoint);
    assert_eq!(
        session.handled_non_loader_eval_errors.len(),
        handled_checkpoint
    );
    assert_eq!(
        value(&mut session, "{checkFallback, checkLater}"),
        "List[0, 0]"
    );

    suspend(
        &mut session,
        "Quiet[Message[captureProbe::quiet]; InputString[\"quiet: \"]; quietLater = 1]",
    );
    assert!(session.messages.len() > message_checkpoint);
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(session.messages.len(), message_checkpoint);
    assert_eq!(
        session.handled_non_loader_eval_errors.len(),
        handled_checkpoint
    );
    assert_eq!(value(&mut session, "quietLater"), "0");
}

#[test]
fn abandonment_restores_test_report_stream_capture_without_a_report() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("reportResult = marker; reportLater = 0")
        .unwrap();
    let message_checkpoint = session.messages.len();
    let print_checkpoint = session.prints.len();
    let handled_checkpoint = session.handled_non_loader_eval_errors.len();

    suspend(
        &mut session,
        "reportResult = TestReport[{TestCreate[Print[\"case\"]; InputString[\"test: \"]; \
         reportLater = 1, \"ok\"]}]",
    );
    assert!(session.prints.len() > print_checkpoint);
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(session.messages.len(), message_checkpoint);
    assert_eq!(session.prints.len(), print_checkpoint);
    assert_eq!(
        session.handled_non_loader_eval_errors.len(),
        handled_checkpoint
    );
    assert_eq!(
        value(&mut session, "{reportResult, reportLater}"),
        "List[marker, 0]"
    );
}

#[test]
fn abandonment_restores_abort_protect_and_check_abort_checkpoints() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("checkAbortFallback = 0; afterAbort = 0")
        .unwrap();
    let depth = session.abort_protect_depth;
    let pending = session.abort_protect_pending;

    suspend(
        &mut session,
        "AbortProtect[Abort[]; CheckAbort[InputString[\"abort: \"], checkAbortFallback = 1]]",
    );
    assert_eq!(session.abort_protect_depth, depth + 1);
    assert_eq!(session.abort_protect_pending, pending + 1);
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(session.abort_protect_depth, depth);
    assert_eq!(session.abort_protect_pending, pending);
    assert_eq!(value(&mut session, "checkAbortFallback"), "0");

    let abort_expr = session.lower_text("Abort[]; afterAbort = 1").unwrap();
    assert!(matches!(
        session.evaluate(abort_expr),
        Err(EvalError::Abort)
    ));
    assert_eq!(value(&mut session, "afterAbort"), "0");
}

#[test]
fn abandonment_restores_block_random_prng_state() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("randomLater = 0").unwrap();
    let saved = session.prng_state;
    let body = session
        .lower_text("InputString[\"random: \"]; randomLater = 1")
        .unwrap();

    assert!(matches!(
        session.evaluate_with_random_state_restored(body),
        Err(EvalError::InputRequired { .. })
    ));
    let _ = session.random_u64();
    assert_ne!(session.prng_state, saved);
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(session.prng_state, saved);
    assert_eq!(value(&mut session, "randomLater"), "0");
}

#[test]
fn abandonment_rolls_back_partial_package_install_without_reporting_failure() {
    let mut session = session_with_test_control_builtins();
    let messages_before = session.messages.len();

    suspend(
        &mut session,
        "DefinePackage[\"AbandonPkg`\", Public[{f}], helper = 41; \
         waiting = InputString[\"package: \"]; f[] := helper]",
    );
    let minted = session
        .definitions
        .keys()
        .filter(|symbol| symbol.context().starts_with("AbandonPkg`"))
        .copied()
        .collect::<Vec<_>>();
    assert!(!minted.is_empty());
    assert!(session.sealed_units.is_empty());

    abandon(&mut session, AbandonReason::CancelledRequest);
    assert!(
        minted
            .iter()
            .all(|symbol| !session.definitions.contains_key(symbol))
    );
    assert!(
        session
            .sealed_units
            .iter()
            .all(|unit| unit.package_context != "AbandonPkg`")
    );
    assert_eq!(session.messages.len(), messages_before);
}

#[test]
fn abandonment_evaluates_only_withcleanup_cleanup_and_abandons_nested_input_lifo() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("primaryLater = 0; cleanupRan = 0; nestedCleanupRan = 0; nestedLater = 0")
        .unwrap();

    suspend(
        &mut session,
        "WithCleanup[InputString[\"primary: \"]; primaryLater = 1, \
         cleanupRan = 1; WithCleanup[InputString[\"nested cleanup: \"], \
         nestedCleanupRan = 1]; nestedLater = 1]",
    );
    let reason = AbandonReason::CancelledRequest;
    assert_eq!(session.abandon_pending_suspension(reason), reason);
    assert!(session.pending_input_request.is_none());
    assert!(session.pending_suspension.is_none());
    assert!(session.abandonment_worklist.is_empty());
    assert!(!session.abandonment_active);
    assert_eq!(
        value(
            &mut session,
            "{primaryLater, cleanupRan, nestedCleanupRan, nestedLater}"
        ),
        "List[0, 1, 1, 0]"
    );

    session
        .eval_text("initPrimary = 0; initCleanup = 0")
        .unwrap();
    suspend(
        &mut session,
        "WithCleanup[InputString[\"init: \"], initPrimary = 1, initCleanup = 1]",
    );
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(
        value(&mut session, "{initPrimary, initCleanup}"),
        "List[0, 1]"
    );
}

#[test]
fn abandonment_restores_an_existing_time_deadline_exactly() {
    let mut session = session_with_test_control_builtins();
    let prior = std::time::Instant::now() + Duration::from_secs(2_000);
    session.current_time_deadline = Some(prior);

    suspend(
        &mut session,
        "TimeConstrained[InputString[\"deadline: \"], 1000]",
    );
    assert_ne!(session.current_time_deadline, Some(prior));
    abandon(&mut session, AbandonReason::CancelledRequest);
    assert_eq!(session.current_time_deadline, Some(prior));
}

#[path = "capture_output.rs"]
mod capture_output;
