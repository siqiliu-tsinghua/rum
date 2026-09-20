use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use std::sync::Arc;

use rum_core::Expr;
use rum_runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, EvalError, FrameAction, FrameRootTrace,
    FrameRootVisitor, PureFrameFinalizer, Session, SuspensionFrame,
};

use super::is_true_expr;

#[derive(Clone, Copy, FrameRootTrace)]
pub(crate) enum ComparatorResultPolicy {
    ExactTrue,
    Custom(fn(&Expr) -> bool),
}

impl ComparatorResultPolicy {
    fn strictly_precedes(self, value: &Expr) -> bool {
        match self {
            Self::ExactTrue => is_true_expr(value),
            Self::Custom(decision) => decision(value),
        }
    }
}

pub(crate) trait MergeSortItem: FrameRootTrace {
    fn ordering_value(&self) -> &Expr;
}

pub(crate) trait MergeCompletion<T>: FrameRootTrace + Send + Sync + 'static {
    fn complete(self, items: Vec<T>) -> Expr;
}

impl MergeSortItem for Expr {
    fn ordering_value(&self) -> &Expr {
        self
    }
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct IndexChunk {
    #[accumulator_link]
    previous: AccumulatorChain<IndexChunk>,
    values: Vec<usize>,
}

#[derive(Default, FrameRootTrace)]
struct PersistentIndexBuffer {
    tail: AccumulatorChain<IndexChunk>,
    pending: Vec<usize>,
    len: usize,
}

impl PersistentIndexBuffer {
    fn push(&mut self, value: usize) {
        self.pending.push(value);
        self.len += 1;
    }

    fn freeze_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        self.tail = AccumulatorChain::new(Some(Arc::new(IndexChunk {
            previous: self.tail.take(),
            values: std::mem::take(&mut self.pending),
        })));
    }

    fn clone_frozen(&self) -> Self {
        assert!(
            self.pending.is_empty(),
            "a merge-sort frame must freeze its pending indices before publication"
        );
        Self {
            tail: self.tail.clone(),
            pending: Vec::new(),
            len: self.len,
        }
    }

    fn into_vec(self) -> Vec<usize> {
        if self.tail.is_none() {
            debug_assert_eq!(self.pending.len(), self.len);
            return self.pending;
        }

        let mut chunks = Vec::new();
        let mut current = self.tail.as_deref();
        while let Some(chunk) = current {
            chunks.push(chunk.values.as_slice());
            current = chunk.previous.as_deref();
        }

        let mut values = Vec::with_capacity(self.len);
        for chunk in chunks.into_iter().rev() {
            values.extend_from_slice(chunk);
        }
        values.extend(self.pending);
        values
    }
}

#[derive(FrameRootTrace)]
struct MergeSortState<T: FrameRootTrace, C: FrameRootTrace> {
    ordering: Expr,
    result_policy: ComparatorResultPolicy,
    items: Arc<Vec<T>>,
    completion: Arc<C>,
    indices: Arc<[usize]>,
    run_width: usize,
    pair_start: usize,
    left_cursor: usize,
    right_cursor: usize,
    merged: PersistentIndexBuffer,
}

impl<T: FrameRootTrace, C: FrameRootTrace> MergeSortState<T, C> {
    fn clone_frozen(&self) -> Self {
        Self {
            ordering: self.ordering.clone(),
            result_policy: self.result_policy,
            items: self.items.clone(),
            completion: self.completion.clone(),
            indices: self.indices.clone(),
            run_width: self.run_width,
            pair_start: self.pair_start,
            left_cursor: self.left_cursor,
            right_cursor: self.right_cursor,
            merged: self.merged.clone_frozen(),
        }
    }

    fn freeze_for_frame(&mut self) {
        self.merged.freeze_pending();
    }
}

enum MergeSortStep {
    Compare(Expr),
    Finish,
}

pub(crate) fn sort_with_comparator<T, C>(
    session: &mut Session,
    items: Vec<T>,
    ordering: Expr,
    result_policy: ComparatorResultPolicy,
    completion: C,
) -> Result<Expr, EvalError>
where
    T: MergeSortItem + Send + Sync + 'static,
    C: MergeCompletion<T>,
{
    let indices = (0..items.len()).collect::<Vec<_>>();
    resume_merge_sort(
        session,
        MergeSortState {
            ordering,
            result_policy,
            items: Arc::new(items),
            completion: Arc::new(completion),
            indices: Arc::from(indices),
            run_width: 1,
            pair_start: 0,
            left_cursor: 0,
            right_cursor: 0,
            merged: PersistentIndexBuffer::default(),
        },
    )
}

fn resume_merge_sort<T, C>(
    session: &mut Session,
    mut state: MergeSortState<T, C>,
) -> Result<Expr, EvalError>
where
    T: MergeSortItem + Send + Sync + 'static,
    C: MergeCompletion<T>,
{
    loop {
        match state.advance() {
            MergeSortStep::Finish => return Box::new(state.into_finalizer()).finish(),
            MergeSortStep::Compare(expression) => match session.evaluate(expression) {
                Ok(value) => {
                    state.apply_comparison(&value);
                }
                Err(EvalError::InputRequired { request }) => {
                    state.freeze_for_frame();
                    rum_runtime::resume_or_suspend!(
                        insert session,
                        MergeSortComparisonFrame { state },
                    );
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            },
        }
    }
}

impl<T, C> MergeSortState<T, C>
where
    T: MergeSortItem,
    C: FrameRootTrace,
{
    fn advance(&mut self) -> MergeSortStep {
        loop {
            let len = self.indices.len();
            if len <= 1 || self.run_width >= len {
                return MergeSortStep::Finish;
            }

            if self.pair_start >= len {
                let merged = std::mem::take(&mut self.merged);
                self.indices = Arc::from(merged.into_vec());
                self.run_width = self.run_width.saturating_mul(2);
                self.pair_start = 0;
                self.left_cursor = 0;
                self.right_cursor = 0;
                continue;
            }

            let (left_start, left_end, right_start, right_end) = self.current_bounds();
            if right_start >= right_end {
                self.drain_left();
                self.finish_pair(right_end);
                continue;
            }

            let left = left_start
                .checked_add(self.left_cursor)
                .filter(|position| *position < left_end)
                .map(|position| self.indices[position]);
            let right = right_start
                .checked_add(self.right_cursor)
                .filter(|position| *position < right_end)
                .map(|position| self.indices[position]);
            match (left, right) {
                (None, _) => {
                    self.drain_right();
                    self.finish_pair(right_end);
                }
                (_, None) => {
                    self.drain_left();
                    self.finish_pair(right_end);
                }
                (Some(left), Some(right)) => {
                    let expression = Expr::call(
                        self.ordering.clone(),
                        vec![
                            self.items[right].ordering_value().clone(),
                            self.items[left].ordering_value().clone(),
                        ],
                    );
                    return MergeSortStep::Compare(expression);
                }
            }
        }
    }

    fn into_finalizer(self) -> MergeSortFinalizer<T, C> {
        MergeSortFinalizer {
            items: self.items,
            completion: self.completion,
            indices: self.indices,
        }
    }

    fn apply_comparison(&mut self, value: &Expr) {
        let (left_start, _, right_start, _) = self.current_bounds();
        if self.result_policy.strictly_precedes(value) {
            let index = self.indices[right_start + self.right_cursor];
            self.right_cursor += 1;
            self.merged.push(index);
        } else {
            let index = self.indices[left_start + self.left_cursor];
            self.left_cursor += 1;
            self.merged.push(index);
        }
    }

    fn drain_left(&mut self) {
        let (left_start, left_end, _, _) = self.current_bounds();
        let start = left_start.saturating_add(self.left_cursor).min(left_end);
        for index in &self.indices[start..left_end] {
            self.merged.push(*index);
        }
    }

    fn drain_right(&mut self) {
        let (_, _, right_start, right_end) = self.current_bounds();
        let start = right_start.saturating_add(self.right_cursor).min(right_end);
        for index in &self.indices[start..right_end] {
            self.merged.push(*index);
        }
    }

    fn finish_pair(&mut self, right_end: usize) {
        self.pair_start = right_end;
        self.left_cursor = 0;
        self.right_cursor = 0;
    }

    fn current_bounds(&self) -> (usize, usize, usize, usize) {
        let len = self.indices.len();
        let left_start = self.pair_start;
        let left_end = left_start.saturating_add(self.run_width).min(len);
        let right_start = left_end;
        let right_end = right_start.saturating_add(self.run_width).min(len);
        (left_start, left_end, right_start, right_end)
    }
}

#[derive(FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct MergeSortComparisonFrame<T: FrameRootTrace, C: FrameRootTrace> {
    state: MergeSortState<T, C>,
}

impl<T, C> SuspensionFrame for MergeSortComparisonFrame<T, C>
where
    T: MergeSortItem + Send + Sync + 'static,
    C: MergeCompletion<T>,
{
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let value = match outcome {
            Ok(value) => value,
            Err(error) => return FrameAction::complete(Err(error)),
        };
        let mut state = self.state.clone_frozen();
        state.apply_comparison(&value);
        match state.advance() {
            MergeSortStep::Compare(expression) => {
                state.freeze_for_frame();
                FrameAction::evaluate(expression, Self { state })
            }
            MergeSortStep::Finish => FrameAction::finalize(state.into_finalizer()),
        }
    }
}

struct MergeSortFinalizer<T, C> {
    items: Arc<Vec<T>>,
    completion: Arc<C>,
    indices: Arc<[usize]>,
}

impl<T, C> PureFrameFinalizer for MergeSortFinalizer<T, C>
where
    T: Send + Sync + 'static,
    C: MergeCompletion<T>,
{
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let items = Arc::try_unwrap(self.items)
            .unwrap_or_else(|_| panic!("completed merge-sort items must be uniquely owned"));
        let completion = Arc::try_unwrap(self.completion)
            .unwrap_or_else(|_| panic!("completed merge-sort callback must be uniquely owned"));
        let mut items = items.into_iter().map(Some).collect::<Vec<_>>();
        let sorted = self
            .indices
            .iter()
            .copied()
            .map(|index| {
                items[index]
                    .take()
                    .expect("each merge-sort item should appear exactly once")
            })
            .collect();
        Ok(completion.complete(sorted))
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ComparatorResultPolicy>();
    assert_traces::<MergeSortComparisonFrame<Expr, Expr>>();
    assert_traces::<MergeSortState<Expr, Expr>>();
};
