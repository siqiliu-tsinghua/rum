//! Completed synchronous events, owned and bounded by the logical epoch.
use super::epoch;
use crate::runtime::Symbol;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) enum EventKey {
    Runtime {
        phase: &'static str,
        source: Option<Symbol>,
    },
    Rhs {
        phase: &'static str,
        source: Option<Symbol>,
    },
    Dispatch {
        kind: &'static str,
        symbol: Symbol,
        source: Option<Symbol>,
    },
    Counter(&'static str),
    Cache(&'static str),
    CacheSymbol {
        event: &'static str,
        symbol: Symbol,
    },
    CacheSource {
        event: &'static str,
        symbol: Symbol,
    },
}

#[derive(Clone, Copy, Default)]
pub(in crate::runtime) struct EventValue {
    pub(in crate::runtime) vector: [u64; 7],
    pub(in crate::runtime) durations: [u64; 4],
    pub(in crate::runtime) input_required: u64,
}

impl EventValue {
    pub(in crate::runtime) fn count(count: u64) -> Self {
        let mut value = Self::default();
        value.vector[0] = count;
        value
    }

    pub(super) fn add(&mut self, delta: Self) {
        for (value, delta) in self.vector.iter_mut().zip(delta.vector) {
            *value = value.saturating_add(delta);
        }
        for (value, delta) in self.durations.iter_mut().zip(delta.durations) {
            *value = value.saturating_add(delta);
        }
        self.input_required = self.input_required.saturating_add(delta.input_required);
    }
}

// Every caller is already behind its existing family gate. Cache families
// additionally require logical collection; no disabled recorder reaches here.
#[cold]
#[inline(never)]
pub(in crate::runtime) fn add(key: EventKey, delta: EventValue, start: Option<Instant>) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    add_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        key,
        delta,
        start,
    );
}

pub(in crate::runtime) fn add_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    key: EventKey,
    delta: EventValue,
    start: Option<Instant>,
) {
    if !super::super::config::lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }
    if let Some(epoch) = epoch::current_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        epoch.event_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            key,
            delta,
            start,
        );
    }
}

pub(in crate::runtime) fn cache(name: &str) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    cache_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        name,
    )
}

pub(in crate::runtime) fn cache_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    name: &str,
) {
    let event = match name {
        "evaluate_entries" => "EVALUATE_ENTRY_COUNT",
        "down_value_scans" => "DOWN_VALUE_SCAN_COUNT",
        "down_value_applications" => "DOWN_VALUE_APPLICATION_COUNT",
        "sub_value_applications" => "SUB_VALUE_APPLICATION_COUNT",
        "builtin_applications" => "BUILTIN_APPLICATION_COUNT",
        "eval_module_entries" => "EVAL_MODULE_ENTRY_COUNT",
        _ => return,
    };
    add_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
        EventKey::Cache(event),
        EventValue::count(1),
        None,
    );
}

pub(in crate::runtime) fn counter(name: &str, start: Instant, nanos: u64) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    counter_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        name,
        start,
        nanos,
    );
}

pub(in crate::runtime) fn counter_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    name: &str,
    start: Instant,
    nanos: u64,
) {
    if name == "definition_rhs_eval" {
        let mut delta = EventValue::count(1);
        delta.durations[0] = nanos;
        add_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            EventKey::Counter("definition_rhs_eval"),
            delta,
            Some(start),
        );
    }
}
