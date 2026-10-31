//! Lexical/import resolution. Never search for a matching final component.

use std::collections::BTreeSet;

use syn::spanned::Spanned;

use super::model::*;
use super::source;

pub(super) fn segments(path: &syn::Path) -> Vec<String> {
    path.segments.iter().map(|s| s.ident.to_string()).collect()
}

pub(super) fn relative(module: &str, path: &[String]) -> Id {
    let mut parts: Vec<_> = module.split("::").map(str::to_owned).collect();
    let mut iter = path.iter().peekable();
    match iter.peek().map(|s| s.as_str()) {
        Some("crate") => {
            parts.truncate(1);
            iter.next();
        }
        Some("self") => {
            iter.next();
        }
        Some("super") => {
            while iter.peek().is_some_and(|s| s.as_str() == "super") {
                parts.pop();
                iter.next();
            }
        }
        _ => parts.clear(),
    }
    parts.extend(iter.cloned());
    parts.join("::")
}

fn visible(binding: &Binding, from: &str) -> bool {
    binding
        .boundary
        .as_ref()
        .is_none_or(|b| from == b || from.starts_with(&format!("{b}::")))
}

fn names(source: &Source, module: &str, name: &str, from: &str) -> BTreeSet<Binding> {
    let Some(m) = source.modules.get(module) else {
        return BTreeSet::new();
    };
    let Some(bindings) = m.bindings.get(name) else {
        return BTreeSet::new();
    };
    let explicit = bindings.iter().any(|b| !b.glob && visible(b, from));
    bindings
        .iter()
        .filter(|b| visible(b, from) && (!explicit || !b.glob))
        .cloned()
        .collect()
}

fn prelude(name: &str) -> Option<&str> {
    match name {
        "Vec" => Some("std::vec::Vec"),
        "String" => Some("std::string::String"),
        "Box" => Some("std::boxed::Box"),
        "Option" => Some("std::option::Option"),
        "Result" => Some("std::result::Result"),
        "Some" => Some("std::option::Option::Some"),
        "None" => Some("std::option::Option::None"),
        "Ok" => Some("std::result::Result::Ok"),
        "Err" => Some("std::result::Result::Err"),
        "Drop" => Some("std::ops::Drop"),
        "bool" | "usize" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64"
        | "isize" | "char" | "str" => Some(name),
        _ => None,
    }
}

fn binding(target: Id) -> Binding {
    Binding {
        target,
        boundary: None,
        glob: false,
        conditional: false,
    }
}

fn external(source: &Source, module: &str, name: &str) -> bool {
    source
        .dependencies
        .get(module.split("::").next().unwrap())
        .is_some_and(|deps| deps.contains(name))
}

fn candidates(source: &Source, module: &str, path: &[String]) -> BTreeSet<Binding> {
    candidates_for(source, module, path, false)
}

// Import propagation may copy canonical workspace identities and external crate
// roots. Opaque external items are terminal: syn cannot establish whether an
// imported item is a module, type or macro. In particular `use quote::quote`
// must never turn the crate root into `quote::quote::quote::...`.
fn candidates_for(
    source: &Source,
    module: &str,
    path: &[String],
    importing: bool,
) -> BTreeSet<Binding> {
    let Some(first) = path.first() else {
        return BTreeSet::new();
    };
    let mut consumed = 1;
    let mut current = match first.as_str() {
        "crate" => BTreeSet::from([binding(module.split("::").next().unwrap().into())]),
        "self" => BTreeSet::from([binding(module.into())]),
        "super" => {
            let mut parts: Vec<_> = module.split("::").collect();
            parts.pop();
            while path.get(consumed).is_some_and(|s| s == "super") {
                parts.pop();
                consumed += 1;
            }
            BTreeSet::from([binding(parts.join("::"))])
        }
        _ => {
            let local = names(source, module, first, module);
            if external(source, module, first)
                && local.iter().all(|b| {
                    !source.modules.contains_key(&b.target) && !source.types.contains_key(&b.target)
                })
            {
                // A same-spelled opaque import may inhabit a different Rust
                // namespace. Keep the crate anchor, but refuse to certify the
                // ambiguous lookup; do not manufacture longer external paths.
                let mut root = binding(first.clone());
                root.conditional = local.iter().any(|b| b.target != *first || b.conditional);
                BTreeSet::from([root])
            } else if !local.is_empty() {
                local
            } else if external(source, module, first) {
                BTreeSet::from([binding(first.clone())])
            } else {
                prelude(first)
                    .map(|p| BTreeSet::from([binding(p.into())]))
                    .unwrap_or_default()
            }
        }
    };
    // Only syntactically anchored external paths (or aliases of the crate
    // root itself) can grow during import propagation. Other external aliases
    // remain unresolved imports, recorded below and rejected at use sites.
    let external_anchor = current
        .iter()
        .all(|b| external(source, module, &b.target) && !source.modules.contains_key(&b.target));
    for part in &path[consumed..] {
        current = current
            .into_iter()
            .flat_map(|b| {
                if source.modules.contains_key(&b.target) {
                    names(source, &b.target, part, module)
                        .into_iter()
                        .map(|mut next| {
                            next.conditional |= b.conditional;
                            next
                        })
                        .collect()
                } else if source.types.contains_key(&b.target) {
                    // Only real inherent bindings are resolved here. An external
                    // spelling remains a rejected dependency edge, never a body.
                    let id = format!("{}::{part}", b.target);
                    if source.functions.contains_key(&id)
                        || source.types[&b.target]
                            .iter()
                            .any(|d| d.variants.contains_key(part))
                    {
                        BTreeSet::from([Binding { target: id, ..b }])
                    } else if let Some(base) = enum_alias(source, &b.target, &mut BTreeSet::new()) {
                        let id = format!("{base}::{part}");
                        if standard_variant(&id).is_some() {
                            BTreeSet::from([Binding { target: id, ..b }])
                        } else {
                            BTreeSet::new()
                        }
                    } else {
                        BTreeSet::new()
                    }
                } else if (!importing || external_anchor || standard_namespace(&b.target))
                    && external(source, module, b.target.split("::").next().unwrap())
                {
                    BTreeSet::from([Binding {
                        target: format!("{}::{part}", b.target),
                        ..b
                    }])
                } else {
                    BTreeSet::new()
                }
            })
            .collect();
    }
    current
}

pub(super) fn path(source: &Source, module: &str, path: &syn::Path) -> Result<Id, String> {
    let names = segments(path);
    let found = candidates(source, module, &names);
    if found.iter().any(|b| b.conditional) {
        return Err("unresolved non-test cfg alternative or opaque import namespace".into());
    }
    let unique: BTreeSet<_> = found.iter().map(|b| b.target.clone()).collect();
    if unique.len() != 1 {
        return Err(format!(
            "path {} has {} exact bindings",
            names.join("::"),
            unique.len()
        ));
    }
    // Unresolved external globs can contribute competing names. No inference
    // from whichever known binding happened to arrive first is permissible.
    if found.iter().any(|b| b.glob) {
        let m = &source.modules[module];
        if m.imports.iter().filter(|i| i.name.is_none()).any(|i| {
            let targets = candidates(source, module, &i.path);
            targets.is_empty()
                || targets.iter().any(|b| {
                    (!source.modules.contains_key(&b.target) && enum_exports(&b.target).is_none())
                        || b.conditional
                })
        }) {
            return Err("open or conditional glob import".into());
        }
    }
    Ok(canonical(&unique.into_iter().next().unwrap()))
}

// Closed standard namespaces are known from their definitions. An opaque
// external alias still cannot grow during import propagation.
fn enum_exports(id: &str) -> Option<&'static [&'static str]> {
    match canonical(id).as_str() {
        "std::option::Option" => Some(&["Some", "None"]),
        "std::result::Result" => Some(&["Ok", "Err"]),
        _ => None,
    }
}

fn standard_namespace(id: &str) -> bool {
    matches!(
        canonical(id).as_str(),
        "std::option" | "std::result" | "std::option::Option" | "std::result::Result"
    )
}

pub(super) fn canonical(id: &str) -> Id {
    let id = id
        .strip_prefix("core::")
        .map(|s| format!("std::{s}"))
        .unwrap_or_else(|| id.into());
    if let Some(rest) = id.strip_prefix("std::prelude::") {
        if let Some((edition, name)) = rest.split_once("::") {
            if matches!(
                edition,
                "v1" | "rust_2015" | "rust_2018" | "rust_2021" | "rust_2024"
            ) {
                if let Some(canonical) = prelude(name) {
                    return canonical.into();
                }
            }
        }
    }
    match id.as_str() {
        "std::option::Some" => "std::option::Option::Some".into(),
        "std::option::None" => "std::option::Option::None".into(),
        "std::result::Ok" => "std::result::Result::Ok".into(),
        "std::result::Err" => "std::result::Result::Err".into(),
        _ => id,
    }
}

pub(super) fn standard_variant(id: &str) -> Option<(&'static str, &'static str)> {
    Some(match canonical(id).as_str() {
        "std::option::Option::Some" => ("std::option::Option", "Some"),
        "std::option::Option::None" => ("std::option::Option", "None"),
        "std::result::Result::Ok" => ("std::result::Result", "Ok"),
        "std::result::Result::Err" => ("std::result::Result", "Err"),
        _ => return None,
    })
}

fn enum_alias(source: &Source, id: &str, active: &mut BTreeSet<Id>) -> Option<Id> {
    if !active.insert(id.into()) {
        return None;
    }
    let result = (|| {
        let [def] = source.types.get(id)?.as_slice() else {
            return None;
        };
        if def.conditional {
            return None;
        }
        let syn::Type::Path(p) = def.alias()? else {
            return None;
        };
        if p.qself.is_some() {
            return None;
        }
        // Alias expansion here is only a namespace lookup, not a type claim.
        let parts = segments(&p.path);
        for end in 1..parts.len() {
            if candidates(source, &def.module, &parts[..end])
                .iter()
                .any(|b| source.types.contains_key(&b.target))
            {
                return None;
            }
        }
        let found = candidates(source, &def.module, &parts);
        let [b] = found.iter().collect::<Vec<_>>()[..] else {
            return None;
        };
        if b.conditional {
            return None;
        }
        let base = canonical(&b.target);
        if matches!(base.as_str(), "std::option::Option" | "std::result::Result") {
            Some(base)
        } else {
            enum_alias(source, &base, active)
        }
    })();
    active.remove(id);
    result
}

pub(super) fn function<'a>(source: &'a Source, id: &str) -> Result<&'a Function, String> {
    match source.functions.get(id).map(Vec::as_slice) {
        Some([f]) if !f.conditional && f.impl_generics().params.is_empty() => Ok(f),
        Some(_) => Err(format!("ambiguous or conditional function {id}")),
        None => Err(format!("not a resolved function definition: {id}")),
    }
}

pub(super) fn imports(source: &mut Source) {
    // Finite lattice: declared workspace IDs, external crate anchors extended
    // by one syntactic import suffix, finite names/visibility boundaries, and
    // two boolean flags. Globs only copy these identities. BTreeSet insertion
    // admits each binding once; no iteration or path-depth budget is involved.
    loop {
        let mut additions = Vec::new();
        for (module, scope) in &source.modules {
            for import in &scope.imports {
                for target in candidates_for(source, module, &import.path, true) {
                    if let Some(name) = &import.name {
                        additions.push((
                            module.clone(),
                            name.clone(),
                            Binding {
                                target: target.target,
                                boundary: import.boundary.clone(),
                                glob: false,
                                conditional: target.conditional || import.conditional,
                            },
                        ));
                    } else if let Some(variants) = enum_exports(&target.target) {
                        for &name in variants {
                            additions.push((
                                module.clone(),
                                name.into(),
                                Binding {
                                    target: format!("{}::{name}", canonical(&target.target)),
                                    boundary: import.boundary.clone(),
                                    glob: true,
                                    conditional: target.conditional || import.conditional,
                                },
                            ));
                        }
                    } else if let Some(export) = source.modules.get(&target.target) {
                        for name in export.bindings.keys() {
                            for bound in names(source, &target.target, name, module) {
                                additions.push((
                                    module.clone(),
                                    name.clone(),
                                    Binding {
                                        target: bound.target,
                                        boundary: import.boundary.clone(),
                                        glob: true,
                                        conditional: target.conditional
                                            || bound.conditional
                                            || import.conditional,
                                    },
                                ));
                            }
                        }
                    }
                }
            }
        }
        let mut changed = false;
        for (module, name, bound) in additions {
            changed |= source
                .modules
                .get_mut(&module)
                .unwrap()
                .bindings
                .entry(name)
                .or_default()
                .insert(bound);
        }
        if !changed {
            break;
        }
    }
    // Preserve an unresolved explicit import as a shadowing error, including
    // when a known glob/prelude spelling would otherwise appear usable.
    let mut unresolved = Vec::new();
    for (module, scope) in &source.modules {
        for (index, import) in scope.imports.iter().enumerate() {
            if let Some(name) = &import.name {
                if candidates_for(source, module, &import.path, true).is_empty() {
                    unresolved.push((
                        module.clone(),
                        name.clone(),
                        Binding {
                            target: format!("{module}::<unresolved-import-{index}>"),
                            boundary: import.boundary.clone(),
                            glob: false,
                            conditional: true,
                        },
                    ));
                }
            }
        }
    }
    for (module, name, bound) in unresolved {
        source
            .modules
            .get_mut(&module)
            .unwrap()
            .bindings
            .entry(name)
            .or_default()
            .insert(bound);
    }
}

pub(super) fn collect_imports(
    source: &mut Source,
    module: &str,
    tree: &syn::UseTree,
    mut prefix: Vec<String>,
    boundary: Option<Id>,
    conditional: bool,
) {
    match tree {
        syn::UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            collect_imports(source, module, &p.tree, prefix, boundary, conditional);
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                collect_imports(
                    source,
                    module,
                    t,
                    prefix.clone(),
                    boundary.clone(),
                    conditional,
                );
            }
        }
        _ => {
            let name = match tree {
                syn::UseTree::Name(n) if n.ident == "self" => prefix.last().cloned(),
                syn::UseTree::Name(n) => {
                    prefix.push(n.ident.to_string());
                    Some(n.ident.to_string())
                }
                syn::UseTree::Rename(n) => {
                    if n.ident != "self" {
                        prefix.push(n.ident.to_string());
                    }
                    Some(n.rename.to_string())
                }
                syn::UseTree::Glob(_) => None,
                _ => unreachable!("all UseTree variants explicitly handled"),
            };
            source
                .modules
                .entry(module.into())
                .or_default()
                .imports
                .push(Import {
                    path: prefix,
                    name,
                    boundary,
                    conditional,
                });
        }
    }
}

pub(super) fn impls(source: &mut Source) {
    for (module, text, site, conditional) in source.impls.clone() {
        let item = ast::<syn::ItemImpl>(&text);
        if item.trait_.is_some() {
            source
                .trait_impls
                .push((module.clone(), text.clone(), site.clone(), conditional));
        }
        let syn::Type::Path(owner) = &*item.self_ty else {
            continue;
        };
        let Ok(owner) = path(source, &module, &owner.path) else {
            continue;
        };
        if let Some((_, tr, _)) = &item.trait_ {
            if path(source, &module, tr).ok().as_deref() == Some("std::ops::Drop") {
                source.drop_types.insert(owner);
            }
            // Trait calls are absent from the admitted surface, including
            // generics, blanket impls and default-provided methods.
            continue;
        }
        if item
            .items
            .iter()
            .any(|i| matches!(i, syn::ImplItem::Macro(_) | syn::ImplItem::Verbatim(_)))
        {
            let canonical = match super::types::exact(source, &module, &item.self_ty) {
                Ok(ExactTy::Nominal(id, _)) => id,
                _ => owner.clone(),
            };
            source.opaque_inherent_owners.insert(canonical);
        }
        for member in &item.items {
            if let syn::ImplItem::Fn(f) = member {
                if source::test_only(&f.attrs) {
                    continue;
                }
                let id = format!("{owner}::{}", f.sig.ident);
                let mut attrs: Vec<_> = item
                    .attrs
                    .iter()
                    .map(|attr| located(attr, &site.file, &source.files))
                    .collect();
                attrs.extend(
                    f.attrs
                        .iter()
                        .map(|attr| located(attr, &site.file, &source.files)),
                );
                source
                    .functions
                    .entry(id.clone())
                    .or_default()
                    .push(Function {
                        id,
                        module: module.clone(),
                        site: Site::new(&site.file, f.span()),
                        sig: located(&f.sig, &site.file, &source.files),
                        body: located(&f.block, &site.file, &source.files),
                        attrs,
                        owner: Some(owner.clone()),
                        boundary: source::boundary(&f.vis, &module),
                        impl_self: Some(located(&item.self_ty, &site.file, &source.files)),
                        impl_generics: located(&item.generics, &site.file, &source.files),
                        substitutions: Default::default(),
                        conditional: conditional || source::conditional(&f.attrs),
                    });
            }
        }
    }
}
