use super::*;

mod counting;
mod deletion;
mod projection;

pub(super) use counting::{recursive_delayed_wrapper_count, target_delayed_wrapper_count};
pub(super) use deletion::{contains_missing_within, delete_missing_from_keyed_data_recursive};
pub(super) use projection::projected_expr_memory_proxy_bytes;

const RECURSIVE_DELETE_MISSING_CALL_DEPTH: usize = 32;

enum BoundedTraversal<T> {
    Complete(T),
    Exhausted,
}
