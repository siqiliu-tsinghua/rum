use super::super::state::{LEXICAL_PROFILE_ACTIVE, LEXICAL_PROFILE_ENABLED};
use crate::runtime::profiling::lease::{self, TestLease};
use crate::runtime::profiling::logical::{Epoch, epoch, segments};
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub(in crate::runtime) struct Enabled {
    configured: u8,
    active: u8,
    lease: TestLease,
}
impl Enabled {
    pub(in crate::runtime) fn lease(&self) -> &TestLease {
        &self.lease
    }
}
pub(in crate::runtime) fn enabled() -> Enabled {
    let lease = lease::lease();
    epoch::reset_for_lease(&lease);
    Enabled {
        configured: LEXICAL_PROFILE_ENABLED.swap(1, Ordering::Relaxed),
        active: LEXICAL_PROFILE_ACTIVE.swap(1, Ordering::Relaxed),
        lease,
    }
}
impl Drop for Enabled {
    fn drop(&mut self) {
        epoch::reset_for_lease(&self.lease);
        LEXICAL_PROFILE_ENABLED.store(self.configured, Ordering::Relaxed);
        LEXICAL_PROFILE_ACTIVE.store(self.active, Ordering::Relaxed);
    }
}
pub(in crate::runtime) fn epoch() -> Arc<Epoch> {
    epoch::current().unwrap()
}
pub(in crate::runtime) fn limit(lease: &TestLease, bytes: usize) {
    epoch::test_limit(lease, bytes);
}
pub(in crate::runtime) fn active() -> usize {
    segments::active_count()
}
pub(in crate::runtime) use crate::runtime::profiling::lease::lease;
