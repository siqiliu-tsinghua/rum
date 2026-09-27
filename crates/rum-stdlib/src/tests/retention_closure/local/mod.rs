//! Local lint: temporal ownership checks and mark lookups, never call traversal.
//! Rejected syntax is still examined for further diagnostics, not admitted.

mod admission_tests;
mod callbacks;
mod calls;
mod cleanup;
mod contract_tests;
mod contracts;
mod expressions;
mod flow;
mod interface;
mod marks;
mod oracle;
mod production;
mod protocol;
mod reads;
mod surface;
mod syntax;
mod tests;
mod type_tests;

use super::{methods, model::*, resolve, types};
use interface::Interfaces;
use marks::Marks;
use std::collections::{BTreeMap, BTreeSet};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

#[derive(Default, Debug)]
pub(super) struct Report {
    pub rejected: BTreeSet<Diagnostic>,
    pub facts: BTreeMap<Site, ExactTy>,
    pub marks: BTreeMap<Id, String>,
    pub calls: BTreeMap<Site, bool>,
    pub bindings: BTreeMap<Site, ExactTy>,
    pub targets: BTreeMap<Site, BTreeSet<Id>>,
}

impl Report {
    pub fn text(&self) -> String {
        self.rejected.iter().map(|d| format!("{d}\n")).collect()
    }
}

fn check(index: &Interfaces, f: &Function, marks: &Marks) -> Report {
    let marks = marks.with_covered(&index.earned);
    let source = &index.signatures;
    let generic = types::mode_parameter(source, &f.module, &f.sig.generics);
    if matches!(generic, Ok(Some(_))) {
        let mut combined = Report::default();
        for id in types::MODES {
            let mode = ExactTy::Nominal(id.into(), Vec::new());
            let instance = match types::with_mode(source, f, Some(&mode)) {
                Ok(f) => f,
                Err(reason) => {
                    combined.rejected.insert(Diagnostic {
                        kind: Kind::RET002,
                        site: f.site.clone(),
                        target: None,
                        witness: vec![f.id.clone()],
                        reason,
                    });
                    continue;
                }
            };
            let report = check_instance(index, &instance, &marks);
            combined.rejected.extend(report.rejected);
            combined.facts.extend(report.facts);
            combined.marks.extend(report.marks);
            combined.calls.extend(report.calls);
            combined.bindings.extend(report.bindings);
            combined.targets.extend(report.targets);
        }
        return combined;
    }
    check_instance(index, f, &marks)
}

fn check_instance(index: &Interfaces, f: &Function, marks: &Marks) -> Report {
    let source = &index.signatures;
    let mut lint = Lint {
        index,
        f,
        marks: marks.view(index),
        inert: marks.inert_targets(),
        report: Report::default(),
        locals: BTreeMap::new(),
        methods: methods::claims(source, f),
        constructors: types::constructor_claims(source, f)
            .into_iter()
            .map(|c| (c.site, c.exact))
            .collect(),
        output: types::exact_output(source, f).ok(),
        diverges: false,
        argument_hazard: None,
    };
    if !source.errors.is_empty() || !source.modules[&f.module].hazards.is_empty() {
        lint.reject(
            f.sig.span(),
            Kind::RET002,
            None,
            "unresolved source or item-generating scope",
        );
    }
    struct Attributes(Vec<proc_macro2::Span>);
    impl<'a> Visit<'a> for Attributes {
        fn visit_attribute(&mut self, a: &'a syn::Attribute) {
            self.0.push(a.span());
        }
    }
    let mut attributes = Attributes(Vec::new());
    attributes.visit_block(&f.body);
    for span in attributes.0 {
        lint.reject(
            span,
            Kind::RET002,
            None,
            "expression/statement attribute is not admitted",
        );
    }
    if let Err(reason) = types::attributes(&f.attrs) {
        lint.reject(f.sig.span(), Kind::RET002, None, reason);
    }
    if f.conditional
        || !f.sig.generics.params.is_empty()
        || !f.impl_generics.params.is_empty()
        || f.sig.generics.where_clause.is_some()
        || f.sig.unsafety.is_some()
        || f.sig.asyncness.is_some()
        || f.sig.abi.is_some()
        || f.sig.variadic.is_some()
    {
        lint.reject(
            f.sig.span(),
            Kind::RET002,
            None,
            "signature outside A1/A2 admissions",
        );
    }
    for input in &f.sig.inputs {
        if let syn::FnArg::Typed(a) = input {
            if matches!(&*a.pat, syn::Pat::Wild(_))
                && matches!(
                    types::in_function(source, f, &a.ty),
                    Ok(ExactTy::Reference(..))
                )
            {
                continue;
            }
        }
        let mut one = f.clone();
        one.sig.inputs = [input.clone()].into_iter().collect();
        match types::exact_parameters(source, &one) {
            Ok(parameters) => {
                for (n, t) in parameters {
                    let ownership = cleanup::ownership(source, &t);
                    if cleanup::unknown(source, &t) {
                        lint.reject(
                            input.span(),
                            Kind::RET002,
                            None,
                            format!("unadmitted parameter cleanup: {ownership:?}"),
                        );
                    }
                    lint.locals.insert(n, Some(t));
                }
            }
            Err(reason) => lint.reject(input.span(), Kind::RET002, None, reason),
        }
    }
    let output = types::exact_output(source, f);
    if let Err(reason) = &output {
        lint.reject(f.sig.span(), Kind::RET002, None, reason);
    }
    lint.block(&f.body, output.as_ref().ok());
    flow::check(source, f, &mut lint.report);
    lint.report
}

struct Lint<'a> {
    index: &'a Interfaces,
    f: &'a Function,
    marks: BTreeMap<Id, Result<String, String>>,
    inert: BTreeSet<Id>,
    report: Report,
    locals: BTreeMap<String, Option<ExactTy>>,
    methods: BTreeMap<Site, Result<methods::Selection, String>>,
    constructors: BTreeMap<Site, ExactTy>,
    output: Option<ExactTy>,
    diverges: bool,
    argument_hazard: Option<Site>,
}

impl Lint<'_> {
    fn source(&self) -> &Source {
        &self.index.signatures
    }

    fn reject(
        &mut self,
        span: proc_macro2::Span,
        kind: Kind,
        target: Option<Id>,
        reason: impl Into<String>,
    ) {
        self.report.rejected.insert(Diagnostic {
            kind,
            site: Site::new(&self.f.site.file, span),
            target,
            witness: vec![self.f.id.clone()],
            reason: reason.into(),
        });
    }

    fn call_mark(&mut self, span: proc_macro2::Span, target: &str) {
        let hazard = !self.inert.contains(target);
        self.report
            .calls
            .entry(Site::new(&self.f.site.file, span))
            .and_modify(|v| *v |= hazard)
            .or_insert(hazard);
        match self
            .marks
            .get(target)
            .cloned()
            .unwrap_or_else(|| Marks::missing(target))
        {
            Ok(stamp) => {
                self.report
                    .targets
                    .entry(Site::new(&self.f.site.file, span))
                    .or_default()
                    .insert(target.into());
                self.report.marks.insert(target.into(), stamp);
            }
            Err(reason) => self.reject(span, Kind::RET002, Some(target.into()), reason),
        }
    }

    fn fact(&mut self, expr: &syn::Expr, ty: Option<ExactTy>) -> Option<ExactTy> {
        if let Some(ty) = &ty {
            // Reuse the effect module's ownership domain; never classify a call
            // by its spelling. Leased wrappers have distinct nominal identities.
            let ownership = cleanup::ownership(self.source(), ty);
            if cleanup::unknown(self.source(), ty) {
                self.reject(
                    expr.span(),
                    Kind::RET002,
                    None,
                    format!("unadmitted cleanup: {ownership:?}"),
                );
            }
            self.report
                .facts
                .insert(Site::new(&self.f.site.file, expr.span()), ty.clone());
        } else {
            self.reject(
                expr.span(),
                Kind::RET002,
                None,
                "expression exact type is unknown",
            ); // FALSIFIER:unknown-type
        }
        ty
    }

    fn block(&mut self, block: &syn::Block, expected: Option<&ExactTy>) -> Option<ExactTy> {
        let old = self.locals.clone();
        let mut tail = Some(ExactTy::Unit);
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Local(local) => {
                    if local.init.is_none() {
                        self.reject(
                            local.span(),
                            Kind::RET002,
                            None,
                            "uninitialized local is not admitted",
                        );
                    }
                    let annotated = match &local.pat {
                        syn::Pat::Type(p) => types::in_function(self.source(), self.f, &p.ty).ok(),
                        _ => None,
                    };
                    let value = local.init.as_ref().and_then(|init| {
                        let value = self.expr(&init.expr, annotated.as_ref());
                        if let Some((_, diverge)) = &init.diverge {
                            self.borrowed_let_else(local, value.clone(), diverge);
                        }
                        value
                    });
                    if annotated.is_some() && annotated != value {
                        self.reject(
                            local.span(),
                            Kind::RET002,
                            None,
                            "annotation lacks matching initializer type",
                        );
                    }
                    if !local.init.as_ref().is_some_and(|i| i.diverge.is_some()) {
                        self.bind(&local.pat, value);
                    }
                    tail = Some(ExactTy::Unit);
                }
                syn::Stmt::Expr(expr, semi) => {
                    let value = self.expr(expr, if semi.is_none() { expected } else { None });
                    tail = if semi.is_some() {
                        Some(ExactTy::Unit)
                    } else {
                        value
                    };
                }
                syn::Stmt::Macro(m) => {
                    self.macro_expr(&m.mac, None);
                    tail = None;
                }
                syn::Stmt::Item(item) => {
                    self.reject(
                        item.span(),
                        Kind::RET002,
                        None,
                        "block-local item/import is not admitted",
                    );
                    tail = None;
                }
            }
        }
        self.locals = old;
        tail
    }

    fn bind(&mut self, pat: &syn::Pat, value: Option<ExactTy>) {
        if let Some(ty) = &value {
            self.report
                .bindings
                .insert(Site::new(&self.f.site.file, pat.span()), ty.clone());
        }
        match pat {
            syn::Pat::Ident(p) if p.by_ref.is_none() && p.subpat.is_none() => {
                self.locals.insert(p.ident.to_string(), value);
            }
            syn::Pat::Type(p) => self.bind(&p.pat, value),
            syn::Pat::Tuple(p) => {
                if let Some(ExactTy::Tuple(ts)) = value {
                    if ts.len() == p.elems.len() {
                        for (p, t) in p.elems.iter().zip(ts) {
                            self.bind(p, Some(t));
                        }
                    } else {
                        self.reject(p.span(), Kind::RET002, None, "tuple binding arity mismatch");
                    }
                } else {
                    self.reject(
                        p.span(),
                        Kind::RET002,
                        None,
                        "tuple binding lacks exact tuple type",
                    );
                }
            }
            syn::Pat::Wild(_) => {}
            _ => {
                self.reject(
                    pat.span(),
                    Kind::RET002,
                    None,
                    "pattern state is not admitted by this local lint",
                );
                struct Names(Vec<String>);
                impl<'a> Visit<'a> for Names {
                    fn visit_pat_ident(&mut self, p: &'a syn::PatIdent) {
                        self.0.push(p.ident.to_string());
                    }
                }
                let mut names = Names(Vec::new());
                names.visit_pat(pat);
                self.locals.extend(names.0.into_iter().map(|n| (n, None)));
            }
        }
    }
}
