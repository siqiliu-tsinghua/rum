use super::*;

use std::num::NonZeroU64;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

const IDLE: u64 = 0;
const CANCELLED_BIT: u64 = 1;
pub(in crate::runtime) const MAX_REQUEST_GENERATION: u64 = u64::MAX >> 1;

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
/// word's current cancellation bit, so completion clears its observation and
/// a later request starts uncancelled. If the session rebuilds or retires its
/// control block, observers of the old block become inert.
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
    // Invariant: 0 is idle; otherwise the word is
    // `(generation << 1) | cancelled`. A bare boolean cannot distinguish a
    // stale request from the current one, while an unconditional prologue clear
    // loses a current cancel. Every writer therefore transitions only its exact
    // generation, and completion closes and observes through the same word.
    phase: AtomicU64,
}

impl RequestInterruptState {
    pub(in crate::runtime) fn new() -> Self {
        Self {
            phase: AtomicU64::new(IDLE),
        }
    }

    pub(in crate::runtime) fn has_cancelled_request(&self) -> bool {
        self.phase.load(Ordering::Relaxed) & CANCELLED_BIT != 0
    }

    pub(in crate::runtime) fn is_idle(&self) -> bool {
        self.phase.load(Ordering::Relaxed) == IDLE
    }

    pub(in crate::runtime) fn try_reserve(&self, generation: NonZeroU64) -> bool {
        self.phase
            .compare_exchange(
                IDLE,
                encode(generation, false),
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    fn cancel(&self, generation: NonZeroU64) {
        let active = encode(generation, false);
        let cancelled = encode(generation, true);
        let _ =
            self.phase
                .compare_exchange(active, cancelled, Ordering::Relaxed, Ordering::Relaxed);
    }

    fn is_active(&self, generation: NonZeroU64) -> bool {
        matches!(
            self.phase.load(Ordering::Relaxed),
            value if value == encode(generation, false) || value == encode(generation, true)
        )
    }

    fn is_cancelled(&self, generation: NonZeroU64) -> bool {
        self.phase.load(Ordering::Relaxed) == encode(generation, true)
    }

    fn close_and_observe(&self, generation: NonZeroU64) -> bool {
        let active = encode(generation, false);
        let cancelled = encode(generation, true);

        match self
            .phase
            .compare_exchange(active, IDLE, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => false,
            Err(value) if value == cancelled => self
                .phase
                .compare_exchange(cancelled, IDLE, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok(),
            Err(_) => false,
        }
    }
}

fn encode(generation: NonZeroU64, cancelled: bool) -> u64 {
    debug_assert!(generation.get() <= MAX_REQUEST_GENERATION);
    (generation.get() << 1) | if cancelled { CANCELLED_BIT } else { 0 }
}
