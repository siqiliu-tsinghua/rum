//! A source proof, not a method-name heuristic. Candidate receiver order follows
//! rustc method/probe.rs (pick_all_method/pick_method) and rust-analyzer
//! method_resolution.rs (iterate_method_candidates_with_autoref/by_receiver).
//! Adapted in spirit only; no upstream implementation code is copied.
//! Only built-in reference dereferences are considered. Inherent candidates
//! win within a slot, never across an earlier trait slot.

mod context;
mod traits;

use super::{model::*, types};
use std::collections::BTreeMap;
use syn::spanned::Spanned;

pub(super) use context::claims;

pub(super) fn all(source: &Source) -> BTreeMap<(Site, LeafKey), (Function, Selection)> {
    let mut pending: Vec<_> = source.functions.values().flatten().cloned().collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut result = BTreeMap::new();
    while let Some(f) = pending.pop() {
        if !seen.insert(key(&f)) {
            continue;
        }
        for (site, claim) in claims(source, &f) {
            if let Ok(selection) = claim {
                pending.push(selection.function.clone());
                result.insert((site, key(&f)), (f.clone(), selection));
            }
        }
    }
    result
}

#[derive(Clone, Debug)]
pub(super) struct Receiver {
    pub exact: ExactTy,
    pub provenance: Provenance,
    pub mutable: bool,
    pub movable: bool,
}

#[derive(Clone)]
pub(super) struct Selection {
    pub function: Function,
    pub receiver: Receiver,
    pub derefs: usize,
    pub autoref: Option<bool>,
    pub order: Vec<ExactTy>,
    pub exclusions: Vec<String>,
    pub method_site: Site,
}

pub(super) fn key(f: &Function) -> LeafKey {
    LeafKey {
        id: f.id.clone(),
        instance: f
            .substitutions
            .iter()
            .filter(|(n, t)| {
                n.as_str() != "Self" || !matches!(t, ExactTy::Nominal(_, args) if args.is_empty())
            })
            .map(|(_, t)| t.clone())
            .collect(),
    }
}

// Finite-instance rule: all generic parameters are unbounded scalar atoms from
// ExactTy's fixed primitive vocabulary. No inferred type parameters, nominal
// arguments, const/lifetime arguments, associated types, or growing terms.
// Instances are installed only at reached calls and keyed by all substitutions.
fn bind(
    generics: &syn::Generics,
    args: &[ExactTy],
    into: &mut BTreeMap<String, ExactTy>,
) -> Result<(), String> {
    if generics.where_clause.is_some() || generics.params.len() != args.len() {
        return Err("method instance has unknown bounds or argument arity".into());
    }
    for (param, arg) in generics.params.iter().zip(args) {
        let syn::GenericParam::Type(p) = param else {
            return Err("method instance has lifetime/const parameters".into());
        };
        if !p.attrs.is_empty()
            || !p.bounds.is_empty()
            || p.default.is_some()
            || !matches!(arg, ExactTy::Scalar(id) if id != "str")
        {
            return Err("method instance is not in the finite scalar substitution set".into());
        }
        if into.insert(p.ident.to_string(), arg.clone()).is_some() {
            return Err("method instance shadows an impl parameter".into());
        }
    }
    Ok(())
}

fn instantiate(
    source: &Source,
    caller: &Function,
    f: &Function,
    owner: &ExactTy,
    call: &syn::ExprMethodCall,
) -> Result<Function, String> {
    let ExactTy::Nominal(_, args) = owner else {
        return Err("receiver is not nominal".into());
    };
    let mut result = f.clone();
    let mut substitutions = BTreeMap::new();
    if !f.impl_generics.params.is_empty() {
        // Direct nominal positional impl parameters only; unification is not
        // guessed from a lone candidate or from the crate compiling.
        let syn::Type::Path(p) = f.impl_self.as_ref().ok_or("missing impl Self")? else {
            return Err("arbitrary impl owner".into());
        };
        let syn::PathArguments::AngleBracketed(a) = &p.path.segments.last().unwrap().arguments
        else {
            return Err("non-positional generic impl".into());
        };
        if a.args.len() != f.impl_generics.params.len() || !a.args.iter().zip(&f.impl_generics.params).all(|(a,p)| {
            matches!((a,p), (syn::GenericArgument::Type(syn::Type::Path(t)), syn::GenericParam::Type(p)) if t.qself.is_none() && t.path.is_ident(&p.ident))
        }) { return Err("non-positional generic impl".into()); }
        bind(&f.impl_generics, args, &mut substitutions)?;
    } else if f.impl_generics.where_clause.is_some() {
        return Err("impl bounds are not proved".into());
    }
    let actual = types::substitute(
        source,
        &f.module,
        f.impl_self.as_ref().ok_or("missing impl owner")?,
        &substitutions,
    )?;
    if actual != *owner {
        return Err("canonical impl owner differs from exact receiver".into());
    }
    let arguments = call
        .turbofish
        .as_ref()
        .map(|a| {
            a.args
                .iter()
                .map(|a| {
                    let syn::GenericArgument::Type(t) = a else {
                        return Err("non-type method argument".into());
                    };
                    types::in_function(source, caller, t)
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()?
        .unwrap_or_default();
    bind(&f.sig.generics, &arguments, &mut substitutions)?;
    substitutions.insert("Self".into(), owner.clone());
    result.substitutions = substitutions;
    result.sig.generics = syn::Generics::default();
    result.impl_generics = syn::Generics::default();
    Ok(result)
}

pub(super) fn select(
    source: &Source,
    caller: &Function,
    call: &syn::ExprMethodCall,
    receiver: Receiver,
) -> Result<Selection, String> {
    if !call.attrs.is_empty() {
        return Err("conditional/attributed method call".into());
    }
    let mut owner = &receiver.exact;
    while let ExactTy::Reference(t, _) = owner {
        owner = t;
    }
    let ExactTy::Nominal(id, _) = owner else {
        return Err("method receiver lacks an exact nominal type".into());
    };
    let target = format!("{id}::{}", call.method);
    if source.opaque_inherent_owners.contains(id) {
        return Err("unexpanded inherent impl prevents uniqueness proof".into());
    }
    let Some([f]) = source.functions.get(&target).map(Vec::as_slice) else {
        return Err(
            "no unique inherent definition; user Deref/unsizing/trait dispatch is not searched"
                .into(),
        ); // MUTATION:method-deref
    };
    if f.conditional || f.owner.as_ref() != Some(id) {
        return Err("conditional or noncanonical inherent owner".into());
    }
    if f.boundary
        .as_ref()
        .is_some_and(|b| caller.module != *b && !caller.module.starts_with(&format!("{b}::")))
    {
        return Err("inherent method is not visible".into());
    }
    types::attributes(&f.attrs)?;
    if f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
        || f.sig.abi.is_some()
        || f.sig.variadic.is_some()
    {
        return Err("method signature has unmodeled calling or return effects".into());
    }
    let f = instantiate(source, caller, f, owner, call)?;
    let Some(syn::FnArg::Receiver(r)) = f.sig.inputs.first() else {
        return Err("associated function has no receiver".into());
    };
    if r.colon_token.is_some() || r.reference.as_ref().is_some_and(|(_, l)| l.is_some()) {
        return Err("arbitrary or lifetime-qualified self type".into());
    }
    let wanted = types::exact_parameters(source, &f)?
        .first()
        .ok_or("missing receiver")?
        .1
        .clone();
    // A selection claim includes a closed signature, never a body certificate.
    types::exact_output(source, &f)?;
    let mut step = receiver.exact.clone();
    let mut derefs = 0;
    let mut mutable = receiver.mutable;
    let mut through_shared = false;
    let mut order = Vec::new();
    let mut exclusions = Vec::new();
    loop {
        for autoref in [None, Some(false), Some(true)] {
            let candidate = match autoref {
                None => step.clone(),
                Some(m) => ExactTy::Reference(Box::new(step.clone()), m),
            };
            order.push(candidate.clone());
            if candidate == wanted {
                if matches!(wanted, ExactTy::Reference(_, true)) {
                    let allowed = if autoref.is_some() {
                        mutable
                    } else {
                        matches!(step, ExactTy::Reference(_, true)) && !through_shared
                    };
                    if !allowed {
                        return Err(
                            "selected mutable receiver lacks mutable place provenance".into()
                        );
                    }
                } else if !matches!(wanted, ExactTy::Reference(..))
                    && (derefs != 0 || !receiver.movable)
                {
                    return Err("selected receiver would move through a borrowed place".into());
                }
                exclusions.push("inherent candidate precedes traits in selected slot".into());
                // References are reborrowed, including an exact receiver-slot
                // match; this is not a move of an &mut binding.
                let (derefs, autoref) = if autoref.is_none() {
                    if let ExactTy::Reference(_, m) = step {
                        (derefs + 1, Some(m))
                    } else {
                        (derefs, autoref)
                    }
                } else {
                    (derefs, autoref)
                };
                return Ok(Selection {
                    function: f,
                    receiver,
                    derefs,
                    autoref,
                    order,
                    exclusions,
                    method_site: Site::new(&caller.site.file, call.method.span()),
                });
            }
            exclusions.extend(traits::exclude(
                source,
                caller,
                &call.method.to_string(),
                &candidate,
            )?); // MUTATION:method-traits
        }
        let ExactTy::Reference(inner, m) = step else {
            break;
        };
        through_shared |= !m;
        mutable = m && !through_shared;
        step = *inner;
        derefs += 1;
    }
    Err("inherent receiver requires unsupported adjustment".into())
}

pub(super) fn site(f: &Function, call: &syn::ExprMethodCall) -> Site {
    Site::new(&f.site.file, call.span())
}
