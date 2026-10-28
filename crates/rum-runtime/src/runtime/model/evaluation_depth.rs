use super::NATIVE_STACK_DEPTH_POLL_STRIDE;

/// The evaluator's logical depth and the depth at which it next checks native
/// stack headroom. The threshold is derived from the depth, so no code outside
/// this type can move one without the other. `Copy`: saving and restoring the
/// whole value keeps a coherent pair.
#[derive(Clone, Copy, Debug)]
pub(in crate::runtime) struct EvaluationDepth {
    depth: usize,
    next_native_check: usize,
}

impl EvaluationDepth {
    pub(in crate::runtime) const ROOT: Self = Self {
        depth: 0,
        next_native_check: NATIVE_STACK_DEPTH_POLL_STRIDE,
    };

    #[inline(always)]
    pub(in crate::runtime) fn get(self) -> usize {
        self.depth
    }

    /// Every move other than the evaluator's own level entry and exit: a
    /// continuation restore, an abandonment reset, a guest-form restore, a
    /// rollback. The threshold becomes `depth + STRIDE`.
    #[inline]
    pub(in crate::runtime) fn restore(&mut self, depth: usize) {
        self.depth = depth;
        self.next_native_check = depth.saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE);
    }

    /// `depth` is this level's entry depth, read once by the evaluator.
    /// Today's rule on that depth: `depth >= next` then `next = depth + STRIDE`.
    #[inline(always)]
    pub(in crate::runtime) fn native_check_due(&mut self, depth: usize) -> bool {
        debug_assert_eq!(depth, self.depth);
        if depth >= self.next_native_check {
            self.next_native_check = depth.saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE);
            true
        } else {
            false
        }
    }

    /// `poll_resumed_native_stack`'s re-arm: threshold = depth + STRIDE.
    #[inline(always)]
    pub(in crate::runtime) fn rearm_native_check(&mut self) {
        self.next_native_check = self.depth.saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE);
    }

    /// The evaluator's level entry, `entered_at + 1`; the threshold is untouched.
    /// `entered_at` is this level's entry depth, read once by the evaluator.
    #[inline(always)]
    pub(in crate::runtime) fn enter_level(&mut self, entered_at: usize) {
        debug_assert_eq!(self.depth, entered_at);
        self.depth = entered_at + 1;
    }

    /// The evaluator's level exit, back to the depth the level entered at; the
    /// threshold keeps its high-water mark, as today. A debug assertion checks
    /// the depth is `entered_at + 1` here.
    #[inline(always)]
    pub(in crate::runtime) fn leave_level(&mut self, entered_at: usize) {
        debug_assert_eq!(self.depth, entered_at + 1);
        self.depth = entered_at;
    }
}
