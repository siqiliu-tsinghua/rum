use super::*;
use crate::runtime::suspension::{
    AbandonmentContext, FrameAbandon, FrameAbandonAction, FrameAction, SuspensionFrame,
};
use std::sync::Arc;

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct NeedsCompleteFrame {
    pub(super) config: NeedsLoadConfig,
}
impl SuspensionFrame for NeedsCompleteFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(session.finish_guest_needs(&self.config, outcome))
    }
}

/// Only BeginPackage's own context frame and package-list tail. Successfully
/// loaded earlier dependencies keep their definitions on a later failure.
#[derive(Clone, rum_derive::FrameRootTrace)]
pub(in crate::runtime) struct BeginPackageRollback {
    pub(super) frame: ContextFrame,
    pub(super) context_depth: usize,
    pub(super) package_count: usize,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct BeginPackageDependencyFrame {
    pub(super) config: Arc<BeginPackageConfig>,
    pub(super) dependency: usize,
    pub(super) begin_result: Expr,
    pub(super) rollback: BeginPackageRollback,
}
impl SuspensionFrame for BeginPackageDependencyFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let mut state = self.as_ref().clone();
        let result = match session.finish_begin_package_dependency(&mut state, outcome) {
            Ok(true) => session.drive_begin_package_dependencies(state),
            Ok(false) => Ok(system_expr("$Failed")),
            Err(error) => Err(error),
        };
        FrameAction::complete(result)
    }
}
impl FrameAbandon for BeginPackageDependencyFrame {
    fn abandon(self: Arc<Self>, context: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        context.rollback_begin_package_dependency(&self.rollback);
        FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(in crate::runtime) struct DeclaredStubLoadFrame {
    pub(in crate::runtime) symbol: Symbol,
}
impl SuspensionFrame for DeclaredStubLoadFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(
            outcome.and_then(|value| session.finish_declared_stub_load(self.symbol, value)),
        )
    }
}

impl Session {
    pub(super) fn finish_guest_needs(
        &mut self,
        config: &NeedsLoadConfig,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let result = outcome?;
        if is_system_symbol_name(&result, "$Failed") {
            self.push_message(
                "Needs",
                "load",
                format!("context `{}` could not be loaded", config.context),
            );
            return Ok(system_expr("$Failed"));
        }
        self.mark_package_loaded(&config.context);
        self.context_path = prepend_context(&config.context, &self.context_path);
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
        Ok(system_expr("Null"))
    }

    pub(super) fn drive_begin_package_dependencies(
        &mut self,
        mut state: BeginPackageDependencyFrame,
    ) -> Result<Expr, EvalError> {
        while let Some(context) = state.config.dependencies.get(state.dependency) {
            let outcome = self.eval_needs(&[Expr::string(context.clone())]);
            if matches!(outcome, Err(EvalError::InputRequired { .. })) {
                crate::resume_or_suspend!(insert self, state);
                return outcome;
            }
            if !self.finish_begin_package_dependency(&mut state, outcome)? {
                return Ok(system_expr("$Failed"));
            }
        }
        self.context_path =
            begin_package_context_path(&state.config.target_context, &state.config.dependencies);
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
        Ok(state.begin_result)
    }

    fn finish_begin_package_dependency(
        &mut self,
        state: &mut BeginPackageDependencyFrame,
        outcome: Result<Expr, EvalError>,
    ) -> Result<bool, EvalError> {
        match outcome {
            Ok(value) if !is_system_symbol_name(&value, "$Failed") => {
                state.dependency += 1;
                Ok(true)
            }
            outcome => {
                self.restore_begin_package_dependency(&state.rollback);
                outcome.map(|_| false)
            }
        }
    }

    pub(in crate::runtime) fn restore_begin_package_dependency(
        &mut self,
        rollback: &BeginPackageRollback,
    ) {
        assert_eq!(
            self.context_stack.get(rollback.context_depth),
            Some(&rollback.frame),
            "BeginPackage must restore its own context frame"
        );
        self.context_stack.truncate(rollback.context_depth);
        self.current_context = rollback.frame.previous_context.clone();
        self.context_path = rollback.frame.previous_context_path.clone();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ContextStack,
        );
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::CurrentContext,
        );
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::ContextPath);
        if self.loaded_packages.len() > rollback.package_count {
            self.loaded_packages.truncate(rollback.package_count);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::LoadedPackages,
            );
            self.bump_evaluation_epoch();
        }
    }
}
