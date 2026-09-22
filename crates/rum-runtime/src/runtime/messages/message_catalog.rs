use super::super::*;
use std::sync::Arc;
mod resume_frames;
use resume_frames::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum MessageQueryTarget {
    Symbol(Symbol),
    Key(MessageNameKey),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum MessageTargetNormalizationError {
    InvalidMessageEmitTarget,
    InvalidMessageCatalogTarget,
    InvalidDefinitionInspectionTarget,
}

#[derive(Clone, Copy)]
enum MessageCatalogSource<'a> {
    User(&'a MessageDefinition),
    Builtin(BuiltinMessageSpec),
}

impl MessageTargetNormalizationError {
    pub(in crate::runtime) fn message(self) -> &'static str {
        match self {
            Self::InvalidMessageEmitTarget => {
                "expected the first argument to be a MessageName expression"
            }
            Self::InvalidMessageCatalogTarget => {
                "expected no arguments, a symbol or symbol-name string, or a MessageName expression"
            }
            Self::InvalidDefinitionInspectionTarget => {
                "expected a symbol or symbol-name string argument"
            }
        }
    }
}

impl Session {
    pub(in crate::runtime) fn resolve_message_name_args(
        &self,
        symbol_expr: &Expr,
        tag_expr: &Expr,
    ) -> Option<MessageNameKey> {
        let symbol = self.normalize_symbol_target_expr(symbol_expr, true)?;

        let tag = match tag_expr {
            Expr::Atom(Atom::String(tag)) => tag.clone(),
            Expr::Atom(Atom::Symbol(tag)) => tag.name().to_string(),
            _ => return None,
        };

        Some(MessageNameKey { symbol, tag })
    }

    pub(in crate::runtime) fn message_name_key_from_expr(
        &self,
        expr: &Expr,
    ) -> Option<MessageNameKey> {
        let Expr::Call { head, args } = expr else {
            return None;
        };

        if !is_system_symbol_name(head, "MessageName") || args.len() != 2 {
            return None;
        }

        self.resolve_message_name_args(&args[0], &args[1])
    }

    pub(in crate::runtime) fn normalize_message_emit_target(
        &self,
        expr: &Expr,
    ) -> Result<MessageNameKey, MessageTargetNormalizationError> {
        self.message_name_key_from_expr(expr)
            .ok_or(MessageTargetNormalizationError::InvalidMessageEmitTarget)
    }

    pub(in crate::runtime) fn define_message(
        &mut self,
        key: MessageNameKey,
        value: Expr,
        delayed: bool,
    ) {
        self.note_active_modules_escaped();
        if !self.message_definitions.contains_key(&key) {
            *self.message_symbol_counts.entry(key.symbol).or_insert(0) += 1;
        }
        let definition = self.message_definitions.entry(key).or_default();

        if delayed {
            definition.immediate = None;
            definition.delayed = Some(value);
        } else {
            definition.immediate = Some(value);
            definition.delayed = None;
        }
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
        );
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::MessageSymbolCounts,
        );
    }

    pub(in crate::runtime) fn render_message_text(
        &mut self,
        key: &MessageNameKey,
        args: &[Expr],
    ) -> Result<String, EvalError> {
        self.render_message_text_with_named(key, args, &[])
    }

    pub(in crate::runtime) fn render_message_text_with_named(
        &mut self,
        key: &MessageNameKey,
        args: &[Expr],
        named_args: &[(String, Expr)],
    ) -> Result<String, EvalError> {
        let template = self.message_definitions.get(key).cloned();
        match template {
            Some(MessageDefinition {
                immediate: Some(expr),
                ..
            }) => Ok(self.finish_render_message_text(expr, key, args, named_args)),
            Some(MessageDefinition {
                delayed: Some(expr),
                ..
            }) => {
                let delayed_result = self.evaluate(expr);
                crate::resume_or_suspend!(
                    self,
                    delayed_result,
                    ok(rendered) => Ok(self.finish_render_message_text(
                        rendered,
                        key,
                        args,
                        named_args,
                    )),
                    suspend => RenderDelayedMessageTextFrame {
                        key: key.clone(),
                        args: Arc::from(args),
                        named_args: Arc::from(named_args),
                    },
                )
            }
            _ => {
                if args.is_empty() {
                    return Ok("message emitted".to_string());
                }

                return Ok(args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "));
            }
        }
    }

    fn finish_render_message_text(
        &self,
        rendered: Expr,
        key: &MessageNameKey,
        args: &[Expr],
        named_args: &[(String, Expr)],
    ) -> String {
        let template = match rendered {
            Expr::Atom(Atom::String(text)) => text,
            other => other.to_string(),
        };

        format_message_template_with_named(&template, key, args, named_args)
    }

    /// Normalizes a symbol or symbol-name string into a symbol target.
    pub fn normalize_symbol_target_expr(
        &self,
        expr: &Expr,
        create_if_missing: bool,
    ) -> Option<Symbol> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.clone()),
            Expr::Atom(Atom::String(raw)) => self.resolve_symbol_name(raw, create_if_missing),
            _ => None,
        }
    }

    pub(in crate::runtime) fn normalize_symbol_target_exprs(
        &self,
        expr: &Expr,
        create_if_missing: bool,
    ) -> Option<Vec<Symbol>> {
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => args
                .iter()
                .map(|item| self.normalize_symbol_target_exprs(item, create_if_missing))
                .collect::<Option<Vec<_>>>()
                .map(|items| items.into_iter().flatten().collect()),
            _ => Some(vec![
                self.normalize_symbol_target_expr(expr, create_if_missing)?,
            ]),
        }
    }

    pub(in crate::runtime) fn normalize_message_query_target(
        &self,
        expr: &Expr,
    ) -> Result<MessageQueryTarget, MessageTargetNormalizationError> {
        if let Some(key) = self.message_name_key_from_expr(expr) {
            Ok(MessageQueryTarget::Key(key))
        } else if let Some(symbol) = self.normalize_symbol_target_expr(expr, true) {
            Ok(MessageQueryTarget::Symbol(symbol))
        } else {
            Err(MessageTargetNormalizationError::InvalidMessageCatalogTarget)
        }
    }

    pub(in crate::runtime) fn normalize_definition_inspection_target(
        &self,
        expr: &Expr,
    ) -> Result<Symbol, MessageTargetNormalizationError> {
        self.normalize_symbol_target_expr(expr, true)
            .ok_or(MessageTargetNormalizationError::InvalidDefinitionInspectionTarget)
    }

    pub(in crate::runtime) fn resolve_message_catalog_query(
        &self,
        query: &str,
    ) -> Option<MessageCatalogQuery> {
        let query = query.trim();

        if query.is_empty() {
            return Some(MessageCatalogQuery::All);
        }

        if let Some((symbol_name, tag)) = query.rsplit_once("::") {
            let symbol_name = symbol_name.trim();
            let tag = tag.trim();

            if symbol_name.is_empty() || tag.is_empty() {
                return None;
            }

            return self.resolve_symbol_name(symbol_name, true).map(|symbol| {
                MessageCatalogQuery::Key(MessageNameKey {
                    symbol,
                    tag: tag.to_string(),
                })
            });
        }

        self.resolve_symbol_name(query, true)
            .map(MessageCatalogQuery::Symbol)
    }

    pub(in crate::runtime) fn message_names_expr_for_symbol(
        &self,
        symbol: Option<&Symbol>,
    ) -> Expr {
        let items = self
            .merged_message_catalog_sources(symbol)
            .into_keys()
            .map(|key| message_name_expr(key.symbol.clone(), key.tag.clone()))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(in crate::runtime) fn build_message_catalog_entry(
        &self,
        key: &MessageNameKey,
        def: &MessageDefinition,
    ) -> MessageCatalogEntry {
        let (template, delayed) = if let Some(rhs) = &def.immediate {
            (rhs.clone(), false)
        } else if let Some(rhs) = &def.delayed {
            (rhs.clone(), true)
        } else {
            (system_expr("Null"), false)
        };

        MessageCatalogEntry {
            message_name: message_name_expr(key.symbol.clone(), key.tag.clone()).to_string(),
            qualified_symbol_name: key.symbol.fully_qualified_name(),
            tag: key.tag.clone(),
            template,
            delayed,
        }
    }

    pub(in crate::runtime) fn message_definition_rule_expr(
        &self,
        key: &MessageNameKey,
        def: &MessageDefinition,
    ) -> Expr {
        let lhs = message_name_expr(key.symbol.clone(), key.tag.clone());

        if let Some(rhs) = &def.immediate {
            Expr::call(system_expr("Rule"), vec![lhs, rhs.clone()])
        } else if let Some(rhs) = &def.delayed {
            Expr::call(system_expr("RuleDelayed"), vec![lhs, rhs.clone()])
        } else {
            Expr::call(system_expr("Rule"), vec![lhs, system_expr("Null")])
        }
    }

    pub(in crate::runtime) fn message_definitions_expr_for_symbol(
        &self,
        symbol: Option<&Symbol>,
    ) -> Expr {
        let sealed_symbols_visible = self.sealed_symbols_visible();
        let items = self
            .message_definitions
            .iter()
            .filter(|(key, _)| {
                sealed_symbols_visible
                    || !crate::runtime::scoping::packages::symbol_is_sealed_private(&key.symbol)
            })
            .filter(|(key, _)| symbol.is_none_or(|symbol| key.symbol == *symbol))
            .map(|(key, def)| self.message_definition_rule_expr(key, def))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(in crate::runtime) fn message_catalog_entries_for_symbol(
        &self,
        symbol: Option<&Symbol>,
    ) -> Vec<MessageCatalogEntry> {
        self.merged_message_catalog_sources(symbol)
            .into_iter()
            .map(|(key, source)| self.build_merged_message_catalog_entry(&key, source))
            .collect()
    }

    pub(in crate::runtime) fn message_catalog_entries_for_key(
        &self,
        key: &MessageNameKey,
    ) -> Vec<MessageCatalogEntry> {
        let mut sources = self.merged_message_catalog_sources(Some(&key.symbol));
        sources
            .remove(key)
            .map(|source| vec![self.build_merged_message_catalog_entry(key, source)])
            .unwrap_or_default()
    }

    pub(in crate::runtime) fn message_tags_expr_for_symbol(&self, symbol: Option<&Symbol>) -> Expr {
        let tags = if symbol.is_some() {
            self.merged_message_catalog_sources(symbol)
                .into_keys()
                .map(|key| key.tag.clone())
                .collect::<Vec<_>>()
        } else {
            self.merged_message_catalog_sources(None)
                .into_keys()
                .map(|key| key.tag.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        };

        Expr::call(
            system_expr("List"),
            tags.into_iter().map(Expr::string).collect(),
        )
    }

    fn merged_message_catalog_sources(
        &self,
        symbol: Option<&Symbol>,
    ) -> BTreeMap<MessageNameKey, MessageCatalogSource<'_>> {
        let sealed_symbols_visible = self.sealed_symbols_visible();
        let mut sources = BTreeMap::new();

        match symbol {
            Some(symbol) => {
                if let Some(spec) = self.builtin_spec_for_symbol(symbol) {
                    for message in spec.messages {
                        sources.insert(
                            MessageNameKey {
                                symbol: symbol.clone(),
                                tag: message.tag.to_string(),
                            },
                            MessageCatalogSource::Builtin(*message),
                        );
                    }
                }
            }
            None => {
                for spec in self.registered_builtin_specs() {
                    let symbol = Symbol::system(spec.name);
                    for message in spec.messages {
                        sources.insert(
                            MessageNameKey {
                                symbol: symbol.clone(),
                                tag: message.tag.to_string(),
                            },
                            MessageCatalogSource::Builtin(*message),
                        );
                    }
                }
            }
        }

        for (key, definition) in &self.message_definitions {
            if (sealed_symbols_visible
                || !crate::runtime::scoping::packages::symbol_is_sealed_private(&key.symbol))
                && symbol.is_none_or(|symbol| key.symbol == *symbol)
            {
                sources.insert(key.clone(), MessageCatalogSource::User(definition));
            }
        }

        sources
    }

    fn build_merged_message_catalog_entry(
        &self,
        key: &MessageNameKey,
        source: MessageCatalogSource<'_>,
    ) -> MessageCatalogEntry {
        match source {
            MessageCatalogSource::User(definition) => {
                self.build_message_catalog_entry(key, definition)
            }
            MessageCatalogSource::Builtin(spec) => MessageCatalogEntry {
                message_name: message_name_expr(key.symbol.clone(), key.tag.clone()).to_string(),
                qualified_symbol_name: key.symbol.fully_qualified_name(),
                tag: key.tag.clone(),
                template: Expr::string(spec.template),
                delayed: false,
            },
        }
    }

    pub(in crate::runtime) fn message_definitions_expr_for_key(
        &self,
        key: &MessageNameKey,
    ) -> Expr {
        if !self.sealed_symbols_visible()
            && crate::runtime::scoping::packages::symbol_is_sealed_private(&key.symbol)
        {
            return Expr::call(system_expr("List"), Vec::new());
        }

        let items = self
            .message_definitions
            .get(key)
            .map(|def| vec![self.message_definition_rule_expr(key, def)])
            .unwrap_or_default();

        Expr::call(system_expr("List"), items)
    }

    pub(in crate::runtime) fn message_tags_expr_for_key(&self, key: &MessageNameKey) -> Expr {
        if !self.sealed_symbols_visible()
            && crate::runtime::scoping::packages::symbol_is_sealed_private(&key.symbol)
        {
            return Expr::call(system_expr("List"), Vec::new());
        }

        let items = self
            .message_definitions
            .contains_key(key)
            .then(|| vec![Expr::string(key.tag.clone())])
            .unwrap_or_default();

        Expr::call(system_expr("List"), items)
    }
}
