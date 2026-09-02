use rum_host::{HostEvalOutcome, HostEvalSuccess, HostSession, HostTyped};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

fn session_with_counting_print(expected: &'static str) -> (HostSession, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let sink_calls = Arc::clone(&calls);
    let mut session = HostSession::new();
    session.set_print_sink(move |line| {
        if line == expected {
            sink_calls.fetch_add(1, Ordering::Relaxed);
        }
    });
    (session, calls)
}

fn describe(outcome: &HostEvalOutcome) -> String {
    match outcome {
        HostEvalOutcome::Success(success) => format!(
            "Success(value={}, termination={:?}, messages={}, prints={:?}, trace={})",
            success.value.text,
            success.termination,
            success.messages.len(),
            success.prints,
            success.trace.len()
        ),
        HostEvalOutcome::InputRequired(suspension) => format!(
            "InputRequired(kind={:?}, prompt={:?}, messages={}, prints={:?})",
            suspension.request.kind,
            suspension.request.prompt,
            suspension.messages.len(),
            suspension.prints
        ),
        HostEvalOutcome::Failure(failure) => format!(
            "Failure(code={:?}, message={:?}, messages={}, prints={:?})",
            failure.code,
            failure.message,
            failure.messages.len(),
            failure.prints
        ),
    }
}

fn record(label: &str, outcome: &HostEvalOutcome) {
    println!("{label}={}", describe(outcome));
}

fn success(session: &mut HostSession, source: &str) -> HostEvalSuccess {
    match session.eval(source) {
        HostEvalOutcome::Success(success) => success,
        other => panic!("expected success for {source:?}, got {other:?}"),
    }
}

fn assert_text(session: &mut HostSession, source: &str, expected: &str) {
    let outcome = session.eval(source);
    record(source, &outcome);
    match outcome {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, expected),
        other => panic!("expected success for {source:?}, got {other:?}"),
    }
}

fn integer_value(session: &mut HostSession, source: &str) -> i64 {
    let result = success(session, source);
    match result.value.typed {
        HostTyped::Integer(value) => value,
        other => panic!("expected integer for {source:?}, got {other:?}"),
    }
}

#[test]
fn public_rollback_matrix_restores_lexical_iterator_attribute_and_option_state() {
    let mut session = HostSession::new();
    success(
        &mut session,
        "block403 = \"outside\"; with403 = \"outside\"; attribute403 = 1; \
         Options[optionProbe403] = {choice403 -> \"default\"}; \
         optionProbe403[OptionsPattern[]] := (InputString[\"option\"]; OptionValue[choice403])",
    );

    let block = session.eval("ValueQ[Block[{block403 = \"inside\"}, InputString[\"block\"]]]");
    record("matrix.block.valueq", &block);
    assert!(matches!(block, HostEvalOutcome::Success(ref value) if value.value.text == "True"));
    assert_text(&mut session, "block403", "\"outside\"");

    let module =
        session.eval("ValueQ[Module[{moduleLocal403 = \"local\"}, InputString[\"module\"]]]");
    record("matrix.module.valueq", &module);
    assert!(matches!(module, HostEvalOutcome::Success(ref value) if value.value.text == "True"));
    assert_text(&mut session, "Names[\"Global`moduleLocal403$*\"]", "List[]");

    let with =
        session.eval("ValueQ[With[{with403 = \"inside\"}, {with403, InputString[\"with\"]}]]");
    record("matrix.with.valueq", &with);
    assert!(matches!(with, HostEvalOutcome::Success(ref value) if value.value.text == "True"));
    assert_text(&mut session, "with403", "\"outside\"");

    let attributes = session.eval(
        "ValueQ[Block[{attribute403}, SetAttributes[attribute403, Protected]; \
         InputString[\"attributes\"]]]",
    );
    record("matrix.attributes.valueq", &attributes);
    assert!(
        matches!(attributes, HostEvalOutcome::Success(ref value) if value.value.text == "True")
    );
    assert_text(&mut session, "Attributes[attribute403]", "List[]");

    let iterator = session.eval("ValueQ[Table[InputString[\"iterator\"], {iterator403, {7}}]]");
    record("matrix.iterator.valueq", &iterator);
    assert!(matches!(iterator, HostEvalOutcome::Success(ref value) if value.value.text == "True"));
    assert_text(&mut session, "iterator403", "iterator403");

    let option = session.eval("ValueQ[optionProbe403[choice403 -> \"explicit\"]]");
    record("matrix.option.valueq", &option);
    assert!(matches!(option, HostEvalOutcome::Success(ref value) if value.value.text == "True"));
    let option_after = session.eval("OptionValue[choice403]");
    record("matrix.option.after", &option_after);
    assert!(matches!(
        option_after,
        HostEvalOutcome::Success(ref value) if value.value.text == "OptionValue[choice403]"
    ));

    let valueq = session.eval("ValueQ[InputString[\"valueq contract\"]]");
    record("matrix.valueq.contract", &valueq);
    assert!(matches!(valueq, HostEvalOutcome::Success(ref value) if value.value.text == "True"));
}

#[test]
fn public_monitor_matrix_restores_reap_monitor_trace_constraints_random_and_abort_state() {
    let (mut session, monitor_refreshes) = session_with_counting_print("monitor-refresh-403");
    success(&mut session, "abortAfter403 = 0");

    let memory_before = integer_value(&mut session, "MemoryInUse[]");
    let reap = session.eval("Monitor[Null, Reap[Sow[Range[2000]]; InputString[\"reap\"]]]");
    record("matrix.reap.monitor", &reap);
    assert!(matches!(reap, HostEvalOutcome::Success(ref value) if value.value.text == "Null"));
    let memory_after = integer_value(&mut session, "MemoryInUse[]");
    let memory_growth = memory_after.saturating_sub(memory_before);
    println!(
        "matrix.reap.memory=before:{memory_before},after:{memory_after},growth:{memory_growth}"
    );
    assert!(
        memory_growth < 16_000,
        "abandoned Reap buckets retained {memory_growth} proxy bytes"
    );

    let monitor = session
        .eval("ValueQ[Monitor[InputString[\"inner monitor\"], Print[\"monitor-refresh-403\"]]]");
    record("matrix.monitor.valueq", &monitor);
    assert!(matches!(monitor, HostEvalOutcome::Success(ref value) if value.value.text == "True"));
    let abandoned_refreshes = monitor_refreshes.load(Ordering::Relaxed);
    let (mut completed_monitor, completed_refreshes) =
        session_with_counting_print("monitor-refresh-403");
    let completed = completed_monitor.eval("Monitor[Null, Print[\"monitor-refresh-403\"]]");
    assert!(matches!(completed, HostEvalOutcome::Success(_)));
    let completed_refreshes = completed_refreshes.load(Ordering::Relaxed);
    println!(
        "matrix.monitor.refreshes=abandoned:{abandoned_refreshes},completed:{completed_refreshes}"
    );
    assert!(
        abandoned_refreshes < completed_refreshes,
        "abandoning Monitor must omit its normal final refresh"
    );

    let trace = session.eval("Monitor[Null, Trace[InputString[\"trace\"]]]");
    record("matrix.trace.monitor", &trace);
    assert!(matches!(trace, HostEvalOutcome::Success(ref value) if value.value.text == "Null"));
    let trace_after = session.eval("1 + 1");
    record("matrix.trace.after", &trace_after);
    assert!(matches!(trace_after, HostEvalOutcome::Success(ref value) if value.trace.is_empty()));

    let time = session.eval("Monitor[Null, TimeConstrained[InputString[\"time\"], 1/20]]");
    record("matrix.time.monitor", &time);
    assert!(matches!(time, HostEvalOutcome::Success(ref value) if value.value.text == "Null"));
    std::thread::sleep(Duration::from_millis(75));
    let time_after = session.eval("Do[Null, {1000}]; \"done\"");
    record("matrix.time.after", &time_after);
    assert!(matches!(
        time_after,
        HostEvalOutcome::Success(ref value) if value.value.text == "\"done\""
    ));

    let memory = session.eval(
        "Monitor[Length[Range[20000]], \
         MemoryConstrained[InputString[\"memory\"], 200000]]",
    );
    record("matrix.memory.monitor", &memory);
    assert!(matches!(memory, HostEvalOutcome::Success(ref value) if value.value.text == "20000"));

    let mut control = HostSession::new();
    success(&mut control, "SeedRandom[403]");
    let expected_random = success(&mut control, "RandomInteger[1000000]").value.text;
    success(&mut session, "SeedRandom[403]");
    let random =
        session.eval("Monitor[Null, BlockRandom[RandomInteger[1000000]; InputString[\"random\"]]]");
    record("matrix.random.monitor", &random);
    assert!(matches!(random, HostEvalOutcome::Success(ref value) if value.value.text == "Null"));
    let random_after = session.eval("RandomInteger[1000000]");
    record("matrix.random.after", &random_after);
    assert!(matches!(
        random_after,
        HostEvalOutcome::Success(ref value) if value.value.text == expected_random
    ));

    let abort_scope = session.eval("Monitor[Null, AbortProtect[InputString[\"abort\"]]]");
    record("matrix.abort_protect.monitor", &abort_scope);
    assert!(
        matches!(abort_scope, HostEvalOutcome::Success(ref value) if value.value.text == "Null")
    );
    let abort = session.eval("Abort[]; abortAfter403 = 1");
    record("matrix.abort_protect.abort", &abort);
    assert_text(&mut session, "abortAfter403", "0");
}

#[test]
fn public_monitor_matrix_restores_capture_package_and_cleanup_owners() {
    let mut session = HostSession::new();
    success(&mut session, "checkFallback403 = 0; cleanupRan403 = 0");

    let evaluation_data = session.eval(
        "Monitor[Null, EvaluationData[Message[General::stop]; Print[\"captured\"]; \
         InputString[\"evaluation data\"]]]",
    );
    record("matrix.capture.evaluation_data", &evaluation_data);
    assert!(matches!(
        evaluation_data,
        HostEvalOutcome::Success(ref value)
            if value.value.text == "Null" && value.messages.is_empty() && value.prints.is_empty()
    ));

    let check = session.eval(
        "Monitor[Null, Check[Message[General::stop]; InputString[\"check\"], \
         checkFallback403 = 1]]",
    );
    record("matrix.capture.check", &check);
    assert!(matches!(
        check,
        HostEvalOutcome::Success(ref value)
            if value.value.text == "Null" && value.messages.is_empty()
    ));
    assert_text(&mut session, "checkFallback403", "0");

    let quiet =
        session.eval("Monitor[Null, Quiet[Message[General::stop]; InputString[\"quiet\"]]]");
    record("matrix.capture.quiet", &quiet);
    assert!(matches!(
        quiet,
        HostEvalOutcome::Success(ref value)
            if value.value.text == "Null" && value.messages.is_empty()
    ));

    let test_report = session.eval(
        "Monitor[Null, TestReport[{TestCreate[Print[\"case\"]; \
         InputString[\"test report\"], \"ok\"]}]]",
    );
    record("matrix.capture.test_report", &test_report);
    assert!(matches!(
        test_report,
        HostEvalOutcome::Success(ref value)
            if value.value.text == "Null" && value.prints.is_empty()
    ));

    let package = session.eval(
        "Monitor[Null, DefinePackage[\"AbandonPublic403`\", Public[{f}], \
         helper = 41; waiting = InputString[\"package\"]; f[] := helper]]",
    );
    record("matrix.package.monitor", &package);
    assert!(matches!(
        package,
        HostEvalOutcome::Success(ref value)
            if value.value.text == "Null" && value.messages.is_empty()
    ));
    assert_text(&mut session, "Names[\"AbandonPublic403`*\"]", "List[]");

    let (mut cleanup_session, cleanup_prints) = session_with_counting_print("cleanup-403");
    let cleanup = cleanup_session
        .eval("ValueQ[WithCleanup[InputString[\"cleanup\"], Print[\"cleanup-403\"]]]");
    record("matrix.with_cleanup.valueq", &cleanup);
    assert!(matches!(cleanup, HostEvalOutcome::Success(ref value) if value.value.text == "True"));
    assert_eq!(
        cleanup_prints.load(Ordering::Relaxed),
        1,
        "WithCleanup abandonment must evaluate its cleanup exactly once"
    );

    let monitor_contract = session.eval("Monitor[Null, InputString[\"ignored\"]]");
    record("matrix.monitor.contract", &monitor_contract);
    assert!(matches!(
        monitor_contract,
        HostEvalOutcome::Success(ref value) if value.value.text == "Null"
    ));
}
