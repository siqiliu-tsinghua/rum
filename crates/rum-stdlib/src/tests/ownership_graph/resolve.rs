//! Resolve lexical names, imports and re-exports to canonical identities.
//! No suffix/name search across definitions is used, including for Drop impls.

use std::collections::{BTreeMap, BTreeSet};

use super::model::*;

fn visible(binding: &Binding, module: &str) -> bool {
    binding
        .boundary
        .as_ref()
        .is_none_or(|boundary| module == boundary || module.starts_with(&format!("{boundary}::")))
}

fn names(source: &Source, scope: &str, name: &str, from: &str) -> BTreeSet<Id> {
    let Some(scope) = source.scopes.get(scope) else {
        return BTreeSet::new();
    };
    if let Some(bindings) = scope.bindings.get(name) {
        let priority = bindings
            .iter()
            .filter(|b| visible(b, from))
            .map(|b| b.priority)
            .min();
        let targets: BTreeSet<_> = bindings
            .iter()
            .filter(|b| Some(b.priority) == priority && visible(b, from))
            .map(|b| b.target.clone())
            .collect();
        if !targets.is_empty() {
            return targets;
        }
    }
    scope
        .lexical_parent
        .as_ref()
        .map(|parent| names(source, parent, name, from))
        .unwrap_or_default()
}

fn external_root(name: &str) -> bool {
    // Dependency identities, not representation exemptions. Unsupported foreign
    // types still become located UnknownConstructs in effects.rs.
    matches!(
        name,
        "std"
            | "core"
            | "alloc"
            | "rum_syntax"
            | "rum_derive"
            | "num_bigint"
            | "num_integer"
            | "num_traits"
            | "serde"
            | "thiserror"
            | "sha1"
            | "sha2"
            | "md5"
            | "crc32fast"
    )
}

fn prelude(name: &str) -> Option<&'static str> {
    Some(match name {
        "Vec" => "std::vec::Vec",
        "Box" => "std::boxed::Box",
        "String" => "std::string::String",
        "Option" => "std::option::Option",
        "Result" => "std::result::Result",
        "Drop" => "std::ops::Drop",
        "bool" => "bool",
        "char" => "char",
        "str" => "str",
        "u8" => "u8",
        "u16" => "u16",
        "u32" => "u32",
        "u64" => "u64",
        "u128" => "u128",
        "usize" => "usize",
        "i8" => "i8",
        "i16" => "i16",
        "i32" => "i32",
        "i64" => "i64",
        "i128" => "i128",
        "isize" => "isize",
        "f32" => "f32",
        "f64" => "f64",
        _ => return None,
    })
}

pub(super) fn path(source: &Source, scope_id: &str, segments: &[String]) -> BTreeSet<Id> {
    let Some(first) = segments.first() else {
        return BTreeSet::new();
    };
    let scope = &source.scopes[scope_id];
    let mut consumed = 1;
    let mut targets = match first.as_str() {
        "crate" => BTreeSet::from([scope.module.split("::").next().unwrap().to_string()]),
        "self" => BTreeSet::from([scope.module.clone()]),
        "super" => {
            let mut module: Vec<_> = scope.module.split("::").collect();
            module.pop();
            while segments.get(consumed).is_some_and(|s| s == "super") {
                module.pop();
                consumed += 1;
            }
            BTreeSet::from([module.join("::")])
        }
        _ => {
            let local = names(source, scope_id, first, &scope.module);
            if !local.is_empty() {
                local
            } else if source.modules.contains(first) || external_root(first) {
                BTreeSet::from([first.clone()])
            } else {
                prelude(first)
                    .map(|p| BTreeSet::from([p.into()]))
                    .unwrap_or_default()
            }
        }
    };
    for segment in &segments[consumed..] {
        targets = targets
            .into_iter()
            .flat_map(|target| {
                if source.modules.contains(&target) {
                    names(source, &target, segment, &scope.module)
                } else if external_root(target.split("::").next().unwrap()) {
                    BTreeSet::from([format!("{target}::{segment}")])
                } else {
                    BTreeSet::new()
                }
            })
            .collect();
    }
    targets
}

pub(super) fn type_path(source: &Source, scope: &str, ty: &syn::Path) -> BTreeSet<Id> {
    path(
        source,
        scope,
        &ty.segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>(),
    )
}

pub(super) fn imports(source: &mut Source) {
    // Monotone fixed point. The finite universe is source bindings and explicit
    // imports; cyclic globs cannot require a depth limit or a wildcard fallback.
    loop {
        let mut additions = Vec::new();
        for (scope_id, scope) in &source.scopes {
            for import in &scope.imports {
                for target in path(source, scope_id, &import.path) {
                    if let Some(name) = &import.name {
                        additions.push((
                            scope_id.clone(),
                            name.clone(),
                            Binding {
                                target,
                                boundary: import.boundary.clone(),
                                priority: 0,
                            },
                        ));
                    } else if let Some(export) = source.scopes.get(&target) {
                        for name in export.bindings.keys() {
                            for target in names(source, &target, name, &scope.module) {
                                additions.push((
                                    scope_id.clone(),
                                    name.clone(),
                                    Binding {
                                        target,
                                        boundary: import.boundary.clone(),
                                        priority: 1,
                                    },
                                ));
                            }
                        }
                    }
                }
            }
        }
        let mut changed = false;
        for (scope, name, binding) in additions {
            changed |= source
                .scopes
                .get_mut(&scope)
                .unwrap()
                .bindings
                .entry(name)
                .or_default()
                .insert(binding);
        }
        if !changed {
            break;
        }
    }
}

pub(super) fn drops(source: &Source) -> BTreeMap<Id, Vec<Location>> {
    let mut result: BTreeMap<Id, Vec<Location>> = BTreeMap::new();
    for item in &source.drops {
        if type_path(source, &item.scope, &item.trait_path)
            != BTreeSet::from(["std::ops::Drop".into()])
        {
            continue;
        }
        if let syn::Type::Path(ty) = &item.ty {
            let ids = type_path(source, &item.scope, &ty.path);
            if ids.len() == 1 {
                result
                    .entry(ids.into_iter().next().unwrap())
                    .or_default()
                    .push(item.location.clone());
            }
        }
    }
    result
}
