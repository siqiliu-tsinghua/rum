use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct SuspendedModuleInitializers {
    items: Arc<[(Symbol, Option<Expr>)]>,
    body: Expr,
    renames: Arc<SymbolExprMap>,
    activation_depth: usize,
    index: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_module(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.record_profile_eval_module_entry();
        let profile_start = lexical_profile_start();
        let eval_cache_profile_start =
            evaluation_cache_profile_source_timing_start("eval_module_by_source");

        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Module", args, 2));
        }

        let runtime_profile_compile = runtime_path_timing_start("module.compile_bindings");
        let config_result = self.compile_scoped_call_from_raw_args(args);
        record_profile_runtime_path_timing(
            runtime_profile_compile,
            if config_result.is_ok() {
                RuntimeProfileOutcome::Ok
            } else {
                RuntimeProfileOutcome::Error
            },
        );
        let config = match config_result {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Module", args, error.message()));
            }
        };
        let items = config.items;

        let mut renames = SymbolExprMap::default();

        let runtime_profile_allocate = runtime_path_timing_start("module.allocate_fresh_symbols");
        for (symbol, _) in &items {
            renames.insert(
                symbol.clone(),
                Expr::Atom(Atom::Symbol(self.allocate_fresh_symbol(symbol))),
            );
        }
        record_profile_runtime_path_timing(runtime_profile_allocate, RuntimeProfileOutcome::Ok);

        let gensyms: Vec<Symbol> = renames
            .values()
            .filter_map(|expr| expr_as_symbol(expr).cloned())
            .collect();
        let activation_depth = self.module_activations.len();
        let side_table_stamp = self.current_side_table_stamp();
        self.module_activations.push(ModuleActivation {
            gensyms,
            escaped: false,
            side_table_stamp,
        });

        let runtime_profile_total = runtime_path_timing_start("module.eval.total");
        let result = self.resume_module_initializers(
            items,
            args[1].clone(),
            renames,
            activation_depth,
            0,
            None,
        );
        record_profile_eval_module(profile_start);
        record_profile_eval_module_by_source(eval_cache_profile_start);
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    fn resume_module_initializers(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        renames: SymbolExprMap,
        activation_depth: usize,
        mut index: usize,
        mut pending: Option<Result<Expr, EvalError>>,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_resume = runtime_path_timing_start("module.resume_initializers");
        loop {
            if let Some(outcome) = pending.take() {
                let symbol = items[index].0.clone();
                match outcome {
                    Ok(value) => {
                        self.store_module_initializer_value(&renames, &symbol, value);
                        index += 1;
                    }
                    Err(error) => {
                        let result = self.finish_module_scope(activation_depth, Err(error));
                        record_profile_runtime_path_timing(
                            runtime_profile_resume,
                            runtime_profile_outcome_for_expr_result(&result),
                        );
                        return result;
                    }
                }
            }

            record_profile_module_resume_entry(index);
            if index >= items.len() {
                let profile_start = lexical_profile_start();
                let eval_cache_profile_start = evaluation_cache_profile_source_timing_start(
                    "module_body_substitution_by_source",
                );
                let body = substitute_lexical_bindings(&body, &renames, &BTreeSet::new());
                record_profile_module_body_substitution(profile_start);
                record_profile_module_body_substitution_by_source(eval_cache_profile_start);
                let runtime_profile_body = runtime_path_timing_start("module.body.evaluate");
                let result = self.evaluate(body);
                record_profile_runtime_path_timing(
                    runtime_profile_body,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                let outcome = crate::resume_or_suspend!(
                    self,
                    result,
                    ok(value) => Ok(value),
                    suspend => ResumeModuleInitializersSuspensionFrame1 { activation_depth },
                );

                let result = match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.finish_module_scope(activation_depth, outcome),
                };
                record_profile_runtime_path_timing(
                    runtime_profile_resume,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                return result;
            }

            let init = items[index].1.clone();
            if let Some(init) = init {
                let profile_start = lexical_profile_start();
                let eval_cache_profile_start = evaluation_cache_profile_source_timing_start(
                    "module_init_substitution_by_source",
                );
                let renamed_init = substitute_lexical_bindings(&init, &renames, &BTreeSet::new());
                record_profile_module_init_substitution(profile_start);
                record_profile_module_init_substitution_by_source(eval_cache_profile_start);
                let runtime_profile_init = runtime_path_timing_start("module.initializer.evaluate");
                let init_result = self.evaluate(renamed_init);
                record_profile_runtime_path_timing(
                    runtime_profile_init,
                    runtime_profile_outcome_for_expr_result(&init_result),
                );
                match init_result {
                    Err(EvalError::InputRequired { request }) => {
                        let state = SuspendedModuleInitializers {
                            items: Arc::from(items),
                            body,
                            renames: Arc::new(renames),
                            activation_depth,
                            index,
                        };
                        crate::resume_or_suspend!(insert self, ResumeModuleInitializersSuspensionFrame2 { state });
                        let result = Err(EvalError::InputRequired { request });
                        record_profile_runtime_path_timing(
                            runtime_profile_resume,
                            runtime_profile_outcome_for_expr_result(&result),
                        );
                        return result;
                    }
                    outcome => pending = Some(outcome),
                }
                continue;
            }

            index += 1;
        }
    }

    fn resume_suspended_module_initializers(
        &mut self,
        mut state: SuspendedModuleInitializers,
        mut pending: Option<Result<Expr, EvalError>>,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_resume = runtime_path_timing_start("module.resume_initializers");
        loop {
            if let Some(outcome) = pending.take() {
                let symbol = state.items[state.index].0;
                match outcome {
                    Ok(value) => {
                        self.store_module_initializer_value(&state.renames, &symbol, value);
                        state.index += 1;
                    }
                    Err(error) => {
                        let result = self.finish_module_scope(state.activation_depth, Err(error));
                        record_profile_runtime_path_timing(
                            runtime_profile_resume,
                            runtime_profile_outcome_for_expr_result(&result),
                        );
                        return result;
                    }
                }
            }

            record_profile_module_resume_entry(state.index);
            if state.index >= state.items.len() {
                let profile_start = lexical_profile_start();
                let eval_cache_profile_start = evaluation_cache_profile_source_timing_start(
                    "module_body_substitution_by_source",
                );
                let body =
                    substitute_lexical_bindings(&state.body, &state.renames, &BTreeSet::new());
                record_profile_module_body_substitution(profile_start);
                record_profile_module_body_substitution_by_source(eval_cache_profile_start);
                let runtime_profile_body = runtime_path_timing_start("module.body.evaluate");
                let result = self.evaluate(body);
                record_profile_runtime_path_timing(
                    runtime_profile_body,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                let outcome = crate::resume_or_suspend!(
                    self,
                    result,
                    ok(value) => Ok(value),
                    suspend => {
                let activation_depth = state.activation_depth;
                ResumeSuspendedModuleInitializersSuspensionFrame1 { activation_depth }
                },
                );

                let result = match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.finish_module_scope(state.activation_depth, outcome),
                };
                record_profile_runtime_path_timing(
                    runtime_profile_resume,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                return result;
            }

            let init = state.items[state.index].1.clone();
            if let Some(init) = init {
                let profile_start = lexical_profile_start();
                let eval_cache_profile_start = evaluation_cache_profile_source_timing_start(
                    "module_init_substitution_by_source",
                );
                let renamed_init =
                    substitute_lexical_bindings(&init, &state.renames, &BTreeSet::new());
                record_profile_module_init_substitution(profile_start);
                record_profile_module_init_substitution_by_source(eval_cache_profile_start);
                let runtime_profile_init = runtime_path_timing_start("module.initializer.evaluate");
                let init_result = self.evaluate(renamed_init);
                record_profile_runtime_path_timing(
                    runtime_profile_init,
                    runtime_profile_outcome_for_expr_result(&init_result),
                );
                match init_result {
                    Err(EvalError::InputRequired { request }) => {
                        crate::resume_or_suspend!(insert self, ResumeSuspendedModuleInitializersSuspensionFrame2 { state });
                        let result = Err(EvalError::InputRequired { request });
                        record_profile_runtime_path_timing(
                            runtime_profile_resume,
                            runtime_profile_outcome_for_expr_result(&result),
                        );
                        return result;
                    }
                    outcome => pending = Some(outcome),
                }
                continue;
            }

            state.index += 1;
        }
    }

    fn store_module_initializer_value(
        &mut self,
        renames: &SymbolExprMap,
        symbol: &Symbol,
        value: Expr,
    ) {
        let Some(fresh_expr) = renames.get(symbol) else {
            return;
        };
        let Some(fresh_symbol) = expr_as_symbol(fresh_expr).cloned() else {
            return;
        };

        let definition = self.definitions.entry(fresh_symbol).or_default();
        definition.store_own_value(value, Box::default(), false);
        self.bump_evaluation_epoch_for_symbol(&fresh_symbol);
    }

    fn finish_module_scope(
        &mut self,
        activation_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let activation = self.module_activations[activation_depth].clone();
        let escaped =
            activation.escaped || self.current_side_table_stamp() != activation.side_table_stamp;

        let mut reclaim = Vec::new();
        if !escaped {
            match &outcome {
                Ok(result) => {
                    reclaim =
                        self.module_gensyms_to_reclaim(&activation, std::slice::from_ref(result));
                }
                Err(_) => {
                    let roots = gensym_gc_extra_roots_for_outcome(&outcome);
                    reclaim = self.module_gensyms_to_reclaim(&activation, &roots);
                }
            }
        }

        let reclaim_set: BTreeSet<Symbol> = reclaim.iter().copied().collect();
        self.remove_gc_candidate_gensyms(&reclaim);
        // Advisory only: a stale entry is rechecked by mark/sweep; a missing
        // entry only skips reclamation. Rollback restores the advisory set and
        // rearms its watermark from the restored size.
        for gensym in &activation.gensyms {
            if !reclaim_set.contains(gensym) {
                self.gc_candidate_gensyms.insert(*gensym);
            }
        }

        self.module_activations.truncate(activation_depth);
        for gensym in reclaim {
            self.clear_symbol_state(&gensym, true, true, true, true, true);
        }

        if self.gc_candidate_gensyms.len() >= self.gensym_gc_watermark {
            let extra_roots = gensym_gc_extra_roots_for_outcome(&outcome);
            self.collect_and_sweep_dead_gensyms(&extra_roots);
        }

        outcome
    }

    pub(in crate::runtime) fn abandon_module_scope(&mut self, activation_depth: usize) {
        let _ = self.finish_module_scope(activation_depth, Err(EvalError::Cancelled));
    }

    fn module_gensyms_to_reclaim(
        &self,
        activation: &ModuleActivation,
        roots: &[Expr],
    ) -> Vec<Symbol> {
        let candidates = activation
            .gensyms
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        let mut retained = std::collections::HashSet::with_capacity(candidates.len());
        let mut worklist = Vec::new();

        for root in roots {
            seed_live_gensyms_from_expr(root, &candidates, &mut retained, &mut worklist);
        }

        while let Some(retained_gensym) = worklist.pop() {
            let Some(definition) = self.definitions.get(&retained_gensym) else {
                continue;
            };

            for_each_definition_expr(definition, &mut |expr| {
                seed_live_gensyms_from_expr(expr, &candidates, &mut retained, &mut worklist);
            });
        }

        activation
            .gensyms
            .iter()
            .filter(|gensym| !retained.contains(*gensym))
            .copied()
            .collect()
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedModuleInitializers>();
};

#[derive(rum_derive::FrameRootTrace)]
struct ResumeModuleInitializersSuspensionFrame1 {
    activation_depth: usize,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeModuleInitializersSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut activation_depth = self.activation_depth.clone();
        let resumed = (|| session.finish_module_scope(activation_depth, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeModuleInitializersSuspensionFrame1 {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.finish_module_activation(self.activation_depth);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct ResumeModuleInitializersSuspensionFrame2 {
    pub(super) state: crate::runtime::scoping::lexical::module::SuspendedModuleInitializers,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeModuleInitializersSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_module_initializers(state, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeModuleInitializersSuspensionFrame2 {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.finish_module_activation(self.state.activation_depth);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct ResumeSuspendedModuleInitializersSuspensionFrame1 {
    activation_depth: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedModuleInitializersSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut activation_depth = self.activation_depth.clone();
        let resumed = (|| session.finish_module_scope(activation_depth, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon
    for ResumeSuspendedModuleInitializersSuspensionFrame1
{
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.finish_module_activation(self.activation_depth);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct ResumeSuspendedModuleInitializersSuspensionFrame2 {
    pub(super) state: crate::runtime::scoping::lexical::module::SuspendedModuleInitializers,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedModuleInitializersSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_module_initializers(state, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon
    for ResumeSuspendedModuleInitializersSuspensionFrame2
{
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.finish_module_activation(self.state.activation_depth);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}
