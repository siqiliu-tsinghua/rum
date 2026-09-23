use super::*;

mod constraints;
mod maintained;
mod memory;
pub(in crate::runtime) use maintained::MemoryOwner;
pub(in crate::runtime) use memory::expr_memory_proxy_bytes as retained_expr_bytes;

#[cfg(test)]
pub(in crate::runtime) mod occurrence;
