use super::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU8, Ordering};

const EVALUATION_CACHE_PROFILE_ENV: &str = "RUM_PROFILE_EVALUATION_CACHE";
static ASSOCIATION_PROFILE_CONFIGURED: AtomicU8 = AtomicU8::new(0);

thread_local! {
    static ASSOCIATION_CALL_STABILITY_MISS_OUTCOMES: RefCell<Vec<CallStabilityProfileOutcome>> =
        const { RefCell::new(Vec::new()) };
}

pub(in crate::runtime) enum AssociationCallCacheStart {
    Complete((bool, StabilityBloom)),
    Miss(AssociationCallCacheMiss),
}

pub(in crate::runtime) struct AssociationCallCacheMiss {
    key: CallStabilityCacheKey,
}

#[inline]
fn push_association_call_stability_miss_outcome(outcome: CallStabilityProfileOutcome) {
    if !association_profile_configured() {
        return;
    }

    push_association_call_stability_miss_outcome_enabled(outcome);
}

#[cold]
#[inline(never)]
fn push_association_call_stability_miss_outcome_enabled(outcome: CallStabilityProfileOutcome) {
    ASSOCIATION_CALL_STABILITY_MISS_OUTCOMES.with(|outcomes| {
        outcomes.borrow_mut().push(outcome);
    });
}

#[inline]
fn pop_association_call_stability_miss_outcome() -> CallStabilityProfileOutcome {
    if !association_profile_configured() {
        return CallStabilityProfileOutcome::MissAbsent;
    }

    pop_association_call_stability_miss_outcome_enabled()
}

#[inline]
fn association_profile_configured() -> bool {
    match ASSOCIATION_PROFILE_CONFIGURED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_association_profile_configured(),
    }
}

#[cold]
fn initialize_association_profile_configured() -> bool {
    let configured = std::env::var_os(EVALUATION_CACHE_PROFILE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    ASSOCIATION_PROFILE_CONFIGURED.store(if configured { 1 } else { 2 }, Ordering::Relaxed);
    configured
}

#[cold]
#[inline(never)]
fn pop_association_call_stability_miss_outcome_enabled() -> CallStabilityProfileOutcome {
    ASSOCIATION_CALL_STABILITY_MISS_OUTCOMES.with(|outcomes| {
        outcomes
            .borrow_mut()
            .pop()
            .expect("an association call-stability miss must have a profiling outcome")
    })
}

impl Session {
    #[inline(never)]
    pub(in crate::runtime) fn association_rooted_cached_expr_stability_with_bloom(
        &mut self,
        expr: &Expr,
        remaining_depth: usize,
    ) -> (bool, StabilityBloom) {
        if remaining_depth == 0 && matches!(expr, Expr::Call { .. }) {
            return self.association_stability_tail_with_bloom(expr.clone());
        }

        match self.begin_association_call_cache_visit(expr) {
            AssociationCallCacheStart::Complete(result) => result,
            AssociationCallCacheStart::Miss(miss) => {
                let Expr::Call { head, args } = expr else {
                    unreachable!("only calls can miss the call-stability cache")
                };
                let result = self.call_stability_uncached_with_bloom_association_rooted(
                    head,
                    args,
                    remaining_depth - 1,
                );
                self.finish_association_call_cache_miss(miss, head, args, result)
            }
        }
    }

    pub(in crate::runtime) fn begin_association_call_cache_visit(
        &mut self,
        expr: &Expr,
    ) -> AssociationCallCacheStart {
        record_evaluation_cache_counter(&STABILITY_CHECK_COUNT, "stability_checks");
        let Expr::Call { head, args } = expr else {
            return AssociationCallCacheStart::Complete(match expr {
                Expr::Atom(Atom::Symbol(symbol)) => (
                    self.symbol_is_atom_evaluation_stable(symbol),
                    symbol_stability_bloom(symbol),
                ),
                Expr::Atom(_) => (true, StabilityBloom::default()),
                Expr::Call { .. } => unreachable!(),
            });
        };

        call_stability_timing_start();
        let key = CallStabilityCacheKey::new(head, args);
        let mut miss_outcome = CallStabilityProfileOutcome::MissAbsent;
        if let Some(entry) = self.call_stability_cache.get(&key) {
            match self.call_stability_entry_outcome(entry, head, args) {
                CallStabilityProfileOutcome::Hit => {
                    record_call_stability_hit(head.as_ref());
                    return AssociationCallCacheStart::Complete((entry.stable, entry.bloom));
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
        self.stability_native_stack_poll_counter =
            self.stability_native_stack_poll_counter.wrapping_add(1);
        if self.stability_native_stack_poll_counter & (STABILITY_NATIVE_STACK_POLL_STRIDE - 1) == 0
            && self.native_stack_budget_exhausted()
        {
            return AssociationCallCacheStart::Complete((false, StabilityBloom::default()));
        }

        push_association_call_stability_miss_outcome(miss_outcome);
        AssociationCallCacheStart::Miss(AssociationCallCacheMiss { key })
    }

    pub(in crate::runtime) fn finish_association_call_cache_miss(
        &mut self,
        miss: AssociationCallCacheMiss,
        head: &SharedExpr,
        args: &SharedArgs,
        result: (bool, StabilityBloom),
    ) -> (bool, StabilityBloom) {
        let (stable, bloom) = result;
        let miss_outcome = pop_association_call_stability_miss_outcome();
        record_call_stability_source_head_timing(head.as_ref(), miss_outcome);
        self.cache_call_stability_result(miss.key, head, args, stable, bloom);
        (stable, bloom)
    }
}
