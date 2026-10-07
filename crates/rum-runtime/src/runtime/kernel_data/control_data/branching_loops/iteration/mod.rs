use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
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
    tail: AccumulatorChain<PersistentExprAccumulatorNode>,
    len: usize,
}

#[derive(Clone, FrameRootTrace, AccumulatorChainNode)]
struct PersistentExprAccumulatorNode {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentExprAccumulatorNode>,
    value: Expr,
}

impl PersistentExprAccumulator {
    pub(super) fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    pub(super) fn appended(&self, value: Expr) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PersistentExprAccumulatorNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }

    pub(super) fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        for current in self.tail.into_completion() {
            suffix.push(current.value);
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
    assert_traces::<PersistentExprAccumulatorNode>();
    assert_traces::<AccumulatorChain<PersistentExprAccumulatorNode>>();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::{type_name, type_name_of_val};
    use std::sync::Weak;

    // The helper's depth certificate proves AccumulatorChain's Drop. These
    // assertions bind both production owning links to that certified mechanism
    // without adding an observer to Expr or changing into_vec in test builds.
    fn assert_drain_link<T>(link: &T) {
        assert_eq!(
            type_name_of_val(link),
            type_name::<AccumulatorChain<PersistentExprAccumulatorNode>>(),
            "owning iteration links must use the certified accumulator drain"
        );
    }

    fn append_range(
        mut collected: PersistentExprAccumulator,
        range: std::ops::Range<i64>,
    ) -> (
        PersistentExprAccumulator,
        Vec<Weak<PersistentExprAccumulatorNode>>,
    ) {
        let mut previous_nodes = Vec::new();
        for value in range {
            collected = collected.appended(Expr::integer(value.into()));
            assert_drain_link(&collected.tail);
            let node = collected.tail.as_deref().unwrap();
            assert_drain_link(&node.previous);
            if let Some(previous) = node.previous.clone().into_inner() {
                previous_nodes.push(Arc::downgrade(&previous));
            }
        }
        (collected, previous_nodes)
    }

    #[test]
    fn appended_chain_abandonment_uses_drain() {
        for length in [16, 64] {
            let (collected, previous_nodes) =
                append_range(PersistentExprAccumulator::freeze(Vec::new()), 0..length);
            assert_eq!(previous_nodes.len(), length as usize - 1);
            assert!(previous_nodes.iter().all(|node| node.upgrade().is_some()));
            drop(collected);
            assert!(previous_nodes.iter().all(|node| node.upgrade().is_none()));
        }
    }

    #[test]
    fn shared_tail_survives_abandoned_clone() {
        let (survivor, shared_nodes) =
            append_range(PersistentExprAccumulator::freeze(Vec::new()), 0..16);
        drop(survivor.clone());
        // A shared into_vec step clones this node, including its owning link.
        // Dropping an intact clone must stop at the still-shared previous node.
        drop(survivor.tail.as_deref().unwrap().clone());
        let (abandoned, branch_nodes) = append_range(survivor.clone(), 16..24);
        drop(abandoned);
        assert!(shared_nodes.iter().all(|node| node.upgrade().is_some()));
        assert!(branch_nodes[0].upgrade().is_some());
        assert!(
            branch_nodes[1..]
                .iter()
                .all(|node| node.upgrade().is_none())
        );
        assert_eq!(
            survivor.into_vec(),
            (0..16)
                .map(|value| Expr::integer(value.into()))
                .collect::<Vec<_>>()
        );
        assert!(shared_nodes.iter().all(|node| node.upgrade().is_none()));
        assert!(branch_nodes.iter().all(|node| node.upgrade().is_none()));
    }

    #[test]
    fn into_vec_preserves_prefix_and_tail_order_when_unique_or_shared() {
        let expected: Vec<_> = (-2..64).map(|value| Expr::integer(value.into())).collect();
        for shared in [false, true] {
            let (collected, previous_nodes) = append_range(
                PersistentExprAccumulator::freeze(expected[..2].to_vec()),
                0..64,
            );
            let survivor = shared.then(|| collected.clone());
            assert_eq!(collected.into_vec(), expected);
            if let Some(survivor) = survivor {
                assert!(previous_nodes.iter().all(|node| node.upgrade().is_some()));
                assert_eq!(survivor.into_vec(), expected);
            }
            assert!(previous_nodes.iter().all(|node| node.upgrade().is_none()));
        }
    }

    #[test]
    fn accumulator_is_abandoned_mid_iteration_without_resuming_body() {
        use crate::runtime::suspension::AbandonReason;
        use crate::runtime::test_builtins::{
            register_test_sameq_builtin, session_with_test_control_builtins,
        };

        let mut session = session_with_test_control_builtins();
        register_test_sameq_builtin(&mut session);
        let failure = session
            .eval_request(
                "i = 99; hits = 0; Table[hits++; If[SameQ[i, 1], InputString[\"first\"], If[SameQ[i, 33], InputString[\"stop\"], i]], {i, 64}]",
            )
            .unwrap_err();
        assert!(matches!(
            failure.error,
            SessionRequestError::InputRequired { request } if request.prompt == "first"
        ));
        let failure = session.resume_eval_request_with_input("ready").unwrap_err();
        assert!(matches!(
            failure.error,
            SessionRequestError::InputRequired { request } if request.prompt == "stop"
        ));
        assert_eq!(
            session.abandon_pending_suspension(AbandonReason::CancelledRequest),
            AbandonReason::CancelledRequest
        );
        assert_eq!(
            session.eval_text("{i, hits}").unwrap().value.to_string(),
            "List[99, 33]"
        );
    }
}
