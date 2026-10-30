//! Cold-load initializers publish only over the uninitialized 0
//! (brief-635 amendment 1). A value another writer stored after this
//! thread's load (a lease holder's swap, `RumProfileReset[]`) is kept.

use super::*;
use crate::runtime::profiling::logical::test_support;

#[test]
fn a_cold_load_keeps_a_flag_published_after_its_load() {
    let _enabled = test_support::enabled();
    let flags: [(&str, &AtomicU8, fn() -> bool); 12] = [
        (
            "LEXICAL_PROFILE_ENABLED",
            &LEXICAL_PROFILE_ENABLED,
            initialize_lexical_profile_enabled,
        ),
        (
            "LEXICAL_PROFILE_ACTIVE",
            &LEXICAL_PROFILE_ACTIVE,
            initialize_lexical_profile_active,
        ),
        (
            "LEXICAL_PROFILE_DETAIL_ENABLED",
            &LEXICAL_PROFILE_DETAIL_ENABLED,
            initialize_lexical_profile_detail_enabled,
        ),
        (
            "LEXICAL_PROFILE_INCREMENTAL_ENABLED",
            &LEXICAL_PROFILE_INCREMENTAL_ENABLED,
            initialize_lexical_profile_incremental_enabled,
        ),
        (
            "LEXICAL_PROFILE_SHAPES_ENABLED",
            &LEXICAL_PROFILE_SHAPES_ENABLED,
            initialize_lexical_profile_shapes_enabled,
        ),
        (
            "LEXICAL_PROFILE_BINDINGS_ENABLED",
            &LEXICAL_PROFILE_BINDINGS_ENABLED,
            initialize_lexical_profile_bindings_enabled,
        ),
        (
            "LEXICAL_PROFILE_SOURCES_ENABLED",
            &LEXICAL_PROFILE_SOURCES_ENABLED,
            initialize_lexical_profile_sources_enabled,
        ),
        (
            "LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED",
            &LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED,
            initialize_lexical_profile_source_fingerprints_enabled,
        ),
        (
            "LEXICAL_PROFILE_PROVENANCE_ENABLED",
            &LEXICAL_PROFILE_PROVENANCE_ENABLED,
            initialize_lexical_profile_provenance_enabled,
        ),
        (
            "RULE_RHS_TIMING_ENABLED",
            &RULE_RHS_TIMING_ENABLED,
            initialize_rule_rhs_timing_enabled,
        ),
        (
            "DEFINITION_RULE_TIMING_ENABLED",
            &DEFINITION_RULE_TIMING_ENABLED,
            initialize_definition_rule_timing_enabled,
        ),
        (
            "BUILTIN_TIMING_ENABLED",
            &BUILTIN_TIMING_ENABLED,
            initialize_builtin_timing_enabled,
        ),
    ];
    let mut overwritten = Vec::new();
    for (name, flag, initialize) in flags {
        let held = flag.load(Ordering::Relaxed);
        for published in [1, 2] {
            flag.store(published, Ordering::Relaxed);
            let answered = initialize();
            let after = flag.load(Ordering::Relaxed);
            flag.store(held, Ordering::Relaxed);
            if after != published || answered != (published == 1) {
                overwritten.push(format!(
                    "{name}: published {published}, after {after}, answered {answered}"
                ));
            }
        }
    }
    assert!(
        overwritten.is_empty(),
        "a cold load replaced a value published after its load: {overwritten:#?}"
    );
}
