use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

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
        crate::resume_or_suspend!(insert self, RunCallDispatchStageSuspensionFrame);
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
        let result = self.run_builtin_handler(handler, head, args);
        record_profile_builtin_timing(profile_builtin_start.flatten(), result.is_ok());
        result.map(Some)
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
        handler(self, head, args)
    }
}

pub(super) trait CallBuiltinSpec: Copy {
    fn name(self) -> &'static str;
    fn handler(self) -> Option<BuiltinFn>;
    fn capability(self) -> Capability;
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
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct RunCallDispatchStageSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for RunCallDispatchStageSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
