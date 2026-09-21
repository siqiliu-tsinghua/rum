use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

mod driver;
mod spec;
mod table_fast_path;

pub(super) use spec::{build_iteration_body_or_nested, evaluate_do_spec_expr, parse_do_spec};

impl Session {
    pub(in crate::runtime) fn with_iteration_product_scope<F>(
        &mut self,
        evaluate: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&mut Session) -> Result<Expr, EvalError>,
    {
        let parent_budget = self.iteration_product_budget.get();
        match evaluate(self) {
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, WithIterationProductScopeSuspensionFrame { parent_budget });
                Err(EvalError::InputRequired { request })
            }
            outcome => {
                self.iteration_product_budget.set(parent_budget);
                outcome
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum DoSpec {
    Count(usize),
    Iterator { symbol: Symbol, values: Vec<Expr> },
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum SuspendedDoSpec {
    Count(usize),
    Iterator { symbol: Symbol, values: Arc<[Expr]> },
}

impl From<DoSpec> for SuspendedDoSpec {
    fn from(spec: DoSpec) -> Self {
        match spec {
            DoSpec::Count(count) => Self::Count(count),
            DoSpec::Iterator { symbol, values } => Self::Iterator {
                symbol,
                values: Arc::from(values),
            },
        }
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentExprAccumulator {
    prefix: SharedArgs,
    tail: Option<Arc<PersistentExprAccumulatorNode>>,
    len: usize,
}

#[derive(Clone, FrameRootTrace)]
struct PersistentExprAccumulatorNode {
    previous: Option<Arc<PersistentExprAccumulatorNode>>,
    value: Expr,
}

impl PersistentExprAccumulator {
    pub(super) fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: None,
            len,
        }
    }

    pub(super) fn appended(&self, value: Expr) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: Some(Arc::new(PersistentExprAccumulatorNode {
                previous: self.tail.clone(),
                value,
            })),
            len: self.len + 1,
        }
    }

    pub(super) fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail;
        while let Some(current) = node {
            let current = match Arc::try_unwrap(current) {
                Ok(current) => current,
                Err(current) => current.as_ref().clone(),
            };
            suffix.push(current.value);
            node = current.previous;
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum IterationConsumer {
    Do,
    Product,
    Sum,
    Table,
}

impl DoSpec {
    pub(super) fn iteration_count(&self) -> usize {
        match self {
            Self::Count(count) => *count,
            Self::Iterator { values, .. } => values.len(),
        }
    }
}

impl IterationConsumer {
    fn product_scope_tag(self) -> u8 {
        self as u8 + 1
    }

    fn inherits_product_from(self, enclosing_consumer_tag: u8) -> bool {
        enclosing_consumer_tag == self.product_scope_tag()
            || enclosing_consumer_tag == Self::Table.product_scope_tag()
    }

    fn accepts_bare_count(self) -> bool {
        matches!(self, Self::Do | Self::Table)
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<DoSpec>();
    assert_traces::<SuspendedDoSpec>();
    assert_traces::<PersistentExprAccumulator>();
};

#[derive(rum_derive::FrameRootTrace)]
struct WithIterationProductScopeSuspensionFrame {
    parent_budget: (usize, u8),
}

impl crate::runtime::suspension::SuspensionFrame for WithIterationProductScopeSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut parent_budget = self.parent_budget.clone();
        let resumed = (|| {
            session.iteration_product_budget.set(parent_budget);
            outcome
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for WithIterationProductScopeSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_iteration_product_budget(self.parent_budget);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}
