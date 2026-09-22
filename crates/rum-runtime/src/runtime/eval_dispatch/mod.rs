use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::atomic::{AtomicU8, Ordering};

const BUILTIN_MEMORY_GROWTH_TEST_ENV: &str = "RUM_TEST_BUILTIN_MEMORY_GROWTH";
const NON_AMPLIFYING_RESULT_ALLOWANCE_BYTES: usize = 1024;
pub(in crate::runtime) const KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS: &[&str] = &[
    // A quote-aware field/row count is known only after CSV parsing. The
    // sound cheap bound (one result node per input byte) rejects long scalar
    // fields and comment-heavy inputs whose decoded result is small.
    "FromCsv",
    // Sections, comments, blank lines, and scalar widths make the node count
    // content-dependent. A byte-length multiplier materially over-projects
    // long values and ignored lines; an exact pass would duplicate the parser.
    "FromIni",
    // The same byte length can be one string/number or a punctuation-dense
    // tree. Its worst-case per-byte node bound is not useful for long scalars.
    "FromJson",
    // Dotted keys, arrays/tables, temporal values, comments, and strings have
    // sharply different expansion. A useful exact bound requires TOML parse
    // state rather than a sound up-front length multiplier.
    "FromToml",
    // Markup, attributes, entities, comments, and text produce very different
    // node counts. The per-byte worst case over-refuses large text nodes.
    "FromXml",
    // Indentation, block scalars, comments, collections, and plain scalars
    // make the result shape parse-dependent; a worst-case length bound is too
    // loose for ordinary block-scalar and comment-heavy documents.
    "FromYaml",
    // Token/trivia/error span cardinality is known only after lexing. One span
    // per byte is sound but rejects long identifiers, strings, and comments
    // that lex to one or a few spans.
    "RumTokens",
    // User-supplied pattern matches decide every boundary. A character-count
    // bound rejects long unmatched text that becomes a single output span;
    // discovering the exact count is the tokenizer operation itself.
    "TokenizeWith",
];
static BUILTIN_MEMORY_GROWTH_TEST_ENABLED: AtomicU8 = AtomicU8::new(0);

impl Session {
    pub(super) fn build_call_dispatch_plan(&mut self, head: &Expr) -> CallDispatchPlan {
        let metadata = self.call_preparation_metadata(head);
        let attributes = metadata.attributes();
        let head_symbol = metadata.head_symbol();
        let builtin_spec = head_symbol
            .as_ref()
            .and_then(|symbol| self.cached_builtin_dispatch_spec_for_symbol(symbol));
        let builtin_dispatch = builtin_spec.map(|spec| spec.dispatch);

        if builtin_dispatch == Some(BuiltinDispatch::SpecialForm) {
            return CallDispatchPlan::special_form(attributes, builtin_spec);
        }

        let definition_symbol = metadata.definition_symbol();
        let value_definition = definition_symbol
            .as_ref()
            .and_then(|symbol| self.definitions.get(symbol));
        let has_down_values = head_symbol.is_some()
            && value_definition.is_some_and(|definition| !definition.down_values.is_empty());
        let has_sub_values =
            value_definition.is_some_and(|definition| !definition.sub_values.is_empty());

        let mut plan = CallDispatchPlan::new(attributes, builtin_spec);

        plan.push(CallDispatchStage::PrepareArguments);

        if attributes.contains(Attribute::Listable) {
            plan.push(CallDispatchStage::Listable);
        }

        if !attributes.contains(Attribute::HoldAllComplete) && self.has_up_value_definitions {
            plan.push(CallDispatchStage::UpValues);
        }

        if has_down_values {
            plan.push(CallDispatchStage::DownValues);
        }

        if has_sub_values {
            plan.push(CallDispatchStage::SubValues);
        }

        if builtin_dispatch == Some(BuiltinDispatch::Ordinary) {
            plan.push(CallDispatchStage::OrdinaryBuiltin);
        }

        if parse_function(head).is_some()
            || parse_composition(head).is_some()
            || is_failure_expr(head)
            || is_success_expr(head)
            || has_association_head(head)
            || is_callable_operator_wrapper(head)
            || is_callable_failsafe_wrapper(head)
        {
            plan.push(CallDispatchStage::Function);
        }

        plan
    }

    pub(super) fn run_call_dispatch_stage<S>(
        &mut self,
        stage: CallDispatchStage,
        head: &Expr,
        attributes: AttributeSet,
        builtin_spec: Option<S>,
        state: &mut CallDispatchState,
    ) -> Result<Option<Expr>, EvalError>
    where
        S: CallBuiltinSpec,
    {
        match stage {
            CallDispatchStage::SpecialForm => {
                self.try_dispatch_builtin_call(builtin_spec, head, state.raw_args())
            }
            CallDispatchStage::PrepareArguments => {
                let raw_args = state.take_raw_args();
                let prepared =
                    self.prepare_arguments_for_call(head.clone(), attributes, raw_args)?;
                state
                    .set_prepared_args(self.normalize_call_args_shared(head, prepared, attributes));
                Ok(None)
            }
            CallDispatchStage::Listable => {
                match self.dispatch_listable_stage(head, state.prepared_args(), attributes)? {
                    ThreadingDispatch::NotApplicable => Ok(None),
                    ThreadingDispatch::Threaded(threaded) => {
                        let threaded_result = self.evaluate(threaded);
                        match threaded_result {
                            Ok(value) => Ok(Some(value)),
                            Err(EvalError::InputRequired { request }) => {
                                self.suspend_threaded_call_dispatch(request)
                            }
                            Err(error) => Err(error),
                        }
                    }
                    ThreadingDispatch::Mismatch => Ok(Some(Expr::call(
                        head.clone(),
                        state.prepared_args().to_vec(),
                    ))),
                }
            }
            CallDispatchStage::UpValues => {
                self.try_apply_up_values(state.prepared_definition_lookup(head, attributes))
            }
            CallDispatchStage::DownValues => {
                self.try_apply_down_values(state.prepared_definition_lookup(head, attributes))
            }
            CallDispatchStage::SubValues => {
                self.try_apply_sub_values(state.prepared_definition_lookup(head, attributes))
            }
            CallDispatchStage::OrdinaryBuiltin => {
                self.try_dispatch_builtin_call(builtin_spec, head, state.prepared_shared_args())
            }
            CallDispatchStage::Function => self.try_apply_function(head, state.prepared_args()),
        }
    }

    #[cold]
    #[inline(never)]
    fn suspend_threaded_call_dispatch(
        &mut self,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        crate::resume_or_suspend!(
            insert self,
            RunCallDispatchStageSuspensionFrame::threaded_call()
        );
        Err(EvalError::InputRequired { request })
    }

    pub(super) fn try_dispatch_builtin_call<S>(
        &mut self,
        builtin_spec: Option<S>,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Option<Expr>, EvalError>
    where
        S: CallBuiltinSpec,
    {
        let Some(spec) = builtin_spec else {
            return Ok(None);
        };
        let Some(handler) = spec.handler() else {
            return Ok(None);
        };
        let capability = spec.capability();

        if !self.capability_policy.is_unrestricted() && !self.capability_policy.allows(capability) {
            if self.value_q_refuses_capability(capability) {
                // `ValueQ` consumes every non-input error. Cancellation is a
                // private terminal signal here and never reaches the host.
                return Err(EvalError::Cancelled);
            }
            return Ok(Some(self.sandbox_denied_builtin(head, spec)));
        }

        let builtin_symbol = expr_as_symbol(head).copied();
        if let Some(symbol) = builtin_symbol {
            self.record_profile_builtin_application(&symbol);
        }
        let profile_builtin_start =
            builtin_symbol.map(|symbol| builtin_timing_start(&symbol, args.len()));
        if !builtin_memory_growth_test_enabled() {
            let result = self.run_builtin_handler(handler, head, args);
            let result = crate::resume_or_suspend!(
                self,
                result,
                ok(value) => Ok(value),
                suspend => RunCallDispatchStageSuspensionFrame::unchecked_builtin(),
            );
            record_profile_builtin_timing(profile_builtin_start.flatten(), result.is_ok());
            return result.map(Some);
        }

        let projection_count_before = self.builtin_memory_projection_count.get();
        let result = self.run_builtin_handler(handler, head, args);
        let result = crate::resume_or_suspend!(
            self,
            result,
            ok(value) => Ok(value),
            suspend => RunCallDispatchStageSuspensionFrame::builtin(
                spec.name(),
                spec.memory_growth(),
                args.clone(),
                projection_count_before,
            ),
        );
        record_profile_builtin_timing(profile_builtin_start.flatten(), result.is_ok());
        self.validate_builtin_memory_growth_slow(
            spec.name(),
            spec.memory_growth(),
            args,
            projection_count_before,
            &result,
        );
        result.map(Some)
    }

    #[inline]
    pub(in crate::runtime) fn record_builtin_memory_projection_for_growth_test(&self) {
        if !builtin_memory_growth_test_enabled() {
            return;
        }

        self.builtin_memory_projection_count
            .set(self.builtin_memory_projection_count.get().saturating_add(1));
    }

    #[cold]
    #[inline(never)]
    fn validate_builtin_memory_growth_slow(
        &mut self,
        name: &'static str,
        contract: MemoryGrowthContract,
        args: &SharedArgs,
        projection_count_before: u64,
        result: &Result<Expr, EvalError>,
    ) {
        let Ok(result) = result else {
            return;
        };

        match contract {
            MemoryGrowthContract::NonAmplifying => {
                let argument_bytes = self.local_args_memory_proxy_bytes(args);
                assert_non_amplifying_result(name, argument_bytes, result, self);
            }
            MemoryGrowthContract::Preflight
                if self.builtin_memory_projection_count.get() == projection_count_before =>
            {
                let argument_bytes = self.local_args_memory_proxy_bytes(args);
                assert_preflight_result(name, argument_bytes, result, self);
            }
            MemoryGrowthContract::Preflight
            | MemoryGrowthContract::Incremental
            | MemoryGrowthContract::Delegated => {}
        }
    }

    fn local_args_memory_proxy_bytes(&mut self, args: &SharedArgs) -> usize {
        args.iter().fold(0usize, |total, arg| {
            total.saturating_add(self.local_expr_memory_proxy_bytes(arg))
        })
    }

    fn local_expr_memory_proxy_bytes(&mut self, expr: &Expr) -> usize {
        let byte_count = self
            .eval_byte_count(&system_expr("ByteCount"), std::slice::from_ref(expr))
            .expect("the internal ByteCount proxy cannot fail");
        expr_as_non_negative_usize(&byte_count)
            .expect("the internal ByteCount proxy always returns a non-negative integer")
    }

    fn sandbox_denied_builtin<S: CallBuiltinSpec>(&mut self, head: &Expr, spec: S) -> Expr {
        let name = expr_as_symbol(head)
            .map(|symbol| symbol.name().to_string())
            .unwrap_or_else(|| spec.name().to_string());
        self.push_message(
            "General",
            "sandbox",
            format!("{name} is not permitted in this sandbox."),
        );
        system_expr("$Failed")
    }

    pub(super) fn run_builtin_handler(
        &mut self,
        handler: BuiltinFn,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        if self.current_memory_limit.is_some() {
            return self.run_accounting_builtin(handler, head, args);
        }
        handler(self, head, args)
    }
}

pub(super) trait CallBuiltinSpec: Copy {
    fn name(self) -> &'static str;
    fn handler(self) -> Option<BuiltinFn>;
    fn capability(self) -> Capability;
    fn memory_growth(self) -> MemoryGrowthContract;
}

impl CallBuiltinSpec for BuiltinSpec {
    fn name(self) -> &'static str {
        self.name
    }

    fn handler(self) -> Option<BuiltinFn> {
        self.handler
    }

    fn capability(self) -> Capability {
        self.capability
    }

    fn memory_growth(self) -> MemoryGrowthContract {
        self.memory_growth
    }
}

impl CallBuiltinSpec for SuspendedBuiltinSpec {
    fn name(self) -> &'static str {
        self.name
    }

    fn handler(self) -> Option<BuiltinFn> {
        self.handler
    }

    fn capability(self) -> Capability {
        self.capability.into_capability()
    }

    fn memory_growth(self) -> MemoryGrowthContract {
        MemoryGrowthContract::from_suspended_code(self.memory_growth_code)
    }
}

#[inline]
pub(in crate::runtime) fn builtin_memory_growth_test_enabled() -> bool {
    if cfg!(any(test, feature = "test-builtin-memory-growth")) {
        return true;
    }

    match BUILTIN_MEMORY_GROWTH_TEST_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_builtin_memory_growth_test_enabled(),
    }
}

#[cold]
fn initialize_builtin_memory_growth_test_enabled() -> bool {
    let enabled = std::env::var_os(BUILTIN_MEMORY_GROWTH_TEST_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    BUILTIN_MEMORY_GROWTH_TEST_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
fn assert_non_amplifying_result(
    name: &'static str,
    argument_bytes: usize,
    result: &Expr,
    session: &mut Session,
) {
    let result_bytes = session.local_expr_memory_proxy_bytes(result);
    assert!(
        result_bytes <= argument_bytes.saturating_add(NON_AMPLIFYING_RESULT_ALLOWANCE_BYTES),
        "builtin {name} declared NonAmplifying but returned {result_bytes} proxy bytes from \
         {argument_bytes} argument proxy bytes (allowance: \
         {NON_AMPLIFYING_RESULT_ALLOWANCE_BYTES} bytes)"
    );
}

#[cold]
fn assert_preflight_result(
    name: &'static str,
    argument_bytes: usize,
    result: &Expr,
    session: &mut Session,
) {
    let result_bytes = session.local_expr_memory_proxy_bytes(result);
    if result_bytes <= argument_bytes.saturating_add(NON_AMPLIFYING_RESULT_ALLOWANCE_BYTES)
        || KNOWN_UNPROJECTED_PREFLIGHT_BUILTINS
            .binary_search(&name)
            .is_ok()
    {
        return;
    }

    panic!(
        "builtin {name} declared Preflight but returned {result_bytes} proxy bytes from \
         {argument_bytes} argument proxy bytes without projecting bytes first (allowance: \
         {NON_AMPLIFYING_RESULT_ALLOWANCE_BYTES} bytes)"
    );
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct RunCallDispatchStageSuspensionFrame {
    builtin_name: Option<&'static str>,
    memory_growth_code: u8,
    args: Option<SharedArgs>,
    projection_count_before: u64,
}

impl RunCallDispatchStageSuspensionFrame {
    fn threaded_call() -> Self {
        Self {
            builtin_name: None,
            memory_growth_code: 0,
            args: None,
            projection_count_before: 0,
        }
    }

    fn unchecked_builtin() -> Self {
        Self {
            builtin_name: None,
            memory_growth_code: 0,
            args: None,
            projection_count_before: 0,
        }
    }

    fn builtin(
        name: &'static str,
        memory_growth: MemoryGrowthContract,
        args: SharedArgs,
        projection_count_before: u64,
    ) -> Self {
        Self {
            builtin_name: Some(name),
            memory_growth_code: memory_growth.suspended_code(),
            args: Some(args),
            projection_count_before,
        }
    }
}

impl crate::runtime::suspension::SuspensionFrame for RunCallDispatchStageSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(value) => {
                let result = Ok(value);
                if let (Some(name), Some(args)) = (self.builtin_name, self.args.as_ref()) {
                    session.validate_builtin_memory_growth_slow(
                        name,
                        MemoryGrowthContract::from_suspended_code(self.memory_growth_code),
                        args,
                        self.projection_count_before,
                        &result,
                    );
                }
                result
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
