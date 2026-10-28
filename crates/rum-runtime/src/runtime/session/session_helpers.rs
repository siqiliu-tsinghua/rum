use super::super::*;
use rum_syntax::SourceText;
use std::time::Instant;

pub(in crate::runtime) struct EvaluationRollbackSnapshot {
    loader: LoaderSnapshot,
    option_value_scopes: Vec<OptionValueScope>,
    module_activations: Vec<ModuleActivation>,
    load_stack: Vec<PathBuf>,
    evaluation_depth: usize,
    match_depth: usize,
    evaluation_stack: Vec<EvaluationStackEntry>,
    prints: Vec<String>,
    handled_non_loader_eval_errors: Vec<RuntimeMessage>,
    reap_scopes: Vec<ReapScope>,
    monitor_scopes: Vec<MonitorScope>,
    monitor_callbacks_enabled: bool,
    current_time_deadline: Option<Instant>,
    current_memory_limit: Option<usize>,
    asserts_enabled: bool,
    abort_protect_depth: usize,
    abort_protect_pending: usize,
    queued_input_responses: VecDeque<String>,
    pending_input_request: Option<RuntimeInputRequest>,
    pending_eval_request: Option<PendingEvalRequest>,
    pending_suspension_generation: Option<SuspensionGeneration>,
}

impl Session {
    pub(in crate::runtime) fn begin_top_level_interrupt_budget(&mut self) {
        self.interrupt_poll_counter = 0;
        self.eval_deadline = self.eval_timeout.map(|timeout| Instant::now() + timeout);
        self.current_memory_limit = self.resource_limits.eval_memory_limit;
        if self.current_memory_limit.is_some() {
            self.note_accounting_arm();
        }
        self.interrupt_armed =
            self.eval_deadline.is_some() || self.active_accepted_request.is_some();
    }

    pub(in crate::runtime) fn install_accepted_request(
        &mut self,
        accepted: AcceptedRequest,
    ) -> Result<(), SessionRequestFailure> {
        let belongs_to_session = {
            let control = self.request_interrupt_control.borrow();
            accepted.belongs_to(&control)
        };
        let valid =
            self.active_accepted_request.is_none() && belongs_to_session && accepted.is_reserved();
        if !valid {
            return Err(SessionRequestFailure {
                error: SessionRequestError::Other {
                    message: "accepted request ticket does not belong to this idle session"
                        .to_string(),
                },
                messages: Vec::new(),
                prints: Vec::new(),
                trace: Vec::new(),
            });
        }

        self.active_accepted_request = Some(accepted);
        Ok(())
    }

    pub(in crate::runtime) fn activate_pending_accepted_request(
        &mut self,
    ) -> Result<(), SessionRequestFailure> {
        if self.active_accepted_request.is_some() {
            return Ok(());
        }
        let accepted = self.pending_accepted_request.borrow_mut().take();
        if let Some(accepted) = accepted {
            self.install_accepted_request(accepted)?;
        }
        Ok(())
    }

    pub(in crate::runtime) fn active_accepted_request_is_cancelled(&self) -> bool {
        self.active_accepted_request
            .as_ref()
            .is_some_and(AcceptedRequest::is_cancelled)
    }

    pub(in crate::runtime) fn take_active_accepted_request(&mut self) -> Option<AcceptedRequest> {
        let accepted = self.active_accepted_request.take();
        self.interrupt_armed = self.eval_deadline.is_some();
        accepted
    }

    pub(in crate::runtime) fn finish_active_accepted_request(&mut self) -> bool {
        self.take_active_accepted_request()
            .is_some_and(AcceptedRequest::finish)
    }

    pub(in crate::runtime) fn capture_loader_snapshot(&self) -> LoaderSnapshot {
        LoaderSnapshot {
            attributes: self.attributes.clone(),
            options: self.options.clone(),
            syntax_information: self.syntax_information.clone(),
            declared_stubs: self.declared_stubs.clone(),
            removed_symbols: self.removed_symbols.clone(),
            symbol_generations: self.symbol_generations.clone(),
            once_values: self.once_values.clone(),
            gc_candidate_gensyms: self.gc_candidate_gensyms.clone(),
            current_context: self.current_context.clone(),
            context_path: self.context_path.clone(),
            context_stack: self.context_stack.clone(),
            package_search_paths: self.package_search_paths.clone(),
            loaded_packages: self.loaded_packages.clone(),
            definitions: self.definitions.clone(),
            installed_system_own_values: self.installed_system_own_values.clone(),
            installed_system_down_values: self.installed_system_down_values.clone(),
            has_up_value_definitions: self.has_up_value_definitions,
            message_definitions: self.message_definitions.clone(),
            exception_types: self.exception_types.clone(),
            next_generated_symbol_id: self.next_generated_symbol_id,
            sealed_units: self.sealed_units.clone(),
            next_sealed_symbol_id: self.next_sealed_symbol_id,
            next_source_unit_epoch: self.next_source_unit_epoch,
            trace_events: self.trace_events.clone(),
            trace_event_depths: self.trace_event_depths.clone(),
            disabled_message_selectors: self.disabled_message_selectors.clone(),
            messages: self.messages.clone(),
            last_message_list: self.last_message_list.clone(),
        }
    }

    pub(in crate::runtime) fn restore_loader_snapshot(&mut self, snapshot: LoaderSnapshot) {
        #[cfg(test)]
        crate::runtime::session::guest_loading::tests::record_restore();
        self.attributes = snapshot.attributes;
        self.bump_attribute_revision();
        self.options = snapshot.options;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Options);
        self.syntax_information = snapshot.syntax_information;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::SyntaxInformation,
        );
        self.declared_stubs = snapshot.declared_stubs;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::DeclaredStubs,
        );
        self.removed_symbols = snapshot.removed_symbols;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::RemovedSymbols,
        );
        self.symbol_generations = snapshot.symbol_generations;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::SymbolGenerations,
        );
        self.once_values = snapshot.once_values;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::OnceValues);
        self.restore_gc_candidate_gensyms(snapshot.gc_candidate_gensyms);
        self.current_context = snapshot.current_context;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::CurrentContext,
        );
        self.context_path = snapshot.context_path;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
        self.context_stack = snapshot.context_stack;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ContextStack,
        );
        self.package_search_paths = snapshot.package_search_paths;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PackageSearchPaths,
        );
        self.loaded_packages = snapshot.loaded_packages;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LoadedPackages,
        );
        self.definitions = snapshot.definitions;
        self.installed_system_own_values = snapshot.installed_system_own_values;
        self.installed_system_down_values = snapshot.installed_system_down_values;
        self.refresh_definition_memory();
        self.has_up_value_definitions = snapshot.has_up_value_definitions;
        self.message_definitions = snapshot.message_definitions;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
        );
        self.exception_types = snapshot.exception_types;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ExceptionTypes,
        );
        self.rebuild_message_symbol_counts();
        self.next_generated_symbol_id = snapshot.next_generated_symbol_id;
        self.sealed_units = snapshot.sealed_units;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::SealedUnits);
        self.next_sealed_symbol_id = snapshot.next_sealed_symbol_id;
        self.next_source_unit_epoch = snapshot.next_source_unit_epoch;
        self.trace_events = snapshot.trace_events;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::TraceEvents);
        self.trace_event_depths = snapshot.trace_event_depths;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::TraceEventDepths,
        );
        self.disabled_message_selectors = snapshot.disabled_message_selectors;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::DisabledMessageSelectors,
        );
        self.messages = snapshot.messages;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Messages);
        self.last_message_list = snapshot.last_message_list;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );
    }

    pub(in crate::runtime) fn capture_evaluation_rollback_snapshot(
        &self,
    ) -> EvaluationRollbackSnapshot {
        EvaluationRollbackSnapshot {
            loader: self.capture_loader_snapshot(),
            option_value_scopes: self.option_value_scopes.clone(),
            module_activations: self.module_activations.clone(),
            load_stack: self.load_stack.clone(),
            evaluation_depth: self.evaluation_depth.get(),
            match_depth: self.match_depth,
            evaluation_stack: self.evaluation_stack.clone(),
            prints: self.prints.clone(),
            handled_non_loader_eval_errors: self.handled_non_loader_eval_errors.clone(),
            reap_scopes: self.reap_scopes.clone(),
            monitor_scopes: self.monitor_scopes.clone(),
            monitor_callbacks_enabled: self.monitor_callbacks_enabled,
            current_time_deadline: self.current_time_deadline,
            current_memory_limit: self.current_memory_limit,
            asserts_enabled: self.asserts_enabled,
            abort_protect_depth: self.abort_protect_depth,
            abort_protect_pending: self.abort_protect_pending,
            queued_input_responses: self.queued_input_responses.clone(),
            pending_input_request: self.pending_input_request.clone(),
            pending_eval_request: self.pending_eval_request,
            pending_suspension_generation: self.pending_suspension_generation(),
        }
    }

    pub(in crate::runtime) fn restore_evaluation_rollback_snapshot(
        &mut self,
        snapshot: EvaluationRollbackSnapshot,
    ) {
        let captured_generation = snapshot.pending_suspension_generation;
        let rearm_accounting =
            self.current_memory_limit.is_none() && snapshot.current_memory_limit.is_some();
        let current_generation = self.pending_suspension_generation();
        match (captured_generation, current_generation) {
            (None, Some(_trial_generation)) => {
                self.abandon_pending_suspension(AbandonReason::CheckpointRollbackDiscard);
                assert!(
                    self.pending_suspension_generation().is_none(),
                    "checkpoint rollback must empty its trial-created suspension before restoring"
                );
            }
            (Some(captured), Some(current)) => {
                assert_eq!(
                    current, captured,
                    "checkpoint evaluation must not replace its captured suspension owner"
                );
            }
            (Some(_), None) => {
                panic!("checkpoint evaluation lost its captured suspension owner");
            }
            (None, None) => {}
        }

        self.restore_loader_snapshot(snapshot.loader);
        self.option_value_scopes = snapshot.option_value_scopes;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::OptionValueScopes,
        );
        self.module_activations = snapshot.module_activations;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ModuleActivations,
        );
        self.load_stack = snapshot.load_stack;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::LoadStack);
        self.evaluation_depth.restore(snapshot.evaluation_depth);
        self.match_depth = snapshot.match_depth;
        self.current_memory_limit = snapshot.current_memory_limit;
        self.replace_accounted_evaluation_stack(snapshot.evaluation_stack);
        self.prints = snapshot.prints;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Prints);
        self.handled_non_loader_eval_errors = snapshot.handled_non_loader_eval_errors;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
        );
        self.reap_scopes = snapshot.reap_scopes;
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ReapScopes);
        self.monitor_scopes = snapshot.monitor_scopes;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::MonitorScopes,
        );
        self.monitor_callbacks_enabled = snapshot.monitor_callbacks_enabled;
        self.current_time_deadline = snapshot.current_time_deadline;
        self.asserts_enabled = snapshot.asserts_enabled;
        self.abort_protect_depth = snapshot.abort_protect_depth;
        self.abort_protect_pending = snapshot.abort_protect_pending;
        self.queued_input_responses = snapshot.queued_input_responses;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::QueuedInputResponses,
        );
        self.pending_input_request = snapshot.pending_input_request;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
        );
        self.pending_eval_request = snapshot.pending_eval_request;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingEvalRequest,
        );
        if rearm_accounting {
            // Loader restoration above can clear caches while unarmed. The
            // restored limit is a cold entry, so rebuild from the actual owners.
            self.note_accounting_arm();
        }
    }

    pub(in crate::runtime) fn message_enabled(&self, symbol: &str, tag: &str) -> bool {
        let candidate = MessageSelector {
            symbol: symbol.to_string(),
            tag: Some(tag.to_string()),
        };

        !self
            .disabled_message_selectors
            .iter()
            .any(|selector| selector.covers(&candidate))
    }

    /// Appends a runtime message if the message selector is currently enabled.
    pub fn push_message(&mut self, symbol: &str, tag: &str, text: impl Into<String>) {
        if !self.message_enabled(symbol, tag) {
            return;
        }

        #[cfg(debug_assertions)]
        if let Some(symbol) = self.resolve_symbol_name(symbol, false) {
            if let Some(spec) = self.builtin_spec_for_symbol(&symbol) {
                if !spec.messages.is_empty() {
                    let key = MessageNameKey {
                        symbol,
                        tag: tag.to_string(),
                    };
                    if !spec.messages.iter().any(|message| message.tag == tag)
                        && !self.message_definitions.contains_key(&key)
                    {
                        debug_assert!(
                            false,
                            "unregistered builtin message {}::{}; add it to the owning catalog file",
                            key.symbol, key.tag
                        );
                    }
                }
            }
        }

        self.current_message_list_dirty = true;
        self.push_accounted_messages(RuntimeMessage {
            symbol: symbol.to_string(),
            tag: tag.to_string(),
            text: text.into(),
        });
    }

    pub(in crate::runtime) fn push_message_unchecked(
        &mut self,
        symbol: &str,
        tag: &str,
        text: impl Into<String>,
    ) {
        if !self.message_enabled(symbol, tag) {
            return;
        }

        self.current_message_list_dirty = true;
        self.push_accounted_messages(RuntimeMessage {
            symbol: symbol.to_string(),
            tag: tag.to_string(),
            text: text.into(),
        });
    }

    pub(in crate::runtime) fn push_print(&mut self, text: impl Into<String>) {
        let text = text.into();
        if let Some(sink) = self.print_sink.as_mut() {
            sink.print_line(&text);
        } else {
            self.push_accounted_prints(text);
        }
    }

    pub(in crate::runtime) fn runtime_message_for_non_loader_eval_error(
        error: &EvalError,
    ) -> Option<RuntimeMessage> {
        if let EvalError::ExceptionThrown { exception } = error {
            return Some(RuntimeMessage {
                symbol: "ThrowException".to_string(),
                tag: "nocatch".to_string(),
                text: format!("Uncaught {exception} returned to top level."),
            });
        }

        let tag = match error {
            EvalError::EvaluationCycleDetected { .. } => "passlim",
            EvalError::RecursionLimitExceeded { .. } => "reclim",
            EvalError::MatchRecursionLimitExceeded { .. } => "matchreclim",
            EvalError::Cancelled => "cancel",
            EvalError::TimeConstraintExceeded => "timelim",
            EvalError::MemoryConstraintExceeded { .. } => "memlim",
            EvalError::StackLimitExceeded { .. }
            | EvalError::IntegerWorkLimitExceeded { .. }
            | EvalError::GeneratedElementLimitExceeded { .. }
            | EvalError::IterationWorkLimitExceeded { .. } => "reslim",
            EvalError::IterationLimitExceeded { .. } => "iterlim",
            EvalError::EvaluationPassLimitExceeded { .. } => "passlim",
            _ => return None,
        };

        Some(RuntimeMessage {
            symbol: "General".to_string(),
            tag: tag.to_string(),
            text: error.to_string(),
        })
    }

    pub(in crate::runtime) fn terminated_evaluation_reason_for_error(
        error: &EvalError,
    ) -> Option<EvalTerminationReason> {
        // Each limit gets a SPECIFIC tag so a user can tell which budget to raise
        // (a generic "ResourceLimit" did not say which of the three work limits
        // fired). The tag mirrors the message text (e.g. "iteration work limit").
        match error {
            EvalError::RecursionLimitExceeded { .. } => Some(EvalTerminationReason::RecursionLimit),
            EvalError::StackLimitExceeded { .. } => Some(EvalTerminationReason::StackLimit),
            EvalError::MatchRecursionLimitExceeded { .. } => {
                Some(EvalTerminationReason::PatternMatchRecursionLimit)
            }
            EvalError::Cancelled => Some(EvalTerminationReason::Cancelled),
            EvalError::TimeConstraintExceeded => Some(EvalTerminationReason::TimeConstraint),
            EvalError::EvaluationCycleDetected { .. }
            | EvalError::IterationLimitExceeded { .. }
            | EvalError::EvaluationPassLimitExceeded { .. } => {
                Some(EvalTerminationReason::IterationLimit)
            }
            EvalError::GeneratedElementLimitExceeded { .. } => {
                Some(EvalTerminationReason::GeneratedElementLimit)
            }
            EvalError::IntegerWorkLimitExceeded { .. } => {
                Some(EvalTerminationReason::IntegerWorkLimit)
            }
            EvalError::IterationWorkLimitExceeded { .. } => {
                Some(EvalTerminationReason::IterationWorkLimit)
            }
            _ => None,
        }
    }

    pub(in crate::runtime) fn terminated_evaluation_reason_for_top_level_error(
        error: &EvalError,
    ) -> Option<EvalTerminationReason> {
        Self::terminated_evaluation_reason_for_error(error).or_else(|| match error {
            EvalError::MemoryConstraintExceeded { .. } => Some(EvalTerminationReason::MemoryLimit),
            _ => None,
        })
    }

    pub(in crate::runtime) fn terminated_evaluation_expr_for_error(
        error: &EvalError,
    ) -> Option<Expr> {
        Self::terminated_evaluation_reason_for_error(error).map(|reason| {
            Expr::call(
                system_expr("TerminatedEvaluation"),
                vec![Expr::string(reason.tag())],
            )
        })
    }

    pub(in crate::runtime) fn emit_non_loader_eval_message(
        &mut self,
        checkpoint: usize,
        error: &EvalError,
    ) -> bool {
        let Some(message) = Self::runtime_message_for_non_loader_eval_error(error) else {
            return false;
        };

        if self.handled_non_loader_eval_errors[checkpoint..]
            .iter()
            .any(|existing| existing == &message)
        {
            return true;
        }

        self.push_accounted_handled_non_loader_eval_errors(message.clone());
        if self.message_enabled(&message.symbol, &message.tag) {
            self.push_accounted_messages(message);
        }
        true
    }

    pub(in crate::runtime) fn emit_top_level_control_flow_messages(&mut self, error: &EvalError) {
        match error {
            EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            } => {
                self.emit_uncaught_throw_message(value, tag, *explicit_tag);
            }
            EvalError::ThrownBypassingCatch {
                value,
                tag,
                explicit_tag,
                cleanup_value,
                cleanup_tag,
                cleanup_explicit_tag,
            } => {
                self.emit_uncaught_throw_message(cleanup_value, cleanup_tag, *cleanup_explicit_tag);
                self.emit_uncaught_throw_message(value, tag, *explicit_tag);
            }
            EvalError::Break => self.push_message(
                "Break",
                "nofwd",
                "No enclosing For, While, Until or Do found for Break[].",
            ),
            EvalError::Continue => self.push_message(
                "Continue",
                "nofwd",
                "No enclosing For, While, Until or Do found for Continue[].",
            ),
            _ => {}
        }
    }

    fn emit_uncaught_throw_message(&mut self, value: &Expr, tag: &Expr, explicit_tag: bool) {
        self.push_message(
            "Throw",
            "nocatch",
            format!(
                "Uncaught {} returned to top level.",
                uncaught_throw_message_expr(value, tag, explicit_tag)
            ),
        );
    }

    pub(in crate::runtime) fn emit_loader_message(&mut self, symbol: &str, error: &EvalError) {
        self.emit_loader_message_with_source(symbol, error, None);
    }

    pub(in crate::runtime) fn emit_loader_message_with_source(
        &mut self,
        symbol: &str,
        error: &EvalError,
        source: Option<&str>,
    ) {
        match error {
            EvalError::LoadPathNotFound { target, searched } => {
                let detail = if searched.is_empty() {
                    format!("could not locate `{target}`")
                } else {
                    format!("could not locate `{target}` in {}", searched.join(", "))
                };
                self.push_message(symbol, "notfound", detail);
            }
            EvalError::Io { path, source } => {
                self.push_message(symbol, "io", format!("could not read `{path}`: {source}"));
            }
            EvalError::Syntax(err) => {
                let detail = source
                    .and_then(|source| syntax_error_message_for_source(source, err))
                    .unwrap_or_else(|| err.to_string());
                self.push_message(symbol, "syntax", detail);
            }
            EvalError::NoFileIoProvider => {
                self.emit_no_file_io_message();
            }
            other => {
                self.push_message(symbol, "load", other.to_string());
            }
        }
    }

    pub(in crate::runtime) fn emit_no_file_io_message(&mut self) {
        self.push_message("General", "noio", "no file provider installed");
    }

    pub fn emit_no_entropy_message(&mut self) {
        self.push_message("General", "noentropy", "no entropy provider installed");
    }

    pub(in crate::runtime) fn invalid_call(&self, symbol: &str, args: &[Expr]) -> Expr {
        Expr::call(system_expr(symbol), args.to_vec())
    }

    /// Builds an invalid-call expression after emitting the supplied message.
    pub fn invalid_call_with_message(
        &mut self,
        symbol: &str,
        args: &[Expr],
        tag: &str,
        text: impl Into<String>,
    ) -> Expr {
        self.push_message(symbol, tag, text);
        self.invalid_call(symbol, args)
    }

    /// Builds the standard invalid-call expression for an exact arity violation.
    pub fn invalid_call_with_exact_arg_count(
        &mut self,
        symbol: &str,
        args: &[Expr],
        expected: usize,
    ) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "argx",
            format!(
                "expected exactly {expected} argument(s), got {}",
                args.len()
            ),
        )
    }

    /// Builds the standard invalid-call expression for an arity range violation.
    pub fn invalid_call_with_arg_range(
        &mut self,
        symbol: &str,
        args: &[Expr],
        min: usize,
        max: usize,
    ) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "argtu",
            format!(
                "expected between {min} and {max} argument(s), got {}",
                args.len()
            ),
        )
    }

    /// Builds the standard invalid-call expression for an argument-form violation.
    pub fn invalid_call_with_arg_form(
        &mut self,
        symbol: &str,
        args: &[Expr],
        detail: impl Into<String>,
    ) -> Expr {
        self.invalid_call_with_message(symbol, args, "argform", detail)
    }

    pub(in crate::runtime) fn messages_since_match(
        &self,
        checkpoint: usize,
        selectors: &[MessageSelector],
    ) -> bool {
        self.messages[checkpoint..]
            .iter()
            .any(|message| selectors.iter().any(|selector| selector.matches(message)))
    }

    pub(in crate::runtime) fn suppress_messages_since(
        &mut self,
        checkpoint: usize,
        selectors: &[MessageSelector],
    ) {
        let trailing_messages = self.split_accounted_messages(checkpoint);

        for message in trailing_messages {
            if !selectors.iter().any(|selector| selector.matches(&message)) {
                self.push_accounted_messages(message);
            }
        }
    }
}

fn syntax_error_message_for_source(source: &str, error: &SyntaxError) -> Option<String> {
    let span = error.span()?;
    let location = SourceText::new(source).location(span.start)?;
    let rendered = error.to_string();
    let byte_location = format!(" at bytes {span}");
    let description = rendered.replacen(&byte_location, "", 1);

    Some(format!(
        "{description} at line {}, column {} (bytes {span})",
        location.line, location.column
    ))
}

fn uncaught_throw_message_expr(value: &Expr, tag: &Expr, explicit_tag: bool) -> String {
    if !explicit_tag && is_system_symbol_name(tag, "None") {
        format!("Throw[{value}]")
    } else {
        format!("Throw[{value}, {tag}]")
    }
}
