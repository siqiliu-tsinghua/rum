use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
pub(super) struct ReapRenderAccumulator {
    prefix: SharedArgs,
    tail: AccumulatorChain<ReapRenderNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct ReapRenderNode {
    #[accumulator_link]
    previous: AccumulatorChain<ReapRenderNode>,
    value: Expr,
}

impl ReapRenderAccumulator {
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
            tail: AccumulatorChain::new(Some(Arc::new(ReapRenderNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }

    pub(super) fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            let current = match Arc::try_unwrap(current) {
                Ok(current) => current,
                Err(_) => panic!("completed Reap accumulator must be uniquely owned"),
            };
            suffix.push(current.value);
            node = current.previous.into_inner();
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedReapGroup {
    pub(super) buckets: Arc<[ReapBucket]>,
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedReapScopeRender {
    pub(super) handler: Option<Expr>,
    pub(super) pattern_list_form: bool,
    pub(super) groups: Arc<[SuspendedReapGroup]>,
    pub(super) group_index: usize,
    pub(super) rendered: ReapRenderAccumulator,
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedReapBucketRender {
    pub(super) handler: Option<Expr>,
    pub(super) buckets: Arc<[ReapBucket]>,
    pub(super) bucket_index: usize,
    pub(super) rendered: ReapRenderAccumulator,
}

impl SuspendedReapScopeRender {
    pub(super) fn freeze(
        state: super::reap::ReapScopeRenderState,
        group_index: usize,
        rendered: Vec<Expr>,
    ) -> Self {
        let groups = state
            .groups
            .into_iter()
            .map(|group| SuspendedReapGroup {
                buckets: Arc::from(group.buckets),
            })
            .collect::<Vec<_>>();
        Self {
            handler: state.handler,
            pattern_list_form: state.pattern_list_form,
            groups: Arc::from(groups),
            group_index,
            rendered: ReapRenderAccumulator::freeze(rendered),
        }
    }
}

impl SuspendedReapBucketRender {
    pub(super) fn freeze(
        state: super::reap::ReapBucketRenderState,
        bucket_index: usize,
        rendered: Vec<Expr>,
    ) -> Self {
        Self {
            handler: state.handler,
            buckets: Arc::from(state.buckets),
            bucket_index,
            rendered: ReapRenderAccumulator::freeze(rendered),
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ReapRenderAccumulator>();
    assert_traces::<SuspendedReapScopeRender>();
    assert_traces::<SuspendedReapBucketRender>();
};
