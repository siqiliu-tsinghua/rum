use super::*;

#[inline]
pub(in crate::runtime) fn lexical_profile_start() -> Option<Instant> {
    if lexical_profile_enabled() {
        Some(Instant::now())
    } else {
        None
    }
}

#[inline]
pub(in crate::runtime) fn rule_rhs_timing_start() -> Option<Instant> {
    if rule_rhs_timing_enabled() {
        Some(Instant::now())
    } else {
        None
    }
}

#[inline]
pub(in crate::runtime) fn definition_rule_timing_start() -> Option<Instant> {
    if !lexical_profile_enabled() {
        return None;
    }

    definition_rule_timing_start_slow()
}

#[cold]
#[inline(never)]
fn definition_rule_timing_start_slow() -> Option<Instant> {
    if !definition_rule_timing_enabled() {
        return None;
    }

    Some(Instant::now())
}

#[inline]
pub(in crate::runtime) fn definition_dispatch_timing_start() -> Option<Instant> {
    if !lexical_profile_enabled() {
        return None;
    }

    definition_dispatch_timing_start_slow()
}

#[cold]
#[inline(never)]
fn definition_dispatch_timing_start_slow() -> Option<Instant> {
    if !definition_rule_timing_enabled() {
        return None;
    }

    Some(Instant::now())
}

#[inline]
pub(in crate::runtime) fn builtin_timing_start(
    symbol: &Symbol,
    arg_count: usize,
) -> Option<BuiltinTimingToken> {
    if !lexical_profile_enabled() {
        return None;
    }

    builtin_timing_start_slow(symbol, arg_count)
}

#[cold]
#[inline(never)]
fn builtin_timing_start_slow(symbol: &Symbol, arg_count: usize) -> Option<BuiltinTimingToken> {
    if !builtin_timing_enabled() {
        return None;
    }

    let key = BuiltinTimingKey {
        symbol: Symbol::new(symbol.context(), symbol.name()),
        source_symbol: evaluation_profile_current_source_symbol()
            .map(|symbol| Symbol::new(symbol.context(), symbol.name())),
        arg_count: arg_count.min(usize::from(u16::MAX)) as u16,
    };
    BUILTIN_TIMING_STACK.with(|stack| {
        stack.borrow_mut().push(BuiltinTimingFrame {
            key,
            child_nanos: 0,
        });
    });
    Some(BuiltinTimingToken {
        start: Instant::now(),
        key,
    })
}

#[inline]
pub(in crate::runtime) fn runtime_path_timing_start(
    phase: &'static str,
) -> Option<RuntimePathTimingToken> {
    if !lexical_profile_enabled() {
        return None;
    }

    runtime_path_timing_start_slow(phase)
}

#[cold]
#[inline(never)]
fn runtime_path_timing_start_slow(phase: &'static str) -> Option<RuntimePathTimingToken> {
    let key = RuntimePathTimingKey {
        phase,
        source_symbol: evaluation_profile_current_source_symbol()
            .map(|symbol| Symbol::new(symbol.context(), symbol.name())),
    };
    RUNTIME_PATH_TIMING_STACK.with(|stack| {
        stack.borrow_mut().push(RuntimePathTimingFrame {
            key,
            child_nanos: 0,
        });
    });
    Some(RuntimePathTimingToken {
        start: Instant::now(),
        key,
    })
}

#[inline]
pub(in crate::runtime) fn runtime_path_timing_start_adopted(
    session: &mut crate::runtime::Session,
    phase: &'static str,
) -> Option<RuntimePathTimingToken> {
    if !lexical_profile_enabled() {
        return None;
    }

    runtime_path_timing_start_adopted_slow(session, phase)
}

#[cold]
#[inline(never)]
fn runtime_path_timing_start_adopted_slow(
    session: &mut crate::runtime::Session,
    phase: &'static str,
) -> Option<RuntimePathTimingToken> {
    let token = runtime_path_timing_start_slow(phase);
    if let Some(site) = crate::runtime::profiling::logical::PhaseKey::from_name(phase)
        .map(crate::runtime::profiling::logical::PhaseKey::site)
    {
        crate::runtime::profiling::logical::open_phase(session, phase, site);
    }
    token
}

pub(in crate::runtime) fn record_profile_rule_rhs_timing(
    phase: &'static str,
    start: Option<Instant>,
    changed: Option<bool>,
) {
    if !rule_rhs_timing_enabled() {
        return;
    }

    let Some(start) = start else {
        return;
    };
    let elapsed_nanos = elapsed_nanos_since(start);
    let key = RuleRhsTimingKey {
        phase,
        source_symbol: current_expression_rebuild_source_symbol(),
    };
    RULE_RHS_TIMING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.calls = entry.calls.saturating_add(1);
        entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        match changed {
            Some(true) => entry.changed = entry.changed.saturating_add(1),
            Some(false) => entry.unchanged = entry.unchanged.saturating_add(1),
            None => {}
        }
    });
}

#[inline]
pub(in crate::runtime) fn record_profile_definition_rule_timing(
    kind: &'static str,
    symbol: &Symbol,
    index: usize,
    start: Option<Instant>,
    result_is_ok: bool,
) {
    if !lexical_profile_enabled() {
        return;
    }

    let Some(start) = start else {
        return;
    };

    record_profile_definition_rule_timing_slow(kind, symbol, index, start, result_is_ok);
}

#[cold]
#[inline(never)]
fn record_profile_definition_rule_timing_slow(
    kind: &'static str,
    symbol: &Symbol,
    index: usize,
    start: Instant,
    result_is_ok: bool,
) {
    if !definition_rule_timing_enabled() {
        return;
    }

    let elapsed_nanos = elapsed_nanos_since(start);
    let key = DefinitionRuleTimingKey {
        kind,
        symbol: Symbol::new(symbol.context(), symbol.name()),
        index: index.min(u32::MAX as usize) as u32,
    };
    DEFINITION_RULE_TIMING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.calls = entry.calls.saturating_add(1);
        entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        if result_is_ok {
            entry.ok = entry.ok.saturating_add(1);
        } else {
            entry.err = entry.err.saturating_add(1);
        }
    });
}

pub(in crate::runtime) fn record_profile_definition_dispatch_candidate(
    kind: &'static str,
    symbol: &Symbol,
) {
    record_profile_definition_dispatch(kind, symbol, |stats| {
        stats.candidates = stats.candidates.saturating_add(1);
    });
}

pub(in crate::runtime) fn record_profile_definition_dispatch_prefilter_skip(
    kind: &'static str,
    symbol: &Symbol,
) {
    record_profile_definition_dispatch(kind, symbol, |stats| {
        stats.prefilter_skips = stats.prefilter_skips.saturating_add(1);
    });
}

pub(in crate::runtime) fn record_profile_definition_dispatch_fast_outcome(
    kind: &'static str,
    symbol: &Symbol,
    matched: bool,
) {
    record_profile_definition_dispatch(kind, symbol, |stats| {
        if matched {
            stats.fast_matches = stats.fast_matches.saturating_add(1);
        } else {
            stats.fast_misses = stats.fast_misses.saturating_add(1);
        }
    });
}

pub(in crate::runtime) fn record_profile_definition_dispatch_matcher_outcome(
    kind: &'static str,
    symbol: &Symbol,
    matched: bool,
) {
    record_profile_definition_dispatch(kind, symbol, |stats| {
        if matched {
            stats.matcher_matches = stats.matcher_matches.saturating_add(1);
        } else {
            stats.matcher_misses = stats.matcher_misses.saturating_add(1);
        }
    });
}

#[inline]
pub(in crate::runtime) fn record_profile_definition_dispatch_phase_timing(
    kind: &'static str,
    symbol: &Symbol,
    phase: &'static str,
    start: Option<Instant>,
) {
    if !lexical_profile_enabled() {
        return;
    }

    let Some(start) = start else {
        return;
    };

    record_profile_definition_dispatch_phase_timing_slow(kind, symbol, phase, start);
}

#[cold]
#[inline(never)]
fn record_profile_definition_dispatch_phase_timing_slow(
    kind: &'static str,
    symbol: &Symbol,
    phase: &'static str,
    start: Instant,
) {
    if !definition_rule_timing_enabled() {
        return;
    }

    let elapsed_nanos = elapsed_nanos_since(start);
    record_profile_definition_dispatch(kind, symbol, |stats| match phase {
        "prefilter" => {
            stats.prefilter_nanos = stats.prefilter_nanos.saturating_add(elapsed_nanos);
        }
        "fast_match" => {
            stats.fast_match_nanos = stats.fast_match_nanos.saturating_add(elapsed_nanos);
        }
        "compiled_matcher" => {
            stats.compiled_matcher_nanos =
                stats.compiled_matcher_nanos.saturating_add(elapsed_nanos);
        }
        "rhs_substitution" => {
            stats.rhs_substitution_nanos =
                stats.rhs_substitution_nanos.saturating_add(elapsed_nanos);
        }
        _ => {}
    });
}

fn record_profile_definition_dispatch<F>(kind: &'static str, symbol: &Symbol, update: F)
where
    F: FnOnce(&mut DefinitionDispatchStats),
{
    if !lexical_profile_enabled() {
        return;
    }

    let key = DefinitionDispatchKey {
        kind,
        symbol: Symbol::new(symbol.context(), symbol.name()),
        source_symbol: evaluation_profile_current_source_symbol()
            .map(|symbol| Symbol::new(symbol.context(), symbol.name())),
    };
    DEFINITION_DISPATCH_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        update(stats.entry(key).or_default());
    });
}

#[inline]
pub(in crate::runtime) fn record_profile_builtin_timing(
    token: Option<BuiltinTimingToken>,
    result_is_ok: bool,
) {
    if !lexical_profile_enabled() {
        return;
    }

    let Some(token) = token else {
        return;
    };

    record_profile_builtin_timing_slow(token, result_is_ok);
}

#[cold]
#[inline(never)]
fn record_profile_builtin_timing_slow(token: BuiltinTimingToken, result_is_ok: bool) {
    if !builtin_timing_enabled() {
        return;
    }

    let elapsed_nanos = elapsed_nanos_since(token.start);
    let child_nanos = BUILTIN_TIMING_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let frame = stack.pop();
        if let Some(parent) = stack.last_mut() {
            parent.child_nanos = parent.child_nanos.saturating_add(elapsed_nanos);
        }
        frame
            .filter(|frame| frame.key == token.key)
            .map(|frame| frame.child_nanos)
            .unwrap_or(0)
    });
    let self_nanos = elapsed_nanos.saturating_sub(child_nanos);
    BUILTIN_TIMING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(token.key).or_default();
        entry.calls = entry.calls.saturating_add(1);
        entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        entry.self_nanos = entry.self_nanos.saturating_add(self_nanos);
        if result_is_ok {
            entry.ok = entry.ok.saturating_add(1);
        } else {
            entry.err = entry.err.saturating_add(1);
        }
    });
}

#[inline]
pub(in crate::runtime) fn record_profile_runtime_path_timing_adopted<
    T: crate::runtime::profiling::logical::PhaseCompletion,
>(
    session: &mut crate::runtime::Session,
    token: Option<RuntimePathTimingToken>,
    result: &Result<T, crate::runtime::EvalError>,
) {
    if !lexical_profile_enabled() {
        return;
    }

    record_profile_runtime_path_timing_adopted_slow(session, token, result)
}

#[cold]
#[inline(never)]
fn record_profile_runtime_path_timing_adopted_slow<
    T: crate::runtime::profiling::logical::PhaseCompletion,
>(
    session: &mut crate::runtime::Session,
    token: Option<RuntimePathTimingToken>,
    result: &Result<T, crate::runtime::EvalError>,
) {
    let outcome = T::timing_outcome(result);
    if let Some(phase) = token.as_ref().map(|token| token.key.phase) {
        if let Some(key) = crate::runtime::profiling::logical::PhaseKey::from_name(phase) {
            crate::runtime::profiling::logical::complete_phase(session, key, outcome, |call| {
                T::finish_logical(call, result)
            });
        }
    }
    record_profile_runtime_path_timing(token, outcome);
}

#[inline]
pub(in crate::runtime) fn record_profile_runtime_path_timing_adopted_defer_pause<
    T: crate::runtime::profiling::logical::PhaseCompletion,
>(
    session: &mut crate::runtime::Session,
    token: Option<RuntimePathTimingToken>,
    result: &Result<T, crate::runtime::EvalError>,
) {
    if !lexical_profile_enabled() {
        return;
    }

    record_profile_runtime_path_timing_adopted_defer_pause_slow(session, token, result)
}

#[cold]
#[inline(never)]
fn record_profile_runtime_path_timing_adopted_defer_pause_slow<
    T: crate::runtime::profiling::logical::PhaseCompletion,
>(
    session: &mut crate::runtime::Session,
    token: Option<RuntimePathTimingToken>,
    result: &Result<T, crate::runtime::EvalError>,
) {
    let outcome = T::timing_outcome(result);
    if !matches!(outcome, RuntimeProfileOutcome::InputRequired) {
        if let Some(phase) = token.as_ref().map(|token| token.key.phase) {
            if let Some(key) = crate::runtime::profiling::logical::PhaseKey::from_name(phase) {
                crate::runtime::profiling::logical::complete_phase(session, key, outcome, |call| {
                    T::finish_logical(call, result)
                });
            }
        }
    }
    record_profile_runtime_path_timing(token, outcome);
}

#[inline]
pub(in crate::runtime) fn record_profile_runtime_path_timing_adopted_ok(
    session: &mut crate::runtime::Session,
    token: Option<RuntimePathTimingToken>,
) {
    if !lexical_profile_enabled() {
        return;
    }

    record_profile_runtime_path_timing_adopted_ok_slow(session, token)
}

#[cold]
#[inline(never)]
fn record_profile_runtime_path_timing_adopted_ok_slow(
    session: &mut crate::runtime::Session,
    token: Option<RuntimePathTimingToken>,
) {
    let outcome = RuntimeProfileOutcome::Ok;
    if let Some(phase) = token.as_ref().map(|token| token.key.phase) {
        if let Some(key) = crate::runtime::profiling::logical::PhaseKey::from_name(phase) {
            crate::runtime::profiling::logical::complete_phase(
                session,
                key,
                outcome,
                crate::runtime::profiling::logical::finish_logical_expr_ok,
            );
        }
    }
    record_profile_runtime_path_timing(token, outcome);
}

#[inline]
pub(in crate::runtime) fn record_profile_runtime_path_timing_adopted_none(
    session: &mut crate::runtime::Session,
    token: Option<RuntimePathTimingToken>,
) {
    if !lexical_profile_enabled() {
        return;
    }

    record_profile_runtime_path_timing_adopted_none_slow(session, token)
}

#[cold]
#[inline(never)]
fn record_profile_runtime_path_timing_adopted_none_slow(
    session: &mut crate::runtime::Session,
    token: Option<RuntimePathTimingToken>,
) {
    let outcome = RuntimeProfileOutcome::None;
    if let Some(phase) = token.as_ref().map(|token| token.key.phase) {
        if let Some(key) = crate::runtime::profiling::logical::PhaseKey::from_name(phase) {
            crate::runtime::profiling::logical::complete_phase(
                session,
                key,
                outcome,
                crate::runtime::profiling::logical::finish_logical_optional_none,
            );
        }
    }
    record_profile_runtime_path_timing(token, outcome);
}

#[inline]
pub(in crate::runtime) fn record_profile_runtime_path_timing(
    token: Option<RuntimePathTimingToken>,
    outcome: RuntimeProfileOutcome,
) {
    if !lexical_profile_enabled() {
        return;
    }

    let Some(token) = token else {
        return;
    };

    record_profile_runtime_path_timing_slow(token, outcome);
}

#[cold]
#[inline(never)]
fn record_profile_runtime_path_timing_slow(
    token: RuntimePathTimingToken,
    outcome: RuntimeProfileOutcome,
) {
    let elapsed_nanos = elapsed_nanos_since(token.start);
    let child_nanos = RUNTIME_PATH_TIMING_STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let frame = stack.pop();
        if let Some(parent) = stack.last_mut() {
            parent.child_nanos = parent.child_nanos.saturating_add(elapsed_nanos);
        }
        frame
            .filter(|frame| frame.key == token.key)
            .map(|frame| frame.child_nanos)
            .unwrap_or(0)
    });
    let self_nanos = elapsed_nanos.saturating_sub(child_nanos);
    RUNTIME_PATH_TIMING_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(token.key).or_default();
        entry.calls = entry.calls.saturating_add(1);
        entry.nanos = entry.nanos.saturating_add(elapsed_nanos);
        entry.self_nanos = entry.self_nanos.saturating_add(self_nanos);
        match outcome {
            RuntimeProfileOutcome::Ok => {
                entry.ok = entry.ok.saturating_add(1);
            }
            RuntimeProfileOutcome::Some => {
                entry.ok = entry.ok.saturating_add(1);
                entry.some = entry.some.saturating_add(1);
            }
            RuntimeProfileOutcome::None => {
                entry.ok = entry.ok.saturating_add(1);
                entry.none = entry.none.saturating_add(1);
            }
            RuntimeProfileOutcome::Return => {
                entry.err = entry.err.saturating_add(1);
                entry.return_err = entry.return_err.saturating_add(1);
            }
            RuntimeProfileOutcome::Throw => {
                entry.err = entry.err.saturating_add(1);
                entry.throw_err = entry.throw_err.saturating_add(1);
            }
            RuntimeProfileOutcome::InputRequired => {
                entry.err = entry.err.saturating_add(1);
                entry.input_required = entry.input_required.saturating_add(1);
            }
            RuntimeProfileOutcome::Error => {
                entry.err = entry.err.saturating_add(1);
            }
        }
    });
}
