//! Narrow existential ownership proof for private generic erasure adapters.
//! Unsupported obligations reject; declaration bounds never replace impl bounds.

use std::collections::BTreeSet;

use super::super::model::{Discovery, DropImpl, Id, Source};
use super::super::resolve;
use super::ownership::{EXPRESSIONS, ownership_path};

fn resolve_one(source: &Source, scope: &str, path: &syn::Path) -> Option<Id> {
    let ids = resolve::type_path(source, scope, path);
    (ids.len() == 1).then(|| ids.into_iter().next().unwrap())
}

fn public(source: &Source, id: &str) -> Option<bool> {
    let (scope, name) = id.rsplit_once("::")?;
    let declarations: Vec<_> = source
        .scopes
        .get(scope)?
        .bindings
        .get(name)?
        .iter()
        .filter(|binding| binding.priority == 0 && binding.target == id)
        .collect();
    (declarations.len() == 1).then(|| declarations[0].boundary.is_none())
}

fn parameter_type(ty: &syn::Type, name: &str) -> bool {
    matches!(ty, syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident(name))
}

// Exactly one ordinary bound, inline or in a where clause. Associated-type
// constraints, additional marker bounds and higher-ranked predicates need a
// different obligation proof and cannot borrow this rule's interim witness.
fn bound(source: &Source, scope: &str, generics: &syn::Generics) -> Option<(String, Id)> {
    if generics.params.len() != 1 {
        return None;
    }
    let parameter = generics.type_params().next()?;
    if parameter.default.is_some() {
        return None;
    }
    let name = parameter.ident.to_string();
    let mut bounds: Vec<_> = parameter.bounds.iter().collect();
    if let Some(clause) = &generics.where_clause {
        for predicate in &clause.predicates {
            let syn::WherePredicate::Type(predicate) = predicate else {
                return None;
            };
            if predicate.lifetimes.is_some() || !parameter_type(&predicate.bounded_ty, &name) {
                return None;
            }
            bounds.extend(&predicate.bounds);
        }
    }
    if bounds.len() != 1 {
        return None;
    }
    let syn::TypeParamBound::Trait(bound) = bounds[0] else {
        return None;
    };
    if bound.lifetimes.is_some()
        || !matches!(bound.modifier, syn::TraitBoundModifier::None)
        || bound
            .path
            .segments
            .iter()
            .any(|s| !matches!(s.arguments, syn::PathArguments::None))
    {
        return None;
    }
    Some((name, resolve_one(source, scope, &bound.path)?))
}

fn owned_parameter(source: &Source, scope: &str, ty: &syn::Type, name: &str) -> bool {
    if parameter_type(ty, name) {
        return true;
    }
    let syn::Type::Path(path) = ty else {
        return false;
    };
    if path.qself.is_some() {
        return false;
    }
    let Some(id) = resolve_one(source, scope, &path.path) else {
        return false;
    };
    if !matches!(
        id.as_str(),
        "std::sync::Arc" | "alloc::sync::Arc" | "std::boxed::Box" | "alloc::boxed::Box"
    ) {
        return false;
    }
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    args.args.len() == 1
        && matches!(&args.args[0], syn::GenericArgument::Type(ty) if parameter_type(ty, name))
}

struct Adapter {
    id: Id,
    field: String,
    bound: Id,
    location: String,
}

fn adapter(source: &Source, implementation: &DropImpl) -> Option<Adapter> {
    if !implementation.positive || implementation.conditional {
        return None;
    }
    let (parameter, required) = bound(source, &implementation.scope, &implementation.generics)?;
    let syn::Type::Path(ty) = &implementation.ty else {
        return None;
    };
    if ty.qself.is_some() {
        return None;
    }
    let id = resolve_one(source, &implementation.scope, &ty.path)?;
    let definition = source.definitions.get(&id)?;
    if definition.alias.is_some() || definition.generics.params.len() != 1 {
        return None;
    }
    let declared = definition.generics.type_params().next()?;
    // An unbounded declaration is valid; otherwise its sole constraint must
    // equal the implementation's constraint (using each lexical scope).
    if declared.default.is_some() {
        return None;
    }
    if !declared.bounds.is_empty() || definition.generics.where_clause.is_some() {
        if bound(source, &definition.scope, &definition.generics)?.1 != required {
            return None;
        }
    }
    let syn::PathArguments::AngleBracketed(args) = &ty.path.segments.last()?.arguments else {
        return None;
    };
    if args.args.len() != 1
        || !matches!(&args.args[0], syn::GenericArgument::Type(ty) if parameter_type(ty, &parameter))
    {
        return None;
    }
    let field = definition.fields.iter().find(|field| {
        !field.conditional
            && owned_parameter(
                source,
                &definition.scope,
                &field.ty,
                &declared.ident.to_string(),
            )
    })?;
    Some(Adapter {
        id,
        field: field.path.clone(),
        bound: required,
        location: implementation.location.display(),
    })
}

pub(super) fn expression_witness(discovered: &Discovery, trait_id: &str) -> Option<String> {
    let source = &discovered.source;
    if public(source, trait_id)? {
        return None;
    }
    let implementations: Vec<_> = source
        .drops
        .iter()
        .filter(|implementation| {
            resolve::type_path(source, &implementation.scope, &implementation.trait_path)
                == BTreeSet::from([trait_id.to_string()])
        })
        .collect();
    if implementations.is_empty() {
        return None;
    }
    // Validate EVERY implementor before selecting any existential witness.
    let mut adapters: Vec<_> = implementations
        .into_iter()
        .map(|i| adapter(source, i))
        .collect::<Option<_>>()?;
    adapters.sort_by(|a, b| (&a.id, &a.location).cmp(&(&b.id, &b.location)));
    let mut interim = None;
    for adapter in adapters {
        let hop = format!(
            "{trait_id} => {}.{} (impl at {}) => {}",
            adapter.id, adapter.field, adapter.location, adapter.bound
        );
        let mut any_implementor = false;
        let mut concrete = Vec::new();
        for implementation in &source.drops {
            if resolve_one(source, &implementation.scope, &implementation.trait_path).as_ref()
                != Some(&adapter.bound)
            {
                continue;
            }
            any_implementor = true;
            if !implementation.positive
                || implementation.conditional
                || !implementation.generics.params.is_empty()
                || implementation.generics.where_clause.is_some()
            {
                continue;
            }
            let syn::Type::Path(ty) = &implementation.ty else {
                continue;
            };
            if ty.qself.is_some()
                || ty
                    .path
                    .segments
                    .iter()
                    .any(|s| !matches!(s.arguments, syn::PathArguments::None))
            {
                continue;
            }
            let Some(id) = resolve_one(source, &implementation.scope, &ty.path) else {
                continue;
            };
            if source
                .definitions
                .get(&id)
                .is_some_and(|d| d.alias.is_none() && d.generics.params.is_empty())
            {
                concrete.push((id, &implementation.location));
            }
        }
        concrete.sort();
        for (id, location) in concrete {
            if let Some(path) = ownership_path(discovered, &id, EXPRESSIONS) {
                return Some(format!(
                    "{hop} => {id} (impl at {}); {path}",
                    location.display()
                ));
            }
        }
        if !any_implementor && public(source, &adapter.bound) == Some(true) && interim.is_none() {
            interim = Some(format!(
                "{hop} (public bound; no in-tree instantiation exists yet)"
            ));
        }
    }
    interim
}
