//! The unarmed loop stores its shared control and known slots together. Its
//! concrete mode removes the growable-buffer tag from the ordinary loop.
use super::*;

pub(in crate::runtime) trait Buffer:
    std::ops::Deref<Target = [Expr]> + Sized
{
    fn snapshot(session: &Session, symbol: &Symbol, explicit: &[Expr]) -> Self;
    fn working(values: Vec<Expr>, capacity: usize) -> Self;
    fn capacity(&self) -> usize;
    fn bytes(&self) -> Option<usize>;
    fn reserve(&mut self);
    fn push(&mut self, value: Expr);
    fn active(&self) -> ActiveValues;
    fn into_vec(self, session: &mut Session) -> Result<Vec<Expr>, EvalError>;
    fn arm(self) -> OptionBuffer;
}

pub(in crate::runtime) struct FlatBuffer {
    values: Arc<[Expr]>,
    len: usize,
}

pub(in crate::runtime) enum OptionBuffer {
    Growable(Arc<Vec<Expr>>),
    Flat(FlatBuffer),
}

impl Buffer for FlatBuffer {
    #[inline(always)]
    fn snapshot(session: &Session, symbol: &Symbol, explicit: &[Expr]) -> Self {
        if let Some(options) = session.options.get(symbol) {
            let len = options.len();
            return Self {
                values: options.shared_rule_exprs(len + explicit.len()),
                len,
            };
        }
        let options = crate::runtime::options::default_system_option_snapshot(symbol);
        let len = options.len();
        Self {
            values: options.into_shared_rule_exprs(len + explicit.len()),
            len,
        }
    }

    fn working(values: Vec<Expr>, capacity: usize) -> Self {
        let len = values.len();
        let spare = capacity.checked_sub(len).expect("option slot bound");
        Self {
            values: values
                .into_iter()
                .chain(std::iter::repeat_with(|| Expr::small_integer(0)).take(spare))
                .collect(),
            len,
        }
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.values.len()
    }

    fn bytes(&self) -> Option<usize> {
        super::super::retained::slice_bytes(self, self.capacity())?
            .checked_add(2 * size_of::<usize>())
    }

    fn reserve(&mut self) {
        unreachable!("the flat option buffer has every output slot")
    }

    #[inline]
    fn push(&mut self, value: Expr) {
        let slot =
            &mut Arc::get_mut(&mut self.values).expect("no view survives a guest bridge")[self.len];
        // Only this method fills slots, in order. Unused slots contain the
        // resource-free small integer installed at construction.
        debug_assert!(matches!(
            slot,
            Expr::Atom(Atom::Integer(rum_core::IntValue::Small(0)))
        ));
        std::mem::forget(std::mem::replace(slot, value));
        self.len += 1;
    }

    #[inline]
    fn active(&self) -> ActiveValues {
        ActiveValues::Flat(self.values.clone())
    }

    fn into_vec(mut self, session: &mut Session) -> Result<Vec<Expr>, EvalError> {
        // Materialize through ordinary Vec growth. Frozen frames account for
        // that Vec's actual spare capacity, including on repeated suspension.
        let capacity = if self.len == 0 {
            0
        } else {
            self.len.next_power_of_two().max(4)
        };
        if let Some(limit) = session.current_memory_limit {
            // The source receipt stays live while the new slots coexist with
            // the flat allocation. No expression tree is copied or walked.
            let projected = capacity
                .checked_mul(size_of::<Expr>())
                .ok_or(EvalError::MemoryConstraintExceeded { limit })?;
            session.enforce_active_memory_limit_for_projected_bytes(projected)?;
        }
        let mut output = Vec::with_capacity(capacity);
        for value in &mut Arc::get_mut(&mut self.values)
            .expect("the option bridge retired its shared view")[..self.len]
        {
            output.push(std::mem::replace(value, Expr::small_integer(0)));
        }
        Ok(output)
    }

    fn arm(self) -> OptionBuffer {
        OptionBuffer::Flat(self)
    }
}

impl Buffer for OptionBuffer {
    fn snapshot(session: &Session, symbol: &Symbol, _: &[Expr]) -> Self {
        Self::Growable(Arc::new(session.option_rule_exprs_for_symbol(symbol)))
    }

    fn working(values: Vec<Expr>, _: usize) -> Self {
        Self::Growable(Arc::new(values))
    }

    #[inline]
    fn capacity(&self) -> usize {
        match self {
            Self::Growable(values) => values.capacity(),
            Self::Flat(values) => values.capacity(),
        }
    }

    fn bytes(&self) -> Option<usize> {
        match self {
            Self::Growable(values) => values_bytes(values)?.checked_add(SHARED_BYTES),
            Self::Flat(values) => values.bytes(),
        }
    }

    fn reserve(&mut self) {
        let Self::Growable(values) = self else {
            unreachable!("the flat option buffer has every output slot")
        };
        Arc::get_mut(values)
            .expect("no view survives a guest bridge")
            .reserve(1);
    }

    fn push(&mut self, value: Expr) {
        match self {
            Self::Growable(values) => Arc::get_mut(values)
                .expect("no view survives a guest bridge")
                .push(value),
            Self::Flat(values) => values.push(value),
        }
    }

    fn active(&self) -> ActiveValues {
        match self {
            Self::Growable(values) => ActiveValues::Shared(values.clone()),
            Self::Flat(values) => values.active(),
        }
    }

    fn into_vec(self, session: &mut Session) -> Result<Vec<Expr>, EvalError> {
        match self {
            Self::Growable(values) => {
                Ok(Arc::try_unwrap(values).expect("the option bridge retired its shared view"))
            }
            Self::Flat(values) => values.into_vec(session),
        }
    }

    fn arm(self) -> Self {
        self
    }
}

impl std::ops::Deref for FlatBuffer {
    type Target = [Expr];
    #[inline]
    fn deref(&self) -> &[Expr] {
        &self.values[..self.len]
    }
}

impl std::ops::Deref for OptionBuffer {
    type Target = [Expr];
    fn deref(&self) -> &[Expr] {
        match self {
            Self::Growable(values) => values,
            Self::Flat(values) => values,
        }
    }
}
