//! Located receiver provenance for diagnostic edges. This is deliberately
//! independent of ownership certification; effects checks every selected body.

use super::super::resolve;
use super::*;
use std::collections::BTreeMap;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

pub(in super::super) fn claims(
    source: &Source,
    f: &Function,
) -> BTreeMap<Site, Result<Selection, String>> {
    struct Conditional(bool);
    impl<'ast> Visit<'ast> for Conditional {
        fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
            self.0 |= a.path().is_ident("cfg") || a.path().is_ident("cfg_attr");
        }
    }
    let mut conditional = Conditional(f.conditional || types::attributes(&f.attrs).is_err());
    for input in &f.sig.inputs {
        conditional.0 |= match input {
            syn::FnArg::Typed(a) => !a.attrs.is_empty(),
            syn::FnArg::Receiver(r) => !r.attrs.is_empty(),
        };
    }
    conditional.visit_block(&f.body);
    let mut context = Context {
        source,
        f,
        locals: BTreeMap::new(),
        closed: !conditional.0,
        claims: BTreeMap::new(),
    };
    // An unrelated unknown generic parameter must not erase an independently
    // checked nominal parameter. Open generic receivers themselves stay unknown.
    for input in &f.sig.inputs {
        let mut single = f.clone();
        single.sig.inputs = [input.clone()].into_iter().collect();
        match input {
            syn::FnArg::Typed(a) => {
                if let Ok(ty) = types::in_function(source, f, &a.ty) {
                    let value = context.value(ty, a.span(), "checked parameter", true, false);
                    context.bind(&a.pat, Some(value));
                } else {
                    context.bind(&a.pat, None);
                }
            }
            syn::FnArg::Receiver(r) => {
                let value = types::exact_parameters(source, &single)
                    .ok()
                    .and_then(|p| p.into_iter().next())
                    .map(|(_, t)| {
                        context.value(
                            t,
                            r.span(),
                            "checked self parameter",
                            true,
                            r.mutability.is_some(),
                        )
                    });
                context.locals.insert("self".into(), value);
            }
        }
    }
    context.visit_block(&f.body);
    context.claims
}

struct Context<'a> {
    source: &'a Source,
    f: &'a Function,
    locals: BTreeMap<String, Option<Receiver>>,
    closed: bool,
    claims: BTreeMap<Site, Result<Selection, String>>,
}

impl Context<'_> {
    fn value(
        &self,
        exact: ExactTy,
        span: proc_macro2::Span,
        basis: &str,
        movable: bool,
        mutable: bool,
    ) -> Receiver {
        Receiver {
            exact,
            provenance: Provenance {
                source: Site::new(&self.f.site.file, span),
                basis: basis.into(),
                projections: Vec::new(),
            },
            mutable,
            movable,
        }
    }

    fn infer(&self, expr: &syn::Expr) -> Option<Receiver> {
        match expr {
            syn::Expr::Path(p) if p.attrs.is_empty() && p.qself.is_none() => {
                self.locals.get(&p.path.get_ident()?.to_string())?.clone()
            }
            syn::Expr::Paren(p) if p.attrs.is_empty() => self.infer(&p.expr),
            syn::Expr::Reference(r) if r.attrs.is_empty() => {
                let mut value = self.infer(&r.expr)?;
                if r.mutability.is_some() && !value.mutable {
                    return None;
                }
                value.exact = ExactTy::Reference(Box::new(value.exact), r.mutability.is_some());
                value.mutable = true;
                value.movable = true;
                value
                    .provenance
                    .projections
                    .push("explicit reference".into());
                Some(value)
            }
            syn::Expr::Field(p) if p.attrs.is_empty() => {
                let mut value = self.infer(&p.base)?;
                self.deref(&mut value);
                value.exact = types::fields(self.source, &value.exact, None)
                    .ok()?
                    .remove(&tokens(&p.member))?;
                value
                    .provenance
                    .projections
                    .push(format!("field {}", tokens(&p.member)));
                Some(value)
            }
            syn::Expr::Call(c) if c.attrs.is_empty() => {
                let syn::Expr::Path(p) = &*c.func else {
                    return None;
                };
                if p.qself.is_some()
                    || p.path
                        .get_ident()
                        .is_some_and(|n| self.locals.contains_key(&n.to_string()))
                {
                    return None;
                }
                let target = resolve::path(self.source, &self.f.module, &p.path).ok()?;
                let f = resolve::function(self.source, &target).ok()?;
                if !f.sig.generics.params.is_empty() || f.sig.asyncness.is_some() {
                    return None;
                }
                Some(self.value(
                    types::exact_output(self.source, f).ok()?,
                    c.span(),
                    "resolved return signature",
                    true,
                    true,
                ))
            }
            syn::Expr::MethodCall(c) => {
                let selection = self.selection(c).ok()?;
                Some(self.value(
                    types::exact_output(self.source, &selection.function).ok()?,
                    c.span(),
                    "resolved method return signature",
                    true,
                    true,
                ))
            }
            syn::Expr::Lit(l) => {
                let name = match &l.lit {
                    syn::Lit::Bool(_) => "bool",
                    syn::Lit::Int(n) if !n.suffix().is_empty() => n.suffix(),
                    _ => return None,
                };
                Some(self.value(
                    ExactTy::Scalar(name.into()),
                    l.span(),
                    "explicit scalar",
                    true,
                    true,
                ))
            }
            _ => None,
        }
    }

    fn selection(&self, call: &syn::ExprMethodCall) -> Result<Selection, String> {
        if !self.closed {
            return Err("method lexical scope contains local items/imports".into());
        }
        let receiver = self
            .infer(&call.receiver)
            .ok_or("method receiver lacks checked nominal/place provenance")?; // MUTATION:method-provenance
        select(self.source, self.f, call, receiver)
    }

    fn deref(&self, value: &mut Receiver) -> Option<bool> {
        let mut borrowed = None;
        while let ExactTy::Reference(t, m) = &value.exact {
            borrowed = Some(borrowed.unwrap_or(true) && *m);
            value.mutable = borrowed.unwrap();
            value.movable = false;
            value.exact = (**t).clone();
            value
                .provenance
                .projections
                .push("built-in dereference".into());
        }
        borrowed
    }

    fn bind(&mut self, pat: &syn::Pat, value: Option<Receiver>) {
        match pat {
            syn::Pat::Ident(p) if p.subpat.is_none() => {
                let value = value.map(|mut v| {
                    if p.by_ref.is_some() {
                        v.exact = ExactTy::Reference(Box::new(v.exact), p.mutability.is_some());
                    }
                    v.mutable = p.mutability.is_some();
                    v.movable = true;
                    v
                });
                self.locals.insert(p.ident.to_string(), value);
            }
            syn::Pat::Type(p) => self.bind(&p.pat, value),
            syn::Pat::Tuple(p) => {
                for (i, p) in p.elems.iter().enumerate() {
                    let child = value.clone().and_then(|mut v| {
                        let ExactTy::Tuple(ts) = &v.exact else {
                            return None;
                        };
                        v.exact = ts.get(i)?.clone();
                        v.provenance.projections.push(format!("tuple {i}"));
                        Some(v)
                    });
                    self.bind(p, child);
                }
            }
            syn::Pat::Struct(p) => self.bind_fields(
                &p.path,
                p.fields
                    .iter()
                    .map(|f| (tokens(&f.member), &*f.pat))
                    .collect(),
                value,
            ),
            syn::Pat::TupleStruct(p) => self.bind_fields(
                &p.path,
                p.elems
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (i.to_string(), p))
                    .collect(),
                value,
            ),
            _ => {
                struct Names(Vec<String>);
                impl<'ast> Visit<'ast> for Names {
                    fn visit_pat_ident(&mut self, p: &'ast syn::PatIdent) {
                        self.0.push(p.ident.to_string());
                    }
                }
                let mut names = Names(Vec::new());
                names.visit_pat(pat);
                for name in names.0 {
                    self.locals.insert(name, None);
                }
            }
        }
    }

    fn bind_fields(
        &mut self,
        path: &syn::Path,
        pats: Vec<(String, &syn::Pat)>,
        value: Option<Receiver>,
    ) {
        let schema = value.and_then(|mut v| {
            let borrowed = self.deref(&mut v);
            let ExactTy::Nominal(id, _) = &v.exact else {
                return None;
            };
            let target = resolve::path(self.source, &self.f.module, path).ok()?;
            let variant = if target == *id {
                None
            } else {
                Some(target.strip_prefix(&format!("{id}::"))?)
            };
            Some((
                types::fields(self.source, &v.exact, variant).ok()?,
                v,
                borrowed,
            ))
        });
        for (name, pat) in pats {
            let child = schema.as_ref().and_then(|(schema, base, borrowed)| {
                let mut v = base.clone();
                v.exact = schema.get(&name)?.clone();
                if let Some(m) = borrowed {
                    v.exact = ExactTy::Reference(Box::new(v.exact), *m);
                }
                v.provenance
                    .projections
                    .push(format!("pattern field {name}"));
                Some(v)
            });
            self.bind(pat, child);
        }
    }
}

impl<'ast> Visit<'ast> for Context<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let old = self.locals.clone();
        let closed = self.closed;
        self.closed &= !block.stmts.iter().any(|s| matches!(s, syn::Stmt::Item(_)));
        for stmt in &block.stmts {
            if let syn::Stmt::Local(local) = stmt {
                let mut inferred = local.init.as_ref().and_then(|i| self.infer(&i.expr));
                if let syn::Pat::Type(p) = &local.pat {
                    let declared = types::in_function(self.source, self.f, &p.ty).ok();
                    inferred = inferred.filter(|v| Some(&v.exact) == declared.as_ref());
                    if let Some(v) = &mut inferred {
                        v.provenance.basis = "annotation checked against initializer".into();
                    }
                }
                if !local.attrs.is_empty() {
                    inferred = None;
                }
                if let Some(init) = &local.init {
                    self.visit_expr(&init.expr);
                    if let Some((_, e)) = &init.diverge {
                        self.visit_expr(e);
                    }
                }
                self.bind(&local.pat, inferred);
            } else {
                visit::visit_stmt(self, stmt);
            }
        }
        self.locals = old;
        self.closed = closed;
    }
    fn visit_expr_match(&mut self, m: &'ast syn::ExprMatch) {
        self.visit_expr(&m.expr);
        let value = self.infer(&m.expr);
        for arm in &m.arms {
            let old = self.locals.clone();
            self.bind(&arm.pat, value.clone());
            if let Some((_, e)) = &arm.guard {
                self.visit_expr(e);
            }
            self.visit_expr(&arm.body);
            self.locals = old;
        }
    }
    fn visit_expr_for_loop(&mut self, f: &'ast syn::ExprForLoop) {
        self.visit_expr(&f.expr);
        let old = self.locals.clone();
        self.bind(&f.pat, None);
        self.visit_block(&f.body);
        self.locals = old;
    }
    fn visit_expr_method_call(&mut self, c: &'ast syn::ExprMethodCall) {
        self.claims.insert(site(self.f, c), self.selection(c));
        visit::visit_expr_method_call(self, c);
    }
    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_macro(&mut self, _: &'ast syn::Macro) {}
    fn visit_item(&mut self, _: &'ast syn::Item) {}
    fn visit_expr_let(&mut self, l: &'ast syn::ExprLet) {
        self.visit_expr(&l.expr);
        self.bind(&l.pat, None);
    }
}
