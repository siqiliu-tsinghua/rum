use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArgumentCountSpec {
    Exact(usize),
    Range { min: usize, max: usize },
}

impl ArgumentCountSpec {
    fn min(self) -> usize {
        match self {
            Self::Exact(count) => count,
            Self::Range { min, .. } => min,
        }
    }

    fn max(self) -> usize {
        match self {
            Self::Exact(count) => count,
            Self::Range { max, .. } => max,
        }
    }

    fn too_few_message_tag(self) -> &'static str {
        match self {
            Self::Exact(_) => "argx",
            Self::Range { .. } => "argt",
        }
    }
}

enum OptionSuffixClassification {
    Valid(Vec<Expr>),
    UnknownOption(Symbol),
    NotOptionSyntax,
}

enum ArgumentPartitionOutcome {
    Success {
        positional: Vec<Expr>,
        options: Vec<Expr>,
    },
    Failure {
        tag: &'static str,
        details: Expr,
        text: String,
    },
}

fn parse_argument_count_spec(expr: &Expr) -> Option<ArgumentCountSpec> {
    if let Some(count) = expr_as_non_negative_usize(expr) {
        return Some(ArgumentCountSpec::Exact(count));
    }

    let items = call_items_with_head(expr, &system_expr("List"))?;
    match items {
        [min, max] => {
            let min = expr_as_non_negative_usize(min)?;
            let max = expr_as_non_negative_usize(max)?;
            (min <= max).then_some(ArgumentCountSpec::Range { min, max })
        }
        _ => None,
    }
}

fn classify_option_suffix(
    args: &[Expr],
    known_options: &SymbolOptions,
) -> OptionSuffixClassification {
    let mut options = Vec::new();

    for arg in args {
        match classify_option_expr(arg, known_options) {
            Some(Ok(mut entries)) => options.append(&mut entries),
            Some(Err(option_name)) => {
                return OptionSuffixClassification::UnknownOption(option_name);
            }
            None => return OptionSuffixClassification::NotOptionSyntax,
        }
    }

    OptionSuffixClassification::Valid(options)
}

fn classify_option_expr(
    expr: &Expr,
    known_options: &SymbolOptions,
) -> Option<Result<Vec<Expr>, Symbol>> {
    match expr {
        Expr::Call { head, args, .. }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            let name = match &args[0] {
                Expr::Atom(Atom::Symbol(symbol)) => symbol.clone(),
                Expr::Atom(Atom::String(name)) => resolve_unqualified_symbol(name)?,
                _ => return None,
            };

            if known_options.contains_key(&name) || known_options.contains_name(name.name()) {
                Some(Ok(vec![expr.clone()]))
            } else {
                Some(Err(name))
            }
        }
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
            let mut flattened = Vec::new();
            for item in args {
                match classify_option_expr(item, known_options)? {
                    Ok(mut entries) => flattened.append(&mut entries),
                    Err(option_name) => return Some(Err(option_name)),
                }
            }
            Some(Ok(flattened))
        }
        _ => None,
    }
}

fn hold_form_expr(expr: Expr) -> Expr {
    Expr::call(system_expr("HoldForm"), vec![expr])
}

fn held_expr(expr: Expr) -> Expr {
    Expr::call(system_expr("Hold"), vec![expr])
}

fn build_argument_partition_failure(tag: &str, details: Expr) -> Expr {
    Expr::call(system_expr("Failure"), vec![Expr::string(tag), details])
}

fn build_argument_count_failure_details(
    symbol: &Symbol,
    held_input: Expr,
    count: usize,
    spec: ArgumentCountSpec,
) -> Expr {
    let mut entries = vec![
        (
            Expr::string("MessageTemplate"),
            message_name_expr(symbol.clone(), spec.too_few_message_tag().to_string()),
        ),
        (Expr::string("HeldInput"), held_expr(held_input.clone())),
    ];

    let params = match spec {
        ArgumentCountSpec::Exact(expected) => vec![
            hold_form_expr(Expr::Atom(Atom::Symbol(symbol.clone()))),
            hold_form_expr(Expr::integer(BigInt::from(count))),
            hold_form_expr(Expr::integer(BigInt::from(expected))),
        ],
        ArgumentCountSpec::Range { min, max } => vec![
            hold_form_expr(Expr::Atom(Atom::Symbol(symbol.clone()))),
            hold_form_expr(Expr::integer(BigInt::from(count))),
            hold_form_expr(Expr::integer(BigInt::from(min))),
            hold_form_expr(Expr::integer(BigInt::from(max))),
        ],
    };

    entries.push((
        Expr::string("MessageParameters"),
        Expr::call(system_expr("List"), params),
    ));
    render_association(entries)
}

fn build_non_option_failure_details(
    symbol: &Symbol,
    held_input: Expr,
    offender: Expr,
    positional_count: usize,
) -> Expr {
    render_association(vec![
        (
            Expr::string("MessageTemplate"),
            message_name_expr(symbol.clone(), "nonopt".to_string()),
        ),
        (
            Expr::string("MessageParameters"),
            Expr::call(
                system_expr("List"),
                vec![
                    hold_form_expr(offender),
                    hold_form_expr(Expr::integer(BigInt::from(positional_count))),
                    hold_form_expr(held_input.clone()),
                ],
            ),
        ),
        (Expr::string("HeldInput"), held_expr(held_input)),
    ])
}

fn build_unknown_option_failure_details(
    symbol: &Symbol,
    held_input: Expr,
    option_name: &Symbol,
) -> Expr {
    render_association(vec![
        (
            Expr::string("MessageTemplate"),
            message_name_expr(symbol.clone(), "optx".to_string()),
        ),
        (
            Expr::string("MessageParameters"),
            Expr::call(
                system_expr("List"),
                vec![
                    hold_form_expr(Expr::Atom(Atom::Symbol(option_name.clone()))),
                    hold_form_expr(held_input.clone()),
                ],
            ),
        ),
        (Expr::string("HeldInput"), held_expr(held_input)),
    ])
}

fn render_argument_count_failure_message(
    symbol: &Symbol,
    count: usize,
    spec: ArgumentCountSpec,
) -> String {
    match spec {
        ArgumentCountSpec::Exact(expected) => {
            let noun = if expected == 1 {
                "argument is"
            } else {
                "arguments are"
            };
            format!("{symbol} called with {count} arguments; {expected} {noun} expected.")
        }
        ArgumentCountSpec::Range { min, max } => format!(
            "{symbol} called with {count} arguments; {min} or {max} arguments are expected."
        ),
    }
}

fn render_non_option_failure_message(
    _symbol: &Symbol,
    offender: &Expr,
    positional_count: usize,
    held_input: &Expr,
) -> String {
    format!(
        "Options expected (instead of {offender}) beyond position {positional_count} in {held_input}. An option must be a rule or a list of rules."
    )
}

fn render_unknown_option_failure_message(
    _symbol: &Symbol,
    option_name: &Symbol,
    held_input: &Expr,
) -> String {
    format!("Unknown option {option_name} in {held_input}.")
}

impl Session {
    pub(in crate::runtime) fn eval_arguments_options(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (call, spec, assoc) = match args {
            [call_expr, spec_expr] => (call_expr, spec_expr, None),
            [call_expr, spec_expr, assoc_expr] => (call_expr, spec_expr, Some(assoc_expr)),
            _ => return Ok(self.invalid_call_with_arg_range("ArgumentsOptions", args, 2, 3)),
        };

        let Some((head_symbol, call_args)) = self.arguments_options_target_call(call) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArgumentsOptions",
                args,
                "expected a held call expression whose head is a symbol",
            ));
        };

        let Some(spec) = parse_argument_count_spec(spec) else {
            return Ok(self.invalid_call_with_arg_form(
                "ArgumentsOptions",
                args,
                "expected an exact argument count or a {min, max} count range",
            ));
        };

        if let Some(assoc_expr) = assoc {
            if association_entries_from_expr(assoc_expr).is_none() {
                return Ok(self.invalid_call_with_arg_form(
                    "ArgumentsOptions",
                    args,
                    "expected an association as the optional third argument",
                ));
            }
        }

        Ok(
            match self.partition_arguments_and_options(&head_symbol, call, &call_args, spec) {
                ArgumentPartitionOutcome::Success {
                    positional,
                    options,
                } => Expr::call(
                    system_expr("List"),
                    vec![
                        Expr::call(system_expr("List"), positional),
                        Expr::call(system_expr("List"), options),
                    ],
                ),
                ArgumentPartitionOutcome::Failure { tag, details, text } => {
                    self.push_message(head_symbol.name(), tag, text);
                    build_argument_partition_failure(tag, details)
                }
            },
        )
    }

    pub(in crate::runtime) fn eval_check_arguments(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (call, spec, assoc) = match args {
            [call_expr, spec_expr] => (call_expr, spec_expr, None),
            [call_expr, spec_expr, assoc_expr] => (call_expr, spec_expr, Some(assoc_expr)),
            _ => return Ok(self.invalid_call_with_arg_range("CheckArguments", args, 2, 3)),
        };

        let Some((head_symbol, call_args)) = self.arguments_options_target_call(call) else {
            return Ok(self.invalid_call_with_arg_form(
                "CheckArguments",
                args,
                "expected a held call expression whose head is a symbol",
            ));
        };

        let Some(spec) = parse_argument_count_spec(spec) else {
            return Ok(self.invalid_call_with_arg_form(
                "CheckArguments",
                args,
                "expected an exact argument count or a {min, max} count range",
            ));
        };

        if let Some(assoc_expr) = assoc {
            if association_entries_from_expr(assoc_expr).is_none() {
                return Ok(self.invalid_call_with_arg_form(
                    "CheckArguments",
                    args,
                    "expected an association as the optional third argument",
                ));
            }
        }

        Ok(
            match self.partition_arguments_and_options(&head_symbol, call, &call_args, spec) {
                ArgumentPartitionOutcome::Success { .. } => system_expr("True"),
                ArgumentPartitionOutcome::Failure { tag, text, .. } => {
                    self.push_message(head_symbol.name(), tag, text);
                    system_expr("False")
                }
            },
        )
    }

    fn arguments_options_target_call(&self, expr: &Expr) -> Option<(Symbol, Vec<Expr>)> {
        let Expr::Call { head, args, .. } = expr else {
            return None;
        };

        Some((expr_as_symbol(head)?.clone(), args.to_vec()))
    }

    fn partition_arguments_and_options(
        &self,
        head_symbol: &Symbol,
        held_call: &Expr,
        call_args: &[Expr],
        spec: ArgumentCountSpec,
    ) -> ArgumentPartitionOutcome {
        let known_options = self.options_for_symbol(head_symbol);

        if call_args.len() < spec.min() {
            return ArgumentPartitionOutcome::Failure {
                tag: spec.too_few_message_tag(),
                details: build_argument_count_failure_details(
                    head_symbol,
                    held_call.clone(),
                    call_args.len(),
                    spec,
                ),
                text: render_argument_count_failure_message(head_symbol, call_args.len(), spec),
            };
        }

        let max_count = spec.max().min(call_args.len());
        for positional_count in spec.min()..=max_count {
            let positional = call_args[..positional_count].to_vec();
            match classify_option_suffix(&call_args[positional_count..], &known_options) {
                OptionSuffixClassification::Valid(options) => {
                    return ArgumentPartitionOutcome::Success {
                        positional,
                        options,
                    };
                }
                OptionSuffixClassification::UnknownOption(option_name) => {
                    return ArgumentPartitionOutcome::Failure {
                        tag: "optx",
                        details: build_unknown_option_failure_details(
                            head_symbol,
                            held_call.clone(),
                            &option_name,
                        ),
                        text: render_unknown_option_failure_message(
                            head_symbol,
                            &option_name,
                            held_call,
                        ),
                    };
                }
                OptionSuffixClassification::NotOptionSyntax => {}
            }
        }

        let offending = call_args
            .last()
            .cloned()
            .unwrap_or_else(|| system_expr("Null"));
        let max_allowed = spec.max();

        ArgumentPartitionOutcome::Failure {
            tag: "nonopt",
            details: build_non_option_failure_details(
                head_symbol,
                held_call.clone(),
                offending.clone(),
                max_allowed,
            ),
            text: render_non_option_failure_message(
                head_symbol,
                &offending,
                max_allowed,
                held_call,
            ),
        }
    }
}
