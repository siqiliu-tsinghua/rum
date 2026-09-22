use super::*;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;

use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

const TAGGED_VALUE_STORE_INDEX_THRESHOLD: usize = 32;
const NO_TAGGED_VALUE_INDEX: usize = usize::MAX;

#[derive(Debug, Default, FrameRootTrace)]
pub(super) struct TaggedValueStoreMetadata {
    up_value_index: Option<Box<TaggedValueStoreIndex>>,
    sub_value_index: Option<Box<TaggedValueStoreIndex>>,
}

impl Clone for TaggedValueStoreMetadata {
    fn clone(&self) -> Self {
        // These indexes are derived from the value tables. Definition snapshots
        // rebuild them lazily instead of duplicating their retained expressions.
        Self::default()
    }
}

impl PartialEq for TaggedValueStoreMetadata {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for TaggedValueStoreMetadata {}

#[derive(Debug, FrameRootTrace)]
struct TaggedValueStoreIndex {
    hash_builder: RandomState,
    first_rule_by_equivalence_hash: HashMap<u64, usize>,
    next_equivalence_collision: Vec<usize>,
}

#[derive(Debug)]
pub(in crate::runtime) struct TaggedValueStoreLookup {
    equivalent_index: Option<usize>,
    equivalence_hash: Option<u64>,
}

impl TaggedValueStoreLookup {
    pub(in crate::runtime) fn equivalent_index(&self) -> Option<usize> {
        self.equivalent_index
    }
}

impl SymbolDefinition {
    pub(in crate::runtime) fn prepare_up_value_store(
        &mut self,
        lhs: &Expr,
    ) -> TaggedValueStoreLookup {
        if self.up_values.len() < TAGGED_VALUE_STORE_INDEX_THRESHOLD {
            return TaggedValueStoreLookup {
                equivalent_index: self
                    .up_values
                    .iter()
                    .position(|entry| lhs_patterns_equivalent(&entry.lhs, lhs)),
                equivalence_hash: None,
            };
        }

        let metadata = self
            .down_value_dispatch
            .get_or_insert_with(|| Box::new(DownValueDispatchMetadata::default()));
        metadata
            .dispatch_index
            .tagged_value_stores_mut(&self.down_values)
            .prepare_up_value_store(&self.up_values, lhs)
    }

    pub(in crate::runtime) fn prepare_sub_value_store(
        &mut self,
        lhs: &Expr,
    ) -> TaggedValueStoreLookup {
        if self.sub_values.len() < TAGGED_VALUE_STORE_INDEX_THRESHOLD {
            return TaggedValueStoreLookup {
                equivalent_index: self
                    .sub_values
                    .iter()
                    .position(|entry| lhs_patterns_equivalent(&entry.lhs, lhs)),
                equivalence_hash: None,
            };
        }

        let metadata = self
            .down_value_dispatch
            .get_or_insert_with(|| Box::new(DownValueDispatchMetadata::default()));
        metadata
            .dispatch_index
            .tagged_value_stores_mut(&self.down_values)
            .prepare_sub_value_store(&self.sub_values, lhs)
    }

    pub(in crate::runtime) fn note_up_value_stored(
        &mut self,
        store_index: usize,
        replaced: bool,
        lookup: TaggedValueStoreLookup,
    ) {
        let Some(metadata) = self.down_value_dispatch.as_mut() else {
            return;
        };
        if replaced {
            return;
        }

        let Some(tagged_value_stores) =
            metadata.dispatch_index.tagged_value_stores_mut_if_present()
        else {
            return;
        };
        if store_index.saturating_add(1) == self.up_values.len() {
            if let (Some(index), Some(equivalence_hash)) = (
                tagged_value_stores.up_value_index.as_mut(),
                lookup.equivalence_hash,
            ) {
                index.append(store_index, equivalence_hash);
            }
        } else {
            tagged_value_stores.up_value_index = None;
        }
        self.prune_dispatch_metadata_if_unused();
    }

    pub(in crate::runtime) fn note_sub_value_stored(
        &mut self,
        store_index: usize,
        replaced: bool,
        lookup: TaggedValueStoreLookup,
    ) {
        let Some(metadata) = self.down_value_dispatch.as_mut() else {
            return;
        };
        if replaced {
            return;
        }

        let Some(tagged_value_stores) =
            metadata.dispatch_index.tagged_value_stores_mut_if_present()
        else {
            return;
        };
        if store_index.saturating_add(1) == self.sub_values.len() {
            if let (Some(index), Some(equivalence_hash)) = (
                tagged_value_stores.sub_value_index.as_mut(),
                lookup.equivalence_hash,
            ) {
                index.append(store_index, equivalence_hash);
            }
        } else {
            tagged_value_stores.sub_value_index = None;
        }
        self.prune_dispatch_metadata_if_unused();
    }

    pub(in crate::runtime) fn note_up_values_rebuilt(&mut self) {
        if let Some(metadata) = self.down_value_dispatch.as_mut() {
            metadata
                .dispatch_index
                .clear_up_value_store(self.down_values.is_empty());
        }
        self.prune_dispatch_metadata_if_unused();
    }

    pub(in crate::runtime) fn note_sub_values_rebuilt(&mut self) {
        if let Some(metadata) = self.down_value_dispatch.as_mut() {
            metadata
                .dispatch_index
                .clear_sub_value_store(self.down_values.is_empty());
        }
        self.prune_dispatch_metadata_if_unused();
    }
}

impl TaggedValueStoreMetadata {
    pub(super) fn clear_up_value_store(&mut self) {
        self.up_value_index = None;
    }

    pub(super) fn clear_sub_value_store(&mut self) {
        self.sub_value_index = None;
    }

    fn prepare_up_value_store(
        &mut self,
        entries: &[UpValue],
        lhs: &Expr,
    ) -> TaggedValueStoreLookup {
        let index = self.up_value_index.get_or_insert_with(|| {
            Box::new(TaggedValueStoreIndex::new(
                entries.iter().map(|entry| &entry.lhs),
                entries.len(),
            ))
        });
        index.prepare_lookup(lhs, |index| entries.get(index).map(|entry| &entry.lhs))
    }

    fn prepare_sub_value_store(
        &mut self,
        entries: &[SubValue],
        lhs: &Expr,
    ) -> TaggedValueStoreLookup {
        let index = self.sub_value_index.get_or_insert_with(|| {
            Box::new(TaggedValueStoreIndex::new(
                entries.iter().map(|entry| &entry.lhs),
                entries.len(),
            ))
        });
        index.prepare_lookup(lhs, |index| entries.get(index).map(|entry| &entry.lhs))
    }

    pub(super) fn is_empty(&self) -> bool {
        self.up_value_index.is_none() && self.sub_value_index.is_none()
    }

    pub(super) fn memory_proxy_bytes(&self) -> usize {
        tagged_value_store_index_memory_proxy_bytes(self.up_value_index.as_deref()).saturating_add(
            tagged_value_store_index_memory_proxy_bytes(self.sub_value_index.as_deref()),
        )
    }
}

impl TaggedValueStoreIndex {
    fn new<'a>(lhs_entries: impl Iterator<Item = &'a Expr>, len: usize) -> Self {
        let hash_builder = RandomState::new();
        let mut first_rule_by_equivalence_hash = HashMap::with_capacity(len);
        let mut next_equivalence_collision = Vec::with_capacity(len);
        for (index, lhs) in lhs_entries.enumerate() {
            let hash = hash_builder.hash_one(canonicalize_pattern_names(lhs));
            let previous = first_rule_by_equivalence_hash
                .insert(hash, index)
                .unwrap_or(NO_TAGGED_VALUE_INDEX);
            next_equivalence_collision.push(previous);
        }
        Self {
            hash_builder,
            first_rule_by_equivalence_hash,
            next_equivalence_collision,
        }
    }

    fn prepare_lookup<'a>(
        &self,
        lhs: &Expr,
        mut lhs_at: impl FnMut(usize) -> Option<&'a Expr>,
    ) -> TaggedValueStoreLookup {
        let canonical_lhs = canonicalize_pattern_names(lhs);
        let equivalence_hash = self.hash_builder.hash_one(&canonical_lhs);
        let mut index = self
            .first_rule_by_equivalence_hash
            .get(&equivalence_hash)
            .copied()
            .unwrap_or(NO_TAGGED_VALUE_INDEX);
        while index != NO_TAGGED_VALUE_INDEX {
            let Some(stored_lhs) = lhs_at(index) else {
                break;
            };
            if canonicalize_pattern_names(stored_lhs) == canonical_lhs {
                return TaggedValueStoreLookup {
                    equivalent_index: Some(index),
                    equivalence_hash: Some(equivalence_hash),
                };
            }
            index = self
                .next_equivalence_collision
                .get(index)
                .copied()
                .unwrap_or(NO_TAGGED_VALUE_INDEX);
        }
        TaggedValueStoreLookup {
            equivalent_index: None,
            equivalence_hash: Some(equivalence_hash),
        }
    }

    fn append(&mut self, index: usize, equivalence_hash: u64) {
        debug_assert_eq!(index, self.next_equivalence_collision.len());
        let previous = self
            .first_rule_by_equivalence_hash
            .insert(equivalence_hash, index)
            .unwrap_or(NO_TAGGED_VALUE_INDEX);
        self.next_equivalence_collision.push(previous);
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
    }
}

fn tagged_value_store_index_memory_proxy_bytes(index: Option<&TaggedValueStoreIndex>) -> usize {
    index.map_or(0, |index| {
        std::mem::size_of::<TaggedValueStoreIndex>().saturating_add(index.memory_proxy_bytes())
    })
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<TaggedValueStoreIndex>();
    assert_traces::<TaggedValueStoreMetadata>();
};

// Test-only oracle access; keep production inventory source locations unchanged.
#[cfg(test)]
mod occurrence_memory {
    crate::runtime::debug_tools::resources::occurrence::tagged::define_oracle!();
}
