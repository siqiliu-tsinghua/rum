//! Structural storage effects and finite generic ownership summaries.
//!
//! Standard storage signatures are explicit identities. All unmodeled foreign
//! representations are unknown, even when they happen to be familiar types.

use std::collections::{BTreeMap, BTreeSet};

use super::model::*;
use super::resolve;

#[derive(Clone)]
pub(super) enum Shape {
    Leaf,
    Parameter(usize),
    Nominal(Id, Vec<Shape>),
    Storage(String, Vec<Shape>),
    Unknown(UnknownKind, String),
}

type Environment = BTreeMap<String, Shape>;
pub(super) type Shapes = BTreeMap<Id, Vec<(Field, Shape)>>;
pub(super) type Ports = BTreeMap<Id, BTreeMap<usize, String>>;

struct Normalizer<'a> {
    source: &'a Source,
    aliases: BTreeSet<Id>,
}

fn unknown(ty: &syn::Type, why: &str) -> Shape {
    Shape::Unknown(
        UnknownKind::UnknownConstruct,
        format!("{why}: {}", spelling(ty)),
    )
}

impl Normalizer<'_> {
    fn ty(&mut self, ty: &syn::Type, scope: &str, owner: &str, env: &Environment) -> Shape {
        match ty {
            syn::Type::Reference(_) | syn::Type::BareFn(_) | syn::Type::Never(_) => Shape::Leaf,
            syn::Type::Ptr(_) => unknown(ty, "raw pointer ownership protocol"),
            syn::Type::Paren(t) => self.ty(&t.elem, scope, owner, env),
            syn::Type::Group(t) => self.ty(&t.elem, scope, owner, env),
            syn::Type::Slice(t) => {
                Shape::Storage("slice[]".into(), vec![self.ty(&t.elem, scope, owner, env)])
            }
            syn::Type::Array(t) => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(n),
                    ..
                }) = &t.len
                {
                    if n.base10_parse::<u64>().ok() == Some(0) {
                        return Shape::Leaf;
                    }
                    return Shape::Storage(
                        format!("array[{}]", n.base10_digits()),
                        vec![self.ty(&t.elem, scope, owner, env)],
                    );
                }
                Shape::Storage(
                    "array[unresolved length]".into(),
                    vec![
                        unknown(ty, "unresolved array length"),
                        self.ty(&t.elem, scope, owner, env),
                    ],
                )
            }
            syn::Type::Tuple(t) => Shape::Storage(
                "tuple".into(),
                t.elems
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        Shape::Storage(format!(".{i}"), vec![self.ty(t, scope, owner, env)])
                    })
                    .collect(),
            ),
            syn::Type::TraitObject(_) | syn::Type::ImplTrait(_) => Shape::Unknown(
                UnknownKind::UnknownHostPayload,
                format!("opaque owned payload {}", spelling(ty)),
            ),
            syn::Type::Path(t) if t.qself.is_none() => {
                if t.path.segments.len() == 1 {
                    let name = t.path.segments[0].ident.to_string();
                    if name == "Self" {
                        let args = self
                            .source
                            .definitions
                            .get(owner)
                            .into_iter()
                            .flat_map(|def| def.generics.type_params())
                            .filter_map(|param| env.get(&param.ident.to_string()).cloned())
                            .collect();
                        return Shape::Nominal(owner.into(), args);
                    }
                    if let Some(parameter) = env.get(&name) {
                        return parameter.clone();
                    }
                }
                let ids = resolve::type_path(self.source, scope, &t.path);
                if ids.len() != 1 {
                    return unknown(
                        ty,
                        &format!(
                            "unresolved/ambiguous type ({})",
                            ids.into_iter().collect::<Vec<_>>().join(", ")
                        ),
                    );
                }
                let id = ids.into_iter().next().unwrap();
                let mut args = Vec::new();
                if let Some(last) = t.path.segments.last() {
                    match &last.arguments {
                        syn::PathArguments::AngleBracketed(a) => {
                            for arg in &a.args {
                                match arg {
                                    syn::GenericArgument::Type(arg) => {
                                        args.push(self.ty(arg, scope, owner, env))
                                    }
                                    syn::GenericArgument::Lifetime(_)
                                    | syn::GenericArgument::Const(_) => {}
                                    _ => return unknown(ty, "associated generic argument"),
                                }
                            }
                        }
                        syn::PathArguments::None => {}
                        _ => return unknown(ty, "parenthesized type arguments"),
                    }
                }
                if let Some(def) = self.source.definitions.get(&id) {
                    let mut actuals = args;
                    for param in def.generics.type_params().skip(actuals.len()) {
                        if let Some(default) = &param.default {
                            let bindings = def
                                .generics
                                .type_params()
                                .zip(&actuals)
                                .map(|(p, a)| (p.ident.to_string(), a.clone()))
                                .collect();
                            actuals.push(self.ty(default, &def.scope, &id, &bindings));
                        } else {
                            actuals.push(unknown(ty, "missing generic type argument"));
                        }
                    }
                    if let Some(alias) = &def.alias {
                        if !self.aliases.insert(id.clone()) {
                            return unknown(ty, "recursive alias expansion");
                        }
                        let bindings = def
                            .generics
                            .type_params()
                            .zip(&actuals)
                            .map(|(p, a)| (p.ident.to_string(), a.clone()))
                            .collect();
                        let expanded = self.ty(alias, &def.scope, &id, &bindings);
                        self.aliases.remove(&id);
                        Shape::Storage(format!("alias({id})"), vec![expanded])
                    } else {
                        Shape::Nominal(id, actuals)
                    }
                } else {
                    storage(&id, args, ty)
                }
            }
            _ => unknown(ty, "unsupported type construct"),
        }
    }
}

fn storage(id: &str, mut args: Vec<Shape>, ty: &syn::Type) -> Shape {
    // Rust 1.85's stable signatures. Allocator parameters outside these stable
    // signatures need their own effects, especially the allocator in Weak.
    let arity = match id {
        "std::collections::HashMap" => Some(2..=3),
        "std::collections::HashSet" => Some(1..=2),
        "std::collections::BTreeMap" | "std::result::Result" | "core::result::Result" => {
            Some(2..=2)
        }
        "std::sync::Arc"
        | "std::rc::Rc"
        | "alloc::sync::Arc"
        | "alloc::rc::Rc"
        | "std::sync::Weak"
        | "std::rc::Weak"
        | "alloc::sync::Weak"
        | "alloc::rc::Weak"
        | "std::marker::PhantomData"
        | "core::marker::PhantomData"
        | "std::hash::BuildHasherDefault"
        | "std::vec::Vec"
        | "std::boxed::Box"
        | "std::option::Option"
        | "alloc::vec::Vec"
        | "alloc::boxed::Box"
        | "core::option::Option"
        | "std::collections::BTreeSet"
        | "std::collections::VecDeque"
        | "std::collections::LinkedList"
        | "std::collections::BinaryHeap"
        | "std::cell::Cell"
        | "std::cell::RefCell"
        | "std::sync::Mutex"
        | "std::sync::RwLock"
        | "std::sync::OnceLock"
        | "std::pin::Pin"
        | "std::vec::IntoIter" => Some(1..=1),
        _ => None,
    };
    if arity.is_some_and(|arity| !arity.contains(&args.len())) {
        return unknown(ty, "unsupported storage signature");
    }
    if (id == "std::collections::HashMap" && args.len() == 2)
        || (id == "std::collections::HashSet" && args.len() == 1)
    {
        args.push(Shape::Unknown(
            UnknownKind::UnknownConstruct,
            "foreign default hasher representation std::collections::hash_map::RandomState".into(),
        ));
    }
    let label = match id {
        "std::sync::Arc" | "std::rc::Rc" | "alloc::sync::Arc" | "alloc::rc::Rc" => {
            "LastStrongHolder"
        }
        "std::sync::Weak"
        | "std::rc::Weak"
        | "alloc::sync::Weak"
        | "alloc::rc::Weak"
        | "std::marker::PhantomData"
        | "core::marker::PhantomData"
        | "std::hash::BuildHasherDefault" => return Shape::Leaf,
        "std::mem::ManuallyDrop"
        | "std::mem::MaybeUninit"
        | "core::mem::ManuallyDrop"
        | "core::mem::MaybeUninit" => return unknown(ty, "manual storage/destructor protocol"),
        "std::vec::Vec"
        | "std::boxed::Box"
        | "std::option::Option"
        | "std::result::Result"
        | "alloc::vec::Vec"
        | "alloc::boxed::Box"
        | "core::option::Option"
        | "core::result::Result"
        | "std::collections::HashMap"
        | "std::collections::BTreeMap"
        | "std::collections::HashSet"
        | "std::collections::BTreeSet"
        | "std::collections::VecDeque"
        | "std::collections::LinkedList"
        | "std::collections::BinaryHeap"
        | "std::cell::Cell"
        | "std::cell::RefCell"
        | "std::sync::Mutex"
        | "std::sync::RwLock"
        | "std::sync::OnceLock"
        | "std::pin::Pin"
        | "std::vec::IntoIter" => "Own",
        "bool"
        | "char"
        | "str"
        | "u8"
        | "u16"
        | "u32"
        | "u64"
        | "u128"
        | "usize"
        | "i8"
        | "i16"
        | "i32"
        | "i64"
        | "i128"
        | "isize"
        | "f32"
        | "f64"
        | "std::string::String"
        | "alloc::string::String" => return Shape::Leaf,
        _ => return unknown(ty, &format!("foreign representation {id}")),
    };
    Shape::Storage(
        format!("{id}[{label}]"),
        args.into_iter()
            .enumerate()
            .map(|(i, arg)| Shape::Storage(format!("arg{i}"), vec![arg]))
            .collect(),
    )
}

pub(super) fn shapes(source: &Source) -> Shapes {
    let mut normalizer = Normalizer {
        source,
        aliases: BTreeSet::new(),
    };
    source
        .definitions
        .iter()
        .filter(|(_, def)| def.alias.is_none())
        .map(|(id, def)| {
            let env = def
                .generics
                .type_params()
                .enumerate()
                .map(|(i, p)| (p.ident.to_string(), Shape::Parameter(i)))
                .collect();
            (
                id.clone(),
                def.fields
                    .iter()
                    .map(|field| {
                        (
                            field.clone(),
                            if field.conditional {
                                unknown(&field.ty, "unresolved conditional storage")
                            } else {
                                normalizer.ty(&field.ty, &def.scope, id, &env)
                            },
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

// Expand only the actually owned parameter ports. Borrowed/Weak/phantom ports
// therefore cannot acquire edges by being instantiated with a recursive type.
pub(super) fn walk(
    shape: &Shape,
    trail: String,
    ports: &Ports,
    run: &mut impl FnMut(&Shape, &str),
) {
    run(shape, &trail);
    match shape {
        Shape::Storage(label, children) => {
            for child in children {
                walk(child, format!("{trail}/{label}"), ports, run);
            }
        }
        Shape::Nominal(id, args) => {
            if let Some(summary) = ports.get(id) {
                for (index, witness) in summary {
                    if let Some(arg) = args.get(*index) {
                        walk(
                            arg,
                            format!("{trail}/{id}<arg{index}> via {witness}"),
                            ports,
                            run,
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

pub(super) fn ports(shapes: &Shapes) -> Ports {
    let mut ports = Ports::new();
    loop {
        let mut found = Vec::new();
        for (id, fields) in shapes {
            for (field, shape) in fields {
                walk(shape, field.path.clone(), &ports, &mut |shape, trail| {
                    if let Shape::Parameter(index) = shape {
                        found.push((id.clone(), *index, trail.to_string()));
                    }
                });
            }
        }
        let mut changed = false;
        for (id, index, witness) in found {
            if let std::collections::btree_map::Entry::Vacant(entry) =
                ports.entry(id).or_default().entry(index)
            {
                entry.insert(witness);
                changed = true;
            }
        }
        if !changed {
            return ports;
        }
    }
}

pub(super) fn opaque_boundaries(source: &Source, ports: &Ports) -> BTreeSet<Unknown> {
    let mut normalizer = Normalizer {
        source,
        aliases: BTreeSet::new(),
    };
    let mut unknowns = BTreeSet::new();
    for boundary in &source.boundaries {
        let env = boundary
            .parameters
            .iter()
            .map(|(name, declaration)| {
                (
                    name.clone(),
                    Shape::Unknown(
                        UnknownKind::UnknownHostPayload,
                        format!("opaque owned generic parameter {}", declaration.trim()),
                    ),
                )
            })
            .collect();
        let shape = normalizer.ty(&boundary.ty, &boundary.scope, "Self", &env);
        walk(&shape, boundary.name.clone(), ports, &mut |shape, trail| {
            if let Shape::Unknown(UnknownKind::UnknownHostPayload, construct) = shape {
                unknowns.insert(Unknown {
                    kind: UnknownKind::UnknownHostPayload,
                    location: boundary.location.clone(),
                    construct: format!("public input {trail}: {construct}"),
                });
            }
        });
    }
    unknowns
}
