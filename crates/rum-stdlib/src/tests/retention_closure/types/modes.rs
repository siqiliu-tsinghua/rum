//! Registered sealed mode domains are finite exceptions, not general generics.
//! Lifetimes are opaque to retention: rustc enforces the invariant brand and
//! borrowing; lifetime erasure here neither creates ownership nor a receipt.

use super::super::{model::*, resolve};

pub(in super::super) const MODES: [&str; 2] = [
    "rum_runtime::runtime::accounting_mode::counter::Unarmed",
    "rum_runtime::runtime::accounting_mode::counter::AccountingShadow",
];

pub(in super::super) struct ModeDomain {
    pub trait_path: &'static str,
    pub implementors: &'static [&'static str],
    zero_sized: bool,
}

pub(in super::super) const MODE_DOMAINS: &[ModeDomain] = &[
    ModeDomain {
        trait_path: "rum_runtime::runtime::accounting_mode::lease::Mode",
        implementors: &MODES,
        // The existing AccountingShadow carries its Arc<Counter>. Its closed
        // exception is preserved; the new profiling implementors are ZSTs.
        zero_sized: false,
    },
    ModeDomain {
        trait_path: "rum_runtime::runtime::suspension::core::profile_mode::Mode",
        implementors: &[
            "rum_runtime::runtime::suspension::core::profile_mode::Ordinary",
            "rum_runtime::runtime::suspension::core::profile_mode::Profiled",
        ],
        zero_sized: true,
    },
];

fn bound(source: &Source, module: &str, p: &syn::TypeParam) -> Result<&'static ModeDomain, String> {
    if !p.attrs.is_empty() || p.default.is_some() || p.bounds.len() != 1 {
        return Err("finite mode requires exactly one registered trait bound".into());
    }
    let syn::TypeParamBound::Trait(b) = &p.bounds[0] else {
        return Err("mode bound must be a registered sealed trait".into());
    };
    if b.lifetimes.is_some()
        || !matches!(b.modifier, syn::TraitBoundModifier::None)
        || b.path
            .segments
            .iter()
            .any(|s| !matches!(s.arguments, syn::PathArguments::None))
    {
        return Err("open or qualified mode bound".into());
    }
    let path = resolve::path(source, module, &b.path)?;
    MODE_DOMAINS
        .iter()
        .find(|domain| domain.trait_path == path)
        .ok_or_else(|| "trait is not a registered finite mode".into())
}

pub(in super::super) fn mode_argument(
    source: &Source,
    module: &str,
    parameter: &syn::TypeParam,
    argument: &ExactTy,
) -> Result<(), String> {
    let domain = bound(source, module, parameter)?;
    domain.check(source)?;
    if !domain.contains(argument) {
        return Err("argument is outside the registered mode domain".into());
    }
    Ok(())
}

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
            syn::GenericParam::Type(p) => {
                bound(source, module, p)?;
                if mode.replace(p.ident.to_string()).is_some() {
                    return Err("only a single registered mode parameter is finite".into());
                }
            }
            _ => return Err("open generic outside a registered mode domain".into()),
        }
    }
    Ok(mode)
}

pub(in super::super) fn is_mode(t: &ExactTy) -> bool {
    // Accounting protocol callers retain their original, narrower admission.
    MODE_DOMAINS[0].contains(t)
}

pub(in super::super) fn with_mode(
    source: &Source,
    f: &Function,
    mode: Option<&ExactTy>,
) -> Result<Function, String> {
    let mut f = f.clone();
    let signature = f.sig();
    let impl_generics = f.impl_generics();
    for g in [&signature.generics, &impl_generics] {
        if let Some(name) = mode_parameter(source, &f.module, g)? {
            let ty = mode.ok_or("finite mode is not instantiated")?;
            let parameter = g
                .type_params()
                .find(|p| p.ident == name)
                .ok_or("missing finite mode parameter")?;
            mode_argument(source, &f.module, parameter, ty)?;
            f.substitutions.insert(name, ty.clone());
        }
    }
    f.sig = without_generics(&f.sig);
    f.impl_generics = String::new();
    Ok(f)
}

/// Check the finite domain from impl headers only. A third, generic or
/// conditional implementation invalidates enumeration rather than vanishing.
pub(in super::super) fn domain(source: &Source) -> Result<(), String> {
    MODE_DOMAINS[0].check(source)
}

impl ModeDomain {
    fn contains(&self, t: &ExactTy) -> bool {
        matches!(t, ExactTy::Nominal(id, args) if args.is_empty() && self.implementors.contains(&id.as_str()))
    }

    pub(in super::super) fn check(&self, source: &Source) -> Result<(), String> {
        let mut modes = std::collections::BTreeSet::new();
        for (module, item, _, conditional) in &source.trait_impls {
            let item = ast::<syn::ItemImpl>(item);
            let Some((_, path, _)) = &item.trait_ else {
                continue;
            };
            if resolve::path(source, module, path).ok().as_deref() != Some(self.trait_path) {
                continue;
            }
            if *conditional
                || !item.generics.params.is_empty()
                || item.generics.where_clause.is_some()
                || item
                    .trait_
                    .as_ref()
                    .is_some_and(|(negative, _, _)| negative.is_some())
            {
                return Err("open or conditional Mode implementation".into());
            }
            let owner = super::exact(source, module, &item.self_ty)?;
            if !self.contains(&owner) || !modes.insert(owner) {
                return Err("Mode implementation domain changed".into());
            }
        }
        if modes
            != self
                .implementors
                .iter()
                .map(|id| ExactTy::Nominal((*id).into(), vec![]))
                .collect()
        {
            return Err("both sealed Mode implementations are required".into());
        }
        if self.zero_sized {
            for id in self.implementors {
                let Some([definition]) = source.types.get(*id).map(Vec::as_slice) else {
                    return Err("mode implementor is not an exact nominal definition".into());
                };
                if definition.conditional
                    || !definition.generics().params.is_empty()
                    || definition.generics().where_clause.is_some()
                    || definition.alias().is_some()
                    || !definition.fields.is_empty()
                    || !definition.variants.is_empty()
                {
                    return Err(
                        "registered mode implementor is not an unconditional zero-sized type"
                            .into(),
                    );
                }
            }
        }
        Ok(())
    }
}

mod projections;
pub(super) use projections::normalize_projections;
