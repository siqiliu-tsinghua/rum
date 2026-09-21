use super::plan::insert_replace_plan_target;
use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenReplaceSequence<T: FrameRootTrace>(Arc<Vec<T>>);

impl<T: FrameRootTrace> FrozenReplaceSequence<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        Self(Arc::new(values))
    }

    pub(super) fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentReplacePartTargets {
    prefix: Arc<Vec<ReplacePartTarget>>,
    tail: AccumulatorChain<PersistentReplacePartTargetNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentReplacePartTargetNode {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentReplacePartTargetNode>,
    values: Arc<Vec<ReplacePartTarget>>,
}

impl PersistentReplacePartTargets {
    pub(super) fn freeze(values: Vec<ReplacePartTarget>) -> Self {
        let len = values.len();
        Self {
            prefix: Arc::new(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    pub(super) fn appended_values(&self, values: Vec<ReplacePartTarget>) -> Self {
        if values.is_empty() {
            return self.clone();
        }
        let added = values.len();
        Self {
            prefix: self.prefix.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PersistentReplacePartTargetNode {
                previous: self.tail.clone(),
                values: Arc::new(values),
            }))),
            len: self.len + added,
        }
    }

    pub(super) fn appended(&self, value: ReplacePartTarget) -> Self {
        self.appended_values(vec![value])
    }

    pub(super) fn into_vec(self) -> Vec<ReplacePartTarget> {
        let mut suffix = Vec::new();
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            let current = Arc::try_unwrap(current).unwrap_or_else(|_| {
                panic!("completed ReplacePart target chain must be uniquely owned")
            });
            let values = Arc::try_unwrap(current.values).unwrap_or_else(|_| {
                panic!("completed ReplacePart target chunk must be uniquely owned")
            });
            suffix.push(values);
            node = current.previous.into_inner();
        }
        suffix.reverse();
        let mut values = Arc::try_unwrap(self.prefix).unwrap_or_else(|_| {
            panic!("completed ReplacePart target prefix must be uniquely owned")
        });
        values.reserve(self.len.saturating_sub(values.len()));
        for chunk in suffix {
            values.extend(chunk);
        }
        values
    }
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct GuardedTargetChunk {
    #[accumulator_link]
    previous: AccumulatorChain<GuardedTargetChunk>,
    start: usize,
    values: Arc<[GuardedReplacePartTarget]>,
}

#[derive(Clone, FrameRootTrace)]
struct GuardedTargetUpdate {
    state: GuardedReplacePartTargetState,
    bindings: Option<FrozenMatchBindings>,
    option_scope: Option<OptionValueScope>,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct GuardedTargetUpdateNode {
    #[accumulator_link]
    previous: AccumulatorChain<GuardedTargetUpdateNode>,
    index: usize,
    update: GuardedTargetUpdate,
}

/// Immutable target storage plus an owned delta table for the suspended-only
/// guarded ReplacePart path. Ordinary selection still reads and writes its
/// Vec directly; this overlay therefore adds no probes to the normal path.
#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedGuardedTargetList {
    prefix: Arc<[GuardedReplacePartTarget]>,
    tail: AccumulatorChain<GuardedTargetChunk>,
    len: usize,
    updates: AccumulatorChain<GuardedTargetUpdateNode>,
}

impl SuspendedGuardedTargetList {
    pub(super) fn freeze(mut values: Vec<GuardedReplacePartTarget>) -> Self {
        let len = values.len();
        let mut updates = AccumulatorChain::default();
        for (index, target) in values.iter_mut().enumerate() {
            if !target.bindings.is_empty() {
                updates = AccumulatorChain::new(Some(Arc::new(GuardedTargetUpdateNode {
                    previous: updates,
                    index,
                    update: GuardedTargetUpdate {
                        state: target.state,
                        bindings: Some(FrozenMatchBindings::freeze(std::mem::take(
                            &mut target.bindings,
                        ))),
                        option_scope: target.option_scope.take(),
                    },
                })));
            }
        }
        Self {
            prefix: Arc::from(values),
            tail: AccumulatorChain::default(),
            len,
            updates,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    fn base(&self, index: usize) -> &GuardedReplacePartTarget {
        if index < self.prefix.len() {
            return &self.prefix[index];
        }
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            if index >= current.start && index < current.start + current.values.len() {
                return &current.values[index - current.start];
            }
            node = current.previous.as_deref();
        }
        panic!("guarded ReplacePart target index must exist");
    }

    pub(super) fn path(&self, index: usize) -> &[isize] {
        &self.base(index).path
    }

    pub(super) fn rule_index(&self, index: usize) -> usize {
        self.base(index).rule_index
    }

    pub(super) fn state(&self, index: usize) -> GuardedReplacePartTargetState {
        self.update(index)
            .map(|update| update.state)
            .unwrap_or_else(|| self.base(index).state)
    }

    pub(super) fn bindings(&self, index: usize) -> &SymbolExprMap {
        self.update(index)
            .and_then(|update| update.bindings.as_ref())
            .map(FrozenMatchBindings::as_map)
            .unwrap_or(&self.base(index).bindings)
    }

    pub(super) fn frozen_bindings(&self, index: usize) -> FrozenMatchBindings {
        if let Some(bindings) = self
            .update(index)
            .and_then(|update| update.bindings.as_ref())
        {
            return bindings.clone();
        }
        debug_assert!(self.base(index).bindings.is_empty());
        FrozenMatchBindings::freeze(SymbolExprMap::default())
    }

    pub(super) fn option_scope(&self, index: usize) -> Option<OptionValueScope> {
        self.update(index)
            .and_then(|update| update.option_scope.clone())
            .or_else(|| self.base(index).option_scope.clone())
    }

    pub(super) fn set_state(&mut self, index: usize, state: GuardedReplacePartTargetState) {
        let prior = self.update(index).cloned();
        self.push_update(
            index,
            GuardedTargetUpdate {
                state,
                bindings: prior.as_ref().and_then(|update| update.bindings.clone()),
                option_scope: prior.and_then(|update| update.option_scope),
            },
        );
    }

    pub(super) fn activate_pattern_target(
        &mut self,
        index: usize,
        bindings: SymbolExprMap,
        option_scope: Option<OptionValueScope>,
    ) {
        let rule_index = self.rule_index(index);
        let path = self.path(index).to_vec();
        if let Some(previous) = (0..index).rev().find(|previous| {
            self.rule_index(*previous) == rule_index
                && self.path(*previous) == path
                && self.state(*previous) == GuardedReplacePartTargetState::PendingGuard
        }) {
            self.set_state(previous, GuardedReplacePartTargetState::Inactive);
        }
        self.push_update(
            index,
            GuardedTargetUpdate {
                state: GuardedReplacePartTargetState::PendingGuard,
                bindings: Some(FrozenMatchBindings::freeze(bindings)),
                option_scope,
            },
        );
    }

    fn update(&self, index: usize) -> Option<&GuardedTargetUpdate> {
        let mut node = self.updates.as_deref();
        while let Some(current) = node {
            if current.index == index {
                return Some(&current.update);
            }
            node = current.previous.as_deref();
        }
        None
    }

    fn push_update(&mut self, index: usize, update: GuardedTargetUpdate) {
        self.updates = AccumulatorChain::new(Some(Arc::new(GuardedTargetUpdateNode {
            previous: self.updates.clone(),
            index,
            update,
        })));
    }

    pub(super) fn append_exact_targets(&mut self, mut values: Vec<GuardedReplacePartTarget>) {
        let start = self.len;
        for offset in 0..values.len() {
            let index = start + offset;
            let rule_index = values[offset].rule_index;
            let path = values[offset].path.clone();
            if let Some(previous) = (0..index).rev().find(|previous| {
                let (previous_rule, previous_path, previous_state) = if *previous >= start {
                    let previous = &values[*previous - start];
                    (
                        previous.rule_index,
                        previous.path.as_slice(),
                        previous.state,
                    )
                } else {
                    (
                        self.rule_index(*previous),
                        self.path(*previous),
                        self.state(*previous),
                    )
                };
                previous_rule == rule_index
                    && previous_path == path
                    && previous_state == GuardedReplacePartTargetState::PendingGuard
            }) {
                if previous >= start {
                    values[previous - start].state = GuardedReplacePartTargetState::Inactive;
                } else {
                    self.set_state(previous, GuardedReplacePartTargetState::Inactive);
                }
            }
        }
        self.append_chunk(values);
    }

    pub(super) fn append_pattern_candidates(
        &mut self,
        values: Vec<GuardedReplacePartTarget>,
    ) -> std::ops::Range<usize> {
        let start = self.len;
        self.append_chunk(values);
        start..self.len
    }

    fn append_chunk(&mut self, values: Vec<GuardedReplacePartTarget>) {
        if values.is_empty() {
            return;
        }
        let start = self.len;
        self.len += values.len();
        self.tail = AccumulatorChain::new(Some(Arc::new(GuardedTargetChunk {
            previous: self.tail.clone(),
            start,
            values: Arc::from(values),
        })));
    }

    pub(super) fn build_plan(&self, rules: &[ReplacePartRule]) -> ReplacePlanNode {
        let mut root = ReplacePlanNode::default();
        for index in 0..self.len {
            if self.state(index) != GuardedReplacePartTargetState::Accepted {
                continue;
            }
            let rule = &rules[self.rule_index(index)];
            insert_replace_plan_target(
                &mut root,
                self.path(index),
                ReplacePlanAction::Immediate(substitute_rule_bindings(
                    &rule.rhs,
                    self.bindings(index),
                )),
            );
        }
        root
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FrozenReplaceSequence<Expr>>();
    assert_traces::<GuardedTargetChunk>();
    assert_traces::<GuardedTargetUpdate>();
    assert_traces::<GuardedTargetUpdateNode>();
    assert_traces::<SuspendedGuardedTargetList>();
    assert_traces::<PersistentReplacePartTargetNode>();
    assert_traces::<PersistentReplacePartTargets>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abandoning_guarded_target_branch_preserves_surviving_chunks_and_updates() {
        let key = Symbol::from_name("x");
        let mut bindings = SymbolExprMap::default();
        bindings.insert(key, Expr::integer(7.into()));
        let target = GuardedReplacePartTarget {
            path: vec![1],
            rule_index: 0,
            bindings: bindings.clone(),
            option_scope: None,
            state: GuardedReplacePartTargetState::PendingGuard,
        };
        let mut survivor = SuspendedGuardedTargetList::freeze(vec![target.clone()]);
        survivor.append_pattern_candidates(vec![GuardedReplacePartTarget {
            path: vec![2],
            bindings: SymbolExprMap::default(),
            state: GuardedReplacePartTargetState::PatternCandidate,
            ..target.clone()
        }]);
        survivor.set_state(0, GuardedReplacePartTargetState::Accepted);
        let mut abandoned = survivor.clone();
        abandoned.set_state(0, GuardedReplacePartTargetState::Inactive);
        abandoned.append_exact_targets(vec![target]);
        drop(abandoned);

        assert_eq!(survivor.len(), 2);
        assert_eq!(survivor.path(0), &[1]);
        assert_eq!(survivor.path(1), &[2]);
        assert_eq!(survivor.state(0), GuardedReplacePartTargetState::Accepted);
        assert_eq!(survivor.bindings(0), &bindings);
        assert_eq!(
            survivor.state(1),
            GuardedReplacePartTargetState::PatternCandidate
        );
    }
}
