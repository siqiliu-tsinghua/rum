use super::profile::*;
use super::*;
use std::panic::Location;
use std::sync::OnceLock;

const CALL_STABILITY_CACHE_MAX_ENTRIES_ENV: &str = "RUM_CALL_STABILITY_CACHE_MAX_ENTRIES";
const STABILITY_DIRTY_BLOOM_MAX_BITS_ENV: &str = "RUM_STABILITY_DIRTY_BLOOM_MAX_BITS";

/// Entry-count bound for the pointer-keyed call-stability cache. Entries pin
/// their head/args trees alive, so the map must stay bounded now that
/// invalidation is generation-based instead of clearing on every write.
const CALL_STABILITY_CACHE_MAX_ENTRIES: usize = 1 << 14;

/// Dirty-bit population bound before bloom-based invalidation falls back to a
/// full call-cache generation bump.
const STABILITY_DIRTY_BLOOM_MAX_BITS: u32 = 96;

pub(in crate::runtime) fn symbol_stability_bloom_bit(symbol: &Symbol) -> u128 {
    let identity = match symbol.identity() {
        Some(value) => value.rotate_left(1) | 1,
        None => 0,
    };
    let mix = u64::from(symbol.interned_id()).wrapping_mul(0x9e3779b97f4a7c15) ^ identity;
    1u128 << (mix >> 57)
}

fn symbol_stability_bloom(symbol: &Symbol) -> StabilityBloom {
    let bit = symbol_stability_bloom_bit(symbol);
    if symbol_uses_generated_stability_bloom(symbol) {
        StabilityBloom::generated(bit)
    } else {
        StabilityBloom::regular(bit)
    }
}

fn symbol_uses_generated_stability_bloom(symbol: &Symbol) -> bool {
    symbol.identity().is_some() || symbol_has_generated_dollar_name(symbol)
}

impl Session {
    /// Full invalidation: definition-state change without a single owning
    /// symbol (package loads, context-path mutation, attribute changes).
    pub(in crate::runtime) fn bump_evaluation_epoch(&mut self) {
        record_evaluation_cache_counter(&EPOCH_BUMP_COUNT, "epoch_bumps");
        self.evaluation_epoch = self.evaluation_epoch.saturating_add(1);
        self.clear_evaluation_caches();
    }

    /// Surgical invalidation for a definition write to one symbol: the
    /// symbol-keyed caches drop that symbol's entries exactly, and the
    /// pointer-keyed call cache is invalidated through a dirty bloom over
    /// written symbols. Re-writing a symbol whose bloom bit is already dirty
    /// records that bit as rewritten instead of bumping the whole call-cache
    /// generation, so unrelated stable entries can still hit.
    #[track_caller]
    pub(in crate::runtime) fn bump_evaluation_epoch_for_symbol(&mut self, symbol: &Symbol) {
        record_evaluation_cache_counter(&SYMBOL_INVALIDATION_COUNT, "symbol_invalidations");
        let source = current_evaluation_profile_source_symbol();
        let location = Location::caller();
        record_invalidation_site("symbol_invalidation", source, Some(symbol), location);
        if let Some(source) = source {
            record_symbol_profile_counter("symbol_invalidations_by_source", &source);
        }
        self.evaluation_epoch = self.evaluation_epoch.saturating_add(1);

        self.stable_symbol_cache.remove(symbol);
        self.symbol_call_effect_cache.remove(&SymbolCallEffectKey {
            symbol: *symbol,
            args_empty: true,
        });
        self.symbol_call_effect_cache.remove(&SymbolCallEffectKey {
            symbol: *symbol,
            args_empty: false,
        });

        self.record_symbol_stability_write(symbol, location);

        if self.in_gensym_sweep {
            return;
        }

        if !self.module_activations.is_empty() {
            for activation in &mut self.module_activations {
                if !activation.gensyms.contains(symbol) {
                    activation.escaped = true;
                }
            }
        }
    }

    fn record_symbol_stability_write(
        &mut self,
        symbol: &Symbol,
        location: &'static Location<'static>,
    ) {
        let bit = symbol_stability_bloom_bit(symbol);
        let generated_lane = symbol_uses_generated_stability_bloom(symbol);
        let dirty_lane = if generated_lane {
            self.stability_dirty_bloom.generated
        } else {
            self.stability_dirty_bloom.regular
        };
        let already_dirty = dirty_lane & bit != 0;
        let source = current_evaluation_profile_source_symbol();
        record_invalidation_site("stability_dirty_write", source, Some(symbol), location);
        if let Some(source) = source {
            record_symbol_profile_counter("stability_dirty_writes_by_source", &source);
            if already_dirty {
                record_invalidation_site(
                    "stability_rewritten_dirty_write",
                    Some(source),
                    Some(symbol),
                    location,
                );
                record_symbol_profile_counter(
                    "stability_rewritten_dirty_writes_by_source",
                    &source,
                );
            } else {
                record_invalidation_site(
                    "stability_new_dirty_write",
                    Some(source),
                    Some(symbol),
                    location,
                );
            }
        } else if already_dirty {
            record_invalidation_site(
                "stability_rewritten_dirty_write",
                None,
                Some(symbol),
                location,
            );
        } else {
            record_invalidation_site("stability_new_dirty_write", None, Some(symbol), location);
        }
        let dirty_bloom_would_saturate =
            !already_dirty && (dirty_lane | bit).count_ones() > stability_dirty_bloom_max_bits();
        if dirty_bloom_would_saturate {
            record_evaluation_cache_counter(
                &DIRTY_BLOOM_SATURATION_COUNT,
                "dirty_bloom_saturations",
            );
            let source = current_evaluation_profile_source_symbol();
            record_invalidation_site("dirty_bloom_saturation", source, Some(symbol), location);
            if let Some(source) = source {
                record_symbol_profile_counter("dirty_bloom_saturations_by_source", &source);
            }
            self.bump_call_stability_generation(Some(symbol), location);
        }

        if already_dirty {
            if generated_lane {
                self.stability_rewritten_dirty_bloom.generated |= bit;
            } else {
                self.stability_rewritten_dirty_bloom.regular |= bit;
            }
        }
        if generated_lane {
            self.stability_dirty_bloom.generated |= bit;
        } else {
            self.stability_dirty_bloom.regular |= bit;
        }
    }

    fn bump_call_stability_generation(
        &mut self,
        target: Option<&Symbol>,
        location: &'static Location<'static>,
    ) {
        record_evaluation_cache_counter(&CALL_GENERATION_BUMP_COUNT, "call_generation_bumps");
        let source = current_evaluation_profile_source_symbol();
        record_invalidation_site("call_generation_bump", source, target, location);
        if let Some(source) = source {
            record_symbol_profile_counter("call_generation_bumps_by_source", &source);
        }
        self.call_stability_generation = self.call_stability_generation.saturating_add(1);
        self.stability_dirty_bloom = StabilityBloom::default();
        self.stability_rewritten_dirty_bloom = StabilityBloom::default();
        if self.call_stability_cache.len() > call_stability_cache_max_entries() {
            self.clear_call_stability_cache_for_size_cap();
        }
    }

    pub(in crate::runtime) fn clear_evaluation_caches(&mut self) {
        record_evaluation_cache_counter(&CACHE_CLEAR_COUNT, "cache_clears");
        if let Some(source) = current_evaluation_profile_source_symbol() {
            record_symbol_profile_counter("cache_clears_by_source", &source);
            record_symbol_profile_counter("call_generation_bumps_by_source", &source);
        }
        self.symbol_cache_generation = self.symbol_cache_generation.saturating_add(1);
        self.call_stability_generation = self.call_stability_generation.saturating_add(1);
        self.stability_dirty_bloom = StabilityBloom::default();
        self.stability_rewritten_dirty_bloom = StabilityBloom::default();
        self.stable_symbol_cache.clear();
        self.symbol_call_effect_cache.clear();
        self.call_stability_cache.clear();
    }

    pub(in crate::runtime) fn cached_expr_is_evaluation_stable(&mut self, expr: &Expr) -> bool {
        self.cached_expr_stability_with_bloom(expr).0
    }

    pub(in crate::runtime) fn cached_expr_stability_with_bloom(
        &mut self,
        expr: &Expr,
    ) -> (bool, StabilityBloom) {
        record_evaluation_cache_counter(&STABILITY_CHECK_COUNT, "stability_checks");
        match expr {
            Expr::Call { head, args } => self.call_stability_with_bloom(head, args),
            Expr::Atom(Atom::Symbol(symbol)) => (
                self.symbol_is_atom_evaluation_stable(symbol),
                symbol_stability_bloom(symbol),
            ),
            Expr::Atom(_) => (true, StabilityBloom::default()),
        }
    }

    fn call_stability_with_bloom(
        &mut self,
        head: &SharedExpr,
        args: &SharedArgs,
    ) -> (bool, StabilityBloom) {
        call_stability_timing_start();
        let key = CallStabilityCacheKey::new(head, args);
        let mut miss_outcome = CallStabilityProfileOutcome::MissAbsent;
        if let Some(entry) = self.call_stability_cache.get(&key) {
            match self.call_stability_entry_outcome(entry, head, args) {
                CallStabilityProfileOutcome::Hit => {
                    record_call_stability_hit(head.as_ref());
                    return (entry.stable, entry.bloom);
                }
                CallStabilityProfileOutcome::MissGeneration => {
                    miss_outcome = CallStabilityProfileOutcome::MissGeneration;
                    record_present_call_stability_miss(head.as_ref(), miss_outcome);
                }
                CallStabilityProfileOutcome::MissPointer => {
                    miss_outcome = CallStabilityProfileOutcome::MissPointer;
                    record_present_call_stability_miss(head.as_ref(), miss_outcome);
                }
                CallStabilityProfileOutcome::MissDirty => {
                    miss_outcome = CallStabilityProfileOutcome::MissDirty;
                    record_present_call_stability_miss(head.as_ref(), miss_outcome);
                }
                CallStabilityProfileOutcome::MissAbsent => {
                    unreachable!("an existing call-stability entry cannot be absent")
                }
            }
        } else {
            record_absent_call_stability_miss(head.as_ref());
        }

        record_evaluation_cache_counter(&CALL_STABILITY_MISS_COUNT, "call_stability_misses");
        let (stable, bloom) = self.call_stability_uncached_with_bloom(head, args);
        record_call_stability_source_head_timing(head.as_ref(), miss_outcome);
        self.cache_call_stability_result(key, head, args, stable, bloom);
        (stable, bloom)
    }

    #[inline(never)]
    fn cache_call_stability_result(
        &mut self,
        key: CallStabilityCacheKey,
        head: &SharedExpr,
        args: &SharedArgs,
        stable: bool,
        bloom: StabilityBloom,
    ) {
        // Only stable verdicts are worth caching: an unstable expression is
        // about to be rewritten by evaluation, so its entry would pin a dead
        // tree and never be asked again, while stable subtrees are re-checked
        // by their parents for as long as they stay alive.
        let rewritten_dirty_overlap = bloom.intersects(self.stability_rewritten_dirty_bloom);
        if stable && !rewritten_dirty_overlap {
            if self.call_stability_cache.len() > call_stability_cache_max_entries() {
                self.clear_call_stability_cache_for_size_cap();
            }
            let dirty_snapshot = self.stability_dirty_bloom;
            record_evaluation_cache_counter(
                &CALL_STABILITY_INSERT_COUNT,
                "call_stability_insertions",
            );
            record_call_stability_source_head_insertion(head.as_ref());
            self.call_stability_cache.insert(
                key,
                CallStabilityCacheEntry {
                    generation: self.call_stability_generation,
                    head: head.clone(),
                    args: args.clone(),
                    stable,
                    bloom,
                    dirty_snapshot,
                },
            );
        } else if stable {
            record_evaluation_cache_counter(
                &CALL_STABILITY_REWRITTEN_DIRTY_SKIP_COUNT,
                "call_stability_rewritten_dirty_skips",
            );
            record_call_stability_source_head_rewritten_dirty_skip(head.as_ref());
        }
    }

    pub(super) fn call_stability_entry_outcome(
        &self,
        entry: &CallStabilityCacheEntry,
        head: &SharedExpr,
        args: &SharedArgs,
    ) -> CallStabilityProfileOutcome {
        if entry.generation != self.call_stability_generation {
            CallStabilityProfileOutcome::MissGeneration
        } else if !SharedExpr::ptr_eq(&entry.head, head) || !SharedArgs::ptr_eq(&entry.args, args) {
            CallStabilityProfileOutcome::MissPointer
        } else if entry
            .bloom
            .intersects(self.call_stability_entry_dirty_delta(entry))
        {
            CallStabilityProfileOutcome::MissDirty
        } else {
            CallStabilityProfileOutcome::Hit
        }
    }

    fn call_stability_entry_dirty_delta(&self, entry: &CallStabilityCacheEntry) -> StabilityBloom {
        StabilityBloom {
            regular: (self.stability_dirty_bloom.regular & !entry.dirty_snapshot.regular)
                | (self.stability_rewritten_dirty_bloom.regular & entry.dirty_snapshot.regular),
            generated: (self.stability_dirty_bloom.generated & !entry.dirty_snapshot.generated)
                | (self.stability_rewritten_dirty_bloom.generated & entry.dirty_snapshot.generated),
        }
    }

    pub(super) fn clear_call_stability_cache_for_size_cap(&mut self) {
        record_evaluation_cache_counter(
            &CALL_STABILITY_SIZE_CAP_CLEAR_COUNT,
            "call_stability_size_cap_clears",
        );
        self.call_stability_cache.clear();
    }
}

#[inline(never)]
fn record_call_stability_hit(head: &Expr) {
    record_evaluation_cache_counter(&CALL_STABILITY_HIT_COUNT, "call_stability_hits");
    record_call_stability_source_head_outcome(head, CallStabilityProfileOutcome::Hit);
    record_call_stability_source_head_timing(head, CallStabilityProfileOutcome::Hit);
}

#[cold]
#[inline(never)]
fn record_present_call_stability_miss(head: &Expr, outcome: CallStabilityProfileOutcome) {
    let (counter, name) = match outcome {
        CallStabilityProfileOutcome::MissGeneration => (
            &CALL_STABILITY_MISS_GENERATION_COUNT,
            "call_stability_miss_generation",
        ),
        CallStabilityProfileOutcome::MissDirty => (
            &CALL_STABILITY_MISS_DIRTY_COUNT,
            "call_stability_miss_dirty",
        ),
        CallStabilityProfileOutcome::MissPointer => (
            &CALL_STABILITY_MISS_POINTER_COUNT,
            "call_stability_miss_pointer",
        ),
        CallStabilityProfileOutcome::Hit | CallStabilityProfileOutcome::MissAbsent => {
            unreachable!("a present call-stability miss must classify its invalidation")
        }
    };
    record_evaluation_cache_counter(counter, name);
    record_call_stability_source_head_outcome(head, outcome);
}

#[inline(never)]
fn record_absent_call_stability_miss(head: &Expr) {
    record_evaluation_cache_counter(
        &CALL_STABILITY_MISS_ABSENT_COUNT,
        "call_stability_miss_absent",
    );
    record_call_stability_source_head_outcome(head, CallStabilityProfileOutcome::MissAbsent);
}

#[inline]
pub(super) fn call_stability_cache_max_entries() -> usize {
    static MAX_ENTRIES: OnceLock<usize> = OnceLock::new();
    *MAX_ENTRIES.get_or_init(|| {
        std::env::var(CALL_STABILITY_CACHE_MAX_ENTRIES_ENV)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(CALL_STABILITY_CACHE_MAX_ENTRIES)
    })
}

#[inline]
fn stability_dirty_bloom_max_bits() -> u32 {
    static MAX_BITS: OnceLock<u32> = OnceLock::new();
    *MAX_BITS.get_or_init(|| {
        std::env::var(STABILITY_DIRTY_BLOOM_MAX_BITS_ENV)
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| *value >= 1 && *value <= 128)
            .unwrap_or(STABILITY_DIRTY_BLOOM_MAX_BITS)
    })
}
