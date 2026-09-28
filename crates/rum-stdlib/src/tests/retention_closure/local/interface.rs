//! Index-time fingerprints and signature-only inputs. A caller never receives
//! any other body's syntax, including impl and default-trait bodies.

use super::super::{model::*, source};
use std::collections::BTreeMap;

pub(super) struct Interfaces {
    pub signatures: Source,
    pub bodies: BTreeMap<Id, String>,
    syntax: BTreeMap<Id, String>,
    pub earned: super::marks::Marks,
    pub read_contracts: BTreeMap<Id, Result<(), String>>,
    pub resume_definition: Result<(), String>,
    pub option_domain: Result<Vec<ExactTy>, String>,
    pub certificate_domain: Result<String, String>,
    pub build: String,
}

pub(super) fn probe_body(f: &Function) -> bool {
    f.site.file == "crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs"
        && ["drive", "start", "resume", "probe"]
            .iter()
            .any(|n| f.id == format!("rum_runtime::runtime::accounting_mode::tests::{n}"))
}

pub(super) fn hash(parts: impl IntoIterator<Item = impl AsRef<[u8]>>) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for part in parts {
        let bytes = part.as_ref();
        for byte in (bytes.len() as u64).to_le_bytes().iter().chain(bytes) {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("fnv1a64:{hash:016x}")
}

fn erase_impl(item: &mut syn::ItemImpl) {
    for item in &mut item.items {
        if let syn::ImplItem::Fn(f) = item {
            f.block.stmts.clear();
        }
    }
}

impl Interfaces {
    fn syntax(f: &Function) -> String {
        let mut parts = vec![
            f.id.clone(),
            f.module.clone(),
            tokens(&f.sig),
            tokens(&f.body),
            format!("{:?}", f.substitutions),
            format!("{}:{:?}:{:?}", f.conditional, f.owner, f.boundary),
            tokens(&f.impl_generics),
            f.impl_self.as_ref().map(tokens).unwrap_or_default(),
        ];
        parts.extend(f.attrs.iter().map(tokens));
        hash(parts)
    }

    pub fn matches(&self, f: &Function) -> bool {
        self.syntax.get(&f.id) == Some(&Self::syntax(f))
    }

    pub fn new(input: &Source) -> Self {
        let mut signatures = input.clone();
        signatures.files.clear();
        signatures.constants.clear();
        for f in signatures.functions.values_mut().flatten() {
            f.body.stmts.clear();
        }
        for (_, item, _, _) in &mut signatures.impls {
            erase_impl(item);
        }
        for (_, item, _, _) in &mut signatures.trait_impls {
            erase_impl(item);
        }
        for tr in signatures.traits.values_mut().flatten() {
            for item in &mut tr.item.items {
                if let syn::TraitItem::Fn(f) = item {
                    f.default = None;
                }
            }
        }
        // Conservative flat lexical/type/selection dependency key. It contains
        // no callee bodies and computes no call graph or transitive closure.
        let mut context = vec![format!(
            "local-v1;target={}-{};hash={}",
            std::env::consts::ARCH,
            std::env::consts::OS,
            cfg!(feature = "hash")
        )];
        for (id, scope) in &signatures.modules {
            context.push(format!("{id}:{:?}:{:?}", scope.bindings, scope.hazards));
            for i in &scope.imports {
                context.push(format!(
                    "{:?}:{:?}:{:?}:{}",
                    i.path, i.name, i.boundary, i.conditional
                ));
            }
        }
        for (id, defs) in &signatures.types {
            for d in defs {
                context.push(format!(
                    "{id}:{}:{}:{:?}",
                    d.module,
                    tokens(&d.generics),
                    d.conditional
                ));
                context.extend(d.attrs.iter().map(tokens));
                if let Some(alias) = &d.alias {
                    context.push(tokens(alias));
                }
                for (variant, fields) in std::iter::once(("", &d.fields))
                    .chain(d.variants.iter().map(|(v, fs)| (v.as_str(), fs)))
                {
                    for f in fields {
                        context.push(format!(
                            "{variant}:{}:{}:{}",
                            f.name,
                            tokens(&f.ty),
                            f.conditional
                        ));
                    }
                }
            }
        }
        for f in signatures.functions.values().flatten() {
            context.push(format!(
                "{}:{}:{}:{:?}:{:?}",
                f.id,
                tokens(&f.sig),
                f.conditional,
                f.owner,
                f.boundary
            ));
            context.extend(f.attrs.iter().map(tokens));
        }
        for (_, item, _, c) in signatures.impls.iter().chain(&signatures.trait_impls) {
            context.push(format!("{c}:{}", tokens(item)));
        }
        for tr in signatures.traits.values().flatten() {
            context.push(tokens(&tr.item));
        }
        context.push(format!(
            "{:?}:{:?}:{:?}",
            signatures.dependencies, signatures.drop_types, signatures.opaque_inherent_owners
        ));
        context.push(format!("{:?}", signatures.value_names));
        for file in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"] {
            context.push(std::fs::read_to_string(source::workspace().join(file)).unwrap());
        }
        let root = source::workspace().join("crates/rum-stdlib/src/tests/retention_closure");
        for dir in [
            "",
            "types",
            "methods",
            "effects",
            "local",
            "local/flow",
            "local/reads",
            "local/oracle",
        ] {
            let mut paths = std::fs::read_dir(root.join(dir))
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| p.extension().is_some_and(|e| e == "rs"))
                .collect::<Vec<_>>();
            paths.sort();
            for path in paths {
                context.push(
                    path.strip_prefix(source::workspace())
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                );
                context.push(std::fs::read_to_string(path).unwrap());
            }
        }
        let context = hash(context);
        let bodies = input
            .functions
            .iter()
            .filter_map(|(id, fs)| {
                let [f] = fs.as_slice() else {
                    return None;
                };
                let mut parts = vec![context.clone(), Self::syntax(f)];
                parts.extend(f.attrs.iter().map(tokens));
                // Include original body lines too: even comment-only source drift
                // expires a mark. Synthetic fixture mutations also change tokens.
                if let Some(text) = input.files.get(&f.site.file) {
                    parts.push(
                        text.lines()
                            .skip(f.site.start.0.saturating_sub(1))
                            .take(f.site.end.0.saturating_sub(f.site.start.0) + 1)
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                }
                Some((id.clone(), hash(parts)))
            })
            .collect();
        let syntax = input
            .functions
            .iter()
            .filter_map(|(id, fs)| {
                let [f] = fs.as_slice() else {
                    return None;
                };
                Some((id.clone(), Self::syntax(f)))
            })
            .collect();
        let mut index = Self {
            certificate_domain: super::certificates::domain(input),
            build: super::certificates::build(),
            option_domain: super::option_domain::members(input),
            read_contracts: super::reads::contracts::validate(input)
                .into_iter()
                .chain(super::rules::validate(input))
                .chain(super::option_boundaries::validate(input))
                .collect(),
            resume_definition: super::resume_macro::definition(input),
            signatures,
            bodies,
            syntax,
            earned: super::marks::Marks::default(),
        };
        // Independent local certificates, before any caller run. This fixed
        // test-population order is not a discovered call graph. Only successful
        // runs produce marks; the caller receives no other body's syntax.
        let mut marks = if input.functions.values().flatten().any(probe_body) {
            super::marks::Marks::contracts(
                super::contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap(),
            )
            .unwrap()
        } else {
            super::marks::Marks::default()
        };
        for name in ["drive", "start", "resume", "probe"] {
            let id = format!("rum_runtime::runtime::accounting_mode::tests::{name}");
            if let Some([f]) = input.functions.get(&id).map(Vec::as_slice) {
                if probe_body(f) {
                    let _ = marks.certify(&index, f);
                }
            }
        }
        index.earned = marks;
        index
    }
}
