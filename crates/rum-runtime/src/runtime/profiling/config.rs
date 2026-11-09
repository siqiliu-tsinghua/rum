use super::*;

#[inline]
pub(in crate::runtime) fn lexical_profile_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_configured_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && lexical_profile_active_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    )
}

#[inline]
pub(super) fn lexical_profile_configured() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_configured_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_configured_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    match LEXICAL_PROFILE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_enabled(),
    }
}

#[inline]
#[cfg(test)]
fn lexical_profile_active() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_active_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_active_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    match LEXICAL_PROFILE_ACTIVE.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_active(),
    }
}

#[inline]
pub(in crate::runtime) fn lexical_profile_detail_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_detail_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_detail_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && match LEXICAL_PROFILE_DETAIL_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_detail_enabled(),
    }
}

#[inline]
#[cfg(test)]
pub(super) fn lexical_profile_incremental_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_incremental_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_incremental_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    match LEXICAL_PROFILE_INCREMENTAL_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_incremental_enabled(),
    }
}

#[inline]
pub(super) fn lexical_profile_shapes_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_shapes_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_shapes_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && match LEXICAL_PROFILE_SHAPES_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_shapes_enabled(),
    }
}

#[inline]
pub(super) fn lexical_profile_bindings_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_bindings_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_bindings_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && match LEXICAL_PROFILE_BINDINGS_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_bindings_enabled(),
    }
}

#[inline]
pub(super) fn lexical_profile_sources_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_sources_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_sources_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && match LEXICAL_PROFILE_SOURCES_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_sources_enabled(),
    }
}

#[inline]
pub(super) fn lexical_profile_source_fingerprints_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_source_fingerprints_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_source_fingerprints_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_sources_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && match LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_source_fingerprints_enabled(),
    }
}

#[inline]
pub(super) fn lexical_profile_provenance_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    lexical_profile_provenance_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn lexical_profile_provenance_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && match LEXICAL_PROFILE_PROVENANCE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_lexical_profile_provenance_enabled(),
    }
}

#[inline]
pub(super) fn rule_rhs_timing_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    rule_rhs_timing_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn rule_rhs_timing_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && match RULE_RHS_TIMING_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_rule_rhs_timing_enabled(),
    }
}

#[inline]
pub(super) fn definition_rule_timing_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    definition_rule_timing_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn definition_rule_timing_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && match DEFINITION_RULE_TIMING_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_definition_rule_timing_enabled(),
    }
}

#[inline]
pub(super) fn builtin_timing_enabled() -> bool {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return false;
    };
    builtin_timing_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

#[inline]
pub(in crate::runtime) fn builtin_timing_enabled_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> bool {
    lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) && match BUILTIN_TIMING_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_builtin_timing_enabled(),
    }
}

/// Publish a cold-loaded flag over the uninitialized 0 only. A value another
/// writer stored after this thread's load (a test holder's swap,
/// `RumProfileReset[]`) is kept, and answered.
pub(in crate::runtime) fn publish_cold_load(flag: &AtomicU8, enabled: bool) -> bool {
    match flag.compare_exchange(
        0,
        if enabled { 1 } else { 2 },
        Ordering::Relaxed,
        Ordering::Relaxed,
    ) {
        Ok(_) => enabled,
        Err(published) => published == 1,
    }
}

#[cold]
fn initialize_lexical_profile_detail_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_DETAIL_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&LEXICAL_PROFILE_DETAIL_ENABLED, enabled)
}

#[cold]
fn initialize_lexical_profile_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&LEXICAL_PROFILE_ENABLED, enabled)
}

#[cold]
fn initialize_lexical_profile_active() -> bool {
    let deferred = std::env::var_os(LEXICAL_PROFILE_DEFER_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    let active = !deferred;
    publish_cold_load(&LEXICAL_PROFILE_ACTIVE, active)
}

#[cold]
fn initialize_lexical_profile_incremental_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_INCREMENTAL_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&LEXICAL_PROFILE_INCREMENTAL_ENABLED, enabled)
}

#[cold]
fn initialize_lexical_profile_shapes_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_SHAPES_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&LEXICAL_PROFILE_SHAPES_ENABLED, enabled)
}

#[cold]
fn initialize_lexical_profile_bindings_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_BINDINGS_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&LEXICAL_PROFILE_BINDINGS_ENABLED, enabled)
}

#[cold]
fn initialize_lexical_profile_sources_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_SOURCES_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&LEXICAL_PROFILE_SOURCES_ENABLED, enabled)
}

#[cold]
fn initialize_lexical_profile_source_fingerprints_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED, enabled)
}

#[cold]
fn initialize_lexical_profile_provenance_enabled() -> bool {
    let enabled = std::env::var_os(LEXICAL_PROFILE_PROVENANCE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&LEXICAL_PROFILE_PROVENANCE_ENABLED, enabled)
}

#[cold]
fn initialize_rule_rhs_timing_enabled() -> bool {
    let enabled = std::env::var_os(RULE_RHS_TIMING_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&RULE_RHS_TIMING_ENABLED, enabled)
}

#[cold]
fn initialize_definition_rule_timing_enabled() -> bool {
    let enabled = std::env::var_os(DEFINITION_RULE_TIMING_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&DEFINITION_RULE_TIMING_ENABLED, enabled)
}

#[cold]
fn initialize_builtin_timing_enabled() -> bool {
    let enabled =
        std::env::var_os(BUILTIN_TIMING_ENV).is_some_and(|value| !value.is_empty() && value != "0");
    publish_cold_load(&BUILTIN_TIMING_ENABLED, enabled)
}

#[cfg(test)]
#[path = "tests/cold_load.rs"]
mod cold_load;

#[cfg(test)]
#[path = "tests/brief713_flags.rs"]
mod brief713_flags;
