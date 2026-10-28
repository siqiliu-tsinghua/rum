use std::cell::Cell;
use std::marker::PhantomData;
use std::thread_local;

use super::{EvalError, NATIVE_STACK_DEPTH_POLL_STRIDE, Session};

thread_local! {
    /// Stack address of the outermost rum entry on this thread; 0 while no rum
    /// code runs here. A property of the thread, never of a Session, so a
    /// Session that moves threads cannot carry a stale one.
    static THREAD_NATIVE_STACK_BASE: Cell<usize> = const { Cell::new(0) };
}

/// Held for the extent of one rum entry on this thread. Nested entries (a
/// resume inside an evaluation, abandonment inside a request, a second Session
/// in a host callback) keep the outermost base. Not `Send`: it cannot leave the
/// thread it describes.
pub(in crate::runtime) struct NativeStackEntry {
    outermost: bool,
    _thread_bound: PhantomData<*const ()>,
}

impl NativeStackEntry {
    #[inline(always)]
    pub(in crate::runtime) fn enter() -> Self {
        let stack_marker = 0_u8;
        let marker_addr = (&stack_marker as *const u8) as usize;
        let outermost = THREAD_NATIVE_STACK_BASE.with(|cell| {
            if cell.get() == 0 {
                cell.set(marker_addr);
                true
            } else {
                false
            }
        });
        Self {
            outermost,
            _thread_bound: PhantomData,
        }
    }
}

impl Drop for NativeStackEntry {
    fn drop(&mut self) {
        if self.outermost {
            THREAD_NATIVE_STACK_BASE.with(|cell| cell.set(0));
        }
    }
}

/// Whether more than `limit` bytes of native stack are in use since this
/// thread's outermost entry. With no entry on this thread, that is an invariant
/// violation: a debug assertion fails, and a release build answers `true`
/// (fail closed).
#[inline]
pub(in crate::runtime) fn native_stack_exhausted(current: usize, limit: usize) -> bool {
    let base = THREAD_NATIVE_STACK_BASE.with(Cell::get);
    debug_assert_ne!(
        base, 0,
        "native stack check ran on a thread with no active NativeStackEntry"
    );
    base == 0 || base.saturating_sub(current) > limit
}

#[cfg(test)]
/// Runs `f` with this thread's record set to `base`, restoring the previous
/// value afterwards, also on unwind. For unit tests that drive the guard
/// without an entry.
pub(in crate::runtime) fn with_thread_native_stack_base<R>(
    base: usize,
    f: impl FnOnce() -> R,
) -> R {
    struct Restore(usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            THREAD_NATIVE_STACK_BASE.with(|cell| cell.set(self.0));
        }
    }
    let _restore = Restore(THREAD_NATIVE_STACK_BASE.with(|cell| cell.replace(base)));
    f()
}

struct NativeStackGuard {
    limit: usize,
    exhausted: Cell<bool>,
}

/// A scope-branded native-recursion depth token.
///
/// Tokens have no public constructor and can only produce a child token. They
/// deliberately do not implement `Clone` or the suspension-frame tracing
/// contract.
///
/// A token therefore cannot be stored in a suspension frame:
///
/// ```compile_fail
/// use rum_runtime::kernel_api::{FrameRootTrace, NativeStackDepth};
///
/// #[derive(FrameRootTrace)]
/// struct InvalidSuspensionFrame<'scope> {
///     depth: NativeStackDepth<'scope>,
/// }
/// ```
#[doc(hidden)]
pub struct NativeStackDepth<'scope> {
    guard: &'scope NativeStackGuard,
    checks_after_edges: usize,
}

impl<'scope> NativeStackDepth<'scope> {
    /// Produces the token for one native-depth-increasing child call.
    ///
    /// `None` means the byte budget was exhausted. Logical depth only selects
    /// when the cold byte check runs; it is never a refusal criterion.
    #[inline]
    pub fn child(&self) -> Option<Self> {
        if self.checks_after_edges > 1 {
            return Some(Self {
                guard: self.guard,
                checks_after_edges: self.checks_after_edges - 1,
            });
        }

        if !self.guard.poll_native_stack_limit() {
            return None;
        }

        Some(Self {
            guard: self.guard,
            checks_after_edges: NATIVE_STACK_DEPTH_POLL_STRIDE,
        })
    }
}

impl NativeStackGuard {
    #[cold]
    #[inline(never)]
    fn poll_native_stack_limit(&self) -> bool {
        let stack_marker = 0_u8;
        let current_addr = (&stack_marker as *const u8) as usize;
        if native_stack_exhausted(current_addr, self.limit) {
            self.exhausted.set(true);
            return false;
        }
        true
    }
}

/// Runs an operation inside a native-recursion guard scope.
///
/// The session's request-base address and native-stack limit are captured once.
/// The higher-ranked closure keeps every [`NativeStackDepth`] branded to this
/// invocation, so returning a token from the scope does not compile:
///
/// ```compile_fail
/// use rum_runtime::Session;
/// use rum_runtime::kernel_api::run_with_native_stack_guard;
///
/// let mut session = Session::new();
/// let escaped = run_with_native_stack_guard(&mut session, |depth| depth);
/// ```
#[doc(hidden)]
#[inline]
pub fn run_with_native_stack_guard<R>(
    session: &Session,
    operation: impl for<'scope> FnOnce(NativeStackDepth<'scope>) -> R,
) -> Result<R, EvalError> {
    #[cfg(feature = "test-native-stack-guard-zero")]
    let limit = {
        let _ = session;
        0
    };
    #[cfg(not(feature = "test-native-stack-guard-zero"))]
    let limit = session.resource_limits.native_stack_limit;

    let guard = NativeStackGuard {
        limit,
        exhausted: Cell::new(false),
    };
    let result = operation(NativeStackDepth {
        guard: &guard,
        checks_after_edges: NATIVE_STACK_DEPTH_POLL_STRIDE,
    });

    if guard.exhausted.get() {
        Err(EvalError::StackLimitExceeded { limit: guard.limit })
    } else {
        Ok(result)
    }
}
