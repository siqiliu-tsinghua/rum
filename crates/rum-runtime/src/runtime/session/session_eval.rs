use super::super::*;
use rum_syntax::{ScriptForm, parse_script_form_with_source};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminatedEvaluationHandling {
    ConvertToValue,
    ConvertCyclesToValue,
}

/// A top-level evaluation input: source text to parse, or syntax a splitter
/// already parsed. Script forms carry their syntax
/// ([`ScriptForm::Parsed`]) so the script loop never parses a form twice;
/// every other entry point evaluates text and parses exactly as before.
enum TopLevelEvalInput<'a> {
    Text(&'a str),
    ScriptForm {
        text: &'a str,
        source: &'a str,
        source_offset: usize,
    },
    Parsed(SyntaxExpr),
}

impl Session {
    pub(in crate::runtime) fn finalize_top_level_control_flow(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Return { value }) => Ok(value),
            Err(EvalError::Abort | EvalError::ProtectedAbort) => Ok(system_expr("$Aborted")),
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime) fn finalize_top_level_eval_value(
        &mut self,
        outcome: Result<Expr, EvalError>,
        handled_error_checkpoint: usize,
    ) -> Result<(Expr, Option<EvalTerminationReason>), EvalError> {
        match self.finalize_top_level_control_flow(outcome) {
            Err(error @ EvalError::ExceptionThrown { .. }) => {
                let failure = uncaught_exception_failure(&error);
                self.emit_non_loader_eval_message(handled_error_checkpoint, &error);
                Ok((failure, None))
            }
            Err(error) => match Self::terminated_evaluation_reason_for_top_level_error(&error) {
                Some(reason) => {
                    let value = Expr::call(
                        system_expr("TerminatedEvaluation"),
                        vec![Expr::string(reason.tag())],
                    );
                    self.emit_non_loader_eval_message(handled_error_checkpoint, &error);
                    Ok((value, Some(reason)))
                }
                None => Err(error),
            },
            Ok(value) => Ok((value, None)),
        }
    }

    pub(in crate::runtime) fn finalize_cycle_termination_value(
        &mut self,
        outcome: Result<Expr, EvalError>,
        handled_error_checkpoint: usize,
    ) -> Result<(Expr, Option<EvalTerminationReason>), EvalError> {
        match self.finalize_top_level_control_flow(outcome) {
            Err(error @ EvalError::EvaluationCycleDetected { .. }) => {
                match Self::terminated_evaluation_reason_for_error(&error) {
                    Some(reason) => {
                        let value = Self::terminated_evaluation_expr_for_error(&error)
                            .expect("termination value should exist for termination reason");
                        self.emit_non_loader_eval_message(handled_error_checkpoint, &error);
                        Ok((value, Some(reason)))
                    }
                    None => Err(error),
                }
            }
            Ok(value) => Ok((value, None)),
            Err(error) => Err(error),
        }
    }

    pub fn inspect(&self, name: &str) -> Option<SymbolInfo> {
        let symbol = self
            .resolve_symbol_name(name, false)
            .or_else(|| self.resolve_symbol_name(name, true))?;
        let builtin = self.builtin_spec_for_symbol(&symbol);
        let definition = self.definitions.get(&symbol);
        let attributes = self.attribute_set_for_symbol(&symbol);
        let message_tags = self
            .message_definitions
            .keys()
            .filter(|key| key.symbol == symbol)
            .map(|key| key.tag.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let message_count = message_tags.len();
        let attribute_names = attributes
            .attributes()
            .into_iter()
            .map(|attribute| attribute.symbol_name().to_string())
            .collect::<Vec<_>>();

        if builtin.is_none()
            && definition.is_none()
            && attributes.is_empty()
            && message_count == 0
            && !is_known_system_symbol(&symbol)
        {
            return None;
        }

        Some(SymbolInfo {
            qualified_name: symbol.fully_qualified_name(),
            context: symbol.context().to_string(),
            display_name: symbol.to_string(),
            summary: builtin.map(|builtin| builtin.summary.to_string()),
            is_builtin: builtin.is_some() || is_known_system_symbol(&symbol),
            attribute_count: attributes.len(),
            attributes: attribute_names,
            has_own_value: definition.and_then(|def| def.own_value.as_ref()).is_some(),
            has_delayed_value: definition
                .and_then(|def| def.delayed_own_value.as_ref())
                .is_some(),
            has_up_values: definition
                .map(|definition| !definition.up_values.is_empty())
                .unwrap_or(false),
            has_down_values: definition
                .map(|definition| !definition.down_values.is_empty())
                .unwrap_or(false),
            has_sub_values: definition
                .map(|definition| !definition.sub_values.is_empty())
                .unwrap_or(false),
            message_count,
            message_tags,
        })
    }

    pub fn inspect_message_catalog(&self, query: Option<&str>) -> Option<Vec<MessageCatalogEntry>> {
        let query = query.map(str::trim).filter(|query| !query.is_empty());

        match query {
            None => Some(self.message_catalog_entries_for_symbol(None)),
            Some(query) => match self.resolve_message_catalog_query(query)? {
                MessageCatalogQuery::All => Some(self.message_catalog_entries_for_symbol(None)),
                MessageCatalogQuery::Symbol(symbol) => {
                    Some(self.message_catalog_entries_for_symbol(Some(&symbol)))
                }
                MessageCatalogQuery::Key(key) => Some(self.message_catalog_entries_for_key(&key)),
            },
        }
    }

    pub fn inspect_definition_catalog(&self, name: &str) -> Option<Vec<DefinitionCatalogEntry>> {
        let symbol = self
            .resolve_symbol_name(name, false)
            .or_else(|| self.resolve_symbol_name(name, true))?;
        let message_count = self
            .message_definitions
            .keys()
            .filter(|key| key.symbol == symbol)
            .count();

        if self.builtin_spec_for_symbol(&symbol).is_none()
            && !self.definitions.contains_key(&symbol)
            && message_count == 0
            && !is_known_system_symbol(&symbol)
        {
            return None;
        }

        Some(self.definition_catalog_entries_for_symbol(&symbol))
    }

    pub fn inspect_rewrite_rules(
        &mut self,
        expr: Expr,
    ) -> Result<Option<Vec<RewriteRuleInfo>>, EvalError> {
        self.resolve_rewrite_rule_infos_owned(expr)
    }

    pub fn resume_inspect_rewrite_rules_with_input(
        &mut self,
        input: impl Into<String>,
    ) -> Result<Option<Vec<RewriteRuleInfo>>, EvalError> {
        let resumed = self.resume_suspended_evaluation(input);
        if self.has_continuation_fault() {
            return Err(self.continuation_fault_outcome(resumed).unwrap_err());
        }
        let resumed = resumed?;
        Ok(Option::<Vec<RewriteRuleInfo>>::from_continuation_expr(
            resumed,
        ))
    }

    pub fn eval_text(&mut self, input: &str) -> Result<EvalOutcome, EvalError> {
        self.eval_text_with_terminated_handling(input, TerminatedEvaluationHandling::ConvertToValue)
    }

    fn eval_text_with_terminated_handling(
        &mut self,
        input: &str,
        terminated_handling: TerminatedEvaluationHandling,
    ) -> Result<EvalOutcome, EvalError> {
        self.eval_input_with_terminated_handling(
            TopLevelEvalInput::Text(input),
            terminated_handling,
        )
    }

    fn eval_input_with_terminated_handling(
        &mut self,
        input: TopLevelEvalInput<'_>,
        terminated_handling: TerminatedEvaluationHandling,
    ) -> Result<EvalOutcome, EvalError> {
        self.begin_top_level_interrupt_budget();
        let message_checkpoint = self.messages.len();
        let print_checkpoint = self.prints.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let previous_message_list_checkpoint = self
            .current_message_list_checkpoint
            .replace(message_checkpoint);
        let previous_message_list_dirty =
            std::mem::replace(&mut self.current_message_list_dirty, false);
        let result = (|| {
            let result = if self.active_accepted_request_is_cancelled() {
                Err(EvalError::Cancelled)
            } else {
                let syntax = match input {
                    TopLevelEvalInput::Text(text) => parse_expr(text)?,
                    TopLevelEvalInput::ScriptForm {
                        text,
                        source,
                        source_offset,
                    } => parse_script_form_with_source(text, source, source_offset)?,
                    TopLevelEvalInput::Parsed(syntax) => syntax,
                };
                let expr = self.lower_syntax_expr(&syntax)?;
                self.evaluate(expr)
            };
            if self.has_continuation_fault() {
                return Err(self.continuation_fault_outcome(result).unwrap_err());
            }
            match terminated_handling {
                TerminatedEvaluationHandling::ConvertToValue => {
                    self.finalize_top_level_eval_value(result, handled_error_checkpoint)
                }
                TerminatedEvaluationHandling::ConvertCyclesToValue => {
                    self.finalize_cycle_termination_value(result, handled_error_checkpoint)
                }
            }
        })();
        self.current_message_list_checkpoint = previous_message_list_checkpoint;
        self.current_message_list_dirty = previous_message_list_dirty;

        match result {
            Ok((value, termination)) => {
                let messages = self.split_accounted_messages(message_checkpoint);
                let prints = self.split_accounted_prints(print_checkpoint);
                self.last_message_list = messages.clone();
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
                );
                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
                );
                Ok(EvalOutcome {
                    value,
                    termination,
                    messages,
                    prints,
                })
            }
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);
                self.emit_top_level_control_flow_messages(&err);
                self.last_message_list = self.messages[message_checkpoint..].to_vec();
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
                );
                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
                );
                Err(err)
            }
        }
    }

    pub fn eval_script(&mut self, source: &str) -> Result<EvalOutcome, EvalError> {
        self.eval_script_with_terminated_handling(
            source,
            TerminatedEvaluationHandling::ConvertToValue,
        )
    }

    fn eval_script_for_loader(&mut self, source: &str) -> Result<EvalOutcome, EvalError> {
        self.eval_script_with_terminated_handling(
            source,
            TerminatedEvaluationHandling::ConvertCyclesToValue,
        )
    }

    fn eval_script_with_terminated_handling(
        &mut self,
        source: &str,
        terminated_handling: TerminatedEvaluationHandling,
    ) -> Result<EvalOutcome, EvalError> {
        let message_checkpoint = self.messages.len();
        let print_checkpoint = self.prints.len();
        let forms = split_script_into_forms(source)?;
        let mut value = system_expr("Null");
        let mut termination = None;
        let mut messages = Vec::new();
        let mut prints = Vec::new();

        for form in forms {
            // A parsed form lowers its carried syntax directly. A text-only
            // form (hard error or lex-failure fallback) reparses its exact raw
            // source range so every resulting span is a file coordinate.
            let result = match form {
                ScriptForm::Parsed { syntax, .. } => self.eval_input_with_terminated_handling(
                    TopLevelEvalInput::Parsed(syntax),
                    terminated_handling,
                ),
                ScriptForm::Unparsed { text, source_span } => {
                    let form_source = source
                        .get(source_span.start..source_span.end)
                        .expect("script form source span belongs to its source");
                    self.eval_input_with_terminated_handling(
                        TopLevelEvalInput::ScriptForm {
                            text: &text,
                            source: form_source,
                            source_offset: source_span.start,
                        },
                        terminated_handling,
                    )
                }
            };
            match result {
                Ok(outcome) => {
                    value = outcome.value;
                    termination = outcome.termination;
                    messages.extend(outcome.messages);
                    prints.extend(outcome.prints);
                    if self.active_accepted_request.is_some()
                        && (termination == Some(EvalTerminationReason::Cancelled)
                            || self.active_accepted_request_is_cancelled())
                    {
                        break;
                    }
                }
                Err(err) => {
                    if !messages.is_empty() {
                        self.messages
                            .splice(message_checkpoint..message_checkpoint, messages);
                        self.refresh_memory_owner(
                            crate::runtime::debug_tools::resources::MemoryOwner::Messages,
                        );
                        self.last_message_list = self.messages[message_checkpoint..].to_vec();
                        self.refresh_memory_owner(
                            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
                        );
                    }
                    if !prints.is_empty() {
                        self.prints
                            .splice(print_checkpoint..print_checkpoint, prints);
                        self.refresh_memory_owner(
                            crate::runtime::debug_tools::resources::MemoryOwner::Prints,
                        );
                    }

                    return Err(err);
                }
            }
        }

        self.last_message_list = messages.clone();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );
        Ok(EvalOutcome {
            value,
            termination,
            messages,
            prints,
        })
    }

    /// Evaluates a non-resumable file batch. On input suspension, its file
    /// transaction stays live until abandonment; read-only inspection sees the
    /// partial state. Cancellation rolls back inner loads before this file.
    pub fn eval_file(&mut self, path: impl AsRef<Path>) -> Result<EvalOutcome, EvalError> {
        let path = path.as_ref();
        let source = self
            .file_io()
            .ok_or(EvalError::NoFileIoProvider)?
            .read_to_string(path)
            .map_err(|source| EvalError::Io {
                path: path.display().to_string(),
                source,
            })?;
        let snapshot = self.capture_loader_snapshot();
        let depth = self.load_stack.len();

        self.load_stack.push(path.to_owned());
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::LoadStack);
        let result = self.eval_script_for_loader(&source);
        if matches!(result, Err(EvalError::InputRequired { .. })) {
            self.defer_public_file_rollback(path, depth, snapshot);
        } else {
            self.close_public_file(path, depth, result.is_err().then_some(snapshot));
        }

        result
    }

    pub(in crate::runtime) fn resolve_rewrite_rule_infos_owned(
        &mut self,
        expr: Expr,
    ) -> Result<Option<Vec<RewriteRuleInfo>>, EvalError> {
        self.drive_rewrite_rule_infos_owned(expr, RewriteRuleInfoContinuation::InspectRewriteRules)
    }
}

fn uncaught_exception_failure(error: &EvalError) -> Expr {
    let EvalError::ExceptionThrown { exception } = error else {
        unreachable!("uncaught exception failures require ExceptionThrown")
    };
    let Expr::Call { head, args } = exception else {
        unreachable!("ExceptionThrown must carry an Exception object")
    };
    assert!(is_system_symbol_name(head, "Exception") && args.len() == 2);

    Expr::call(
        system_expr("Failure"),
        vec![
            Expr::string("UncaughtException"),
            Expr::call(
                system_expr("Association"),
                vec![
                    Expr::call(
                        system_expr("Rule"),
                        vec![Expr::string("Type"), args[0].clone()],
                    ),
                    Expr::call(
                        system_expr("Rule"),
                        vec![Expr::string("Payload"), args[1].clone()],
                    ),
                ],
            ),
        ],
    )
}
