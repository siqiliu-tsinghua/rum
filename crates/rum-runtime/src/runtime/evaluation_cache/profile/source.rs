use super::super::*;
use super::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Copy, Debug)]
pub(super) struct SymbolTimingFrame {
    name: &'static str,
    symbol: Symbol,
    child_nanos: u64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CallStabilityTimingFrame {
    start: Instant,
    child_nanos: u64,
}

thread_local! {
    pub(super) static EVALUATION_PROFILE_SOURCE_STACK: RefCell<Vec<Symbol>> = const { RefCell::new(Vec::new()) };
    pub(super) static EVALUATION_PROFILE_SOURCE_DEPTH: Cell<usize> = const { Cell::new(0) };
    pub(super) static EVALUATION_PROFILE_TIMING_STACK: RefCell<Vec<SymbolTimingFrame>> = const { RefCell::new(Vec::new()) };
    pub(super) static CALL_STABILITY_SOURCE_HEAD_STATS: RefCell<HashMap<CallStabilitySourceHeadKey, CallStabilitySourceHeadStats>> =
        RefCell::new(HashMap::new());
    pub(super) static CALL_STABILITY_TIMING_STACK: RefCell<Vec<CallStabilityTimingFrame>> = const { RefCell::new(Vec::new()) };
}

pub(in crate::runtime) struct EvaluationProfileSourceGuard {
    active: bool,
}

impl Drop for EvaluationProfileSourceGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        EVALUATION_PROFILE_SOURCE_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
        EVALUATION_PROFILE_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

#[inline]
pub(in crate::runtime) fn evaluation_profile_definition_source(
    symbol: Symbol,
) -> EvaluationProfileSourceGuard {
    if !evaluation_cache_profile_enabled() {
        return EvaluationProfileSourceGuard { active: false };
    }

    EVALUATION_PROFILE_SOURCE_STACK.with(|stack| {
        stack.borrow_mut().push(symbol);
    });
    EVALUATION_PROFILE_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    EvaluationProfileSourceGuard { active: true }
}

#[inline]
pub(in crate::runtime) fn evaluation_cache_profile_source_timing_start(
    name: &'static str,
) -> Option<Instant> {
    if !evaluation_cache_profile_enabled() {
        return None;
    }

    let symbol = current_evaluation_profile_source_symbol()?;
    let start = Instant::now();
    EVALUATION_PROFILE_TIMING_STACK.with(|stack| {
        stack.borrow_mut().push(SymbolTimingFrame {
            name,
            symbol,
            child_nanos: 0,
        });
    });
    Some(start)
}

pub(in crate::runtime) fn evaluation_profile_current_source_symbol() -> Option<Symbol> {
    current_evaluation_profile_source_symbol()
}

pub(in crate::runtime) fn record_profile_definition_rhs_eval_by_source(start: Option<Instant>) {
    record_current_source_timing("definition_rhs_eval_by_source", start);
}

pub(in crate::runtime) fn record_profile_eval_module_by_source(start: Option<Instant>) {
    record_current_source_timing("eval_module_by_source", start);
}

pub(in crate::runtime) fn record_profile_module_init_substitution_by_source(
    start: Option<Instant>,
) {
    record_current_source_timing("module_init_substitution_by_source", start);
}

pub(in crate::runtime) fn record_profile_module_body_substitution_by_source(
    start: Option<Instant>,
) {
    record_current_source_timing("module_body_substitution_by_source", start);
}

fn record_current_source_timing(name: &'static str, start: Option<Instant>) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    let Some(symbol) = current_evaluation_profile_source_symbol() else {
        return;
    };
    record_symbol_timing_profile_counter(name, symbol, start);
}

pub(in crate::runtime::evaluation_cache) fn current_evaluation_profile_source_symbol()
-> Option<Symbol> {
    if !evaluation_cache_profile_enabled() {
        return None;
    }

    EVALUATION_PROFILE_SOURCE_STACK.with(|stack| stack.borrow().last().copied())
}

fn record_symbol_timing_profile_counter(
    name: &'static str,
    symbol: Symbol,
    start: Option<Instant>,
) {
    let Some(start) = start else {
        return;
    };
    let elapsed_nanos = start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
    let child_nanos = EVALUATION_PROFILE_TIMING_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let frame = stack.pop();
        if let Some(parent) = stack.last_mut() {
            parent.child_nanos = parent.child_nanos.saturating_add(elapsed_nanos);
        }

        frame
            .filter(|frame| frame.name == name && frame.symbol == symbol)
            .map(|frame| frame.child_nanos)
            .unwrap_or(0)
    });
    let self_nanos = elapsed_nanos.saturating_sub(child_nanos);

    let mut counters = symbol_timing_profile_counters()
        .lock()
        .expect("profile timing counter lock should not be poisoned");
    let entry = counters.entry((name, symbol)).or_default();
    entry.count = entry.count.saturating_add(1);
    entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
    entry.self_nanos = entry.self_nanos.saturating_add(self_nanos);
}

#[cold]
#[inline(never)]
fn call_stability_source_head_key_enabled(head: &Expr) -> Option<CallStabilitySourceHeadKey> {
    let source = current_evaluation_profile_source_symbol()?;
    Some(CallStabilitySourceHeadKey {
        source,
        head: call_stability_head_shape(head),
    })
}

fn call_stability_head_shape(head: &Expr) -> CallStabilityHeadShape {
    match head {
        Expr::Atom(Atom::Symbol(symbol)) => CallStabilityHeadShape::AtomSymbol(*symbol),
        Expr::Atom(_) => CallStabilityHeadShape::AtomOther,
        Expr::Call {
            head: nested_head, ..
        } => match nested_head.as_ref() {
            Expr::Atom(Atom::Symbol(symbol)) => CallStabilityHeadShape::CallHeadSymbol(*symbol),
            _ => CallStabilityHeadShape::CallOther,
        },
    }
}

pub(in crate::runtime::evaluation_cache) fn record_call_stability_source_head_outcome(
    head: &Expr,
    outcome: CallStabilityProfileOutcome,
) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    record_call_stability_source_head_outcome_enabled(head, outcome);
}

#[cold]
#[inline(never)]
fn record_call_stability_source_head_outcome_enabled(
    head: &Expr,
    outcome: CallStabilityProfileOutcome,
) {
    let Some(key) = call_stability_source_head_key_enabled(head) else {
        return;
    };
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.checks = entry.checks.saturating_add(1);
        match outcome {
            CallStabilityProfileOutcome::Hit => {
                entry.hits = entry.hits.saturating_add(1);
            }
            CallStabilityProfileOutcome::MissAbsent => {
                entry.misses = entry.misses.saturating_add(1);
                entry.miss_absent = entry.miss_absent.saturating_add(1);
            }
            CallStabilityProfileOutcome::MissGeneration => {
                entry.misses = entry.misses.saturating_add(1);
                entry.miss_generation = entry.miss_generation.saturating_add(1);
            }
            CallStabilityProfileOutcome::MissDirty => {
                entry.misses = entry.misses.saturating_add(1);
                entry.miss_dirty = entry.miss_dirty.saturating_add(1);
            }
            CallStabilityProfileOutcome::MissPointer => {
                entry.misses = entry.misses.saturating_add(1);
                entry.miss_pointer = entry.miss_pointer.saturating_add(1);
            }
        }
    });
}

pub(in crate::runtime::evaluation_cache) fn call_stability_timing_start() {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    call_stability_timing_start_enabled();
}

#[cold]
#[inline(never)]
fn call_stability_timing_start_enabled() {
    CALL_STABILITY_TIMING_STACK.with(|stack| {
        stack.borrow_mut().push(CallStabilityTimingFrame {
            start: Instant::now(),
            child_nanos: 0,
        });
    });
}

pub(in crate::runtime::evaluation_cache) fn record_call_stability_source_head_timing(
    head: &Expr,
    outcome: CallStabilityProfileOutcome,
) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    record_call_stability_source_head_timing_enabled(head, outcome);
}

#[cold]
#[inline(never)]
fn record_call_stability_source_head_timing_enabled(
    head: &Expr,
    outcome: CallStabilityProfileOutcome,
) {
    let timing = CALL_STABILITY_TIMING_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let frame = stack.pop()?;
        let elapsed_nanos = elapsed_nanos_since(frame.start);
        if let Some(parent) = stack.last_mut() {
            parent.child_nanos = parent.child_nanos.saturating_add(elapsed_nanos);
        }
        Some((elapsed_nanos, frame.child_nanos))
    });
    let Some((elapsed_nanos, child_nanos)) = timing else {
        return;
    };
    let Some(key) = call_stability_source_head_key_enabled(head) else {
        return;
    };
    let self_nanos = elapsed_nanos.saturating_sub(child_nanos);

    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.total_nanos = entry.total_nanos.saturating_add(elapsed_nanos);
        entry.self_nanos = entry.self_nanos.saturating_add(self_nanos);
        match outcome {
            CallStabilityProfileOutcome::Hit => {
                entry.hit_nanos = entry.hit_nanos.saturating_add(elapsed_nanos);
            }
            CallStabilityProfileOutcome::MissAbsent => {
                entry.miss_absent_nanos = entry.miss_absent_nanos.saturating_add(elapsed_nanos);
            }
            CallStabilityProfileOutcome::MissGeneration => {
                entry.miss_generation_nanos =
                    entry.miss_generation_nanos.saturating_add(elapsed_nanos);
            }
            CallStabilityProfileOutcome::MissDirty => {
                entry.miss_dirty_nanos = entry.miss_dirty_nanos.saturating_add(elapsed_nanos);
            }
            CallStabilityProfileOutcome::MissPointer => {
                entry.miss_pointer_nanos = entry.miss_pointer_nanos.saturating_add(elapsed_nanos);
            }
        }
    });
}

pub(in crate::runtime::evaluation_cache) fn record_call_stability_source_head_insertion(
    head: &Expr,
) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    record_call_stability_source_head_insertion_enabled(head);
}

#[cold]
#[inline(never)]
fn record_call_stability_source_head_insertion_enabled(head: &Expr) {
    let Some(key) = call_stability_source_head_key_enabled(head) else {
        return;
    };
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.insertions = entry.insertions.saturating_add(1);
    });
}

pub(in crate::runtime::evaluation_cache) fn record_call_stability_source_head_rewritten_dirty_skip(
    head: &Expr,
) {
    if !evaluation_cache_profile_enabled() {
        return;
    }

    record_call_stability_source_head_rewritten_dirty_skip_enabled(head);
}

#[cold]
#[inline(never)]
fn record_call_stability_source_head_rewritten_dirty_skip_enabled(head: &Expr) {
    let Some(key) = call_stability_source_head_key_enabled(head) else {
        return;
    };
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.rewritten_dirty_skips = entry.rewritten_dirty_skips.saturating_add(1);
    });
}

fn elapsed_nanos_since(start: Instant) -> u64 {
    start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}
