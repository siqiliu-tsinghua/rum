use super::super::source;
use super::*;

#[test]
fn finite_modes_and_opaque_brands() {
    let source = source::test_module(
        source::production(),
        "crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs",
        "rum_runtime::runtime::accounting_mode::tests",
    );
    for name in ["start", "drive", "resume"] {
        let f = resolve::function(
            &source,
            &format!("rum_runtime::runtime::accounting_mode::tests::{name}"),
        )
        .unwrap();
        let mut instances = BTreeSet::new();
        for id in types::MODES {
            let mode = ExactTy::Nominal(id.into(), Vec::new());
            let instance = types::with_mode(&source, f, Some(&mode)).unwrap();
            let params = types::exact_parameters(&source, &instance).unwrap();
            let target = format!("{}::reserve", protocol::CONTEXT);
            let method = types::with_mode(
                &source,
                protocol::signature(&source, &target).unwrap(),
                Some(&mode),
            )
            .unwrap();
            assert!(
                protocol::method(&source, &params[0].1, "reserve").is_some(),
                "receiver={:?}; params={:?}; output={:?}",
                params[0].1,
                types::exact_parameters(&source, &method),
                types::exact_output(&source, &method)
            );
            assert!(matches!(&params[0].1, ExactTy::Nominal(_, args) if args == &[mode]));
            instances.insert(types::exact_output(&source, &instance).unwrap());
        }
        assert_eq!(
            instances.len(),
            2,
            "both sealed substitutions must be checked"
        );
        let mut open = f.clone();
        open.sig.generics = syn::parse_quote!(<T>);
        assert!(types::with_mode(&source, &open, None).is_err());
        assert!(types::with_mode(&source, f, Some(&ExactTy::Scalar("u8".into()))).is_err());
    }
    let module = "rum_runtime::runtime::accounting_mode::tests";
    let mut changed = source.clone();
    changed.trait_impls.retain(|(m, item, _, _)| {
        item.trait_.as_ref().is_none_or(|(_, p, _)| {
            resolve::path(&source, m, p).ok().as_deref()
                != Some("rum_runtime::runtime::accounting_mode::lease::Mode")
        })
    });
    assert!(
        types::domain(&changed).is_err(),
        "missing finite mode implementation must close enumeration"
    );
    assert!(
        protocol::method(
            &source,
            &protocol::borrow(
                protocol::borrow(protocol::nominal(protocol::ARGS), false),
                false
            ),
            "clone"
        )
        .is_none()
    );
    let a: syn::Type = syn::parse_quote!(&'id SharedArgs);
    let b: syn::Type = syn::parse_quote!(&SharedArgs);
    assert_eq!(
        types::exact(&source, module, &a),
        types::exact(&source, module, &b)
    );
}
