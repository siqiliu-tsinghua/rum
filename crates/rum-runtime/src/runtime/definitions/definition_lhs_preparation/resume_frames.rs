use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
pub(super) enum DefinitionLhsRegion {
    Root,
    ActivePattern,
    RawPattern,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct DefinitionLhsPreparationState {
    pub(super) work: DefinitionLhsPreparationWork,
    pub(super) parents: DefinitionLhsParentStack,
    pub(super) continuation: DefinitionLhsPreparationContinuation,
    pub(super) prepared_outer_call: Option<PreparedDefinitionOuterCall>,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) enum DefinitionLhsPreparationWork {
    Vacant,
    PrepareNode {
        expr: Expr,
        region: DefinitionLhsRegion,
    },
    AwaitPatternAtom {
        source: Expr,
    },
    AwaitCallHead {
        source_head: SharedExpr,
        args: SharedArgs,
        capture_outer_call: bool,
    },
    PrepareCallArguments(DefinitionLhsCallState),
    AwaitCallArgument {
        call: DefinitionLhsCallState,
        argument: DefinitionLhsCallArgumentContinuation,
        source: Expr,
    },
    CompleteNode(Expr),
}

#[derive(Clone, Default, rum_derive::FrameRootTrace)]
pub(super) struct DefinitionLhsParentStack {
    tail: Option<Arc<DefinitionLhsParentNode>>,
}

#[derive(rum_derive::FrameRootTrace)]
struct DefinitionLhsParentNode {
    previous: Option<Arc<DefinitionLhsParentNode>>,
    context: DefinitionLhsParentContext,
}

impl DefinitionLhsParentStack {
    pub(super) fn push(&mut self, context: DefinitionLhsParentContext) {
        self.tail = Some(Arc::new(DefinitionLhsParentNode {
            previous: self.tail.clone(),
            context,
        }));
    }

    pub(super) fn pop(&mut self) -> Option<DefinitionLhsParentContext> {
        let node = self.tail.take()?;
        self.tail = node.previous.clone();
        Some(node.context.clone())
    }
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) enum DefinitionLhsParentContext {
    CallArgument {
        call: DefinitionLhsCallState,
        argument: DefinitionLhsCallArgumentContinuation,
    },
    StructuralNode {
        call: DefinitionLhsStructuralCallState,
        source: DefinitionLhsStructuralSource,
    },
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct DefinitionLhsCallState {
    pub(super) head: SharedExpr,
    pub(super) attributes: AttributeSet,
    pub(super) args: SharedArgs,
    pub(super) cursor: usize,
    pub(super) prepared: Option<PreparedArgumentAccumulator>,
    pub(super) unevaluated_restores: Option<Vec<(usize, Expr)>>,
    pub(super) outer_dispatch_plan: Option<SuspendedCallDispatchPlan>,
}

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
pub(super) struct DefinitionLhsCallArgumentContinuation {
    pub(super) source_index: usize,
    pub(super) allow_sequence_splice: bool,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct DefinitionLhsStructuralCallState {
    pub(super) head: SharedExpr,
    pub(super) args: SharedArgs,
    pub(super) selection: DefinitionLhsStructuralSelection,
    pub(super) child_region: DefinitionLhsRegion,
    pub(super) prepared: Option<PreparedArgumentAccumulator>,
}

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
pub(super) enum DefinitionLhsStructuralSource {
    Head,
    Argument(usize),
}

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
pub(super) enum DefinitionLhsStructuralSelection {
    All { cursor: usize },
    One { index: Option<usize> },
    RawTree { head_pending: bool, cursor: usize },
}

impl DefinitionLhsStructuralSelection {
    pub(super) fn next(&mut self, len: usize) -> Option<DefinitionLhsStructuralSource> {
        match self {
            Self::All { cursor } if *cursor < len => {
                let index = *cursor;
                *cursor += 1;
                Some(DefinitionLhsStructuralSource::Argument(index))
            }
            Self::All { .. } => None,
            Self::One { index } => index
                .take()
                .filter(|index| *index < len)
                .map(DefinitionLhsStructuralSource::Argument),
            Self::RawTree { head_pending, .. } if *head_pending => {
                *head_pending = false;
                Some(DefinitionLhsStructuralSource::Head)
            }
            Self::RawTree { cursor, .. } if *cursor < len => {
                let index = *cursor;
                *cursor += 1;
                Some(DefinitionLhsStructuralSource::Argument(index))
            }
            Self::RawTree { .. } => None,
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DefinitionLhsPreparationSuspensionFrame {
    pub(super) state:
        crate::runtime::definitions::definition_lhs_preparation::DefinitionLhsPreparationState,
}

impl crate::runtime::suspension::SuspensionFrame for DefinitionLhsPreparationSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = session.resume_definition_lhs_preparation(state, outcome);
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<DefinitionLhsCallArgumentContinuation>();
    assert_traces::<DefinitionLhsCallState>();
    assert_traces::<DefinitionLhsParentContext>();
    assert_traces::<DefinitionLhsParentNode>();
    assert_traces::<DefinitionLhsParentStack>();
    assert_traces::<DefinitionLhsPreparationState>();
    assert_traces::<DefinitionLhsPreparationSuspensionFrame>();
    assert_traces::<DefinitionLhsPreparationWork>();
    assert_traces::<DefinitionLhsRegion>();
    assert_traces::<DefinitionLhsStructuralCallState>();
    assert_traces::<DefinitionLhsStructuralSelection>();
    assert_traces::<DefinitionLhsStructuralSource>();
};
