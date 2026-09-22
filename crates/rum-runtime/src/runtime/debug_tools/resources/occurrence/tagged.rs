//! Expand only in the private owner's test module. This keeps accounting
//! here while permitting direct field reads, without changing production
//! visibility or relying on parent-relative source paths in the inventory.

macro_rules! define_oracle {
    () => {
        use super::*;
        use crate::runtime::debug_tools::resources::occurrence::Bytes;
        use std::mem::size_of;

        impl TaggedValueStoreMetadata {
            pub(in crate::runtime) fn occurrence_bytes(&self) -> Bytes {
                let mut bytes = Bytes::Exact(0);
                for index in [self.up_value_index.as_ref(), self.sub_value_index.as_ref()]
                    .into_iter()
                    .flatten()
                {
                    bytes = bytes
                        .add(Bytes::Exact(size_of::<TaggedValueStoreIndex>()))
                        .add(
                            Bytes::Exact(index.first_rule_by_equivalence_hash.capacity())
                                .multiply(size_of::<(u64, usize)>()),
                        )
                        .add(
                            Bytes::Exact(index.next_equivalence_collision.capacity())
                                .multiply(size_of::<usize>()),
                        );
                }
                bytes
            }

            pub(in crate::runtime) fn occurrence_layout() -> [(&'static str, usize); 2] {
                [
                    ("TaggedValueStoreMetadata", size_of::<Self>()),
                    ("TaggedValueStoreIndex", size_of::<TaggedValueStoreIndex>()),
                ]
            }
        }
    };
}

pub(in crate::runtime) use define_oracle;
