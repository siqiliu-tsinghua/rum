use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;
mod resume;
use resume::{BeginPackageDependencyFrame, NeedsCompleteFrame};
pub(in crate::runtime) use resume::{BeginPackageRollback, DeclaredStubLoadFrame};

mod file_names;
mod file_names_glob;

use self::file_names_glob::{collect_file_name_matches, parse_file_names_args};

pub(in crate::runtime) fn builtin_file_names(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_file_names(args)
}

#[derive(Clone, Debug, PartialEq, Eq, rum_derive::FrameRootTrace)]
struct NeedsLoadConfig {
    context: String,
    load_spec: String,
}

#[derive(Clone, Debug, PartialEq, Eq, rum_derive::FrameRootTrace)]
struct BeginPackageConfig {
    target_context: String,
    dependencies: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoaderSpecNormalizationError {
    InvalidLoadTarget,
    InvalidNeedsContext,
    InvalidNeedsLoadTarget,
    InvalidBeginPackageContext,
    InvalidBeginPackageDependencies,
    WrongArgCount {
        expected: &'static str,
        actual: usize,
    },
}

impl LoaderSpecNormalizationError {
    fn message(self) -> Option<&'static str> {
        match self {
            Self::InvalidLoadTarget => Some("expected a string or File[\"...\"] load target"),
            Self::InvalidNeedsContext => Some(
                "expected a string context name, optionally followed by a string or File[\"...\"] load target",
            ),
            Self::InvalidNeedsLoadTarget => {
                Some("expected a string or File[\"...\"] as the explicit load target")
            }
            Self::InvalidBeginPackageContext => Some(
                "expected a string context name, optionally followed by a list of string dependencies",
            ),
            Self::InvalidBeginPackageDependencies => {
                Some("expected the dependency list to be a list of strings")
            }
            Self::WrongArgCount { .. } => None,
        }
    }
}

impl Session {
    fn ensure_capability_allowed(
        &mut self,
        operator: &str,
        capability: Capability,
    ) -> Result<(), Expr> {
        if self.capability_policy.is_unrestricted() || self.capability_policy.allows(capability) {
            return Ok(());
        }

        self.push_message(
            "General",
            "sandbox",
            format!("{operator} is not permitted in this sandbox."),
        );
        Err(system_expr("$Failed"))
    }

    pub(super) fn eval_context(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(Expr::string(self.current_context.clone())),
            [Expr::Atom(Atom::Symbol(symbol))] => Ok(Expr::string(symbol.context().to_string())),
            [Expr::Atom(Atom::String(raw_name))] => self
                .resolve_symbol_name(raw_name, false)
                .map(|symbol| Expr::string(symbol.context().to_string()))
                .map_or_else(|| Ok(Expr::call(system_expr("Context"), args.to_vec())), Ok),
            [_] => Ok(self.invalid_call_with_arg_form(
                "Context",
                args,
                "expected a symbol or string argument",
            )),
            _ => Ok(self.invalid_call_with_message(
                "Context",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }

    pub(super) fn eval_contexts(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let pattern = match args {
            [] => None,
            [Expr::Atom(Atom::String(pattern))] => Some(pattern.as_str()),
            [_] => {
                return Ok(self.invalid_call_with_arg_form(
                    "Contexts",
                    args,
                    "expected a string context-name pattern",
                ));
            }
            _ => {
                return Ok(self.invalid_call_with_message(
                    "Contexts",
                    args,
                    "argx",
                    format!("expected 0 or 1 argument(s), got {}", args.len()),
                ));
            }
        };

        let mut contexts = BTreeSet::new();
        contexts.insert(SYSTEM_CONTEXT.to_string());
        contexts.insert(GLOBAL_CONTEXT.to_string());
        contexts.insert(normalize_context_name(&self.current_context));
        contexts.extend(
            self.context_path
                .iter()
                .map(|context| normalize_context_name(context)),
        );
        contexts.extend(
            self.loaded_packages
                .iter()
                .map(|context| normalize_context_name(context)),
        );

        let sealed_symbols_visible = self.sealed_symbols_visible();
        for symbol in self
            .definitions
            .keys()
            .chain(self.attributes.keys())
            .chain(self.options.keys())
            .chain(self.syntax_information.keys())
            .chain(self.declared_stubs.iter())
            .chain(self.removed_symbols.iter())
        {
            if !sealed_symbols_visible
                && crate::runtime::scoping::packages::symbol_is_sealed_private(symbol)
            {
                continue;
            }
            contexts.insert(symbol.context().to_string());
        }
        for key in self.message_definitions.keys() {
            if !sealed_symbols_visible
                && crate::runtime::scoping::packages::symbol_is_sealed_private(&key.symbol)
            {
                continue;
            }
            contexts.insert(key.symbol.context().to_string());
        }
        for (context, _) in self.symbol_generations.keys() {
            contexts.insert(normalize_context_name(context));
        }

        Ok(Expr::call(
            system_expr("List"),
            contexts
                .into_iter()
                .filter(|context| {
                    pattern.is_none_or(|pattern| {
                        self.wildcard_name_pattern_matches(pattern, context, false)
                    })
                })
                .map(Expr::string)
                .collect(),
        ))
    }

    pub(super) fn eval_file_names(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if let Err(denied) = self.ensure_capability_allowed("FileNames", Capability::FileRead) {
            return Ok(denied);
        }

        if !self.has_file_io() {
            self.emit_no_file_io_message();
            return Ok(system_expr("$Failed"));
        }

        if args.len() > 3 {
            return Ok(self.invalid_call_with_arg_range("FileNames", args, 0, 3));
        }
        let Some(config) = parse_file_names_args(args) else {
            return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
        };

        let file_io = self
            .file_io()
            .expect("FileNames checked that a file provider is installed");
        let matches =
            collect_file_name_matches(self, file_io, &config.pattern, config.dirs, config.depth);

        Ok(Expr::call(
            system_expr("List"),
            matches.into_iter().map(Expr::string).collect(),
        ))
    }

    pub(super) fn eval_find_file(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let spec = match self.compile_load_target_from_resolved_args(args) {
            Ok(spec) => spec,
            Err(LoaderSpecNormalizationError::WrongArgCount { .. }) => {
                return Ok(self.invalid_call_with_exact_arg_count("FindFile", args, 1));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "FindFile",
                    args,
                    error
                        .message()
                        .expect("loader target normalization errors should have messages"),
                ));
            }
        };

        if !self.has_file_io() {
            self.emit_no_file_io_message();
            return Ok(system_expr("$Failed"));
        }

        Ok(match self.find_load_target(&spec) {
            Some((path, _)) => Expr::string(path.display().to_string()),
            None => {
                self.emit_loader_message(
                    "FindFile",
                    &EvalError::LoadPathNotFound {
                        target: spec.clone(),
                        searched: self.load_target_candidates(&spec).1,
                    },
                );
                system_expr("$Failed")
            }
        })
    }

    pub(super) fn eval_get(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if let Err(denied) = self.ensure_capability_allowed("Get", Capability::FileRead) {
            return Ok(denied);
        }

        let spec = match self.compile_load_target_from_resolved_args(args) {
            Ok(spec) => spec,
            Err(LoaderSpecNormalizationError::WrongArgCount { .. }) => {
                return Ok(self.invalid_call_with_exact_arg_count("Get", args, 1));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "Get",
                    args,
                    error
                        .message()
                        .expect("loader target normalization errors should have messages"),
                ));
            }
        };

        if !self.has_file_io() {
            self.emit_no_file_io_message();
            return Ok(system_expr("$Failed"));
        }

        let path = match self.resolve_load_target(&spec) {
            Ok(path) => path,
            Err(err) => {
                self.emit_loader_message("Get", &err);
                return Ok(system_expr("$Failed"));
            }
        };

        self.start_guest_get(path)
    }

    pub(super) fn eval_needs(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if let Err(denied) = self.ensure_capability_allowed("Needs", Capability::FileRead) {
            return Ok(denied);
        }

        let config = match self.compile_needs_call_from_resolved_args(args) {
            Ok(config) => config,
            Err(LoaderSpecNormalizationError::WrongArgCount { expected, actual }) => {
                return Ok(self.invalid_call_with_message(
                    "Needs",
                    args,
                    "argx",
                    format!("expected {expected} argument(s), got {actual}"),
                ));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "Needs",
                    args,
                    error
                        .message()
                        .expect("loader spec normalization errors should have messages"),
                ));
            }
        };

        if self
            .loaded_packages
            .iter()
            .any(|loaded| loaded == &config.context)
        {
            return Ok(system_expr("Null"));
        }

        if !self.has_file_io() {
            self.emit_no_file_io_message();
            return Ok(system_expr("$Failed"));
        }

        let outcome = self.eval_get(&[Expr::string(config.load_spec.clone())]);
        if matches!(outcome, Err(EvalError::InputRequired { .. })) {
            crate::resume_or_suspend!(insert self, NeedsCompleteFrame { config });
            outcome
        } else {
            self.finish_guest_needs(&config, outcome)
        }
    }

    pub(super) fn eval_begin(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [context] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Begin", args, 1));
        };

        let Expr::Atom(Atom::String(context)) = context else {
            return Ok(self.invalid_call_with_arg_form(
                "Begin",
                args,
                "expected a string context name",
            ));
        };

        let target = expand_context_spec(context, &self.current_context);
        Ok(self.begin_context(target))
    }

    pub(super) fn eval_end(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("End", args, 0));
        }

        Ok(self.end_context().unwrap_or_else(|| {
            self.invalid_call_with_message(
                "End",
                &[],
                "context",
                "no matching Begin[] frame is active",
            )
        }))
    }

    pub(super) fn eval_begin_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let config = match self.compile_begin_package_call_from_resolved_args(args) {
            Ok(config) => config,
            Err(LoaderSpecNormalizationError::WrongArgCount { expected, actual }) => {
                return Ok(self.invalid_call_with_message(
                    "BeginPackage",
                    args,
                    "argx",
                    format!("expected {expected} argument(s), got {actual}"),
                ));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "BeginPackage",
                    args,
                    error
                        .message()
                        .expect("loader spec normalization errors should have messages"),
                ));
            }
        };

        let context_depth = self.context_stack.len();
        let package_count = self.loaded_packages.len();
        let begin_result = self.begin_package_context(config.target_context.clone());
        let rollback = BeginPackageRollback {
            frame: self.context_stack[context_depth].clone(),
            context_depth,
            package_count,
        };
        self.drive_begin_package_dependencies(BeginPackageDependencyFrame {
            config: Arc::new(config),
            dependency: 0,
            begin_result,
            rollback,
        })
    }

    pub(super) fn eval_end_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("EndPackage", args, 0));
        }

        Ok(self.end_package_context().unwrap_or_else(|| {
            self.invalid_call_with_message(
                "EndPackage",
                &[],
                "context",
                "no matching BeginPackage[] frame is active",
            )
        }))
    }

    fn compile_load_target_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<String, LoaderSpecNormalizationError> {
        match args {
            [spec] => {
                load_spec_from_expr(spec).ok_or(LoaderSpecNormalizationError::InvalidLoadTarget)
            }
            _ => Err(LoaderSpecNormalizationError::WrongArgCount {
                expected: "1",
                actual: args.len(),
            }),
        }
    }

    fn compile_needs_call_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<NeedsLoadConfig, LoaderSpecNormalizationError> {
        match args {
            [Expr::Atom(Atom::String(context))] => {
                let context = normalize_context_name(context);
                Ok(NeedsLoadConfig {
                    load_spec: context.clone(),
                    context,
                })
            }
            [Expr::Atom(Atom::String(context)), spec] => Ok(NeedsLoadConfig {
                context: normalize_context_name(context),
                load_spec: load_spec_from_expr(spec)
                    .ok_or(LoaderSpecNormalizationError::InvalidNeedsLoadTarget)?,
            }),
            [_, ..] if args.len() <= 2 => Err(LoaderSpecNormalizationError::InvalidNeedsContext),
            _ => Err(LoaderSpecNormalizationError::WrongArgCount {
                expected: "1 or 2",
                actual: args.len(),
            }),
        }
    }

    fn compile_begin_package_call_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<BeginPackageConfig, LoaderSpecNormalizationError> {
        match args {
            [Expr::Atom(Atom::String(context))] => Ok(BeginPackageConfig {
                target_context: expand_context_spec(context, &self.current_context),
                dependencies: Vec::new(),
            }),
            [Expr::Atom(Atom::String(context)), dependencies] => Ok(BeginPackageConfig {
                target_context: expand_context_spec(context, &self.current_context),
                dependencies: expr_as_string_list(dependencies)
                    .ok_or(LoaderSpecNormalizationError::InvalidBeginPackageDependencies)?
                    .into_iter()
                    .map(normalize_context_name)
                    .collect(),
            }),
            [_, ..] if args.len() <= 2 => {
                Err(LoaderSpecNormalizationError::InvalidBeginPackageContext)
            }
            _ => Err(LoaderSpecNormalizationError::WrongArgCount {
                expected: "1 or 2",
                actual: args.len(),
            }),
        }
    }
}

fn begin_package_context_path(target_context: &str, dependencies: &[String]) -> Vec<String> {
    let mut path = vec![normalize_context_name(target_context)];

    for dependency in dependencies {
        let dependency = normalize_context_name(dependency);
        if !path.iter().any(|existing| existing == &dependency) {
            path.push(dependency);
        }
    }

    if !path.iter().any(|existing| existing == SYSTEM_CONTEXT) {
        path.push(SYSTEM_CONTEXT.to_string());
    }

    path
}
