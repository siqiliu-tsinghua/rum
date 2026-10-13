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
    assert_eq!(rows.len(), 3);
    for row in rows {
        let witness = ownership::expression_witness(&discovered, row).unwrap();
        println!("ADAPTER_WITNESS\t{}\t{witness}", row.row());
    }
}
