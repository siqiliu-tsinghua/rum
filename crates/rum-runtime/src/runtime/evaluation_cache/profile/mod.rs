use super::*;
use std::collections::HashMap;
use std::panic::Location;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

mod report;
mod source;

pub(in crate::runtime) use report::print_evaluation_cache_profile_snapshot;
use source::{
    CALL_STABILITY_SOURCE_HEAD_STATS, CALL_STABILITY_TIMING_STACK, EVALUATION_PROFILE_SOURCE_DEPTH,
    EVALUATION_PROFILE_SOURCE_STACK, EVALUATION_PROFILE_TIMING_STACK,
};
pub(super) use source::{
    call_stability_timing_start, current_evaluation_profile_source_symbol,
    record_call_stability_source_head_insertion, record_call_stability_source_head_outcome,
    record_call_stability_source_head_rewritten_dirty_skip,
    record_call_stability_source_head_timing,
};
pub(in crate::runtime) use source::{
    evaluation_cache_profile_source_timing_start, evaluation_profile_current_source_symbol,
    evaluation_profile_definition_source, record_profile_definition_rhs_eval_by_source,
    record_profile_eval_module_by_source, record_profile_module_body_substitution_by_source,
    record_profile_module_init_substitution_by_source,
};

const EVALUATION_CACHE_PROFILE_ENV: &str = "RUM_PROFILE_EVALUATION_CACHE";
const EVALUATION_CACHE_PROFILE_DEFER_ENV: &str = "RUM_PROFILE_EVALUATION_CACHE_DEFER";
const EVALUATION_CACHE_PROFILE_INCREMENTAL_ENV: &str = "RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL";
static EVALUATION_CACHE_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static EVALUATION_CACHE_PROFILE_ACTIVE: AtomicU8 = AtomicU8::new(0);
static EVALUATION_CACHE_PROFILE_INCREMENTAL_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static EPOCH_BUMP_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CACHE_CLEAR_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static SYMBOL_INVALIDATION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_GENERATION_BUMP_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static DIRTY_BLOOM_SATURATION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_STABILITY_SIZE_CAP_CLEAR_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static STABILITY_CHECK_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_STABILITY_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_STABILITY_MISS_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_STABILITY_MISS_ABSENT_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_STABILITY_MISS_GENERATION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_STABILITY_MISS_DIRTY_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_STABILITY_MISS_POINTER_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_STABILITY_INSERT_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static CALL_STABILITY_REWRITTEN_DIRTY_SKIP_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static EVALUATE_ENTRY_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static DOWN_VALUE_SCAN_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static DOWN_VALUE_APPLICATION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static SUB_VALUE_APPLICATION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static BUILTIN_APPLICATION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static EVAL_MODULE_ENTRY_COUNT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default)]
struct SymbolTimingProfile {
    count: u64,
    nanos: u64,
    self_nanos: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct InvalidationSiteKey {
    event: &'static str,
    file: &'static str,
    line: u32,
    source: Option<Symbol>,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct InvalidationSiteStats {
    count: u64,
    target_with_identity: u64,
    target_generated_dollar: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct InvalidationFamilyKey {
    event: &'static str,
    file: &'static str,
    line: u32,
    source: Option<Symbol>,
    target_family: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
struct CallStabilitySourceHeadStats {
    checks: u64,
    hits: u64,
    misses: u64,
    miss_absent: u64,
    miss_generation: u64,
    miss_dirty: u64,
    miss_pointer: u64,
    insertions: u64,
    rewritten_dirty_skips: u64,
    total_nanos: u64,
    self_nanos: u64,
    hit_nanos: u64,
    miss_absent_nanos: u64,
    miss_generation_nanos: u64,
    miss_dirty_nanos: u64,
    miss_pointer_nanos: u64,
}

impl Session {
    pub(in crate::runtime) fn record_profile_evaluate_entry(&self) {
        record_evaluation_cache_counter(&EVALUATE_ENTRY_COUNT, "evaluate_entries");
    }

    pub(in crate::runtime) fn record_profile_down_value_scan(&self, symbol: &Symbol) {
        record_evaluation_cache_counter(&DOWN_VALUE_SCAN_COUNT, "down_value_scans");
        record_symbol_profile_counter("down_value_scans", symbol);
    }

    pub(in crate::runtime) fn record_profile_down_value_application(&self, symbol: &Symbol) {
        record_evaluation_cache_counter(&DOWN_VALUE_APPLICATION_COUNT, "down_value_applications");
        record_symbol_profile_counter("down_value_applications", symbol);
    }

    pub(in crate::runtime) fn record_profile_sub_value_application(&self, symbol: &Symbol) {
        record_evaluation_cache_counter(&SUB_VALUE_APPLICATION_COUNT, "sub_value_applications");
        record_symbol_profile_counter("sub_value_applications", symbol);
    }

    pub(in crate::runtime) fn record_profile_builtin_application(&self, symbol: &Symbol) {
        record_evaluation_cache_counter(&BUILTIN_APPLICATION_COUNT, "builtin_applications");
        record_symbol_profile_counter("builtin_applications", symbol);
    }

    pub(in crate::runtime) fn record_profile_eval_module_entry(&self) {
        record_evaluation_cache_counter(&EVAL_MODULE_ENTRY_COUNT, "eval_module_entries");
        if let Some(symbol) = current_evaluation_profile_source_symbol() {
            record_symbol_profile_counter("eval_module_entries_by_source", &symbol);
        }
    }
}

#[inline]
fn evaluation_cache_profile_enabled() -> bool {
    evaluation_cache_profile_configured() && evaluation_cache_profile_active()
}

#[inline]
fn evaluation_cache_profile_configured() -> bool {
    match EVALUATION_CACHE_PROFILE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_evaluation_cache_profile_enabled(),
    }
}

#[inline]
fn evaluation_cache_profile_active() -> bool {
    match EVALUATION_CACHE_PROFILE_ACTIVE.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_evaluation_cache_profile_active(),
    }
}

#[inline]
fn evaluation_cache_profile_incremental_enabled() -> bool {
    match EVALUATION_CACHE_PROFILE_INCREMENTAL_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_evaluation_cache_profile_incremental_enabled(),
    }
}

#[cold]
fn initialize_evaluation_cache_profile_enabled() -> bool {
    let enabled = std::env::var_os(EVALUATION_CACHE_PROFILE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    EVALUATION_CACHE_PROFILE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn initialize_evaluation_cache_profile_active() -> bool {
    let deferred = std::env::var_os(EVALUATION_CACHE_PROFILE_DEFER_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    let active = !deferred;
    EVALUATION_CACHE_PROFILE_ACTIVE.store(if active { 1 } else { 2 }, Ordering::Relaxed);
    active
}

#[cold]
fn initialize_evaluation_cache_profile_incremental_enabled() -> bool {
    let enabled = std::env::var_os(EVALUATION_CACHE_PROFILE_INCREMENTAL_ENV)
        .map(|value| !value.is_empty() && value != "0")
        .unwrap_or(true);
    EVALUATION_CACHE_PROFILE_INCREMENTAL_ENABLED
        .store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[inline]
pub(super) fn record_evaluation_cache_counter(counter: &AtomicU64, name: &str) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    record_evaluation_cache_counter_slow(counter, name);
}

#[cold]
#[inline(never)]
fn record_evaluation_cache_counter_slow(counter: &AtomicU64, name: &str) {
    let value = counter.fetch_add(1, Ordering::Relaxed) + 1;
    if evaluation_cache_profile_incremental_enabled() && value.is_power_of_two() {
        eprintln!("rum-profile evaluation-cache {name}={value}");
    }
}

fn symbol_profile_counters() -> &'static Mutex<HashMap<(&'static str, Symbol), u64>> {
    static SYMBOL_PROFILE_COUNTERS: OnceLock<Mutex<HashMap<(&'static str, Symbol), u64>>> =
        OnceLock::new();
    SYMBOL_PROFILE_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn invalidation_site_profile_counters()
-> &'static Mutex<HashMap<InvalidationSiteKey, InvalidationSiteStats>> {
    static INVALIDATION_SITE_PROFILE_COUNTERS: OnceLock<
        Mutex<HashMap<InvalidationSiteKey, InvalidationSiteStats>>,
    > = OnceLock::new();
    INVALIDATION_SITE_PROFILE_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn invalidation_family_profile_counters()
-> &'static Mutex<HashMap<InvalidationFamilyKey, InvalidationSiteStats>> {
    static INVALIDATION_FAMILY_PROFILE_COUNTERS: OnceLock<
        Mutex<HashMap<InvalidationFamilyKey, InvalidationSiteStats>>,
    > = OnceLock::new();
    INVALIDATION_FAMILY_PROFILE_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn symbol_timing_profile_counters()
-> &'static Mutex<HashMap<(&'static str, Symbol), SymbolTimingProfile>> {
    static SYMBOL_TIMING_PROFILE_COUNTERS: OnceLock<
        Mutex<HashMap<(&'static str, Symbol), SymbolTimingProfile>>,
    > = OnceLock::new();
    SYMBOL_TIMING_PROFILE_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn record_symbol_profile_counter(name: &'static str, symbol: &Symbol) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    record_symbol_profile_counter_slow(name, *symbol);
}

#[cold]
#[inline(never)]
fn record_symbol_profile_counter_slow(name: &'static str, symbol: Symbol) {
    let mut counters = symbol_profile_counters()
        .lock()
        .expect("profile counter lock should not be poisoned");
    let value = counters.entry((name, symbol)).or_insert(0);
    *value += 1;
    if evaluation_cache_profile_incremental_enabled() && value.is_power_of_two() {
        eprintln!("rum-profile evaluation-cache {name}[{symbol}]={value}");
    }
}

pub(super) fn record_invalidation_site(
    event: &'static str,
    source: Option<Symbol>,
    target: Option<&Symbol>,
    location: &'static Location<'static>,
) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    let target_text = target.map(|symbol| Symbol::new(symbol.context(), symbol.name()));
    let target_family = target.map(symbol_family_name);
    let target_has_identity = target.and_then(Symbol::identity).is_some();
    let target_is_generated_dollar = target.is_some_and(symbol_has_generated_dollar_name);
    let key = InvalidationSiteKey {
        event,
        file: location.file(),
        line: location.line(),
        source,
        target_text,
    };
    let mut counters = invalidation_site_profile_counters()
        .lock()
        .expect("profile invalidation-site counter lock should not be poisoned");
    let entry = counters.entry(key).or_default();
    entry.count = entry.count.saturating_add(1);
    if target_has_identity {
        entry.target_with_identity = entry.target_with_identity.saturating_add(1);
    }
    if target_is_generated_dollar {
        entry.target_generated_dollar = entry.target_generated_dollar.saturating_add(1);
    }

    let family_key = InvalidationFamilyKey {
        event,
        file: location.file(),
        line: location.line(),
        source,
        target_family,
    };
    let mut family_counters = invalidation_family_profile_counters()
        .lock()
        .expect("profile invalidation-family counter lock should not be poisoned");
    let family_entry = family_counters.entry(family_key).or_default();
    family_entry.count = family_entry.count.saturating_add(1);
    if target_has_identity {
        family_entry.target_with_identity = family_entry.target_with_identity.saturating_add(1);
    }
    if target_is_generated_dollar {
        family_entry.target_generated_dollar =
            family_entry.target_generated_dollar.saturating_add(1);
    }
}

pub(in crate::runtime) fn reset_evaluation_cache_profile_counters() {
    if !evaluation_cache_profile_configured() {
        return;
    }

    macro_rules! reset {
        ($($counter:ident),+ $(,)?) => {
            $( $counter.store(0, Ordering::Relaxed); )+
        };
    }

    reset!(
        EPOCH_BUMP_COUNT,
        CACHE_CLEAR_COUNT,
        SYMBOL_INVALIDATION_COUNT,
        CALL_GENERATION_BUMP_COUNT,
        DIRTY_BLOOM_SATURATION_COUNT,
        CALL_STABILITY_SIZE_CAP_CLEAR_COUNT,
        STABILITY_CHECK_COUNT,
        CALL_STABILITY_HIT_COUNT,
        CALL_STABILITY_MISS_COUNT,
        CALL_STABILITY_MISS_ABSENT_COUNT,
        CALL_STABILITY_MISS_GENERATION_COUNT,
        CALL_STABILITY_MISS_DIRTY_COUNT,
        CALL_STABILITY_MISS_POINTER_COUNT,
        CALL_STABILITY_INSERT_COUNT,
        CALL_STABILITY_REWRITTEN_DIRTY_SKIP_COUNT,
        EVALUATE_ENTRY_COUNT,
        DOWN_VALUE_SCAN_COUNT,
        DOWN_VALUE_APPLICATION_COUNT,
        SUB_VALUE_APPLICATION_COUNT,
        BUILTIN_APPLICATION_COUNT,
        EVAL_MODULE_ENTRY_COUNT,
    );

    symbol_profile_counters()
        .lock()
        .expect("profile counter lock should not be poisoned")
        .clear();
    invalidation_site_profile_counters()
        .lock()
        .expect("profile invalidation-site counter lock should not be poisoned")
        .clear();
    invalidation_family_profile_counters()
        .lock()
        .expect("profile invalidation-family counter lock should not be poisoned")
        .clear();
    symbol_timing_profile_counters()
        .lock()
        .expect("profile timing counter lock should not be poisoned")
        .clear();
    EVALUATION_PROFILE_SOURCE_STACK.with(|stack| stack.borrow_mut().clear());
    EVALUATION_PROFILE_SOURCE_DEPTH.with(|depth| depth.set(0));
    EVALUATION_PROFILE_TIMING_STACK.with(|stack| stack.borrow_mut().clear());
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| stats.borrow_mut().clear());
    CALL_STABILITY_TIMING_STACK.with(|stack| stack.borrow_mut().clear());
    EVALUATION_CACHE_PROFILE_ACTIVE.store(1, Ordering::Relaxed);
}
