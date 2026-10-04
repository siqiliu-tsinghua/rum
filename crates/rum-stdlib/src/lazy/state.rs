//! Flat realization state. No source or stage owns another machine node.
use rum_core::{Expr, Rational};
use rum_runtime::kernel_api::FrameRootVisitor;
use rum_runtime::kernel_api::{CancellationObserver, EvalError, FrameRootTrace, Session};

#[derive(Clone, FrameRootTrace)]
pub(super) struct PullMachine {
    pub(super) source: PullSource,
    /// Outermost first; the source port is stages.len().
    pub(super) stages: Vec<PullStage>,
}

#[derive(Clone, FrameRootTrace)]
pub(super) enum PullSource {
    Range {
        current: Rational,
        step: Rational,
    },
    Permutations {
        current: Vec<Expr>,
        exhausted: bool,
    },
    Subsets {
        elements: Vec<Expr>,
        k: usize,
        combo: Vec<usize>,
    },
    Tuples {
        inner_lists: Vec<Expr>,
        digits: Vec<usize>,
        exhausted: bool,
    },
    NestList {
        function: Expr,
        current: Expr,
    },
}

#[derive(Clone, FrameRootTrace)]
pub(super) enum PullStage {
    Map {
        function: Expr,
    },
    Select {
        predicate: Expr,
    },
    Take {
        remaining: usize,
    },
    Drop {
        remaining: usize,
    },
    FoldList {
        function: Expr,
        accumulator: Expr,
        emitted_init: bool,
    },
}

#[derive(Clone, Copy, FrameRootTrace)]
pub(super) struct SavedPullLimits {
    pub(super) iteration_limit: usize,
    pub(super) source_pulls: usize,
}

/// Recorded only on a callback's InputRequired error edge.
#[derive(Clone, FrameRootTrace)]
pub(super) enum AwaitCursor {
    Map { stage: usize },
    Select { stage: usize, candidate: Expr },
    Fold { stage: usize },
    Nest { yield_value: Expr },
}

/// A live program counter, never stored as a frozen frame field.
pub(super) enum DriveCursor {
    Down {
        at: usize,
    },
    Up {
        boundary: usize,
        result: Option<Expr>,
    },
}

pub(super) struct PullContext {
    pub(super) limits: SavedPullLimits,
    cancellation: CancellationObserver,
    pub(super) pending: Option<Box<AwaitCursor>>,
}

impl PullContext {
    pub(super) fn new(session: &Session) -> Self {
        Self::restore(
            session,
            SavedPullLimits {
                iteration_limit: session.resource_limits().iteration_work_limit,
                source_pulls: 0,
            },
        )
    }

    pub(super) fn restore(session: &Session, limits: SavedPullLimits) -> Self {
        Self {
            limits,
            cancellation: session.cancellation_observer(),
            pending: None,
        }
    }

    pub(super) fn charge_source_pull(&mut self) -> Result<(), EvalError> {
        self.observe_cancellation()?;
        if self.limits.source_pulls >= self.limits.iteration_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: self.limits.iteration_limit,
            });
        }
        self.limits.source_pulls += 1;
        Ok(())
    }

    /// The only additional checkpoint: after a guest callback returns Ok,
    /// before applying its value. It does not consume source work.
    pub(super) fn observe_cancellation(&self) -> Result<(), EvalError> {
        if self.cancellation.is_cancelled() {
            Err(EvalError::Cancelled)
        } else {
            Ok(())
        }
    }

    #[cold]
    #[inline(never)]
    pub(super) fn await_callback(&mut self, error: EvalError, cursor: AwaitCursor) -> EvalError {
        if let EvalError::InputRequired { .. } = &error {
            self.pending = Some(Box::new(cursor));
        }
        error
    }
}
