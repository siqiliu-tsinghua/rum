use super::{contracts, effects, fixtures, leaves, model::*, resolve, types};

pub(super) const ACCEPTED: &[&str] = &[
    "root",
    "aliases",
    "let_else",
    "explicit_return",
    "partial",
    "borrowed",
    "annotated",
    "qualified",
    "empty",
    "named_generic",
    "named_variant",
    "borrowed_pattern",
    "nested_references",
    "reordered_alias",
    "explicit_prelude",
    "globbed::root",
];

#[test]
fn enum_constructors_preserve_payloads() {
    let source = fixtures::program();
    for name in ACCEPTED {
        let proof = effects::check(
            &source,
            &format!("fixture::constructors::{name}"),
            &contracts(),
        )
        .unwrap_or_else(|error| panic!("VALID_CONTROL {name}\t{error}"));
        assert!(
            proof.facts.iter().all(|fact| fact.provenance.is_some()
                || matches!(fact.exact, Some(ExactTy::Function(_))))
        );
        println!(
            "VALID_CONTROL {name} accepted bodies={} facts={}",
            proof.bodies.len(),
            proof.facts.len()
        );
    }
    for (name, kind, line, start, end, target) in [
        (
            "callback_bad",
            Kind::RET004,
            52,
            5,
            38,
            Some("fixture::accepted::substrate::callback"),
        ),
        (
            "nested_bad",
            Kind::RET004,
            60,
            5,
            42,
            Some("fixture::accepted::substrate::callback"),
        ),
        ("raw_bad", Kind::RET001, 55, 5, 50, None),
    ] {
        let root = format!("fixture::constructors::{name}");
        let error = effects::check(&source, &root, &contracts())
            .err()
            .expect("payload propagation must reject");
        println!("PAYLOAD_CONTROL {name}\t{error}");
        assert_eq!(
            error,
            Diagnostic {
                kind,
                site: site(line, start, line, end),
                target: target.map(str::to_owned),
                witness: vec![root],
                reason: if kind == Kind::RET004 {
                    "callback payload has no retention transfer"
                } else {
                    "unleased owned payload"
                }
                .into()
            }
        );
    }
    for name in [
        "guarded",
        "shadowed",
        "branch_mismatch",
        "unchecked_cleanup",
    ] {
        let error = effects::check(
            &source,
            &format!("fixture::constructors::{name}"),
            &contracts(),
        )
        .err()
        .expect("unsupported ownership must reject");
        println!("OWNERSHIP_CONTROL {name}\t{error}");
        assert_eq!(error.kind, Kind::RET002);
    }
}

pub(super) fn site(line: usize, column: usize, end_line: usize, end_column: usize) -> Site {
    Site {
        file: "crates/rum-stdlib/src/tests/retention_closure/fixtures/constructors.rs".into(),
        start: (line, column),
        end: (end_line, end_column),
    }
}

#[test]
fn variant_shadowing_never_uses_std_intrinsics() {
    let source = fixtures::program();
    for (name, target, line, start, end) in [
        ("some_root", "Some", 95, 35, 48),
        ("ok_root", "Ok", 98, 33, 46),
    ] {
        let root = format!("fixture::constructors::local::{name}");
        let error = effects::check(&source, &root, &contracts())
            .err()
            .expect("same-name local function must enter its rejected body");
        println!("SHADOW_CONTROL {name}\t{error}");
        assert_eq!(
            error,
            Diagnostic {
                kind: Kind::RET002,
                site: site(line, start, line, end),
                target: None,
                witness: vec![root, format!("fixture::constructors::local::{target}")],
                reason: "method dispatch requires autoderef/autoref/trait resolution".into()
            }
        );
    }
}

#[test]
fn partial_moves_keep_sibling_obligations() {
    let source = fixtures::program();
    let proof = effects::check(&source, "fixture::constructors::partial", &contracts())
        .unwrap_or_else(|error| panic!("PARTIAL_CONTROL\t{error}"));
    let fields: Vec<_> = proof
        .facts
        .iter()
        .filter(|f| f.effect == "field-projection")
        .collect();
    assert_eq!(fields.len(), 2);
    assert!(fields.iter().all(|f| matches!(f.ty, Ty::Retained(_))));
    let error = effects::check(&source, "fixture::constructors::partial_bad", &contracts())
        .err()
        .expect("raw sibling remains an obligation");
    println!("SIBLING_CONTROL\t{error}");
    assert_eq!(
        error,
        Diagnostic {
            kind: Kind::RET001,
            site: site(46, 5, 46, 55),
            target: None,
            witness: vec!["fixture::constructors::partial_bad".into()],
            reason: "unleased owned payload".into()
        }
    );
    // Independence: cleanup is rejected, but exact identity remains knowable.
    let f = resolve::function(&source, "fixture::constructors::unchecked_cleanup").unwrap();
    let (_, exact) = types::exact_parameters(&source, f).unwrap().remove(0);
    assert_eq!(
        exact.spelling(),
        "std::option::Option<fixture::constructors::Cleanup>"
    );
    assert_eq!(
        types::retention(&types::ownership(&source, &exact)),
        Some(Kind::RET002)
    );
}

#[test]
fn trusted_instances_are_individually_exercised() {
    let source = fixtures::program();
    let contracts = leaves::instance_contracts(&source);
    let root = "fixture::constructors::instances";
    let proof = effects::check(&source, root, &contracts).unwrap_or_else(|e| panic!("{e}"));
    leaves::validate(&source, &contracts, &proof.leaves).unwrap();
    assert_eq!(proof.leaves.len(), 2);
    for key in &proof.leaves {
        println!("INSTANCE_EXERCISED {}", key.display());
    }
    let mut missing = contracts.clone();
    missing.remove(0);
    let error = effects::check(&source, root, &missing)
        .err()
        .expect("missing u8 contract must reject");
    println!("INSTANCE_MISSING\t{error}");
    assert_eq!(
        error,
        Diagnostic {
            kind: Kind::RET002,
            site: site(117, 6, 117, 16),
            target: None,
            witness: vec![root.into()],
            reason: "generic call substitution has no exact exercised contract".into()
        }
    );
    // Deleting one concrete record cannot mark its same-name sibling used.
    let used = std::collections::BTreeSet::from([contracts[0].key()]);
    let error = leaves::validate(&source, &missing, &used).unwrap_err();
    println!("INSTANCE_UNUSED {error}");
    assert_eq!(
        error,
        "unexercised trusted entry fixture::constructors::keep::<usize>"
    );
    let mut stale = contracts.clone();
    stale[0].record[11].push('0');
    assert!(
        leaves::validate(&source, &stale, &proof.leaves)
            .unwrap_err()
            .contains("stale dependency")
    );
    let wildcard = contracts[0].record.join("\t").replace("::<u8>", "::<*>");
    assert_eq!(
        leaves::read(&wildcard).err().unwrap(),
        "wildcard trusted entry"
    );
}
