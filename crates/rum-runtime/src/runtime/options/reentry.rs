use super::*;

/// Typed dispatch data for the operation that consumes merged options.
///
/// Implementations must re-enter the consumer with the supplied, already
/// evaluated options. The kernel freezes the call only when an option suspends;
/// every captured expression is traversed by the option-loop frame's derive.
/// A descriptor carries no opaque closure captures.
#[doc = include_str!("reentry.md")]
pub trait OptionReentry: Copy + FrameRootTrace + Send + Sync + 'static {
    fn symbol_name(self) -> &'static str;

    fn resume(
        self,
        session: &mut Session,
        head: &Expr,
        args: &[Expr],
        options: Vec<Expr>,
    ) -> Result<Expr, EvalError>;
}
