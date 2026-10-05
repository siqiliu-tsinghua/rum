use super::*;

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct MatcherPatternState {
    pub(super) stack: Vec<MatcherPatternWork>,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct MatcherPatternWork {
    pub(super) original_head: Expr,
    pub(super) original_args: SharedArgs,
    pub(super) resolved_head: Option<Expr>,
    pub(super) prefix: Vec<Expr>,
    pub(super) next: usize,
}

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
pub(super) enum MatcherPatternDestination {
    Root,
    Head,
    Argument(usize),
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) enum MatcherPatternAwait {
    Evaluate {
        destination: MatcherPatternDestination,
        pending_expr: Expr,
    },
    PreparedCall {
        destination: MatcherPatternDestination,
    },
}

impl MatcherPatternState {
    pub(super) fn destination(&self) -> MatcherPatternDestination {
        match self.stack.last() {
            None => MatcherPatternDestination::Root,
            Some(work) if work.resolved_head.is_none() => MatcherPatternDestination::Head,
            Some(work) => MatcherPatternDestination::Argument(work.next),
        }
    }
}
