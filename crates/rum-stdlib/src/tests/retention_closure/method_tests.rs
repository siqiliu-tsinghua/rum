//! Compiled controls first; exact located falsifiers and independent guards.

use super::{contracts, effects, fixtures, model::*, regions, resolve};

const PREFIX: &str = "fixture::methods::";

fn negative(name: &str, kind: Kind, line: usize, chain: &[&str]) {
    let source = fixtures::program();
    let root = format!("{PREFIX}{name}");
    let error = if kind == Kind::RET002 {
        let f = resolve::function(&source, &root).unwrap();
        regions::definition_edges(&source, f)
            .rejected
            .into_iter()
            .find(|r| r.site.start.0 == line)
            .unwrap_or_else(|| panic!("required located method rejection: {name}"))
    } else {
        effects::check(&source, &root, &contracts())
            .err()
            .expect("required rejection")
    };
    println!("METHOD_FALSIFIER {name}\t{error}");
    assert_eq!(error.kind, kind);
    assert!(error.site.file.ends_with("fixtures/methods.rs"));
    assert_eq!(error.site.start.0, line);
    assert_eq!(
        error.witness,
        chain
            .iter()
            .map(|s| format!("{PREFIX}{s}"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn nominal_method_selection_is_exact() {
    let source = fixtures::program();
    let f = resolve::function(&source, &format!("{PREFIX}nominal")).unwrap();
    let edges = regions::definition_edges(&source, f);
    assert!(edges.rejected.is_empty(), "{:?}", edges.rejected);
    assert_eq!(
        edges
            .resolved
            .iter()
            .map(|r| r.target.as_str())
            .collect::<Vec<_>>(),
        [
            "fixture::methods::Left::choose",
            "fixture::methods::Right::choose"
        ]
    );
    for name in [
        "nominal",
        "consuming",
        "projected",
        "annotated",
        "patterned",
        "recursive",
    ] {
        effects::check(&source, &format!("{PREFIX}{name}"), &contracts())
            .unwrap_or_else(|e| panic!("{e}"));
    }
    negative("collision", Kind::RET002, 32, &["collision"]);
    negative(
        "allocation",
        Kind::RET001,
        101,
        &["allocation", "Allocator::allocate", "allocating_helper"],
    );
}

#[test]
fn receiver_adjustments_fail_closed() {
    negative("trait_wins", Kind::RET002, 77, &["trait_wins"]);
    negative("user_deref", Kind::RET002, 92, &["user_deref"]);
    let source = fixtures::program();
    for name in ["future_receiver", "array_receiver"] {
        let f = resolve::function(&source, &format!("{PREFIX}{name}")).unwrap();
        let edges = regions::definition_edges(&source, f);
        assert!(edges.resolved.is_empty());
        assert_eq!(edges.rejected.len(), 1);
    }
    for name in ["mutable", "nested"] {
        effects::check(&source, &format!("{PREFIX}{name}"), &contracts())
            .unwrap_or_else(|e| panic!("{e}"));
    }
    // The compiled nominal control cannot license a signature rewritten by an
    // opaque attribute, or a second impl hidden in an unexpanded item.
    let mut changed = source.clone();
    let f = &mut changed
        .functions
        .get_mut(&format!("{PREFIX}nominal"))
        .unwrap()[0];
    f.attrs.push(syn::parse_quote!(#[opaque_attribute]));
    let f = f.clone();
    assert!(regions::definition_edges(&changed, &f).resolved.is_empty());
    let mut changed = source.clone();
    changed
        .opaque_inherent_owners
        .insert(format!("{PREFIX}Left"));
    let f = resolve::function(&changed, &format!("{PREFIX}nested")).unwrap();
    assert!(regions::definition_edges(&changed, f).resolved.is_empty());
}

#[test]
fn installed_instances_do_not_cross() {
    let source = fixtures::program();
    let proof = effects::check(&source, &format!("{PREFIX}instances"), &contracts())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        proof
            .instances
            .iter()
            .filter(|k| k.id == "fixture::methods::Generic::extract")
            .count(),
        2
    );
    assert!(
        proof
            .facts
            .iter()
            .any(|f| f.exact == Some(ExactTy::Scalar("u8".into())))
    );
    assert!(
        proof
            .facts
            .iter()
            .any(|f| f.exact == Some(ExactTy::Scalar("usize".into())))
    );
    assert!(effects::check(&source, &format!("{PREFIX}open"), &contracts()).is_err());
    assert!(effects::check(&source, &format!("{PREFIX}growing"), &contracts()).is_err());
    let proof = effects::check(&source, &format!("{PREFIX}method_instances"), &contracts())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        proof
            .instances
            .iter()
            .filter(|k| k.id == "fixture::methods::Left::echo")
            .count(),
        2
    );
}
