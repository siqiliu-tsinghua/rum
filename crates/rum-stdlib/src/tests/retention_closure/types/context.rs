//! Diagnostic type facts are not retention certificates. Expected enum types
//! flow from concrete return signatures through tails/returns/branches. A
//! method's name never supplies its type; its enclosing constructor can still
//! have a known instance. This deliberately does not attempt Rust inference.

use super::super::{model::*, resolve, typed_resume_macro};
use super::{arguments, exact, exact_output, exact_parameters, fields};
use std::collections::BTreeMap;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

#[derive(Clone, Debug)]
pub(in super::super) struct TypeClaim {
    pub site: Site,
    pub exact: ExactTy,
    pub target: Id,
}

pub(in super::super) fn constructor_claims(source: &Source, function: &Function) -> Vec<TypeClaim> {
    let mut context = Context {
        source,
        function,
        output: exact_output(source, function).ok(),
        expected: None,
        locals: exact_parameters(source, function)
            .unwrap_or_default()
            .into_iter()
            .map(|(n, t)| (n, Some(t)))
            .collect(),
        claims: Vec::new(),
    };
    context.expected = context.output.clone();
    context.visit_block(&function.body());
    context.claims
}

struct Context<'a> {
    source: &'a Source,
    function: &'a Function,
    output: Option<ExactTy>,
    expected: Option<ExactTy>,
    locals: BTreeMap<String, Option<ExactTy>>,
    claims: Vec<TypeClaim>,
}

impl Context<'_> {
    fn infer(&self, expr: &syn::Expr) -> Option<ExactTy> {
        match expr {
            syn::Expr::Path(p) if p.qself.is_none() => {
                self.locals.get(&p.path.get_ident()?.to_string())?.clone()
            }
            syn::Expr::Lit(l) => match &l.lit {
                syn::Lit::Bool(_) => Some(ExactTy::Scalar("bool".into())),
                syn::Lit::Int(n) if !n.suffix().is_empty() => {
                    Some(ExactTy::Scalar(n.suffix().into()))
                }
                _ => None,
            },
            syn::Expr::Reference(r) => Some(ExactTy::Reference(
                Box::new(self.infer(&r.expr)?),
                r.mutability.is_some(),
            )),
            syn::Expr::Paren(p) => self.infer(&p.expr),
            syn::Expr::Field(p) => {
                let mut base = self.infer(&p.base)?;
                while let ExactTy::Reference(t, _) = base {
                    base = *t;
                }
                fields(self.source, &base, None)
                    .ok()?
                    .remove(&tokens(&p.member))
            }
            syn::Expr::Call(c) => {
                let syn::Expr::Path(p) = &*c.func else {
                    return None;
                };
                let target = self.target(p)?;
                if let Some((family, variant)) = resolve::standard_variant(&target) {
                    let args = arguments(self.source, &self.function.module, &p.path).ok()?;
                    if !args.is_empty() {
                        Some(ExactTy::Nominal(family.into(), args))
                    } else if variant == "Some" && c.args.len() == 1 {
                        Some(ExactTy::Nominal(
                            family.into(),
                            vec![self.infer(&c.args[0])?],
                        ))
                    } else {
                        None
                    }
                } else {
                    exact_output(self.source, resolve::function(self.source, &target).ok()?).ok()
                }
            }
            _ => None,
        }
    }

    fn target(&self, path: &syn::ExprPath) -> Option<Id> {
        if path.qself.is_some()
            || path
                .path
                .get_ident()
                .is_some_and(|n| self.locals.contains_key(&n.to_string()))
        {
            return None;
        }
        resolve::path(self.source, &self.function.module, &path.path).ok()
    }

    fn descend(&mut self, expr: &syn::Expr, expected: Option<ExactTy>) {
        let old = std::mem::replace(&mut self.expected, expected);
        self.visit_expr(expr);
        self.expected = old;
    }

    fn constructor(
        &mut self,
        path: &syn::ExprPath,
        expr: &syn::Expr,
        args: Vec<&syn::Expr>,
    ) -> bool {
        let Some(target) = self.target(path) else {
            return false;
        };
        let Some((family, variant)) = resolve::standard_variant(&target) else {
            return false;
        };
        let explicit =
            arguments(self.source, &self.function.module, &path.path).unwrap_or_default();
        let alias_instance = if path.path.segments.len() > 1
            && path.path.segments.last().unwrap().arguments.is_empty()
        {
            let mut base = path.path.clone();
            base.segments.pop();
            base.segments.pop_punct();
            exact(
                self.source,
                &self.function.module,
                &syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: base,
                }),
            )
            .ok()
        } else {
            None
        };
        let ty = if alias_instance.is_some() {
            alias_instance
        } else if !explicit.is_empty() {
            Some(ExactTy::Nominal(family.into(), explicit))
        } else {
            self.expected.clone().or_else(|| self.infer(expr))
        };
        let Some(ty @ ExactTy::Nominal(_, _)) = ty else {
            return false;
        };
        if !matches!(&ty,ExactTy::Nominal(id,ts) if id==family && ts.len()==if family=="std::option::Option"{1}else{2})
        {
            return false;
        }
        let Ok(schema) = fields(self.source, &ty, Some(variant)) else {
            return false;
        };
        if args.len() != schema.len() {
            return false;
        }
        self.claims.push(TypeClaim {
            site: Site::new(&self.function.site.file, expr.span()),
            exact: ty,
            target,
        });
        for (i, arg) in args.into_iter().enumerate() {
            self.descend(arg, schema.get(&i.to_string()).cloned());
        }
        true
    }
}

impl<'ast> Visit<'ast> for Context<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let old = self.locals.clone();
        // Block items shadow throughout their block. Do not claim a constructor
        // in an unmodeled local item/import scope.
        if block.stmts.iter().any(|s| matches!(s, syn::Stmt::Item(_))) {
            return;
        }
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Expr(expr, semi) => self.descend(
                    expr,
                    if semi.is_none() {
                        self.expected.clone()
                    } else {
                        None
                    },
                ),
                syn::Stmt::Local(local) => {
                    if let Some(init) = &local.init {
                        let inferred = self.infer(&init.expr);
                        // An annotation is a checked provenance source only if
                        // the independently inferred initializer agrees.
                        let declared = match &local.pat {
                            syn::Pat::Type(p) => exact(self.source, &self.function.module, &p.ty)
                                .ok()
                                .filter(|t| Some(t) == inferred.as_ref()),
                            _ => None,
                        };
                        self.descend(&init.expr, declared.clone());
                        if let Some((_, failure)) = &init.diverge {
                            self.descend(failure, None);
                        }
                        self.visit_pat(&local.pat);
                        let pat = match &local.pat {
                            syn::Pat::Type(p) => &*p.pat,
                            p => p,
                        };
                        if let syn::Pat::Ident(p) = pat {
                            self.locals
                                .insert(p.ident.to_string(), declared.or(inferred));
                        }
                    }
                }
                syn::Stmt::Macro(m) => {
                    let old = self.expected.clone();
                    if m.semi_token.is_some() {
                        self.expected = None;
                    }
                    self.visit_macro(&m.mac);
                    self.expected = old;
                }
                _ => {}
            }
        }
        self.locals = old;
    }

    fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
        let path: syn::Path = syn::parse_str(&pat.ident.to_string()).unwrap();
        if resolve::path(self.source, &self.function.module, &path)
            .ok()
            .and_then(|id| resolve::standard_variant(&id))
            .is_none()
        {
            self.locals.insert(pat.ident.to_string(), None);
        }
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let Some(arm) = typed_resume_macro::checked(self.source, self.function, mac) else {
            return;
        };
        self.descend(&arm.result, None);
        let payload = self.infer(&arm.result).and_then(|ty| match ty {
            ExactTy::Nominal(id, args) if id == "std::result::Result" && args.len() == 2 => {
                Some(args[0].clone())
            }
            _ => None,
        });
        let enclosing = self.locals.clone();
        // Unsupported patterns still shadow their enclosing names as unknown.
        self.visit_pat(&arm.value);
        if let syn::Pat::Ident(p) = &arm.value {
            if p.subpat.is_none() {
                self.locals.insert(
                    p.ident.to_string(),
                    payload.map(|ty| {
                        if p.by_ref.is_some() {
                            ExactTy::Reference(Box::new(ty), p.mutability.is_some())
                        } else {
                            ty
                        }
                    }),
                );
            }
        }
        self.descend(&arm.success, self.expected.clone());
        self.locals = enclosing;
        self.descend(&arm.session, None);
        self.descend(&arm.frame, None);
    }

    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        match expr {
            syn::Expr::Call(c) => {
                if let syn::Expr::Path(p) = &*c.func {
                    if self.constructor(p, expr, c.args.iter().collect()) {
                        return;
                    }
                }
                for arg in &c.args {
                    self.descend(arg, None);
                }
            }
            syn::Expr::Path(p) => {
                self.constructor(p, expr, Vec::new());
            }
            syn::Expr::Return(r) => {
                if let Some(e) = &r.expr {
                    self.descend(e, self.output.clone());
                }
            }
            syn::Expr::If(i) => {
                self.descend(&i.cond, None);
                self.visit_block(&i.then_branch);
                if let Some((_, other)) = &i.else_branch {
                    self.descend(other, self.expected.clone());
                }
            }
            syn::Expr::Match(m) => {
                self.descend(&m.expr, None);
                for arm in &m.arms {
                    let old = self.locals.clone();
                    self.visit_pat(&arm.pat);
                    self.descend(&arm.body, self.expected.clone());
                    self.locals = old;
                }
            }
            syn::Expr::Block(b) => self.visit_block(&b.block),
            syn::Expr::Paren(p) => self.descend(&p.expr, self.expected.clone()),
            syn::Expr::Macro(m) => self.visit_macro(&m.mac),
            syn::Expr::Closure(_) => {}
            _ => {
                let old = self.expected.take();
                visit::visit_expr(self, expr);
                self.expected = old;
            }
        }
    }
}
