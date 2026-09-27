//! Exact borrowed option reads. No general patterns, coercions or method search.
pub(super) mod contracts;
mod oracle;
mod tests;

use super::*;

pub(super) const SHARED: &str = "rum_core::expr::SharedExpr";
pub(super) const HELPER: &str = "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name";

pub(super) fn borrowed_call<'p>(
    source: &Source,
    f: &Function,
    pat: &'p syn::Pat,
    value: Option<&ExactTy>,
) -> Result<Vec<(&'p syn::Pat, ExactTy)>, String> {
    let syn::Pat::Struct(p) = pat else {
        return Err("let-else requires the exact borrowed Expr::Call pattern".into());
    };
    if value != Some(&protocol::borrow(protocol::nominal(protocol::EXPR), false))
        || p.qself.is_some()
        || p.rest.is_some()
        || !p.attrs.is_empty()
        || resolve::path(source, &f.module, &p.path).ok().as_deref()
            != Some("rum_core::expr::Expr::Call")
    {
        return Err(
            "pattern requires exactly &Expr and Expr::Call, without extra reference layers".into(),
        );
    }
    let mut schema = types::fields(source, &protocol::nominal(protocol::EXPR), Some("Call"))?;
    if schema
        != BTreeMap::from([
            ("head".into(), protocol::nominal(SHARED)),
            ("args".into(), protocol::nominal(protocol::ARGS)),
        ])
    {
        return Err("borrowed Expr::Call field schema changed".into());
    }
    let mut bindings = vec![];
    let mut names = BTreeSet::new();
    for field in &p.fields {
        let syn::Pat::Ident(binding) = &*field.pat else {
            return Err("borrowed field must be a plain binding; no partial move".into());
        };
        if !field.attrs.is_empty()
            || !binding.attrs.is_empty()
            || binding.by_ref.is_some()
            || binding.mutability.is_some()
            || binding.subpat.is_some()
            || !names.insert(binding.ident.to_string())
        {
            return Err("borrowed field binding changes reference or ownership mode".into());
        }
        let ty = schema
            .remove(&tokens(&field.member))
            .ok_or("wrong borrowed field")?;
        bindings.push((&*field.pat, protocol::borrow(ty, false)));
    }
    if !schema.is_empty() {
        return Err("borrowed pattern must bind both fields".into());
    }
    Ok(bindings)
}

impl Lint<'_> {
    pub(super) fn borrowed_let_else(
        &mut self,
        local: &syn::Local,
        value: Option<ExactTy>,
        otherwise: &syn::Expr,
    ) {
        let entry = self.locals.clone();
        let entry_diverges = self.diverges;
        self.diverges = false;
        self.expr(otherwise, None);
        if !self.diverges {
            self.reject(
                otherwise.span(),
                Kind::RET002,
                None,
                "let-else must diverge on every else path",
            );
        }
        self.locals = entry;
        self.diverges = entry_diverges;
        match borrowed_call(self.source(), self.f, &local.pat, value.as_ref()) {
            Ok(bindings) => {
                for (pat, ty) in bindings {
                    self.bind(pat, Some(ty));
                }
            }
            Err(reason) => {
                self.reject(local.pat.span(), Kind::RET002, None, reason);
                self.bind(&local.pat, None);
            }
        }
    }

    pub(super) fn shared_expr_coercion(
        &mut self,
        arg: &syn::Expr,
        value: Option<&ExactTy>,
        parameter: Option<&ExactTy>,
    ) -> bool {
        if value != Some(&protocol::borrow(protocol::nominal(SHARED), false))
            || parameter != Some(&protocol::borrow(protocol::nominal(protocol::EXPR), false))
        {
            return false;
        }
        // This adjustment is a concrete call, with its own mark and oracle
        // claim. Keep the expression's unadjusted &SharedExpr type fact.
        self.call_mark(arg.span(), &super::contracts::target("I02"));
        true
    }
}
