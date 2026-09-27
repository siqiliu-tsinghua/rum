//! Direct evidence for two concrete read contracts, collected at indexing.
//! No call graph: these declarations are checked independently, then erased.
use super::*;

pub(in super::super) fn validate(source: &Source) -> BTreeMap<Id, Result<(), String>> {
    BTreeMap::from([
        (super::super::contracts::target("I02"), deref(source)),
        (HELPER.into(), helper(source)),
    ])
}

fn deref(source: &Source) -> Result<(), String> {
    let candidates = source
        .trait_impls
        .iter()
        .filter(|(module, item, _, _)| {
            types::exact(source, module, &item.self_ty).ok() == Some(protocol::nominal(SHARED))
                && item.trait_.as_ref().is_some_and(|(_, path, _)| {
                    resolve::path(source, module, path).ok().as_deref() == Some("std::ops::Deref")
                })
        })
        .collect::<Vec<_>>();
    let [(_, item, _, conditional)] = candidates.as_slice() else {
        return Err("I02 requires one concrete SharedExpr Deref implementation".into());
    };
    let expected: syn::ItemImpl = syn::parse_quote! {
        impl Deref for SharedExpr {
            type Target = Expr;
            fn deref(&self) -> &Self::Target { &self.0 }
        }
    };
    if *conditional || tokens(item) != tokens(&expected) {
        return Err("I02 concrete Deref implementation changed; re-discharge its contract".into());
    }
    Ok(())
}

fn helper(source: &Source) -> Result<(), String> {
    let f = resolve::function(source, HELPER)?;
    let sig: syn::Signature = syn::parse_quote! {
        fn is_system_symbol_name(expr: &Expr, name: &str) -> bool
    };
    let body: syn::Block = syn::parse_quote! {
        { expr_as_symbol(expr).map(|symbol| symbol.is_system_name(name)).unwrap_or(false) }
    };
    if f.conditional
        || !f.attrs.is_empty()
        || tokens(&f.sig) != tokens(&sig)
        || tokens(&f.body) != tokens(&body)
        || resolve::path(source, &f.module, &syn::parse_quote!(expr_as_symbol))
            .ok()
            .as_deref()
            != Some("rum_runtime::runtime::expr_helpers::expr_as_symbol")
    {
        return Err("B13 concrete helper changed; re-discharge its contract".into());
    }
    Ok(())
}
