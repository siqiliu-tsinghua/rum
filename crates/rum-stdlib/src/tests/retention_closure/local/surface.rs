//! Local desugarings adapted from rustc_ast_lowering::expr and rust-analyzer
//! expr_store::lower (MIT/Apache-2.0): branch/early return, iterator-next loop,
//! and borrowed Index output. No upstream implementation text is copied.
use super::*;

impl Lint<'_> {
    pub(super) fn try_expr(&mut self, t: &syn::ExprTry) -> Option<ExactTy> {
        let input = self.expr(&t.expr, None)?;
        let ExactTy::Nominal(id, ts) = &input else {
            return self.no(t.span(), "? requires concrete Result");
        };
        let Some(ExactTy::Nominal(out, os)) = &self.output else {
            return self.no(t.span(), "? needs Result return type");
        };
        if id != "std::result::Result"
            || out != id
            || ts.len() != 2
            || os.len() != 2
            || ts[1] != os[1]
        {
            return self.no(
                t.span(),
                "? requires identical concrete Result error; From conversion refused",
            );
        }
        // Flow checks the success payload and the early-error cleanup separately.
        Some(ts[0].clone())
    }

    pub(super) fn no(&mut self, span: proc_macro2::Span, reason: &str) -> Option<ExactTy> {
        self.reject(span, Kind::RET002, None, reason);
        None
    }

    pub(super) fn binary(&mut self, b: &syn::ExprBinary) -> Option<ExactTy> {
        let left = self.expr(&b.left, None);
        let right = self.expr(&b.right, left.as_ref());
        let (Some(a), Some(c)) = (left, right) else {
            return None;
        };
        if a != c {
            return self.no(
                b.span(),
                "operator operands must have identical proved types",
            );
        }
        if a == protocol::nominal(protocol::EXPR) && matches!(b.op, syn::BinOp::Eq(_)) {
            self.call_mark(
                b.span(),
                &format!("<{} as std::cmp::PartialEq>::eq", protocol::EXPR),
            );
            return Some(protocol::scalar("bool"));
        }
        if a == protocol::scalar("usize") {
            return match b.op {
                syn::BinOp::Add(_) => Some(a),
                syn::BinOp::Eq(_) => Some(protocol::scalar("bool")),
                _ => self.no(b.span(), "primitive operator outside the probe surface"),
            };
        }
        self.no(b.span(), "overloaded operator has no exact contract")
    }

    pub(super) fn index_expr(&mut self, i: &syn::ExprIndex) -> Option<ExactTy> {
        let base = self.expr(&i.expr, None)?;
        let index = self.expr(&i.index, Some(&protocol::scalar("usize")));
        if index != Some(protocol::scalar("usize")) {
            return self.no(i.span(), "index must be exactly usize");
        }
        match protocol::base(&base) {
            ExactTy::Nominal(id, args) if id == protocol::ARGS && args.is_empty() => {
                self.call_mark(i.span(), &contracts::target("I01"))
            }
            ExactTy::Nominal(id, args)
                if id == "std::vec::Vec" && args == &[protocol::nominal(protocol::EXPR)] => {}
            _ => {
                return self.no(
                    i.span(),
                    "index requires exact SharedArgs or Vec<Expr>; other Deref/Index refused",
                );
            }
        }
        self.call_mark(i.span(), &contracts::target("I04"));
        Some(protocol::nominal(protocol::EXPR))
    }

    pub(super) fn for_expr(&mut self, f: &syn::ExprForLoop) -> Option<ExactTy> {
        let syn::Expr::Range(r) = &*f.expr else {
            return self.no(f.span(), "for requires exact Range<usize>");
        };
        if f.label.is_some() || !matches!(r.limits, syn::RangeLimits::HalfOpen(_)) {
            return self.no(f.span(), "only unlabeled half-open range is admitted");
        }
        let (Some(a), Some(b)) = (&r.start, &r.end) else {
            return self.no(r.span(), "range requires both bounds");
        };
        let usize_ = protocol::scalar("usize");
        if self.expr(a, Some(&usize_)) != Some(usize_.clone())
            || self.expr(b, Some(&usize_)) != Some(usize_.clone())
        {
            return self.no(r.span(), "range bounds must both be usize");
        }
        self.fact(
            &f.expr,
            Some(ExactTy::Nominal(
                "std::ops::Range".into(),
                vec![usize_.clone()],
            )),
        );
        let old = self.locals.clone();
        self.bind(&f.pat, Some(usize_));
        let output = self.block(&f.body, Some(&ExactTy::Unit));
        self.locals = old;
        self.diverges = false; // empty range is a normal exit
        if output != Some(ExactTy::Unit) {
            return self.no(f.span(), "for body must return unit");
        }
        Some(ExactTy::Unit)
    }

    pub(super) fn match_expr(
        &mut self,
        m: &syn::ExprMatch,
        expected: Option<&ExactTy>,
    ) -> Option<ExactTy> {
        let value = self.expr(&m.expr, None)?;
        let ExactTy::Nominal(id, ts) = &value else {
            return self.no(m.span(), "match needs concrete Result");
        };
        if id != "std::result::Result" || ts.len() != 2 {
            return self.no(m.span(), "only Result match is admitted here");
        }
        let old = self.locals.clone();
        let mut seen = BTreeSet::new();
        let mut result = None;
        let mut all_diverge = true;
        for arm in &m.arms {
            self.locals = old.clone();
            self.diverges = false;
            let syn::Pat::TupleStruct(p) = &arm.pat else {
                return self.no(
                    arm.span(),
                    "Result match requires explicit variant patterns",
                );
            };
            let target = resolve::path(self.source(), &self.f.module, &p.path).ok()?;
            let Some((family, variant)) = resolve::standard_variant(&target) else {
                return self.no(p.span(), "pattern is not standard Result variant");
            };
            if family != id
                || arm.guard.is_some()
                || p.elems.len() != 1
                || !seen.insert(variant.to_owned())
            {
                return self.no(p.span(), "guarded/duplicate/inexact Result match");
            }
            self.bind(&p.elems[0], Some(ts[usize::from(variant == "Err")].clone()));
            let ty = self.expr(&arm.body, expected);
            if !self.diverges {
                if result.is_some() && result != ty {
                    self.no(arm.span(), "branch type mismatch");
                }
                result = ty;
                all_diverge = false;
            }
        }
        self.locals = old;
        self.diverges = all_diverge;
        if seen != BTreeSet::from(["Ok".into(), "Err".into()]) {
            return self.no(m.span(), "nonexhaustive Result match");
        }
        result.or_else(|| expected.cloned())
    }

    pub(super) fn struct_expr(&mut self, s: &syn::ExprStruct) -> Option<ExactTy> {
        let target = resolve::path(self.source(), &self.f.module, &s.path).ok()?;
        if target != protocol::PROBE || s.rest.is_some() || s.qself.is_some() {
            return self.no(s.span(), "only exact probe continuation struct is admitted");
        }
        let ty = protocol::nominal(&target);
        let mut fields = types::fields(self.source(), &ty, None).ok()?;
        for f in &s.fields {
            let expected = fields.remove(&tokens(&f.member))?;
            if self.expr(&f.expr, Some(&expected)) != Some(expected) {
                self.no(f.span(), "continuation field type mismatch");
            }
        }
        if !fields.is_empty() {
            return self.no(s.span(), "missing continuation field");
        }
        Some(ty)
    }
}
