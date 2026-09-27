//! Private option owners. Defaults and merged occurrences share one buffer;
//! the prefix boundary is immutable. Only borrowed read access crosses the
//! consumer API. All mutation and publication remain inside this module.
use super::lease::Mode;
use super::retained::{ActivePrefix, ActiveValues, expr_bytes, values_bytes};
use super::*;
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::Arc;

mod driver;
pub(in crate::runtime) use driver::{drive, resume};

#[cfg(test)]
#[path = "tests/options.rs"]
mod tests;

/// A read-only option view whose lifetime is bounded by the kernel's retained
/// owner and activation bridge. Cloning an expression creates a new occurrence
/// with its own retention obligation; this view grants no receipt for copies.
///
/// ```no_run
/// use rum_core::Expr;
/// use rum_runtime::kernel_api::RetainedOptions;
/// fn inspect(view: RetainedOptions<'_>) { let borrowed: &[Expr] = &view; }
/// ```
///
/// ```compile_fail,E0616
/// use rum_runtime::kernel_api::RetainedOptions;
/// fn extract(view: RetainedOptions<'_>) { let raw = view.values; }
/// ```
///
/// ```compile_fail
/// use rum_core::Expr;
/// use rum_runtime::kernel_api::RetainedOptions;
/// fn escape(view: RetainedOptions<'_>) -> &'static [Expr] { &view }
/// ```
pub struct RetainedOptions<'a> {
    values: &'a [Expr],
}

impl std::ops::Deref for RetainedOptions<'_> {
    type Target = [Expr];
    fn deref(&self) -> &[Expr] {
        self.values
    }
}

impl<'a> IntoIterator for RetainedOptions<'a> {
    type Item = &'a Expr;
    type IntoIter = std::slice::Iter<'a, Expr>;
    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

pub(in crate::runtime) struct OptionState<M: RetentionMode> {
    values: Arc<Vec<Expr>>,
    receipt: M::Receipt,
    bytes: Option<usize>,
    defaults: usize,
    generation: u64,
}

const SHARED_BYTES: usize = size_of::<Vec<Expr>>() + 2 * size_of::<usize>();

pub(in crate::runtime) fn snapshot<M: Mode>(
    session: &mut Session,
    symbol: &Symbol,
) -> Result<OptionState<M>, EvalError> {
    // The SymbolOptions intermediate is destroyed before this boundary admits
    // any guest work. It is never kept alive over a callback.
    let values = session.option_rule_exprs_for_symbol(symbol);
    let defaults = values.len();
    let state = OptionState {
        values: Arc::new(values),
        receipt: M::receipt(session),
        bytes: None,
        defaults,
        generation: session.option_domain_generation(),
    };
    state.charge(session)
}

impl<M: Mode> OptionState<M> {
    fn charge(mut self, session: &mut Session) -> Result<Self, EvalError> {
        if M::ARMED {
            self.bytes = values_bytes(&self.values).and_then(|n| n.checked_add(SHARED_BYTES));
            M::resize(&mut self.receipt, || self.bytes);
            session.enforce_active_memory_limit_for_projected_bytes(0)?;
        }
        Ok(self)
    }

    fn adopt(&mut self, session: &mut Session, value: Expr) -> Result<(), EvalError> {
        let mut incoming = M::receipt(session);
        M::resize(&mut incoming, || Some(expr_bytes(&value)));
        let values = Arc::get_mut(&mut self.values).expect("no view survives a guest bridge");
        if values.len() == values.capacity() {
            let capacity = values.capacity();
            if M::ARMED {
                let desired = capacity.max(2).checked_mul(2);
                let projected = desired
                    .and_then(|n| n.checked_sub(capacity))
                    .and_then(|n| n.checked_mul(size_of::<Expr>()));
                session.enforce_active_memory_limit_for_projected_bytes(projected.ok_or(
                    EvalError::MemoryConstraintExceeded {
                        limit: session.current_memory_limit.unwrap_or(usize::MAX),
                    },
                )?)?;
            }
            values.reserve(1);
            if M::ARMED {
                self.bytes = self.bytes.and_then(|n| {
                    n.checked_add((values.capacity() - capacity).checked_mul(size_of::<Expr>())?)
                });
                M::resize(&mut self.receipt, || self.bytes);
            }
        }
        if M::ARMED {
            self.bytes = self
                .bytes
                .and_then(|n| n.checked_sub(size_of::<Expr>()))
                .and_then(|n| n.checked_add(expr_bytes(&value)));
        }
        values.push(value);
        M::resize(&mut self.receipt, || self.bytes);
        drop(incoming);
        if M::ARMED {
            session.enforce_active_memory_limit_for_projected_bytes(0)?;
        }
        Ok(())
    }

    /// The shared owner stays on the session's explicit activation stack while
    /// borrowed views are used. Cleanup runs on ordinary errors and unwinding.
    fn guest<T>(&self, session: &mut Session, body: impl FnOnce(&mut Session) -> T) -> T {
        session
            .accounting_route
            .get_or_insert_with(Default::default)
            .prefixes
            .push(ActivePrefix {
                values: ActiveValues::Shared(self.values.clone()),
                charged: M::ARMED,
            });
        session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
        );
        let outcome = catch_unwind(AssertUnwindSafe(|| body(session)));
        session
            .accounting_route
            .as_mut()
            .expect("option bridge route")
            .prefixes
            .pop()
            .expect("live option bridge");
        session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
        );
        match outcome {
            Ok(value) => value,
            Err(panic) => resume_unwind(panic),
        }
    }

    fn arm(self, session: &mut Session) -> Result<OptionState<AccountingShadow>, EvalError> {
        assert!(!M::ARMED, "only the unarmed owner changes specialization");
        OptionState {
            values: self.values,
            receipt: AccountingShadow::receipt(session),
            bytes: None,
            defaults: self.defaults,
            generation: self.generation,
        }
        .charge(session)
    }

    fn freeze<C: RetentionContinuation>(
        self,
        session: &mut Session,
        error: EvalError,
        continuation: impl FnOnce() -> C,
    ) -> EvalError {
        let values =
            Arc::try_unwrap(self.values).expect("the option bridge retired its shared view");
        let cx = RetentionContext::<M> {
            session,
            brand: PhantomData,
            mode: PhantomData,
        };
        cx.suspend(
            RetainedValues {
                values,
                receipt: self.receipt,
                brand: PhantomData,
            },
            error,
            continuation,
        )
        .outcome
        .expect_err("freezing preserves the exception")
    }
}
