//! One branded accumulator per scope. There is no raw extraction or mutable
//! container view. Publication leaves the scope; callbacks must be adopted.

use super::lease::{Mode, RetentionMode};
use super::*;
use std::marker::PhantomData;

type Brand<'id> = PhantomData<&'id mut &'id ()>;

/// Owned expression occurrences and their inseparable, non-cloneable receipt.
/// The invariant brand prevents moving this handle out of its scope.
#[doc = include_str!("protocol.md")]
pub struct RetainedValues<'id, M: RetentionMode> {
    pub(super) values: Vec<Expr>,
    pub(super) receipt: M::Receipt,
    pub(super) brand: Brand<'id>,
}

/// Outgoing W, accessible as an Expr only after the accounting scope returns.
pub struct RetainedResult<'id, M: RetentionMode> {
    pub(super) outcome: Result<Expr, EvalError>,
    pub(super) receipt: Option<M::Receipt>,
    brand: Brand<'id>,
}

pub struct RetentionContext<'session, 'id, M: RetentionMode> {
    pub(super) session: &'session mut Session,
    pub(super) brand: Brand<'id>,
    pub(super) mode: PhantomData<M>,
}

pub(in crate::runtime) struct ActivePrefix {
    pub(in crate::runtime) values: Vec<Expr>,
    pub(in crate::runtime) charged: bool,
}

pub(super) fn values_bytes(values: &Vec<Expr>) -> Option<usize> {
    let spare = (values.capacity() - values.len()).checked_mul(size_of::<Expr>())?;
    values
        .iter()
        .try_fold(spare, |total, value| total.checked_add(expr_bytes(value)))
}

pub(super) fn expr_bytes(expr: &Expr) -> usize {
    crate::runtime::debug_tools::resources::retained_expr_bytes(expr)
}

pub(super) fn scoped<M: Mode, F>(session: &mut Session, body: F) -> Result<Expr, EvalError>
where
    F: for<'id> FnOnce(
        RetentionContext<'_, 'id, M>,
        RetainedValues<'id, M>,
    ) -> Result<RetainedResult<'id, M>, EvalError>,
{
    let receipt = M::receipt(session);
    let values = RetainedValues {
        values: Vec::new(),
        receipt,
        brand: PhantomData,
    };
    let result = body(
        RetentionContext {
            session,
            brand: PhantomData,
            mode: PhantomData,
        },
        values,
    )?;
    // The outgoing expression exists before its receipt retires. There is no
    // guest call or public unleased extraction between this handoff and return.
    let RetainedResult {
        outcome, receipt, ..
    } = result;
    drop(receipt);
    outcome
}

impl Session {
    /// Explicit unarmed bridge entry. Ordinary builtin dispatch is unchanged.
    /// Reserve/charge/release erase to the Unarmed receipt, (). Only a guest
    /// crossing temporarily lends the owned prefix to the activation bridge.
    pub fn with_unarmed_retention<F>(&mut self, body: F) -> Result<Expr, EvalError>
    where
        F: for<'id> FnOnce(
            RetentionContext<'_, 'id, Unarmed>,
            RetainedValues<'id, Unarmed>,
        ) -> Result<RetainedResult<'id, Unarmed>, EvalError>,
    {
        assert!(
            self.current_memory_limit.is_none(),
            "use the armed accounting context"
        );
        scoped::<Unarmed, F>(self, body)
    }
}

impl AccountingContext<'_> {
    /// Lease-bearing extension to the legacy evaluation adapter. The older
    /// evaluate/compound pilot remains outside retention coverage.
    pub fn with_retention<F>(&mut self, body: F) -> Result<Expr, EvalError>
    where
        F: for<'id> FnOnce(
            RetentionContext<'_, 'id, AccountingShadow>,
            RetainedValues<'id, AccountingShadow>,
        ) -> Result<RetainedResult<'id, AccountingShadow>, EvalError>,
    {
        scoped::<AccountingShadow, F>(self.session, body)
    }
}

#[allow(private_bounds)]
impl<'id, M: Mode> RetentionContext<'_, 'id, M> {
    /// Reserve the proposed extra slots before allocation, then charge actual
    /// capacity. The unarmed specialization is just Vec::reserve_exact.
    pub fn reserve(
        &mut self,
        values: &mut RetainedValues<'id, M>,
        additional: usize,
    ) -> Result<(), EvalError> {
        if M::ARMED {
            let desired = values.values.len().checked_add(additional);
            let delta = desired
                .and_then(|n| n.checked_sub(values.values.capacity()).or(Some(0)))
                .and_then(|n| n.checked_mul(size_of::<Expr>()));
            let projected = values_bytes(&values.values).and_then(|n| n.checked_add(delta?));
            M::resize(&mut values.receipt, || projected);
            self.session
                .enforce_active_memory_limit_for_projected_bytes(0)?;
        }
        values.values.reserve_exact(additional);
        self.charge(values)
    }

    pub(super) fn charge(&mut self, values: &mut RetainedValues<'id, M>) -> Result<(), EvalError> {
        M::resize(&mut values.receipt, || values_bytes(&values.values));
        if M::ARMED {
            self.session
                .enforce_active_memory_limit_for_projected_bytes(0)?;
        }
        Ok(())
    }

    /// Import a known owned occurrence. Shared children remain separate
    /// occurrences. Unknown callback values use CallbackValue::adopt instead.
    pub fn push(
        &mut self,
        values: &mut RetainedValues<'id, M>,
        value: Expr,
    ) -> Result<(), EvalError> {
        self.reserve(values, 1)?;
        if M::ARMED {
            // The empty slot is already reserved; replace that inline charge.
            let bytes = values_bytes(&values.values)
                .and_then(|n| n.checked_sub(size_of::<Expr>()))
                .and_then(|n| n.checked_add(expr_bytes(&value)));
            M::resize(&mut values.receipt, || bytes);
            self.session
                .enforce_active_memory_limit_for_projected_bytes(0)?;
        }
        values.values.push(value);
        Ok(())
    }

    /// Release the children, retaining and charging spare capacity.
    pub fn clear(&mut self, values: &mut RetainedValues<'id, M>) -> Result<(), EvalError> {
        values.values.clear();
        self.charge(values)
    }

    /// Consume the sole accumulator and prepare publication to outgoing W.
    pub fn finish_list(
        self,
        values: RetainedValues<'id, M>,
    ) -> Result<RetainedResult<'id, M>, EvalError> {
        let RetainedValues {
            values,
            mut receipt,
            ..
        } = values;
        let head = system_expr("List");
        M::resize(&mut receipt, || {
            values_bytes(&values)?
                .checked_add(expr_bytes(&head))?
                .checked_add(size_of::<Expr>())
        });
        if M::ARMED {
            self.session
                .enforce_active_memory_limit_for_projected_bytes(0)?;
        }
        let value = Expr::call(head, values);
        Ok(RetainedResult {
            outcome: Ok(value),
            receipt: Some(receipt),
            brand: PhantomData,
        })
    }
}

impl<'id, M: RetentionMode> RetainedResult<'id, M> {
    pub(super) fn failure(error: EvalError) -> Self {
        Self {
            outcome: Err(error),
            receipt: None,
            brand: PhantomData,
        }
    }
}
