use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod guarded;
mod guarded_plan;
mod parsing;
mod plan;
mod plan_application;
mod replace_at;
mod replace_part;
mod suspended_state;

use suspended_state::{
    FrozenReplaceSequence, PersistentReplacePartTargets, SuspendedGuardedTargetList,
};

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct ReplacePartRule {
    position_specs: Vec<ReplacePartPositionSpec>,
    rhs: Expr,
    guards: Box<[Expr]>,
    delayed: bool,
    heads: ReplacePartHeadsMode,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
enum ReplacePartPositionSpec {
    Exact(Vec<PartSelector>),
    Pattern(Expr),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ReplacePlanNode {
    action: Option<ReplacePlanAction>,
    children: BTreeMap<isize, ReplacePlanNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReplacePlanAction {
    Immediate(Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum ReplacePartHeadsMode {
    Automatic,
    Include,
    Exclude,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct ReplacePartTarget {
    path: Vec<isize>,
    rhs: Expr,
    delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct GuardedReplacePartTarget {
    path: Vec<isize>,
    rule_index: usize,
    bindings: SymbolExprMap,
    option_scope: Option<OptionValueScope>,
    state: GuardedReplacePartTargetState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum GuardedReplacePartTargetState {
    PatternCandidate,
    PendingGuard,
    Accepted,
    Inactive,
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<GuardedReplacePartTarget>();
    assert_traces::<GuardedReplacePartTargetState>();
    assert_traces::<ReplacePartHeadsMode>();
    assert_traces::<ReplacePartPositionSpec>();
    assert_traces::<ReplacePartRule>();
    assert_traces::<ReplacePartTarget>();
};
