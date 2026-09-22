//! Expand only in the private owner's test module. This keeps accounting
//! here while permitting direct field reads, without changing production
//! visibility or relying on parent-relative source paths in the inventory.

macro_rules! define_oracle {
    () => {
        use super::*;
        use crate::runtime::debug_tools::resources::occurrence::Bytes;
        use std::mem::size_of;

        impl AssociationLookupCache {
            pub(in crate::runtime) fn occurrence_bytes(&self) -> Bytes {
                let mut bytes = Bytes::Exact(self.entries.capacity())
                    .multiply(size_of::<AssociationLookupCacheEntry>());
                // Do not probe, remove stale entries, or upgrade weak witnesses here.
                for entry in &self.entries {
                    bytes = bytes
                        .add(
                            Bytes::Exact(entry.index.buckets.capacity()).multiply(size_of::<(
                                u64,
                                usize,
                            )>(
                            )),
                        )
                        .add(
                            Bytes::Exact(entry.index.next_collision.capacity())
                                .multiply(size_of::<usize>()),
                        );
                }
                bytes
            }

            pub(in crate::runtime) fn occurrence_layout() -> [(&'static str, usize); 4] {
                [
                    ("AssociationLookupCache (test)", size_of::<Self>()),
                    (
                        "AssociationLookupCacheEntry",
                        size_of::<AssociationLookupCacheEntry>(),
                    ),
                    ("AssociationKeyIndex", size_of::<AssociationKeyIndex>()),
                    ("AssociationWriteIndex", size_of::<AssociationWriteIndex>()),
                ]
            }

            /// Raw shape for exact formula tests, with no byte calculation.
            pub(in crate::runtime) fn occurrence_capacities(&self) -> (usize, Vec<(usize, usize)>) {
                (
                    self.entries.capacity(),
                    self.entries
                        .iter()
                        .map(|entry| {
                            (
                                entry.index.buckets.capacity(),
                                entry.index.next_collision.capacity(),
                            )
                        })
                        .collect(),
                )
            }
        }
    };
}

pub(in crate::runtime) use define_oracle;
