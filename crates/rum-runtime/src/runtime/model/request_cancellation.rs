use super::*;

use std::num::NonZeroU64;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

const IDLE: u64 = 0;
const FLAG_MASK: u64 = 0b11;
const GENERATION_SHIFT: u32 = 2;
/// Reserved, not cancelled. The poll does not see this.
const ACTIVE: u64 = 0b00;
/// Cancelled, and the retirement has not consumed it. The poll sees this.
const CANCELLED: u64 = 0b01;
/// The retirement consumed the cancel. The poll does not see this.
const CONSUMED: u64 = 0b10;
/// A further cancel of the same generation arrived after consume. The poll sees this.
const ESCALATED: u64 = 0b11;
const UNCONSUMED_BIT: u64 = 0b01;
pub(in crate::runtime) const MAX_REQUEST_GENERATION: u64 = u64::MAX >> GENERATION_SHIFT;

/// The error returned when a session already has a reserved cancellable request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcceptRequestError {
    RequestAlreadyAccepted,
}

impl fmt::Display for AcceptRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestAlreadyAccepted => {
                formatter.write_str("the session already has an accepted cancellable request")
            }
        }
    }
}

impl std::error::Error for AcceptRequestError {}

/// A linear ticket reserving one cancellable request identity.
///
/// The ticket is intentionally neither `Clone` nor `Copy`. Dropping it before
/// a request entrypoint consumes it retires the identity and makes every
/// handle obtained from it inert.
pub struct AcceptedRequest {
    control: Arc<RequestInterruptState>,
    generation: NonZeroU64,
}

impl AcceptedRequest {
    pub(in crate::runtime) fn new(
        control: Arc<RequestInterruptState>,
        generation: NonZeroU64,
    ) -> Self {
        Self {
            control,
            generation,
        }
    }

    /// Returns a cloneable cancellation handle qualified to this request.
    pub fn cancel_handle(&self) -> RequestCancelHandle {
        RequestCancelHandle {
            control: Arc::clone(&self.control),
            generation: self.generation,
        }
    }

    pub(in crate::runtime) fn belongs_to(&self, control: &Arc<RequestInterruptState>) -> bool {
        Arc::ptr_eq(&self.control, control)
    }

    pub(in crate::runtime) fn is_reserved(&self) -> bool {
        self.control.is_active(self.generation)
    }

    pub(in crate::runtime) fn is_cancelled(&self) -> bool {
        self.control.is_cancelled(self.generation)
    }

    /// Records that a retirement has consumed this request's cancel.
    ///
    /// `is_cancelled` is then false until a later `cancel` of this same
    /// generation. A stale generation is left unchanged.
    pub(in crate::runtime) fn consume_cancellation(&self) {
        self.control.consume_cancellation(self.generation);
    }

    pub(in crate::runtime) fn finish(self) -> bool {
        self.control.close_and_observe(self.generation)
    }
}

impl fmt::Debug for AcceptedRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AcceptedRequest")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

impl Drop for AcceptedRequest {
    fn drop(&mut self) {
        self.control.close_and_observe(self.generation);
    }
}

/// A cloneable cancellation handle bound to exactly one accepted request.
#[derive(Clone)]
pub struct RequestCancelHandle {
    control: Arc<RequestInterruptState>,
    generation: NonZeroU64,
}

impl RequestCancelHandle {
    /// Requests cooperative cancellation of the accepted request that produced
    /// this handle. Calls after that request completes are inert.
    pub fn cancel(&self) {
        self.control.cancel(self.generation);
    }
}

impl fmt::Debug for RequestCancelHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestCancelHandle")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

/// A generation-agnostic observer of the currently running request.
///
/// The observer does not capture a request generation. It reports the phase
/// word's unconsumed cancel bit, so a retirement that has consumed its cancel
/// is not observed until a later cancel of that generation sets the bit again.
/// Completion clears the observation, and a later request starts uncancelled.
/// If the session rebuilds or retires its control block, observers of the old
/// block become inert.
#[derive(Clone)]
pub struct CancellationObserver {
    control: Arc<RequestInterruptState>,
}

impl CancellationObserver {
    pub(in crate::runtime) fn new(control: Arc<RequestInterruptState>) -> Self {
        Self { control }
    }

    /// Returns whether the request running on this control block is cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.control.has_cancelled_request()
    }
}

impl fmt::Debug for CancellationObserver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CancellationObserver")
            .finish_non_exhaustive()
    }
}

pub(in crate::runtime) struct RequestInterruptState {
    // Invariant: 0 is idle; otherwise the word is `(generation << 2) | flags`.
    // Flags are ACTIVE (0), CANCELLED (1), CONSUMED (2), or ESCALATED (3).
    // The low bit is the unconsumed cancel the poll observes. A bare boolean
    // cannot distinguish a stale request from the current one, and it cannot
    // record a cancel that arrives after a retirement consumed the first.
    // Every writer therefore transitions only its exact generation, and
    // completion closes and observes through the same word.
    phase: AtomicU64,
}

impl RequestInterruptState {
    /// Called only after the logical profiler's enabled gate. The retained Arc,
    /// together with this generation, identifies the owner across Session moves.
    pub(in crate::runtime) fn logical_generation(&self) -> u64 {
        self.phase.load(Ordering::Relaxed) >> GENERATION_SHIFT
    }

    pub(in crate::runtime) fn new() -> Self {
        Self {
            phase: AtomicU64::new(IDLE),
        }
    }

    pub(in crate::runtime) fn has_cancelled_request(&self) -> bool {
        self.phase.load(Ordering::Relaxed) & UNCONSUMED_BIT != 0
    }

    pub(in crate::runtime) fn is_idle(&self) -> bool {
        self.phase.load(Ordering::Relaxed) == IDLE
    }

    pub(in crate::runtime) fn try_reserve(&self, generation: NonZeroU64) -> bool {
        self.phase
            .compare_exchange(
                IDLE,
                encode(generation, ACTIVE),
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    fn cancel(&self, generation: NonZeroU64) {
        loop {
            let current = self.phase.load(Ordering::Relaxed);
            let Some(flags) = flags_of(current, generation) else {
                return;
            };
            // Setting the unconsumed bit takes ACTIVE to CANCELLED and
            // CONSUMED to ESCALATED. A word that already has it is unchanged.
            if flags & UNCONSUMED_BIT != 0 {
                return;
            }
            if self
                .phase
                .compare_exchange(
                    current,
                    encode(generation, flags | UNCONSUMED_BIT),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return;
            }
        }
    }

    fn consume_cancellation(&self, generation: NonZeroU64) {
        let _ = self.phase.compare_exchange(
            encode(generation, CANCELLED),
            encode(generation, CONSUMED),
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }

    fn is_active(&self, generation: NonZeroU64) -> bool {
        flags_of(self.phase.load(Ordering::Relaxed), generation).is_some()
    }

    fn is_cancelled(&self, generation: NonZeroU64) -> bool {
        matches!(
            flags_of(self.phase.load(Ordering::Relaxed), generation),
            Some(CANCELLED | ESCALATED)
        )
    }

    fn close_and_observe(&self, generation: NonZeroU64) -> bool {
        loop {
            let current = self.phase.load(Ordering::Relaxed);
            let Some(flags) = flags_of(current, generation) else {
                return false;
            };
            if self
                .phase
                .compare_exchange(current, IDLE, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return flags != ACTIVE;
            }
        }
    }
}

fn encode(generation: NonZeroU64, flags: u64) -> u64 {
    debug_assert!(generation.get() <= MAX_REQUEST_GENERATION);
    debug_assert!(flags <= FLAG_MASK);
    (generation.get() << GENERATION_SHIFT) | flags
}

fn flags_of(phase: u64, generation: NonZeroU64) -> Option<u64> {
    if phase >> GENERATION_SHIFT == generation.get() {
        Some(phase & FLAG_MASK)
    } else {
        None
    }
}
