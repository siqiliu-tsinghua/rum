//! Admission-only receipts. Neither the ledger's value nor a receipt's payload
//! is part of the Session memory proxy. The ledger allocation itself is.

use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::{Arc, Mutex};

#[derive(Default, rum_derive::FrameRootTrace)]
pub(in crate::runtime) struct RetentionLedger {
    state: Mutex<LedgerState>,
}

#[derive(rum_derive::FrameRootTrace)]
struct LedgerState {
    bytes: Option<usize>,
    live: usize,
    transfers: usize,
    releases: usize,
}

impl Default for LedgerState {
    fn default() -> Self {
        Self {
            bytes: Some(0),
            live: 0,
            transfers: 0,
            releases: 0,
        }
    }
}

/// Lease-ledger observations, separate from MemoryInUse/MaxMemoryUsed.
/// Admission additionally sees deferred unarmed bridge/frame contributions;
/// inspecting this ledger never allocates a receipt for those owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetentionSnapshot {
    pub bytes: Option<usize>,
    pub live: usize,
    pub transfers: usize,
    pub releases: usize,
}

impl RetentionLedger {
    pub(super) fn snapshot(&self) -> RetentionSnapshot {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        RetentionSnapshot {
            bytes: if self.state.is_poisoned() {
                None
            } else {
                state.bytes
            },
            live: state.live,
            transfers: state.transfers,
            releases: state.releases,
        }
    }
}

/// A linear receipt. Only the protocol can construct, resize or transfer one.
/// Drop releases its original ledger even after the originating Session dies.
#[derive(rum_derive::FrameRootTrace)]
pub struct RetentionLease {
    ledger: Arc<RetentionLedger>,
    bytes: Option<usize>,
}

impl RetentionLease {
    pub(super) fn new(session: &mut Session) -> Self {
        let route = session
            .accounting_route
            .get_or_insert_with(Default::default);
        let ledger = route.retention.get_or_insert_with(Default::default).clone();
        ledger.state.lock().unwrap_or_else(|e| e.into_inner()).live += 1;
        session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
        );
        Self {
            ledger,
            bytes: Some(0),
        }
    }

    pub(super) fn resize(&mut self, bytes: Option<usize>) {
        let mut state = self.ledger.state.lock().unwrap_or_else(|e| e.into_inner());
        state.bytes = state
            .bytes
            .and_then(|total| total.checked_sub(self.bytes?)?.checked_add(bytes?));
        self.bytes = bytes;
    }

    pub(super) fn transfer(&mut self, bytes: Option<usize>) {
        self.resize(bytes);
        self.ledger
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .transfers += 1;
    }
}

impl Drop for RetentionLease {
    fn drop(&mut self) {
        let mut state = self.ledger.state.lock().unwrap_or_else(|e| e.into_inner());
        state.bytes = state.bytes.and_then(|total| total.checked_sub(self.bytes?));
        state.live -= 1;
        state.releases += 1;
    }
}

mod sealed {
    pub trait Sealed {}
}

/// The existing explicit accounting seam: Unarmed has a zero-sized receipt.
pub trait RetentionMode: sealed::Sealed {
    type Receipt;
}

impl sealed::Sealed for Unarmed {}
impl sealed::Sealed for AccountingShadow {}
impl RetentionMode for Unarmed {
    type Receipt = ();
}
impl RetentionMode for AccountingShadow {
    type Receipt = RetentionLease;
}

pub(in crate::runtime) trait Mode: RetentionMode {
    type Options: super::options::buffer::Buffer;
    type OptionDomain: Copy;
    const ARMED: bool;
    fn option_domain(session: &Session) -> Self::OptionDomain;
    fn option_handoff<R: crate::runtime::options::OptionReentry>(
        session: &mut Session,
        generation: u64,
        domain: Self::OptionDomain,
    ) -> bool;
    fn receipt(session: &mut Session) -> Self::Receipt;
    fn bytes(receipt: &Self::Receipt) -> Option<usize>;
    fn resize(receipt: &mut Self::Receipt, bytes: impl FnOnce() -> Option<usize>);
    fn freeze(receipt: Self::Receipt, session: &mut Session) -> Option<RetentionLease>;
}

impl Mode for Unarmed {
    type Options = super::options::buffer::FlatBuffer;
    type OptionDomain = &'static [crate::runtime::options::OptionReentryDeclaration];
    const ARMED: bool = false;
    #[inline(always)]
    fn option_domain(session: &Session) -> Self::OptionDomain {
        session.certified_option_reentry_domain()
    }
    #[inline(always)]
    fn option_handoff<R: crate::runtime::options::OptionReentry>(
        session: &mut Session,
        generation: u64,
        domain: Self::OptionDomain,
    ) -> bool {
        // Entry/resume already checked R. The only domain installer registers
        // all members of this immutable slice, and registration only adds.
        // Identity as well as generation is needed: a replacement Session can
        // have the same generation but another installed domain.
        (generation == session.option_domain_generation()
            && std::ptr::eq(domain, session.certified_option_reentry_domain()))
            || session.check_option_resume::<R>(generation)
    }
    #[inline(always)]
    fn receipt(_: &mut Session) {}
    #[inline(always)]
    fn bytes(_: &()) -> Option<usize> {
        Some(0)
    }
    #[inline(always)]
    fn resize(_: &mut (), _: impl FnOnce() -> Option<usize>) {}
    fn freeze(_: (), session: &mut Session) -> Option<RetentionLease> {
        session
            .current_memory_limit
            .map(|_| RetentionLease::new(session))
    }
}

impl Mode for AccountingShadow {
    type Options = super::options::buffer::OptionBuffer;
    type OptionDomain = ();
    const ARMED: bool = true;
    #[inline(always)]
    fn option_domain(_: &Session) {}
    fn option_handoff<R: crate::runtime::options::OptionReentry>(
        session: &mut Session,
        generation: u64,
        _: (),
    ) -> bool {
        session.check_option_resume::<R>(generation)
    }
    fn receipt(session: &mut Session) -> RetentionLease {
        RetentionLease::new(session)
    }
    #[inline]
    fn bytes(receipt: &RetentionLease) -> Option<usize> {
        receipt.bytes
    }
    fn resize(receipt: &mut RetentionLease, bytes: impl FnOnce() -> Option<usize>) {
        receipt.resize(bytes());
    }
    fn freeze(receipt: RetentionLease, _: &mut Session) -> Option<RetentionLease> {
        Some(receipt)
    }
}

impl Session {
    pub fn retention_snapshot(&self) -> Option<RetentionSnapshot> {
        Some(
            self.accounting_route
                .as_ref()?
                .retention
                .as_ref()?
                .snapshot(),
        )
    }

    /// Called only inside the existing current_memory_limit branch.
    pub(in crate::runtime) fn retained_admission_bytes(&self) -> Option<usize> {
        let Some(route) = &self.accounting_route else {
            return Some(0);
        };
        let ledger = route
            .retention
            .as_ref()
            .map_or(Some(0), |r| r.snapshot().bytes)?;
        let bytes = route
            .prefixes
            .iter()
            .filter(|prefix| !prefix.charged)
            .try_fold(ledger, |sum, prefix| {
                sum.checked_add(prefix.values.bytes()?)
            })?;
        let mut capacity = FrozenCapacityVisitor(Some(bytes));
        if let Some(owner) = self.pending_suspension.as_ref() {
            owner.trace_frame_roots(&mut capacity);
        }
        if let Some(owner) = &self.resuming_suspension {
            owner.trace_frame_roots(&mut capacity);
        }
        for owner in &self.abandonment_worklist {
            owner.trace_frame_roots(&mut capacity);
        }
        capacity.0
    }
}

struct FrozenCapacityVisitor(Option<usize>);

impl FrameRootVisitor for FrozenCapacityVisitor {
    fn visit_expr(&mut self, _: &Expr) {}
    fn visit_symbol(&mut self, _: &Symbol) {}
    fn visit_retained_capacity(&mut self, spare_slots: usize) {
        self.0 = self
            .0
            .and_then(|n| n.checked_add(spare_slots.checked_mul(size_of::<Expr>())?));
    }
}

// No guest code is run under a lock. This structural implementation makes a
// newly added root in ledger/frozen control records a traversal obligation too.
impl<T: FrameRootTrace> FrameRootTrace for Mutex<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.lock()
            .unwrap_or_else(|error| error.into_inner())
            .trace_roots(visitor);
    }
}
