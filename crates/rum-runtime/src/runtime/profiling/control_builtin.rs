use super::logical::{AggregateKey, Call, PhaseCompletion, Policy, Storage, executing_epoch};
use super::{
    BuiltinTimingToken, builtin_timing_start, lexical_profile_enabled,
    record_profile_builtin_timing,
};
use crate::runtime::{EvalError, Expr, Session, Symbol};

pub(in crate::runtime) struct AdoptedBuiltinTimingToken(Box<BuiltinRecord>);

struct BuiltinRecord {
    legacy: Option<BuiltinTimingToken>,
    call: Option<Call>,
    storage: Option<Storage>,
}

impl Session {
    #[inline]
    pub(in crate::runtime) fn builtin_timing_start_adopted(
        &self,
        symbol: &Symbol,
        arity: usize,
    ) -> Option<AdoptedBuiltinTimingToken> {
        if !lexical_profile_enabled() {
            return None;
        }
        self.builtin_timing_start_adopted_slow(symbol, arity)
    }

    #[cold]
    #[inline(never)]
    fn builtin_timing_start_adopted_slow(
        &self,
        symbol: &Symbol,
        arity: usize,
    ) -> Option<AdoptedBuiltinTimingToken> {
        // The legacy starter gates timing and captures source and truncated arity.
        let legacy = builtin_timing_start(symbol, arity)?;
        let storage = executing_epoch()
            .and_then(|epoch| epoch.storage(std::mem::size_of::<BuiltinRecord>() + 16));
        let call = storage.as_ref().and_then(|_| {
            Call::start(
                self,
                AggregateKey::Builtin {
                    symbol: legacy.key.symbol,
                    source: legacy.key.source_symbol,
                    arity: legacy.key.arg_count,
                },
                Policy::Expr,
            )
        });
        Some(AdoptedBuiltinTimingToken(Box::new(BuiltinRecord {
            legacy: Some(legacy),
            call,
            storage,
        })))
    }

    #[inline]
    pub(in crate::runtime) fn record_profile_builtin_timing_adopted_boundary(
        &mut self,
        token: Option<AdoptedBuiltinTimingToken>,
        result: &Result<Expr, EvalError>,
    ) {
        if !lexical_profile_enabled() {
            return;
        }
        let Some(token) = token else {
            return;
        };
        self.record_profile_builtin_timing_adopted_boundary_slow(token, result);
    }

    #[cold]
    #[inline(never)]
    fn record_profile_builtin_timing_adopted_boundary_slow(
        &mut self,
        token: AdoptedBuiltinTimingToken,
        result: &Result<Expr, EvalError>,
    ) {
        let BuiltinRecord {
            legacy,
            call,
            storage,
        } = *token.0;
        if matches!(result, Err(EvalError::InputRequired { .. })) {
            if let Some(boundary) = call.and_then(|call| call.pause()) {
                self.logical_insert_boundary(boundary, None, false);
            }
        } else {
            Expr::finish_logical(call, result);
        }
        record_profile_builtin_timing(legacy, result.is_ok());
        drop(storage);
    }
}
