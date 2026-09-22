//! Explicit accounting route with maintained Session owners and total.
//! Observation walks every owner; armed checkpoints read the maintained total.
//!
//! A legacy BuiltinFn cannot receive a typed mode. Only registrations through
//! register_accounting_builtin receive AccountingContext. Every other builtin
//! uses the legacy adapter, even if it declares Incremental. Recursive
//! legacy Session entries reselect a mode from this Session's active limit;
//! the cost of those selectors is part of the unarmed acceptance gate.

use super::*;
use std::mem::size_of;
use std::sync::atomic::Ordering;

mod continuation;
mod counter;

pub(in crate::runtime) use counter::{AccountingMode, Counter, Unarmed};
pub use counter::{AccountingShadow, AccountingSnapshot};

/// Additive companion to BuiltinFn. The existing spec and handler are unchanged.
pub type AccountingBuiltinFn =
    fn(&mut AccountingContext<'_>, &Expr, &SharedArgs) -> Result<Expr, EvalError>;

pub(in crate::runtime) struct AccountingRoute {
    pub(in crate::runtime) adapters: Vec<(Symbol, AccountingBuiltinFn)>,
    pub(in crate::runtime) shadow: Option<AccountingShadow>,
    // None means arithmetic invalidity, never inferred freshness. Rebuilt on arm.
    pub(in crate::runtime) evaluation_stack_bytes: Option<usize>,
    pub(in crate::runtime) owner_bytes: [Option<usize>; 53],
    pub(in crate::runtime) total_bytes: Option<usize>,
}

impl Default for AccountingRoute {
    fn default() -> Self {
        Self {
            adapters: Vec::new(),
            shadow: None,
            evaluation_stack_bytes: None,
            owner_bytes: [None; 53],
            total_bytes: None,
        }
    }
}

impl AccountingRoute {
    pub(in crate::runtime) fn memory_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(
                self.adapters
                    .capacity()
                    .saturating_mul(size_of::<(Symbol, AccountingBuiltinFn)>()),
            )
            .saturating_add(
                self.adapters
                    .iter()
                    .map(|(symbol, _)| symbol.to_string().len())
                    .sum::<usize>(),
            )
            .saturating_add(
                self.shadow
                    .as_ref()
                    .map_or(0, |_| size_of::<Counter>() + 2 * size_of::<usize>()),
            )
    }
}

/// Armed extension adapter. The callback receives the mode explicitly, and its
/// guest calls and named continuations retain the same session-owned handle.
/// Armed checkpoints use the maintained total; cold entry walks all owners.
pub struct AccountingContext<'a> {
    session: &'a mut Session,
    shadow: AccountingShadow,
}

impl AccountingContext<'_> {
    pub fn evaluate(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        // Restore the explicit frame/callback token at this legacy Session entry.
        // No global, TLS, or expression-pointer lookup participates in routing.
        self.session
            .accounting_route
            .as_mut()
            .expect("armed route")
            .shadow = Some(self.shadow.clone());
        self.session.evaluate(expr)
    }

    pub fn shadow(&self) -> AccountingShadow {
        self.shadow.clone()
    }

    /// Representative host mutation. The bare Session method remains supported
    /// and maintains the same owner ledger; this adapter also records a shadow event.
    pub fn install_system_own_value(&mut self, name: &str, value: Expr) {
        self.shadow.0.host_mutations.fetch_add(1, Ordering::Relaxed);
        self.session.install_system_own_value(name, value);
    }

    /// The single pilot's guest-evaluation and continuation boundary.
    pub fn compound_expression(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        self.compound_from(args, 0)
    }
}

impl Session {
    /// Register an explicit armed companion alongside an ordinary public spec.
    /// Unarmed evaluation calls spec.handler unchanged. Armed evaluation calls
    /// the companion. A later register_builtin for this name removes it and
    /// returns that registration to the legacy adapter. The historical shadow
    /// counter name `walker_fallbacks` counts these calls, not session walks.
    pub fn register_accounting_builtin(&mut self, spec: BuiltinSpec, handler: AccountingBuiltinFn) {
        self.register_builtin(spec);
        self.accounting_route
            .get_or_insert_with(Default::default)
            .adapters
            .push((Symbol::system(spec.name), handler));
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
        );
    }

    /// Returns the last shadow observation, including after a local limit exits.
    pub fn accounting_shadow(&self) -> Option<AccountingShadow> {
        self.accounting_route.as_ref()?.shadow.clone()
    }

    /// Explicit host entry, also valid while an armed request is suspended.
    /// None means no active memory limit. Legacy host mutation remains supported
    /// by the owner ledger and is outside this representative shadow coverage.
    pub fn accounting_context(&mut self) -> Option<AccountingContext<'_>> {
        self.current_memory_limit?;
        let shadow = self.ensure_accounting_shadow();
        Some(AccountingContext {
            session: self,
            shadow,
        })
    }

    pub(in crate::runtime) fn ensure_accounting_shadow(&mut self) -> AccountingShadow {
        let absent = self
            .accounting_route
            .as_ref()
            .is_none_or(|route| route.shadow.is_none());
        let shadow = self
            .accounting_route
            .get_or_insert_with(Default::default)
            .shadow
            .get_or_insert_with(AccountingShadow::default)
            .clone();
        if absent {
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
            );
        }
        shadow
    }

    pub(in crate::runtime) fn note_accounting_arm(&mut self) {
        self.ensure_accounting_shadow()
            .0
            .arms
            .fetch_add(1, Ordering::Relaxed);
        self.initialize_accounted_session_memory();
    }

    #[cold]
    pub(in crate::runtime) fn run_accounting_builtin(
        &mut self,
        legacy: BuiltinFn,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let shadow = self.ensure_accounting_shadow();
        let handler = expr_as_symbol(head).and_then(|symbol| {
            self.accounting_route
                .as_ref()?
                .adapters
                .iter()
                .find(|(registered, _)| registered == symbol)
                .map(|(_, handler)| *handler)
        });
        if let Some(handler) = handler {
            shadow.0.callbacks.fetch_add(1, Ordering::Relaxed);
            handler(
                &mut AccountingContext {
                    session: self,
                    shadow,
                },
                head,
                args,
            )
        } else {
            shadow.0.walker_fallbacks.fetch_add(1, Ordering::Relaxed);
            legacy(self, head, args)
        }
    }
}
