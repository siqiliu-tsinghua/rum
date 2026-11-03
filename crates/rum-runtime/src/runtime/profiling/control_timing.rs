use super::logical::{PhaseCompletion, PhaseKey, complete_phase};
use super::timing::record_profile_runtime_path_timing;
use super::{RuntimePathTimingToken, RuntimeProfileOutcome, lexical_profile_enabled};
use crate::runtime::{EvalError, Expr, Session};

impl Session {
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
