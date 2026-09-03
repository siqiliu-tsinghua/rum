use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
enum OptionValueQuery {
    Single { original: Expr, name: Symbol },
    Many(Vec<(Expr, Symbol)>),
}

impl Session {
    pub(in crate::runtime) fn eval_options(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [target] => {
                let Some(symbol) = self.normalize_symbol_target_expr(target, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Options",
                        args,
                        OptionsNormalizationError::InvalidTarget.message(),
                    ));
                };

                Ok(self.options_for_symbol(&symbol).expr())
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("Options", args, 1)),
        }
    }

    pub(in crate::runtime) fn eval_set_options(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() < 2 {
            return Ok(self.invalid_call_with_arg_range("SetOptions", args_slice, 2, usize::MAX));
        }

        let Some(mut targets) = self.normalize_symbol_target_exprs(&args_slice[0], true) else {
            return Ok(self.invalid_call_with_arg_form(
                "SetOptions",
                args_slice,
                OptionsNormalizationError::InvalidTarget.message(),
            ));
        };
        let preserve_target_list_shape = matches!(&args_slice[0], Expr::Call { head, .. } if is_system_symbol_name(head, "List"));

        let tail = Expr::call(system_expr("List"), args_slice[1..].to_vec());
        let resolved_result = self.evaluate(tail);
        let resolved = crate::resume_or_suspend!(
            self,
            resolved_result,
            ok(value) => Ok(value),
            suspend => {
        let targets = Arc::<[Symbol]>::from(std::mem::take(&mut targets));
                    let args = args.clone();
        EvalSetOptionsSuspensionFrame { targets, preserve_target_list_shape, args }
        },
        )?;

        self.finish_set_options_from_resolved(
            &targets,
            preserve_target_list_shape,
            args_slice,
            resolved,
        )
    }

    pub(in crate::runtime) fn eval_option_value(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        match args_slice {
            [option_expr] => {
                let query = match self.parse_option_value_query(option_expr) {
                    Ok(query) => query,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            "OptionValue",
                            args,
                            error.message(),
                        ));
                    }
                };
                let Some(scope) = self.option_value_scopes.last().cloned() else {
                    return Ok(self.invalid_call_with_arg_range("OptionValue", args, 2, 3));
                };

                self.resolve_option_value_query(scope.target, Some(scope.explicit), &query)
            }
            [target_expr, option_expr] => {
                let Some(target) = self.normalize_symbol_target_expr(target_expr, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "OptionValue",
                        args,
                        OptionsNormalizationError::InvalidTarget.message(),
                    ));
                };
                let query = match self.parse_option_value_query(option_expr) {
                    Ok(query) => query,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            "OptionValue",
                            args,
                            error.message(),
                        ));
                    }
                };

                self.resolve_option_value_query(target, None, &query)
            }
            [target_expr, explicit_expr, option_expr] => {
                let Some(target) = self.normalize_symbol_target_expr(target_expr, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "OptionValue",
                        args,
                        OptionsNormalizationError::InvalidTarget.message(),
                    ));
                };
                let query = match self.parse_option_value_query(option_expr) {
                    Ok(query) => query,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            "OptionValue",
                            args,
                            error.message(),
                        ));
                    }
                };

                let mut query = Some(query);
                let explicit_result = self.evaluate(explicit_expr.clone());
                let explicit = crate::resume_or_suspend!(
                    self,
                    explicit_result,
                    ok(value) => Ok(value),
                    suspend => {
                let target = target.clone();
                                    let query = Arc::new(
                                        query
                                            .take()
                                            .expect("option query is frozen only after input suspension"),
                                    );
                                    let args = args.clone();
                EvalOptionValueSuspensionFrame { target, query, args }
                },
                )?;

                self.finish_option_value_from_explicit(
                    target,
                    query.as_ref().expect("ordinary option query remains owned"),
                    args_slice,
                    explicit,
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("OptionValue", args_slice, 1, 3)),
        }
    }

    fn finish_set_options_from_resolved(
        &mut self,
        targets: &[Symbol],
        preserve_target_list_shape: bool,
        original_args: &[Expr],
        resolved: Expr,
    ) -> Result<Expr, EvalError> {
        let updates = match self.parse_option_entries_from_resolved_set_options(&resolved) {
            Ok(updates) => updates,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "SetOptions",
                    original_args,
                    error.message(),
                ));
            }
        };

        let results = targets
            .iter()
            .map(|target| self.apply_set_options_updates(target, original_args, &updates))
            .collect::<Vec<_>>();

        if preserve_target_list_shape {
            Ok(Expr::call(system_expr("List"), results))
        } else {
            Ok(results
                .into_iter()
                .next()
                .unwrap_or_else(|| self.invalid_call("SetOptions", original_args)))
        }
    }

    fn apply_set_options_updates(
        &mut self,
        target: &Symbol,
        original_args: &[Expr],
        updates: &[SymbolOptionEntry],
    ) -> Expr {
        if !self.ensure_symbol_not_protected_for_write("SetOptions", target) {
            return self.invalid_call("SetOptions", original_args);
        }

        let mut current = self.options_for_symbol(target);

        for update in updates {
            if !current.contains_key(&update.name) && !current.contains_name(update.name.name()) {
                self.push_message(
                    "SetOptions",
                    "optnf",
                    format!("{} is not a known option for {target}.", update.name),
                );
                return self.invalid_call("SetOptions", original_args);
            }
        }

        for update in updates {
            let replaced = current.set_existing(&update.name, update.value.clone(), update.delayed);
            if !replaced {
                let _ = current.set_existing_by_name(
                    update.name.name(),
                    update.value.clone(),
                    update.delayed,
                );
            }
        }

        self.set_options_for_symbol(target.clone(), current.clone());
        current.expr()
    }

    fn finish_option_value_from_explicit(
        &mut self,
        target: Symbol,
        query: &OptionValueQuery,
        original_args: &[Expr],
        explicit_resolved: Expr,
    ) -> Result<Expr, EvalError> {
        let explicit = match self.parse_option_entries_from_resolved(&explicit_resolved) {
            Ok(entries) => SymbolOptions::from_entries(entries),
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "OptionValue",
                    original_args,
                    error.message(),
                ));
            }
        };

        self.resolve_option_value_query(target, Some(explicit), query)
    }

    fn resolve_option_value_query(
        &mut self,
        target: Symbol,
        explicit: Option<SymbolOptions>,
        query: &OptionValueQuery,
    ) -> Result<Expr, EvalError> {
        match query {
            OptionValueQuery::Single { original, name } => {
                self.resolve_option_value_scalar(target, explicit, original.clone(), *name)
            }
            OptionValueQuery::Many(items) => {
                let mut values = Vec::with_capacity(items.len());
                for (original, name) in items {
                    values.push(self.resolve_option_value_scalar(
                        target.clone(),
                        explicit.clone(),
                        original.clone(),
                        *name,
                    )?);
                }
                Ok(Expr::call(system_expr("List"), values))
            }
        }
    }

    fn resolve_option_value_scalar(
        &mut self,
        target: Symbol,
        explicit: Option<SymbolOptions>,
        original_option_expr: Expr,
        option_name: Symbol,
    ) -> Result<Expr, EvalError> {
        let defaults = self.options_for_symbol(&target);

        if let Some(explicit) = explicit {
            if let Some(entry) = explicit
                .entry(&option_name)
                .or_else(|| explicit.entry_by_name(option_name.name()))
                .cloned()
            {
                if !defaults.contains_key(&option_name)
                    && !defaults.contains_name(option_name.name())
                {
                    self.push_message(
                        "OptionValue",
                        "nodef",
                        format!("Unknown option {option_name} for {target}."),
                    );
                }
                return self.resolve_option_entry_value(&entry);
            }
        }

        if let Some(entry) = defaults
            .entry(&option_name)
            .or_else(|| defaults.entry_by_name(option_name.name()))
            .cloned()
        {
            self.resolve_option_entry_value(&entry)
        } else {
            self.push_message(
                "OptionValue",
                "optnf",
                format!("Option name {option_name} not found in defaults for {target}."),
            );
            Ok(original_option_expr)
        }
    }

    fn parse_option_value_query(
        &self,
        expr: &Expr,
    ) -> Result<OptionValueQuery, OptionsNormalizationError> {
        if let Some(name) = self.normalize_symbol_target_expr(expr, true) {
            return Ok(OptionValueQuery::Single {
                original: expr.clone(),
                name,
            });
        }

        let Expr::Call { head, args, .. } = expr else {
            return Err(OptionsNormalizationError::InvalidOptionName);
        };

        if !is_system_symbol_name(head, "List") {
            return Err(OptionsNormalizationError::InvalidOptionName);
        }

        let mut names = Vec::with_capacity(args.len());

        for item in args {
            let Some(name) = self.normalize_symbol_target_expr(item, true) else {
                return Err(OptionsNormalizationError::InvalidOptionName);
            };
            names.push((item.clone(), name));
        }

        Ok(OptionValueQuery::Many(names))
    }

    fn parse_option_entries_from_resolved_set_options(
        &self,
        expr: &Expr,
    ) -> Result<Vec<SymbolOptionEntry>, OptionsNormalizationError> {
        let Expr::Call { head, args, .. } = expr else {
            return Err(OptionsNormalizationError::InvalidRules);
        };

        if !is_system_symbol_name(head, "List") {
            return Err(OptionsNormalizationError::InvalidRules);
        }

        let mut entries = Vec::new();

        for item in args {
            entries.extend(self.parse_option_entries_from_resolved(item)?);
        }

        if entries.is_empty() {
            return Err(OptionsNormalizationError::InvalidRules);
        }

        Ok(entries)
    }

    pub(in crate::runtime::options) fn parse_option_entries_from_resolved(
        &self,
        expr: &Expr,
    ) -> Result<Vec<SymbolOptionEntry>, OptionsNormalizationError> {
        match expr {
            Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
                let mut entries = Vec::new();
                for item in args {
                    entries.extend(self.parse_option_entries_from_resolved(item)?);
                }
                Ok(entries)
            }
            Expr::Call { head, args, .. }
                if (is_system_symbol_name(head, "Rule")
                    || is_system_symbol_name(head, "RuleDelayed"))
                    && args.len() == 2 =>
            {
                let Some(name) = self.normalize_symbol_target_expr(&args[0], true) else {
                    return Err(OptionsNormalizationError::InvalidRules);
                };
                Ok(vec![SymbolOptionEntry {
                    name,
                    value: args[1].clone(),
                    delayed: is_system_symbol_name(head, "RuleDelayed"),
                }])
            }
            _ => Err(OptionsNormalizationError::InvalidRules),
        }
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<OptionValueQuery>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalSetOptionsSuspensionFrame {
    targets: std::sync::Arc<[rum_core::Symbol]>,
    preserve_target_list_shape: bool,
    args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalSetOptionsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut targets = self.targets.clone();
        #[allow(unused_mut)]
        let mut preserve_target_list_shape = self.preserve_target_list_shape.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(resolved) => (|| {
                session.finish_set_options_from_resolved(
                    targets.as_ref(),
                    preserve_target_list_shape,
                    args.as_ref(),
                    resolved,
                )
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalOptionValueSuspensionFrame {
    pub(super) target: rum_core::Symbol,
    pub(super) query: std::sync::Arc<crate::runtime::options::builtins::OptionValueQuery>,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalOptionValueSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        #[allow(unused_mut)]
        let mut query = self.query.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(resolved) => (|| {
                session.finish_option_value_from_explicit(
                    target,
                    query.as_ref(),
                    args.as_ref(),
                    resolved,
                )
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
