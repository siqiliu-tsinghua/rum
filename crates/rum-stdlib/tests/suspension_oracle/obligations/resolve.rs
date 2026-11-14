//! Conservative lexical resolution. A name match is never a dispatch choice.
use std::collections::{BTreeMap, BTreeSet};

use super::{
    discovery::{Call, Function, Index},
    identity::*,
};

fn anchored(module: &str, path: &str) -> String {
    let mut parts: Vec<_> = module.split("::").collect();
    let mut tail: Vec<_> = path.split("::").filter(|s| !s.is_empty()).collect();
    if tail.first() == Some(&"crate") {
        parts.truncate(1);
        tail.remove(0);
    } else if tail.first() == Some(&"self") {
        tail.remove(0);
    } else if tail.first() == Some(&"super") {
        while tail.first() == Some(&"super") {
            parts.pop();
            tail.remove(0);
        }
    } else if tail.first().is_some_and(|p| {
        *p == parts[0] || matches!(*p, "std" | "core" | "alloc") || p.starts_with("rum_")
    }) {
        return tail.join("::");
    }
    parts.extend(tail);
    parts.join("::")
}

fn visible(vis: Option<&syn::Visibility>, declared: &str, requester: &str) -> bool {
    let within = |scope: &str| requester == scope || requester.starts_with(&format!("{scope}::"));
    match vis {
        Some(syn::Visibility::Public(_)) => true,
        Some(syn::Visibility::Restricted(r)) => within(&anchored(declared, &path_text(&r.path))),
        _ => within(declared),
    }
}

fn absolute(
    index: &Index,
    name: &str,
    requester: &str,
    seen: &mut BTreeSet<(String, String)>,
) -> BTreeSet<String> {
    if !seen.insert((name.into(), requester.into())) {
        return BTreeSet::new();
    }
    let Some((module, leaf)) = name.rsplit_once("::") else {
        return if index.modules.contains(name) {
            BTreeSet::from([name.into()])
        } else {
            BTreeSet::new()
        };
    };
    if let Some(names) = index.symbols.get(name) {
        if visible(index.visibility.get(name), module, requester) {
            return names.clone();
        }
    }
    let mut result = BTreeSet::new();
    for (scope, alias, target) in &index.uses {
        if scope != module || (alias != leaf && alias != "*") {
            continue;
        }
        let vis = index
            .import_visibility
            .get(&(scope.clone(), format!("{alias}:{target}")));
        if !visible(vis, module, requester) {
            continue;
        }
        let target = anchored(module, target);
        if alias == "*" {
            let sources = absolute(index, &target, module, &mut seen.clone());
            for source in sources {
                result.extend(absolute(
                    index,
                    &format!("{source}::{leaf}"),
                    module,
                    &mut seen.clone(),
                ));
            }
        } else {
            result.extend(absolute(index, &target, module, &mut seen.clone()));
        }
    }
    // A module may itself be an imported namespace. Never append a suffix to
    // a guessed target: recursively require its declaration and visibility.
    for source in absolute(index, module, requester, &mut seen.clone()) {
        if source != module {
            result.extend(absolute(
                index,
                &format!("{source}::{leaf}"),
                requester,
                &mut seen.clone(),
            ));
        }
    }
    result
}

fn paths(index: &Index, module: &str, path: &str) -> BTreeSet<String> {
    let key = (module.to_owned(), path.to_owned());
    if let Some(names) = index.path_cache.borrow().get(&key) {
        return names.clone();
    }
    let result = absolute(index, &anchored(module, path), module, &mut BTreeSet::new());
    index.path_cache.borrow_mut().insert(key, result.clone());
    result
}

fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|p| p.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn type_arguments(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Reference(r) => type_arguments(&r.elem),
        syn::Type::Paren(p) => type_arguments(&p.elem),
        syn::Type::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| normalized(&s.arguments))
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn same_arguments(owner: &str, receiver: &syn::Type) -> bool {
    let Ok(owner) = syn::parse_str::<syn::Type>(owner) else {
        return false;
    };
    type_arguments(&owner) == type_arguments(receiver)
}

fn qualify_type(index: &Index, module: &str, ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Reference(r) => {
            let mut r = r.clone();
            r.elem = Box::new(qualify_type(index, module, &r.elem));
            syn::Type::Reference(r)
        }
        syn::Type::Path(p) => {
            let names = nominal(index, module, ty);
            if names.len() == 1 {
                let text = format!(
                    "{}{}",
                    names.first().unwrap(),
                    normalized(&p.path.segments.last().unwrap().arguments)
                );
                syn::parse_str(&text).unwrap_or_else(|_| ty.clone())
            } else {
                ty.clone()
            }
        }
        _ => ty.clone(),
    }
}

fn nominal(index: &Index, module: &str, ty: &syn::Type) -> BTreeSet<String> {
    match ty {
        syn::Type::Reference(r) => nominal(index, module, &r.elem),
        syn::Type::Paren(p) => nominal(index, module, &p.elem),
        syn::Type::Path(p) if p.qself.is_none() => {
            let path = path_text(&p.path);
            paths(index, module, &path)
        }
        _ => BTreeSet::new(),
    }
}

fn owner_types(index: &Index, f: &Function) -> BTreeSet<String> {
    syn::parse_str::<syn::Type>(&f.owner)
        .map(|t| nominal(index, &f.module, &t))
        .unwrap_or_default()
}

pub(super) fn prepare(index: &mut Index) {
    index.owners = index
        .functions
        .iter()
        .map(|f| owner_types(index, f))
        .collect();
    for (i, f) in index.functions.iter().enumerate() {
        if f.owner.is_empty() {
            index
                .functions_by_name
                .entry(f.id.qualified.clone())
                .or_default()
                .push(i);
        } else {
            index
                .methods
                .entry(f.sig.ident.to_string())
                .or_default()
                .push(i);
        }
    }
}

fn return_type(f: &Function) -> Option<syn::Type> {
    if let syn::ReturnType::Type(_, ty) = &f.sig.output {
        let mut ty = (**ty).clone();
        // Preserve the declared result type. Unwrapping here would assign a
        // fictitious receiver to result combinators.
        if let syn::Type::Path(p) = &mut ty {
            if p.path.segments.len() == 1 {
                let token = format!("{}::{}", f.module, normalized(&p.path));
                if let Ok(qualified) = syn::parse_str(&token) {
                    ty = qualified;
                }
            }
        }
        Some(ty)
    } else {
        None
    }
}

pub(super) fn infer(
    index: &Index,
    function: usize,
    locals: &BTreeMap<String, syn::Type>,
    expr: &syn::Expr,
) -> Option<syn::Type> {
    let f = &index.functions[function];
    match expr {
        syn::Expr::Path(p) if p.path.segments.len() == 1 => {
            locals.get(&p.path.segments[0].ident.to_string()).cloned()
        }
        syn::Expr::Reference(r) => infer(index, function, locals, &r.expr),
        syn::Expr::Paren(p) => infer(index, function, locals, &p.expr),
        syn::Expr::Struct(s) => syn::parse_str(&normalized(&s.path)).ok(),
        syn::Expr::Field(field) => {
            let ty = infer(index, function, locals, &field.base)?;
            let owners = nominal(index, &f.module, &ty);
            let mut types = Vec::new();
            for owner in owners {
                if let Some((module, ty)) = index.fields.get(&(owner, normalized(&field.member))) {
                    types.push(qualify_type(index, module, ty));
                }
            }
            (types.len() == 1).then(|| types.remove(0))
        }
        syn::Expr::Try(t) => {
            let ty = infer(index, function, locals, &t.expr)?;
            if let syn::Type::Path(p) = ty {
                if let syn::PathArguments::AngleBracketed(a) = &p.path.segments.last()?.arguments {
                    if let Some(syn::GenericArgument::Type(t)) = a.args.first() {
                        return Some(t.clone());
                    }
                }
            }
            None
        }
        syn::Expr::Call(_) | syn::Expr::MethodCall(_) => {
            // Resolution does not infer a callee from its result; recursion is
            // bounded by the receiver expression's syntax depth.
            let ids = candidates(index, function, locals, expr);
            if ids.len() == 1 {
                return_type(&index.functions[*ids.first().unwrap()])
            } else {
                None
            }
        }
        _ => None,
    }
}

fn candidates(
    index: &Index,
    function: usize,
    locals: &BTreeMap<String, syn::Type>,
    expr: &syn::Expr,
) -> BTreeSet<usize> {
    let f = &index.functions[function];
    let mut result = BTreeSet::new();
    match expr {
        syn::Expr::Call(c) => {
            if let syn::Expr::Path(p) = unparen(&c.func) {
                if p.path.segments.len() == 1
                    && locals.contains_key(&p.path.segments[0].ident.to_string())
                {
                    return result;
                }
                let path = path_text(&p.path);
                let names = paths(index, &f.module, &path);
                for name in names {
                    result.extend(
                        index
                            .functions_by_name
                            .get(&name)
                            .into_iter()
                            .flatten()
                            .copied(),
                    );
                }
                if p.path.segments.len() > 1 || p.qself.is_some() {
                    let name = p.path.segments.last().unwrap().ident.to_string();
                    let owners = if let Some(q) = &p.qself {
                        nominal(index, &f.module, &q.ty)
                    } else {
                        let prefix = path.rsplit_once("::").unwrap().0;
                        if prefix == "Self" {
                            owner_types(index, f)
                        } else {
                            paths(index, &f.module, prefix)
                        }
                    };
                    let receiver_type = if let Some(q) = &p.qself {
                        Some((*q.ty).clone())
                    } else if path
                        .rsplit_once("::")
                        .is_some_and(|(prefix, _)| prefix == "Self")
                    {
                        syn::parse_str(&f.owner).ok()
                    } else {
                        let mut path = p.path.clone();
                        path.segments.pop();
                        Some(syn::Type::Path(syn::TypePath { qself: None, path }))
                    };
                    for &i in index.methods.get(&name).into_iter().flatten() {
                        if !index.owners[i].is_disjoint(&owners)
                            && receiver_type
                                .as_ref()
                                .is_some_and(|ty| same_arguments(&index.functions[i].owner, ty))
                        {
                            result.insert(i);
                        }
                    }
                }
            }
        }
        syn::Expr::MethodCall(c) => {
            let Some(ty) = infer(index, function, locals, &c.receiver) else {
                return result;
            };
            let owners = nominal(index, &f.module, &ty);
            for &i in index
                .methods
                .get(&c.method.to_string())
                .into_iter()
                .flatten()
            {
                if !owners.is_disjoint(&index.owners[i])
                    && same_arguments(&index.functions[i].owner, &ty)
                {
                    result.insert(i);
                }
            }
        }
        _ => {}
    }
    result
}

fn unparen(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Paren(p) => unparen(&p.expr),
        syn::Expr::Group(g) => unparen(&g.expr),
        _ => expr,
    }
}

fn effect(f: &Function) -> bool {
    let output = normalized(&f.sig.output);
    output
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|t| {
            matches!(
                t,
                "EvalError"
                    | "FrameAction"
                    | "SchemaAbort"
                    | "SessionRequestError"
                    | "SessionRequestFailure"
                    | "SessionEvalResponse"
                    | "SessionRequest"
                    | "CallbackValue"
                    | "CallbackFailure"
                    | "HostOutcome"
            )
        })
}

fn unresolved_effect(index: &Index, c: &Call, effect_names: &BTreeSet<String>) -> bool {
    let f = &index.functions[c.function];
    let (name, args, indirect) = match &c.expression {
        syn::Expr::Call(e) => {
            let name = if let syn::Expr::Path(p) = &*e.func {
                Some(p.path.segments.last().unwrap().ident.to_string())
            } else {
                None
            };
            (name, &e.args, !matches!(&*e.func, syn::Expr::Path(_)))
        }
        syn::Expr::MethodCall(e) => (Some(e.method.to_string()), &e.args, false),
        _ => return false,
    };
    if matches!(&c.expression, syn::Expr::Call(_))
        && name.as_ref().is_some_and(|n| c.locals.contains_key(n))
    {
        return true;
    }
    if name.as_ref().is_some_and(|n| effect_names.contains(n)) {
        return true;
    }
    if name.as_ref().is_some_and(|n| {
        c.locals.get(n).is_some_and(|ty| {
            let t = normalized(ty);
            t.contains("Fn") || t.contains("fn (") || t.contains("BuiltinFn")
        })
    }) {
        return true;
    }
    indirect
        || args.iter().any(|e| {
            infer(index, c.function, &c.locals, e).is_some_and(|ty| {
                nominal(index, &f.module, &ty).iter().any(|t| {
                    t == "rum_runtime::runtime::model::session::Session" || t == "fixture::Session"
                })
            })
        })
}

pub(super) fn graph(
    index: Index,
    calls: Vec<Call>,
    mut diagnostics: Diagnostics,
    files: usize,
) -> DiscoveredGraph {
    let targets: Vec<_> = calls
        .iter()
        .map(|c| candidates(&index, c.function, &c.locals, &c.expression))
        .collect();
    let mut may: BTreeSet<_> = index
        .functions
        .iter()
        .enumerate()
        .filter(|(_, f)| effect(f))
        .map(|(i, _)| i)
        .collect();
    let effect_names = may
        .iter()
        .map(|&i| index.functions[i].sig.ident.to_string())
        .collect();
    let unknown: Vec<_> = calls
        .iter()
        .map(|c| unresolved_effect(&index, c, &effect_names))
        .collect();
    loop {
        let before = may.len();
        for ((c, ts), unresolved) in calls.iter().zip(&targets).zip(&unknown) {
            if !ts.is_disjoint(&may) || (ts.is_empty() && *unresolved) {
                may.insert(c.function);
            }
        }
        if before == may.len() {
            break;
        }
    }
    let mut sizes = BTreeMap::<(usize, String), usize>::new();
    for c in &calls {
        *sizes.entry((c.function, c.tokens.clone())).or_default() += 1;
    }
    let mut destruction = index.destruction.clone();
    destruction.sort();
    // A conservative dependency fingerprint includes every ADT and Drop body.
    // It may over-invalidate; it cannot bless an unseen destructor change.
    let destructors = digest(destruction.join("\n").as_bytes());
    let mut rows = BTreeMap::new();
    let mut source_rows = BTreeMap::new();
    for ((c, ts), unresolved) in calls.iter().zip(&targets).zip(&unknown) {
        let possible = !ts.is_disjoint(&may) || (ts.is_empty() && *unresolved);
        let f = &index.functions[c.function];
        let context = format!(
            "{}#{}#{}",
            f.id.qualified,
            digest(c.tokens.as_bytes()),
            c.occurrence
        );
        if ts.len() != 1 {
            if !possible {
                continue;
            }
            diagnostics.push(Diagnostic::new(
                Kind::AmbiguousDispatch,
                context,
                format!(
                    "{};{};{}",
                    c.span.display(),
                    c.tokens,
                    ts.iter()
                        .map(|i| index.functions[*i].id.key())
                        .collect::<Vec<_>>()
                        .join(";")
                ),
            ));
            continue;
        }
        let t = &index.functions[*ts.first().unwrap()];
        let edge = EdgeKey {
            producer: t.id.clone(),
            consumer: ConsumerSiteId {
                function: f.id.key(),
                call_digest: digest(c.tokens.as_bytes()),
                occurrence: c.occurrence,
            },
        };
        let pass = if c.pass != "lexical" {
            c.pass.clone()
        } else if t.trait_name != "inherent" && !t.trait_name.is_empty() {
            "typed".into()
        } else if matches!(&c.expression, syn::Expr::Call(e) if !matches!(&*e.func, syn::Expr::Path(_)))
        {
            "supplemental".into()
        } else {
            "lexical".into()
        };
        let row = Row {
            edge,
            policies: Vec::new(),
            aliases: Vec::new(),
            span: c.span.clone(),
            tokens: c.tokens.clone(),
            tail: c.tail.clone(),
            // Caller bindings can add an implicit post-producer Drop without
            // changing a tail token or a type declaration. Conservatively pin
            // the normalized caller body as well as the ADT/Drop dependencies.
            destructors: digest(
                format!("{}\n{}", destructors, normalized(f.body.as_ref().unwrap())).as_bytes(),
            ),
            pass,
            strict: c.strict,
            group_size: sizes[&(c.function, c.tokens.clone())],
            order: digest(normalized(f.body.as_ref().unwrap()).as_bytes()),
            legacy_function: f.legacy.clone(),
        };
        let key = row.edge.key();
        if possible {
            rows.insert(key.clone(), row.clone());
        }
        if source_rows.insert(key.clone(), row).is_some() {
            diagnostics.push(Diagnostic::new(
                Kind::AmbiguousDispatch,
                key,
                "duplicate-qualified-source-key",
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    DiscoveredGraph {
        rows,
        source_rows,
        diagnostics,
        files,
        functions: index.functions.len(),
        calls: calls.len(),
    }
}
