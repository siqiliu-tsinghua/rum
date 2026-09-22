//! Exact, finite type expansion and implicit cleanup classification.

use std::collections::BTreeSet;

use super::model::*;
use super::{leaves, resolve};

pub(super) fn attributes(attrs: &[syn::Attribute]) -> Result<(), String> {
    for attr in attrs {
        if !matches!(
            tokens(attr.path()).as_str(),
            "doc" | "allow" | "warn" | "deny" | "forbid" | "inline" | "cold" | "must_use"
        ) {
            return Err(format!("unclassified attribute {}", tokens(&attr.meta)));
        }
    }
    Ok(())
}

pub(super) fn ty(source: &Source, module: &str, syntax: &syn::Type) -> Result<Ty, String> {
    expand(source, module, syntax, &mut BTreeSet::new())
}

fn expand(
    source: &Source,
    module: &str,
    syntax: &syn::Type,
    active: &mut BTreeSet<Id>,
) -> Result<Ty, String> {
    match syntax {
        syn::Type::Tuple(t) if t.elems.is_empty() => Ok(Ty::Unit),
        syn::Type::Tuple(t) => t
            .elems
            .iter()
            .map(|t| expand(source, module, t, active))
            .collect::<Result<Vec<_>, _>>()
            .map(Ty::Tuple),
        syn::Type::Reference(r) if r.lifetime.is_none() => Ok(Ty::Borrow(
            Box::new(expand(source, module, &r.elem, active)?),
            r.mutability.is_some(),
        )),
        syn::Type::Paren(p) => expand(source, module, &p.elem, active),
        syn::Type::Path(p) if p.qself.is_none() => {
            let id = resolve::path(source, module, &p.path)?;
            if source.drop_types.contains(&id) {
                return Err(format!("unclassified Drop for {id}"));
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
            ) {
                return Ok(Ty::Scalar(id));
            }
            if matches!(
                id.as_str(),
                "std::vec::Vec"
                    | "std::boxed::Box"
                    | "std::string::String"
                    | "std::collections::BTreeMap"
                    | "std::collections::HashMap"
                    | "std::collections::HashSet"
            ) {
                return Ok(Ty::Raw(id));
            }
            if p.path.segments.iter().any(|s| !s.arguments.is_empty()) {
                return Err(format!("generic substitution or projection for {id}"));
            }
            if let Some(class) = leaves::nominal(&id) {
                return Ok(class);
            }
            let Some([definition]) = source.types.get(&id).map(Vec::as_slice) else {
                return Err(format!("opaque or ambiguous type {id}"));
            };
            if definition.conditional || !definition.generics.params.is_empty() {
                return Err(format!(
                    "conditional or generic type {id} at {}",
                    definition.site.display()
                ));
            }
            attributes(&definition.attrs)?;
            if !active.insert(id.clone()) {
                return Err(format!(
                    "recursive type/alias requires an exact cleanup contract: {id}"
                ));
            }
            let result = if let Some(alias) = &definition.alias {
                expand(source, &definition.module, alias, active)
            } else {
                let mut class = Ty::Fixed(id.clone());
                for field in &definition.fields {
                    match expand(source, &definition.module, field, active)? {
                        Ty::Scalar(_) | Ty::Unit | Ty::Fixed(_) => {}
                        _ => class = Ty::Raw(id.clone()),
                    }
                }
                Ok(class)
            };
            active.remove(&id);
            result
        }
        _ => Err(format!(
            "type is outside the admitted surface: {}",
            tokens(syntax)
        )),
    }
}

pub(super) fn output(source: &Source, f: &Function) -> Result<Ty, String> {
    match &f.sig.output {
        syn::ReturnType::Default => Ok(Ty::Unit),
        syn::ReturnType::Type(_, t) => ty(source, &f.module, t),
    }
}

pub(super) fn signature(source: &Source, f: &Function) -> Result<Vec<(String, Ty)>, String> {
    attributes(&f.attrs)?;
    if f.conditional
        || !f.sig.generics.params.is_empty()
        || f.sig.generics.where_clause.is_some()
        || f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
        || f.sig.abi.is_some()
        || f.sig.variadic.is_some()
    {
        return Err("only unconditional monomorphic safe Rust signatures are admitted".into());
    }
    f.sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(arg) => {
                let syn::Pat::Ident(name) = &*arg.pat else {
                    return Err("only explicit identifier arguments are admitted".into());
                };
                if name.by_ref.is_some() || name.subpat.is_some() {
                    return Err("argument pattern has implicit ownership effects".into());
                }
                Ok((name.ident.to_string(), ty(source, &f.module, &arg.ty)?))
            }
            syn::FnArg::Receiver(receiver) => {
                let owner = f.owner.as_ref().ok_or("unresolved Self receiver")?;
                let t: syn::Type = syn::parse_str(owner).map_err(|e| format!("receiver: {e}"))?;
                let owner_ty = ty(source, &f.module, &t)?;
                Ok((
                    "self".into(),
                    if receiver.reference.is_some() {
                        Ty::Borrow(Box::new(owner_ty), receiver.mutability.is_some())
                    } else {
                        owner_ty
                    },
                ))
            }
        })
        .collect()
}

pub(super) fn retention(ty: &Ty) -> Option<Kind> {
    match ty {
        Ty::Raw(_) => Some(Kind::RET001),
        Ty::Callback(_) => Some(Kind::RET004),
        Ty::Tuple(ts) => ts.iter().find_map(retention),
        _ => None,
    }
}
