use super::super::*;
use super::common::*;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn host_session_and_cancel_handle_are_send() {
    fn is_send<T: Send>() {}

    is_send::<rum_runtime::Session>();
    is_send::<HostSession>();
    is_send::<HostCancelHandle>();
    // HostSession is intentionally not asserted Sync: it is owned, mutable
    // session state, with Cell/FnMut internals that should not be shared.
}

#[test]
fn parallel_host_sessions_move_to_threads_and_isolate_definitions() {
    const THREADS: usize = 8;

    let handles = (0..THREADS)
        .map(|i| {
            let session = HostSession::new();
            thread::spawn(move || {
                let mut session = session;
                let source = format!("x = {i}; x * x");
                match session.eval(source) {
                    HostEvalOutcome::Success(success) => (i, success.value.text),
                    other => panic!("thread {i} expected success, got {other:?}"),
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        let (i, text) = handle.join().expect("host session worker should not panic");
        assert_eq!(text, (i * i).to_string(), "thread {i}");
        eprintln!("parallel_host_sessions thread={i} result={text}");
    }
}

#[test]
fn host_reset_removes_print_sink_and_restores_buffering() {
    let mut session = HostSession::new();
    let streamed = Arc::new(Mutex::new(Vec::new()));
    let sink_streamed = Arc::clone(&streamed);

    session.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
        sink_streamed.lock().unwrap().push(line.to_string());
    }));
    session.reset();

    let outcome = successful_eval(session.eval("Print[7]; 8"));
    assert!(streamed.lock().unwrap().is_empty());
    assert_eq!(outcome.prints, vec!["7"]);
}

#[test]
fn host_session_introspection_enumerates_user_definitions_and_contexts() {
    let mut session = HostSession::new();

    let fresh_definitions = session.definitions();
    assert!(
        fresh_definitions
            .iter()
            .all(|summary| summary.name != "System`Plus"),
        "native builtin Plus should not appear in definition summaries"
    );

    let fresh_contexts = session.contexts();
    assert_eq!(
        fresh_contexts,
        vec!["Global`".to_string(), "System`".to_string()]
    );

    successful_eval(session.eval("a = 1; f[x_] := x; f[2] := 9"));

    let definitions = session.definitions();
    assert_eq!(
        definitions,
        vec![
            HostDefinitionSummary {
                name: "Global`a".to_string(),
                has_own_value: true,
                down_value_count: 0,
                up_value_count: 0,
                sub_value_count: 0,
            },
            HostDefinitionSummary {
                name: "Global`f".to_string(),
                has_own_value: false,
                down_value_count: 2,
                up_value_count: 0,
                sub_value_count: 0,
            },
        ]
    );

    let contexts = session.contexts();
    assert_eq!(contexts, vec!["Global`".to_string(), "System`".to_string()]);
    eprintln!("introspection definitions={definitions:?} contexts={contexts:?}");
}

#[test]
fn host_session_introspection_survives_poisoned_session() {
    let mut session = HostSession::new();

    successful_eval(session.eval("p = 7"));
    match session.eval_with_test_panic() {
        HostEvalOutcome::Failure(failure) => assert_eq!(failure.code, HostFailureCode::Panic),
        other => panic!("expected panic failure, got {other:?}"),
    }
    assert!(session.poisoned_for_test());

    let definitions = session.definitions();
    assert!(
        definitions
            .iter()
            .any(|summary| summary.name == "Global`p" && summary.has_own_value)
    );

    let contexts = session.contexts();
    assert!(contexts.iter().any(|context| context == "Global`"));
    assert!(contexts.iter().any(|context| context == "System`"));
    eprintln!("poisoned_introspection definitions={definitions:?} contexts={contexts:?}");
}

#[test]
fn host_boundary_catches_panic_and_poisons_session() {
    let mut session = HostSession::new();

    let caught = session.eval_with_test_panic();

    let (caught_code, caught_message) = match caught {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Panic);
            assert!(failure.message.contains("host panic boundary test"));
            (failure.code, failure.message)
        }
        other => panic!("expected panic failure, got {other:?}"),
    };
    assert!(session.poisoned_for_test());

    let followup = session.eval("1 + 1");
    match followup {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Poisoned);
            assert!(failure.message.contains("poisoned"));
            eprintln!(
                "host_boundary_catches_panic_and_poisons_session caught_code={caught_code:?} caught_message={caught_message:?} followup_code={:?} followup_message={:?}",
                failure.code, failure.message
            );
        }
        other => panic!("expected poisoned failure, got {other:?}"),
    }
}

#[test]
fn host_boundary_normal_eval_does_not_poison_session() {
    let mut session = HostSession::new();

    match session.eval("1 + 2") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "3"),
        other => panic!("expected normal success, got {other:?}"),
    }
    assert!(!session.poisoned_for_test());

    match session.eval("3 + 4") {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "7");
            eprintln!(
                "host_boundary_normal_eval_does_not_poison_session first=3 second=7 poisoned={}",
                session.poisoned_for_test()
            );
        }
        other => panic!("expected follow-up success, got {other:?}"),
    }
}

#[test]
fn host_session_reset_clears_user_state() {
    let mut session = HostSession::new();

    let assigned = successful_eval(session.eval("x = 41; x"));
    assert_eq!(assigned.value.text, "41");

    session.reset();

    let reset_symbol = successful_eval(session.eval("x"));
    assert_eq!(reset_symbol.value.text, "x");
    assert_eq!(reset_symbol.value.typed, HostTyped::Symbol("x".to_string()));

    let arithmetic = successful_eval(session.eval("2 + 2"));
    assert_eq!(arithmetic.value.text, "4");

    eprintln!(
        "host_session_reset_clears_user_state assigned=41 after_reset={:?} arithmetic=4 poisoned={}",
        reset_symbol.value.text,
        session.poisoned_for_test()
    );
}

#[test]
fn host_session_reset_recovers_poisoned_session() {
    let mut session = HostSession::new();

    let caught = session.eval_with_test_panic();
    match caught {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Panic);
            assert!(failure.message.contains("host panic boundary test"));
        }
        other => panic!("expected panic failure, got {other:?}"),
    }
    assert!(session.poisoned_for_test());

    let poisoned = session.eval("1 + 1");
    match poisoned {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Poisoned);
            assert!(failure.message.contains("poisoned"));
        }
        other => panic!("expected poisoned failure, got {other:?}"),
    }

    session.reset();
    assert!(!session.poisoned_for_test());

    let recovered = successful_eval(session.eval("1 + 1"));
    assert_eq!(recovered.value.text, "2");

    eprintln!(
        "host_session_reset_recovers_poisoned_session poisoned_before_reset=true recovered_value=2 poisoned_after_reset={}",
        session.poisoned_for_test()
    );
}

#[test]
fn host_session_reset_restores_original_config_not_mutated_state() {
    let mut limits = HostResourceLimits::default();
    limits.recursion_limit = 77;
    limits.eval_memory_limit = Some(1_000_000);
    let original_config = HostConfig::default()
        .with_resource_limits(limits)
        .with_current_context("Original")
        .with_context_path(["System", "Original"]);
    let mut session = HostSession::builder().with_config(original_config).build();

    assert_eq!(session.configuration().current_context, "Original`");
    assert_eq!(
        session.configuration().resource_limits.eval_memory_limit,
        Some(1_000_000)
    );
    assert_eq!(
        session.configuration().context_path,
        vec!["System`".to_string(), "Original`".to_string()]
    );

    let changed = successful_eval(
        session.eval("Begin[\"Changed`\"]; $ContextPath = {\"Changed`\"}; $Context"),
    );
    assert_eq!(changed.value.text, "\"Changed`\"");
    assert_eq!(session.configuration().current_context, "Changed`");
    assert_eq!(
        session.configuration().context_path,
        vec!["Changed`".to_string()]
    );

    session.reset();

    let reset_config = session.configuration();
    assert_eq!(reset_config.current_context, "Original`");
    assert_eq!(
        reset_config.context_path,
        vec!["System`".to_string(), "Original`".to_string()]
    );
    assert_eq!(reset_config.resource_limits.recursion_limit, 77);
    assert_eq!(
        reset_config.resource_limits.eval_memory_limit,
        Some(1_000_000)
    );

    let reset_context = successful_eval(session.eval("$Context"));
    assert_eq!(reset_context.value.text, "\"Original`\"");

    eprintln!(
        "host_session_reset_restores_original_config_not_mutated_state current_context={:?} context_path={:?} recursion_limit={}",
        reset_config.current_context,
        reset_config.context_path,
        reset_config.resource_limits.recursion_limit
    );
}

#[test]
fn host_session_snapshot_bytes_roundtrip_and_restore_replaces_user_state() {
    let mut a = HostSession::new();
    successful_eval(
        a.eval("a = 1; f[x_] := x; g[2] := 20; h[x_] := x; SetAttributes[h, Orderless]"),
    );
    let bytes = a.snapshot_to_bytes().expect("snapshot should render");

    let mut b = HostSession::new();
    successful_eval(b.eval("z = 99"));
    b.restore_from_bytes(&bytes)
        .expect("snapshot bytes should restore");

    assert_eq!(b.definitions(), a.definitions());
    assert_eq!(successful_eval(b.eval("f[7]")).value.text, "7");
    assert_eq!(
        successful_eval(b.eval("Attributes[h]")).value.text,
        "List[Orderless]"
    );
    assert_eq!(successful_eval(b.eval("z")).value.text, "z");

    eprintln!(
        "host_session_snapshot_bytes_roundtrip_and_restore_replaces_user_state definitions={:?} bytes={} f7=7 attributes_h=List[Orderless] z_after_restore=z",
        b.definitions(),
        bytes.len()
    );
}

#[test]
fn host_session_snapshot_roundtrip_relinks_sealed_values() {
    let package = "DefinePackage[\"ValuePkg`\", Public[{mk, accepts}], SetAttributes[tag, Protected]; mk[x_] := tag[x]; accepts[tag[_]] := True; accepts[_] := False]";
    let mut source = HostSession::new();
    assert_eq!(
        successful_eval(source.eval(package)).value.text,
        "\"ValuePkg`\""
    );
    assert_eq!(
        successful_eval(source.eval("thm = ValuePkg`mk[7]"))
            .value
            .text,
        "ValuePkg`Private`tag[7]"
    );
    let bytes = source
        .snapshot_to_bytes()
        .expect("sealed snapshot should render");

    let mut restored = HostSession::new();
    restored
        .restore_from_bytes(&bytes)
        .expect("sealed snapshot should restore");
    assert_eq!(
        successful_eval(restored.eval("ValuePkg`accepts[thm]"))
            .value
            .text,
        "True"
    );
    assert_eq!(
        successful_eval(restored.eval("ValuePkg`Private`tag = 99"))
            .value
            .text,
        "99"
    );
    assert_eq!(
        successful_eval(restored.eval("ValuePkg`accepts[thm]"))
            .value
            .text,
        "True"
    );
}

#[test]
fn host_session_snapshot_roundtrip_rederives_lazy_list_identically() {
    let mut source = HostSession::new();
    assert_eq!(
        successful_eval(source.eval("lz = LazyRange[3, 2]"))
            .value
            .text,
        "LazyList[LazyRange[3, 2]]"
    );
    let bytes = source.snapshot_to_bytes().expect("snapshot should render");

    let mut restored = HostSession::new();
    restored
        .restore_from_bytes(&bytes)
        .expect("lazy-list snapshot bytes should restore");

    assert_eq!(
        successful_eval(restored.eval("lz")).value.text,
        "LazyList[LazyRange[3, 2]]"
    );
    assert_eq!(
        successful_eval(restored.eval("Take[lz, 3]")).value.text,
        "List[3, 5, 7]"
    );
}
