use super::{
    decode::{self, Key},
    native, support,
};
use crate::runtime::profiling::{
    logical::{
        AggregateKey, Call, Outcome, Policy, epoch, publication,
        synchronous::{EventKey, EventValue},
    },
    tests::engine_support,
};
use crate::runtime::{Session, Symbol};

#[test]
fn union_and_equal_keys() {
    let _enabled = engine_support::enabled();
    let _flags = support::Flags::mode("all");
    let e = engine_support::epoch();
    let session = Session::new();
    let symbol = Symbol::with_identity("ctx", "name", 3);
    for key in [
        AggregateKey::Runtime {
            phase: "compound.item.suspend",
            source: None,
        },
        AggregateKey::Builtin {
            symbol: Symbol::system("RumProfileSnapshot"),
            source: None,
            arity: 1,
        },
        AggregateKey::Definition {
            kind: "down_value_rhs_eval",
            symbol: Symbol::from_name("witness"),
            index: 0,
        },
        AggregateKey::Runtime {
            phase: "module.eval.total",
            source: None,
        },
        AggregateKey::Runtime {
            phase: "module.eval.total",
            source: Some(symbol),
        },
    ] {
        for outcome in [Outcome::Ok, Outcome::Throw] {
            Call::start(&session, key, Policy::Expr)
                .unwrap()
                .finish(outcome);
        }
    }
    let events = [
        EventKey::Runtime {
            phase: "runtime.sync",
            source: Some(symbol),
        },
        EventKey::Rhs {
            phase: "rhs.instantiate",
            source: Some(symbol),
        },
        EventKey::Dispatch {
            kind: "down_value",
            symbol: Symbol::from_name("witness"),
            source: None,
        },
        EventKey::Counter("definition_rhs_eval"),
        EventKey::Cache("EVALUATE_ENTRY_COUNT"),
        EventKey::CacheSymbol {
            event: "builtin_applications",
            symbol: Symbol::system("RumProfileSnapshot"),
        },
        EventKey::CacheSource {
            event: "definition_rhs_eval_by_source",
            symbol,
        },
    ];
    for key in events {
        for _ in 0..2 {
            e.event(
                key,
                EventValue {
                    vector: [1, 2, 3, 4, 5, 6, 7],
                    durations: [8, 9, 10, 11],
                    input_required: 12,
                },
                None,
            );
        }
    }
    let snapshot = support::observe("union");
    assert_eq!(snapshot.rows.len(), 5 + 7 + 2 + 5);
    assert_eq!(
        snapshot.rows[&Key::Counter("definition_rhs_eval".into())].vector,
        [2]
    );
    assert_eq!(
        snapshot.rows[&Key::Counter("eval_module".into())].vector,
        [4]
    );
    assert_eq!(
        snapshot.rows[&Key::Counter("eval_module".into())].accounting,
        [0, 0, 0]
    );
    for key in [
        "runtime|compound.item.suspend|-",
        "builtin|s(System`,RumProfileSnapshot,n)|-|1",
        "definition-rule|down_value_rhs_eval|s(Global`,witness,n)|0",
        "definition-dispatch|down_value|s(Global`,witness,n)|-",
        "cache-symbol|builtin_applications|s(System`,RumProfileSnapshot,n)",
        "cache-source|definition_rhs_eval_by_source|s(ctx,name,3)",
    ] {
        assert!(
            snapshot.keys.iter().any(|actual| actual == key),
            "missing pinned spelling {key}"
        );
    }
    println!(
        "Schema union verdict: PASS native=5 side=7 alias_source=1 alias_counter=1 defaults_unreplaced=5 equal_keys_summed=true parsed_parity=true"
    );
}

#[test]
fn no_truncation() {
    let _enabled = engine_support::enabled();
    let _flags = support::Flags::mode("lexical");
    let session = Session::new();
    for i in 0..96 {
        Call::start(
            &session,
            AggregateKey::Definition {
                kind: "down_value_rhs_eval",
                symbol: Symbol::from_name(format!("many{i}")),
                index: 0,
            },
            Policy::Expr,
        )
        .unwrap()
        .finish(Outcome::Ok);
    }
    let snapshot = support::observe("96 definitions");
    assert_eq!(
        snapshot
            .rows
            .keys()
            .filter(|key| matches!(key, Key::Definition(..)))
            .count(),
        96
    );
    assert_eq!(snapshot.rows.len(), 104);
    println!(
        "Schema no-truncation verdict: PASS native_definitions=96 published_definitions=96 total=104 duplicates=0"
    );
}

#[test]
fn distinct_identities_and_adversarial_components() {
    let _enabled = engine_support::enabled();
    let _flags = support::Flags::mode("lexical");
    let e = engine_support::epoch();
    let symbols = [
        Symbol::with_identity("Global`", "same", 3),
        Symbol::with_identity("Global`", "same", 9),
        Symbol::new("System`", "same"),
        Symbol::new("System", "same"),
        Symbol::new("X`", "a"),
        Symbol::new("X", "`a"),
        Symbol::new("", "-"),
        Symbol::new("%|,(#)\n\r\t\\\"", "名%|,(#)\n\r\t\\\""),
    ];
    assert_eq!(
        symbols[4].fully_qualified_name(),
        symbols[5].fully_qualified_name()
    );
    for s in symbols {
        e.event(
            EventKey::Runtime {
                phase: "runtime.sync",
                source: Some(s),
            },
            EventValue::count(1),
            None,
        );
    }
    e.event(
        EventKey::Runtime {
            phase: "runtime.sync",
            source: None,
        },
        EventValue::count(1),
        None,
    );
    let snapshot = support::observe("\"\\\n\r\tλ");
    assert_eq!(snapshot.rows.len(), 17);
    assert!(
        snapshot
            .keys
            .contains(&"runtime|runtime.sync|s(Global`,same,3)".into())
    );
    assert!(
        snapshot
            .keys
            .contains(&"runtime|runtime.sync|s(Global`,same,9)".into())
    );
    assert!(snapshot.keys.contains(&"runtime|runtime.sync|-".into()));
    assert!(
        snapshot
            .keys
            .contains(&"runtime|runtime.sync|s(,-,n)".into())
    );
    assert!(
        snapshot
            .keys
            .iter()
            .any(|text| text.contains("%25%7C%2C%28%23%29"))
    );
    println!(
        "Schema identities verdict: PASS identities=3,9 distinct_contexts_preserved=true adversarial_symbols=8 none_source_distinct=true line_parses=true"
    );
}

#[test]
fn mid_interval_durations() {
    let _enabled = engine_support::enabled();
    let _flags = support::Flags::mode("lexical");
    let e = engine_support::epoch();
    let session = Session::new();
    let call = Call::start(
        &session,
        AggregateKey::Runtime {
            phase: "compound.item.suspend",
            source: None,
        },
        Policy::Expr,
    )
    .unwrap();
    let key = Key::Runtime("compound.item.suspend".into(), None);
    let before = native::observe(&e);
    let snapshot = decode::parse(&publication::render("running"));
    let after = native::observe(&e);
    native::parity(&snapshot, &before, &after);
    assert!(before[&key].durations[0] > 0);
    assert_eq!(snapshot.rows[&key].accounting, [1, 0, 0]);
    println!(
        "Schema running duration reads: before={:?} guest={:?} after={:?}",
        before[&key].durations, snapshot.rows[&key].durations, after[&key].durations
    );
    call.finish(Outcome::Ok);
    let closed = support::observe("closed");
    assert_eq!(closed.rows, native::observe(&e));
    println!("Schema durations verdict: PASS running_between_native_reads=true closed_exact=true");
}

#[test]
fn determinism() {
    if let Ok(reverse) = std::env::var("RUM_SCHEMA_TEST_REVERSE") {
        let _enabled = engine_support::enabled();
        let _flags = support::Flags::mode("lexical");
        let mut names = vec!["z", "a", "m", "b"];
        if reverse == "1" {
            names.reverse();
        }
        for name in names {
            let _ = Symbol::from_name(name);
        }
        let e = engine_support::epoch();
        for name in ["z", "a", "m", "b"] {
            e.event(
                EventKey::CacheSymbol {
                    event: "builtin_applications",
                    symbol: Symbol::from_name(name),
                },
                EventValue::count(1),
                None,
            );
        }
        let parsed = support::observe("deterministic");
        for key in parsed.keys {
            println!("ORDERED_KEY {key}");
        }
        return;
    }
    let mut orders = Vec::new();
    for reverse in ["0", "1"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "runtime::profiling::tests::publication::model::determinism",
                "--exact",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("RUM_SCHEMA_TEST_REVERSE", reverse)
            .output()
            .unwrap();
        println!(
            "DETERMINISM_CHILD reverse={reverse} stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        orders.push(
            String::from_utf8(output.stdout)
                .unwrap()
                .lines()
                // libtest can prefix the first line with its test-name banner.
                .filter_map(|line| line.split_once("ORDERED_KEY ").map(|(_, key)| key))
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(orders[0].len(), 12);
    assert_eq!(orders[0], orders[1]);
    println!(
        "Schema determinism verdict: PASS processes=2 reverse_interner_insertion=true ordered_keys_identical=true"
    );
}

#[test]
fn absent_deferred_and_unregistered() {
    let enabled = engine_support::enabled();
    let _flags = support::Flags::mode("deferred");
    let snapshot = decode::parse(&publication::render(""));
    assert_eq!(snapshot.validity, "absent");
    assert_eq!(snapshot.mode, "none");
    assert!(snapshot.rows.is_empty());
    assert!(epoch::existing().is_none());
    let e = engine_support::epoch();
    e.event(
        EventKey::Counter("definition_rhs_eval"),
        EventValue::count(1),
        None,
    );
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                assert_eq!(epoch::publication_state().1, None);
                assert!(epoch::publication_state().0.is_none());
                assert!(e.events().is_empty());
                assert_eq!(
                    decode::parse(&publication::render("unregistered")).validity,
                    "absent"
                );
                enabled.lease().allow_current_thread();
                assert_eq!(e.events().len(), 1);
            })
            .join()
            .unwrap()
    });
    println!(
        "Schema absent verdict: PASS epoch_not_created=true deferred_mode=none unregistered_refused=true"
    );
}
