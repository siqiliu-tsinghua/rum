mod resume_frames;
use resume_frames::*;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeclarePackageSpec {
    context: String,
    names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PersistContextSelection {
    All,
    None,
    Automatic,
    Explicit(BTreeSet<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PersistContextFilters {
    included: PersistContextSelection,
    excluded: PersistContextSelection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackageToolNormalizationError {
    InvalidContext,
    InvalidNames,
    InvalidPath,
    InvalidObjects,
}

impl PackageToolNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidContext => "expected a string context name",
            Self::InvalidNames => "expected a list of string symbol names",
            Self::InvalidPath => "expected a string or File[\"...\"] path",
            Self::InvalidObjects => {
                "expected a symbol, a string names pattern or context, or a list of them"
            }
        }
    }
}

impl Session {
    pub(super) fn eval_declare_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let spec = match self.compile_declare_package_spec(args) {
            Ok(spec) => spec,
            Err(PackageToolNormalizationError::InvalidContext) => {
                return Ok(self.invalid_call_with_arg_form(
                    "DeclarePackage",
                    args,
                    PackageToolNormalizationError::InvalidContext.message(),
                ));
            }
            Err(PackageToolNormalizationError::InvalidNames) => {
                return Ok(self.invalid_call_with_arg_form(
                    "DeclarePackage",
                    args,
                    PackageToolNormalizationError::InvalidNames.message(),
                ));
            }
            Err(_) => return Ok(self.invalid_call_with_exact_arg_count("DeclarePackage", args, 2)),
        };

        let mut inserted_stub = false;
        for name in spec.names {
            inserted_stub |= self
                .declared_stubs
                .insert(Symbol::new(spec.context.clone(), name));
        }

        self.context_path = prepend_context(&spec.context, &self.context_path);
        if inserted_stub {
            self.bump_evaluation_epoch();
        }
        Ok(Expr::string(spec.context))
    }

    pub(super) fn eval_save(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_persist_package_state("Save", args, false)
    }

    pub(super) fn eval_dump_save(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_persist_package_state("DumpSave", args, true)
    }

    pub(super) fn eval_once_builtin(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (expr, location, options) = match args {
            [expr] => (expr.clone(), None, None),
            [expr, second] => {
                if let Some(options) = self.try_compile_package_tool_options(second) {
                    (expr.clone(), None, Some(options))
                } else {
                    (expr.clone(), Some(second.clone()), None)
                }
            }
            [expr, location, options_expr] => {
                let options = match self.compile_assigned_options_from_resolved(options_expr) {
                    Ok(options) => options,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form("Once", args, error.message()));
                    }
                };
                (expr.clone(), Some(location.clone()), Some(options))
            }
            _ => return Ok(self.invalid_call_with_arg_range("Once", args, 1, 3)),
        };

        self.report_unknown_package_tool_options("Once", &options);
        let should_cache = match self.once_should_cache(options.as_ref()) {
            Ok(should_cache) => should_cache,
            Err((option, value)) => {
                self.push_message(
                    "Once",
                    "pers",
                    format!(
                        "Unsupported value {value} for option {option} in Once; supported values are ExpirationDate -> None and PersistenceTime -> None or 0."
                    ),
                );
                return Ok(Expr::call(system_expr("Once"), args.to_vec()));
            }
        };
        if !should_cache {
            return self.evaluate(expr);
        }

        let cache_key = self.once_cache_key(&expr, location.as_ref());

        if let Some(value) = self.once_values.get(&cache_key).cloned() {
            return Ok(value);
        }

        let result = self.evaluate(expr);
        crate::resume_or_suspend!(
            self,
            result,
            ok(value_expr) => { let session = self; let value = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(value_expr); (|| {
                    session.once_values.insert(cache_key, value.clone());
                    Ok(value)
                })() },
            suspend => EvalOnceBuiltinSuspensionFrame {
                cache_key: std::sync::Arc::new(cache_key),
            },
        )
    }

    fn compile_declare_package_spec(
        &self,
        args: &[Expr],
    ) -> Result<DeclarePackageSpec, PackageToolNormalizationError> {
        match args {
            [Expr::Atom(Atom::String(context)), names_expr] => Ok(DeclarePackageSpec {
                context: expand_context_spec(context, &self.current_context),
                names: expr_as_string_list(names_expr)
                    .ok_or(PackageToolNormalizationError::InvalidNames)?,
            }),
            [_, _] => Err(PackageToolNormalizationError::InvalidContext),
            _ => Err(PackageToolNormalizationError::InvalidNames),
        }
    }

    fn eval_persist_package_state(
        &mut self,
        operator: &str,
        args: &[Expr],
        overwrite: bool,
    ) -> Result<Expr, EvalError> {
        let (path_expr, objects_expr, explicit_options) = match args {
            [path_expr, objects_expr] => (path_expr, objects_expr, None),
            [path_expr, objects_expr, options_expr] => {
                let options = match self.compile_assigned_options_from_resolved(options_expr) {
                    Ok(options) => options,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            operator,
                            args,
                            error.message(),
                        ));
                    }
                };
                (path_expr, objects_expr, Some(options))
            }
            _ => return Ok(self.invalid_call_with_arg_range(operator, args, 2, 3)),
        };

        let Some(path_spec) = load_spec_from_expr(path_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                operator,
                args,
                PackageToolNormalizationError::InvalidPath.message(),
            ));
        };

        let context_filters = match self.compile_persist_context_filters(explicit_options.as_ref())
        {
            Ok(filters) => filters,
            Err(message) => return Ok(self.invalid_call_with_arg_form(operator, args, message)),
        };
        if let Some(invalid) =
            self.first_unknown_package_tool_option(operator, explicit_options.as_ref())
        {
            self.push_message(
                operator,
                "optx",
                format!(
                    "Unknown option {invalid} in {}.",
                    Expr::call(system_expr(operator), args.to_vec())
                ),
            );
            return Ok(self.invalid_call(operator, args));
        }
        if overwrite {
            let compression_level = self.runtime_option_expr_for_symbol(
                explicit_options.as_ref(),
                &Symbol::system("DumpSave"),
                "CompressionLevel",
            )?;
            if let Some(value) = compression_level {
                let supported = matches!(
                    &value,
                    Expr::Atom(Atom::Integer(level)) if level.as_i64() == Some(1)
                );
                if !supported {
                    self.push_message(
                        "DumpSave",
                        "optval",
                        format!(
                            "Unsupported value {value} for option CompressionLevel in DumpSave; only 1 is currently supported."
                        ),
                    );
                    return Ok(Expr::call(system_expr("DumpSave"), args.to_vec()));
                }
            }
        }

        let path = self.resolve_output_path(&path_spec);
        if !self.has_file_io() {
            self.emit_no_file_io_message();
            return Ok(system_expr("$Failed"));
        }

        let symbols = match self.compile_save_symbols_from_expr(objects_expr) {
            Ok(symbols) => symbols,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(operator, args, error.message()));
            }
        };
        let mut symbols = self.filter_persist_symbols_by_contexts(symbols, &context_filters);
        if !self.sealed_symbols_visible() {
            symbols.retain(|symbol| {
                !crate::runtime::scoping::packages::symbol_is_sealed_private(symbol)
            });
        }

        let include_attributes = if overwrite {
            self.dump_save_include_attributes(explicit_options.as_ref())
        } else {
            true
        };
        let sealed_reference = symbols.iter().find_map(|symbol| {
            self.saved_symbol_statements(symbol, include_attributes)
                .iter()
                .find_map(|statement| {
                    crate::runtime::scoping::packages::sealed_symbols_in_expr(statement)
                        .into_iter()
                        .next()
                })
        });
        if let Some(sealed) = sealed_reference {
            self.push_message(
                operator,
                "sealedref",
                format!(
                    "Cannot save a definition containing sealed reference {}.",
                    sealed.fully_qualified_name()
                ),
            );
            return Ok(system_expr("$Failed"));
        }

        let rendered = self.render_saved_symbols(&symbols, overwrite, include_attributes);
        if let Err(source) = self.write_saved_content(&path, &rendered, overwrite) {
            self.push_message(
                operator,
                "io",
                format!("could not write `{}`: {source}", path.display()),
            );
            return Ok(system_expr("$Failed"));
        }

        if overwrite {
            Ok(Expr::call(
                system_expr("List"),
                symbols
                    .into_iter()
                    .map(|symbol| Expr::string(symbol.fully_qualified_name()))
                    .collect(),
            ))
        } else {
            Ok(system_expr("Null"))
        }
    }

    fn compile_save_symbols_from_expr(
        &self,
        expr: &Expr,
    ) -> Result<Vec<Symbol>, PackageToolNormalizationError> {
        let mut collected = Vec::new();
        self.collect_save_symbols(expr, &mut collected)?;
        Ok(collected)
    }

    fn collect_save_symbols(
        &self,
        expr: &Expr,
        collected: &mut Vec<Symbol>,
    ) -> Result<(), PackageToolNormalizationError> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => {
                if !collected.iter().any(|existing| existing == symbol) {
                    collected.push(symbol.clone());
                }
                Ok(())
            }
            Expr::Atom(Atom::String(context)) if context.ends_with('`') => {
                let context = expand_context_spec(context, &self.current_context);
                for symbol in self.symbols_in_context(&context) {
                    if !collected.iter().any(|existing| existing == &symbol) {
                        collected.push(symbol);
                    }
                }
                Ok(())
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
                for item in args {
                    self.collect_save_symbols(item, collected)?;
                }
                Ok(())
            }
            _ => {
                let Some(symbols) = self.resolve_names_matching_expr(expr) else {
                    return Err(PackageToolNormalizationError::InvalidObjects);
                };

                for symbol in symbols {
                    if !collected.iter().any(|existing| existing == &symbol) {
                        collected.push(symbol);
                    }
                }

                Ok(())
            }
        }
    }

    fn symbols_in_context(&self, context: &str) -> BTreeSet<Symbol> {
        let mut symbols = BTreeSet::new();

        symbols.extend(
            self.definitions
                .keys()
                .filter(|symbol| symbol.context() == context)
                .cloned(),
        );
        symbols.extend(
            self.attributes
                .keys()
                .filter(|symbol| symbol.context() == context)
                .cloned(),
        );
        symbols.extend(
            self.options
                .keys()
                .filter(|symbol| symbol.context() == context)
                .cloned(),
        );
        symbols.extend(
            self.syntax_information
                .keys()
                .filter(|symbol| symbol.context() == context)
                .cloned(),
        );
        symbols.extend(
            self.message_definitions
                .keys()
                .filter(|key| key.symbol.context() == context)
                .map(|key| key.symbol.clone()),
        );

        symbols.retain(|symbol| !is_module_gensym_name(symbol.name()));
        symbols
    }

    fn resolve_output_path(&self, spec: &str) -> PathBuf {
        let path = PathBuf::from(spec);

        if path.is_absolute() {
            return path;
        }

        self.load_stack
            .last()
            .and_then(|current| current.parent().map(|parent| parent.join(&path)))
            .unwrap_or(path)
    }

    pub(super) fn render_saved_symbols(
        &self,
        symbols: &[Symbol],
        textual_dump: bool,
        include_attributes: bool,
    ) -> String {
        let mut blocks = Vec::new();
        let symbols = symbols
            .iter()
            .copied()
            .filter(|symbol| {
                !self.is_installed_protected_builtin_own_value_symbol(symbol)
                    && !self.is_installed_system_down_values_symbol(symbol)
            })
            .collect::<Vec<_>>();

        if textual_dump {
            if let Some(block) = self.render_dump_save_package_restore(&symbols) {
                blocks.push(block);
            }
        }

        for symbol in &symbols {
            let symbol_block = self.render_saved_symbol(symbol, include_attributes);
            if !symbol_block.is_empty() {
                blocks.push(symbol_block);
            }
        }

        if blocks.is_empty() {
            String::new()
        } else {
            format!("{}\n", blocks.join("\n\n"))
        }
    }

    pub(super) fn render_dump_save_package_restore(&self, symbols: &[Symbol]) -> Option<String> {
        let contexts = self.dump_save_package_restore_contexts(symbols);
        if contexts.is_empty() {
            return None;
        }

        let context_list = Expr::call(
            system_expr("List"),
            contexts.into_iter().map(Expr::string).collect(),
        );
        let context_path_restore = Expr::call(
            system_expr("Set"),
            vec![
                system_expr("$ContextPath"),
                Expr::call(
                    system_expr("Join"),
                    vec![context_list.clone(), system_expr("$ContextPath")],
                ),
            ],
        );
        let packages_restore = Expr::call(
            system_expr("Set"),
            vec![
                system_expr("$Packages"),
                Expr::call(
                    system_expr("Union"),
                    vec![system_expr("$Packages"), context_list],
                ),
            ],
        );

        Some(format!("{context_path_restore}\n{packages_restore}"))
    }

    fn dump_save_package_restore_contexts(&self, symbols: &[Symbol]) -> Vec<String> {
        let saved_contexts: BTreeSet<&str> = symbols.iter().map(Symbol::context).collect();
        let mut contexts = Vec::new();

        for context in &self.loaded_packages {
            if is_private_context(context) {
                continue;
            }

            if !saved_contexts
                .iter()
                .any(|saved| *saved == context.as_str() || saved.starts_with(context.as_str()))
            {
                continue;
            }

            if !contexts.iter().any(|existing| existing == context) {
                contexts.push(context.clone());
            }
        }

        // Emit in a deterministic (sorted) order so the auto-generated package
        // restore preamble is byte-identical across a DumpSave/Get round-trip.
        // A cold dump enumerates `loaded_packages` in load order, but a restored
        // session reconstructs them sorted (the preamble's own `Union`), so an
        // unsorted preamble makes the first restore+redump differ by exactly
        // these two lines. Sorting both sides makes the snapshot reproducible.
        contexts.sort();
        contexts
    }

    fn render_saved_symbol(&self, symbol: &Symbol, include_attributes: bool) -> String {
        self.saved_symbol_statements(symbol, include_attributes)
            .into_iter()
            .map(|form| form.to_string())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub(super) fn saved_symbol_statements(
        &self,
        symbol: &Symbol,
        include_attributes: bool,
    ) -> Vec<Expr> {
        let symbol_expr = Expr::Atom(Atom::Symbol(symbol.clone()));
        let mut forms = Vec::new();

        if include_attributes && self.attributes.contains_key(symbol) {
            let attributes = self.attribute_set_for_symbol(symbol);
            if !attributes.is_empty() {
                forms.push(Expr::call(
                    system_expr("SetAttributes"),
                    vec![symbol_expr.clone(), attributes.expr()],
                ));
            }
        }

        for entry in self.definition_catalog_entries_for_symbol(symbol) {
            forms.push(self.save_definition_expr(&entry));
        }

        if self.options.contains_key(symbol) {
            forms.push(Expr::call(
                system_expr("Set"),
                vec![
                    Expr::call(system_expr("Options"), vec![symbol_expr.clone()]),
                    self.options_for_symbol(symbol).expr(),
                ],
            ));
        }

        if self.syntax_information.contains_key(symbol) {
            forms.push(Expr::call(
                system_expr("Set"),
                vec![
                    Expr::call(system_expr("SyntaxInformation"), vec![symbol_expr.clone()]),
                    self.syntax_information_for_symbol(symbol),
                ],
            ));
        }

        for (key, definition) in self
            .message_definitions
            .iter()
            .filter(|(key, _)| key.symbol == *symbol)
        {
            if let Some(expr) = self.save_message_definition_expr(key, definition) {
                forms.push(expr);
            }
        }

        forms
    }

    fn save_definition_expr(&self, entry: &DefinitionCatalogEntry) -> Expr {
        let head = match (entry.kind, entry.delayed) {
            (DefinitionKind::UpValue, true) => "UpSetDelayed",
            (DefinitionKind::UpValue, false) => "UpSet",
            (_, true) => "SetDelayed",
            (_, false) => "Set",
        };

        Expr::call(
            system_expr(head),
            vec![entry.lhs.clone(), entry.rhs.clone()],
        )
    }

    fn save_message_definition_expr(
        &self,
        key: &MessageNameKey,
        definition: &MessageDefinition,
    ) -> Option<Expr> {
        let (head, rhs) = if let Some(expr) = &definition.immediate {
            ("Set", expr.clone())
        } else if let Some(expr) = &definition.delayed {
            ("SetDelayed", expr.clone())
        } else {
            return None;
        };

        Some(Expr::call(
            system_expr(head),
            vec![message_name_expr(key.symbol.clone(), key.tag.clone()), rhs],
        ))
    }

    fn write_saved_content(
        &self,
        path: &Path,
        content: &str,
        overwrite: bool,
    ) -> Result<(), FileIoError> {
        let mode = if overwrite {
            FileWriteMode::Replace
        } else {
            FileWriteMode::Append
        };

        self.file_io()
            .ok_or_else(|| FileIoError::new("no file provider installed"))?
            .write_string(path, content, mode)
    }

    fn once_cache_key(&self, expr: &Expr, location: Option<&Expr>) -> String {
        let location = location
            .map(ToString::to_string)
            .unwrap_or_else(|| "KernelSession".to_string());
        format!("{location}\u{1f}{}", expr)
    }

    fn try_compile_package_tool_options(&self, expr: &Expr) -> Option<SymbolOptions> {
        self.compile_assigned_options_from_resolved(expr).ok()
    }

    fn first_unknown_package_tool_option(
        &self,
        operator: &str,
        explicit: Option<&SymbolOptions>,
    ) -> Option<Symbol> {
        let explicit = explicit?;
        let defaults = default_system_options(&Symbol::system(operator));
        explicit
            .iter()
            .find(|(name, _, _)| {
                !defaults
                    .iter()
                    .any(|(default_name, _, _)| default_name.name() == name.name())
            })
            .map(|(name, _, _)| name.clone())
    }

    fn report_unknown_package_tool_options(
        &mut self,
        operator: &str,
        explicit: &Option<SymbolOptions>,
    ) {
        let Some(explicit) = explicit.as_ref() else {
            return;
        };

        let defaults = default_system_options(&Symbol::system(operator));
        for (name, _, _) in explicit.iter() {
            if !defaults
                .iter()
                .any(|(default_name, _, _)| default_name.name() == name.name())
            {
                self.push_message(
                    "OptionValue",
                    "nodef",
                    format!("Unknown option {name} for {operator}."),
                );
            }
        }
    }

    fn once_should_cache(
        &self,
        explicit: Option<&SymbolOptions>,
    ) -> Result<bool, (&'static str, Expr)> {
        let once = Symbol::system("Once");

        if let Some((value, _)) =
            self.raw_option_value_for_symbol(explicit, &once, "ExpirationDate")
        {
            if !is_system_symbol_name(&value, "None") {
                return Err(("ExpirationDate", value));
            }
        }

        match self.raw_option_value_for_symbol(explicit, &once, "PersistenceTime") {
            None => Ok(true),
            Some((value, _)) if is_system_symbol_name(&value, "None") => Ok(true),
            Some((Expr::Atom(Atom::Integer(value)), _)) if value.as_i64() == Some(0) => Ok(false),
            Some((value, _)) => Err(("PersistenceTime", value)),
        }
    }

    fn dump_save_include_attributes(&self, explicit: Option<&SymbolOptions>) -> bool {
        let default = default_system_options(&Symbol::system("DumpSave"));

        explicit
            .and_then(|options| options.get_by_name("SymbolAttributes"))
            .or_else(|| default.get(&Symbol::system("SymbolAttributes")))
            .map(|value| is_system_symbol_name(value, "True"))
            .unwrap_or(true)
    }

    fn compile_persist_context_filters(
        &self,
        explicit: Option<&SymbolOptions>,
    ) -> Result<PersistContextFilters, &'static str> {
        let default = default_system_options(&Symbol::system("Save"));
        let included = explicit
            .and_then(|options| options.get_by_name("IncludedContexts"))
            .or_else(|| default.get(&Symbol::system("IncludedContexts")))
            .and_then(|value| self.parse_persist_context_selection(value))
            .ok_or(
                "expected IncludedContexts to be All, Automatic, None, a context-name string, or a list of context-name strings",
            )?;
        let excluded = explicit
            .and_then(|options| options.get_by_name("ExcludedContexts"))
            .or_else(|| default.get(&Symbol::system("ExcludedContexts")))
            .and_then(|value| self.parse_persist_context_selection(value))
            .ok_or(
                "expected ExcludedContexts to be All, Automatic, None, a context-name string, or a list of context-name strings",
            )?;

        Ok(PersistContextFilters { included, excluded })
    }

    fn parse_persist_context_selection(&self, value: &Expr) -> Option<PersistContextSelection> {
        if is_system_symbol_name(value, "All") {
            return Some(PersistContextSelection::All);
        }
        if is_system_symbol_name(value, "None") {
            return Some(PersistContextSelection::None);
        }
        if is_system_symbol_name(value, "Automatic") {
            return Some(PersistContextSelection::Automatic);
        }
        if let Expr::Atom(Atom::String(context)) = value {
            return Some(PersistContextSelection::Explicit(BTreeSet::from([
                expand_context_spec(context, &self.current_context),
            ])));
        }

        expr_as_string_list(value).map(|contexts| {
            PersistContextSelection::Explicit(
                contexts
                    .into_iter()
                    .map(|context| expand_context_spec(&context, &self.current_context))
                    .collect(),
            )
        })
    }

    fn filter_persist_symbols_by_contexts(
        &self,
        symbols: Vec<Symbol>,
        filters: &PersistContextFilters,
    ) -> Vec<Symbol> {
        symbols
            .into_iter()
            .filter(|symbol| self.persist_context_is_included(symbol.context(), filters))
            .filter(|symbol| {
                !self.is_installed_protected_builtin_own_value_symbol(symbol)
                    && !self.is_installed_system_down_values_symbol(symbol)
            })
            .collect()
    }

    fn persist_context_is_included(&self, context: &str, filters: &PersistContextFilters) -> bool {
        self.persist_context_matches_selection(context, &filters.included, true)
            && !self.persist_context_matches_selection(context, &filters.excluded, false)
    }

    fn persist_context_matches_selection(
        &self,
        context: &str,
        selection: &PersistContextSelection,
        include_mode: bool,
    ) -> bool {
        match selection {
            PersistContextSelection::All => true,
            PersistContextSelection::None => false,
            PersistContextSelection::Automatic => {
                if include_mode {
                    self.context_path.iter().any(|entry| entry == context)
                        || self.current_context == context
                } else {
                    context == SYSTEM_CONTEXT
                }
            }
            PersistContextSelection::Explicit(contexts) => contexts.contains(context),
        }
    }
}

fn is_private_context(context: &str) -> bool {
    context.ends_with("`Private`")
}

fn is_module_gensym_name(name: &str) -> bool {
    name.as_bytes()
        .windows(2)
        .enumerate()
        .any(|(index, pair)| index > 0 && pair[0] == b'$' && pair[1].is_ascii_digit())
}
