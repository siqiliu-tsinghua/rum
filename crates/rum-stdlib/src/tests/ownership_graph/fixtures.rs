//! Independent rediscovery, row deletion, and structural positive/negative probes.

use super::{evidence, graph, model::*, source};

#[test]
fn rediscovery_and_each_missing_mutual_member() {
    // Crucially, production discovery accepts only source, not any expected ids.
    let discovered = graph::discover(source::production());
    let required = [
        "rum_stdlib::associations::basic::LookupOutcome",
        "rum_stdlib::associations::key_reshape::traversal::NestedPathNodes",
        "rum_stdlib::associations::key_reshape::traversal::NestedPathNode",
    ];
    for id in required {
        let row = discovered
            .cycles
            .iter()
            .find(|row| row.id == id)
            .unwrap_or_else(|| panic!("structural rediscovery missed {id}"));
        assert!(!row.witness.is_empty());
        println!("REDISCOVERED {}", row.row());
    }
    let rows = evidence::cycle_rows(&discovered);
    for id in &required[1..] {
        let reduced: Vec<_> = rows
            .iter()
            .filter(|row| !row.starts_with(&format!("{id}\t")))
            .cloned()
            .collect();
        assert_eq!(reduced.len() + 1, rows.len());
        let failure =
            evidence::compare(&rows, &reduced).expect_err("deleting one member must fail");
        assert!(
            failure.starts_with(&format!("missing inventory row: {id}\t")),
            "{failure}"
        );
        assert_eq!(failure.lines().count(), 1);
        println!("DELIBERATE FAILURE (caught): {failure}");
    }
    let drop = &discovered.drops[required[0]];
    assert_eq!(drop.len(), 1);
    assert_eq!(
        drop[0].file,
        "crates/rum-stdlib/src/associations/basic/lookup.rs"
    );
    println!("CROSS-FILE DROP {} -> {}", required[0], drop[0].display());
}

#[test]
fn fresh_names_cross_module_pair_and_recursion_after_comma() {
    let source = source::fixture(&[
        (
            "crates/fixture/src/lib.rs",
            r#"
            use std::collections::HashMap;
            mod first;
            mod second;
            struct FreshlyInvented { next: Option<Box<Self>> }
            struct AfterComma { table: HashMap<u8, (bool, Box<AfterComma>)> }
        "#,
        ),
        (
            "crates/fixture/src/first.rs",
            "pub struct West { pub east: Vec<crate::second::East> }",
        ),
        (
            "crates/fixture/src/second.rs",
            "use crate::first::West as Renamed; pub struct East { pub west: Box<Renamed> }",
        ),
    ]);
    let discovered = graph::discover(source);
    assert_eq!(discovered.unknowns.len(), 1, "{:?}", discovered.unknowns);
    assert!(
        discovered
            .unknowns
            .iter()
            .next()
            .unwrap()
            .construct
            .contains("foreign default hasher")
    );
    let names: Vec<_> = discovered.cycles.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        names,
        [
            "fixture::AfterComma",
            "fixture::FreshlyInvented",
            "fixture::first::West",
            "fixture::second::East"
        ]
    );
    let comma = &discovered.cycles[0].witness;
    assert!(comma.contains("arg1/tuple/.1"), "{comma}");
    for row in evidence::cycle_rows(&discovered) {
        println!("FIXTURE REDISCOVERED {row}");
    }
}

#[test]
fn owning_ports_and_nonowning_storage_do_not_mix_instantiations() {
    let discovered = graph::discover(source::fixture(&[(
        "crates/fixture/src/lib.rs",
        r#"
        use std::sync::{Arc, Weak};
        use std::rc::Rc;
        use std::marker::PhantomData;
        struct Own<T> { payload: T }
        struct Borrow<'a, T> { payload: &'a T }
        type Alias<T> = Own<T>;
        struct Positive { value: Alias<Box<Positive>> }
        struct Negative<'a> { value: Borrow<'a, Negative<'a>>, weak: Weak<Negative<'a>>,
            ghost: PhantomData<Negative<'a>>, zero: [Box<Negative<'a>>; 0], callback: fn(Negative<'a>) }
        struct Shared { next: Arc<Option<Rc<Shared>>> }
        struct Key { map: std::collections::HashMap<Box<Key>, u8> }
        struct Hasher { map: std::collections::HashMap<u8, u8, Box<Hasher>> }
        struct Changing<T> { next: Box<Changing<Vec<T>>>, payload: T }
        fn local() { struct LocallyNamed { next: Vec<Self> } }
    "#,
    )]));
    assert_eq!(discovered.unknowns.len(), 1, "{:?}", discovered.unknowns);
    assert!(
        discovered
            .unknowns
            .iter()
            .next()
            .unwrap()
            .construct
            .contains("foreign default hasher")
    );
    let ids: Vec<_> = discovered.cycles.iter().map(|c| c.id.as_str()).collect();
    for name in ["Positive", "Shared", "Key", "Hasher", "Changing"] {
        assert!(
            ids.contains(&format!("fixture::{name}").as_str()),
            "{ids:?}"
        );
    }
    assert_eq!(ids.len(), 6, "{ids:?}");
    assert!(ids.iter().any(|id| id.ends_with("::LocallyNamed")));
    let shared = discovered
        .cycles
        .iter()
        .find(|c| c.id == "fixture::Shared")
        .unwrap();
    assert_eq!(shared.witness.matches("LastStrongHolder").count(), 2);
}

#[test]
fn opaque_constructs_are_located_and_inventory_removal_fails() {
    let discovered = graph::discover(source::fixture(&[(
        "crates/fixture/src/lib.rs",
        r#"
        pub trait Payload {}
        struct Opaque { host: Box<dyn Payload>, foreign: elsewhere::Foreign,
            callback: fn(Box<Opaque>), borrowed: &'static dyn Payload }
        pub fn input(value: Box<dyn Payload>) {}
        fn local() { let capture = 1; let closure = move || capture; create_types!(); }
        build_type!();
    "#,
    )]));
    assert!(discovered.cycles.is_empty());
    let hosts = evidence::unknown_rows(&discovered, UnknownKind::UnknownHostPayload);
    let unknowns = evidence::unknown_rows(&discovered, UnknownKind::UnknownConstruct);
    assert_eq!(hosts.len(), 2, "{hosts:?}");
    assert_eq!(unknowns.len(), 4, "{unknowns:?}");
    for row in &discovered.unknowns {
        assert!(row.location.line > 0 && row.location.column > 0);
    }
    let failure = evidence::compare(&hosts, &hosts[1..]).unwrap_err();
    println!("DELIBERATE UNKNOWN FAILURE (caught): {failure}");
}

#[test]
fn resolution_uses_declarations_not_container_spellings() {
    let discovered = graph::discover(source::fixture(&[(
        "crates/fixture/src/lib.rs",
        r#"
        mod custom {
            pub struct Vec<T> { ignored: std::marker::PhantomData<T> }
            pub struct Arc<T> { ignored: std::marker::PhantomData<T> }
            pub struct Recursive { own: std::vec::Vec<Self> }
        }
        use custom::{Vec, Arc};
        use custom::Recursive as Renamed;
        struct BorrowedNames { a: Vec<BorrowedNames>, b: Arc<BorrowedNames> }
        struct Root { recursive: Renamed }
        struct Second<T, U> { selected: U, ignored: std::marker::PhantomData<T> }
        type Alias<A, B> = Second<A, B>;
        struct AfterTwoParameters { next: Alias<u8, Box<AfterTwoParameters>> }
    "#,
    )]));
    assert!(discovered.unknowns.is_empty(), "{:?}", discovered.unknowns);
    let ids: Vec<_> = discovered.cycles.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        ["fixture::AfterTwoParameters", "fixture::custom::Recursive"]
    );
}

#[test]
fn coverage_is_not_inferred_from_test_like_filenames() {
    let discovered = graph::discover(source::fixture(&[
        (
            "crates/fixture/src/lib.rs",
            r#"
            #[cfg(test)] #[path = "support.rs"] mod excluded;
            #[path = "tests_are_production.rs"] mod ordinary;
        "#,
        ),
        (
            "crates/fixture/src/tests_are_production.rs",
            "struct Real { next: Vec<Self> }",
        ),
        (
            "crates/fixture/src/support.rs",
            "struct TestOnly { bad: Mystery }",
        ),
        (
            "crates/fixture/src/orphan.rs",
            "struct Unlinked { next: Vec<Self> }",
        ),
        (
            "crates/fixture/src/bin/helper.rs",
            "struct Binary { next: Vec<Self> } fn main() {}",
        ),
    ]));
    assert_eq!(discovered.cycles.len(), 2);
    assert_eq!(discovered.unknowns.len(), 1);
    assert!(
        discovered
            .unknowns
            .iter()
            .next()
            .unwrap()
            .construct
            .contains("not reachable")
    );
    assert_eq!(discovered.source.test_files.len(), 1);
}

#[test]
fn manually_checked_definition_counts() {
    let source = source::production();
    for (file, expected) in [
        (
            "crates/rum-stdlib/src/associations/basic.rs",
            vec!["LookupOutcome", "LookupPlan", "PreparedLookup"],
        ),
        (
            "crates/rum-stdlib/src/associations/key_reshape/traversal.rs",
            vec![
                "BoundedTraversal",
                "FlattenAssociationWork",
                "NestedPathNode",
                "NestedPathNodes",
                "RenderNestedPathResult",
                "RenderNestedPathWork",
            ],
        ),
    ] {
        let names: Vec<_> = source
            .definitions
            .iter()
            .filter(|(_, d)| d.location.file == file && d.alias.is_none())
            .map(|(id, _)| id.rsplit("::").next().unwrap())
            .collect();
        assert_eq!(names, expected);
        println!(
            "HAND CROSS-CHECK {file}: {} definitions: {}",
            names.len(),
            names.join(", ")
        );
    }
}

#[test]
fn unknown_configuration_cannot_invent_an_unconditional_cycle() {
    let discovered = graph::discover(source::fixture(&[(
        "crates/fixture/src/lib.rs",
        r#"
        #[cfg(feature = "one")] struct One { next: Box<Two> }
        #[cfg(not(feature = "one"))] struct Two { next: Box<One> }
        fn production() {
            #[cfg(test)] { let test_capture = || 0; test_macro!(); }
            #[cfg(test)] let also_test = || 1;
        }
    "#,
    )]));
    assert!(discovered.cycles.is_empty());
    assert_eq!(discovered.unknowns.len(), 4, "{:?}", discovered.unknowns);
    assert!(
        discovered
            .unknowns
            .iter()
            .all(|u| !u.construct.contains("captures") && !u.construct.contains("test_macro"))
    );
}

#[test]
fn inline_module_path_attribute_and_super_imports_resolve() {
    let discovered = graph::discover(source::fixture(&[
        (
            "crates/fixture/src/lib.rs",
            r#"
            mod inline {
                #[path = "explicit.rs"] mod child;
                pub struct Upper { child: Box<child::Lower> }
            }
        "#,
        ),
        (
            "crates/fixture/src/inline/explicit.rs",
            "pub struct Lower { parent: Box<super::Upper> }",
        ),
    ]));
    assert!(discovered.unknowns.is_empty(), "{:?}", discovered.unknowns);
    assert_eq!(discovered.cycles.len(), 2);
}

#[test]
fn named_generic_public_inputs_are_opaque_but_borrowed_inputs_are_not() {
    let discovered = graph::discover(source::fixture(&[(
        "crates/fixture/src/lib.rs",
        r#"
        pub fn input<T>(owned: T, borrowed: &T, nested: Vec<T>) where T: Send {}
        pub struct Generic<T> { value: T }
        impl<T> Generic<T> { pub fn new(value: T) -> Self { Self { value } } }
    "#,
    )]));
    let hosts = evidence::unknown_rows(&discovered, UnknownKind::UnknownHostPayload);
    assert_eq!(hosts.len(), 3, "{hosts:?}");
    assert!(
        hosts
            .iter()
            .all(|row| row.contains("opaque owned generic parameter T"))
    );
    assert!(evidence::unknown_rows(&discovered, UnknownKind::UnknownConstruct).is_empty());
    assert!(discovered.cycles.is_empty());
}

#[test]
fn unmodeled_lexical_modules_and_async_captures_are_explicit_unknowns() {
    let discovered = graph::discover(source::fixture(&[(
        "crates/fixture/src/lib.rs",
        r#"
        struct Ordinary { scalar: u8 }
        async fn future(value: String) { }
        fn local() { mod inner { struct Hidden { next: Vec<Self> } } }
    "#,
    )]));
    assert!(discovered.cycles.is_empty());
    assert_eq!(discovered.unknowns.len(), 3, "{:?}", discovered.unknowns);
    assert!(
        discovered
            .unknowns
            .iter()
            .any(|u| u.construct.contains("async function future captures"))
    );
    assert!(
        discovered
            .unknowns
            .iter()
            .any(|u| u.construct.contains("function/block-local module scope"))
    );
}
