use super::*;
use crate::tests::ownership_graph::model::{Field, Location, Source};

const SUSPENDED: &str = "rum_runtime::runtime::suspension::core::SuspendedEvaluation";

// Mutate discovery's test-side source snapshot, never a production file. The
// extra field owns a recursive generic chain and an unrelated recursive opaque
// payload. Existing expression-bearing suspension fields remain alongside it.
fn with_unrelated_payload(mut source: Source) -> Source {
    let fixture = source::fixture(&[(
        "crates/fixture/src/lib.rs",
        "pub trait OtherPayload {}\n\
         pub struct OtherLink { next: Option<Box<Self>> }\n\
         impl OtherPayload for OtherLink {}\n\
         pub struct OtherChain<T> { next: Option<Box<Self>>, payload: T }\n",
    )]);
    source.definitions.extend(fixture.definitions);
    source.scopes.extend(fixture.scopes);
    source.modules.extend(fixture.modules);
    source.drops.extend(fixture.drops);
    let suspension = source.definitions.get_mut(SUSPENDED).unwrap();
    suspension.fields.push(Field {
        path: "unrelated_payload".into(),
        ty: syn::parse_str("fixture::OtherChain<Box<dyn fixture::OtherPayload>>").unwrap(),
        location: Location {
            file: "test-side-suspension-copy.rs".into(),
            line: 1,
            column: 1,
        },
        conditional: false,
    });
    source
}

fn unrelated_entry(discovered: &Discovery, entries: &[Entry]) -> Entry {
    let row = host_rows(discovered)
        .into_values()
        .find(|row| {
            row.construct
                .starts_with(&format!("{SUSPENDED}.unrelated_payload/"))
        })
        .expect("new field must be discovered as an opaque payload row");
    let mut entry = entries
        .iter()
        .find(|entry| entry.disposition == Disposition::Unbounded)
        .unwrap()
        .clone();
    entry.row = row.row();
    entry
}

#[test]
fn missing_stale_and_unrelated_expression_debt_are_rejected() {
    let discovered = graph::discover(with_unrelated_payload(source::production()));
    let mut ledger = entries(LEDGER).unwrap();
    let added = unrelated_entry(&discovered, &ledger);
    assert!(
        discovered
            .cycles
            .iter()
            .any(|cycle| cycle.id == "fixture::OtherLink")
    );
    assert!(
        discovered
            .cycles
            .iter()
            .any(|cycle| cycle.id == "fixture::OtherChain")
    );
    let missing = check(&discovered, &ledger).unwrap_err();
    assert_eq!(
        missing,
        format!("check 1: missing ledger entry: {}", added.row)
    );

    ledger.push(added.clone());
    let attribution = check(&discovered, &ledger).unwrap_err();
    assert!(
        attribution.starts_with(
            "check 3: opaque ownership via fixture::OtherPayload has no verified path"
        ),
        "{attribution}"
    );
    assert!(attribution.contains(&added.row));

    let mut restored = discovered.source;
    restored
        .definitions
        .get_mut(SUSPENDED)
        .unwrap()
        .fields
        .retain(|field| field.path != "unrelated_payload");
    let restored = graph::discover(restored);
    let stale = check(&restored, &ledger).unwrap_err();
    assert_eq!(stale, format!("check 2: stale ledger entry: {}", added.row));
    ledger.pop();
    check(&restored, &ledger).unwrap();
}

#[test]
fn metadata_duplicates_and_witnesses_are_enforced() {
    let discovered = graph::discover(source::production());
    let ledger = entries(LEDGER).unwrap();
    let mut duplicate = ledger.clone();
    duplicate.push(ledger[0].clone());
    assert!(
        check(&discovered, &duplicate)
            .unwrap_err()
            .contains("duplicate ledger entry:")
    );
    let row = ledger
        .iter()
        .find(|entry| entry.disposition == Disposition::Unbounded)
        .unwrap();
    let actual = host_rows(&discovered)[&row.row];
    for (debt, coverage) in [("-", "unresolved"), (OWED_BY, "resolved")] {
        let mut bad = row.clone();
        bad.owed_by = debt.into();
        bad.external_implementation_coverage = coverage.into();
        assert!(
            check_entry(&discovered, actual, &bad)
                .unwrap_err()
                .contains("Unbounded requires")
        );
    }
    let mut bad = row.clone();
    bad.witness = "unverified prose".into();
    assert!(
        check_entry(&discovered, actual, &bad)
            .unwrap_err()
            .contains("witness mismatch:")
    );
    bad.disposition = Disposition::PublicInput;
    assert!(
        check_entry(&discovered, actual, &bad)
            .unwrap_err()
            .contains("disposition disagrees")
    );
    assert!(entries(&LEDGER.replacen("\tPublicInput\t", "\tWaived\t", 1)).is_err());
}

fn adapter_fixture(storage: &str, visibility: &str, extra: &str) -> Discovery {
    let fixture = format!(
        "{visibility} trait Bound {{}}\n\
         trait Erased {{}}\n\
         struct Adapter<F: Bound> {{ inner: {storage} }}\n\
         impl<F> Erased for Adapter<F> where F: Bound {{}}\n\
         struct Queue {{ payload: Box<dyn Erased> }}\n{extra}"
    );
    graph::discover(source::fixture_with_roots(
        &[
            ("crates/fixture/src/lib.rs", &fixture),
            (
                "crates/rum-core/src/lib.rs",
                "pub mod expr { pub struct Expr; }",
            ),
        ],
        &["fixture", "rum-core"],
    ))
}

fn adapter_row(discovered: &Discovery) -> &Unknown {
    discovered
        .unknowns
        .iter()
        .find(|row| {
            row.kind == UnknownKind::UnknownHostPayload
                && row.construct.starts_with("fixture::Queue.payload/")
        })
        .unwrap()
}

#[test]
fn adapter_phantom_parameter_cannot_supply_ownership() {
    let positive = adapter_fixture("F", "pub", "");
    assert!(ownership::expression_witness(&positive, adapter_row(&positive)).is_ok());
    let negative = adapter_fixture("std::marker::PhantomData<F>", "pub", "");
    let error = ownership::expression_witness(&negative, adapter_row(&negative)).unwrap_err();
    assert!(error.contains("no verified path"), "{error}");
    println!("PHANTOM ADAPTER REJECTED: {error}");
}

#[test]
fn adapter_private_bound_cannot_supply_interim_witness() {
    for storage in ["F", "Box<F>", "std::sync::Arc<F>"] {
        let positive = adapter_fixture(storage, "pub", "");
        let witness = ownership::expression_witness(&positive, adapter_row(&positive)).unwrap();
        assert!(
            witness.contains("Adapter.inner")
                && witness.contains("public bound; no in-tree instantiation exists yet")
        );
        let negative = adapter_fixture(storage, "", "");
        let error = ownership::expression_witness(&negative, adapter_row(&negative)).unwrap_err();
        assert!(error.contains("no verified path"), "{error}");
        println!("PRIVATE BOUND ({storage}) REJECTED: {error}");
    }
}

#[test]
fn adapter_concrete_instantiation_invalidates_interim_witness() {
    let positive = adapter_fixture("Box<F>", "pub", "");
    let row = adapter_row(&positive);
    let interim = Entry {
        row: row.row(),
        disposition: Disposition::Unbounded,
        witness: ownership::expression_witness(&positive, row).unwrap(),
        owed_by: OWED_BY.into(),
        external_implementation_coverage: "unresolved".into(),
    };
    check_entry(&positive, row, &interim).unwrap();
    let negative = adapter_fixture(
        "Box<F>",
        "pub",
        "struct Concrete { value: rum_core::expr::Expr }\nimpl Bound for Concrete {}",
    );
    let row = adapter_row(&negative);
    let error = check_entry(&negative, row, &interim).unwrap_err();
    assert!(error.starts_with("witness mismatch:"), "{error}");
    assert!(
        error.contains("fixture::Adapter.inner")
            && error.contains("fixture::Concrete.value -> rum_core::expr::Expr"),
        "{error}"
    );
    println!("STALE INTERIM WITNESS REJECTED: {error}");
    let restored = Entry {
        witness: ownership::expression_witness(&negative, row).unwrap(),
        ..interim
    };
    check_entry(&negative, row, &restored).unwrap();
}

#[test]
fn adapter_extra_implementation_obligations_are_not_discarded() {
    let mut input = source::fixture(&[(
        "crates/fixture/src/lib.rs",
        "pub trait Bound {} trait Erased {} struct Adapter<F: Bound> { inner: F } impl<F: Bound> Erased for Adapter<F> {} struct Queue { payload: Box<dyn Erased> }",
    )]);
    let positive = graph::discover(input);
    assert!(ownership::expression_witness(&positive, adapter_row(&positive)).is_ok());
    input = positive.source;
    input.drops[0].generics = syn::parse_quote!(<F: Bound + Clone>);
    let negative = graph::discover(input);
    assert!(ownership::expression_witness(&negative, adapter_row(&negative)).is_err());
}

#[test]
fn adapter_production_witnesses_are_computed() {
    let discovered = graph::discover(source::production());
    let rows: Vec<_> = host_rows(&discovered)
        .into_values()
        .filter(|row| {
            row.construct
                .starts_with("rum_runtime::runtime::suspension::typed::")
                && row.construct.contains(": opaque owned payload dyn Erased")
        })
        .collect();
    assert_eq!(
        rows.iter()
            .map(|row| row.construct.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([
            "rum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame",
            "rum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer",
            "rum_runtime::runtime::suspension::typed::chain::FrameChain.frames/std::vec::Vec[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame",
            "rum_runtime::runtime::suspension::typed::entry::Entry.Typed.0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame",
        ])
    );
    for row in rows {
        let witness = ownership::expression_witness(&discovered, row).unwrap();
        println!("ADAPTER_WITNESS\t{}\t{witness}", row.row());
    }
}

fn blanket_fixture(bound: &str, visibility: &str, implementation: &str, extra: &str) -> Discovery {
    let fixture = format!(
        "{bound}\n\
         {visibility} trait Erased {{}}\n\
         {implementation}\n\
         struct Queue {{ payload: Box<dyn Erased> }}\n{extra}"
    );
    graph::discover(source::fixture_with_roots(
        &[
            ("crates/fixture/src/lib.rs", &fixture),
            (
                "crates/rum-core/src/lib.rs",
                "pub mod expr { pub struct Expr; }",
            ),
        ],
        &["fixture", "rum-core"],
    ))
}

const BLANKET_IMPL: &str = "impl<F: Bound> Erased for F {}";
const BLANKET_CONCRETE: &str =
    "struct Concrete { value: rum_core::expr::Expr }\nimpl Bound for Concrete {}";
const BLANKET_WITNESS: &str = "fixture::Erased => impl<F: fixture::Bound> for F (impl at crates/fixture/src/lib.rs:3:1) => fixture::Bound => fixture::Concrete (impl at crates/fixture/src/lib.rs:6:1); fixture::Concrete.value -> rum_core::expr::Expr";

fn blanket_positive() -> Discovery {
    let positive = blanket_fixture("pub trait Bound {}", "", BLANKET_IMPL, BLANKET_CONCRETE);
    let witness = ownership::expression_witness(&positive, adapter_row(&positive)).unwrap();
    assert_eq!(witness, BLANKET_WITNESS);
    println!("BLANKET POSITIVE: {witness}");
    positive
}

fn blanket_rejected(negative: &Discovery, reason: &str) {
    let error = ownership::expression_witness(negative, adapter_row(negative)).unwrap_err();
    assert!(error.contains("no verified path"), "{error}");
    println!("BLANKET REJECTED ({reason}): {error}");
}

#[test]
fn blanket_concrete_hop_and_implementation_form() {
    let positive = blanket_positive();
    let where_clause = blanket_fixture(
        "pub trait Bound {}",
        "",
        "impl<F> Erased for F where F: Bound {}",
        BLANKET_CONCRETE,
    );
    assert_eq!(
        ownership::expression_witness(&where_clause, adapter_row(&where_clause)).unwrap(),
        BLANKET_WITNESS
    );
    for conditional in [false, true] {
        let mut input =
            blanket_fixture("pub trait Bound {}", "", BLANKET_IMPL, BLANKET_CONCRETE).source;
        input.drops[0].positive = conditional;
        input.drops[0].conditional = conditional;
        let negative = graph::discover(input);
        blanket_rejected(
            &negative,
            if conditional {
                "conditional"
            } else {
                "negative"
            },
        );
    }
    let mut input = positive.source;
    input.drops[0].generics = syn::parse_quote!(<F: Bound, G>);
    blanket_rejected(&graph::discover(input), "two type parameters");
}

#[test]
fn blanket_extra_or_generic_bounds_are_rejected() {
    blanket_positive();
    for (bound, implementation, extra) in [
        (
            "pub trait Bound {}",
            "impl<F: Bound + Clone> Erased for F {}",
            BLANKET_CONCRETE,
        ),
        (
            "pub trait Bound<T> {}",
            "impl<F: Bound<u8>> Erased for F {}",
            "struct Concrete { value: rum_core::expr::Expr }\nimpl Bound<u8> for Concrete {}",
        ),
    ] {
        let negative = blanket_fixture(bound, "", implementation, extra);
        blanket_rejected(&negative, implementation);
    }
}

#[test]
fn blanket_self_must_be_the_bare_parameter() {
    blanket_positive();
    for self_type in ["Box<F>", "&F", "Vec<F>", "crate::F", "<F as Bound>::Item"] {
        let negative = blanket_fixture(
            "pub trait Bound {}",
            "",
            &format!("impl<F: Bound> Erased for {self_type} {{}}"),
            BLANKET_CONCRETE,
        );
        blanket_rejected(&negative, self_type);
    }
}

#[test]
fn blanket_public_erased_trait_is_rejected() {
    blanket_positive();
    let negative = blanket_fixture("pub trait Bound {}", "pub", BLANKET_IMPL, BLANKET_CONCRETE);
    blanket_rejected(&negative, "public erased trait");
}

#[test]
fn blanket_expression_path_and_interim_limits() {
    blanket_positive();
    let private_concrete = blanket_fixture("trait Bound {}", "", BLANKET_IMPL, BLANKET_CONCRETE);
    assert_eq!(
        ownership::expression_witness(&private_concrete, adapter_row(&private_concrete)).unwrap(),
        BLANKET_WITNESS
    );
    let interim = blanket_fixture("pub trait Bound {}", "", BLANKET_IMPL, "");
    assert_eq!(
        ownership::expression_witness(&interim, adapter_row(&interim)).unwrap(),
        "fixture::Erased => impl<F: fixture::Bound> for F (impl at crates/fixture/src/lib.rs:3:1) => fixture::Bound (public bound; no in-tree instantiation exists yet)"
    );
    let private_empty = blanket_fixture("trait Bound {}", "", BLANKET_IMPL, "");
    blanket_rejected(&private_empty, "private bound without implementor");
    for bound in ["trait Bound {}", "pub trait Bound {}"] {
        let negative = blanket_fixture(
            bound,
            "",
            BLANKET_IMPL,
            "struct Concrete { value: u64 }\nimpl Bound for Concrete {}\nstruct Unrelated { value: rum_core::expr::Expr }",
        );
        blanket_rejected(&negative, "implementor lacks expression ownership");
    }
}

#[test]
fn blanket_checks_every_erased_implementor() {
    blanket_positive();
    let supported = blanket_fixture(
        "pub trait Bound {}",
        "",
        BLANKET_IMPL,
        &format!(
            "{BLANKET_CONCRETE}\nstruct Adapter<T: Bound> {{ inner: T }}\nimpl<T: Bound> Erased for Adapter<T> {{}}"
        ),
    );
    assert!(ownership::expression_witness(&supported, adapter_row(&supported)).is_ok());
    for unsupported in [
        "struct Other { value: rum_core::expr::Expr }\nimpl Erased for Other {}",
        "struct Phantom<T> { value: std::marker::PhantomData<T> }\nimpl<T: Bound> Erased for Phantom<T> {}",
        "impl<G: Bound + Clone> Erased for G {}",
    ] {
        let negative = blanket_fixture(
            "pub trait Bound {}",
            "",
            BLANKET_IMPL,
            &format!("{BLANKET_CONCRETE}\n{unsupported}"),
        );
        blanket_rejected(&negative, unsupported);
    }
}
