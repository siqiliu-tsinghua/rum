use super::logical::{
    AdoptedSite, Outcome, PhaseCompletion, PhaseKey, Policy, complete_phase, error_outcome,
    executing_epoch, open_phase,
};
use super::timing::{record_profile_runtime_path_timing, runtime_path_timing_start};
use super::{RuntimePathTimingToken, RuntimeProfileOutcome, lexical_profile_enabled};
use crate::runtime::{EvalError, Expr, Session};

pub(in crate::runtime) struct IncomingPathTimingToken {
    legacy: RuntimePathTimingToken,
}

impl Session {
    #[inline]
    pub(in crate::runtime) fn runtime_path_timing_start_adopted_incoming(
        &self,
        phase: &'static str,
        incoming: &Result<Expr, EvalError>,
    ) -> Option<IncomingPathTimingToken> {
        if !lexical_profile_enabled() {
            return None;
        }
        self.runtime_path_timing_start_adopted_incoming_slow(phase, incoming)
    }

    #[cold]
    #[inline(never)]
    fn runtime_path_timing_start_adopted_incoming_slow(
        &self,
        phase: &'static str,
        incoming: &Result<Expr, EvalError>,
    ) -> Option<IncomingPathTimingToken> {
        let frozen = match incoming {
            Ok(_) => Outcome::Ok,
            Err(EvalError::InputRequired { .. }) => {
                if let Some(epoch) = executing_epoch() {
                    epoch.fault();
                }
                return None;
            }
            Err(error) => error_outcome(error),
        };
        let legacy = runtime_path_timing_start(phase)?;
        if let Some(key) = PhaseKey::from_name(phase) {
            open_phase(
                self,
                key,
                AdoptedSite {
                    policy: Policy::Incoming(frozen),
                },
            );
        }
        Some(IncomingPathTimingToken { legacy })
    }

    #[inline]
    pub(in crate::runtime) fn record_profile_runtime_path_timing_adopted_incoming(
        &mut self,
        token: Option<IncomingPathTimingToken>,
        legacy_incoming: RuntimeProfileOutcome,
        result: &Result<Expr, EvalError>,
    ) {
        let Some(token) = token else {
            return;
        };
        self.record_profile_runtime_path_timing_adopted_incoming_slow(
            token,
            legacy_incoming,
            result,
        );
    }

    #[cold]
    #[inline(never)]
    fn record_profile_runtime_path_timing_adopted_incoming_slow(
        &mut self,
        token: IncomingPathTimingToken,
        legacy_incoming: RuntimeProfileOutcome,
        result: &Result<Expr, EvalError>,
    ) {
        if !lexical_profile_enabled() {
            return;
        }
        if let Some(key) = PhaseKey::from_name(token.legacy.key.phase) {
            let mut boundary = None;
            complete_phase(self, key, RuntimeProfileOutcome::Ok, |call| {
                if matches!(result, Err(EvalError::InputRequired { .. })) {
                    boundary = call.and_then(|call| call.pause());
                } else if let Some(call) = call {
                    call.finish_incoming();
                }
            });
            if let Some(boundary) = boundary {
                self.logical_insert_boundary(boundary, None, false);
            }
        }
        record_profile_runtime_path_timing(Some(token.legacy), legacy_incoming);
    }

    #[inline]
    pub(in crate::runtime) fn record_profile_runtime_path_timing_adopted_forwarded(
        &mut self,
        token: Option<RuntimePathTimingToken>,
        result: &Result<Expr, EvalError>,
        legacy_outcome: RuntimeProfileOutcome,
    ) {
        if !lexical_profile_enabled() {
            return;
        }
        let Some(token) = token else {
            return;
        };
        self.record_profile_runtime_path_timing_adopted_forwarded_slow(
            token,
            result,
            legacy_outcome,
        );
    }

    #[cold]
    #[inline(never)]
    fn record_profile_runtime_path_timing_adopted_forwarded_slow(
        &mut self,
        token: RuntimePathTimingToken,
        result: &Result<Expr, EvalError>,
        legacy_outcome: RuntimeProfileOutcome,
    ) {
        // The starter already transferred a forwarded pause before its consumer.
        if !matches!(result, Err(EvalError::InputRequired { .. })) {
            if let Some(key) = PhaseKey::from_name(token.key.phase) {
                complete_phase(self, key, legacy_outcome, |call| {
                    Expr::finish_logical(call, result)
                });
            }
        }
        record_profile_runtime_path_timing(Some(token), legacy_outcome);
    }
}
