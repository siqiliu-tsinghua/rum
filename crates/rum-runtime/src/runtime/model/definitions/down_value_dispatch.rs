use super::tagged_value_store::TaggedValueStoreMetadata;
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::sync::OnceLock;

const DOWN_VALUE_DISPATCH_INDEX_THRESHOLD: usize = 32;
const NO_DOWN_VALUE_INDEX: usize = usize::MAX;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct LinearDownValueScan {
    next_rule_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct IndexedDownValueScan {
    next_non_ground_position: usize,
    exact_rule_index: Option<usize>,
    next_linear_index: usize,
    fallback_rule_index: Option<usize>,
    evaluation_epoch: u64,
}

pub(in crate::runtime) trait DownValueScan:
    Copy + FrameRootTrace + Send + Sync + 'static
{
    fn next_rule_index(
        &mut self,
        definition: &SymbolDefinition,
        current_evaluation_epoch: u64,
    ) -> Option<usize>;
}

#[derive(Clone, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub(super) struct DownValueDispatchMetadata {
    shallow_signatures: Option<Vec<DownValueShallowSignature>>,
    ground_count: usize,
    pub(super) dispatch_index: DownValueDispatchIndexCache,
}

#[derive(Debug, Default, FrameRootTrace)]
pub(super) struct DownValueDispatchIndexCache {
    index: OnceLock<Box<DownValueDispatchIndex>>,
}

impl Clone for DownValueDispatchIndexCache {
    fn clone(&self) -> Self {
        // This is derived dispatch data. Definition snapshots and Block frames
        // should not duplicate it; the first wide lookup rebuilds it lazily.
        Self::default()
    }
}

impl PartialEq for DownValueDispatchIndexCache {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for DownValueDispatchIndexCache {}

#[derive(Clone, Debug, FrameRootTrace)]
struct DownValueDispatchIndex {
    hash_builder: RandomState,
    first_rule_by_equivalence_hash: HashMap<u64, usize>,
    next_equivalence_collision: Vec<usize>,
    non_ground_rule_indices: Vec<usize>,
    wrapped_ground_rule_indices: Vec<usize>,
    tagged_value_stores: TaggedValueStoreMetadata,
}

impl SymbolDefinition {
    pub(in crate::runtime) fn note_down_value_stored(
        &mut self,
        symbol: &Symbol,
        new_is_ground: bool,
        replaced_was_ground: Option<bool>,
        store_index: usize,
    ) {
        if self.down_values.len() == 1 && replaced_was_ground.is_none() {
            self.note_down_values_rebuilt(symbol);
            return;
        }
        if self.down_value_dispatch.is_none() {
            self.note_down_values_rebuilt(symbol);
            return;
        }

        let new_signature = DownValueShallowSignature::from_lhs(
            symbol,
            &self
                .down_values
                .get(store_index)
                .expect("stored DownValue index remains present")
                .lhs,
        );
        let appended = replaced_was_ground.is_none()
            && store_index.saturating_add(1) == self.down_values.len();
        let appended_down_value = appended.then(|| {
            self.down_values
                .get(store_index)
                .expect("appended DownValue index remains present")
        });
        let metadata = self
            .down_value_dispatch
            .as_mut()
            .expect("stored DownValue has dispatch metadata");

        match replaced_was_ground {
            Some(true) if !new_is_ground => {
                metadata.ground_count = metadata.ground_count.saturating_sub(1);
            }
            Some(false) if new_is_ground => {
                metadata.ground_count = metadata.ground_count.saturating_add(1);
            }
            None if new_is_ground => {
                metadata.ground_count = metadata.ground_count.saturating_add(1);
            }
            _ => {}
        }

        if let Some(signature) = new_signature {
            if let Some(signatures) = metadata.shallow_signatures.as_mut() {
                if replaced_was_ground.is_some() {
                    signatures[store_index] = signature;
                } else {
                    signatures.insert(store_index, signature);
                }
            }
        } else {
            metadata.shallow_signatures = None;
        }

        match replaced_was_ground {
            Some(was_ground) if was_ground == new_is_ground => {}
            None if appended => {
                if let (Some(index), Some(down_value)) =
                    (metadata.dispatch_index.get_mut(), appended_down_value)
                {
                    index.append(store_index, down_value);
                }
            }
            _ => metadata
                .dispatch_index
                .rebuild_down_values_preserving_tags(&self.down_values),
        }
    }

    pub(in crate::runtime) fn note_down_values_rebuilt(&mut self, symbol: &Symbol) {
        if self.down_values.is_empty() {
            if let Some(metadata) = self.down_value_dispatch.as_mut() {
                metadata.shallow_signatures = None;
                metadata.ground_count = 0;
                metadata
                    .dispatch_index
                    .rebuild_down_values_preserving_tags(&self.down_values);
            }
            self.prune_dispatch_metadata_if_unused();
            return;
        }

        let ground_count = self
            .down_values
            .iter()
            .filter(|down_value| down_value.lhs_is_ground)
            .count();
        let shallow_signatures = self.collect_down_value_shallow_signatures(symbol);
        let metadata = self
            .down_value_dispatch
            .get_or_insert_with(|| Box::new(DownValueDispatchMetadata::default()));
        metadata.ground_count = ground_count;
        metadata
            .dispatch_index
            .rebuild_down_values_preserving_tags(&self.down_values);
        metadata.shallow_signatures = shallow_signatures;
    }

    fn collect_down_value_shallow_signatures(
        &self,
        symbol: &Symbol,
    ) -> Option<Vec<DownValueShallowSignature>> {
        let mut signatures = Vec::with_capacity(self.down_values.len());
        for down_value in &self.down_values {
            let signature = DownValueShallowSignature::from_lhs(symbol, &down_value.lhs)?;
            signatures.push(signature);
        }
        Some(signatures)
    }

    pub(in crate::runtime) fn down_value_shallow_signatures(
        &self,
    ) -> Option<&[DownValueShallowSignature]> {
        self.down_value_dispatch
            .as_ref()?
            .shallow_signatures
            .as_deref()
    }

    pub(in crate::runtime) fn has_ground_down_value_index(&self) -> bool {
        self.down_value_dispatch
            .as_ref()
            .is_some_and(|metadata| metadata.ground_count >= DOWN_VALUE_DISPATCH_INDEX_THRESHOLD)
    }

    pub(in crate::runtime) fn has_non_ground_down_values(&self) -> bool {
        self.down_value_dispatch
            .as_ref()
            .is_some_and(|metadata| metadata.ground_count < self.down_values.len())
    }

    pub(in crate::runtime) fn equivalent_down_value_index(
        &self,
        lhs: &Expr,
        guards: &[Expr],
        lhs_is_ground: bool,
    ) -> Option<usize> {
        if self.down_values.len() < DOWN_VALUE_DISPATCH_INDEX_THRESHOLD {
            return self.down_values.iter().position(|entry| {
                down_value_identity_equivalent(entry, lhs, guards, lhs_is_ground)
            });
        }

        let Some(metadata) = self.down_value_dispatch.as_ref() else {
            return self.down_values.iter().position(|entry| {
                down_value_identity_equivalent(entry, lhs, guards, lhs_is_ground)
            });
        };
        metadata
            .dispatch_index
            .get_or_init(&self.down_values)
            .equivalent_rule_index(&self.down_values, lhs, guards, lhs_is_ground)
    }

    pub(in crate::runtime) fn start_indexed_down_value_scan(
        &self,
        candidate: &Expr,
        evaluation_epoch: u64,
    ) -> Option<IndexedDownValueScan> {
        if !self.has_ground_down_value_index() {
            return None;
        }

        let metadata = self
            .down_value_dispatch
            .as_ref()
            .expect("indexed DownValue scan requires dispatch metadata");
        let index = metadata.dispatch_index.get_or_init(&self.down_values);
        Some(IndexedDownValueScan {
            next_non_ground_position: 0,
            exact_rule_index: index.exact_rule_index(&self.down_values, candidate),
            next_linear_index: 0,
            fallback_rule_index: None,
            evaluation_epoch,
        })
    }

    pub(in crate::runtime) fn down_value_dispatch_memory_proxy_bytes(&self) -> usize {
        self.down_value_dispatch.as_ref().map_or(0, |metadata| {
            std::mem::size_of::<DownValueDispatchMetadata>()
                .saturating_add(metadata.dispatch_index.memory_proxy_bytes())
        })
    }

    pub(super) fn prune_dispatch_metadata_if_unused(&mut self) {
        let unused = self.down_values.is_empty()
            && self
                .down_value_dispatch
                .as_ref()
                .is_some_and(|metadata| !metadata.dispatch_index.has_tagged_value_stores());
        if unused {
            self.down_value_dispatch = None;
        }
    }
}

impl DownValueScan for LinearDownValueScan {
    #[inline]
    fn next_rule_index(
        &mut self,
        definition: &SymbolDefinition,
        _current_evaluation_epoch: u64,
    ) -> Option<usize> {
        let index = self.next_rule_index;
        if index >= definition.down_values.len() {
            return None;
        }
        self.next_rule_index += 1;
        Some(index)
    }
}

impl DownValueScan for IndexedDownValueScan {
    fn next_rule_index(
        &mut self,
        definition: &SymbolDefinition,
        current_evaluation_epoch: u64,
    ) -> Option<usize> {
        if self.fallback_rule_index.is_none() && self.evaluation_epoch != current_evaluation_epoch {
            self.fallback_rule_index = Some(self.next_linear_index);
        }

        if let Some(next_rule_index) = self.fallback_rule_index {
            if next_rule_index >= definition.down_values.len() {
                return None;
            }
            self.fallback_rule_index = Some(next_rule_index.saturating_add(1));
            return Some(next_rule_index);
        }

        let index = definition
            .down_value_dispatch
            .as_ref()
            .expect("indexed DownValue cursor requires dispatch metadata")
            .dispatch_index
            .get()
            .expect("indexed DownValue cursor requires a live index");
        let next_non_ground = index
            .non_ground_rule_indices
            .get(self.next_non_ground_position)
            .copied();
        let selected = match (next_non_ground, self.exact_rule_index) {
            (Some(non_ground), Some(exact)) if exact < non_ground => {
                self.exact_rule_index = None;
                Some(exact)
            }
            (Some(non_ground), _) => {
                self.next_non_ground_position += 1;
                Some(non_ground)
            }
            (None, Some(exact)) => {
                self.exact_rule_index = None;
                Some(exact)
            }
            (None, None) => None,
        };
        if let Some(index) = selected {
            self.next_linear_index = index.saturating_add(1);
        }
        selected
    }
}

impl DownValueDispatchIndexCache {
    fn get_or_init(&self, down_values: &[DownValue]) -> &DownValueDispatchIndex {
        self.index
            .get_or_init(|| Box::new(DownValueDispatchIndex::new(down_values)))
            .as_ref()
    }

    fn get(&self) -> Option<&DownValueDispatchIndex> {
        self.index.get().map(Box::as_ref)
    }

    fn get_mut(&mut self) -> Option<&mut DownValueDispatchIndex> {
        self.index.get_mut().map(Box::as_mut)
    }

    pub(super) fn tagged_value_stores_mut(
        &mut self,
        down_values: &[DownValue],
    ) -> &mut TaggedValueStoreMetadata {
        self.index
            .get_or_init(|| Box::new(DownValueDispatchIndex::new(down_values)));
        &mut self
            .index
            .get_mut()
            .expect("initialized definition indexes remain present")
            .tagged_value_stores
    }

    pub(super) fn tagged_value_stores_mut_if_present(
        &mut self,
    ) -> Option<&mut TaggedValueStoreMetadata> {
        Some(&mut self.index.get_mut()?.tagged_value_stores)
    }

    pub(super) fn clear_up_value_store(&mut self, down_values_empty: bool) {
        let Some(index) = self.index.get_mut() else {
            return;
        };
        index.tagged_value_stores.clear_up_value_store();
        if down_values_empty && index.tagged_value_stores.is_empty() {
            self.index.take();
        }
    }

    pub(super) fn clear_sub_value_store(&mut self, down_values_empty: bool) {
        let Some(index) = self.index.get_mut() else {
            return;
        };
        index.tagged_value_stores.clear_sub_value_store();
        if down_values_empty && index.tagged_value_stores.is_empty() {
            self.index.take();
        }
    }

    pub(super) fn has_tagged_value_stores(&self) -> bool {
        self.index
            .get()
            .is_some_and(|index| !index.tagged_value_stores.is_empty())
    }

    fn rebuild_down_values_preserving_tags(&mut self, down_values: &[DownValue]) {
        let Some(mut previous) = self.index.take() else {
            return;
        };
        let tagged_value_stores = std::mem::take(&mut previous.tagged_value_stores);
        if tagged_value_stores.is_empty() {
            return;
        }

        let mut rebuilt = Box::new(DownValueDispatchIndex::new(down_values));
        rebuilt.tagged_value_stores = tagged_value_stores;
        self.index
            .set(rebuilt)
            .expect("cleared definition index cache accepts rebuilt index");
    }

    fn memory_proxy_bytes(&self) -> usize {
        self.get().map_or(0, |index| {
            std::mem::size_of::<DownValueDispatchIndex>().saturating_add(index.memory_proxy_bytes())
        })
    }
}

impl DownValueDispatchIndex {
    fn new(down_values: &[DownValue]) -> Self {
        let ground_count = down_values
            .iter()
            .filter(|down_value| down_value.lhs_is_ground)
            .count();
        let hash_builder = RandomState::new();
        let mut first_rule_by_equivalence_hash = HashMap::with_capacity(down_values.len());
        let mut next_equivalence_collision = Vec::with_capacity(down_values.len());
        let mut non_ground_rule_indices =
            Vec::with_capacity(down_values.len().saturating_sub(ground_count));
        let mut wrapped_ground_rule_indices = Vec::new();

        for (index, down_value) in down_values.iter().enumerate() {
            let hash = down_value_equivalence_hash(&hash_builder, down_value);
            let previous = first_rule_by_equivalence_hash
                .insert(hash, index)
                .unwrap_or(NO_DOWN_VALUE_INDEX);
            next_equivalence_collision.push(previous);

            if !down_value.lhs_is_ground {
                non_ground_rule_indices.push(index);
            } else if strip_hold_pattern(&down_value.lhs) != &down_value.lhs {
                wrapped_ground_rule_indices.push(index);
            }
        }

        Self {
            hash_builder,
            first_rule_by_equivalence_hash,
            next_equivalence_collision,
            non_ground_rule_indices,
            wrapped_ground_rule_indices,
            tagged_value_stores: TaggedValueStoreMetadata::default(),
        }
    }

    fn append(&mut self, index: usize, down_value: &DownValue) {
        debug_assert_eq!(index, self.next_equivalence_collision.len());
        let hash = down_value_equivalence_hash(&self.hash_builder, down_value);
        let previous = self
            .first_rule_by_equivalence_hash
            .insert(hash, index)
            .unwrap_or(NO_DOWN_VALUE_INDEX);
        self.next_equivalence_collision.push(previous);

        if !down_value.lhs_is_ground {
            self.non_ground_rule_indices.push(index);
        } else if strip_hold_pattern(&down_value.lhs) != &down_value.lhs {
            self.wrapped_ground_rule_indices.push(index);
        }
    }

    fn equivalent_rule_index(
        &self,
        down_values: &[DownValue],
        lhs: &Expr,
        guards: &[Expr],
        lhs_is_ground: bool,
    ) -> Option<usize> {
        let hash = down_value_lhs_equivalence_hash(&self.hash_builder, lhs, lhs_is_ground);
        let mut index = self.first_rule_by_equivalence_hash.get(&hash).copied()?;

        while index != NO_DOWN_VALUE_INDEX {
            let down_value = down_values.get(index)?;
            if down_value_identity_equivalent(down_value, lhs, guards, lhs_is_ground) {
                return Some(index);
            }
            index = *self.next_equivalence_collision.get(index)?;
        }
        None
    }

    fn exact_rule_index(&self, down_values: &[DownValue], candidate: &Expr) -> Option<usize> {
        let hash = self.hash_builder.hash_one(candidate);
        if let Some(mut index) = self.first_rule_by_equivalence_hash.get(&hash).copied() {
            while index != NO_DOWN_VALUE_INDEX {
                let down_value = down_values.get(index)?;
                if down_value.lhs_is_ground && strip_hold_pattern(&down_value.lhs) == candidate {
                    return Some(index);
                }
                index = *self.next_equivalence_collision.get(index)?;
            }
        }

        self.wrapped_ground_rule_indices
            .iter()
            .copied()
            .find(|index| {
                down_values.get(*index).is_some_and(|down_value| {
                    down_value.lhs_is_ground && strip_hold_pattern(&down_value.lhs) == candidate
                })
            })
    }

    fn memory_proxy_bytes(&self) -> usize {
        self.first_rule_by_equivalence_hash
            .capacity()
            .saturating_mul(std::mem::size_of::<(u64, usize)>())
            .saturating_add(
                self.next_equivalence_collision
                    .capacity()
                    .saturating_mul(std::mem::size_of::<usize>()),
            )
            .saturating_add(
                self.non_ground_rule_indices
                    .capacity()
                    .saturating_mul(std::mem::size_of::<usize>()),
            )
            .saturating_add(
                self.wrapped_ground_rule_indices
                    .capacity()
                    .saturating_mul(std::mem::size_of::<usize>()),
            )
            .saturating_add(self.tagged_value_stores.memory_proxy_bytes())
    }
}

fn down_value_equivalence_hash(hash_builder: &RandomState, down_value: &DownValue) -> u64 {
    down_value_lhs_equivalence_hash(hash_builder, &down_value.lhs, down_value.lhs_is_ground)
}

fn down_value_lhs_equivalence_hash(
    hash_builder: &RandomState,
    lhs: &Expr,
    lhs_is_ground: bool,
) -> u64 {
    if lhs_is_ground {
        hash_builder.hash_one(lhs)
    } else {
        hash_builder.hash_one(canonicalize_pattern_names(lhs))
    }
}

fn down_value_identity_equivalent(
    down_value: &DownValue,
    lhs: &Expr,
    guards: &[Expr],
    lhs_is_ground: bool,
) -> bool {
    down_value.lhs_is_ground == lhs_is_ground
        && if lhs_is_ground {
            down_value.guards.is_empty() && guards.is_empty() && &down_value.lhs == lhs
        } else {
            down_value_patterns_equivalent(&down_value.lhs, &down_value.guards, lhs, guards)
        }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<DownValueDispatchIndex>();
    assert_traces::<DownValueDispatchIndexCache>();
    assert_traces::<DownValueDispatchMetadata>();
};

// Test-only oracle access; keep production inventory source locations unchanged.
#[cfg(test)]
mod occurrence_memory {
    crate::runtime::debug_tools::resources::occurrence::dispatch::define_oracle!();
}
