use super::super::*;

#[cfg(test)]
std::thread_local! {
    static VALUE_Q_ROLLBACK_SNAPSHOT_CAPTURES: std::cell::Cell<u64> =
        const { std::cell::Cell::new(0) };
    static VALUE_Q_SUPPRESSION_SETUPS: std::cell::Cell<u64> =
        const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn record_value_q_rollback_snapshot_capture() {
    VALUE_Q_ROLLBACK_SNAPSHOT_CAPTURES.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
fn record_value_q_suppression_setup() {
    VALUE_Q_SUPPRESSION_SETUPS.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
pub(in crate::runtime) fn take_value_q_trial_setup_counts() -> (u64, u64) {
    (
        VALUE_Q_ROLLBACK_SNAPSHOT_CAPTURES.with(|count| count.replace(0)),
        VALUE_Q_SUPPRESSION_SETUPS.with(|count| count.replace(0)),
    )
}

impl Session {
    pub(in crate::runtime) fn eval_definition(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => {
                    if self.sealed_inspection_hidden(&symbol) {
                        Ok(Expr::call(system_expr("HoldComplete"), Vec::new()))
                    } else {
                        Ok(self.definition_expr_for_symbol(&symbol))
                    }
                }
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("Definition", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("Definition", args, 1)),
        }
    }

    pub(in crate::runtime) fn eval_information(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let [target] = args.as_ref() else {
            return Ok(self.invalid_call_with_arg_range("Information", args, 1, 1));
        };

        let Some(symbol) = self.normalize_symbol_target_expr(target, true) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let include_symbol_state = !self.sealed_inspection_hidden(&symbol);

        Ok(self.information_data_expr_for_symbol(&symbol, include_symbol_state))
    }

    pub(in crate::runtime) fn eval_value_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("ValueQ", args, 1));
        };

        if self.cached_expr_is_evaluation_stable(target) {
            return Ok(system_expr("False"));
        }

        Ok(self.eval_value_q_via_trial(target))
    }

    pub(in crate::runtime) fn eval_value_q_via_trial(&mut self, target: &Expr) -> Expr {
        let original = target.clone();
        let outcome = self.evaluate_value_q_trial(original.clone());

        let value_q = match outcome {
            Ok(value) => value != original,
            Err(EvalError::InputRequired { .. }) => true,
            Err(_) => false,
        };

        system_expr(if value_q { "True" } else { "False" })
    }

    fn evaluate_value_q_trial(&mut self, target: Expr) -> Result<Expr, EvalError> {
        #[cfg(test)]
        record_value_q_rollback_snapshot_capture();
        let snapshot = self.capture_evaluation_rollback_snapshot();
        let saved_prng_state = self.prng_state;
        let saved_print_sink = self.print_sink.take();
        let saved_capability_policy = self.capability_policy;
        let saved_effect_suppression = self.value_q_effect_suppression_active;

        #[cfg(test)]
        record_value_q_suppression_setup();
        self.value_q_effect_suppression_active = true;
        self.capability_policy =
            saved_capability_policy.denying([Capability::FileWrite, Capability::Process]);

        let trial =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.evaluate(target)));

        self.prng_state = saved_prng_state;
        self.print_sink = saved_print_sink;
        self.capability_policy = saved_capability_policy;
        self.value_q_effect_suppression_active = saved_effect_suppression;
        self.restore_evaluation_rollback_snapshot(snapshot);

        match trial {
            Ok(outcome) => outcome,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    pub(in crate::runtime) fn value_q_refuses_capability(&self, capability: Capability) -> bool {
        self.value_q_effect_suppression_active
            && matches!(capability, Capability::FileWrite | Capability::Process)
    }

    pub(in crate::runtime) fn eval_own_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.own_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("OwnValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("OwnValues", args, 1)),
        }
    }

    pub(in crate::runtime) fn eval_up_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.up_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("UpValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("UpValues", args, 1)),
        }
    }

    pub(in crate::runtime) fn eval_down_values(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.down_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("DownValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("DownValues", args, 1)),
        }
    }

    pub(in crate::runtime) fn eval_sub_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.sub_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("SubValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("SubValues", args, 1)),
        }
    }

    pub(in crate::runtime) fn eval_default_values(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.default_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("DefaultValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("DefaultValues", args, 1)),
        }
    }

    pub(in crate::runtime) fn default_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let default_symbol = Symbol::system("Default");
        let mut items = self
            .definitions
            .get(&default_symbol)
            .into_iter()
            .flat_map(|definition| definition.down_values.iter())
            .filter(|entry| default_value_lhs_targets_symbol(&entry.lhs, symbol))
            .map(|entry| {
                Expr::call(
                    system_expr("RuleDelayed"),
                    vec![
                        Expr::call(system_expr("HoldPattern"), vec![entry.lhs.clone()]),
                        rebuild_definition_rhs_guards(entry.rhs.clone(), &entry.guards),
                    ],
                )
            })
            .collect::<Vec<_>>();

        if let Some(syntax_information) = self.syntax_information.get(symbol) {
            items.push(Expr::call(
                system_expr("RuleDelayed"),
                vec![
                    Expr::call(
                        system_expr("HoldPattern"),
                        vec![syntax_information_lhs_expr(symbol)],
                    ),
                    syntax_information.clone(),
                ],
            ));
        }

        Expr::call(system_expr("List"), items)
    }

    fn definition_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        Expr::call(
            system_expr("HoldComplete"),
            self.definition_statements_for_symbol(symbol),
        )
    }

    fn sealed_inspection_hidden(&self, symbol: &Symbol) -> bool {
        !self.sealed_symbols_visible()
            && crate::runtime::scoping::packages::symbol_is_sealed_private(symbol)
    }

    fn definition_statements_for_symbol(&self, symbol: &Symbol) -> Vec<Expr> {
        let mut statements = Vec::new();

        let attributes = self.attribute_set_for_symbol(symbol).expr();
        if !is_empty_list_expr(&attributes) {
            statements.push(definition_assignment_expr(
                false,
                Expr::call(
                    system_expr("Attributes"),
                    vec![Expr::Atom(Atom::Symbol(symbol.clone()))],
                ),
                attributes,
            ));
        }

        statements.extend(self.message_definition_statements_for_symbol(symbol));

        let options = self.options_for_symbol(symbol).expr();
        if !is_empty_list_expr(&options) {
            statements.push(definition_assignment_expr(
                false,
                Expr::call(
                    system_expr("Options"),
                    vec![Expr::Atom(Atom::Symbol(symbol.clone()))],
                ),
                options,
            ));
        }

        statements.extend(
            self.definition_catalog_entries_for_symbol(symbol)
                .into_iter()
                .map(definition_statement_for_catalog_entry),
        );
        statements.extend(self.default_value_definition_statements_for_symbol(symbol));
        if let Some(syntax_information) = self.syntax_information.get(symbol) {
            statements.push(definition_assignment_expr(
                false,
                syntax_information_lhs_expr(symbol),
                syntax_information.clone(),
            ));
        }
        statements
    }

    fn message_definition_statements_for_symbol(&self, symbol: &Symbol) -> Vec<Expr> {
        self.message_definitions
            .iter()
            .filter(|(key, _)| key.symbol == *symbol)
            .filter_map(|(key, definition)| {
                definition
                    .immediate
                    .as_ref()
                    .map(|rhs| {
                        definition_assignment_expr(
                            false,
                            message_name_expr(key.symbol.clone(), key.tag.clone()),
                            rhs.clone(),
                        )
                    })
                    .or_else(|| {
                        definition.delayed.as_ref().map(|rhs| {
                            definition_assignment_expr(
                                true,
                                message_name_expr(key.symbol.clone(), key.tag.clone()),
                                rhs.clone(),
                            )
                        })
                    })
            })
            .collect()
    }

    fn default_value_definition_statements_for_symbol(&self, symbol: &Symbol) -> Vec<Expr> {
        let default_symbol = Symbol::system("Default");
        self.definitions
            .get(&default_symbol)
            .into_iter()
            .flat_map(|definition| definition.down_values.iter())
            .filter(|entry| default_value_lhs_targets_symbol(&entry.lhs, symbol))
            .map(|entry| {
                definition_assignment_expr(
                    entry.delayed,
                    entry.lhs.clone(),
                    rebuild_definition_rhs_guards(entry.rhs.clone(), &entry.guards),
                )
            })
            .collect()
    }

    fn information_data_expr_for_symbol(
        &self,
        symbol: &Symbol,
        include_symbol_state: bool,
    ) -> Expr {
        let empty_list = || Expr::call(system_expr("List"), Vec::new());
        let mut entries = vec![
            information_rule("ObjectType", Expr::string("Symbol")),
            information_rule(
                "Usage",
                if include_symbol_state {
                    self.usage_message_expr_for_symbol(symbol)
                } else {
                    Expr::string(symbol.fully_qualified_name())
                },
            ),
            information_rule("Documentation", system_expr("None")),
            information_rule(
                "Messages",
                if include_symbol_state {
                    self.message_tags_expr_for_symbol(Some(symbol))
                } else {
                    empty_list()
                },
            ),
            information_values_rule(
                "OwnValues",
                symbol,
                if include_symbol_state {
                    self.own_values_expr_for_symbol(symbol)
                } else {
                    empty_list()
                },
            ),
            information_values_rule(
                "UpValues",
                symbol,
                if include_symbol_state {
                    self.up_values_expr_for_symbol(symbol)
                } else {
                    empty_list()
                },
            ),
            information_values_rule(
                "DownValues",
                symbol,
                if include_symbol_state {
                    self.down_values_expr_for_symbol(symbol)
                } else {
                    empty_list()
                },
            ),
            information_values_rule(
                "SubValues",
                symbol,
                if include_symbol_state {
                    self.sub_values_expr_for_symbol(symbol)
                } else {
                    empty_list()
                },
            ),
            information_values_rule(
                "DefaultValues",
                symbol,
                if include_symbol_state {
                    self.default_values_expr_for_symbol(symbol)
                } else {
                    empty_list()
                },
            ),
            information_rule("NValues", system_expr("None")),
            information_rule("FormatValues", system_expr("None")),
            information_rule(
                "Options",
                none_if_empty_list(if include_symbol_state {
                    self.options_for_symbol(symbol).expr()
                } else {
                    empty_list()
                }),
            ),
            information_rule(
                "Attributes",
                if include_symbol_state {
                    self.attribute_set_for_symbol(symbol).expr()
                } else {
                    empty_list()
                },
            ),
            information_rule("FullName", Expr::string(symbol.fully_qualified_name())),
        ];
        if include_symbol_state
            && crate::runtime::scoping::packages::symbol_is_sealed_private(symbol)
        {
            if let Some(unit) = self.sealed_source_unit_for_symbol(symbol) {
                entries.push(information_rule(
                    "SealedUnit",
                    Expr::string(unit.package_context.clone()),
                ));
            }
        }

        Expr::call(
            system_expr("InformationData"),
            vec![Expr::call(system_expr("Association"), entries)],
        )
    }

    fn usage_message_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let key = MessageNameKey {
            symbol: symbol.clone(),
            tag: "usage".to_string(),
        };

        self.message_definitions
            .get(&key)
            .and_then(|definition| {
                definition
                    .immediate
                    .clone()
                    .or_else(|| definition.delayed.clone())
            })
            .or_else(|| {
                self.builtin_spec_for_symbol(symbol)
                    .map(|spec| Expr::string(spec.summary))
            })
            .unwrap_or_else(|| Expr::string(symbol.fully_qualified_name()))
    }
}

fn information_rule(name: &str, value: Expr) -> Expr {
    Expr::call(system_expr("Rule"), vec![Expr::string(name), value])
}

fn definition_statement_for_catalog_entry(entry: DefinitionCatalogEntry) -> Expr {
    definition_assignment_expr(entry.delayed, entry.lhs, entry.rhs)
}

fn definition_assignment_expr(delayed: bool, lhs: Expr, rhs: Expr) -> Expr {
    Expr::call(
        system_expr(if delayed { "SetDelayed" } else { "Set" }),
        vec![lhs, rhs],
    )
}

fn syntax_information_lhs_expr(symbol: &Symbol) -> Expr {
    Expr::call(
        system_expr("SyntaxInformation"),
        vec![Expr::Atom(Atom::Symbol(symbol.clone()))],
    )
}

fn information_values_rule(name: &str, symbol: &Symbol, values: Expr) -> Expr {
    information_rule(name, information_value_form_or_none(name, symbol, values))
}

fn information_value_form_or_none(name: &str, symbol: &Symbol, values: Expr) -> Expr {
    match &values {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") && args.is_empty() => {
            system_expr("None")
        }
        _ => Expr::call(
            Expr::Atom(Atom::Symbol(Symbol::new(
                "Information`",
                "InformationValueForm",
            ))),
            vec![
                system_expr(name),
                Expr::Atom(Atom::Symbol(symbol.clone())),
                values,
            ],
        ),
    }
}

fn none_if_empty_list(expr: Expr) -> Expr {
    match &expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") && args.is_empty() => {
            system_expr("None")
        }
        _ => expr,
    }
}

fn is_empty_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args } if is_system_symbol_name(head, "List") && args.is_empty()
    )
}
