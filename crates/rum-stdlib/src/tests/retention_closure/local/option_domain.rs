//! A flat reconciliation of the installer's one declaration, concrete impls,
//! actual runtime installation and finite build keys. No call traversal.
use super::super::source;
use super::*;

pub(super) const DECLARATION: &str = "crates/rum-stdlib/src/option_domain.rs";
pub(super) const TRAIT: &str = "rum_runtime::runtime::options::reentry::OptionReentry";

pub(super) fn compiled_members() -> Vec<ExactTy> {
    crate::option_domain::DECLARATION
        .iter()
        .map(|d| protocol::nominal(d.identity().1))
        .collect()
}

fn has_derive(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .any(|attr| {
            attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|paths| {
                paths
                    .iter()
                    .any(|path| path.segments.last().is_some_and(|p| p.ident == name))
            })
        })
}

pub(super) fn copy_member(source: &Source, ty: &ExactTy) -> bool {
    if !compiled_members().contains(ty) {
        return false;
    }
    let ExactTy::Nominal(id, _) = ty else {
        return false;
    };
    let Some([d]) = source.types.get(id).map(Vec::as_slice) else {
        return false;
    };
    !d.conditional
        && d.generics().params.is_empty()
        && d.fields.is_empty()
        && !d.variants.is_empty()
        && d.variants.values().all(Vec::is_empty)
        && !source.drop_types.contains(id)
        && has_derive(&d.attrs(), "Copy")
        && has_derive(&d.attrs(), "FrameRootTrace")
}

pub(super) fn specialize(
    source: &Source,
    f: &Function,
    arguments: &[ExactTy],
    members: &[ExactTy],
) -> Result<Function, String> {
    if !f.impl_generics().params.is_empty()
        || f.sig().generics.where_clause.is_some()
        || f.sig().generics.params.len() != arguments.len()
    {
        return Err("exact option instance arity or bounds differ".into());
    }
    types::domain(source)?;
    let mut instance = f.clone();
    let mut seen = BTreeSet::new();
    for (parameter, ty) in f.sig().generics.params.iter().zip(arguments) {
        let syn::GenericParam::Type(p) = parameter else {
            return Err("only declared option type parameters".into());
        };
        if !p.attrs.is_empty() || p.default.is_some() || p.bounds.len() != 1 {
            return Err("exact option bound required".into());
        }
        let syn::TypeParamBound::Trait(bound) = &p.bounds[0] else {
            return Err("option trait bound required".into());
        };
        let bound_id = resolve::path(source, &f.module, &bound.path)?;
        if bound.lifetimes.is_some()
            || !matches!(bound.modifier, syn::TraitBoundModifier::None)
            || !seen.insert(bound_id.clone())
        {
            return Err("changed option bound".into());
        }
        let valid = match bound_id.as_str() {
            "rum_runtime::runtime::accounting_mode::lease::Mode" => types::is_mode(ty),
            TRAIT => members.contains(ty) && copy_member(source, ty),
            _ => false,
        };
        if !valid {
            return Err("instance outside certified M x R domain".into());
        }
        instance
            .substitutions
            .insert(p.ident.to_string(), ty.clone());
    }
    instance.sig = without_generics(&instance.sig);
    Ok(instance)
}

pub(super) fn members(source: &Source) -> Result<Vec<ExactTy>, String> {
    let text = source
        .files
        .get(DECLARATION)
        .ok_or("missing option declaration")?;
    let file = syn::parse_file(text).map_err(|e| e.to_string())?;
    let declarations: Vec<_> = file
        .items
        .iter()
        .filter_map(|item| {
            if let syn::Item::Const(c) = item {
                Some(c)
            } else {
                None
            }
        })
        .collect();
    let [declaration] = declarations.as_slice() else {
        return Err("one option declaration required".into());
    };
    if declaration.ident != "DECLARATION" || !declaration.attrs.is_empty() {
        return Err("conditional or renamed option declaration".into());
    }
    let syn::Expr::Reference(reference) = &*declaration.expr else {
        return Err("static domain slice required".into());
    };
    let syn::Expr::Array(array) = &*reference.expr else {
        return Err("literal finite domain required".into());
    };
    let mut members = BTreeSet::new();
    for expr in &array.elems {
        let syn::Expr::Call(call) = expr else {
            return Err("concrete declaration constructor required".into());
        };
        let syn::Expr::Path(path) = &*call.func else {
            return Err("domain constructor path required".into());
        };
        if path.qself.is_some()
            || resolve::path(source, "rum_stdlib::option_domain", &path.path)?
                != "rum_runtime::runtime::options::reentry::OptionReentryDeclaration::of"
        {
            return Err("wrong domain constructor".into());
        }
        let arguments = types::arguments(source, "rum_stdlib::option_domain", &path.path)?;
        let [ty @ ExactTy::Nominal(id, args)] = arguments.as_slice() else {
            return Err("one concrete descriptor type required".into());
        };
        let literals = call.args.iter().cloned().collect::<Vec<_>>();
        let [
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(key),
                ..
            }),
        ] = literals.as_slice()
        else {
            return Err("one literal certificate key required".into());
        };
        if !args.is_empty() || key.value() != *id || id.contains('*') || !members.insert(ty.clone())
        {
            return Err("duplicate, wildcard or mismatched descriptor key".into());
        }
    }
    if members.is_empty() {
        return Err("certified option domain cannot be empty".into());
    }
    let mut implementations = BTreeSet::new();
    for (module, item, _, conditional) in &source.trait_impls {
        let item = ast::<syn::ItemImpl>(item);
        let Some((_, path, _)) = &item.trait_ else {
            continue;
        };
        if resolve::path(source, module, path).ok().as_deref() != Some(TRAIT) {
            continue;
        }
        if *conditional || !item.generics.params.is_empty() || item.generics.where_clause.is_some()
        {
            return Err("conditional or open option implementation".into());
        }
        let ty = types::exact(source, module, &item.self_ty)?;
        if !members.contains(&ty) || !implementations.insert(ty.clone()) {
            return Err("option implementation outside declaration".into());
        }
        let ExactTy::Nominal(id, _) = ty else {
            return Err("nominal descriptor required".into());
        };
        let Some([definition]) = source.types.get(&id).map(Vec::as_slice) else {
            return Err("unique descriptor declaration required".into());
        };
        if definition.conditional
            || !definition.generics().params.is_empty()
            || source.drop_types.contains(&id)
            || !definition.fields.is_empty()
            || definition.variants.is_empty()
            || definition.variants.values().any(|fs| !fs.is_empty())
            || !has_derive(&definition.attrs(), "Copy")
            || !has_derive(&definition.attrs(), "FrameRootTrace")
        {
            return Err(
                "descriptor must have checked fieldless Copy and root trace derives".into(),
            );
        }
    }
    if implementations != members {
        return Err("declaration and option impl headers differ".into());
    }
    let installer = resolve::function(source, "rum_stdlib::install_core")?;
    struct Installs(Vec<String>);
    impl<'a> Visit<'a> for Installs {
        fn visit_expr_method_call(&mut self, call: &'a syn::ExprMethodCall) {
            if ["install_option_reentry_domain", "register_option_reentry"]
                .iter()
                .any(|name| call.method == *name)
            {
                self.0.push(tokens(call));
            }
            visit::visit_expr_method_call(self, call);
        }
    }
    let mut installs = Installs(vec![]);
    installs.visit_block(&installer.body());
    let expected: syn::Expr =
        syn::parse_quote!(session.install_option_reentry_domain(option_domain::DECLARATION));
    if installer.conditional || installs.0 != [tokens(&expected)] {
        return Err("installer must consume the one domain declaration".into());
    }
    Ok(members.into_iter().collect())
}

pub(super) fn keys(members: &[ExactTy]) -> Vec<String> {
    types::MODES
        .iter()
        .flat_map(|mode| {
            members.iter().map(move |r| {
                format!(
                    "M={mode};R={};hash={}",
                    r.spelling(),
                    cfg!(feature = "hash")
                )
            })
        })
        .collect()
}

pub(super) fn reconcile(
    source: &Source,
    session: &rum_runtime::Session,
    certificate_keys: &[String],
) -> Result<(), String> {
    let members = members(source)?;
    let declared: BTreeSet<_> = members.iter().map(ExactTy::spelling).collect();
    let compiled: BTreeSet<_> = crate::option_domain::DECLARATION
        .iter()
        .map(|d| d.identity().1.to_string())
        .collect();
    if declared != compiled {
        return Err("lint and compiled declaration differ".into());
    }
    let installed: BTreeSet<_> = session
        .certified_option_reentry_domain()
        .iter()
        .map(|d| d.identity())
        .collect();
    let expected: BTreeSet<_> = crate::option_domain::DECLARATION
        .iter()
        .map(|d| d.identity())
        .collect();
    if installed != expected
        || session
            .registered_option_reentry_ids()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            != expected.iter().map(|(id, _)| *id).collect()
    {
        return Err("actual runtime domain or registrations differ".into());
    }
    if certificate_keys != keys(&members) {
        return Err("exact M x R feature keys differ".into());
    }
    Ok(())
}

#[test]
fn local_option_domain_reconciliation_and_independent_drift_controls() {
    let source = source::production();
    let members = members(&source).unwrap();
    let keys = keys(&members);
    let mut session = rum_runtime::Session::new();
    crate::install_core(&mut session);
    reconcile(&source, &session, &keys).unwrap();
    assert!(!members.contains(&protocol::nominal("outside::HostType")));
    let mut declaration = source.clone();
    *declaration.files.get_mut(DECLARATION).unwrap() = "const DECLARATION: &[D] = &[];".into();
    assert!(
        reconcile(&declaration, &session, &keys)
            .unwrap_err()
            .contains("empty")
    );
    let mut implementation = source.clone();
    let (_, _, _, conditional) = implementation
        .trait_impls
        .iter_mut()
        .find(|(module, item, _, _)| {
            ast::<syn::ItemImpl>(item)
                .trait_
                .as_ref()
                .is_some_and(|(_, p, _)| {
                    resolve::path(&source, module, p).ok().as_deref() == Some(TRAIT)
                })
        })
        .unwrap();
    *conditional = true;
    assert!(
        reconcile(&implementation, &session, &keys)
            .unwrap_err()
            .contains("conditional")
    );
    let descriptor = members[0].spelling();
    let mut false_derive = source.clone();
    let attribute: syn::Attribute = syn::parse_quote!(#[doc = "Copy FrameRootTrace"]);
    false_derive.types.get_mut(&descriptor).unwrap()[0].attrs = vec![tokens(&attribute)];
    assert!(
        reconcile(&false_derive, &session, &keys)
            .unwrap_err()
            .contains("derives")
    );
    let mut installer = source.clone();
    installer
        .functions
        .get_mut("rum_stdlib::install_core")
        .unwrap()[0]
        .body = "{ }".into();
    assert!(
        reconcile(&installer, &session, &keys)
            .unwrap_err()
            .contains("installer")
    );
    assert!(
        reconcile(&source, &session, &keys[1..])
            .unwrap_err()
            .contains("keys")
    );
    session.install_option_reentry_domain(&[]);
    assert!(
        reconcile(&source, &session, &keys)
            .unwrap_err()
            .contains("runtime")
    );
    println!(
        "OPTION544_DOMAIN: declaration/impl-feature/installer/runtime/keys mutations rejected; outside type excluded; keys={keys:?}"
    );
}
