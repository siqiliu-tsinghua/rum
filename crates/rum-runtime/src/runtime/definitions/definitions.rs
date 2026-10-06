mod destructuring;
mod resume_frames;
use super::rule_lhs::continuation::RuleLhsCompletionFrame;
use resume_frames::*;

use super::super::*;
use super::definition_lhs_preparation::{
    DefinitionLhsPreparationContinuation, PreparedDefinitionAssignment, PreparedDefinitionLhs,
};

impl Session {
    pub(in crate::runtime) fn eval_normalize_rules(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr] => self.resume_normalize_rules(expr.clone()),
            _ => Ok(self.invalid_call_with_exact_arg_count("NormalizeRules", args, 1)),
        }
    }

    pub(in crate::runtime) fn eval_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Set", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = if expr_is_self_evaluating_atom(&args[1]) {
            let runtime_profile_rhs = runtime_path_timing_start("set.rhs.evaluate");
            record_profile_runtime_path_timing(runtime_profile_rhs, RuntimeProfileOutcome::Ok);
            let runtime_profile_suspend = runtime_path_timing_start("set.rhs.suspend");
            record_profile_runtime_path_timing(runtime_profile_suspend, RuntimeProfileOutcome::Ok);
            args[1].clone()
        } else {
            let runtime_profile_rhs = runtime_path_timing_start("set.rhs.evaluate");
            let rhs_result = self.evaluate(args[1].clone());
            record_profile_runtime_path_timing(
                runtime_profile_rhs,
                runtime_profile_outcome_for_expr_result(&rhs_result),
            );
            let runtime_profile_suspend = runtime_path_timing_start("set.rhs.suspend");
            let rhs_result = crate::resume_or_suspend!(
                self,
                rhs_result,
                ok(value) => Ok(value),
                suspend => {
            let lhs = lhs.clone();
            EvalSetSuspensionFrame { lhs }
            },
            );
            record_profile_runtime_path_timing(
                runtime_profile_suspend,
                runtime_profile_outcome_for_expr_result(&rhs_result),
            );
            rhs_result?
        };

        let runtime_profile_finish = runtime_path_timing_start("set.finish");
        let result = self.resume_set_with_rhs(lhs, rhs);
        record_profile_runtime_path_timing(
            runtime_profile_finish,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    pub(in crate::runtime) fn eval_set_delayed(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("SetDelayed", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = args[1].clone();
        self.finish_definition_assignment("SetDelayed", lhs, rhs, true)
    }

    pub(in crate::runtime) fn eval_rule(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Rule", args, 2));
        }

        let lhs_result = self.evaluate_rule_lhs(args[0].clone());
        crate::resume_or_suspend!(
            self,
            lhs_result,
            ok(lhs) => self.finish_rule_lhs(lhs, args[1].clone(), false),
            suspend => RuleLhsCompletionFrame { rhs: args[1].clone(), delayed: false },
        )
    }

    pub(in crate::runtime) fn eval_rule_delayed(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("RuleDelayed", args, 2));
        }

        let lhs_result = self.evaluate_rule_lhs(args[0].clone());
        crate::resume_or_suspend!(
            self,
            lhs_result,
            ok(lhs) => self.finish_rule_lhs(lhs, args[1].clone(), true),
            suspend => RuleLhsCompletionFrame { rhs: args[1].clone(), delayed: true },
        )
    }

    #[inline(always)]
    pub(super) fn finish_rule_lhs(
        &mut self,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        if delayed {
            return Ok(Expr::call(system_expr("RuleDelayed"), vec![lhs, rhs]));
        }
        let rhs_result = self.evaluate(rhs);
        let rhs = crate::resume_or_suspend!(
            self,
            rhs_result,
            ok(value) => Ok(value),
            suspend => {
        let lhs = lhs.clone();
        EvalRuleSuspensionFrame { lhs }
        },
        )?;
        self.resume_rule_with_rhs(lhs, rhs)
    }

    fn resume_normalize_rules(&mut self, original: Expr) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(
            original.clone(),
            RewriteRuleExprContinuation::NormalizeRules { original },
        )
    }

    pub(in crate::runtime) fn resume_normalize_rules_after_rule_exprs(
        &mut self,
        original: Expr,
        rule_exprs: Option<Vec<Expr>>,
    ) -> Result<Expr, EvalError> {
        match rule_exprs {
            Some(rule_exprs) => {
                let rules = self
                    .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
                    .expect("normalized rewrite rule expressions should recompile");
                Ok(Self::normalized_rules_expr(&rules))
            }
            None => Ok(self.invalid_call_with_arg_form(
                "NormalizeRules",
                &[original],
                "expected a rule, a rule list, or a symbol that evaluates to rules",
            )),
        }
    }

    fn resume_set_with_rhs(&mut self, lhs: Expr, rhs: Expr) -> Result<Expr, EvalError> {
        if matches!(&lhs, Expr::Atom(Atom::Symbol(target)) if target.context() != SYSTEM_CONTEXT || !matches!(target.name(), "$ContextPath" | "$Packages" | "$Path"))
        {
            let runtime_profile_symbol = runtime_path_timing_start("set.try_symbol_own_value");
            let symbol_result = self.try_set_symbol_own_value_lvalue("Set", &lhs, &rhs);
            record_profile_runtime_path_timing(
                runtime_profile_symbol,
                runtime_profile_outcome_for_bool_result(&symbol_result),
            );
            debug_assert!(matches!(&symbol_result, Ok(true)));
            symbol_result?;
            return Ok(rhs);
        }

        self.finish_definition_assignment("Set", lhs, rhs, false)
    }

    fn resume_rule_with_rhs(&mut self, lhs: Expr, rhs: Expr) -> Result<Expr, EvalError> {
        Ok(Expr::call(system_expr("Rule"), vec![lhs, rhs]))
    }

    fn finish_definition_assignment(
        &mut self,
        symbol: &str,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        if !delayed {
            if is_context_path_symbol_expr(&lhs) {
                let sym = Symbol::system("$ContextPath");
                if !self.ensure_symbol_not_protected_for_write(symbol, &sym) {
                    return Ok(rhs);
                }
                let Some(contexts) = expr_as_string_list(&rhs) else {
                    return Ok(self.invalid_call_with_arg_form(
                        symbol,
                        &[lhs, rhs],
                        "expected $ContextPath to be assigned a list of string contexts",
                    ));
                };
                self.context_path = normalize_context_list(contexts);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::ContextPath,
                );
                self.bump_evaluation_epoch();
                return Ok(rhs);
            }

            if is_packages_symbol_expr(&lhs) {
                let sym = Symbol::system("$Packages");
                if !self.ensure_symbol_not_protected_for_write(symbol, &sym) {
                    return Ok(rhs);
                }
                let Some(packages) = expr_as_string_list(&rhs) else {
                    return Ok(self.invalid_call_with_arg_form(
                        symbol,
                        &[lhs, rhs],
                        "expected $Packages to be assigned a list of string contexts",
                    ));
                };
                self.loaded_packages = normalize_context_list(packages);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::LoadedPackages,
                );
                self.bump_evaluation_epoch();
                return Ok(rhs);
            }

            if is_path_symbol_expr(&lhs) {
                let sym = Symbol::system("$Path");
                if !self.ensure_symbol_not_protected_for_write(symbol, &sym) {
                    return Ok(rhs);
                }
                let Some(paths) = expr_as_string_list(&rhs) else {
                    return Ok(self.invalid_call_with_arg_form(
                        symbol,
                        &[lhs, rhs],
                        "expected $Path to be assigned a list of string paths",
                    ));
                };
                self.package_search_paths =
                    dedup_paths(paths.into_iter().map(PathBuf::from).collect());
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::PackageSearchPaths,
                );
                return Ok(rhs);
            }

            let runtime_profile_symbol = runtime_path_timing_start("set.try_symbol_own_value");
            let symbol_result = self.try_set_symbol_own_value_lvalue(symbol, &lhs, &rhs);
            record_profile_runtime_path_timing(
                runtime_profile_symbol,
                runtime_profile_outcome_for_bool_result(&symbol_result),
            );
            if symbol_result? {
                return Ok(rhs);
            }

            if let Some(target) = self.normalize_options_target_expr(&lhs) {
                if !self.ensure_symbol_not_protected_for_write(symbol, &target) {
                    return Ok(rhs);
                }

                let options = match self.compile_assigned_options_from_resolved(&rhs) {
                    Ok(options) => options,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            symbol,
                            &[lhs, rhs],
                            error.message(),
                        ));
                    }
                };

                self.set_options_for_symbol(target, options);
                return Ok(rhs);
            }

            if let Some((kind, target)) = self.normalize_value_table_target_expr(&lhs) {
                if !self.ensure_symbol_not_protected_for_write(symbol, &target) {
                    return Ok(rhs);
                }

                return self.replace_value_table_from_expr(kind, target, rhs);
            }

            if let Some(target) = self.normalize_syntax_information_target_expr(&lhs) {
                if !self.ensure_symbol_not_protected_for_write(symbol, &target) {
                    return Ok(rhs);
                }

                let syntax_information =
                    match self.compile_assigned_syntax_information_from_resolved(&rhs) {
                        Ok(syntax_information) => syntax_information,
                        Err(error) => {
                            return Ok(self.invalid_call_with_arg_form(
                                symbol,
                                &[lhs, rhs],
                                error.message(),
                            ));
                        }
                    };

                self.set_syntax_information_for_symbol(target, syntax_information);
                return Ok(rhs);
            }
        }

        if !delayed {
            let runtime_profile_list = runtime_path_timing_start("set.try_list_destructuring");
            let list_result =
                self.try_set_list_destructuring_lvalue(symbol, lhs.clone(), rhs.clone());
            record_profile_runtime_path_timing(
                runtime_profile_list,
                runtime_profile_outcome_for_optional_result(&list_result),
            );
            if let Some(value) = list_result? {
                return Ok(value);
            }

            let runtime_profile_assoc = runtime_path_timing_start("set.try_association_lvalue");
            let assoc_result = self.try_set_association_lvalue(lhs.clone(), rhs.clone());
            record_profile_runtime_path_timing(
                runtime_profile_assoc,
                runtime_profile_outcome_for_optional_result(&assoc_result),
            );
            if let Some(value) = assoc_result? {
                return Ok(value);
            }

            if let Some(value) = self.try_set_positional_part_lvalue(lhs.clone(), rhs.clone())? {
                return Ok(value);
            }
        }

        let runtime_profile_prepare_lhs = runtime_path_timing_start("set.prepare_lhs");
        let result = self.prepare_definition_lhs(
            lhs,
            DefinitionLhsPreparationContinuation::Set { rhs, delayed },
        );
        record_profile_runtime_path_timing(
            runtime_profile_prepare_lhs,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    pub(super) fn finish_prepared_definition_assignment(
        &mut self,
        prepared: PreparedDefinitionLhs,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        let symbol = if delayed { "SetDelayed" } else { "Set" };
        let target = prepared.into_assignment();
        let runtime_profile_normalize_target = runtime_path_timing_start("set.normalize_target");
        record_profile_runtime_path_timing(
            runtime_profile_normalize_target,
            if matches!(
                &target,
                PreparedDefinitionAssignment::Message(_)
                    | PreparedDefinitionAssignment::SyntaxInformation { .. }
                    | PreparedDefinitionAssignment::Definition(_)
            ) {
                RuntimeProfileOutcome::Some
            } else {
                RuntimeProfileOutcome::None
            },
        );

        match target {
            PreparedDefinitionAssignment::Message(key) => {
                if !self.ensure_symbol_not_protected_for_write(symbol, &key.symbol) {
                    return Ok(if delayed { system_expr("Null") } else { rhs });
                }
                self.define_message(key, rhs.clone(), delayed);
                Ok(if delayed { system_expr("Null") } else { rhs })
            }
            PreparedDefinitionAssignment::SyntaxInformation {
                lhs,
                symbol: target,
            } => {
                if !self.ensure_symbol_not_protected_for_write(symbol, &target) {
                    return Ok(if delayed { system_expr("Null") } else { rhs });
                }
                Ok(Expr::call(system_expr(symbol), vec![lhs, rhs]))
            }
            PreparedDefinitionAssignment::Definition(prepared) => {
                if !self.ensure_symbol_not_protected_for_write(symbol, prepared.symbol()) {
                    return Ok(if delayed { system_expr("Null") } else { rhs });
                }
                let runtime_profile_store = runtime_path_timing_start("set.store_definition");
                self.store_prepared_definition(prepared, rhs.clone(), delayed);
                record_profile_runtime_path_timing(
                    runtime_profile_store,
                    RuntimeProfileOutcome::Ok,
                );
                Ok(if delayed { system_expr("Null") } else { rhs })
            }
            PreparedDefinitionAssignment::Invalid(lhs) => {
                Ok(Expr::call(system_expr(symbol), vec![lhs, rhs]))
            }
        }
    }

    fn try_set_symbol_own_value_lvalue(
        &mut self,
        symbol: &str,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<bool, EvalError> {
        let runtime_profile_lvalue = runtime_path_timing_start("set.symbol_own_value.lvalue");
        let Some(target) = expr_as_symbol(lhs).cloned() else {
            record_profile_runtime_path_timing(runtime_profile_lvalue, RuntimeProfileOutcome::None);
            return Ok(false);
        };
        record_profile_runtime_path_timing(runtime_profile_lvalue, RuntimeProfileOutcome::Some);

        let runtime_profile_protected =
            runtime_path_timing_start("set.symbol_own_value.protected_check");
        if !self.ensure_symbol_not_protected_for_write(symbol, &target) {
            record_profile_runtime_path_timing(
                runtime_profile_protected,
                RuntimeProfileOutcome::None,
            );
            return Ok(true);
        }
        record_profile_runtime_path_timing(runtime_profile_protected, RuntimeProfileOutcome::Some);

        self.installed_system_own_values.remove(&target);

        let runtime_profile_store = runtime_path_timing_start("set.symbol_own_value.store");
        let (stored_rhs, guards) = split_definition_rhs_guards(rhs.clone());
        let definition = self.definitions.entry(target.clone()).or_default();
        definition.store_own_value(stored_rhs, guards, false);
        record_profile_runtime_path_timing(runtime_profile_store, RuntimeProfileOutcome::Ok);

        let runtime_profile_trace = runtime_path_timing_start("set.symbol_own_value.trace");
        self.push_definition_stored_trace(&target, DefinitionKind::OwnValue, || rhs.clone());
        record_profile_runtime_path_timing(runtime_profile_trace, RuntimeProfileOutcome::Ok);

        let runtime_profile_epoch = runtime_path_timing_start("set.symbol_own_value.epoch_bump");
        self.bump_evaluation_epoch_for_symbol(&target);
        record_profile_runtime_path_timing(runtime_profile_epoch, RuntimeProfileOutcome::Ok);
        Ok(true)
    }

    fn try_set_list_destructuring_lvalue(
        &mut self,
        symbol: &str,
        lhs: Expr,
        rhs: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        if list_expr_args(&lhs).is_none() {
            return Ok(None);
        }

        if !list_destructuring_shapes_match(&lhs, &rhs) {
            self.push_message(
                symbol,
                "shape",
                format!("Lists {lhs} and {rhs} are not the same shape."),
            );
            return Ok(Some(rhs));
        }

        self.store_list_destructuring_assignments(symbol, lhs, rhs)
            .map(Some)
    }
}

fn is_context_path_symbol_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(Atom::Symbol(symbol)) if symbol.context() == SYSTEM_CONTEXT && symbol.name() == "$ContextPath")
}

fn is_packages_symbol_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(Atom::Symbol(symbol)) if symbol.context() == SYSTEM_CONTEXT && symbol.name() == "$Packages")
}

fn is_path_symbol_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(Atom::Symbol(symbol)) if symbol.context() == SYSTEM_CONTEXT && symbol.name() == "$Path")
}

fn normalize_context_list(contexts: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for context in contexts {
        let context = normalize_context_name(context);
        if !normalized.iter().any(|existing| existing == &context) {
            normalized.push(context);
        }
    }
    normalized
}

fn list_expr_args(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::structure::LIST) => {
            Some(args)
        }
        _ => None,
    }
}

fn list_destructuring_shapes_match(lhs: &Expr, rhs: &Expr) -> bool {
    let Some(lhs_items) = list_expr_args(lhs) else {
        return true;
    };

    let Some(rhs_items) = list_expr_args(rhs) else {
        return false;
    };

    lhs_items.len() == rhs_items.len()
        && lhs_items
            .iter()
            .zip(rhs_items)
            .all(|(lhs_item, rhs_item)| list_destructuring_shapes_match(lhs_item, rhs_item))
}

fn runtime_profile_outcome_for_optional_result(
    result: &Result<Option<Expr>, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(Some(_)) => RuntimeProfileOutcome::Some,
        Ok(None) => RuntimeProfileOutcome::None,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

fn runtime_profile_outcome_for_bool_result(
    result: &Result<bool, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(true) => RuntimeProfileOutcome::Some,
        Ok(false) => RuntimeProfileOutcome::None,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}
