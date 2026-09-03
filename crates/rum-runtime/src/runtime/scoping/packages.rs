mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

pub(in crate::runtime) const SEALED_IDENTITY_MARKER: u64 = 1 << 62;

pub(in crate::runtime) fn symbol_is_sealed_private(symbol: &Symbol) -> bool {
    symbol
        .identity()
        .is_some_and(|identity| identity & SEALED_IDENTITY_MARKER != 0)
}

pub(in crate::runtime) fn sealed_symbols_in_expr(expr: &Expr) -> BTreeSet<Symbol> {
    fn collect(expr: &Expr, sealed: &mut BTreeSet<Symbol>) {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) if symbol_is_sealed_private(symbol) => {
                sealed.insert(*symbol);
            }
            Expr::Call { head, args, .. } => {
                collect(head, sealed);
                for arg in args {
                    collect(arg, sealed);
                }
            }
            _ => {}
        }
    }

    let mut sealed = BTreeSet::new();
    collect(expr, &mut sealed);
    sealed
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct SourceUnit {
    pub(in crate::runtime) package_context: String,
    pub(in crate::runtime) held_wrapper: Expr,
    pub(in crate::runtime) epoch: u64,
    pub(in crate::runtime) public: Vec<Symbol>,
    pub(in crate::runtime) package_scope: Vec<Symbol>,
    pub(in crate::runtime) private_index: BTreeMap<String, Symbol>,
}

#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) struct PackageInstallPlan {
    forms: Vec<Expr>,
    source_unit: SourceUnit,
    minted_sealed: Vec<Symbol>,
}

impl PackageInstallPlan {
    fn into_continuation_expr(self, index: usize) -> Expr {
        let source_unit = self.source_unit;
        let private_index = source_unit
            .private_index
            .into_iter()
            .map(|(name, symbol)| {
                Expr::call(
                    system_expr("InternalContinuationPackagePrivateSymbol"),
                    vec![Expr::string(name), Expr::Atom(Atom::Symbol(symbol))],
                )
            })
            .collect();
        Expr::call(
            system_expr("InternalContinuationPackageInstall"),
            vec![
                self.forms.into_continuation_expr(),
                Expr::string(source_unit.package_context),
                source_unit.held_wrapper,
                Expr::string(source_unit.epoch.to_string()),
                symbols_into_continuation_expr(source_unit.public),
                symbols_into_continuation_expr(source_unit.package_scope),
                Expr::call(
                    system_expr("InternalContinuationPackagePrivateIndex"),
                    private_index,
                ),
                symbols_into_continuation_expr(self.minted_sealed),
                Expr::string(index.to_string()),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> (Self, usize) {
        let Expr::Call { head, args, .. } = expr else {
            panic!("unexpected package-install continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationPackageInstall"),
            "unexpected package-install continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let forms = Vec::<Expr>::from_continuation_expr(
            args.next().expect("package install stores its forms"),
        );
        let package_context = continuation_string(
            args.next().expect("package install stores its context"),
            "package context",
        );
        let held_wrapper = args
            .next()
            .expect("package install stores its held wrapper");
        let epoch = continuation_string(
            args.next().expect("package install stores its epoch"),
            "package epoch",
        )
        .parse()
        .expect("package continuation epoch should be a u64");
        let public = symbols_from_continuation_expr(
            args.next()
                .expect("package install stores its public symbols"),
        );
        let package_scope = symbols_from_continuation_expr(
            args.next()
                .expect("package install stores its package-scope symbols"),
        );
        let private_index_expr = args
            .next()
            .expect("package install stores its private index");
        let Expr::Call {
            head: private_head,
            args: private_entries,
            ..
        } = private_index_expr
        else {
            panic!("unexpected package private-index continuation payload")
        };
        assert!(is_system_symbol_name(
            &private_head,
            "InternalContinuationPackagePrivateIndex"
        ));
        let private_index = private_entries
            .into_iter()
            .map(|entry| {
                let Expr::Call { head, args, .. } = entry else {
                    panic!("unexpected package private-symbol continuation payload")
                };
                assert!(is_system_symbol_name(
                    &head,
                    "InternalContinuationPackagePrivateSymbol"
                ));
                let mut args = args.into_iter();
                let name = continuation_string(
                    args.next().expect("private symbol stores its name"),
                    "private symbol name",
                );
                let symbol =
                    continuation_symbol(args.next().expect("private symbol stores its symbol"));
                debug_assert!(args.next().is_none());
                (name, symbol)
            })
            .collect();
        let minted_sealed = symbols_from_continuation_expr(
            args.next()
                .expect("package install stores its minted symbols"),
        );
        let index = continuation_string(
            args.next().expect("package install stores its cursor"),
            "package cursor",
        )
        .parse()
        .expect("package continuation cursor should be a usize");
        debug_assert!(args.next().is_none());
        (
            Self {
                forms,
                source_unit: SourceUnit {
                    package_context,
                    held_wrapper,
                    epoch,
                    public,
                    package_scope,
                    private_index,
                },
                minted_sealed,
            },
            index,
        )
    }
}

fn symbols_into_continuation_expr(symbols: Vec<Symbol>) -> Expr {
    Expr::call(
        system_expr("InternalContinuationPackageSymbols"),
        symbols
            .into_iter()
            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
            .collect(),
    )
}

fn symbols_from_continuation_expr(expr: Expr) -> Vec<Symbol> {
    let Expr::Call { head, args, .. } = expr else {
        panic!("unexpected package-symbol continuation payload")
    };
    assert!(is_system_symbol_name(
        &head,
        "InternalContinuationPackageSymbols"
    ));
    args.into_iter().map(continuation_symbol).collect()
}

fn continuation_symbol(expr: Expr) -> Symbol {
    let Expr::Atom(Atom::Symbol(symbol)) = expr else {
        panic!("package continuation symbol should be a symbol: {expr:?}")
    };
    symbol
}

fn continuation_string(expr: Expr, field: &str) -> String {
    let Expr::Atom(Atom::String(value)) = expr else {
        panic!("package continuation {field} should be a string: {expr:?}")
    };
    value
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedPackageInstall {
    plan: Arc<PackageInstallPlan>,
    index: usize,
}

#[derive(Default)]
struct PackageDeclarations {
    public: Vec<Symbol>,
    package_scope: Vec<Symbol>,
}

impl Session {
    pub(in crate::runtime) fn sealed_symbols_visible(&self) -> bool {
        self.capability_policy.allows(Capability::Diagnostic)
    }

    pub(in crate::runtime) fn eval_define_package(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let Some(package_context) = args.first().and_then(valid_package_context_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "DefinePackage",
                args,
                "expected a non-empty valid context string with a trailing context separator as the first argument",
            ));
        };

        let held_wrapper = Expr::call(system_expr("DefinePackage"), args.to_vec());
        let (declarations, body_forms) =
            match collect_package_declarations_and_body(&args[1..], &self.current_context) {
                Ok(parts) => parts,
                Err(detail) => return Ok(self.package_declaration_failure(detail)),
            };

        let bound = scan_package_bindings(self, &body_forms, &self.current_context);
        let mut rewrites = SymbolExprMap::default();
        let mut public = Vec::with_capacity(declarations.public.len());
        let mut package_scope = Vec::with_capacity(declarations.package_scope.len());
        let public_names = declarations
            .public
            .iter()
            .map(|symbol| symbol.name().to_string())
            .collect::<BTreeSet<_>>();
        let package_scope_names = declarations
            .package_scope
            .iter()
            .map(|symbol| symbol.name().to_string())
            .collect::<BTreeSet<_>>();

        for source in declarations.public {
            let target = Symbol::new(package_context.clone(), source.name());
            rewrites.insert(source, Expr::Atom(Atom::Symbol(target)));
            public.push(target);
        }

        let package_scope_context = format!("{package_context}PackageScope`");
        for source in declarations.package_scope {
            let target = Symbol::new(package_scope_context.clone(), source.name());
            rewrites.insert(source, Expr::Atom(Atom::Symbol(target)));
            package_scope.push(target);
        }

        let private_context = format!("{package_context}Private`");
        let mut private_index = BTreeMap::new();
        for source in bound {
            if public_names.contains(source.name()) || package_scope_names.contains(source.name()) {
                continue;
            }

            let target = self.allocate_sealed_symbol(&private_context, source.name());
            rewrites.insert(source, Expr::Atom(Atom::Symbol(target)));
            private_index.insert(target.name().to_string(), target);
        }

        let forms = body_forms
            .iter()
            .map(|form| substitute_lexical_bindings(form, &rewrites, &BTreeSet::new()))
            .collect();
        let minted_sealed = private_index.values().copied().collect::<Vec<_>>();
        let replacement_index = self
            .sealed_units
            .iter()
            .position(|unit| unit.package_context == package_context);

        if let Some(index) = replacement_index {
            let old_sealed = self.sealed_units[index]
                .private_index
                .values()
                .copied()
                .collect::<Vec<_>>();
            for symbol in old_sealed {
                self.clear_symbol_state(&symbol, true, true, true, true, true);
            }
        }

        self.resume_package_install(
            PackageInstallPlan {
                forms,
                source_unit: SourceUnit {
                    package_context,
                    held_wrapper,
                    epoch: 0,
                    public,
                    package_scope,
                    private_index,
                },
                minted_sealed,
            },
            0,
        )
    }

    pub(in crate::runtime) fn eval_restore_sealed_references(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !self.restoring_snapshot {
            self.push_message(
                "RestoreSealedReferences",
                "mode",
                "RestoreSealedReferences is only valid inside a snapshot restore.",
            );
            return Ok(system_expr("$Failed"));
        }

        let [units_expr, held_expr] = args else {
            return Ok(self.invalid_call_with_arg_form(
                "RestoreSealedReferences",
                args,
                "expected a unit context or list of unit contexts and HoldComplete[form]",
            ));
        };
        let Some(contexts) = restore_unit_contexts(units_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "RestoreSealedReferences",
                args,
                "expected a unit context string, context rule, or list of them",
            ));
        };
        let Expr::Call {
            head: held_head,
            args: held_args,
            ..
        } = held_expr
        else {
            return Ok(self.invalid_call_with_arg_form(
                "RestoreSealedReferences",
                args,
                "expected the second argument to be HoldComplete[form]",
            ));
        };
        if !is_system_symbol_name(held_head, "HoldComplete") || held_args.len() != 1 {
            return Ok(self.invalid_call_with_arg_form(
                "RestoreSealedReferences",
                args,
                "expected the second argument to be HoldComplete[form]",
            ));
        }

        let mut rewrites = SymbolExprMap::default();
        for context in contexts {
            let Some(unit) = self
                .sealed_units
                .iter()
                .find(|unit| unit.package_context == context)
            else {
                self.push_message(
                    "RestoreSealedReferences",
                    "unit",
                    format!("Snapshot restore could not find current sealed unit {context}."),
                );
                return Ok(system_expr("$Failed"));
            };
            let private_context = format!("{}Private`", unit.package_context);
            for (name, sealed) in &unit.private_index {
                rewrites.insert(
                    Symbol::new(private_context.clone(), name),
                    Expr::Atom(Atom::Symbol(*sealed)),
                );
            }
        }

        let rewritten = substitute_lexical_bindings(&held_args[0], &rewrites, &BTreeSet::new());
        let result = self.evaluate(rewritten);
        crate::resume_or_suspend!(
            self,
            result,
            ok(_value_expr) => { let _session = self; let _value = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(_value_expr); (|| { Ok(system_expr("Null")) })() },
            suspend => EvalRestoreSealedReferencesSuspensionFrame,
        )
    }

    pub(in crate::runtime) fn sealed_source_unit_for_symbol(
        &self,
        symbol: &Symbol,
    ) -> Option<&SourceUnit> {
        self.sealed_units
            .iter()
            .find(|unit| unit.private_index.values().any(|sealed| sealed == symbol))
    }

    fn allocate_sealed_symbol(&mut self, private_context: &str, name: &str) -> Symbol {
        // Session-scoped counter: identities must be distinct within one
        // session's lifetime (nothing in-language survives a reset), so
        // numeric reuse across independent sessions is permitted.
        self.next_sealed_symbol_id += 1;
        Symbol::with_identity(
            private_context,
            name,
            SEALED_IDENTITY_MARKER | self.next_sealed_symbol_id,
        )
    }

    fn resume_package_install(
        &mut self,
        plan: PackageInstallPlan,
        mut index: usize,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(form) = plan.forms.get(index) else {
                return Ok(self.finish_package_install(plan));
            };

            match self.evaluate(form.clone()) {
                Ok(_) => index += 1,
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedPackageInstall {
                        plan: Arc::new(plan),
                        index,
                    };
                    crate::resume_or_suspend!(insert self, ResumePackageInstallSuspensionFrame { state });
                    crate::resume_or_suspend!(insert self, ContinuePackageInstallFrame);
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Ok(self.fail_package_install(&plan, error)),
            }
        }
    }

    fn finish_package_install(&mut self, mut plan: PackageInstallPlan) -> Expr {
        self.next_source_unit_epoch += 1;
        plan.source_unit.epoch = self.next_source_unit_epoch;
        let package_context = plan.source_unit.package_context.clone();

        if let Some(index) = self
            .sealed_units
            .iter()
            .position(|unit| unit.package_context == package_context)
        {
            self.sealed_units[index] = plan.source_unit;
        } else {
            self.sealed_units.push(plan.source_unit);
        }

        Expr::string(package_context)
    }

    fn fail_package_install(&mut self, plan: &PackageInstallPlan, error: EvalError) -> Expr {
        self.rollback_package_install(plan);

        self.push_message(
            "DefinePackage",
            "failed",
            format!(
                "Registration of {} failed while evaluating its body: {error}",
                plan.source_unit.package_context
            ),
        );
        system_expr("$Failed")
    }

    pub(in crate::runtime) fn rollback_package_install(&mut self, plan: &PackageInstallPlan) {
        for symbol in &plan.minted_sealed {
            self.clear_symbol_state(symbol, true, true, true, true, true);
        }

        if let Some(index) = self
            .sealed_units
            .iter()
            .position(|unit| unit.package_context == plan.source_unit.package_context)
        {
            self.sealed_units.remove(index);
        }
    }

    fn package_declaration_failure(&mut self, detail: &str) -> Expr {
        self.push_message("DefinePackage", "decl", detail);
        system_expr("$Failed")
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedPackageInstall>();
};

fn restore_unit_contexts(expr: &Expr) -> Option<Vec<String>> {
    if let Expr::Call { head, args, .. } = expr {
        if is_system_symbol_name(head, "List") {
            let contexts = args
                .iter()
                .map(restore_unit_context)
                .collect::<Option<Vec<_>>>()?;
            return (!contexts.is_empty()).then_some(contexts);
        }
    }

    Some(vec![restore_unit_context(expr)?])
}

fn restore_unit_context(expr: &Expr) -> Option<String> {
    if let Expr::Call { head, args, .. } = expr {
        if is_system_symbol_name(head, "Rule") {
            let [context, _epoch] = args.as_slice() else {
                return None;
            };
            return valid_package_context_expr(context);
        }
    }

    valid_package_context_expr(expr)
}

fn valid_package_context_expr(expr: &Expr) -> Option<String> {
    let Expr::Atom(Atom::String(context)) = expr else {
        return None;
    };

    let body = context.strip_suffix('`')?;
    if body.is_empty() || !body.split('`').all(valid_context_part) {
        return None;
    }

    Some(normalize_context_name(context.clone()))
}

fn valid_context_part(part: &str) -> bool {
    let mut chars = part.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '$')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '$')
}

fn collect_package_declarations_and_body(
    args: &[Expr],
    current_context: &str,
) -> Result<(PackageDeclarations, Vec<Expr>), &'static str> {
    let mut declarations = PackageDeclarations::default();
    let mut saw_public = false;
    let mut saw_package_scope = false;
    let mut body = Vec::new();

    for arg in args {
        let Expr::Call { head, args, .. } = arg else {
            flatten_compound_expression(arg, &mut body);
            continue;
        };

        let target = if is_system_symbol_name(head, "Public") {
            if saw_public {
                return Err("Public may be declared at most once per source unit");
            }
            saw_public = true;
            &mut declarations.public
        } else if is_system_symbol_name(head, "PackageScope") {
            if saw_package_scope {
                return Err("PackageScope may be declared at most once per source unit");
            }
            saw_package_scope = true;
            &mut declarations.package_scope
        } else {
            flatten_compound_expression(arg, &mut body);
            continue;
        };

        let [
            Expr::Call {
                head: list_head,
                args: symbols,
                ..
            },
        ] = args.as_slice()
        else {
            return Err("package declarations must have the form Head[{symbols...}]");
        };
        if !is_system_symbol_name(list_head, "List") {
            return Err("package declarations must contain a list of plain symbols");
        }

        let mut seen = target.iter().copied().collect::<BTreeSet<_>>();
        for item in symbols {
            let Some(symbol) = expr_as_symbol(item).copied() else {
                return Err("package declarations must contain only plain symbols");
            };
            if symbol.context() != current_context {
                return Err("package declarations must contain only current-context symbols");
            }
            if seen.insert(symbol) {
                target.push(symbol);
            }
        }
    }

    let public_names = declarations
        .public
        .iter()
        .map(Symbol::name)
        .collect::<BTreeSet<_>>();
    if declarations
        .package_scope
        .iter()
        .any(|symbol| public_names.contains(symbol.name()))
    {
        return Err("a symbol cannot be both Public and PackageScope");
    }

    Ok((declarations, body))
}

fn flatten_compound_expression(expr: &Expr, forms: &mut Vec<Expr>) {
    if let Expr::Call { head, args, .. } = expr {
        if is_system_symbol_name(head, "CompoundExpression") {
            for arg in args {
                flatten_compound_expression(arg, forms);
            }
            return;
        }
    }

    forms.push(expr.clone());
}

fn scan_package_bindings(
    session: &Session,
    forms: &[Expr],
    current_context: &str,
) -> BTreeSet<Symbol> {
    let mut bound = BTreeSet::new();

    for form in forms {
        let Expr::Call { head, args, .. } = form else {
            continue;
        };

        if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "SetDelayed"))
            && args.len() == 2
        {
            if let Some(target) = session.normalize_definition_assignment_target(&args[0]) {
                let symbol = match target {
                    DefinitionAssignmentTarget::Message(key) => key.symbol,
                    DefinitionAssignmentTarget::SyntaxInformation(symbol) => symbol,
                    DefinitionAssignmentTarget::Definition(target) => target.symbol().to_owned(),
                };
                insert_package_local(&mut bound, symbol, current_context);
            }
            continue;
        }

        if (is_system_symbol_name(head, "TagSet") || is_system_symbol_name(head, "TagSetDelayed"))
            && args.len() == 3
        {
            if let Some(symbol) = expr_as_symbol(&args[0]).copied() {
                insert_package_local(&mut bound, symbol, current_context);
            }
            continue;
        }

        if (is_system_symbol_name(head, "UpSet") || is_system_symbol_name(head, "UpSetDelayed"))
            && args.len() == 2
        {
            for symbol in collect_upset_target_symbols(&args[0]) {
                insert_package_local(&mut bound, symbol, current_context);
            }
            continue;
        }

        if (is_system_symbol_name(head, "SetAttributes")
            || is_system_symbol_name(head, "SetOptions"))
            && !args.is_empty()
        {
            collect_package_symbol_targets(&args[0], current_context, &mut bound);
        }
    }

    bound
}

fn collect_package_symbol_targets(
    expr: &Expr,
    current_context: &str,
    bound: &mut BTreeSet<Symbol>,
) {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            insert_package_local(bound, *symbol, current_context);
        }
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
            for arg in args {
                collect_package_symbol_targets(arg, current_context, bound);
            }
        }
        _ => {}
    }
}

fn insert_package_local(bound: &mut BTreeSet<Symbol>, symbol: Symbol, current_context: &str) {
    if symbol.context() == current_context {
        bound.insert(symbol);
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<PackageInstallPlan>();
    assert_traces::<SourceUnit>();
};
