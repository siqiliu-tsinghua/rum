//! Closed, private option-rule boundaries. This is a flat declaration audit at
//! indexing, never a caller's traversal of another body. Runtime falsifiers
//! discharge the ownership claims; these checks expire them on source drift.
use super::*;

#[path = "rule_tests.rs"]
mod tests;

pub(super) const RULE: &str = "rum_runtime::runtime::accounting_mode::rule";
pub(super) const FRAME: &str =
    "rum_runtime::runtime::options::resume_frames::RuntimeOptionSuspensionFrame";
pub(super) const FROZEN: &str = "rum_runtime::runtime::accounting_mode::frozen::FrozenValues";

pub(super) fn extra_targets() -> BTreeMap<String, String> {
    let mut rows = BTreeMap::new();
    for (n, name) in ["clone_result", "evaluate_rule", "resume_rule"]
        .iter()
        .enumerate()
    {
        rows.insert(format!("R{:02}", n + 1), format!("{RULE}::{name}"));
    }
    for (m, mode) in types::MODES.iter().enumerate() {
        for (n, target) in [
            format!("{RULE}::clone_owned"),
            format!("{RULE}::clone_child"),
            format!("{RULE}::finish_rule"),
            format!("{RULE}::evaluate_in"),
            format!("{RULE}::resume_in"),
            format!("{}::adopt_outcome", protocol::CONTEXT),
            format!("{}::into_rule", protocol::CALLBACK),
            format!("{}::suspend_rule", protocol::FAILURE),
        ]
        .iter()
        .enumerate()
        {
            rows.insert(
                format!("R{:02}{m}", n + 4),
                protocol::key(target, &protocol::nominal(mode)),
            );
        }
    }
    rows.insert("R12".into(), format!("{FROZEN}::rule_name"));
    rows.insert("R13".into(), format!("{FRAME}::resume"));
    rows.insert("R14".into(), format!("{RULE}::finish_armed"));
    rows.insert("R15".into(), format!("{RULE}::suspend_armed"));
    rows
}

/// Contract bodies are explicitly enumerated, not discovered through calls.
/// Include the compiled declarations AND the parsed index: synthetic body,
/// signature, cfg, field and impl mutations must fail before a mark is used.
const FILES: &[(&str, &str, &str)] = &[
    (
        "crates/rum-runtime/src/runtime/accounting_mode/rule.rs",
        RULE,
        include_str!("../../../../../rum-runtime/src/runtime/accounting_mode/rule.rs"),
    ),
    (
        "crates/rum-runtime/src/runtime/accounting_mode/bridge.rs",
        "rum_runtime::runtime::accounting_mode::bridge",
        include_str!("../../../../../rum-runtime/src/runtime/accounting_mode/bridge.rs"),
    ),
    (
        "crates/rum-runtime/src/runtime/accounting_mode/frozen.rs",
        "rum_runtime::runtime::accounting_mode::frozen",
        include_str!("../../../../../rum-runtime/src/runtime/accounting_mode/frozen.rs"),
    ),
    (
        "crates/rum-runtime/src/runtime/options/resume_frames.rs",
        "rum_runtime::runtime::options::resume_frames",
        include_str!("../../../../../rum-runtime/src/runtime/options/resume_frames.rs"),
    ),
];

pub(super) fn declarations(source: &Source, files: &[(&str, &str, &str)]) -> Result<(), String> {
    for (path, module, text) in files {
        if source.files.get(*path).map(String::as_str) != Some(*text) {
            return Err(format!("rule boundary source changed: {path}"));
        }
        for item in syn::parse_file(text).unwrap().items {
            match item {
                syn::Item::Fn(item) => {
                    let id = format!("{module}::{}", item.sig.ident);
                    let f = resolve::function(source, &id)?;
                    if f.conditional
                        || tokens(&f.sig) != tokens(&item.sig)
                        || tokens(&f.body) != tokens(&item.block)
                    {
                        return Err(format!("rule boundary declaration changed: {id}"));
                    }
                }
                syn::Item::Impl(item) => {
                    // Only the listed production impls; test modules are not
                    // visited, and no call edge is inspected.
                    let candidates = if item.trait_.is_some() {
                        &source.trait_impls
                    } else {
                        &source.impls
                    };
                    if !candidates.iter().any(|(m, i, _, conditional)| {
                        m == module && !conditional && tokens(i) == tokens(&item)
                    }) {
                        return Err(format!(
                            "rule boundary impl changed: {module}::{}",
                            tokens(&item.self_ty)
                        ));
                    }
                    // Trait implementations stay in the trait index; auditing
                    // their exact item above does not invent an inherent body.
                    for method in item.items.iter().filter(|_| item.trait_.is_none()) {
                        if let syn::ImplItem::Fn(method) = method {
                            let matching: Vec<_> = source
                                .functions
                                .values()
                                .flatten()
                                .filter(|f| {
                                    f.site.file == *path
                                        && f.sig.ident == method.sig.ident
                                        && f.impl_self
                                            .as_ref()
                                            .is_some_and(|s| tokens(s) == tokens(&item.self_ty))
                                })
                                .collect();
                            if matching.len() != 1
                                || matching[0].conditional
                                || tokens(&matching[0].sig) != tokens(&method.sig)
                                || tokens(&matching[0].body) != tokens(&method.block)
                            {
                                return Err(format!(
                                    "rule boundary indexed method changed: {}",
                                    method.sig.ident
                                ));
                            }
                        }
                    }
                }
                syn::Item::Enum(item) => {
                    let id = format!("{module}::{}", item.ident);
                    let Some([ty]) = source.types.get(&id).map(Vec::as_slice) else {
                        return Err(format!("boundary enum schema missing: {id}"));
                    };
                    // Compare every variant field as well as the fieldless
                    // option descriptors; neither admits a schema mutation.
                    let expected = item
                        .variants
                        .iter()
                        .map(|variant| {
                            (
                                variant.ident.to_string(),
                                variant
                                    .fields
                                    .iter()
                                    .enumerate()
                                    .map(|(i, f)| {
                                        (
                                            f.ident
                                                .as_ref()
                                                .map(ToString::to_string)
                                                .unwrap_or_else(|| i.to_string()),
                                            tokens(&f.ty),
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect::<BTreeMap<_, _>>();
                    let actual = ty
                        .variants
                        .iter()
                        .map(|(name, fields)| {
                            (
                                name.clone(),
                                fields
                                    .iter()
                                    .map(|f| (f.name.clone(), tokens(&f.ty)))
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect::<BTreeMap<_, _>>();
                    if ty.conditional
                        || tokens(&ty.generics) != tokens(&item.generics)
                        || ty.attrs.iter().map(tokens).collect::<Vec<_>>()
                            != item.attrs.iter().map(tokens).collect::<Vec<_>>()
                        || actual != expected
                        || ty.variants.values().flatten().any(|f| f.conditional)
                    {
                        return Err(format!("boundary enum schema changed: {id}"));
                    }
                }
                syn::Item::Struct(item) => {
                    let id = format!("{module}::{}", item.ident);
                    let Some([ty]) = source.types.get(&id).map(Vec::as_slice) else {
                        return Err(format!("rule boundary schema missing: {id}"));
                    };
                    let expected = item
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            (
                                f.ident
                                    .as_ref()
                                    .map(ToString::to_string)
                                    .unwrap_or_else(|| i.to_string()),
                                tokens(&f.ty),
                            )
                        })
                        .collect::<Vec<_>>();
                    if ty.conditional
                        || tokens(&ty.generics) != tokens(&item.generics)
                        || ty.fields.iter().any(|f| f.conditional)
                        || ty
                            .fields
                            .iter()
                            .map(|f| (f.name.clone(), tokens(&f.ty)))
                            .collect::<Vec<_>>()
                            != expected
                    {
                        return Err(format!("rule boundary schema changed: {id}"));
                    }
                }
                _ => {}
            }
        }
    }
    if source.drop_types.contains(FRAME) {
        return Err("option continuation acquired custom Drop".into());
    }
    Ok(())
}

pub(super) fn validate(source: &Source) -> BTreeMap<Id, Result<(), String>> {
    // Fixture-only indexes cannot use production rule contracts accidentally.
    let verdict =
        declarations(source, FILES).and_then(|()| super::resume_macro::definition(source));
    extra_targets()
        .into_values()
        .chain(["B10", "B11", "B12"].map(contracts::target))
        .map(|target| (target, verdict.clone()))
        .collect()
}

/// Bare constructor/system_expr/frame calls get no ambient permission. These
/// contracts are usable only inside the unchanged guarded adapter body.
pub(super) fn caller_domain(index: &Interfaces, f: &Function, target: &str) -> Result<(), String> {
    let caller = if target == contracts::target("B10") || target == contracts::target("B11") {
        format!("{RULE}::finish_rule")
    } else if target == contracts::target("B12") {
        format!("{}::suspend_error", protocol::CONTEXT)
    } else {
        return Ok(());
    };
    if f.id != caller || !index.matches(f) {
        return Err(format!(
            "guarded rule/frame caller precondition not established for {target}"
        ));
    }
    Ok(())
}

pub(super) fn dependencies(target: &str) -> Vec<String> {
    if extra_targets().values().any(|t| t == target) {
        ["B04", "B09", "B10", "B11", "B12"]
            .map(contracts::target)
            .into_iter()
            .chain(extra_targets().into_values().filter(|t| t != target))
            .collect()
    } else {
        vec![]
    }
}

pub(super) fn outgoing(target: &str) -> bool {
    ["clone_result", "evaluate_rule", "resume_rule"]
        .iter()
        .any(|n| target == format!("{RULE}::{n}"))
}
