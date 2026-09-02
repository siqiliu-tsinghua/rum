use rum_core::{Atom, Expr};
use rum_runtime::{CapabilityPolicy, Session};
use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use crate::config::HostConfig;
use crate::cursor::{completion_prefix, symbol_at_cursor};
use crate::model::{
    HostCompletionRequest, HostCompletionResponse, HostDefinitionSummary, HostEvalOutcome,
    HostEvalRequest, HostFailure, HostFailureCode, HostInputSuspension, HostInspectRequest,
    HostInspectResponse, HostIsCompleteRequest, HostIsCompleteResponse, HostMessageTemplate,
    HostParseRequest, HostParseResponse, HostTyped, completeness_status, eval_failure_from_runtime,
    eval_success_from_runtime, host_failure_from_runtime, inspect_response_from_runtime,
    synthesized_host_failure,
};

/// Builder for [`HostSession`].
///
/// Use this when the default session configuration needs tracing, sandboxing,
/// core-builtin installation changes, or downstream builtin-family installers
/// before construction.
#[derive(Clone, Debug)]
pub struct HostSessionBuilder {
    config: HostConfig,
}

impl Default for HostSessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HostSessionBuilder {
    /// Creates a builder with [`HostConfig::default`].
    pub fn new() -> Self {
        Self {
            config: HostConfig::default(),
        }
    }

    /// Replaces the full host configuration used by the builder.
    pub fn with_config(mut self, config: HostConfig) -> Self {
        self.config = config;
        self
    }

    /// Enables or disables evaluation trace collection.
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.config.tracing_enabled = enabled;
        self
    }

    /// Replaces the capability policy for the built session.
    pub fn with_capability_policy(mut self, policy: CapabilityPolicy) -> Self {
        self.config.capability_policy = policy;
        self
    }

    /// Enables or disables installation of the standard core builtins.
    pub fn with_core_builtins(mut self, enabled: bool) -> Self {
        self.config.install_core_builtins = enabled;
        self
    }

    /// Adds a downstream builtin-family installer.
    ///
    /// Installers are applied in the order added, after the optional core
    /// builtin installation. They are part of the stored host configuration, so
    /// sessions built from cloned configurations keep them, and
    /// [`HostSession::reset`] re-applies them. Snapshot images capture user
    /// definitions only; installer-provided builtin handlers come from the
    /// restored session's configuration.
    pub fn with_builtin_installer(
        mut self,
        installer: impl Fn(&mut Session) + Send + Sync + 'static,
    ) -> Self {
        self.config = self.config.with_builtin_installer(installer);
        self
    }

    /// Builds a new host session from the builder configuration.
    pub fn build(self) -> HostSession {
        let session = HostSession::build_inner_session(&self.config);
        let install_core_builtins = self.config.install_core_builtins;

        HostSession {
            session,
            config: self.config,
            install_core_builtins,
            poisoned: false,
        }
    }
}

/// Stateful embedding session for evaluating `rum` source.
///
/// A session owns runtime definitions, contexts, pending input continuations,
/// tracing state, timeout/cancel configuration, and the poison boundary used
/// after a caught panic.
pub struct HostSession {
    session: Session,
    config: HostConfig,
    install_core_builtins: bool,
    poisoned: bool,
}

/// Cloneable cancellation handle for a [`HostSession`].
///
/// A handle interrupts the session instance that produced it. Handles returned
/// before [`HostSession::reset`] are invalidated by that reset.
#[derive(Clone, Debug)]
pub struct HostCancelHandle {
    flag: Arc<AtomicBool>,
}

impl HostCancelHandle {
    /// Requests cancellation of the associated session's current evaluation.
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

impl Default for HostSession {
    fn default() -> Self {
        Self::new()
    }
}

impl HostSession {
    /// Creates a session with the default trusted-host configuration.
    ///
    /// The default installs the standard core builtins and uses the default
    /// unrestricted capability policy.
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Creates a builder for configuring a host session before construction.
    pub fn builder() -> HostSessionBuilder {
        HostSessionBuilder::new()
    }

    fn build_inner_session(config: &HostConfig) -> Session {
        let mut session = Session::with_config(config.runtime_config());
        session.set_file_io(config.file_io.clone());
        session.set_entropy_source(config.entropy_source.clone());

        if config.install_core_builtins {
            rum_stdlib::install_core(&mut session);
        }
        for installer in &config.builtin_installers {
            installer.install(&mut session);
        }

        session
    }

    /// Rebuilds this session to its original post-construction state.
    ///
    /// This re-applies core builtins and downstream builtin installers from the
    /// stored configuration, then drops all user state, including definitions,
    /// attributes, contexts, loaded packages, messages, gensyms, queued input
    /// responses, and any installed print sink. Reinstall the print sink after
    /// reset. Reset also invalidates any previously returned
    /// [`HostCancelHandle`], because the rebuilt session has a fresh interrupt
    /// flag; obtain a new handle after reset. If the session was poisoned by a
    /// caught panic, reset clears the poisoned flag.
    pub fn reset(&mut self) {
        self.session = Self::build_inner_session(&self.config);
        self.poisoned = false;
    }

    /// Serializes this session's WL-level user definitions to a text image.
    ///
    /// The image captures own values, delayed own values, down values, up values,
    /// sub values, and attributes for symbols that have definitions. It does not
    /// capture attribute-only symbols, `$` variables, options, messages,
    /// installer-provided builtin handlers, the gensym counter, PRNG state, or
    /// contexts beyond what the generated source re-establishes. This is a text
    /// image, not a binary `.mx` dump.
    pub fn snapshot_to_bytes(&self) -> Result<Vec<u8>, HostFailure> {
        self.session
            .snapshot_definitions_source()
            .map(String::into_bytes)
            .map_err(snapshot_encode_failure)
    }

    /// Replaces this session's user state by restoring a text snapshot image.
    ///
    /// The snapshot format is the UTF-8 source emitted by [`Self::snapshot_to_bytes`].
    /// Restore decodes the bytes, resets the session to its original
    /// post-construction state, then evaluates the snapshot source. The image
    /// captures WL-level definitions and attributes for defined symbols; it does
    /// not capture attribute-only symbols, `$` variables, options, messages,
    /// installer-provided builtin handlers, the gensym counter, PRNG state, or
    /// contexts beyond what the source re-establishes.
    pub fn restore_from_bytes(&mut self, bytes: &[u8]) -> Result<(), HostFailure> {
        let source = std::str::from_utf8(bytes).map_err(snapshot_decode_failure)?;
        self.reset();
        if source.trim().is_empty() {
            return Ok(());
        }

        self.guard(
            |session| match session.session.eval_snapshot_restore_request(source) {
                Ok(_) => Ok(()),
                Err(failure) => match eval_failure_from_runtime(failure) {
                    HostEvalOutcome::Failure(failure) => Err(failure),
                    HostEvalOutcome::InputRequired(suspension) => {
                        Err(snapshot_input_failure(suspension))
                    }
                    HostEvalOutcome::Success(_) => unreachable!("request failure cannot succeed"),
                },
            },
            |message| Err(panic_failure(message)),
        )
    }

    /// Returns the current session configuration.
    ///
    /// The returned value reflects runtime configuration plus the host-level
    /// core-builtin installation flag.
    pub fn configuration(&self) -> HostConfig {
        let config = self.session.configuration();

        HostConfig {
            resource_limits: config.resource_limits.into(),
            capability_policy: config.capability_policy,
            current_context: config.current_context,
            context_path: config.context_path,
            package_search_paths: config.package_search_paths,
            file_io: self.config.file_io.clone(),
            entropy_source: self.config.entropy_source.clone(),
            tracing_enabled: config.tracing_enabled,
            asserts_enabled: config.asserts_enabled,
            install_core_builtins: self.install_core_builtins,
            builtin_installers: self.config.builtin_installers.clone(),
        }
    }

    /// Installs an exclusive streaming destination for `Print` output.
    ///
    /// While a sink is installed, every `Print` line is delivered to it as the
    /// line is produced during evaluation. Printed lines are not also buffered,
    /// so [`crate::HostEvalSuccess::prints`] and [`HostFailure::prints`] are
    /// empty while the sink is active.
    ///
    /// The sink runs on the evaluating thread and must not panic. A sink panic
    /// is caught like any other panic at the host boundary and poisons the
    /// session. The sink does not survive [`Self::reset`]; reinstall it after a
    /// reset.
    pub fn set_print_sink(&mut self, sink: impl FnMut(&str) + Send + 'static) {
        self.session.set_print_sink(Box::new(sink));
    }

    /// Removes the streaming `Print` sink and restores per-request buffering.
    pub fn clear_print_sink(&mut self) {
        self.session.clear_print_sink();
    }

    /// Parses source and returns a host-facing syntax rendering.
    ///
    /// A poisoned session returns [`HostFailureCode::Poisoned`].
    pub fn parse(
        &mut self,
        request: impl Into<HostParseRequest>,
    ) -> Result<HostParseResponse, HostFailure> {
        if self.poisoned {
            return Err(poisoned_failure());
        }

        let request = request.into();
        self.guard(
            |session| {
                session
                    .session
                    .parse_request(&request.source)
                    .map(|response| HostParseResponse {
                        source: response.parsed.source().to_string(),
                        syntax: response.parsed.syntax().to_string(),
                    })
                    .map_err(|failure| host_failure_from_runtime(failure))
            },
            |message| Err(panic_failure(message)),
        )
    }

    /// Lowers source and returns its canonical core-expression rendering.
    ///
    /// This is a diagnostic-grade surface for developer tooling. The rendered
    /// format is not a stability promise. Lowering resolves symbols against the
    /// session's current context state.
    ///
    /// A poisoned session returns [`HostFailureCode::Poisoned`].
    pub fn lower_dump(&mut self, source: &str) -> Result<String, HostFailure> {
        if self.poisoned {
            return Err(poisoned_failure());
        }

        self.guard(
            |session| {
                session
                    .session
                    .lower_request(source)
                    .map(|response| response.expr.to_string())
                    .map_err(host_failure_from_runtime)
            },
            |message| Err(panic_failure(message)),
        )
    }

    /// Evaluates source and returns a structured host outcome.
    ///
    /// Input suspension is reported as [`HostEvalOutcome::InputRequired`] and
    /// should be resumed with [`Self::resume_with_input`].
    /// While any evaluation is suspended for input, a new evaluation request is
    /// rejected with [`HostFailureCode::Other`] without changing the pending
    /// continuation. Resume it or call [`Self::reset`] first.
    pub fn eval(&mut self, request: impl Into<HostEvalRequest>) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        let request = request.into();
        self.guard(
            |session| session.eval_request_unprotected(request),
            panic_eval_outcome,
        )
    }

    /// Invokes a resolved function head with typed host arguments.
    ///
    /// This is the injection-safe invocation path: arguments are constructed as
    /// data and are never parsed as source. Evaluation otherwise follows ordinary
    /// call semantics, including normal argument evaluation and attributes.
    /// [`HostTyped::Symbol`] arguments resolve exactly as the same symbol text
    /// would in source.
    ///
    /// Like [`Self::eval`], this rejects a new call while an evaluation is
    /// suspended for input and leaves that continuation untouched.
    pub fn call(&mut self, head: &str, args: &[HostTyped]) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        self.guard(
            |session| session.call_unprotected(head, args),
            panic_eval_outcome,
        )
    }

    /// Evaluates a multi-form script and returns a structured host outcome.
    ///
    /// Forms are evaluated in source order. On success, the response value and
    /// `termination` describe the final form, while messages and buffered prints
    /// aggregate across all forms.
    ///
    /// Queue batch input with [`Self::queue_input_response`] before calling this
    /// method. If a form reaches `Input` or `InputString` with an empty queue,
    /// the outcome is [`HostEvalOutcome::InputRequired`]. Script suspension is
    /// not currently resumable through [`Self::resume_with_input`]: that call
    /// returns a [`HostFailureCode::Other`] failure, and the remaining script
    /// forms are not evaluated. Once a script suspends, other evaluation
    /// entrypoints are rejected; use [`Self::reset`], [`Self::restore_from_bytes`]
    /// (which resets first), or replace the whole session.
    pub fn eval_script(&mut self, source: &str) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        self.guard(
            |session| match session.session.eval_script_request(source) {
                Ok(response) => HostEvalOutcome::Success(eval_success_from_runtime(response)),
                Err(failure) => eval_failure_from_runtime(failure),
            },
            panic_eval_outcome,
        )
    }

    /// Evaluates a multi-form script read through the configured file provider.
    ///
    /// File contents are read through the session's configured file I/O
    /// provider, then evaluated with the same semantics as [`Self::eval_script`].
    /// A session without a provider, including one built from
    /// [`HostConfig::sandboxed`], returns a [`HostFailureCode::Other`] failure.
    /// A pending input suspension is rejected before the file is read.
    pub fn eval_file(&mut self, path: impl AsRef<Path>) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        self.guard(
            |session| match session.session.eval_file_request(path) {
                Ok(response) => HostEvalOutcome::Success(eval_success_from_runtime(response)),
                Err(failure) => eval_failure_from_runtime(failure),
            },
            panic_eval_outcome,
        )
    }

    fn eval_request_unprotected(&mut self, request: HostEvalRequest) -> HostEvalOutcome {
        match self.session.eval_request(&request.source) {
            Ok(response) => HostEvalOutcome::Success(eval_success_from_runtime(response)),
            Err(failure) => eval_failure_from_runtime(failure),
        }
    }

    fn call_unprotected(&mut self, head: &str, args: &[HostTyped]) -> HostEvalOutcome {
        if let Err(failure) = self.session.reject_overlapping_evaluation_request() {
            return eval_failure_from_runtime(failure);
        }

        if head.trim().is_empty() {
            return HostEvalOutcome::Failure(synthesized_host_failure(
                HostFailureCode::Other,
                "call head must not be empty",
            ));
        }

        let Some(head) = self.session.resolve_symbol_name(head, true) else {
            return HostEvalOutcome::Failure(synthesized_host_failure(
                HostFailureCode::Other,
                format!("unable to resolve call head: {head:?}"),
            ));
        };

        let mut expr_args = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            match arg.to_expr(&self.session) {
                Ok(expr) => expr_args.push(expr),
                Err(message) => {
                    return HostEvalOutcome::Failure(synthesized_host_failure(
                        HostFailureCode::Other,
                        format!("call argument at index {index} is invalid: {message}"),
                    ));
                }
            }
        }

        let expr = Expr::call(Expr::Atom(Atom::Symbol(head)), expr_args);
        match self.session.eval_expr_request(expr) {
            Ok(response) => HostEvalOutcome::Success(eval_success_from_runtime(response)),
            Err(failure) => eval_failure_from_runtime(failure),
        }
    }

    /// Returns a cancellation handle for this session.
    ///
    /// The handle may be cloned and used from another thread to request
    /// interruption of the current evaluation.
    pub fn cancel_handle(&self) -> HostCancelHandle {
        HostCancelHandle {
            flag: self.session.interrupt_handle(),
        }
    }

    /// Returns summaries of symbols with user-visible definitions.
    pub fn definitions(&self) -> Vec<HostDefinitionSummary> {
        self.session
            .definition_summaries()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Queries message templates globally or through a symbol/message selector.
    ///
    /// `None` queries the global catalog. A selector may identify a symbol or a
    /// specific `symbol::tag`; an invalid selector returns `None`. The
    /// per-symbol inspection view remains available through
    /// [`HostInspectResponse::message_catalog`].
    pub fn message_catalog(&self, query: Option<&str>) -> Option<Vec<HostMessageTemplate>> {
        self.session.inspect_message_catalog(query).map(|entries| {
            entries
                .into_iter()
                .map(|entry| HostMessageTemplate {
                    message_name: entry.message_name,
                    qualified_symbol_name: entry.qualified_symbol_name,
                    tag: entry.tag,
                    template: entry.template.to_string(),
                    delayed: entry.delayed,
                })
                .collect()
        })
    }

    /// Returns the names of contexts known to this session.
    pub fn contexts(&self) -> Vec<String> {
        self.session.context_names()
    }

    /// Evaluates source with a wall-clock timeout for this request.
    ///
    /// The timeout is cleared after the request completes or fails.
    /// A pending input suspension is rejected without replacing its continuation.
    pub fn eval_with_timeout(
        &mut self,
        request: impl Into<HostEvalRequest>,
        timeout: Duration,
    ) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        let request = request.into();
        self.guard(
            |session| {
                session.session.set_eval_timeout(Some(timeout));
                let outcome = session.eval_request_unprotected(request);
                session.session.set_eval_timeout(None);
                outcome
            },
            panic_eval_outcome,
        )
    }

    /// Queues one response for a later `Input` or `InputString` call.
    ///
    /// This is the intended way to pre-supply input for batch and script
    /// evaluation. Queue responses in consumption order before calling
    /// [`Self::eval_script`] or [`Self::eval_file`]. [`Self::reset`] clears all
    /// queued responses. Queuing a response while a continuation is already
    /// pending does not resume or supersede it, and does not permit a new
    /// evaluation request; use [`Self::resume_with_input`] for that continuation.
    pub fn queue_input_response(&mut self, input: impl Into<String>) {
        self.session.queue_input_response(input);
    }

    /// Resumes a pending `Input` or `InputString` continuation.
    ///
    /// The host should call this only after receiving
    /// [`HostEvalOutcome::InputRequired`] or observing
    /// [`Self::pending_input_request`].
    pub fn resume_with_input(&mut self, input: impl Into<String>) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        self.guard(
            |session| match session.session.resume_eval_request_with_input(input) {
                Ok(response) => HostEvalOutcome::Success(eval_success_from_runtime(response)),
                Err(failure) => eval_failure_from_runtime(failure),
            },
            panic_eval_outcome,
        )
    }

    /// Inspects the symbol located at a source cursor offset.
    ///
    /// If no symbol is found at the cursor, the response reports `found: false`.
    pub fn inspect(&mut self, request: HostInspectRequest) -> HostInspectResponse {
        if self.poisoned {
            return unavailable_inspect_response(None);
        }

        self.guard(
            |session| {
                let target = symbol_at_cursor(&request.source, request.cursor_offset);
                let response = target
                    .as_deref()
                    .and_then(|symbol| session.session.inspect_request(symbol));

                inspect_response_from_runtime(target, response)
            },
            |_| unavailable_inspect_response(None),
        )
    }

    /// Inspects a symbol by name.
    ///
    /// The name may be qualified or unqualified according to the runtime's
    /// symbol resolution rules.
    pub fn inspect_symbol(&mut self, symbol: &str) -> HostInspectResponse {
        if self.poisoned {
            return unavailable_inspect_response(Some(symbol.to_string()));
        }

        self.guard(
            |session| {
                inspect_response_from_runtime(
                    Some(symbol.to_string()),
                    session.session.inspect_request(symbol),
                )
            },
            |_| unavailable_inspect_response(Some(symbol.to_string())),
        )
    }

    /// Returns completion candidates and replacement bounds for a cursor.
    pub fn complete(&mut self, request: HostCompletionRequest) -> HostCompletionResponse {
        if self.poisoned {
            return unavailable_completion_response();
        }

        self.guard(
            |session| {
                let (replacement_start, replacement_end, prefix) =
                    completion_prefix(&request.source, request.cursor_offset);
                let completion = session.session.complete_request(&prefix);

                HostCompletionResponse {
                    prefix: completion.prefix,
                    matches: completion.matches,
                    replacement_start,
                    replacement_end,
                }
            },
            |_| unavailable_completion_response(),
        )
    }

    /// Checks whether source is complete enough to submit for evaluation.
    pub fn is_complete(
        &mut self,
        request: impl Into<HostIsCompleteRequest>,
    ) -> HostIsCompleteResponse {
        if self.poisoned {
            return poisoned_is_complete_response();
        }

        let request = request.into();
        self.guard(
            |session| {
                let response = session.session.is_complete_request(&request.source);

                HostIsCompleteResponse {
                    status: completeness_status(response.status),
                    indent: response.indent,
                    diagnostic: response.diagnostic.map(|diagnostic| diagnostic.message),
                }
            },
            |_| panic_is_complete_response(),
        )
    }

    /// Returns the input request currently suspended in this session, if any.
    pub fn pending_input_request(&self) -> Option<crate::HostInputRequest> {
        self.session
            .pending_input_request()
            .cloned()
            .map(Into::into)
    }

    fn guard<T>(
        &mut self,
        body: impl FnOnce(&mut Self) -> T,
        on_panic: impl FnOnce(String) -> T,
    ) -> T {
        match catch_unwind(AssertUnwindSafe(|| body(self))) {
            Ok(outcome) => outcome,
            Err(payload) => {
                self.poisoned = true;
                on_panic(panic_payload_message(&*payload))
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn eval_with_test_panic(&mut self) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        self.guard(
            |_session| panic!("host panic boundary test"),
            panic_eval_outcome,
        )
    }

    #[cfg(test)]
    pub(crate) fn poisoned_for_test(&self) -> bool {
        self.poisoned
    }
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "panic".to_string()
}

fn panic_failure(message: String) -> HostFailure {
    synthesized_host_failure(
        HostFailureCode::Panic,
        format!("host session caught panic: {message}"),
    )
}

fn poisoned_failure() -> HostFailure {
    synthesized_host_failure(
        HostFailureCode::Poisoned,
        "host session is poisoned after a previously caught panic",
    )
}

fn snapshot_decode_failure(error: std::str::Utf8Error) -> HostFailure {
    synthesized_host_failure(
        HostFailureCode::Other,
        format!("snapshot bytes are not valid UTF-8: {error}"),
    )
}

fn snapshot_encode_failure(message: String) -> HostFailure {
    synthesized_host_failure(HostFailureCode::Other, message)
}

fn snapshot_input_failure(suspension: HostInputSuspension) -> HostFailure {
    HostFailure {
        code: HostFailureCode::Other,
        exit_status: None,
        message: "snapshot restore unexpectedly requested input".to_string(),
        syntax_rendered: None,
        messages: suspension.messages,
        prints: suspension.prints,
        trace: suspension.trace,
        outputs: suspension.outputs,
    }
}

fn panic_eval_outcome(message: String) -> HostEvalOutcome {
    HostEvalOutcome::Failure(panic_failure(message))
}

fn poisoned_eval_outcome() -> HostEvalOutcome {
    HostEvalOutcome::Failure(poisoned_failure())
}

fn unavailable_inspect_response(target: Option<String>) -> HostInspectResponse {
    HostInspectResponse {
        target,
        found: false,
        info: None,
        definitions: Vec::new(),
        message_catalog: Vec::new(),
    }
}

fn unavailable_completion_response() -> HostCompletionResponse {
    HostCompletionResponse {
        prefix: String::new(),
        matches: Vec::new(),
        replacement_start: 0,
        replacement_end: 0,
    }
}

fn panic_is_complete_response() -> HostIsCompleteResponse {
    HostIsCompleteResponse {
        status: crate::HostCompletenessStatus::Invalid,
        indent: None,
        diagnostic: Some("host session caught panic".to_string()),
    }
}

fn poisoned_is_complete_response() -> HostIsCompleteResponse {
    HostIsCompleteResponse {
        status: crate::HostCompletenessStatus::Invalid,
        indent: None,
        diagnostic: Some("host session is poisoned after a previously caught panic".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostCompletenessStatus;

    #[test]
    fn panic_is_complete_response_marks_session_poisoned_and_invalid() {
        let mut session = HostSession::new();

        let response = session.guard(
            |_session| panic!("host completeness panic boundary test"),
            |_| panic_is_complete_response(),
        );

        assert_eq!(response.status, HostCompletenessStatus::Invalid);
        assert!(response.indent.is_none());
        assert_eq!(
            response.diagnostic.as_deref(),
            Some("host session caught panic")
        );
        assert!(session.poisoned_for_test());

        let poisoned = session.is_complete("1 + 1");
        assert_eq!(poisoned.status, HostCompletenessStatus::Invalid);
        assert!(
            poisoned
                .diagnostic
                .as_deref()
                .is_some_and(|text| text.contains("poisoned"))
        );
    }
}
