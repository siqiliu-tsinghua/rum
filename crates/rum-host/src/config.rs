use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use rum_runtime::{
    Capability, CapabilityPolicy, EntropyProvider, FileIoProvider, ResourceLimits, SessionConfig,
    kernel_api::Session,
};

use crate::{StdEntropySource, StdFileIo};

/// Cloneable installer for a downstream builtin family.
///
/// Installers receive the runtime kernel [`Session`] and may register builtin
/// specs through the public kernel API. Host sessions apply installers in
/// stored order after the optional core builtin installation whenever a session
/// is built or reset.
#[derive(Clone)]
pub struct BuiltinInstaller {
    inner: Arc<dyn Fn(&mut Session) + Send + Sync + 'static>,
}

impl BuiltinInstaller {
    /// Wraps a builtin-family installer callback.
    pub fn new(installer: impl Fn(&mut Session) + Send + Sync + 'static) -> Self {
        Self {
            inner: Arc::new(installer),
        }
    }

    /// Wraps a shared builtin-family installer callback.
    pub fn from_arc(installer: Arc<dyn Fn(&mut Session) + Send + Sync + 'static>) -> Self {
        Self { inner: installer }
    }

    pub(crate) fn install(&self, session: &mut Session) {
        (self.inner)(session);
    }
}

impl fmt::Debug for BuiltinInstaller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BuiltinInstaller")
    }
}

impl PartialEq for BuiltinInstaller {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for BuiltinInstaller {}

/// Resource limits applied by a host session.
///
/// These limits are copied into the runtime session configuration before each
/// session is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostResourceLimits {
    /// Maximum evaluator recursion depth.
    pub recursion_limit: usize,
    /// Maximum native stack bytes the evaluator may consume from request start.
    ///
    /// The default is derived from a minimum supported declared stack of 2 MiB,
    /// with 512 KiB reserved for poll headroom and 512 KiB for non-evaluator
    /// frames. Hosts that provision a larger worker stack can derive its budget
    /// with [`native_stack_limit_for_declared_stack`](crate::native_stack_limit_for_declared_stack).
    /// This is a declared-stack contract; the host does not discover or verify
    /// the current thread's actual stack bounds.
    pub native_stack_limit: usize,
    /// Maximum number of `ReplaceRepeated` iterations.
    pub replace_repeated_limit: usize,
    /// Maximum number of evaluation passes for one request.
    pub evaluation_pass_limit: usize,
    /// Optional per-request ceiling for modeled memory proxy bytes, not process RSS.
    ///
    /// Nested `MemoryConstrained` frames min-combine with this ceiling. A breach
    /// inside such a frame is handled in-band by that frame (`$Aborted` or its
    /// fallback), and the request ceiling remains armed outside the frame.
    pub eval_memory_limit: Option<usize>,
    /// Maximum number of generated elements a request may produce.
    pub generated_element_limit: usize,
    /// Maximum integer work budget for exact integer operations.
    pub integer_work_limit: usize,
    /// Maximum iteration work budget for iterative builtins.
    pub iteration_work_limit: usize,
}

impl Default for HostResourceLimits {
    fn default() -> Self {
        ResourceLimits::default().into()
    }
}

impl From<ResourceLimits> for HostResourceLimits {
    fn from(limits: ResourceLimits) -> Self {
        Self {
            recursion_limit: limits.recursion_limit,
            native_stack_limit: limits.native_stack_limit,
            replace_repeated_limit: limits.replace_repeated_limit,
            evaluation_pass_limit: limits.evaluation_pass_limit,
            eval_memory_limit: limits.eval_memory_limit,
            generated_element_limit: limits.generated_element_limit,
            integer_work_limit: limits.integer_work_limit,
            iteration_work_limit: limits.iteration_work_limit,
        }
    }
}

impl From<HostResourceLimits> for ResourceLimits {
    fn from(limits: HostResourceLimits) -> Self {
        Self {
            recursion_limit: limits.recursion_limit,
            native_stack_limit: limits.native_stack_limit,
            replace_repeated_limit: limits.replace_repeated_limit,
            evaluation_pass_limit: limits.evaluation_pass_limit,
            eval_memory_limit: limits.eval_memory_limit,
            generated_element_limit: limits.generated_element_limit,
            integer_work_limit: limits.integer_work_limit,
            iteration_work_limit: limits.iteration_work_limit,
        }
    }
}

/// Host-facing session configuration.
///
/// `HostConfig` is the stable embedding configuration surface for resource
/// limits, capability policy, context state, file I/O and entropy providers,
/// tracing, assertions, builtin-family installers, and whether core builtins
/// are installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostConfig {
    /// Resource limits installed into the runtime session.
    pub resource_limits: HostResourceLimits,
    /// Capability policy checked by capability-gated builtins.
    pub capability_policy: CapabilityPolicy,
    /// Current context used when evaluating source.
    pub current_context: String,
    /// Context path searched when resolving unqualified symbols.
    pub context_path: Vec<String>,
    /// Package search roots used by package-loading requests.
    pub package_search_paths: Vec<PathBuf>,
    /// File I/O provider used by `Get`, `Needs`, `FindFile`, `Save`, and `DumpSave`.
    pub file_io: Option<FileIoProvider>,
    /// Entropy provider used by `ReseedRandom[]`, `RandomBytes`, and `RandomPassword`.
    pub entropy_source: Option<EntropyProvider>,
    /// Whether evaluation tracing is enabled.
    pub tracing_enabled: bool,
    /// Whether runtime assertion checks are enabled.
    pub asserts_enabled: bool,
    /// Whether `rum-stdlib` core builtins are installed when the session is built.
    pub install_core_builtins: bool,
    /// Downstream builtin-family installers applied after the optional core install.
    pub builtin_installers: Vec<BuiltinInstaller>,
}

impl Default for HostConfig {
    fn default() -> Self {
        let config = SessionConfig::default();

        Self {
            resource_limits: config.resource_limits.into(),
            capability_policy: config.capability_policy,
            current_context: config.current_context,
            context_path: config.context_path,
            package_search_paths: config.package_search_paths,
            file_io: Some(StdFileIo::provider()),
            entropy_source: Some(StdEntropySource::provider()),
            tracing_enabled: config.tracing_enabled,
            asserts_enabled: config.asserts_enabled,
            install_core_builtins: true,
            builtin_installers: Vec::new(),
        }
    }
}

impl HostConfig {
    /// Returns a configuration for untrusted source.
    ///
    /// The sandboxed configuration installs a deterministic `{Pure}` capability
    /// policy, omits file I/O and entropy providers, and otherwise uses the
    /// default host configuration.
    pub fn sandboxed() -> Self {
        Self::default()
            .with_capability_policy(CapabilityPolicy::allow([Capability::Pure]))
            .without_file_io()
            .without_entropy_source()
    }

    /// Returns a deterministic configuration for build-step and compile-time evaluation.
    ///
    /// For the same input, a hermetic session produces the same output across
    /// sessions and resets. The fixed-seed deterministic PRNG remains available,
    /// so `SeedRandom[seed]` and `RandomInteger` work and replay identically. The
    /// profile exposes no wall clock, host entropy, file I/O, process control,
    /// interactive suspension, or diagnostics.
    ///
    /// This currently shares its implementation with [`Self::sandboxed`], whose
    /// framing is evaluation of untrusted source. They are separate named
    /// contracts and may diverge in the future without changing either promise.
    pub fn hermetic() -> Self {
        Self::sandboxed()
    }

    /// Returns the trusted local-scripting configuration used by command-line hosts.
    ///
    /// This profile has full ambient authority: an unrestricted capability policy,
    /// standard file I/O and host entropy providers, and the core builtins. It is
    /// the shape used by `rum-cli` and `rum-repl`. Never evaluate untrusted code in
    /// a scripting session; use [`Self::sandboxed`] or [`Self::hermetic`] instead.
    ///
    /// `HostConfig::default()` defines this profile, keeping the existing trusted
    /// default and the named scripting contract identical.
    pub fn scripting() -> Self {
        Self::default()
    }

    /// Replaces the resource limits for sessions built from this configuration.
    pub fn with_resource_limits(mut self, limits: HostResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// Replaces the capability policy for sessions built from this configuration.
    pub fn with_capability_policy(mut self, policy: CapabilityPolicy) -> Self {
        self.capability_policy = policy;
        self
    }

    /// Sets the current context used for source evaluation.
    pub fn with_current_context(mut self, context: impl Into<String>) -> Self {
        self.current_context = context.into();
        self
    }

    /// Sets the context path used for unqualified symbol lookup.
    pub fn with_context_path<I, S>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.context_path = contexts.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the package search paths used by package-loading requests.
    pub fn with_package_search_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.package_search_paths = paths.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the file I/O provider used by package loading and session persistence.
    pub fn with_file_io(mut self, provider: impl Into<Option<FileIoProvider>>) -> Self {
        self.file_io = provider.into();
        self
    }

    /// Removes the file I/O provider from sessions built from this configuration.
    pub fn without_file_io(mut self) -> Self {
        self.file_io = None;
        self
    }

    /// Sets the entropy provider used by no-argument random seeding and random bytes.
    pub fn with_entropy_source(mut self, provider: impl Into<Option<EntropyProvider>>) -> Self {
        self.entropy_source = provider.into();
        self
    }

    /// Removes the entropy provider from sessions built from this configuration.
    pub fn without_entropy_source(mut self) -> Self {
        self.entropy_source = None;
        self
    }

    /// Enables or disables collection of evaluation trace events.
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.tracing_enabled = enabled;
        self
    }

    /// Enables or disables runtime assertion checks.
    pub fn with_asserts_enabled(mut self, enabled: bool) -> Self {
        self.asserts_enabled = enabled;
        self
    }

    /// Enables or disables installation of the standard core builtins.
    pub fn with_core_builtins(mut self, enabled: bool) -> Self {
        self.install_core_builtins = enabled;
        self
    }

    /// Adds a downstream builtin-family installer to sessions built from this configuration.
    ///
    /// Installers are cloned with the configuration, applied in insertion order
    /// after the optional core builtin installation, and re-applied by
    /// [`crate::HostSession::reset`]. Snapshot images capture user definitions
    /// only; installer-provided builtin handlers come from the restored
    /// session's configuration.
    pub fn with_builtin_installer(
        mut self,
        installer: impl Fn(&mut Session) + Send + Sync + 'static,
    ) -> Self {
        self.builtin_installers
            .push(BuiltinInstaller::new(installer));
        self
    }

    pub(crate) fn runtime_config(&self) -> SessionConfig {
        SessionConfig::default()
            .with_resource_limits(self.resource_limits.into())
            .with_capability_policy(self.capability_policy)
            .with_current_context(self.current_context.clone())
            .with_context_path(self.context_path.clone())
            .with_package_search_paths(self.package_search_paths.clone())
            .with_tracing(self.tracing_enabled)
            .with_asserts_enabled(self.asserts_enabled)
    }
}
