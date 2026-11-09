//! Strict flag-read gates; mounted inside config to exercise every getter.
use super::*;
use crate::runtime::Session;
use crate::runtime::profiling::logical::test_support;

#[test]
fn brief713_unregistered_flag_read_is_refused() {
    let _enabled = test_support::enabled();
    let flags = [
        &LEXICAL_PROFILE_ENABLED,
        &LEXICAL_PROFILE_ACTIVE,
        &LEXICAL_PROFILE_DETAIL_ENABLED,
        &LEXICAL_PROFILE_INCREMENTAL_ENABLED,
        &LEXICAL_PROFILE_SHAPES_ENABLED,
        &LEXICAL_PROFILE_BINDINGS_ENABLED,
        &LEXICAL_PROFILE_SOURCES_ENABLED,
        &LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED,
        &LEXICAL_PROFILE_PROVENANCE_ENABLED,
        &RULE_RHS_TIMING_ENABLED,
        &DEFINITION_RULE_TIMING_ENABLED,
        &BUILTIN_TIMING_ENABLED,
    ];
    let saved = {
        let _publication = crate::runtime::profiling::lease::publish()
            .expect("STOP: lease holder must publish profiling flags");
        flags.map(|flag| flag.swap(1, Ordering::Relaxed))
    };
    let cache = Session::test_cache_profile_flags([1; 3]);
    let observed = std::thread::spawn(|| {
        let mut values = vec![
            ("LEXICAL_PROFILE_ENABLED", lexical_profile_configured()),
            ("LEXICAL_PROFILE_ACTIVE", lexical_profile_active()),
            (
                "LEXICAL_PROFILE_DETAIL_ENABLED",
                lexical_profile_detail_enabled(),
            ),
            (
                "LEXICAL_PROFILE_INCREMENTAL_ENABLED",
                lexical_profile_incremental_enabled(),
            ),
            (
                "LEXICAL_PROFILE_SHAPES_ENABLED",
                lexical_profile_shapes_enabled(),
            ),
            (
                "LEXICAL_PROFILE_BINDINGS_ENABLED",
                lexical_profile_bindings_enabled(),
            ),
            (
                "LEXICAL_PROFILE_SOURCES_ENABLED",
                lexical_profile_sources_enabled(),
            ),
            (
                "LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED",
                lexical_profile_source_fingerprints_enabled(),
            ),
            (
                "LEXICAL_PROFILE_PROVENANCE_ENABLED",
                lexical_profile_provenance_enabled(),
            ),
            ("RULE_RHS_TIMING_ENABLED", rule_rhs_timing_enabled()),
            (
                "DEFINITION_RULE_TIMING_ENABLED",
                definition_rule_timing_enabled(),
            ),
            ("BUILTIN_TIMING_ENABLED", builtin_timing_enabled()),
        ];
        values.extend(Session::brief713_cache_flag_reads());
        values
    })
    .join()
    .unwrap();
    for (flag, saved) in flags.into_iter().zip(saved) {
        let _publication = crate::runtime::profiling::lease::publish()
            .expect("STOP: lease holder must publish profiling flags");
        flag.store(saved, Ordering::Relaxed);
    }
    Session::test_cache_profile_flags(cache);
    eprintln!("B713 FLAG_READ {observed:?}");
    assert_eq!(observed.len(), 15);
    assert!(observed.iter().all(|(_, enabled)| !enabled));
}

#[test]
fn brief713_unregistered_start_is_refused() {
    let _enabled = test_support::enabled();
    let started =
        std::thread::spawn(|| crate::runtime::profiling::lexical_profile_start().is_some())
            .join()
            .unwrap();
    eprintln!("B713 START unregistered_start={started}");
    assert!(!started, "an unregistered start must be refused");
}
