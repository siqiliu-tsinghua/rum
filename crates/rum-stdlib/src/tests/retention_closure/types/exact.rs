//! Nominal identities and lazy substitution, independently of ownership.
//! Inspired by rustc TyKind::Adt(def, args), not compiler implementation code.
//! Deliberately limited to concrete type arguments and elided references. No
//! trait solving, const evaluation, lifetime unification, or coercion inference.

use std::collections::{BTreeMap, BTreeSet};

use super::super::model::*;
use super::super::resolve;

pub(in super::super) fn exact(
    source: &Source,
    module: &str,
    syntax: &syn::Type,
) -> Result<ExactTy, String> {
    expand(
        source,
        module,
        syntax,
        &BTreeMap::new(),
        &mut BTreeSet::new(),
    )
}

pub(in super::super) fn substitute(
    source: &Source,
    module: &str,
    syntax: &syn::Type,
    substitutions: &BTreeMap<String, ExactTy>,
) -> Result<ExactTy, String> {
    expand(source, module, syntax, substitutions, &mut BTreeSet::new())
}

pub(in super::super) fn arguments(
    source: &Source,
    module: &str,
    path: &syn::Path,
) -> Result<Vec<ExactTy>, String> {
    let mut result = Vec::new();
    for segment in &path.segments {
        match &segment.arguments {
            syn::PathArguments::None => {}
            syn::PathArguments::AngleBracketed(args) if result.is_empty() => {
                for argument in &args.args {
                    let syn::GenericArgument::Type(ty) = argument else {
                        return Err("only concrete type arguments are modeled".into());
                    };
                    result.push(exact(source, module, ty)?);
                }
            }
            _ => return Err("multiple generic argument lists or associated projection".into()),
        }
    }
    Ok(result)
}

pub(in super::super) fn substitutions(
    definition: &Nominal,
    args: &[ExactTy],
) -> Result<BTreeMap<String, ExactTy>, String> {
    if definition.generics.params.len() != args.len() {
        return Err("exact type argument count differs".into());
    }
    definition
        .generics
        .params
        .iter()
        .zip(args)
        .map(|(parameter, ty)| {
            let syn::GenericParam::Type(parameter) = parameter else {
                return Err("lifetime/const type substitution not modeled".into());
            };
            Ok((parameter.ident.to_string(), ty.clone()))
        })
        .collect()
}

fn expand(
    source: &Source,
    module: &str,
    syntax: &syn::Type,
    substitutions_: &BTreeMap<String, ExactTy>,
    active: &mut BTreeSet<Id>,
) -> Result<ExactTy, String> {
    match syntax {
        syn::Type::Tuple(t) if t.elems.is_empty() => Ok(ExactTy::Unit),
        syn::Type::Tuple(t) => t
            .elems
            .iter()
            .map(|t| expand(source, module, t, substitutions_, active))
            .collect::<Result<_, _>>()
            .map(ExactTy::Tuple),
        syn::Type::Reference(r) if r.lifetime.is_none() => Ok(ExactTy::Reference(
            Box::new(expand(source, module, &r.elem, substitutions_, active)?),
            r.mutability.is_some(),
        )),
        syn::Type::Slice(t) => Ok(ExactTy::Slice(Box::new(expand(
            source,
            module,
            &t.elem,
            substitutions_,
            active,
        )?))),
        syn::Type::Array(t) => {
            let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            }) = &t.len
            else {
                return Err("array length requires a literal".into());
            };
            Ok(ExactTy::Array(
                Box::new(expand(source, module, &t.elem, substitutions_, active)?),
                n.base10_parse().map_err(|e| format!("array length: {e}"))?,
            ))
        }
        syn::Type::Paren(p) => expand(source, module, &p.elem, substitutions_, active),
        syn::Type::Path(p) if p.qself.is_none() => {
            if let Some(id) = p.path.get_ident() {
                if let Some(ty) = substitutions_.get(&id.to_string()) {
                    return Ok(ty.clone());
                }
            }
            let id = resolve::path(source, module, &p.path)?;
            let mut args = Vec::new();
            for segment in &p.path.segments {
                match &segment.arguments {
                    syn::PathArguments::None => {}
                    syn::PathArguments::AngleBracketed(a) if args.is_empty() => {
                        for arg in &a.args {
                            let syn::GenericArgument::Type(t) = arg else {
                                return Err("only concrete type arguments are modeled".into());
                            };
                            args.push(expand(source, module, t, substitutions_, active)?);
                        }
                    }
                    _ => return Err("associated/generic projection not modeled".into()),
                }
            }
            if matches!(
                id.as_str(),
                "bool"
                    | "usize"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "isize"
                    | "char"
                    | "str"
            ) && args.is_empty()
            {
                return Ok(ExactTy::Scalar(id));
            }
            if let Some(defs) = source.types.get(&id) {
                let [def] = defs.as_slice() else {
                    return Err(format!("ambiguous type {id}"));
                };
                if def.conditional {
                    return Err(format!("conditional type {id}"));
                }
                let params = substitutions(def, &args)?;
                if let Some(alias) = &def.alias {
                    if !active.insert(id.clone()) {
                        return Err(format!(
                            "recursive type/alias requires an exact cleanup contract: {id}"
                        ));
                    }
                    let result = expand(source, &def.module, alias, &params, active);
                    active.remove(&id);
                    return result;
                }
                return Ok(ExactTy::Nominal(id, args));
            }
            let arity = match id.as_str() {
                "std::option::Option"
                | "std::vec::Vec"
                | "std::boxed::Box"
                | "std::collections::HashSet" => 1,
                "std::result::Result"
                | "std::collections::BTreeMap"
                | "std::collections::HashMap" => 2,
                "std::string::String" => 0,
                _ => return Err(format!("opaque or ambiguous type {id}")),
            };
            if args.len() != arity {
                return Err(format!("exact type argument count differs for {id}"));
            }
            Ok(ExactTy::Nominal(id, args))
        }
        _ => Err(format!(
            "type is outside the admitted surface: {}",
            tokens(syntax)
        )),
    }
}

pub(in super::super) fn fields(
    source: &Source,
    ty: &ExactTy,
    variant: Option<&str>,
) -> Result<BTreeMap<String, ExactTy>, String> {
    if let ExactTy::Tuple(ts) = ty {
        return Ok(ts
            .iter()
            .enumerate()
            .map(|(i, t)| (i.to_string(), t.clone()))
            .collect());
    }
    let ExactTy::Nominal(id, args) = ty else {
        return Err("field base has no nominal/tuple identity".into());
    };
    match (id.as_str(), variant) {
        ("std::option::Option", Some("Some")) if args.len() == 1 => {
            return Ok(BTreeMap::from([("0".into(), args[0].clone())]));
        }
        ("std::option::Option", Some("None")) if args.len() == 1 => return Ok(BTreeMap::new()),
        ("std::result::Result", Some("Ok")) if args.len() == 2 => {
            return Ok(BTreeMap::from([("0".into(), args[0].clone())]));
        }
        ("std::result::Result", Some("Err")) if args.len() == 2 => {
            return Ok(BTreeMap::from([("0".into(), args[1].clone())]));
        }
        _ => {}
    }
    let Some([def]) = source.types.get(id).map(Vec::as_slice) else {
        return Err(format!("no exact fields for {id}"));
    };
    if def.conditional {
        return Err(format!("conditional fields for {id}"));
    }
    let params = substitutions(def, args)?;
    let fields = match variant {
        Some(v) => def
            .variants
            .get(v)
            .ok_or_else(|| format!("no variant {id}::{v}"))?,
        None if def.variants.is_empty() => &def.fields,
        _ => return Err("enum field projection requires a checked variant".into()),
    };
    fields
        .iter()
        .map(|f| {
            if f.conditional {
                return Err(format!(
                    "conditional field {} at {}",
                    f.name,
                    def.site.display()
                ));
            }
            Ok((
                f.name.clone(),
                substitute(source, &def.module, &f.ty, &params)?,
            ))
        })
        .collect()
}

pub(in super::super) fn variants(source: &Source, ty: &ExactTy) -> Result<Vec<String>, String> {
    let ExactTy::Nominal(id, _) = ty else {
        return Err("variant base is not a nominal type".into());
    };
    Ok(match id.as_str() {
        "std::option::Option" => vec!["None".into(), "Some".into()],
        "std::result::Result" => vec!["Err".into(), "Ok".into()],
        _ => source
            .types
            .get(id)
            .and_then(|defs| (defs.len() == 1).then(|| defs[0].variants.keys().cloned().collect()))
            .ok_or("variant definition unresolved")?,
    })
}
