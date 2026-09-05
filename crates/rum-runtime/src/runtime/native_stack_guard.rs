use std::cell::Cell;

use super::{EvalError, NATIVE_STACK_DEPTH_POLL_STRIDE, Session};

struct NativeStackGuard {
    base: usize,
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
        let consumed = self.base.saturating_sub(current_addr);
        if consumed > self.limit {
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
    let limit = 0;
    #[cfg(not(feature = "test-native-stack-guard-zero"))]
    let limit = session.resource_limits.native_stack_limit;

    let guard = NativeStackGuard {
        base: session.native_stack_base,
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
