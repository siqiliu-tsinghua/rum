//! The TSV owns dispositions; Rust declarations and catalog routes detect drift.

use std::collections::{BTreeMap, BTreeSet};

use syn::parse::Parser;

use super::model::*;

const DECLARATIONS: &str = "crates/rum-stdlib/src/catalog/memory_growth_contract.rs";

pub(super) fn census(source: &Source) -> BTreeMap<String, String> {
    let parsed = syn::parse_file(&source.files[DECLARATIONS]).unwrap();
    let mut declarations = BTreeMap::new();
    for item in parsed.items {
        let syn::Item::Macro(item) = item else {
            continue;
        };
        if !item
            .mac
            .path
            .is_ident("installed_builtin_memory_growth_inventory")
        {
            continue;
        }
        let parser = |input: syn::parse::ParseStream<'_>| {
            while !input.is_empty() {
                let attrs = input.call(syn::Attribute::parse_outer)?;
                let name: syn::LitStr = input.parse()?;
                input.parse::<syn::Token![=>]>()?;
                let growth: syn::Ident = input.parse()?;
                input.parse::<syn::Token![,]>()?;
                assert!(
                    declarations
                        .insert(name.value(), (growth.to_string(), attrs))
                        .is_none()
                );
            }
            Ok(())
        };
        parser.parse2(item.mac.tokens).unwrap();
    }
    assert!(!declarations.is_empty(), "live declaration macro missing");
    // Discovery retains all cfg alternatives. This comparison separately checks
    // the actual installed configuration; only the catalog's exact hash feature
    // is understood here, and an unfamiliar condition is an error.
    let active: BTreeMap<_, _> = declarations
        .iter()
        .filter_map(|(name, (growth, attrs))| {
            let enabled = match attrs.as_slice() {
                [] => true,
                [attr] if tokens(&attr.meta) == "cfg (feature = \"hash\")" => {
                    cfg!(feature = "hash")
                }
                _ => panic!("unknown declaration cfg for {name}"),
            };
            enabled.then(|| (name.clone(), growth.clone()))
        })
        .collect();
    let mut compiled = BTreeMap::new();
    for spec in rum_runtime::core_builtin_specs()
        .iter()
        .chain(crate::tests::stdlib_catalog_specs())
    {
        let value = format!("{:?}", spec.memory_growth);
        if let Some(previous) = compiled.insert(spec.name.to_string(), value.clone()) {
            assert_eq!(
                previous, value,
                "conflicting live declaration for {}",
                spec.name
            );
        }
    }
    assert_eq!(
        active, compiled,
        "source declarations versus compiled catalogs"
    );

    // Keep the director's explicit type annotation: assignments come entirely
    // from this artifact, never from parsing a narrative document.
    let mut classes: BTreeMap<String, String> = BTreeMap::new();
    for line in include_str!("inventory/obligations.tsv")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let row: Vec<_> = line.split('\t').collect();
        assert_eq!(row.len(), 8, "complete obligation record: {line}");
        if row[2] == "Candidate" {
            continue;
        }
        assert!(matches!(row[2], "N" | "Unsplit" | "Unsettled"));
        assert!(classes.insert(row[1].into(), row[2].into()).is_none());
        let (growth, _) = declarations
            .get(row[1])
            .expect("obligation has live declaration");
        assert!(row[4] == DECLARATIONS || row[4].starts_with(&format!("{DECLARATIONS}:")));
        assert!(
            !row[5].is_empty() && !row[6].is_empty(),
            "quantity and evidence required"
        );
        if row[2] == "Unsplit" {
            assert_eq!(growth, "Incremental");
        }
    }
    for (class, count) in [("N", 78), ("Unsplit", 147), ("Unsettled", 90)] {
        assert_eq!(
            classes.values().filter(|c| c.as_str() == class).count(),
            count,
            "{class}"
        );
    }
    // The fifteen previously distinguished projectable rows are dispositions,
    // not covered retention regions. Preserve them to catch new Incremental
    // declarations instead of silently excluding an expanding residual set.
    let mut projectable = BTreeSet::new();
    for line in include_str!("inventory/projectable.tsv")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let row: Vec<_> = line.split('\t').collect();
        assert_eq!(row.len(), 3);
        assert_eq!(row[1], "ProjectablePending");
        assert!(!row[2].is_empty());
        assert!(!classes.contains_key(row[0]));
        assert!(projectable.insert(row[0].to_string()));
    }
    assert_eq!(projectable.len(), 15);
    let residual: BTreeSet<_> = declarations
        .iter()
        .filter(|(name, (growth, _))| growth == "Incremental" && !classes.contains_key(*name))
        .map(|(name, _)| name.clone())
        .collect();
    assert_eq!(
        residual, projectable,
        "every live Incremental disposition is accounted for"
    );
    classes
}
