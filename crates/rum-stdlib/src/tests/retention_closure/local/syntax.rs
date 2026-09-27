//! Read rejected macro arguments for diagnostics only. This does not expand or
//! admit a macro, synthesize a call edge, or grant coverage to its arms.

use super::*;
use syn::parse::{Parse, ParseStream, Parser};

pub(super) fn rejection(expr: &syn::Expr) -> &'static str {
    match expr {
        syn::Expr::Try(_) => "? has no admission rule",
        syn::Expr::Binary(_) | syn::Expr::Unary(_) => "operator has no local admission rule",
        syn::Expr::Closure(_) => "closure has no admission rule",
        syn::Expr::Index(_) => "index/Deref boundary has no admission rule",
        syn::Expr::ForLoop(_) => "for/IntoIterator has no admission rule",
        syn::Expr::Range(_) => "range has no admission rule",
        _ => "expression outside the local admitted surface",
    }
}

struct ResumeArguments(Vec<syn::Expr>);
impl Parse for ResumeArguments {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = vec![input.parse()?, {
            input.parse::<syn::Token![,]>()?;
            input.parse()?
        }];
        input.parse::<syn::Token![,]>()?;
        let ok: syn::Ident = input.parse()?;
        if ok != "ok" {
            return Err(input.error("expected ok arm"));
        }
        let binding;
        syn::parenthesized!(binding in input);
        let _: syn::Ident = binding.parse()?;
        input.parse::<syn::Token![=>]>()?;
        args.push(input.parse()?);
        input.parse::<syn::Token![,]>()?;
        let suspend: syn::Ident = input.parse()?;
        if suspend != "suspend" {
            return Err(input.error("expected suspend arm"));
        }
        input.parse::<syn::Token![=>]>()?;
        args.push(input.parse()?);
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
        Ok(Self(args))
    }
}

impl Lint<'_> {
    pub(super) fn macro_expr(
        &mut self,
        mac: &syn::Macro,
        expected: Option<&ExactTy>,
    ) -> Option<ExactTy> {
        self.reject(
            mac.span(),
            Kind::RET003,
            None,
            "macro has no admission rule",
        );
        let name = tokens(&mac.path);
        if name == "crate :: resume_or_suspend" {
            if let Ok(args) = syn::parse2::<ResumeArguments>(mac.tokens.clone()) {
                for arg in args.0 { self.expr(&arg, None); }
            }
        } else if expected.is_some_and(|t| matches!(t, ExactTy::Nominal(id,_) if types::OWNED_CONTAINERS.contains(&id.as_str()))) {
            let args = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
                .parse2(mac.tokens.clone());
            if let Ok(args) = args {
                for arg in args { self.expr(&arg, None); }
                // This is a diagnostic about the required argument type at an
                // already rejected macro, independent of the macro's name.
                // The actual expansion type remains unknown; no fact or mark
                // is minted from contextual typing alone.
                if let Some(call) = &self.argument_hazard {
                    self.reject(mac.span(), Kind::RET001, None, format!(
                        "raw owned container required at rejected macro argument: {} would be live across consuming call at {} (callee not proved inert; expansion type unknown)",
                        expected.unwrap().spelling(), call.display()));
                }
            }
        }
        None
    }
}
