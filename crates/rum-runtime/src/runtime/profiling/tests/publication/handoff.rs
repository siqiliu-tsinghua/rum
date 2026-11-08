use super::{decode::Key, support};
use crate::runtime::{
    SessionRequestError,
    profiling::{
        logical::{epoch, publication},
        tests::engine_support,
    },
};
use std::collections::BTreeMap;

#[test]
fn five_modes_handoff_and_own_open() {
    let enabled = engine_support::enabled();
    for mode in ["disabled", "lexical", "timing", "cache", "all"] {
        let _flags = support::Flags::mode(mode);
        for minimal in [false, true] {
            let mut session = support::session();
            session
                .eval_text(if minimal {
                    "f[] := Module[{x=3}, (InputString[\"a\"]; x)]"
                } else {
                    "witness[x_] := Module[{v=x}, (InputString[\"a\"]; InputString[\"b\"]; v+1)]"
                })
                .unwrap();
            support::reset_epoch(enabled.lease());
            let first = session.eval_request(if minimal { "f[]" } else { "witness[6]" });
            assert!(matches!(
                first.unwrap_err().error,
                SessionRequestError::InputRequired { .. }
            ));
            let _paused = support::observe("paused-A");
            let (mut session, on_b) = std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        enabled.lease().allow_current_thread();
                        if !minimal {
                            assert!(matches!(
                                session
                                    .resume_eval_request_with_input("first")
                                    .unwrap_err()
                                    .error,
                                SessionRequestError::InputRequired { .. }
                            ));
                            support::observe("paused-B");
                        }
                        assert_eq!(
                            session
                                .resume_eval_request_with_input("second")
                                .unwrap()
                                .value
                                .to_string(),
                            if minimal { "3" } else { "7" }
                        );
                        let on_b = support::observe("terminal");
                        (session, on_b)
                    })
                    .join()
                    .unwrap()
            });
            let on_a = support::observe("terminal");
            assert_eq!(on_b.keys, on_a.keys);
            assert_eq!(on_b.rows, on_a.rows);
            assert_eq!(on_a.mode, support::mode_token(mode));
            let mut counts = BTreeMap::new();
            for key in on_a.rows.keys() {
                *counts.entry(key.family()).or_insert(0) += 1;
            }
            if mode == "disabled" {
                assert!(counts.is_empty());
                assert!(epoch::existing().is_none());
            } else {
                for family in [
                    "runtime",
                    "definition-rule",
                    "definition-dispatch",
                    "counter",
                    "cache",
                ] {
                    assert!(
                        counts.get(family).is_some_and(|n| *n > 0),
                        "{mode} {family}"
                    );
                }
                assert_eq!(
                    counts.contains_key("builtin"),
                    matches!(mode, "timing" | "all")
                );
                assert_eq!(
                    counts.contains_key("rule-rhs"),
                    matches!(mode, "timing" | "all")
                );
                assert_eq!(
                    counts.contains_key("cache-symbol"),
                    matches!(mode, "cache" | "all")
                );
                assert_eq!(on_a.rows.keys().any(|key| matches!(key, Key::CacheSource(event, _) if event == "eval_module_by_source")), matches!(mode,"cache"|"all"));
                if matches!(mode, "cache" | "all") {
                    assert!(on_a.rows.keys().any(|key| matches!(key, Key::CacheSource(event, _) if event == "definition_rhs_eval_by_source")));
                }
            }
            println!("SCHEMA_MODE mode={mode} minimal={minimal} family_keys={counts:?}");
            assert_eq!(
                session
                    .eval_text("RumProfileSnapshot[\"mode\"]")
                    .unwrap()
                    .value
                    .to_string(),
                "Null"
            );
            if minimal {
                println!("SCHEMA_EXAMPLE mode={mode} {}", publication::render(mode));
            }
            println!(
                "Schema handoff verdict: PASS mode={mode} minimal={minimal} terminal_thread_parity=true parsed_native_parity=true own_open_gate_exact=true result={}",
                if minimal { 3 } else { 7 }
            );
        }
    }
}
