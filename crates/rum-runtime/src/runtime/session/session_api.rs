mod resume_frames;
use resume_frames::*;

use super::super::*;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::Duration;

struct SnapshotRestoreMode<'a> {
    session: &'a mut Session,
}

/// A statically proven system downvalue identity.
///
/// The fields and general constructor stay private. Published constructors
/// name the exact catalog identities that may bypass lhs preparation, so a
/// downstream caller cannot turn an arbitrary expression into this proof.
#[doc(hidden)]
pub struct TrustedSystemDownValueLhs {
    symbol: Symbol,
    lhs: Expr,
}

impl TrustedSystemDownValueLhs {
    fn new(name: &str, lhs: Expr) -> Self {
        Self {
            symbol: Symbol::system(name),
            lhs,
        }
    }

    fn arithmetic_default(head: &'static str) -> Self {
        TrustedSystemDownValueLhs::new(
            "Default",
            Expr::call(
                Expr::system_symbol("Default"),
                vec![Expr::system_symbol(head)],
            ),
        )
    }

    #[doc(hidden)]
    pub fn default_for_plus() -> Self {
        Self::arithmetic_default("Plus")
    }

    #[doc(hidden)]
    pub fn default_for_times() -> Self {
        Self::arithmetic_default("Times")
    }

    pub(in crate::runtime) fn into_parts(self) -> (Symbol, Expr) {
        (self.symbol, self.lhs)
    }
}

impl Drop for SnapshotRestoreMode<'_> {
    fn drop(&mut self) {
        self.session.restoring_snapshot = false;
    }
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: SessionConfig) -> Self {
        let mut session = Self::empty();
        session.apply_config(config);
        session
    }

    pub fn eval_snapshot_restore_request(
        &mut self,
        source: &str,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        debug_assert!(!self.restoring_snapshot);
        self.restoring_snapshot = true;
        let restore_mode = SnapshotRestoreMode { session: self };
        let result = restore_mode.session.eval_script_request(source);
        drop(restore_mode);
        result
    }

    pub fn configuration(&self) -> SessionConfig {
        SessionConfig {
            resource_limits: self.resource_limits,
            capability_policy: self.capability_policy,
            current_context: self.current_context.clone(),
            context_path: self.context_path.clone(),
            package_search_paths: self.package_search_paths.clone(),
            tracing_enabled: self.tracing_enabled,
            asserts_enabled: self.asserts_enabled,
        }
    }

    pub fn apply_config(&mut self, config: SessionConfig) {
        let config = config.normalized();
        self.resource_limits = config.resource_limits;
        self.capability_policy = config.capability_policy;
        self.current_context = config.current_context;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::CurrentContext,
        );
        self.context_path = config.context_path;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
        self.package_search_paths = config.package_search_paths;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PackageSearchPaths,
        );
        self.tracing_enabled = config.tracing_enabled;
        self.asserts_enabled = config.asserts_enabled;
    }

    pub fn set_recursion_limit(&mut self, limit: usize) {
        self.resource_limits.recursion_limit = limit;
    }

    /// Sets the native-stack bytes evaluator descent may consume from its request base.
    ///
    /// This does not resize or discover the current thread's stack. Embedders
    /// that provision a worker stack should derive its matching budget with
    /// [`native_stack_limit_for_declared_stack`]; callers entering evaluation
    /// beneath substantial native frames should lower that budget.
    pub fn set_native_stack_limit(&mut self, limit: usize) {
        self.resource_limits.native_stack_limit = limit;
    }

    pub fn set_replace_repeated_limit(&mut self, limit: usize) {
        self.resource_limits.replace_repeated_limit = limit;
    }

    pub fn set_evaluation_pass_limit(&mut self, limit: usize) {
        self.resource_limits.evaluation_pass_limit = limit;
    }

    pub fn set_generated_element_limit(&mut self, limit: usize) {
        self.resource_limits.generated_element_limit = limit;
    }

    pub fn set_integer_work_limit(&mut self, limit: usize) {
        self.resource_limits.integer_work_limit = limit;
    }

    pub fn set_iteration_work_limit(&mut self, limit: usize) {
        self.resource_limits.iteration_work_limit = limit;
    }

    /// Reserves a unique identity for one cancellable request.
    pub fn accept_cancellable_request(&self) -> Result<AcceptedRequest, AcceptRequestError> {
        let mut control = self.request_interrupt_control.borrow_mut();
        if !control.is_idle() {
            return Err(AcceptRequestError::RequestAlreadyAccepted);
        }

        if self.next_request_generation.get() == 0 {
            *control = Arc::new(RequestInterruptState::new());
            self.next_request_generation.set(1);
        }

        let generation = NonZeroU64::new(self.next_request_generation.get())
            .expect("request generation zero is the exhaustion sentinel");
        if !control.try_reserve(generation) {
            return Err(AcceptRequestError::RequestAlreadyAccepted);
        }

        self.next_request_generation
            .set(if generation.get() == MAX_REQUEST_GENERATION {
                0
            } else {
                generation.get() + 1
            });

        Ok(AcceptedRequest::new(Arc::clone(&control), generation))
    }

    /// Returns a generation-agnostic observer of request cancellation.
    pub fn cancellation_observer(&self) -> CancellationObserver {
        CancellationObserver::new(Arc::clone(&self.request_interrupt_control.borrow()))
    }

    /// Returns a qualified handle for the active request, or reserves the next request.
    pub fn request_cancel_handle(&self) -> RequestCancelHandle {
        if let Some(active) = self.active_accepted_request.as_ref() {
            return active.cancel_handle();
        }
        if let Some(pending) = self.pending_accepted_request.borrow().as_ref() {
            return pending.cancel_handle();
        }

        let accepted = self
            .accept_cancellable_request()
            .expect("an idle session should reserve its next cancellable request");
        let handle = accepted.cancel_handle();
        *self.pending_accepted_request.borrow_mut() = Some(accepted);
        handle
    }

    pub fn set_eval_timeout(&mut self, timeout: Option<Duration>) {
        self.eval_timeout = timeout;
        self.eval_deadline = None;
        self.interrupt_armed =
            self.eval_timeout.is_some() || self.active_accepted_request.is_some();
    }

    pub fn set_resource_limits(&mut self, limits: ResourceLimits) {
        self.resource_limits = limits;
    }

    pub fn resource_limits(&self) -> ResourceLimits {
        self.resource_limits
    }

    /// Returns the active dynamic memory limit, if one is installed.
    pub fn active_memory_limit(&self) -> Option<usize> {
        self.current_memory_limit
    }

    pub fn set_capability_policy(&mut self, policy: CapabilityPolicy) {
        self.capability_policy = policy;
    }

    pub fn capability_policy(&self) -> CapabilityPolicy {
        self.capability_policy
    }

    pub fn enable_tracing(&mut self, enabled: bool) {
        self.tracing_enabled = enabled;
    }

    pub fn enable_asserts(&mut self, enabled: bool) {
        self.asserts_enabled = enabled;
    }

    pub fn asserts_enabled(&self) -> bool {
        self.asserts_enabled
    }

    pub fn trace(&self) -> &[TraceEvent] {
        &self.trace_events
    }

    pub fn take_trace(&mut self) -> Vec<TraceEvent> {
        self.trace_event_depths.clear();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::TraceEventDepths,
        );
        let taken = std::mem::take(&mut self.trace_events);
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::TraceEvents);
        taken
    }

    pub fn messages(&self) -> &[RuntimeMessage] {
        &self.messages
    }

    pub fn take_messages(&mut self) -> Vec<RuntimeMessage> {
        let taken = std::mem::take(&mut self.messages);
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Messages);
        taken
    }

    pub fn prints(&self) -> &[String] {
        &self.prints
    }

    pub fn take_prints(&mut self) -> Vec<String> {
        let taken = std::mem::take(&mut self.prints);
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Prints);
        taken
    }

    pub fn pending_input_request(&self) -> Option<&RuntimeInputRequest> {
        self.pending_input_request.as_ref()
    }

    pub fn take_pending_input_request(&mut self) -> Option<RuntimeInputRequest> {
        let request = self.pending_input_request.take();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
        );
        request
    }

    pub fn queue_input_response(&mut self, value: impl Into<String>) {
        self.queued_input_responses.push_back(value.into());
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::QueuedInputResponses,
        );
        self.pending_input_request = None;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
        );
    }

    pub fn register_builtin(&mut self, spec: BuiltinSpec) {
        let symbol = Symbol::system(spec.name);
        if let Some(route) = &mut self.accounting_route {
            route
                .adapters
                .retain(|(registered, _)| *registered != symbol);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::AccountingRoute,
            );
        }
        let spec = self
            .builtins
            .get(&symbol)
            .copied()
            .map(|fallback| spec.with_fallback_dispatch(fallback))
            .unwrap_or(spec);
        let replaced = self.builtins.insert(symbol, spec);
        if replaced.is_some() {
            self.release_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::Builtins,
                std::mem::size_of::<(Symbol, BuiltinSpec)>(),
            );
        }
        self.charge_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::Builtins,
            std::mem::size_of::<(Symbol, BuiltinSpec)>(),
        );
        self.builtin_dispatch_spec_cache.clear();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::BuiltinDispatchSpecCache,
        );

        if !spec.attributes.is_empty() {
            self.attributes.insert(symbol, spec.attributes);
            self.bump_attribute_revision();
        } else {
            self.bump_evaluation_epoch();
        }
    }

    #[doc(hidden)]
    pub fn install_system_own_value(&mut self, name: &str, value: Expr) {
        let symbol = Symbol::system(name);
        let definition = self.definitions.entry(symbol).or_default();
        definition.store_own_value(value, Box::default(), false);
        self.installed_system_own_values.insert(symbol);
        self.bump_evaluation_epoch_for_symbol(&symbol);
    }

    #[doc(hidden)]
    pub fn install_system_down_value(&mut self, lhs: TrustedSystemDownValueLhs, rhs: Expr) {
        let symbol = lhs.symbol;
        let installed_lhs = lhs.lhs.clone();
        self.store_trusted_system_down_value(lhs, rhs.clone());
        self.installed_system_down_values
            .entry(symbol)
            .or_default()
            .push((installed_lhs, rhs));
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::InstalledSystemDownValues,
        );
    }

    #[doc(hidden)]
    pub fn registered_builtin_specs(&self) -> impl Iterator<Item = &BuiltinSpec> {
        self.builtins.values()
    }

    pub fn current_context(&self) -> &str {
        &self.current_context
    }

    pub fn context_path(&self) -> &[String] {
        &self.context_path
    }

    pub fn definition_summaries(&self) -> Vec<SessionDefinitionSummary> {
        let mut summaries = self
            .definitions
            .iter()
            .filter(|(symbol, _)| {
                !self.is_installed_protected_builtin_own_value_symbol(symbol)
                    && !self.is_installed_system_down_values_symbol(symbol)
            })
            .map(|(symbol, definition)| SessionDefinitionSummary {
                name: symbol.fully_qualified_name(),
                has_own_value: definition.own_value.is_some()
                    || definition.delayed_own_value.is_some(),
                down_value_count: definition.down_values.len(),
                up_value_count: definition.up_values.len(),
                sub_value_count: definition.sub_values.len(),
            })
            .collect::<Vec<_>>();

        summaries.sort_by(|left, right| left.name.cmp(&right.name));
        summaries
    }

    /// Renders this session's WL-level user definitions as textual dump source.
    ///
    /// The image captures own values, delayed own values, down values, up values,
    /// sub values, and attributes for symbols that have definitions. It does not
    /// capture attribute-only symbols, `$` variables, options, messages, the
    /// gensym counter, PRNG state, or contexts beyond what the generated source
    /// re-establishes. This is a text image, not a binary `.mx` dump.
    pub fn snapshot_definitions_source(&self) -> Result<String, String> {
        let mut symbols = self
            .definitions
            .keys()
            .filter(|symbol| {
                !self.is_installed_protected_builtin_own_value_symbol(symbol)
                    && !self.is_installed_system_down_values_symbol(symbol)
            })
            .cloned()
            .collect::<Vec<_>>();
        symbols.sort();

        if self.sealed_units.is_empty()
            && symbols
                .iter()
                .all(|symbol| !crate::runtime::scoping::packages::symbol_is_sealed_private(symbol))
        {
            for symbol in &symbols {
                for statement in self.saved_symbol_statements(symbol, true) {
                    if let Some(orphan) =
                        crate::runtime::scoping::packages::sealed_symbols_in_expr(&statement)
                            .into_iter()
                            .next()
                    {
                        return Err(snapshot_orphan_error(&orphan));
                    }
                }
            }
            return Ok(self.render_saved_symbols(&symbols, true, true));
        }

        let persisted_symbols = symbols
            .iter()
            .copied()
            .filter(|symbol| !crate::runtime::scoping::packages::symbol_is_sealed_private(symbol))
            .collect::<Vec<_>>();
        let persisted_set = persisted_symbols.iter().copied().collect::<BTreeSet<_>>();
        let mut owned_units = BTreeMap::new();
        for (unit_index, unit) in self.sealed_units.iter().enumerate() {
            for symbol in unit.public.iter().chain(&unit.package_scope) {
                owned_units.entry(*symbol).or_insert(unit_index);
            }
        }

        let mut blocks = self
            .sealed_units
            .iter()
            .map(|unit| unit.held_wrapper.to_string())
            .collect::<Vec<_>>();
        let mut emitted_owned = BTreeSet::new();
        for (unit_index, unit) in self.sealed_units.iter().enumerate() {
            for symbol in unit.public.iter().chain(&unit.package_scope) {
                if !persisted_set.contains(symbol) || !emitted_owned.insert(*symbol) {
                    continue;
                }
                for statement in self.saved_symbol_statements(symbol, true) {
                    blocks.push(self.render_snapshot_statement(statement, Some(unit_index))?);
                }
            }
        }

        if let Some(preamble) = self.render_dump_save_package_restore(&persisted_symbols) {
            blocks.push(preamble);
        }
        for symbol in persisted_symbols
            .iter()
            .filter(|symbol| !owned_units.contains_key(symbol))
        {
            for statement in self.saved_symbol_statements(symbol, true) {
                blocks.push(self.render_snapshot_statement(statement, None)?);
            }
        }

        if blocks.is_empty() {
            Ok(String::new())
        } else {
            Ok(format!("{}\n", blocks.join("\n\n")))
        }
    }

    fn render_snapshot_statement(
        &self,
        statement: Expr,
        owning_unit: Option<usize>,
    ) -> Result<String, String> {
        let mut unit_indices = BTreeSet::new();
        if let Some(index) = owning_unit {
            unit_indices.insert(index);
        }

        for sealed in crate::runtime::scoping::packages::sealed_symbols_in_expr(&statement) {
            let Some(index) = self.sealed_units.iter().position(|unit| {
                unit.private_index
                    .values()
                    .any(|current| current == &sealed)
            }) else {
                return Err(snapshot_orphan_error(&sealed));
            };
            unit_indices.insert(index);
        }

        if unit_indices.is_empty() {
            return Ok(statement.to_string());
        }

        let mut units = unit_indices
            .into_iter()
            .map(|index| {
                let unit = &self.sealed_units[index];
                Expr::call(
                    system_expr("Rule"),
                    vec![
                        Expr::string(unit.package_context.clone()),
                        Expr::integer(BigInt::from(unit.epoch)),
                    ],
                )
            })
            .collect::<Vec<_>>();
        let units = if units.len() == 1 {
            units.pop().expect("one re-link unit should be present")
        } else {
            Expr::call(system_expr("List"), units)
        };
        Ok(Expr::call(
            system_expr("RestoreSealedReferences"),
            vec![
                units,
                Expr::call(system_expr("HoldComplete"), vec![statement]),
            ],
        )
        .to_string())
    }

    pub(in crate::runtime) fn is_installed_protected_builtin_own_value_symbol(
        &self,
        symbol: &Symbol,
    ) -> bool {
        let Some(definition) = self.definitions.get(symbol) else {
            return false;
        };

        symbol.is_system()
            && self.installed_system_own_values.contains(symbol)
            && self.builtin_spec_for_symbol(symbol).is_some()
            && self.symbol_is_protected(symbol)
            && definition.own_value.is_some()
            && definition.delayed_own_value.is_none()
            && definition.own_value_guards.is_none()
            && definition.up_values.is_empty()
            && definition.down_values.is_empty()
            && definition.sub_values.is_empty()
    }

    pub(in crate::runtime) fn is_installed_system_down_values_symbol(
        &self,
        symbol: &Symbol,
    ) -> bool {
        let Some(installed_down_values) = self.installed_system_down_values.get(symbol) else {
            return false;
        };
        let Some(definition) = self.definitions.get(symbol) else {
            return false;
        };

        symbol.is_system()
            && definition.own_value.is_none()
            && definition.delayed_own_value.is_none()
            && definition.own_value_guards.is_none()
            && definition.up_values.is_empty()
            && definition.sub_values.is_empty()
            && definition.down_values.len() == installed_down_values.len()
            && definition
                .down_values
                .iter()
                .zip(installed_down_values)
                .all(|(down_value, (lhs, rhs))| {
                    !down_value.delayed
                        && down_value.guards.is_empty()
                        && &down_value.lhs == lhs
                        && &down_value.rhs == rhs
                })
    }

    pub fn context_names(&self) -> Vec<String> {
        let mut contexts = self
            .context_path
            .iter()
            .cloned()
            .chain(std::iter::once(self.current_context.clone()))
            .collect::<BTreeSet<_>>();

        contexts.extend(
            self.definitions
                .keys()
                .map(|symbol| symbol.context().to_string()),
        );

        contexts.into_iter().collect()
    }

    pub fn set_current_context(&mut self, context: impl Into<String>) {
        self.current_context = normalize_context_name(context.into());
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::CurrentContext,
        );
    }

    pub fn set_context_path<I, S>(&mut self, contexts: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.context_path = contexts
            .into_iter()
            .map(|context| normalize_context_name(context.into()))
            .collect();
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
    }

    pub fn package_search_paths(&self) -> &[PathBuf] {
        &self.package_search_paths
    }

    pub fn set_file_io(&mut self, provider: Option<FileIoProvider>) {
        self.file_io = provider;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::FileIo);
    }

    pub fn file_io(&self) -> Option<&dyn FileIo> {
        self.file_io.as_ref().map(FileIoProvider::as_file_io)
    }

    pub fn has_file_io(&self) -> bool {
        self.file_io.is_some()
    }

    pub fn set_entropy_source(&mut self, provider: Option<EntropyProvider>) {
        self.entropy_source = provider;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::EntropySource,
        );
    }

    pub fn entropy_source(&self) -> Option<&dyn EntropySource> {
        self.entropy_source
            .as_ref()
            .map(EntropyProvider::as_entropy_source)
    }

    pub fn has_entropy_source(&self) -> bool {
        self.entropy_source.is_some()
    }

    pub fn fill_entropy(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        let Some(source) = self.entropy_source() else {
            return Err(EntropyError::new("no entropy provider installed"));
        };

        source.fill(buf)
    }

    pub fn seed_random_with_integer(&mut self, seed: &BigInt) {
        self.prng_state = PrngState::from_seed_integer(seed);
    }

    pub fn seed_random_with_bytes(&mut self, bytes: &[u8]) {
        self.prng_state = PrngState::from_seed_bytes(bytes);
    }

    pub fn seed_random_from_entropy(&mut self) -> Result<(), EntropyError> {
        let mut seed = [0_u8; 32];
        self.fill_entropy(&mut seed)?;
        self.seed_random_with_bytes(&seed);
        Ok(())
    }

    pub fn random_u64(&mut self) -> u64 {
        self.prng_state.next_u64()
    }

    pub fn random_fill_bytes(&mut self, buf: &mut [u8]) {
        self.prng_state.fill_bytes(buf);
    }

    pub fn evaluate_with_random_state_restored(&mut self, body: Expr) -> Result<Expr, EvalError> {
        let saved = self.prng_state;
        let result = self.evaluate(body);
        let outcome = crate::resume_or_suspend!(
            self,
            result,
            ok(value) => Ok(value),
            suspend => EvaluateWithRandomStateRestoredSuspensionFrame { saved },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            other => {
                self.prng_state = saved;
                other
            }
        }
    }

    pub fn set_package_search_paths<I, P>(&mut self, paths: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.package_search_paths = paths.into_iter().map(Into::into).collect();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PackageSearchPaths,
        );
    }

    pub fn add_package_search_path(&mut self, path: impl Into<PathBuf>) {
        self.package_search_paths.push(path.into());
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PackageSearchPaths,
        );
    }

    pub fn lower_text(&self, input: &str) -> Result<Expr, EvalError> {
        let syntax = parse_expr(input)?;
        self.lower_syntax_expr(&syntax)
    }

    pub fn complete_symbol_prefix(&self, prefix: &str) -> Vec<String> {
        let prefix = prefix.trim();
        let explicit_context = prefix.contains('`');
        let mut candidates = BTreeSet::new();

        for symbol in self.builtins.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for symbol in self.definitions.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for symbol in self.attributes.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for symbol in self.syntax_information.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for symbol in &self.declared_stubs {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for key in self.message_definitions.keys() {
            self.insert_completion_candidate(
                &mut candidates,
                &key.symbol,
                prefix,
                explicit_context,
            );
        }

        for context in self
            .context_path
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(self.current_context.as_str()))
            .chain(self.loaded_packages.iter().map(String::as_str))
        {
            let context = normalize_context_name(context);

            if context.starts_with(prefix) {
                candidates.insert(context);
            }
        }

        candidates.into_iter().collect()
    }

    pub fn loaded_packages(&self) -> &[String] {
        &self.loaded_packages
    }

    pub(in crate::runtime) fn mark_package_loaded(&mut self, context: &str) {
        let context = normalize_context_name(context);

        if !self.loaded_packages.iter().any(|loaded| loaded == &context) {
            self.loaded_packages.push(context);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::LoadedPackages,
            );
            self.bump_evaluation_epoch();
        }
    }

    fn insert_completion_candidate(
        &self,
        candidates: &mut BTreeSet<String>,
        symbol: &Symbol,
        prefix: &str,
        explicit_context: bool,
    ) {
        let sealed_private = crate::runtime::scoping::packages::symbol_is_sealed_private(symbol);
        if sealed_private && !self.sealed_symbols_visible() {
            return;
        }

        if sealed_private {
            let context = normalize_context_name(symbol.context());
            if context.starts_with(prefix) {
                candidates.insert(context);
            }
        }

        let candidate = if explicit_context {
            symbol.fully_qualified_name()
        } else {
            symbol.qualified_name()
        };

        if candidate.starts_with(prefix) {
            candidates.insert(candidate);
        }
    }

    pub(in crate::runtime) fn begin_context(&mut self, context: String) -> Expr {
        let previous_context = self.current_context.clone();
        let previous_context_path = self.context_path.clone();
        self.context_stack.push(ContextFrame {
            kind: ContextFrameKind::Begin,
            previous_context,
            previous_context_path,
            opened_context: context.clone(),
        });
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ContextStack,
        );
        self.current_context = context.clone();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::CurrentContext,
        );
        self.context_path = prepend_context(&context, &self.context_path);
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
        Expr::string(context)
    }

    pub(in crate::runtime) fn end_context(&mut self) -> Option<Expr> {
        let frame = self.context_stack.pop()?;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ContextStack,
        );

        if frame.kind != ContextFrameKind::Begin {
            self.context_stack.push(frame);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::ContextStack,
            );
            return None;
        }

        let current = self.current_context.clone();
        self.current_context = frame.previous_context;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::CurrentContext,
        );
        self.context_path = frame.previous_context_path;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
        Some(Expr::string(current))
    }

    pub(in crate::runtime) fn begin_package_context(&mut self, context: String) -> Expr {
        let previous_context = self.current_context.clone();
        let previous_context_path = self.context_path.clone();
        self.context_stack.push(ContextFrame {
            kind: ContextFrameKind::BeginPackage,
            previous_context,
            previous_context_path,
            opened_context: context.clone(),
        });
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ContextStack,
        );
        self.current_context = context.clone();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::CurrentContext,
        );
        self.context_path = vec![context.clone(), SYSTEM_CONTEXT.to_string()];
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
        self.mark_package_loaded(&context);
        Expr::string(context)
    }

    pub(in crate::runtime) fn end_package_context(&mut self) -> Option<Expr> {
        let frame = self.context_stack.pop()?;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ContextStack,
        );

        if frame.kind != ContextFrameKind::BeginPackage {
            self.context_stack.push(frame);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::ContextStack,
            );
            return None;
        }

        self.current_context = frame.previous_context;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::CurrentContext,
        );
        self.context_path = prepend_context(&frame.opened_context, &frame.previous_context_path);
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
        Some(system_expr("Null"))
    }
}

fn snapshot_orphan_error(symbol: &Symbol) -> String {
    let owner_key = symbol
        .context()
        .strip_suffix("Private`")
        .unwrap_or(symbol.context());
    format!(
        "snapshot refused orphaned sealed symbol {} (owner key {owner_key})",
        symbol.fully_qualified_name()
    )
}
