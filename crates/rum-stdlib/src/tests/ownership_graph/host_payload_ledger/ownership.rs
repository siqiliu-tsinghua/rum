//! Proofs start at the exact opaque field, never at its enclosing type.

use std::collections::{BTreeSet, VecDeque};

use super::super::model::{Discovery, Id, Unknown, UnknownKind};
use super::super::{effects, resolve};

const EXPRESSIONS: &[&str] = &[
    "rum_core::expr::Expr",
    "rum_core::expr::SharedExpr",
    "rum_core::expr::SharedArgs",
];

fn payload_trait(discovered: &Discovery, row: &Unknown) -> Result<Id, String> {
    let (trail, ty) = row
        .construct
        .split_once(": opaque owned payload ")
        .ok_or("no supported opaque storage type at this row")?;
    let shapes = effects::shapes(&discovered.source);
    let ports = effects::ports(&shapes);
    for (owner, fields) in &shapes {
        for (field, shape) in fields {
            if field.location != row.location {
                continue;
            }
            let mut matches = false;
            effects::walk(
                shape,
                format!("{owner}.{}", field.path),
                &ports,
                &mut |shape, path| {
                    if let effects::Shape::Unknown(UnknownKind::UnknownHostPayload, construct) =
                        shape
                    {
                        matches |= row.construct == format!("{path}: {construct}");
                    }
                },
            );
            if !matches {
                continue;
            }
            let mut scope = &discovered.source.definitions[owner].scope;
            // Normalized aliases carry canonical identities. Generic aliases
            // can substitute a caller-scoped opaque argument: reject those
            // rather than guess its lexical origin from a rendered type.
            for step in trail.split('/').skip(1) {
                if let Some(alias) = step
                    .strip_prefix("alias(")
                    .and_then(|s| s.strip_suffix(')'))
                {
                    let definition = &discovered.source.definitions[alias];
                    if definition.generics.type_params().next().is_some() {
                        return Err(
                            "generic alias opaque scope needs substitution provenance".into()
                        );
                    }
                    scope = &definition.scope;
                }
            }
            let ty: syn::TypeTraitObject =
                syn::parse_str(ty).map_err(|e| format!("opaque type: {e}"))?;
            let bounds: Vec<_> = ty
                .bounds
                .iter()
                .filter_map(|bound| match bound {
                    syn::TypeParamBound::Trait(bound) => Some(bound),
                    _ => None,
                })
                .collect();
            if bounds.len() != 1 {
                return Err("opaque proof requires one unambiguous trait bound".into());
            }
            let ids = resolve::type_path(&discovered.source, scope, &bounds[0].path);
            if ids.len() != 1 {
                return Err(format!("unresolved opaque trait: {ids:?}"));
            }
            return Ok(ids.into_iter().next().unwrap());
        }
    }
    Err("row is not the discovered ownership path of a field".into())
}

pub(super) fn declared_witness(discovered: &Discovery, row: &Unknown) -> Result<String, String> {
    use rum_runtime::kernel_api::{BoundedDestruction, EntropySource, FileIo, PrintSink};

    fn requires_declaration<T: ?Sized + BoundedDestruction>() {}
    // Prove the implication for every implementor, not just a chosen concrete
    // type (or a marker implementation restricted to the trait object itself).
    fn file_io<T: ?Sized + FileIo>() {
        requires_declaration::<T>();
    }
    fn entropy<T: ?Sized + EntropySource>() {
        requires_declaration::<T>();
    }
    fn print_sink<T: ?Sized + PrintSink>() {
        requires_declaration::<T>();
    }
    file_io::<dyn FileIo>();
    entropy::<dyn EntropySource>();
    print_sink::<dyn PrintSink>();

    let id = payload_trait(discovered, row)?;
    // Tie the exact public identities compiled above back to the field's
    // resolved trait, so an export redirected to a different trait cannot pass.
    if !["FileIo", "EntropySource", "PrintSink"].iter().any(|name| {
        resolve::type_path(
            &discovered.source,
            "rum_runtime",
            &syn::parse_str::<syn::Path>(&format!("rum_runtime::kernel_api::{name}")).unwrap(),
        ) == BTreeSet::from([id.clone()])
    }) {
        return Err(format!(
            "no compiler-checked destruction declaration for {id}"
        ));
    }
    Ok(format!(
        "{id}: rum_runtime::runtime::host_payload::BoundedDestruction (compiler-checked supertrait)"
    ))
}

pub(super) fn expression_witness(discovered: &Discovery, row: &Unknown) -> Result<String, String> {
    let trait_id = payload_trait(discovered, row)?;
    // Despite its historical name, source.drops records all explicit trait
    // implementations. Resolve both sides; a same-spelled unrelated trait or
    // an Expr somewhere else on FrameAction cannot supply this witness.
    let mut implementations = Vec::new();
    for implementation in &discovered.source.drops {
        if resolve::type_path(
            &discovered.source,
            &implementation.scope,
            &implementation.trait_path,
        ) != BTreeSet::from([trait_id.clone()])
        {
            continue;
        }
        let syn::Type::Path(ty) = &implementation.ty else {
            continue;
        };
        // A generic implementation requires an instantiation/obligation proof,
        // which this existential witness deliberately does not pretend to make.
        if ty.qself.is_some()
            || ty
                .path
                .segments
                .iter()
                .any(|s| !matches!(s.arguments, syn::PathArguments::None))
        {
            continue;
        }
        let ids = resolve::type_path(&discovered.source, &implementation.scope, &ty.path);
        if ids.len() == 1 {
            let id = ids.into_iter().next().unwrap();
            if discovered
                .source
                .definitions
                .get(&id)
                .is_some_and(|def| def.alias.is_none() && def.generics.params.is_empty())
            {
                implementations.push((id, &implementation.location));
            }
        }
    }
    implementations.sort();
    for (id, location) in implementations {
        if let Some(path) = ownership_path(discovered, &id, EXPRESSIONS) {
            return Ok(format!(
                "{trait_id} => {id} (impl at {}); {path}",
                location.display()
            ));
        }
    }
    Err(format!(
        "opaque ownership via {trait_id} has no verified path to Expr/SharedExpr/SharedArgs; cannot attribute Expr debt"
    ))
}

fn ownership_path(discovered: &Discovery, start: &str, targets: &[&str]) -> Option<String> {
    let mut queue = VecDeque::from([(start.to_string(), Vec::<String>::new())]);
    let mut visited = BTreeSet::new();
    while let Some((id, path)) = queue.pop_front() {
        if !visited.insert(id.clone()) {
            continue;
        }
        if targets.contains(&id.as_str()) && discovered.source.definitions.contains_key(&id) {
            return Some(path.join(" ; "));
        }
        for edge in discovered.edges.get(&id).into_iter().flatten() {
            let mut next = path.clone();
            next.push(format!("{} -> {}", edge.witness, edge.target));
            queue.push_back((edge.target.clone(), next));
        }
    }
    None
}

#[test]
fn attribution_resolves_alias_scopes_and_excludes_nonowning_paths() {
    use super::super::{graph, source};

    let discovered = graph::discover(source::fixture(&[(
        "crates/fixture/src/lib.rs",
        r#"
        mod live {
            pub struct Expr;
            pub trait Payload {}
            pub type Erased = dyn Payload;
            pub struct Saved { expr: Expr }
            impl Payload for Saved {}
        }
        mod unrelated {
            pub struct Expr;
            pub trait Payload {}
            pub type Erased = dyn Payload;
            pub struct Chain { next: Option<Box<Self>> }
            impl Payload for Chain {}
        }
        struct FrameAction {
            sibling: live::Expr,
            live: Box<live::Erased>,
            unrelated: Box<unrelated::Erased>,
        }
        struct Borrowed {
            reference: &'static live::Expr,
            weak: std::sync::Weak<live::Expr>,
            phantom: std::marker::PhantomData<live::Expr>,
            callback: fn(live::Expr) -> live::Expr,
            empty: [live::Expr; 0],
        }
        struct SameName { expr: unrelated::Expr }
        struct Retains<T> { item: T }
        type Owned = Box<Retains<live::Expr>>;
        struct ViaAlias { expr: Owned }
        type GenericAlias<T> = Box<T>;
        struct NeedsProvenance { payload: GenericAlias<dyn live::Payload> }
        "#,
    )]));
    for (field, expected) in [
        ("live", "fixture::live::Payload"),
        ("unrelated", "fixture::unrelated::Payload"),
    ] {
        let row = discovered
            .unknowns
            .iter()
            .find(|row| {
                row.kind == UnknownKind::UnknownHostPayload
                    && row
                        .construct
                        .starts_with(&format!("fixture::FrameAction.{field}/"))
            })
            .unwrap();
        assert_eq!(payload_trait(&discovered, row).unwrap(), expected);
    }
    let targets = &["fixture::live::Expr"];
    for id in ["Borrowed", "SameName", "unrelated::Chain"] {
        assert!(ownership_path(&discovered, &format!("fixture::{id}"), targets).is_none());
    }
    for id in ["FrameAction", "live::Saved", "ViaAlias"] {
        assert!(ownership_path(&discovered, &format!("fixture::{id}"), targets).is_some());
    }
    let generic = discovered
        .unknowns
        .iter()
        .find(|row| {
            row.construct
                .starts_with("fixture::NeedsProvenance.payload/")
        })
        .unwrap();
    assert!(
        payload_trait(&discovered, generic)
            .unwrap_err()
            .contains("substitution provenance")
    );
}
