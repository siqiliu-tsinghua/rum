use super::lease::Mode;
use super::retained::{ActivePrefix, RetainedValues, RetentionContext, expr_bytes};
use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

/// Owns the context and its prefix until adoption. Neither a successful result
/// nor an error permits the next guest step or an unleased raw extraction.
#[must_use = "adopt the callback result before retaining it or evaluating again"]
pub struct CallbackValue<'session, 'id, M: lease::RetentionMode> {
    context: RetentionContext<'session, 'id, M>,
    prefix: RetainedValues<'id, M>,
    value: Expr,
    receipt: M::Receipt,
}

/// An exceptional outcome is terminal: finish the scope or transfer its prefix
/// on a real suspension. Its owned context prevents retaining a raw EvalError
/// (which can contain expressions) across another evaluation.
pub struct CallbackFailure<'session, 'id, M: lease::RetentionMode> {
    context: RetentionContext<'session, 'id, M>,
    prefix: RetainedValues<'id, M>,
    error: EvalError,
}

#[allow(private_bounds)]
impl<'id, M: Mode> CallbackFailure<'_, 'id, M> {
    pub fn finish(self) -> RetainedResult<'id, M> {
        RetainedResult::failure(self.error)
    }

    pub fn suspend<C: RetentionContinuation>(
        self,
        continuation: impl FnOnce() -> C,
    ) -> RetainedResult<'id, M> {
        self.context.suspend(self.prefix, self.error, continuation)
    }
}

#[allow(private_bounds)]
impl<'session, 'id, M: Mode> CallbackValue<'session, 'id, M> {
    pub fn adopt(
        self,
    ) -> Result<
        (RetentionContext<'session, 'id, M>, RetainedValues<'id, M>),
        CallbackFailure<'session, 'id, M>,
    > {
        let Self {
            mut context,
            mut prefix,
            value,
            receipt,
        } = self;
        if let Err(error) = context.reserve(&mut prefix, 1) {
            return Err(CallbackFailure {
                context,
                prefix,
                error,
            });
        }
        prefix.values.push(value);
        // Charge the destination while the source is live, then release that
        // source before admission. There is no guest call in this handoff.
        M::resize(&mut prefix.receipt, || {
            retained::values_bytes(&prefix.values)
        });
        drop(receipt);
        if M::ARMED {
            if let Err(error) = context
                .session
                .enforce_active_memory_limit_for_projected_bytes(0)
            {
                return Err(CallbackFailure {
                    context,
                    prefix,
                    error,
                });
            }
        }
        Ok((context, prefix))
    }
}

#[allow(private_bounds)]
impl<'session, 'id, M: Mode> RetentionContext<'session, 'id, M> {
    pub fn evaluate(
        self,
        mut prefix: RetainedValues<'id, M>,
        expr: Expr,
    ) -> Result<CallbackValue<'session, 'id, M>, CallbackFailure<'session, 'id, M>> {
        // The explicit ownership bridge exists in both modes. Unarmed crosses
        // without a weight walk, lease, or lock. A later arm can see the actual
        // prefix, including every outer bridge in a nested callback chain.
        let route = self
            .session
            .accounting_route
            .get_or_insert_with(Default::default);
        route.prefixes.push(ActivePrefix {
            values: std::mem::take(&mut prefix.values),
            charged: M::ARMED,
        });
        self.session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
        );
        let outcome = catch_unwind(AssertUnwindSafe(|| self.session.evaluate(expr)));
        prefix.values = self
            .session
            .accounting_route
            .as_mut()
            .expect("bridge route")
            .prefixes
            .pop()
            .expect("live bridge prefix")
            .values;
        self.session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
        );
        let value = match outcome {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => {
                return Err(CallbackFailure {
                    context: self,
                    prefix,
                    error,
                });
            }
            Err(panic) => resume_unwind(panic),
        };
        self.adopt_outcome(prefix, Ok(value))
    }

    /// Private resume boundary: errors are terminal and retain their original
    /// payload; successful values acquire a receipt before any builder grows.
    pub(super) fn adopt_outcome(
        self,
        prefix: RetainedValues<'id, M>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<CallbackValue<'session, 'id, M>, CallbackFailure<'session, 'id, M>> {
        let value = match outcome {
            Ok(value) => value,
            Err(error) => {
                return Err(CallbackFailure {
                    context: self,
                    prefix,
                    error,
                });
            }
        };
        let mut receipt = M::receipt(self.session);
        M::resize(&mut receipt, || Some(expr_bytes(&value)));
        if M::ARMED {
            if let Err(error) = self
                .session
                .enforce_active_memory_limit_for_projected_bytes(0)
            {
                return Err(CallbackFailure {
                    context: self,
                    prefix,
                    error,
                });
            }
        }
        Ok(CallbackValue {
            context: self,
            prefix,
            value,
            receipt,
        })
    }
}

#[allow(private_bounds)]
impl<'id, M: Mode> CallbackValue<'_, 'id, M> {
    /// The rule-specific adoption reserves both child slots in one allocation.
    /// The callback receipt is live before reserve, and retires only after the
    /// destination receipt covers its actual capacity and value occurrence.
    pub(super) fn into_rule(self, name: &Expr) -> Result<RetainedResult<'id, M>, EvalError> {
        let Self {
            mut context,
            mut prefix,
            value,
            receipt,
        } = self;
        assert!(prefix.values.is_empty(), "rule starts with an empty prefix");
        if !M::ARMED && context.session.current_memory_limit.is_some() {
            // A guest may leave a newly armed limit active. Re-enter with a
            // real receipt before the formerly unarmed builder can grow.
            return Ok(RetainedResult {
                outcome: super::rule::finish_armed(context.session, name, value),
                receipt: None,
                brand: std::marker::PhantomData,
            });
        }
        context.reserve(&mut prefix, 2)?;
        prefix.values.push(value);
        M::resize(&mut prefix.receipt, || {
            retained::values_bytes(&prefix.values)
        });
        drop(receipt);
        if M::ARMED {
            context
                .session
                .enforce_active_memory_limit_for_projected_bytes(0)?;
        }
        super::rule::finish_rule(context, prefix, name)
    }
}

#[allow(private_bounds)]
impl<'id, M: Mode> CallbackFailure<'_, 'id, M> {
    /// Preserve the macro's order: success never arrives here, ordinary errors
    /// leave with their payload, and only initialized InputRequired captures a
    /// name. The continuation itself contains no raw expression owner.
    pub(super) fn suspend_rule(mut self, name: &Expr) -> RetainedResult<'id, M> {
        assert!(
            self.prefix.values.is_empty(),
            "rule failure starts with an empty prefix"
        );
        if !matches!(self.error, EvalError::InputRequired { .. })
            || self.context.session.pending_suspension.is_none()
        {
            return self.finish();
        }
        if !M::ARMED && self.context.session.current_memory_limit.is_some() {
            return RetainedResult::failure(super::rule::suspend_armed(
                self.context.session,
                self.error,
                name,
            ));
        }
        if let Err(error) = super::rule::clone_child(&mut self.context, &mut self.prefix, name) {
            self.context.session.abandon_pending_suspension(
                crate::runtime::suspension::AbandonReason::HelperBestEffortDiscard,
            );
            return RetainedResult::failure(error);
        }
        self.context.suspend(self.prefix, self.error, || {
            crate::runtime::options::RuntimeOptionSuspensionFrame
        })
    }
}
