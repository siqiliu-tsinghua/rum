//! Exact, finite type expansion and implicit cleanup classification.

use std::collections::BTreeSet;

use super::leaves;
use super::model::*;

/// Closed nominal owned-container set. The local lint's exhaustive exercise
/// test must reject a produced value of every entry, including through aliases.
pub(super) const OWNED_CONTAINERS: &[&str] = &[
    "std::vec::Vec",
    "std::boxed::Box",
    "std::string::String",
    "std::collections::BTreeMap",
    "std::collections::HashMap",
    "std::collections::HashSet",
];

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

#[path = "types/context.rs"]
mod context;
#[path = "types/exact.rs"]
mod exact;
mod modes;
pub(super) use context::constructor_claims;
pub(super) use exact::{arguments, exact, field, fields, substitute, variants};
pub(super) use modes::{
    MODE_DOMAINS, MODES, domain, is_mode, mode_argument, mode_parameter, with_mode,
};

pub(super) fn ownership(source: &Source, exact: &ExactTy) -> Ty {
    ownership_inner(source, exact, &mut BTreeSet::new())
}

fn ownership_inner(source: &Source, exact: &ExactTy, active: &mut BTreeSet<Id>) -> Ty {
    let check = |t: &ExactTy, active: &mut BTreeSet<Id>| ownership_inner(source, t, active);
    match exact {
        ExactTy::Unit => Ty::Unit,
        ExactTy::Scalar(id) => Ty::Scalar(id.clone()),
        ExactTy::Tuple(ts) => Ty::Tuple(ts.iter().map(|t| check(t, active)).collect()),
        // Checked provenance does not imply a receipt for a borrowed owner.
        ExactTy::Reference(inner, mutable) => Ty::Borrow(
            Box::new(Ty::Unchecked(format!(
                "borrowed {}; no receipt",
                inner.spelling()
            ))),
            *mutable,
        ),
        ExactTy::Nominal(id, args) => {
            if source.drop_types.contains(id) {
                return Ty::Unchecked(format!("unclassified Drop for {id}"));
            }
            if let Some(state) = leaves::nominal(id) {
                return state;
            }
            if OWNED_CONTAINERS.contains(&id.as_str()) {
                return Ty::Raw(exact.spelling());
            }
            if !active.insert(id.clone()) {
                return Ty::Unchecked(format!(
                    "recursive type/alias requires an exact cleanup contract: {id}"
                ));
            }
            let result = (|| -> Result<Ty, String> {
                if let Some([def]) = source.types.get(id).map(Vec::as_slice) {
                    attributes(&def.attrs)?;
                }
                let vs = variants(source, exact)?;
                if vs.is_empty() {
                    Ok(Ty::Fields(
                        fields(source, exact, None)?
                            .into_iter()
                            .map(|(n, t)| (n, check(&t, active)))
                            .collect(),
                    ))
                } else {
                    Ok(Ty::Variants(
                        vs.into_iter()
                            .map(|v| {
                                let fs = fields(source, exact, Some(&v))?
                                    .into_iter()
                                    .map(|(n, t)| (n, check(&t, active)))
                                    .collect();
                                Ok((v, fs))
                            })
                            .collect::<Result<_, String>>()?,
                    ))
                }
            })();
            active.remove(id);
            let _ = args;
            result.unwrap_or_else(Ty::Unchecked)
        }
        _ => Ty::Unchecked(format!("cleanup not modeled for {}", exact.spelling())),
    }
}

pub(super) fn in_function(source: &Source, f: &Function, t: &syn::Type) -> Result<ExactTy, String> {
    let mut substitutions = f.substitutions.clone();
    if !substitutions.contains_key("Self") {
        if let Some(owner) = &f.impl_self {
            if let Ok(ty) = substitute(source, &f.module, owner, &substitutions) {
                substitutions.insert("Self".into(), ty);
            }
        }
    }
    let mut t = t.clone();
    modes::normalize_projections(source, &f.module, &mut t, &mut substitutions)?;
    substitute(source, &f.module, &t, &substitutions)
}

pub(super) fn exact_output(source: &Source, f: &Function) -> Result<ExactTy, String> {
    match &f.sig.output {
        syn::ReturnType::Default => Ok(ExactTy::Unit),
        syn::ReturnType::Type(_, t) => in_function(source, f, t),
    }
}

pub(super) fn exact_parameters(
    source: &Source,
    f: &Function,
) -> Result<Vec<(String, ExactTy)>, String> {
    f.sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(arg) => {
                let syn::Pat::Ident(p) = &*arg.pat else {
                    return Err("only explicit identifier arguments are admitted".into());
                };
                if p.by_ref.is_some() || p.subpat.is_some() {
                    return Err("argument pattern has implicit ownership effects".into());
                }
                Ok((p.ident.to_string(), in_function(source, f, &arg.ty)?))
            }
            syn::FnArg::Receiver(r) => {
                if r.colon_token.is_some() {
                    return Err("arbitrary self type is not modeled".into());
                }
                let t = in_function(source, f, &syn::parse_str::<syn::Type>("Self").unwrap())?;
                Ok((
                    "self".into(),
                    if r.reference.is_some() {
                        ExactTy::Reference(Box::new(t), r.mutability.is_some())
                    } else {
                        t
                    },
                ))
            }
        })
        .collect()
}

pub(super) fn output(source: &Source, f: &Function) -> Result<Ty, String> {
    Ok(ownership(source, &exact_output(source, f)?))
}

pub(super) fn signature(source: &Source, f: &Function) -> Result<Vec<(String, Ty)>, String> {
    attributes(&f.attrs)?;
    if f.conditional
        || !f.sig.generics.params.is_empty()
        || !f.impl_generics.params.is_empty()
        || f.sig.generics.where_clause.is_some()
        || f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
        || f.sig.abi.is_some()
        || f.sig.variadic.is_some()
    {
        return Err("only unconditional monomorphic safe Rust signatures are admitted".into());
    }
    exact_parameters(source, f)?
        .into_iter()
        .map(|(name, exact)| {
            let state = ownership(source, &exact);
            if let Ty::Unchecked(reason) = state {
                Err(reason)
            } else {
                Ok((name, state))
            }
        })
        .collect()
}

pub(super) fn retention(ty: &Ty) -> Option<Kind> {
    match ty {
        Ty::Raw(_) => Some(Kind::RET001),
        Ty::Callback(_) => Some(Kind::RET004),
        Ty::Tuple(ts) => ts.iter().find_map(retention),
        Ty::Fields(fs) => fs.values().find_map(retention),
        Ty::Variants(vs) => vs.values().flat_map(|fs| fs.values()).find_map(retention),
        Ty::Unchecked(_) => Some(Kind::RET002),
        _ => None,
    }
}
