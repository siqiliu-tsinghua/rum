use super::{decode, native};
use crate::runtime::profiling::{
    self,
    logical::{self, epoch, publication},
    state,
};
use crate::runtime::{
    BuiltinSpec, EvalError, Expr, MemoryGrowthContract, Session, SharedArgs, Symbol,
};
use std::sync::atomic::{AtomicU8, Ordering};

pub(super) struct Flags {
    flags: Vec<(&'static AtomicU8, u8)>,
    cache: [u8; 3],
}
impl Flags {
    pub fn mode(mode: &str) -> Self {
        let enabled = |value| if value { 1 } else { 2 };
        let timing = matches!(mode, "timing" | "all");
        let flags = [
            (&state::LEXICAL_PROFILE_ENABLED, enabled(mode != "disabled")),
            (&state::LEXICAL_PROFILE_ACTIVE, enabled(mode != "deferred")),
            (&state::BUILTIN_TIMING_ENABLED, enabled(timing)),
            (&state::RULE_RHS_TIMING_ENABLED, enabled(timing)),
            (&state::DEFINITION_RULE_TIMING_ENABLED, enabled(timing)),
            (&state::LEXICAL_PROFILE_INCREMENTAL_ENABLED, 2),
            (
                &state::LEXICAL_PROFILE_DETAIL_ENABLED,
                enabled(mode == "all"),
            ),
            (
                &state::LEXICAL_PROFILE_SHAPES_ENABLED,
                enabled(mode == "all"),
            ),
            (
                &state::LEXICAL_PROFILE_BINDINGS_ENABLED,
                enabled(mode == "all"),
            ),
            (
                &state::LEXICAL_PROFILE_SOURCES_ENABLED,
                enabled(mode == "all"),
            ),
            (
                &state::LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED,
                enabled(mode == "all"),
            ),
            (
                &state::LEXICAL_PROFILE_PROVENANCE_ENABLED,
                enabled(mode == "all"),
            ),
        ]
        .into_iter()
        .map(|(flag, value)| (flag, flag.swap(value, Ordering::Relaxed)))
        .collect();
        Self {
            flags,
            cache: Session::test_cache_profile_flags([
                enabled(matches!(mode, "cache" | "all")),
                1,
                2,
            ]),
        }
    }
}
impl Drop for Flags {
    fn drop(&mut self) {
        for (flag, old) in &self.flags {
            flag.store(*old, Ordering::Relaxed);
        }
        Session::test_cache_profile_flags(self.cache);
    }
}

pub(super) fn mode_token(mode: &str) -> &'static str {
    match mode {
        "disabled" | "deferred" => "none",
        "lexical" => "lexical",
        "cache" => "lexical+cache",
        "timing" => "lexical+builtin-timing+rule-rhs-timing+definition-rule-timing",
        "all" => "lexical+builtin-timing+rule-rhs-timing+definition-rule-timing+cache",
        _ => panic!("unknown test mode"),
    }
}

pub(super) fn observe(label: &str) -> decode::Snapshot {
    let epoch = epoch::existing();
    let before = epoch.as_ref().map(|e| native::observe(e));
    let text = publication::render(label);
    let parsed = decode::parse(&text);
    if let Some(epoch) = epoch {
        native::parity(&parsed, before.as_ref().unwrap(), &native::observe(&epoch));
        assert_eq!(parsed.epoch, Some(epoch.id.0));
    } else {
        assert!(parsed.rows.is_empty());
        assert_eq!(parsed.epoch, None);
    }
    assert_eq!(parsed.label, label);
    assert_eq!(parsed.reason, None);
    parsed
}

fn snapshot(session: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let parsed = observe("own");
    let own = decode::Key::Builtin(
        native::parts(Symbol::system("RumProfileSnapshot")),
        None,
        args.len() as u16,
    );
    if profiling::builtin_timing_enabled() {
        let row = &parsed.rows[&own];
        assert_eq!(row.vector, [0, 0, 0]);
        assert_eq!(row.accounting, [1, 0, 0]);
    } else {
        assert!(!parsed.rows.contains_key(&own));
    }
    session.eval_rum_profile_snapshot(args)
}
fn reset(session: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    session.eval_rum_profile_reset(args)
}

pub(super) fn session() -> Session {
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    crate::runtime::test_builtins::register_test_arithmetic_builtins(&mut session);
    session.register_builtin(BuiltinSpec::ordinary(
        "RumProfileSnapshot",
        "Snapshot contract",
        snapshot,
        MemoryGrowthContract::Incremental,
    ));
    session.register_builtin(BuiltinSpec::ordinary(
        "RumProfileReset",
        "Reset contract",
        reset,
        MemoryGrowthContract::NonAmplifying,
    ));
    session
}

pub(super) fn reset_epoch(lease: &profiling::lease::TestLease) {
    profiling::reset_lexical_profile_counters();
    crate::runtime::reset_evaluation_cache_profile_counters();
    logical::epoch::reset_for_lease(lease);
}
