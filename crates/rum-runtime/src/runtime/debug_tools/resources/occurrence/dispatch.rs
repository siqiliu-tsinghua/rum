//! Expand only in the private owner's test module. This keeps accounting
//! here while permitting direct field reads, without changing production
//! visibility or relying on parent-relative source paths in the inventory.

macro_rules! define_oracle {
    () => {
        use super::*;
        use crate::runtime::debug_tools::resources::occurrence::Bytes;
        use std::mem::size_of;

        impl SymbolDefinition {
            pub(in crate::runtime) fn occurrence_dispatch_bytes(&self) -> Bytes {
                let Some(metadata) = self.down_value_dispatch.as_ref() else {
                    return Bytes::Exact(0);
                };
                let mut bytes = Bytes::Exact(size_of::<DownValueDispatchMetadata>());
                // Historical omission: shallow_signatures and their nested args. Do not
                // silently add them while specifying the existing occurrence ceiling.
                if let Some(index) = metadata.dispatch_index.index.get() {
                    bytes = bytes
                        .add(Bytes::Exact(size_of::<DownValueDispatchIndex>()))
                        .add(
                            Bytes::Exact(index.first_rule_by_equivalence_hash.capacity())
                                .multiply(size_of::<(u64, usize)>()),
                        )
                        .add(
                            Bytes::Exact(index.next_equivalence_collision.capacity())
                                .multiply(size_of::<usize>()),
                        )
                        .add(
                            Bytes::Exact(index.non_ground_rule_indices.capacity())
                                .multiply(size_of::<usize>()),
                        )
                        .add(
                            Bytes::Exact(index.wrapped_ground_rule_indices.capacity())
                                .multiply(size_of::<usize>()),
                        )
                        .add(index.tagged_value_stores.occurrence_bytes());
                }
                bytes
            }

            pub(in crate::runtime) fn occurrence_dispatch_layout() -> Vec<(&'static str, usize)> {
                let mut sizes = vec![
                    (
                        "DownValueDispatchMetadata",
                        size_of::<DownValueDispatchMetadata>(),
                    ),
                    (
                        "DownValueDispatchIndexCache",
                        size_of::<DownValueDispatchIndexCache>(),
                    ),
                    (
                        "DownValueDispatchIndex",
                        size_of::<DownValueDispatchIndex>(),
                    ),
                    (
                        "DownValueShallowSignature",
                        size_of::<DownValueShallowSignature>(),
                    ),
                    ("LinearDownValueScan", size_of::<LinearDownValueScan>()),
                    ("IndexedDownValueScan", size_of::<IndexedDownValueScan>()),
                ];
                sizes.extend(TaggedValueStoreMetadata::occurrence_layout());
                sizes
            }
        }
    };
}

pub(in crate::runtime) use define_oracle;
