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

    fn merged_option_exprs_for_symbol<
        M: crate::runtime::accounting_mode::Mode,
        R: OptionReentry,
    >(
        &mut self,
        symbol: &Symbol,
        explicit: &[Expr],
        reentry: R,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let state =
            crate::runtime::accounting_mode::options::snapshot::<M>(self, symbol, explicit)?;
        crate::runtime::accounting_mode::options::drive::<M, R>(
            self, state, symbol, explicit, 0, reentry, head, args,
        )
    }

    /// Evaluates merged options and invokes their typed consumer for a System` symbol.
    /// Only registered types in the installed certified domain enter the loop.
    /// The public domain installer is a trust seam; repository lint certificates
    /// cover the repository declaration. Suspensions own the
    /// descriptor and frozen call; both paths return the consumer's expression,
    /// never the option vector. Ordinary call data stays borrowed.
    pub fn merged_option_exprs_for_system_symbol<R: OptionReentry>(
        &mut self,
        reentry: R,
        head: &Expr,
        args: &[Expr],
        explicit: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !self.check_option_reentry::<R>() {
            return Ok(system_expr("$Failed"));
        }
        if self.current_memory_limit.is_some() {
            self.merged_option_exprs_for_symbol::<crate::runtime::accounting_mode::AccountingShadow, R>(
                &Symbol::system(reentry.symbol_name()), explicit, reentry, head, args,
            )
        } else {
            self.merged_option_exprs_for_symbol::<crate::runtime::accounting_mode::Unarmed, R>(
                &Symbol::system(reentry.symbol_name()),
                explicit,
                reentry,
                head,
                args,
            )
        }
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

    #[inline]
    pub(in crate::runtime) fn evaluate_runtime_option_expr(
        &mut self,
        expr: &Expr,
    ) -> Result<Expr, EvalError> {
        let Expr::Call { head, args } = expr else {
            return crate::runtime::accounting_mode::rule::clone_result(self, expr);
        };

        if is_system_symbol_name(head, "RuleDelayed") && args.len() == 2 {
            crate::runtime::accounting_mode::rule::evaluate_rule(self, &args[0], &args[1])
        } else {
            crate::runtime::accounting_mode::rule::clone_result(self, expr)
        }
    }
}

#[cfg(test)]
#[path = "tests/rule.rs"]
mod rule_tests;
