//! Resolution uses the ownership scanner's lexical bindings, never a suffix hit.
use super::scan::{Facts, Function, Population, Site, Use, text};
use crate::{
    model::{Binding, Source},
    resolve,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const FRAME_TRAITS: [&str; 4] = [
    "rum_runtime::runtime::suspension::core::SuspensionFrame",
    "rum_runtime::runtime::suspension::core::PureFrameFinalizer",
    "rum_runtime::runtime::suspension::typed::TypedFrame",
    "rum_runtime::runtime::suspension::typed::SlotFinalizer",
];
const CODEC: &str = "rum_runtime::runtime::suspension::typed::channel::Codec";

pub fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
pub fn stable(prefix: &str, identity: &str) -> String {
    let mut value = 0xcbf29ce484222325_u64;
    for byte in identity.bytes() {
        value ^= u64::from(byte);
        value = value.wrapping_mul(0x100000001b3);
    }
    format!("{prefix}{value:016x}")
}
// Block coordinates distinguish the loader's lexical scopes. Receipts retain
// the enclosing function and type identity while allowing source lines to move.
fn without_positions(id: &str) -> String {
    id.split("::")
        .map(|part| {
            if part.starts_with("block@") {
                "block".into()
            } else if part.starts_with("impl(") {
                part.rsplit_once('@').map_or(part, |(p, _)| p).to_owned()
            } else {
                part.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("::")
}

#[derive(Clone)]
pub struct Frame {
    pub identity: String,
    pub name: String,
    pub kind: String,
    pub site: Site,
    pub input: String,
    pub output: String,
    pub population: Population,
}
pub struct Registry {
    pub slot: String,
    pub mode: String,
    pub site: Site,
}
pub struct Call {
    pub site: Use,
    pub targets: Vec<String>,
    pub reason: String,
}
pub struct Report {
    pub frames: Vec<Frame>,
    pub registry: Vec<Registry>,
    pub calls: Vec<Call>,
    pub unresolved: Vec<String>,
    pub closure: Vec<String>,
}

fn type_id(source: &Source, scope: &str, ty: &syn::Type) -> Option<String> {
    if !source.scopes.contains_key(scope) {
        return None;
    }
    let syn::Type::Path(ty) = ty else {
        return None;
    };
    if ty.qself.is_some() {
        return None;
    }
    let ids = resolve::type_path(source, scope, &ty.path);
    let id = if ids.len() == 1 {
        ids.into_iter().next().unwrap()
    } else if ty.path.is_ident("bool") {
        "bool".into()
    } else {
        return None;
    };
    let mut arguments = Vec::new();
    for segment in &ty.path.segments {
        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
            for arg in &args.args {
                let syn::GenericArgument::Type(arg) = arg else {
                    return None;
                };
                arguments.push(type_id(source, scope, arg)?);
            }
        }
    }
    Some(if arguments.is_empty() {
        id
    } else {
        format!("{id}<{}>", arguments.join(","))
    })
}

pub fn classify(mut source: Source, facts: &Facts) -> Report {
    // Add actual top-level free-function declarations to the shared binding
    // resolver. This is a census-local index, not an ownership discovery edit.
    for f in facts
        .functions
        .iter()
        .filter(|f| f.owner.is_empty() && f.identity == format!("{}::{}", f.scope, f.name))
    {
        if let Some(scope) = source.scopes.get_mut(&f.scope) {
            scope
                .bindings
                .entry(f.name.clone())
                .or_default()
                .insert(Binding {
                    target: f.identity.clone(),
                    boundary: f.boundary.clone(),
                    priority: 0,
                });
        }
    }
    resolve::imports(&mut source);
    let mut report = Report {
        frames: Vec::new(),
        registry: Vec::new(),
        calls: Vec::new(),
        unresolved: facts.unresolved.clone(),
        closure: Vec::new(),
    };
    for implementation in &source.drops {
        let traits = resolve::type_path(&source, &implementation.scope, &implementation.trait_path);
        let name = implementation
            .trait_path
            .segments
            .last()
            .unwrap()
            .ident
            .to_string();
        let Some(trait_id) = traits.iter().next().filter(|_| traits.len() == 1) else {
            if [
                "SuspensionFrame",
                "PureFrameFinalizer",
                "TypedFrame",
                "SlotFinalizer",
                "Codec",
            ]
            .contains(&name.as_str())
            {
                report.unresolved.push(format!(
                    "implementation-trait\t{}\t{}\t{}",
                    implementation.location.display(),
                    text(&implementation.trait_path),
                    text(&implementation.ty)
                ));
            }
            continue;
        };
        if !FRAME_TRAITS.contains(&trait_id.as_str()) && trait_id != CODEC {
            continue;
        }
        let key = (
            super::scan::normalize(std::path::Path::new(&implementation.location.file)),
            implementation.location.line,
            implementation.location.column,
        );
        let Some(ast) = facts.implementations.get(&key) else {
            report.unresolved.push(format!(
                "implementation-AST\t{}",
                implementation.location.display()
            ));
            continue;
        };
        if trait_id == CODEC {
            report.registry.push(Registry {
                slot: compact(&text(&implementation.ty)),
                mode: compact(&ast.mode),
                site: ast.site.clone(),
            });
            continue;
        }
        let syn::Type::Path(ty) = &implementation.ty else {
            report
                .unresolved
                .push(format!("frame-type\t{}", implementation.location.display()));
            continue;
        };
        let ids = resolve::type_path(&source, &implementation.scope, &ty.path);
        if ids.len() != 1 {
            report.unresolved.push(format!(
                "frame-type\t{}\t{ids:?}",
                implementation.location.display()
            ));
            continue;
        }
        let owner = without_positions(ids.iter().next().unwrap());
        report.frames.push(Frame {
            identity: format!("{owner} implements {trait_id}"),
            name: ty.path.segments.last().unwrap().ident.to_string(),
            kind: name,
            site: ast.site.clone(),
            input: ast.input.clone(),
            output: ast.output.clone(),
            population: ast.population,
        });
    }
    let functions: Vec<_> = facts.functions.iter().collect();
    let ids: BTreeSet<_> = functions.iter().map(|f| f.identity.as_str()).collect();
    let mut by_name: BTreeMap<&str, Vec<&Function>> = BTreeMap::new();
    for function in &functions {
        by_name.entry(&function.name).or_default().push(function);
    }
    let owners: BTreeMap<_, _> = functions
        .iter()
        .map(|f| {
            (
                f.identity.as_str(),
                syn::parse_str(&f.owner)
                    .ok()
                    .and_then(|ty| type_id(&source, &f.scope, &ty)),
            )
        })
        .collect();
    for site in &facts.uses {
        if !matches!(
            site.kind.as_str(),
            "call" | "method" | "reference" | "construction"
        ) {
            continue;
        }
        let candidates = by_name
            .get(site.name.as_str())
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let (targets, reason) = targets(&source, candidates, &ids, &owners, site);
        report.calls.push(Call {
            site: site.clone(),
            targets,
            reason,
        });
    }
    report.frames.sort_by(|a, b| a.identity.cmp(&b.identity));
    report.registry.sort_by(|a, b| a.slot.cmp(&b.slot));
    report
}

fn targets(
    source: &Source,
    functions: &[&Function],
    ids: &BTreeSet<&str>,
    owners: &BTreeMap<&str, Option<String>>,
    site: &Use,
) -> (Vec<String>, String) {
    if !source.scopes.contains_key(&site.scope) {
        return (Vec::new(), "lexical scope unresolved".into());
    }
    if site.shadowed {
        return (
            Vec::new(),
            "local binding/function-pointer target unresolved".into(),
        );
    }
    let mut result = BTreeSet::new();
    if site.kind == "method" {
        // Only a literal self receiver is established here. Field/parameter
        // types require provenance and remain explicit instead of being guessed.
        if site.receiver != "self" {
            return (Vec::new(), "receiver type unresolved".into());
        }
        if let Some(Some(owner)) = owners.get(site.caller.as_str()) {
            for f in functions.iter().filter(|f| f.name == site.name) {
                if owners.get(f.identity.as_str()) == Some(&Some(owner.clone())) {
                    result.insert(f.identity.clone());
                }
            }
        }
    } else if let Ok(path) = syn::parse_str::<syn::ExprPath>(&site.target) {
        if path.path.segments.len() == 1 && path.qself.is_none() {
            result.extend(
                resolve::type_path(source, &site.scope, &path.path)
                    .into_iter()
                    .filter(|id| ids.contains(id.as_str())),
            );
        } else {
            let mut owner_path = path.path.clone();
            owner_path.segments.pop();
            let owner = if let Some(qself) = path.qself {
                type_id(source, &site.scope, &qself.ty)
            } else if owner_path.is_ident("Self") {
                owners.get(site.caller.as_str()).cloned().flatten()
            } else {
                type_id(
                    source,
                    &site.scope,
                    &syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: owner_path.clone(),
                    }),
                )
            };
            if let Some(owner) = owner {
                for f in functions.iter().filter(|f| f.name == site.name) {
                    if owners.get(f.identity.as_str()) == Some(&Some(owner.clone())) {
                        result.insert(f.identity.clone());
                    }
                }
            }
            // A qualified free function has no owner type.
            result.extend(
                resolve::type_path(source, &site.scope, &path.path)
                    .into_iter()
                    .filter(|id| ids.contains(id.as_str())),
            );
        }
    }
    let reason = match result.len() {
        0 => "path/alias or function-pointer target unresolved",
        1 => "resolved lexical declaration",
        _ => "ambiguous declaration/trait selection",
    };
    (result.into_iter().collect(), reason.into())
}

pub fn wire_graph(report: &mut Report, facts: &Facts, decoders: &BTreeSet<String>) {
    let functions: BTreeMap<_, _> = facts
        .functions
        .iter()
        .filter(|f| !f.test)
        .map(|f| (f.identity.clone(), f))
        .collect();
    let legacy_slots: BTreeSet<_> = report
        .registry
        .iter()
        .filter(|r| r.mode == "LegacyMode")
        .map(|r| r.slot.as_str())
        .collect();
    let roots: Vec<_> = functions
        .values()
        .filter(|f| {
            legacy_slots.contains(compact(&f.owner).as_str())
                && f.trait_name.ends_with("LegacyCodec")
                && matches!(f.name.as_str(), "old_encode" | "old_decode")
        })
        .map(|f| f.identity.clone())
        .collect();
    let mut queue: VecDeque<_> = roots.iter().cloned().map(|id| (id.clone(), id)).collect();
    let mut seen = BTreeSet::new();
    while let Some((root, id)) = queue.pop_front() {
        if !seen.insert((root.clone(), id.clone())) {
            continue;
        }
        let f = functions[&id];
        let provider = roots.contains(&id);
        let forbidden = f.signature.contains("Session")
            || f.trait_name.ends_with("TypedFrame")
            || f.trait_name.ends_with("Transport")
            || f.trait_name.ends_with("Codec")
                && !provider
                && !f.trait_name.ends_with("LegacyCodec");
        let admitted = provider
            || decoders.contains(&id)
            || f.name.contains("continuation_expr")
            || f.name == "finish_suspended_prepared_call";
        report.closure.push(format!(
            "{root}\t{id}\t{}\t{}",
            if forbidden {
                "forbidden"
            } else if admitted {
                "wire"
            } else {
                "unresolved-purity"
            },
            f.site.fields()
        ));
        for call in report.calls.iter().filter(|call| call.site.caller == id) {
            if call.targets.len() == 1 {
                let target = &call.targets[0];
                if functions.contains_key(target) {
                    queue.push_back((root.clone(), target.clone()));
                }
            } else {
                report.closure.push(format!(
                    "{root}\t{}\tunresolved-call\t{}",
                    call.site.target,
                    call.site.site.fields()
                ));
            }
        }
    }
    report.closure.sort();
}

pub fn legacy_name(name: &str) -> bool {
    name.contains("continuation_expr")
        || name.starts_with("decode_prepared_interpreted")
        || name == "from_block_continuation_expr"
}
pub fn boundary_name(name: &str) -> bool {
    legacy_name(name)
        || matches!(
            name,
            "insert_suspension_frame"
                | "Evaluate"
                | "Finalize"
                | "evaluate"
                | "finalize"
                | "encode"
                | "decode"
                | "old_encode"
                | "old_decode"
                | "pack"
                | "unpack"
        )
}
