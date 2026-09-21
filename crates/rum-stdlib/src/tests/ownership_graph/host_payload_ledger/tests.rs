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
