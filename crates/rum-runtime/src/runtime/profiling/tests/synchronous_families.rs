//! Stage-A collection proofs. Registered below epoch to inspect clock equality.
use super::*;
use crate::runtime::profiling::logical::{Call, PhaseKey, synchronous};
use crate::runtime::profiling::state;
use crate::runtime::profiling::tests::engine_support as support;
use crate::runtime::profiling::{self, RuntimeProfileOutcome};
use crate::runtime::{Session, SessionRequestError, Symbol};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicU8;
use std::time::Duration;

struct Flags {
    lexical: Vec<(&'static AtomicU8, u8)>,
    cache: [u8; 3],
}

impl Flags {
    fn set(logical: u8, active: u8, rhs: u8, timing: u8, cache: u8, cache_active: u8) -> Self {
        let lexical = [
            (&state::LEXICAL_PROFILE_ENABLED, logical),
            (&state::LEXICAL_PROFILE_ACTIVE, active),
            (&state::RULE_RHS_TIMING_ENABLED, rhs),
            (&state::DEFINITION_RULE_TIMING_ENABLED, timing),
            (&state::LEXICAL_PROFILE_INCREMENTAL_ENABLED, 2),
        ]
        .into_iter()
        .map(|(flag, value)| (flag, flag.swap(value, Ordering::Relaxed)))
        .collect();
        Self {
            lexical,
            cache: Session::test_cache_profile_flags([cache, cache_active, 2]),
        }
    }
}

impl Drop for Flags {
    fn drop(&mut self) {
        for (flag, value) in &self.lexical {
            flag.store(*value, Ordering::Relaxed);
        }
        Session::test_cache_profile_flags(self.cache);
    }
}

fn rows(epoch: &Epoch) -> BTreeMap<EventKey, ([u64; 7], [u64; 4], u64)> {
    epoch
        .event_rows()
        .into_iter()
        .map(|(key, value)| (key, (value.vector, value.durations, value.input_required)))
        .collect()
}

#[test]
fn equal_keys_sum_and_distinct_semantic_keys_survive() {
    let _enabled = support::enabled();
    let epoch = support::epoch();
    let mut keys = Vec::new();
    for identity in [3, 9] {
        let symbol = Symbol::with_identity("Global`", "same", identity);
        keys.extend([
            EventKey::Runtime {
                phase: "sync",
                source: Some(symbol),
            },
            EventKey::Rhs {
                phase: "sync",
                source: Some(symbol),
            },
            EventKey::Dispatch {
                kind: "down",
                symbol,
                source: Some(symbol),
            },
            EventKey::CacheSymbol {
                event: "down_value_scans",
                symbol,
            },
            EventKey::CacheSource {
                event: "definition_rhs_eval_by_source",
                symbol,
            },
        ]);
    }
    keys.extend([
        EventKey::Counter("definition_rhs_eval"),
        EventKey::Cache("EVALUATE_ENTRY_COUNT"),
    ]);
    let delta = EventValue {
        vector: [1, 2, 3, 4, 5, 6, 7],
        durations: [8, 9, 10, 11],
        input_required: 12,
    };
    for key in &keys {
        epoch.event(*key, delta, None);
        epoch.event(*key, delta, None);
    }
    assert_eq!(epoch.event_rows().len(), keys.len());
    let actual = rows(&epoch);
    for key in keys {
        assert_eq!(
            actual[&key],
            ([2, 4, 6, 8, 10, 12, 14], [16, 18, 20, 22], 24)
        );
    }
}

#[test]
fn runtime_excludes_every_declared_native_phase() {
    let _enabled = support::enabled();
    let epoch = support::epoch();
    let session = Session::new();
    for phase in PhaseKey::ALL {
        assert_eq!(PhaseKey::from_name(phase.name()), Some(phase));
        let call = Call::start(
            &session,
            AggregateKey::Runtime {
                phase: phase.name(),
                source: None,
            },
            Policy::Expr,
        )
        .unwrap();
        let token = profiling::runtime_path_timing_start(phase.name());
        profiling::record_profile_runtime_path_timing(token, RuntimeProfileOutcome::Ok);
        call.finish(Outcome::Ok);
    }
    assert_eq!(epoch.rows().len(), PhaseKey::ALL.len());
    assert!(
        epoch.event_rows().is_empty(),
        "native phases must have no side writer"
    );
    for outcome in [
        RuntimeProfileOutcome::Ok,
        RuntimeProfileOutcome::Some,
        RuntimeProfileOutcome::None,
        RuntimeProfileOutcome::Return,
        RuntimeProfileOutcome::Throw,
        RuntimeProfileOutcome::Error,
    ] {
        let token = profiling::runtime_path_timing_start("synchronous.control");
        profiling::record_profile_runtime_path_timing(token, outcome);
    }
    assert_eq!(
        rows(&epoch)[&EventKey::Runtime {
            phase: "synchronous.control",
            source: None
        }]
            .0,
        [6, 3, 3, 1, 1, 1, 1]
    );
}

#[test]
fn non_adopted_input_required_fault_preserves_guest_outcome() {
    fn guest() -> (Vec<String>, String) {
        let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
        let mut outcome = session
            .eval_request(r#"Function[{},(InputString["fault-a"];InputString["fault-b"];7)][]"#);
        let mut prompts = Vec::new();
        for (prompt, answer) in [("fault-a", "answer-a"), ("fault-b", "answer-b")] {
            let SessionRequestError::InputRequired { request } = outcome.unwrap_err().error else {
                panic!("expected the next guest input pause");
            };
            assert_eq!(request.kind, crate::runtime::RuntimeInputKind::InputString);
            assert_eq!(request.prompt, prompt);
            prompts.push(request.prompt);
            outcome = session.resume_eval_request_with_input(answer);
        }
        let value = outcome.unwrap().value.to_string();
        assert_eq!(value, "7");
        (prompts, value)
    }

    let _enabled = support::enabled();
    let expected = {
        let _disabled = Flags::set(2, 2, 2, 2, 2, 2);
        guest()
    };
    let _flags = Flags::set(1, 1, 2, 2, 2, 2);
    let epoch = support::epoch();
    let phase = "synchronous.forced_input_required";
    assert!(PhaseKey::from_name(phase).is_none());
    let token = profiling::runtime_path_timing_start(phase);
    assert!(token.is_some());
    profiling::record_profile_runtime_path_timing(token, RuntimeProfileOutcome::InputRequired);
    assert_eq!(epoch.faults.load(Ordering::Relaxed), 1);
    assert!(!epoch.is_valid());
    assert!(epoch.event_rows().is_empty());
    assert_eq!(
        guest(),
        expected,
        "diagnostic fault changed the guest outcome"
    );
    assert_eq!(epoch.faults.load(Ordering::Relaxed), 1);
    assert!(epoch.event_rows().is_empty());
    eprintln!(
        "Synchronous InputRequired verdict: PASS faults=1 valid=false side_rows=0 guest_pauses=2 prompt_order=unchanged guest_result=7"
    );
}

fn dispatch_decisions(symbol: &Symbol) {
    profiling::record_profile_definition_dispatch_candidate("down", symbol);
    profiling::record_profile_definition_dispatch_prefilter_skip("down", symbol);
    profiling::record_profile_definition_dispatch_fast_outcome("down", symbol, true);
    profiling::record_profile_definition_dispatch_fast_outcome("down", symbol, false);
    profiling::record_profile_definition_dispatch_matcher_outcome("down", symbol, true);
    profiling::record_profile_definition_dispatch_matcher_outcome("down", symbol, false);
}

#[test]
fn dispatch_delta_is_applied_once_in_each_field() {
    let _enabled = support::enabled();
    let _flags = Flags::set(1, 1, 1, 1, 1, 1);
    let epoch = support::epoch();
    let source = Symbol::with_identity("Global`", "dispatch_source", 3);
    let _source = crate::runtime::evaluation_profile_definition_source(source);
    for identity in [3, 9] {
        let symbol = Symbol::with_identity("Global`", "dispatch_target", identity);
        dispatch_decisions(&symbol);
        dispatch_decisions(&symbol);
        let mut expected = [0; 4];
        for (index, phase) in [
            "prefilter",
            "fast_match",
            "compiled_matcher",
            "rhs_substitution",
        ]
        .into_iter()
        .enumerate()
        {
            let key = profiling::DefinitionDispatchKey {
                kind: "down",
                symbol: Symbol::new(symbol.context(), symbol.name()),
                source_symbol: Some(Symbol::new(source.context(), source.name())),
            };
            let fields = |value: &profiling::DefinitionDispatchStats| {
                [
                    value.prefilter_nanos,
                    value.fast_match_nanos,
                    value.compiled_matcher_nanos,
                    value.rhs_substitution_nanos,
                ]
            };
            let before = state::DEFINITION_DISPATCH_STATS.with_borrow(|stats| fields(&stats[&key]));
            profiling::record_profile_definition_dispatch_phase_timing(
                "down",
                &symbol,
                phase,
                profiling::definition_dispatch_timing_start(),
            );
            let after = state::DEFINITION_DISPATCH_STATS.with_borrow(|stats| fields(&stats[&key]));
            expected[index] = after[index] - before[index];
        }
        assert_eq!(
            rows(&epoch)[&EventKey::Dispatch {
                kind: "down",
                symbol,
                source: Some(source)
            }],
            ([2, 2, 2, 2, 2, 2, 0], expected, 0)
        );
    }
    assert_eq!(epoch.event_rows().len(), 2);
}

fn cache_entries(session: &Session, symbol: &Symbol) {
    session.record_profile_evaluate_entry();
    session.record_profile_down_value_scan(symbol);
    session.record_profile_down_value_application(symbol);
    session.record_profile_sub_value_application(symbol);
    session.record_profile_builtin_application(symbol);
    session.record_profile_eval_module_entry();
}

#[test]
fn each_family_obeys_its_flag_matrix_and_exact_cache_names() {
    let enabled = support::enabled();
    for logical in [1, 2] {
        for active in [1, 2] {
            for rhs in [1, 2] {
                for timing in [1, 2] {
                    for cache in [1, 2] {
                        for cache_active in [1, 2] {
                            let _flags =
                                Flags::set(logical, active, rhs, timing, cache, cache_active);
                            reset_for_lease(enabled.lease());
                            let epoch = support::epoch();
                            let symbol = Symbol::new("Global`", "cache_source");
                            let rebuild = Symbol::new("Global`", "rebuild_source");
                            let _source =
                                crate::runtime::evaluation_profile_definition_source(symbol);
                            let _rebuild = profiling::expression_rebuild_source_symbol(rebuild);
                            let session = Session::new();
                            let token = profiling::runtime_path_timing_start("matrix.runtime");
                            profiling::record_profile_runtime_path_timing(
                                token,
                                RuntimeProfileOutcome::Some,
                            );
                            profiling::record_profile_rule_rhs_timing(
                                "matrix.rhs",
                                profiling::rule_rhs_timing_start(),
                                Some(true),
                            );
                            dispatch_decisions(&symbol);
                            profiling::record_profile_definition_dispatch_phase_timing(
                                "down",
                                &symbol,
                                "prefilter",
                                profiling::definition_dispatch_timing_start(),
                            );
                            profiling::record_profile_definition_rhs_eval(
                                profiling::lexical_profile_start(),
                            );
                            cache_entries(&session, &symbol);
                            let start =
                                crate::runtime::evaluation_cache_profile_source_timing_start(
                                    "definition_rhs_eval_by_source",
                                );
                            crate::runtime::record_profile_definition_rhs_eval_by_source(start);
                            let start =
                                crate::runtime::evaluation_cache_profile_source_timing_start(
                                    "eval_module_by_source",
                                );
                            crate::runtime::record_profile_eval_module_by_source(start, false);
                            let mut expected = BTreeMap::new();
                            if logical == 1 && active == 1 {
                                let source = (cache == 1 && cache_active == 1).then_some(symbol);
                                expected.insert(
                                    EventKey::Runtime {
                                        phase: "matrix.runtime",
                                        source,
                                    },
                                    [1, 1, 0, 1, 0, 0, 0],
                                );
                                expected.insert(
                                    EventKey::Dispatch {
                                        kind: "down",
                                        symbol,
                                        source,
                                    },
                                    [1, 1, 1, 1, 1, 1, 0],
                                );
                                expected.insert(
                                    EventKey::Counter("definition_rhs_eval"),
                                    [1, 0, 0, 0, 0, 0, 0],
                                );
                                if rhs == 1 {
                                    expected.insert(
                                        EventKey::Rhs {
                                            phase: "matrix.rhs",
                                            source: Some(rebuild),
                                        },
                                        [1, 1, 0, 0, 0, 0, 0],
                                    );
                                }
                                if cache == 1 && cache_active == 1 {
                                    for name in [
                                        "EVALUATE_ENTRY_COUNT",
                                        "DOWN_VALUE_SCAN_COUNT",
                                        "DOWN_VALUE_APPLICATION_COUNT",
                                        "SUB_VALUE_APPLICATION_COUNT",
                                        "BUILTIN_APPLICATION_COUNT",
                                        "EVAL_MODULE_ENTRY_COUNT",
                                    ] {
                                        expected
                                            .insert(EventKey::Cache(name), [1, 0, 0, 0, 0, 0, 0]);
                                    }
                                    for event in [
                                        "down_value_scans",
                                        "down_value_applications",
                                        "sub_value_applications",
                                        "builtin_applications",
                                        "eval_module_entries_by_source",
                                    ] {
                                        expected.insert(
                                            EventKey::CacheSymbol { event, symbol },
                                            [1, 0, 0, 0, 0, 0, 0],
                                        );
                                    }
                                    expected.insert(
                                        EventKey::CacheSource {
                                            event: "definition_rhs_eval_by_source",
                                            symbol,
                                        },
                                        [1, 0, 0, 0, 0, 0, 0],
                                    );
                                }
                            }
                            let actual = epoch
                                .event_rows()
                                .into_iter()
                                .map(|(k, v)| (k, v.vector))
                                .collect::<BTreeMap<_, _>>();
                            assert_eq!(
                                actual, expected,
                                "flags={logical}/{active}/{rhs}/{timing}/{cache}/{cache_active}"
                            );
                            if timing == 2 {
                                for (key, value) in epoch.event_rows() {
                                    if matches!(key, EventKey::Dispatch { .. }) {
                                        assert_eq!(value.durations, [0; 4]);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn registered_threads_add_to_one_epoch_row() {
    let enabled = support::enabled();
    let epoch = support::epoch();
    let key = EventKey::Counter("definition_rhs_eval");
    synchronous::add(key, EventValue::count(2), None);
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                enabled.lease().allow_current_thread();
                assert!(Arc::ptr_eq(&epoch, &current().unwrap()));
                synchronous::add(key, EventValue::count(3), None);
            })
            .join()
            .unwrap();
    });
    assert_eq!(epoch.event_rows().len(), 1);
    assert_eq!(rows(&epoch)[&key].0, [5, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn reset_rejects_retired_epochs_and_old_timed_markers() {
    let enabled = support::enabled();
    let _flags = Flags::set(1, 1, 1, 1, 1, 1);
    let old = support::epoch();
    let key = EventKey::Counter("definition_rhs_eval");
    old.event(key, EventValue::count(7), None);
    let symbol = Symbol::new("Global`", "old");
    let _source = crate::runtime::evaluation_profile_definition_source(symbol);
    let runtime = profiling::runtime_path_timing_start("old.runtime");
    let rhs = profiling::rule_rhs_timing_start();
    let dispatch = profiling::definition_dispatch_timing_start();
    let scalar = profiling::lexical_profile_start();
    let source = crate::runtime::evaluation_cache_profile_source_timing_start(
        "definition_rhs_eval_by_source",
    );
    reset_for_lease(enabled.lease());
    let replacement = support::epoch();
    assert!(!Arc::ptr_eq(&old, &replacement));
    old.event(key, EventValue::count(1), None);
    profiling::record_profile_runtime_path_timing(runtime, RuntimeProfileOutcome::Ok);
    profiling::record_profile_rule_rhs_timing("old.rhs", rhs, Some(false));
    profiling::record_profile_definition_dispatch_phase_timing(
        "down",
        &symbol,
        "prefilter",
        dispatch,
    );
    profiling::record_profile_definition_rhs_eval(scalar);
    crate::runtime::record_profile_definition_rhs_eval_by_source(source);
    assert_eq!(rows(&old)[&key].0, [7, 0, 0, 0, 0, 0, 0]);
    assert!(
        replacement.event_rows().is_empty(),
        "old completion charged replacement"
    );
    assert!(replacement.is_valid());
}

#[test]
fn equal_clock_invalidates_once_without_a_row() {
    let _enabled = support::enabled();
    let epoch = support::epoch();
    let birth = epoch.data.lock().unwrap().event_birth;
    for _ in 0..2 {
        epoch.event(
            EventKey::Counter("definition_rhs_eval"),
            EventValue::count(1),
            Some(birth),
        );
    }
    assert!(!epoch.is_valid());
    assert_eq!(epoch.faults.load(Ordering::Relaxed), 1);
    assert!(epoch.event_rows().is_empty());
}

#[test]
fn cap_retirement_is_once_and_guest_continuation_still_finishes() {
    let enabled = support::enabled();
    let _flags = Flags::set(1, 1, 2, 2, 2, 2);
    support::limit(enabled.lease(), 64 * 1024);
    let epoch = support::epoch();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    let first = session
        .eval_request(r#"Function[{},(InputString["first"];InputString["second"];7)][]"#)
        .unwrap_err();
    assert!(
        matches!(first.error, SessionRequestError::InputRequired { request } if request.prompt == "first")
    );
    assert!(epoch.is_valid());
    assert!(epoch.rows().iter().any(|(_, c)| c.open > 0));
    for identity in 1..=4096 {
        epoch.event(
            EventKey::CacheSymbol {
                event: "down_value_scans",
                symbol: Symbol::with_identity("Global`", "cap", identity),
            },
            EventValue::count(1),
            None,
        );
        if !epoch.is_valid() {
            break;
        }
    }
    assert!(
        !epoch.is_valid(),
        "side index must exhaust the shared budget"
    );
    let terminal = epoch
        .rows()
        .into_iter()
        .map(|(key, c)| {
            assert_eq!(c.open, 0);
            (key, (c.calls, c.abandoned, c.abandonment_reasons))
        })
        .collect::<BTreeMap<_, _>>();
    assert!(
        terminal
            .values()
            .any(|(_, _, reasons)| reasons[Retire::DiagnosticCap.index()] > 0)
    );
    let events = rows(&epoch);
    epoch.retire(Retire::DiagnosticCap);
    epoch.event(EventKey::Counter("after-cap"), EventValue::count(1), None);
    assert_eq!(events, rows(&epoch));
    let second = session
        .resume_eval_request_with_input("answer-one")
        .unwrap_err();
    assert!(
        matches!(second.error, SessionRequestError::InputRequired { request } if request.prompt == "second")
    );
    assert_eq!(
        session
            .resume_eval_request_with_input("answer-two")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    let after = epoch
        .rows()
        .into_iter()
        .map(|(key, c)| (key, (c.calls, c.abandoned, c.abandonment_reasons)))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(after, terminal);
    assert_eq!(epoch.faults.load(Ordering::Relaxed), 0);
}

#[test]
fn unregistered_thread_cannot_write_or_observe_side_rows() {
    let _enabled = support::enabled();
    let epoch = support::epoch();
    let key = EventKey::Counter("definition_rhs_eval");
    epoch.event(key, EventValue::count(1), None);
    let other = Arc::clone(&epoch);
    std::thread::spawn(move || {
        other.event(key, EventValue::count(9), None);
        synchronous::add(key, EventValue::count(9), None);
        assert!(other.event_rows().is_empty());
        assert!(current().is_none());
    })
    .join()
    .unwrap();
    assert_eq!(rows(&epoch)[&key].0, [1, 0, 0, 0, 0, 0, 0]);
    assert!(epoch.is_valid());
}

#[test]
fn session_drop_retains_completed_side_rows() {
    let _enabled = support::enabled();
    let epoch = support::epoch();
    let session = Session::new();
    let call = Call::start(
        &session,
        AggregateKey::Runtime {
            phase: "drop.native",
            source: None,
        },
        Policy::Expr,
    )
    .unwrap();
    epoch.event(
        EventKey::Counter("definition_rhs_eval"),
        EventValue::count(3),
        None,
    );
    let before = rows(&epoch);
    drop(session);
    drop(call);
    assert_eq!(rows(&epoch), before);
    assert!(epoch.is_valid());
    let native = epoch.rows();
    assert_eq!(native[0].1.open, 0);
    assert_eq!(
        native[0].1.abandonment_reasons[Retire::SessionDrop.index()],
        1
    );
}

#[test]
fn old_module_alias_completion_does_not_charge_replacement() {
    let _enabled = support::enabled();
    let _flags = Flags::set(1, 1, 1, 1, 1, 1);
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    session
        .eval_text(r#"f[]:=Module[{x=7},(InputString["alias"];x)]"#)
        .unwrap();
    profiling::reset_lexical_profile_counters();
    crate::runtime::reset_evaluation_cache_profile_counters();
    let first = session.eval_request("f[]").unwrap_err();
    assert!(
        matches!(first.error, SessionRequestError::InputRequired { request } if request.prompt == "alias")
    );
    let old = support::epoch();
    profiling::reset_lexical_profile_counters();
    crate::runtime::reset_evaluation_cache_profile_counters();
    let frozen = rows(&old);
    let replacement = support::epoch();
    assert_eq!(
        session
            .resume_eval_request_with_input("answer")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(rows(&old), frozen);
    assert_eq!(state::EVAL_MODULE_COUNT.load(Ordering::Relaxed), 0);
    assert!(replacement.rows().iter().all(|(k, _)| !matches!(
        k,
        AggregateKey::Runtime {
            phase: "module.eval.total",
            ..
        }
    )));
    for (key, _) in replacement.event_rows() {
        assert!(!matches!(
            key,
            EventKey::Counter("eval_module")
                | EventKey::CacheSource {
                    event: "eval_module_by_source",
                    ..
                }
        ));
    }
}

#[test]
fn source_event_precedes_legacy_mutex_lock() {
    let enabled = support::enabled();
    let _flags = Flags::set(1, 1, 1, 1, 1, 1);
    let epoch = support::epoch();
    let symbol = Symbol::new("Global`", "lock_order");
    let key = EventKey::CacheSource {
        event: "definition_rhs_eval_by_source",
        symbol,
    };
    let (ready, started) = std::sync::mpsc::channel();
    let mut visible_while_locked = false;
    std::thread::scope(|scope| {
        let mut worker = None;
        Session::test_with_source_timing_lock(|| {
            worker = Some(scope.spawn(|| {
                enabled.lease().allow_current_thread();
                let _source = crate::runtime::evaluation_profile_definition_source(symbol);
                let start = crate::runtime::evaluation_cache_profile_source_timing_start(
                    "definition_rhs_eval_by_source",
                );
                ready.send(()).unwrap();
                crate::runtime::record_profile_definition_rhs_eval_by_source(start);
            }));
            started.recv_timeout(Duration::from_secs(5)).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < deadline {
                if rows(&epoch).contains_key(&key) {
                    visible_while_locked = true;
                    break;
                }
                std::thread::yield_now();
            }
        });
        // Release the legacy lock before joining, including in the mutant.
        worker.unwrap().join().unwrap();
    });
    assert!(
        visible_while_locked,
        "epoch update was blocked by the legacy source mutex"
    );
    assert_eq!(rows(&epoch)[&key].0, [1, 0, 0, 0, 0, 0, 0]);
}
