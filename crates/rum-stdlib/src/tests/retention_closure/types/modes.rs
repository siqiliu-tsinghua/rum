//! The sealed accounting mode is a finite exception, not general generics.
//! Lifetimes are opaque to retention: rustc enforces the invariant brand and
//! borrowing; lifetime erasure here neither creates ownership nor a receipt.

use super::super::{model::*, resolve};

pub(in super::super) const MODES: [&str; 2] = [
    "rum_runtime::runtime::accounting_mode::counter::Unarmed",
    "rum_runtime::runtime::accounting_mode::counter::AccountingShadow",
];

pub(in super::super) fn mode_parameter(
    source: &Source,
    module: &str,
    g: &syn::Generics,
) -> Result<Option<String>, String> {
    if g.where_clause.is_some() {
        return Err("open generic where clause is not a finite mode".into());
    }
    let mut mode = None;
    for param in &g.params {
        match param {
            syn::GenericParam::Lifetime(p) if p.attrs.is_empty() => {}
            syn::GenericParam::Type(p)
                if p.attrs.is_empty() && p.default.is_none() && p.bounds.len() == 1 =>
            {
                let syn::TypeParamBound::Trait(b) = &p.bounds[0] else {
                    return Err("mode bound must be the sealed Mode trait".into());
                };
                if b.lifetimes.is_some()
                    || !matches!(b.modifier, syn::TraitBoundModifier::None)
                    || resolve::path(source, module, &b.path)?
                        != "rum_runtime::runtime::accounting_mode::lease::Mode"
                    || mode.replace(p.ident.to_string()).is_some()
                {
                    return Err("only the single sealed accounting mode is finite".into());
                }
            }
            _ => return Err("open generic outside the two accounting modes".into()),
        }
    }
    Ok(mode)
}

pub(in super::super) fn is_mode(t: &ExactTy) -> bool {
    matches!(t, ExactTy::Nominal(id, args) if args.is_empty() && MODES.contains(&id.as_str()))
}

pub(in super::super) fn with_mode(
    source: &Source,
    f: &Function,
    mode: Option<&ExactTy>,
) -> Result<Function, String> {
    let mut f = f.clone();
    for g in [&f.sig.generics, &f.impl_generics] {
        if let Some(name) = mode_parameter(source, &f.module, g)? {
            domain(source)?;
            let ty = mode
                .filter(|t| is_mode(t))
                .ok_or("finite mode is not instantiated")?;
            f.substitutions.insert(name, ty.clone());
        }
    }
    f.sig.generics = syn::Generics::default();
    f.impl_generics = syn::Generics::default();
    Ok(f)
}

/// Check the finite domain from impl headers only. A third, generic or
/// conditional implementation invalidates enumeration rather than vanishing.
pub(in super::super) fn domain(source: &Source) -> Result<(), String> {
    let mut modes = std::collections::BTreeSet::new();
    for (module, item, _, conditional) in &source.trait_impls {
        let Some((_, path, _)) = &item.trait_ else {
            continue;
        };
        if resolve::path(source, module, path).ok().as_deref()
            != Some("rum_runtime::runtime::accounting_mode::lease::Mode")
        {
            continue;
        }
        if *conditional || !item.generics.params.is_empty() || item.generics.where_clause.is_some()
        {
            return Err("open or conditional Mode implementation".into());
        }
        let owner = super::exact(source, module, &item.self_ty)?;
        if !is_mode(&owner) || !modes.insert(owner) {
            return Err("Mode implementation domain changed".into());
        }
    }
    if modes
        != MODES
            .iter()
            .map(|id| ExactTy::Nominal((*id).into(), vec![]))
            .collect()
    {
        return Err("both sealed Mode implementations are required".into());
    }
    Ok(())
}
