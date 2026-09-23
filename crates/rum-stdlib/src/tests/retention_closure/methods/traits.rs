//! Trait impls are exclusion evidence, never admitted dispatch. Unknown scopes,
//! bounds, generated impls, and opaque dependencies all fail closed.

use super::super::{model::*, resolve, types};
use std::collections::BTreeSet;

pub(super) fn exclude(
    source: &Source,
    caller: &Function,
    name: &str,
    candidate: &ExactTy,
) -> Result<Vec<String>, String> {
    // Conservative superset of all receiver methods in the pinned 1.85.0
    // prelude traits, including their supertraits. See inventory provenance.
    if include_str!("../inventory/prelude-methods.txt")
        .lines()
        .any(|n| n == name)
    {
        return Err(format!(
            "prelude trait candidate {name} cannot be excluded at {}",
            candidate.spelling()
        ));
    }
    let mut traits = BTreeSet::new();
    let mut module = Some(caller.module.as_str());
    while let Some(id) = module {
        let scope = &source.modules[id];
        if !scope.hazards.is_empty() {
            return Err("method trait scope contains an unexpanded item".into());
        }
        for bindings in scope.bindings.values() {
            for b in bindings {
                if b.boundary.as_ref().is_some_and(|p| {
                    caller.module != *p && !caller.module.starts_with(&format!("{p}::"))
                }) {
                    continue;
                }
                if source.traits.contains_key(&b.target) {
                    traits.insert(b.target.clone());
                } else if !source.types.contains_key(&b.target)
                    && !source.functions.contains_key(&b.target)
                    && !source.modules.contains_key(&b.target)
                {
                    // Only import bindings can introduce opaque traits. Local
                    // const/static declarations do not inhabit that namespace.
                    if scope.imports.iter().any(|i| {
                        i.name.is_none()
                            || i.name.as_ref().is_some_and(|n| {
                                scope.bindings.get(n).is_some_and(|bs| bs.contains(b))
                            })
                    }) {
                        return Err(format!("opaque imported trait candidate {}", b.target));
                    }
                }
            }
        }
        for import in scope.imports.iter().filter(|i| i.name.is_none()) {
            let path: syn::Path =
                syn::parse_str(&import.path.join("::")).map_err(|_| "unknown glob")?;
            let target = resolve::path(source, id, &path)?;
            if import.conditional || !source.modules.contains_key(&target) {
                return Err("open or conditional trait glob scope".into());
            }
        }
        module = id.rsplit_once("::").map(|(p, _)| p);
    }
    let mut exclusions = Vec::new();
    for id in traits {
        let defs = &source.traits[&id];
        if defs.len() != 1 || defs[0].conditional {
            return Err(format!("conditional trait {id}"));
        }
        let tr = &defs[0].item;
        types::attributes(&tr.attrs)?;
        for item in &tr.items {
            if let syn::TraitItem::Fn(f) = item {
                types::attributes(&f.attrs)?;
            }
        }
        if !tr.supertraits.is_empty()
            || tr
                .items
                .iter()
                .any(|i| matches!(i, syn::TraitItem::Macro(_)))
        {
            return Err(format!(
                "trait {id} has unmodeled supertraits or generated candidates"
            ));
        }
        let Some(method) = tr.items.iter().find_map(|i| match i {
            syn::TraitItem::Fn(f) if f.sig.ident == name => Some(f),
            _ => None,
        }) else {
            continue;
        };
        let Some(syn::FnArg::Receiver(r)) = method.sig.inputs.first() else {
            continue;
        };
        if r.colon_token.is_some() {
            return Err(format!("trait {id} has arbitrary self type"));
        }
        let mut seen = 0;
        for (module, item, _, conditional) in &source.trait_impls {
            let Some((_, path, _)) = &item.trait_ else {
                unreachable!()
            };
            let target = resolve::path(source, module, path);
            if target.as_ref().is_ok_and(|t| *t != id) {
                continue;
            }
            if target.is_err() || !item.generics.params.is_empty() {
                return Err(format!(
                    "unknown or blanket trait impl may supply {id}::{name}"
                ));
            }
            let owner = types::exact(source, module, &item.self_ty)?;
            let receiver = if r.reference.is_some() {
                ExactTy::Reference(Box::new(owner), r.mutability.is_some())
            } else {
                owner
            };
            seen += 1;
            if receiver == *candidate {
                return Err(format!(
                    "earlier trait candidate {id}::{name} at {} (conditional={conditional})",
                    candidate.spelling()
                ));
            }
        }
        // Item-generating macros elsewhere could add a blanket impl. Never
        // infer that an indexed trait with zero matching impls is sealed.
        if source.modules.values().any(|m| !m.hazards.is_empty()) {
            return Err(format!("generated impls prevent exclusion of {id}::{name}"));
        }
        exclusions.push(format!(
            "{id}::{name}: {seen} indexed impl receivers differ from {}",
            candidate.spelling()
        ));
    }
    if exclusions.is_empty() {
        exclusions.push(format!(
            "closed scope has no trait named method {name} at {}",
            candidate.spelling()
        ));
    }
    // Keep the full impl index (including reference and blanket self types) in
    // Source; none is installed as an effects body or trusted contract.
    Ok(exclusions)
}
