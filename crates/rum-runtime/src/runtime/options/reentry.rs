use super::*;
use std::any::TypeId;

/// A downstream installer's declaration of one audited concrete option type.
///
/// This is a public trust seam, not an unforgeable certificate: external crates
/// can construct declarations too. Repository lint certificates cover only the
/// repository's reconciled declaration, implementation and feature configuration.
#[doc = include_str!("tests/outside.md")]
#[derive(Clone, Copy)]
pub struct OptionReentryDeclaration {
    type_id: fn() -> TypeId,
    pub(crate) key: &'static str,
}

impl OptionReentryDeclaration {
    pub const fn of<R: OptionReentry>(key: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<R>,
            key,
        }
    }

    pub fn identity(self) -> (TypeId, &'static str) {
        ((self.type_id)(), self.key)
    }
}

impl Session {
    /// Replace the certified option domain and register its concrete types.
    /// Existing option continuations expire even if a replacement has the same
    /// members. Plain `register_option_reentry` never adds domain membership.
    /// Calling this trust seam with another declaration leaves the repository's
    /// lint-certified build domain; it cannot mint a lint certificate.
    pub fn install_option_reentry_domain(&mut self, entries: &'static [OptionReentryDeclaration]) {
        for (index, entry) in entries.iter().enumerate() {
            assert!(!entry.key.is_empty(), "option domain key must be explicit");
            assert!(
                entries[..index]
                    .iter()
                    .all(|previous| previous.identity().0 != entry.identity().0
                        && previous.key != entry.key),
                "duplicate option domain member or key"
            );
        }
        let route = self.accounting_route.get_or_insert_with(Default::default);
        route.option_domain_generation = route
            .option_domain_generation
            .checked_add(1)
            .expect("option domain generation overflow");
        route.option_domain = entries;
        for entry in entries {
            if !route.option_reentries.contains(&entry.identity().0) {
                route.option_reentries.push(entry.identity().0);
            }
        }
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
        );
    }

    /// Read-only reconciliation of actual registration and certified membership.
    pub fn option_reentry_registration<R: OptionReentry>(&self) -> (bool, Option<&'static str>) {
        let Some(route) = self.accounting_route.as_ref() else {
            return (false, None);
        };
        let id = TypeId::of::<R>();
        (
            route.option_reentries.contains(&id),
            route
                .option_domain
                .iter()
                .find(|entry| entry.identity().0 == id)
                .map(|entry| entry.key),
        )
    }

    /// Actual registered concrete identities, for installer reconciliation.
    /// Registration alone does not confer certified-domain membership.
    pub fn registered_option_reentry_ids(&self) -> &[TypeId] {
        self.accounting_route
            .as_ref()
            .map_or(&[], |route| &route.option_reentries)
    }

    /// The installed trust declaration, distinct from plain registrations.
    pub fn certified_option_reentry_domain(&self) -> &'static [OptionReentryDeclaration] {
        self.accounting_route
            .as_ref()
            .map_or(&[], |route| route.option_domain)
    }

    pub(in crate::runtime) fn option_domain_generation(&self) -> u64 {
        self.accounting_route
            .as_ref()
            .map_or(0, |route| route.option_domain_generation)
    }

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
        let (registered, certified) = self.option_reentry_registration::<R>();
        let registered = registered && certified.is_some();
        if !registered {
            self.push_message(
                "General",
                "optreentry",
                "The option reentry type is not registered in the certified domain.",
            );
        }
        registered
    }

    pub(in crate::runtime) fn check_option_resume<R: OptionReentry>(
        &mut self,
        generation: u64,
    ) -> bool {
        if !self.check_option_reentry::<R>() {
            return false;
        }
        if generation != self.option_domain_generation() {
            self.push_message(
                "General",
                "optreentry",
                "The option reentry domain changed during suspension.",
            );
            return false;
        }
        true
    }
}

/// Typed dispatch data for the operation that consumes merged options.
///
/// Implementations must re-enter the consumer with the supplied, already
/// evaluated options. The kernel freezes the call only when an option suspends;
/// every captured expression is traversed by the option-loop frame's derive.
/// A descriptor carries no opaque closure captures.
/// Kernel entry and resume require registration and membership in the installed
/// certified domain. Installing a host declaration asserts runtime trust; it
/// does not acquire a repository lint certificate.
#[doc = include_str!("reentry.md")]
pub trait OptionReentry: Copy + FrameRootTrace + Send + Sync + 'static {
    fn symbol_name(self) -> &'static str;

    fn resume(
        self,
        session: &mut Session,
        head: &Expr,
        args: &[Expr],
        options: crate::runtime::accounting_mode::RetainedOptions<'_>,
    ) -> Result<Expr, EvalError>;
}
