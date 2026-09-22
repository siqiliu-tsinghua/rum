use super::*;

impl Session {
    pub(in crate::runtime) fn eval_syntax_information(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [target] => {
                let Some(symbol) = self.normalize_symbol_target_expr(target, true) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "SyntaxInformation",
                        args,
                        SyntaxInformationNormalizationError::InvalidTarget.message(),
                    ));
                };

                Ok(self.syntax_information_for_symbol(&symbol))
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("SyntaxInformation", args, 1)),
        }
    }

    pub(in crate::runtime) fn syntax_information_for_symbol(&self, symbol: &Symbol) -> Expr {
        self.syntax_information
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| default_system_syntax_information(symbol))
    }

    pub(in crate::runtime) fn set_syntax_information_for_symbol(
        &mut self,
        symbol: Symbol,
        syntax: Expr,
    ) {
        self.note_active_modules_escaped();
        if syntax == default_system_syntax_information(&symbol) {
            self.syntax_information.remove(&symbol);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::SyntaxInformation,
            );
        } else {
            self.syntax_information.insert(symbol, syntax);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::SyntaxInformation,
            );
        }
    }

    pub(in crate::runtime) fn compile_assigned_syntax_information_from_resolved(
        &self,
        resolved: &Expr,
    ) -> Result<Expr, SyntaxInformationNormalizationError> {
        let Expr::Call { head, args } = resolved else {
            return Err(SyntaxInformationNormalizationError::InvalidRules);
        };

        if !is_system_symbol_name(head, "List") {
            return Err(SyntaxInformationNormalizationError::InvalidRules);
        }

        for item in args {
            let Expr::Call {
                head: rule_head,
                args: rule_args,
            } = item
            else {
                return Err(SyntaxInformationNormalizationError::InvalidRules);
            };

            if !is_system_symbol_name(rule_head, "Rule") || rule_args.len() != 2 {
                return Err(SyntaxInformationNormalizationError::InvalidRules);
            }

            if !matches!(rule_args[0], Expr::Atom(Atom::String(_))) {
                return Err(SyntaxInformationNormalizationError::InvalidRules);
            }
        }

        Ok(resolved.clone())
    }
}

fn empty_list_expr() -> Expr {
    Expr::call(system_expr("List"), Vec::new())
}

fn syntax_rule(name: &str, value: Expr) -> Expr {
    Expr::call(system_expr("Rule"), vec![Expr::string(name), value])
}

fn syntax_argument_pattern(items: Vec<Expr>) -> Expr {
    syntax_rule("ArgumentsPattern", Expr::call(system_expr("List"), items))
}

fn blank_expr() -> Expr {
    Expr::call(system_expr("Blank"), Vec::new())
}

fn blank_null_sequence_expr() -> Expr {
    Expr::call(system_expr("BlankNullSequence"), Vec::new())
}

fn blank_sequence_expr() -> Expr {
    Expr::call(system_expr("BlankSequence"), Vec::new())
}

fn optional_blank_expr() -> Expr {
    Expr::call(system_expr("Optional"), vec![blank_expr()])
}

fn optional_blank_association_expr() -> Expr {
    Expr::call(
        system_expr("Optional"),
        vec![Expr::call(
            system_expr("Blank"),
            vec![system_expr("Association")],
        )],
    )
}

fn default_system_syntax_information(symbol: &Symbol) -> Expr {
    if symbol.context() != SYSTEM_CONTEXT {
        return empty_list_expr();
    }

    let rules = match symbol.name() {
        "Begin" | "FindFile" | "Get" | "Options" | "SyntaxInformation" => {
            vec![syntax_argument_pattern(vec![blank_expr()])]
        }
        "Names" => vec![
            syntax_argument_pattern(vec![optional_blank_expr(), system_expr("OptionsPattern")]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "IgnoreCase".to_string(),
                    "ResolveContextAliases".to_string(),
                    "SpellingCorrection".to_string(),
                ]),
            ),
        ],
        "Protect" => vec![syntax_argument_pattern(vec![blank_null_sequence_expr()])],
        "Unprotect" => vec![syntax_argument_pattern(vec![blank_sequence_expr()])],
        "BeginPackage" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_null_sequence_expr(),
        ])],
        "End" | "EndPackage" => vec![syntax_argument_pattern(Vec::new())],
        "SetOptions" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_null_sequence_expr(),
        ])],
        "OptionValue" => vec![syntax_argument_pattern(vec![blank_null_sequence_expr()])],
        "ArgumentsOptions" | "CheckArguments" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            optional_blank_association_expr(),
        ])],
        "Trace" | "TraceDetail" | "TracePrint" => vec![
            syntax_argument_pattern(vec![blank_expr(), optional_blank_expr()]),
            syntax_rule("OptionNames", string_list_expr(vec!["Heads".to_string()])),
        ],
        "RepeatedTiming" => vec![syntax_argument_pattern(vec![blank_expr()])],
        "Short" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
        ])],
        "On" | "Off" => vec![syntax_argument_pattern(vec![blank_sequence_expr()])],
        "Sow" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
        ])],
        "Reap" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
        ])],
        "TimeConstrained" | "MemoryConstrained" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            optional_blank_expr(),
        ])],
        "Dialog" => vec![
            syntax_argument_pattern(vec![optional_blank_expr(), system_expr("OptionsPattern")]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "DialogProlog".to_string(),
                    "DialogSymbols".to_string(),
                ]),
            ),
        ],
        "MemoryInUse" | "MaxMemoryUsed" | "Stack" => {
            vec![syntax_argument_pattern(Vec::new())]
        }
        "Monitor" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "EvaluationMonitor".to_string(),
                    "StepMonitor".to_string(),
                ]),
            ),
        ],
        "EvaluationData" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
        ])],
        "TestCreate" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            optional_blank_expr(),
        ])],
        "TestReport" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                optional_blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "SameTest".to_string(),
                    "TimeConstraint".to_string(),
                    "MemoryConstraint".to_string(),
                ]),
            ),
        ],
        "StringCount" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "StringDelete" | "StringContainsQ" | "StringStartsQ" | "StringEndsQ" | "StringMatchQ"
        | "StringFreeQ" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "StringPosition" | "StringCases" | "StringReplace" | "StringSplit" | "Cases" | "Count"
        | "MemberQ" | "FreeQ" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "DeleteCases" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                optional_blank_expr(),
                optional_blank_expr(),
                optional_blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule("OptionNames", string_list_expr(vec!["Heads".to_string()])),
        ],
        "Position" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "Replace" | "ReplaceList" | "ReplaceAt" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
        ])],
        "Level" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            optional_blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "Depth" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "CheckAbort" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "ArrayFilter" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            blank_expr(),
            blank_expr(),
            system_expr("OptionsPattern"),
        ])],
        "UpSet" | "UpSetDelayed" => {
            vec![syntax_argument_pattern(vec![blank_expr(), blank_expr()])]
        }
        "MessageName" => vec![syntax_argument_pattern(vec![blank_expr(), blank_expr()])],
        "Message" => vec![syntax_argument_pattern(vec![blank_sequence_expr()])],
        "Needs" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
        ])],
        "Module" | "Block" => vec![syntax_argument_pattern(vec![
            Expr::call(system_expr("List"), vec![blank_sequence_expr()]),
            blank_expr(),
        ])],
        "Function" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            optional_blank_expr(),
            optional_blank_expr(),
        ])],
        "Table" | "Do" => vec![
            syntax_argument_pattern(vec![blank_expr(), blank_sequence_expr()]),
            syntax_rule(
                "LocalVariables",
                Expr::call(
                    system_expr("List"),
                    vec![
                        Expr::string("Table"),
                        Expr::call(
                            system_expr("List"),
                            vec![Expr::integer(BigInt::from(2)), system_expr("Infinity")],
                        ),
                    ],
                ),
            ),
        ],
        "DeclarePackage" => vec![syntax_argument_pattern(vec![
            blank_expr(),
            Expr::call(system_expr("List"), vec![blank_sequence_expr()]),
        ])],
        "Save" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "ExcludedContexts".to_string(),
                    "IncludedContexts".to_string(),
                ]),
            ),
        ],
        "DumpSave" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "CompressionLevel".to_string(),
                    "SymbolAttributes".to_string(),
                ]),
            ),
        ],
        "Once" => vec![
            syntax_argument_pattern(vec![
                blank_expr(),
                optional_blank_expr(),
                system_expr("OptionsPattern"),
            ]),
            syntax_rule(
                "OptionNames",
                string_list_expr(vec![
                    "ExpirationDate".to_string(),
                    "PersistenceTime".to_string(),
                ]),
            ),
        ],
        _ => Vec::new(),
    };

    if rules.is_empty() {
        empty_list_expr()
    } else {
        Expr::call(system_expr("List"), rules)
    }
}
