//! Associated assignments are resolved only inside a registered finite domain.
use super::*;

/// Resolve only associated assignments of an already checked finite member.
/// No trait search, blanket implementation, associated default, or generic
/// associated type enters this exception.
fn projection(
    source: &Source,
    module: &str,
    p: &syn::TypePath,
    substitutions: &std::collections::BTreeMap<String, ExactTy>,
) -> Option<Result<ExactTy, String>> {
    let (owner, qualifier) = if let Some(q) = &p.qself {
        if q.position + 1 != p.path.segments.len() || q.position == 0 {
            return None;
        }
        let owner = super::super::substitute(source, module, &q.ty, substitutions).ok()?;
        let mut trait_path = p.path.clone();
        trait_path.segments.pop();
        trait_path.segments.pop_punct();
        (owner, Some(trait_path))
    } else {
        if p.path.segments.len() != 2 {
            return None;
        }
        let first = p.path.segments.first()?;
        if !matches!(first.arguments, syn::PathArguments::None) {
            return None;
        }
        (substitutions.get(&first.ident.to_string())?.clone(), None)
    };
    let domain = MODE_DOMAINS.iter().find(|d| d.contains(&owner))?;
    Some((|| {
        domain.check(source)?;
        if let Some(qualifier) = qualifier {
            if resolve::path(source, module, &qualifier)? != domain.trait_path {
                return Err("projection is not from the member's registered trait".into());
            }
        }
        let name = p.path.segments.last().ok_or("missing associated name")?;
        if !matches!(name.arguments, syn::PathArguments::None) {
            return Err("generic associated mode type is not finite".into());
        }
        let Some([declaration]) = source.traits.get(domain.trait_path).map(Vec::as_slice) else {
            return Err("registered trait definition is not unique".into());
        };
        if declaration.conditional {
            return Err("conditional mode trait".into());
        }
        let declared: Vec<_> = declaration
            .item
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Type(t) if t.ident == name.ident => Some(t),
                _ => None,
            })
            .collect();
        let [declared] = declared.as_slice() else {
            return Err("associated name is not unique in the registered trait".into());
        };
        super::super::attributes(&declared.attrs)?;
        if !declared.generics.params.is_empty()
            || declared.generics.where_clause.is_some()
            || declared.default.is_some()
        {
            return Err("generic or defaulted mode associated type".into());
        }
        for (module, item, _, _) in &source.trait_impls {
            if item.trait_.as_ref().is_none_or(|(_, p, _)| {
                resolve::path(source, module, p).ok().as_deref() != Some(domain.trait_path)
            }) || super::super::exact(source, module, &item.self_ty)? != owner
            {
                continue;
            }
            let assignments: Vec<_> = item
                .items
                .iter()
                .filter_map(|item| match item {
                    syn::ImplItem::Type(t) if t.ident == name.ident => Some(t),
                    _ => None,
                })
                .collect();
            let [assignment] = assignments.as_slice() else {
                return Err("associated assignment is missing or ambiguous".into());
            };
            super::super::attributes(&assignment.attrs)?;
            if !assignment.generics.params.is_empty() || assignment.generics.where_clause.is_some()
            {
                return Err("generic mode associated assignment".into());
            }
            return super::super::exact(source, module, &assignment.ty);
        }
        Err("checked mode implementation is missing".into())
    })())
}

pub(in super::super) fn normalize_projections(
    source: &Source,
    module: &str,
    t: &mut syn::Type,
    substitutions: &mut std::collections::BTreeMap<String, ExactTy>,
) -> Result<(), String> {
    if let syn::Type::Path(p) = t {
        if let Some(projected) = projection(source, module, p, substitutions) {
            let projected = projected?;
            let mut name = format!("__RumFiniteProjection{}", substitutions.len());
            while substitutions.contains_key(&name) {
                name.push('_');
            }
            *t = syn::parse_str(&name).map_err(|e| format!("finite projection syntax: {e}"))?;
            substitutions.insert(name, projected);
            return Ok(());
        }
    }
    match t {
        syn::Type::Reference(r) => {
            normalize_projections(source, module, &mut r.elem, substitutions)?
        }
        syn::Type::Slice(s) => normalize_projections(source, module, &mut s.elem, substitutions)?,
        syn::Type::Array(a) => normalize_projections(source, module, &mut a.elem, substitutions)?,
        syn::Type::Paren(p) => normalize_projections(source, module, &mut p.elem, substitutions)?,
        syn::Type::Tuple(tuple) => {
            for t in &mut tuple.elems {
                normalize_projections(source, module, t, substitutions)?;
            }
        }
        syn::Type::Path(p) if p.qself.is_none() => {
            for segment in &mut p.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for arg in &mut args.args {
                        if let syn::GenericArgument::Type(t) = arg {
                            normalize_projections(source, module, t, substitutions)?;
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
