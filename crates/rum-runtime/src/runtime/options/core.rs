use super::*;

impl Session {
    /// Resolves a boolean system option for builtin families.
    pub fn system_option_bool(
        &mut self,
        symbol_name: &str,
        option_name: &str,
        fallback: bool,
    ) -> Result<bool, EvalError> {
        let Some(value) = self
            .option_value_for_symbol(&Symbol::system(symbol_name), &Symbol::system(option_name))?
        else {
            return Ok(fallback);
        };

        Ok(is_system_symbol_name(&value, "True"))
    }

    pub(in crate::runtime) fn options_for_symbol(&self, symbol: &Symbol) -> SymbolOptions {
        self.options
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| default_system_options(symbol))
    }

    pub(in crate::runtime) fn set_options_for_symbol(
        &mut self,
        symbol: Symbol,
        options: SymbolOptions,
    ) {
        self.note_active_modules_escaped();
        let default = default_system_options(&symbol);

        if options == default {
            self.options.remove(&symbol);
            self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Options);
        } else {
            self.options.insert(symbol, options);
            self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Options);
        }
    }

    pub(in crate::runtime) fn option_value_for_symbol(
        &mut self,
        symbol: &Symbol,
        option: &Symbol,
    ) -> Result<Option<Expr>, EvalError> {
        let options = self.options_for_symbol(symbol);
        let entry = options
            .entry(option)
            .or_else(|| options.entry_by_name(option.name()))
            .cloned();

        match entry {
            Some(entry) => self.resolve_option_entry_value(&entry).map(Some),
            None => Ok(None),
        }
    }

    /// Returns the resolved option value as an option rule expression for a symbol.
    pub fn option_rule_expr_for_symbol(
        &mut self,
        symbol: &Symbol,
        option: &Symbol,
    ) -> Result<Option<Expr>, EvalError> {
        let options = self.options_for_symbol(symbol);
        let entry = options
            .entry(option)
            .or_else(|| options.entry_by_name(option.name()))
            .cloned();

        let Some(entry) = entry else {
            return Ok(None);
        };
        let value = self.resolve_option_entry_value(&entry)?;
        Ok(Some(Expr::call(
            system_expr("Rule"),
            vec![Expr::Atom(Atom::Symbol(option.clone())), value],
        )))
    }

    pub(in crate::runtime) fn option_rule_exprs_for_symbol(&self, symbol: &Symbol) -> Vec<Expr> {
        self.options_for_symbol(symbol).rule_exprs()
    }

    pub(in crate::runtime) fn merged_option_exprs_for_symbol(
        &mut self,
        symbol: &Symbol,
        explicit: &[Expr],
    ) -> Result<Vec<Expr>, EvalError> {
        let mut merged = Vec::new();
        for expr in self.option_rule_exprs_for_symbol(symbol) {
            merged.push(self.evaluate_runtime_option_expr(&expr)?);
        }
        for expr in explicit {
            merged.push(self.evaluate_runtime_option_expr(expr)?);
        }
        Ok(merged)
    }

    /// Returns evaluated default and explicit options for a System` symbol.
    pub fn merged_option_exprs_for_system_symbol(
        &mut self,
        symbol_name: &str,
        explicit: &[Expr],
    ) -> Result<Vec<Expr>, EvalError> {
        self.merged_option_exprs_for_symbol(&Symbol::system(symbol_name), explicit)
    }

    pub(in crate::runtime) fn raw_option_value_for_symbol(
        &self,
        explicit: Option<&SymbolOptions>,
        target: &Symbol,
        option_name: &str,
    ) -> Option<(Expr, bool)> {
        explicit
            .and_then(|options| options.entry_by_name(option_name).cloned())
            .or_else(|| {
                self.options_for_symbol(target)
                    .entry_by_name(option_name)
                    .cloned()
            })
            .map(|entry| (entry.value, entry.delayed))
    }

    pub(in crate::runtime) fn runtime_option_expr_for_symbol(
        &mut self,
        explicit: Option<&SymbolOptions>,
        target: &Symbol,
        option_name: &str,
    ) -> Result<Option<Expr>, EvalError> {
        let Some((value, delayed)) =
            self.raw_option_value_for_symbol(explicit, target, option_name)
        else {
            return Ok(None);
        };

        if delayed {
            Ok(Some(value))
        } else {
            self.evaluate(value).map(Some)
        }
    }

    pub(in crate::runtime) fn compile_assigned_options_from_resolved(
        &self,
        resolved: &Expr,
    ) -> Result<SymbolOptions, OptionsNormalizationError> {
        let entries = self.parse_option_entries_from_resolved(resolved)?;
        Ok(SymbolOptions::from_entries(entries))
    }

    pub(in crate::runtime::options) fn resolve_option_entry_value(
        &mut self,
        entry: &SymbolOptionEntry,
    ) -> Result<Expr, EvalError> {
        if entry.delayed {
            self.evaluate(entry.value.clone())
        } else {
            Ok(entry.value.clone())
        }
    }

    fn evaluate_runtime_option_expr(&mut self, expr: &Expr) -> Result<Expr, EvalError> {
        let Expr::Call { head, args } = expr else {
            return Ok(expr.clone());
        };

        if is_system_symbol_name(head, "RuleDelayed") && args.len() == 2 {
            let value = self.evaluate(args[1].clone())?;
            Ok(Expr::call(
                system_expr("Rule"),
                vec![args[0].clone(), value],
            ))
        } else {
            Ok(expr.clone())
        }
    }
}
