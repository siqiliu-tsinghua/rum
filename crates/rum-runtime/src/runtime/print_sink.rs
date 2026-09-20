use super::host_payload::BoundedDestruction;

/// A destination for `Print` output with declared bounded-stack destruction.
///
/// The sink runs on the evaluating thread. Implementors must declare
/// [`BoundedDestruction`] for their state, including reachable owned values.
pub trait PrintSink: BoundedDestruction + Send {
    /// Receives one printed line.
    fn print_line(&mut self, line: &str);
}

/// A closure whose captured state has a declared destruction bound.
///
/// Construct this adapter with [`Self::assert_bounded_destruction`]. The
/// assertion covers what the closure captures, including destruction by the
/// last strong holder of shared ownership; the compiler does not verify it.
pub struct ClosurePrintSink<F> {
    closure: F,
}

impl<F: FnMut(&str) + Send> ClosurePrintSink<F> {
    /// Asserts that destroying the closure and all state it captures uses
    /// bounded native stack space, independent of reachable ownership depth.
    ///
    /// This is the caller's [`BoundedDestruction`] declaration for the captured
    /// state, not a compiler-checked guarantee or a time bound.
    pub fn assert_bounded_destruction(closure: F) -> Self {
        Self { closure }
    }
}

// The private field can only be initialized through the constructor's explicit
// assertion about the closure and all captured state.
impl<F: FnMut(&str) + Send> BoundedDestruction for ClosurePrintSink<F> {}

impl<F: FnMut(&str) + Send> PrintSink for ClosurePrintSink<F> {
    fn print_line(&mut self, line: &str) {
        (self.closure)(line);
    }
}
