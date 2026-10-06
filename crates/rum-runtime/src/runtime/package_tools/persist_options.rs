use super::*;
use crate::runtime::kernel_api::{FrozenOptionReadContinuation, OptionReadContinuation};
use std::sync::Arc;

pub(super) struct PersistOptionWorking<'a> {
    pub(super) args: &'a [Expr],
    pub(super) operator: &'static str,
    pub(super) overwrite: bool,
    pub(super) path_spec: String,
    pub(super) explicit: Option<&'a SymbolOptions>,
    pub(super) context_filters: PersistContextFilters,
}

#[derive(FrameRootTrace)]
pub(super) struct PersistOptionRead {
    args: SharedArgs,
    operator: &'static str,
    overwrite: bool,
    path_spec: String,
    explicit: Option<Arc<SymbolOptions>>,
    context_filters: PersistContextFilters,
}

impl OptionReadContinuation<Option<Expr>> for PersistOptionWorking<'_> {
    type Frozen = PersistOptionRead;
    fn finish(self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        if let Some(value) = value {
            let supported = matches!(
                &value,
                Expr::Atom(Atom::Integer(level)) if level.as_i64() == Some(1)
            );
            if !supported {
                session.push_message(
                        "DumpSave",
                        "optval",
                        format!(
                            "Unsupported value {value} for option CompressionLevel in DumpSave; only 1 is currently supported."
                        ),
                    );
                return Ok(Expr::call(system_expr("DumpSave"), self.args.to_vec()));
            }
        }
        self.persist(session)
    }
    fn freeze(self) -> Self::Frozen {
        PersistOptionRead {
            args: SharedArgs::new(self.args.to_vec()),
            operator: self.operator,
            overwrite: self.overwrite,
            path_spec: self.path_spec,
            explicit: self.explicit.map(|options| Arc::new(options.clone())),
            context_filters: self.context_filters,
        }
    }
}

impl FrozenOptionReadContinuation<Option<Expr>> for PersistOptionRead {
    fn finish(&self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        PersistOptionWorking {
            args: &self.args,
            operator: self.operator,
            overwrite: self.overwrite,
            path_spec: self.path_spec.clone(),
            explicit: self.explicit.as_deref(),
            context_filters: self.context_filters.clone(),
        }
        .finish(session, value)
    }
}

impl PersistOptionWorking<'_> {
    pub(super) fn persist(self, session: &mut Session) -> Result<Expr, EvalError> {
        let path = session.resolve_output_path(&self.path_spec);
        if !session.has_file_io() {
            session.emit_no_file_io_message();
            return Ok(system_expr("$Failed"));
        }

        let symbols = match session.compile_save_symbols_from_expr(&self.args[1]) {
            Ok(symbols) => symbols,
            Err(error) => {
                return Ok(session.invalid_call_with_arg_form(
                    self.operator,
                    self.args,
                    error.message(),
                ));
            }
        };
        let mut symbols =
            session.filter_persist_symbols_by_contexts(symbols, &self.context_filters);
        if !session.sealed_symbols_visible() {
            symbols.retain(|symbol| {
                !crate::runtime::scoping::packages::symbol_is_sealed_private(symbol)
            });
        }

        let include_attributes = if self.overwrite {
            session.dump_save_include_attributes(self.explicit)
        } else {
            true
        };
        let sealed_reference = symbols.iter().find_map(|symbol| {
            session
                .saved_symbol_statements(symbol, include_attributes)
                .iter()
                .find_map(|statement| {
                    crate::runtime::scoping::packages::sealed_symbols_in_expr(statement)
                        .into_iter()
                        .next()
                })
        });
        if let Some(sealed) = sealed_reference {
            session.push_message(
                self.operator,
                "sealedref",
                format!(
                    "Cannot save a definition containing sealed reference {}.",
                    sealed.fully_qualified_name()
                ),
            );
            return Ok(system_expr("$Failed"));
        }

        let rendered = session.render_saved_symbols(&symbols, self.overwrite, include_attributes);
        if let Err(source) = session.write_saved_content(&path, &rendered, self.overwrite) {
            session.push_message(
                self.operator,
                "io",
                format!("could not write `{}`: {source}", path.display()),
            );
            return Ok(system_expr("$Failed"));
        }

        if self.overwrite {
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
}
