//! Local expression typing and conservative syntax admissions.

use super::*;

impl Lint<'_> {
    pub(super) fn expr(&mut self, expr: &syn::Expr, expected: Option<&ExactTy>) -> Option<ExactTy> {
        let ty = match expr {
            syn::Expr::Lit(lit) => match &lit.lit {
                syn::Lit::Bool(_) => Some(ExactTy::Scalar("bool".into())),
                syn::Lit::Int(i) if matches!(i.suffix(), "usize" | "u8" | "i32") => {
                    Some(ExactTy::Scalar(i.suffix().into()))
                }
                syn::Lit::Int(i) if i.suffix().is_empty() => expected.filter(|t| matches!(t, ExactTy::Scalar(id) if ["usize", "i64", "u8", "i32"].contains(&id.as_str()))).cloned(),
                syn::Lit::Str(_) => Some(protocol::borrow(protocol::scalar("str"), false)),
                _ => None,
            },
            syn::Expr::Path(p) if p.qself.is_none() => p
                .path
                .get_ident()
                .and_then(|n| self.locals.get(&n.to_string()).cloned())
                .flatten()
                .or_else(|| {
                    self.constructors
                        .get(&Site::new(&self.f.site.file, p.span()))
                        .cloned()
                }),
            syn::Expr::Call(c) => self.call(c, expected),
            syn::Expr::MethodCall(c) => self.method(c),
            syn::Expr::Try(t) => self.try_expr(t),
            syn::Expr::Binary(b) => self.binary(b),
            syn::Expr::Index(i) => self.index_expr(i),
            syn::Expr::ForLoop(f) => self.for_expr(f),
            syn::Expr::Match(m) => self.match_expr(m, expected),
            syn::Expr::Assign(a) => {
                let before = self.expr(&a.left, None);
                let after = self.expr(&a.right, before.as_ref());
                if before != after || !matches!(&*a.left, syn::Expr::Path(p) if p.path.get_ident().is_some()) {
                    self.reject(a.span(), Kind::RET002, None, "assignment requires one whole local of identical type");
                }
                Some(ExactTy::Unit)
            }
            syn::Expr::Struct(s) => self.struct_expr(s),
            syn::Expr::Tuple(t) => {
                let types = t
                    .elems
                    .iter()
                    .map(|e| self.expr(e, None))
                    .collect::<Option<Vec<_>>>();
                types.map(|ts| {
                    if ts.is_empty() {
                        ExactTy::Unit
                    } else {
                        ExactTy::Tuple(ts)
                    }
                })
            }
            syn::Expr::Paren(p) => self.expr(&p.expr, expected),
            syn::Expr::Block(b) if b.label.is_none() => self.block(&b.block, expected),
            syn::Expr::Reference(r) => self
                .expr(&r.expr, None)
                .map(|t| ExactTy::Reference(Box::new(t), r.mutability.is_some())),
            syn::Expr::Field(p) => {
                let base = self.expr(&p.base, None);
                base.and_then(|mut t| {
                    while let ExactTy::Reference(inner, _) = t {
                        t = *inner;
                    }
                    types::field(self.source(), &t, &tokens(&p.member)).ok()
                })
            }
            syn::Expr::Return(r) => {
                let output = self.output.clone();
                let ty = r.expr
                    .as_ref()
                    .map_or(Some(ExactTy::Unit), |e| self.expr(e, output.as_ref()));
                if ty != output { self.reject(r.span(), Kind::RET002, None, "return type differs from exact signature"); }
                self.diverges = true;
                ty
            }
            syn::Expr::If(branch) => {
                self.expr(&branch.cond, Some(&ExactTy::Scalar("bool".into())));
                let condition_diverges = self.diverges;
                self.diverges = false;
                let a = self.block(&branch.then_branch, expected);
                let then_diverges = self.diverges;
                self.diverges = false;
                let b = branch
                    .else_branch
                    .as_ref()
                    .map_or(Some(ExactTy::Unit), |(_, e)| self.expr(e, expected));
                let else_diverges = self.diverges;
                self.diverges = condition_diverges || (then_diverges && else_diverges);
                if then_diverges { b } else if else_diverges || a == b { a } else { None }
            }
            syn::Expr::Macro(m) => self.macro_expr(&m.mac, expected),
            _ => {
                self.reject(expr.span(), Kind::RET002, None, syntax::rejection(expr));
                visit::visit_expr(self, expr);
                None
            }
        };
        self.fact(expr, ty)
    }
}

impl<'ast> Visit<'ast> for Lint<'_> {
    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        self.expr(expr, None);
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.block(block, None);
    }
    fn visit_item(&mut self, _: &'ast syn::Item) {}
}
