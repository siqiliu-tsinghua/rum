use super::*;
use std::any::TypeId;

impl Session {
    /// Permit this concrete descriptor in the kernel's option loop. Registration
    /// is a host trust decision, not a proof of the implementation's behavior.
    /// Direct external calls to OptionReentry::resume are outside this guarantee.
    pub fn register_option_reentry<R: OptionReentry>(&mut self) {
        let route = self.accounting_route.get_or_insert_with(Default::default);
        let id = TypeId::of::<R>();
        if !route.option_reentries.contains(&id) {
            route.option_reentries.push(id);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
            );
        }
    }

    pub(super) fn check_option_reentry<R: OptionReentry>(&mut self) -> bool {
        let registered = self
            .accounting_route
            .as_ref()
            .is_some_and(|route| route.option_reentries.contains(&TypeId::of::<R>()));
        if !registered {
            self.push_message(
                "General",
                "optreentry",
                "The option reentry type is not registered.",
            );
        }
        registered
    }
}

/// Typed dispatch data for the operation that consumes merged options.
///
/// Implementations must re-enter the consumer with the supplied, already
/// evaluated options. The kernel freezes the call only when an option suspends;
/// every captured expression is traversed by the option-loop frame's derive.
/// A descriptor carries no opaque closure captures.
/// The kernel invokes only types registered with Session::register_option_reentry.
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
