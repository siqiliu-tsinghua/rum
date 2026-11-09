//! Q29: state from an enabled generation must clear in a disabled generation.
use super::{Flags, profiling, support};

#[test]
fn brief713_disabled_reset_clears_prior_generation_state() {
    const CACHE_ROW: &str = "cache-source|eval_module_by_source|f";
    const LEXICAL_COUNTER: &str = "counter|eval_module";
    {
        let _lease = support::lease();
        let _flags = Flags::set(1, 1, 1, 1, 1, 1);
        profiling::reset_lexical_profile_counters();
        crate::runtime::reset_evaluation_cache_profile_counters();
        let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
        assert_eq!(
            session
                .eval_text("f[]:=Module[{x=3},x];f[]")
                .unwrap()
                .value
                .to_string(),
            "3"
        );
        let seeded = profiling::logical_profile_snapshot();
        eprintln!(
            "B713 RESET_CLASS seeded {CACHE_ROW}={:?} {LEXICAL_COUNTER}={:?}",
            seeded.logical.get(CACHE_ROW),
            seeded.logical.get(LEXICAL_COUNTER)
        );
        assert_eq!(seeded.logical[CACHE_ROW], vec![1]);
        assert_eq!(seeded.logical[LEXICAL_COUNTER], vec![1]);
    }

    // A raw next lease performs the fenced epoch handoff, but owns neither
    // legacy counter reset. Both fixture flag swaps use publish().
    let _lease = support::lease();
    let _flags = Flags::set(2, 2, 2, 2, 2, 2);
    profiling::reset_lexical_profile_counters();
    crate::runtime::reset_evaluation_cache_profile_counters();
    let after = profiling::logical_profile_snapshot();
    let cache_cleared = !after.logical.contains_key(CACHE_ROW);
    let lexical_cleared = after.logical[LEXICAL_COUNTER] == vec![0];
    eprintln!(
        "B713 RESET_CLASS disabled {CACHE_ROW}={:?} {LEXICAL_COUNTER}={:?} cache_cleared={cache_cleared} lexical_cleared={lexical_cleared}",
        after.logical.get(CACHE_ROW),
        after.logical.get(LEXICAL_COUNTER)
    );
    assert_eq!(
        (cache_cleared, lexical_cleared),
        (true, true),
        "disabled reset retained {CACHE_ROW}={:?} {LEXICAL_COUNTER}={:?}",
        after.logical.get(CACHE_ROW),
        after.logical.get(LEXICAL_COUNTER)
    );
    for flag in [
        "LEXICAL_PROFILE_ENABLED",
        "LEXICAL_PROFILE_ACTIVE",
        "EVALUATION_CACHE_PROFILE_ENABLED",
        "EVALUATION_CACHE_PROFILE_ACTIVE",
    ] {
        assert_eq!(after.flags[flag], 2, "reset must not activate {flag}");
    }
}
