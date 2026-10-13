//! Reviewed data: tests never regenerate these rows from live registration.
use super::catalog;
use std::collections::{BTreeMap, BTreeSet};

pub fn rows(text: &str, width: usize) -> Vec<Vec<&str>> {
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), width, "{line}");
            assert!(fields.iter().all(|f| !f.is_empty()));
            fields
        })
        .collect()
}
pub fn active(feature: &str) -> bool {
    match feature {
        "always" => true,
        "hash" => cfg!(feature = "hash"),
        _ => panic!("undeclared feature predicate {feature}"),
    }
}
pub fn catalog_rows() -> Vec<Vec<&'static str>> {
    rows(include_str!("inventory/catalog.tsv"), 11)
}
pub fn entries() -> Vec<Vec<&'static str>> {
    let mut entries = rows(include_str!("inventory/effects.tsv"), 11);
    entries.extend(rows(include_str!("inventory/shared-effects.tsv"), 11));
    entries
}

#[test]
fn installed_catalog_and_descriptor_coverage_are_exact() {
    let mut session = rum_runtime::Session::new();
    let preinstalled: BTreeSet<_> = session
        .registered_builtin_specs()
        .map(|s| format!("System`{}", s.name))
        .collect();
    rum_stdlib::install_core(&mut session);
    let installed: BTreeMap<_, _> = session
        .registered_builtin_specs()
        .map(|s| (format!("System`{}", s.name), s))
        .collect();
    let all = catalog_rows();
    let manifest: Vec<_> = all.iter().filter(|r| active(r[1])).collect();
    let names: Vec<_> = manifest.iter().map(|r| r[0].to_owned()).collect();
    catalog::reconcile(&installed.keys().cloned().collect::<Vec<_>>(), &names, true).unwrap();
    let mut all_names = BTreeSet::new();
    for row in &all {
        assert!(all_names.insert(row[0]), "duplicate feature row {}", row[0]);
    }
    let mut referred = BTreeSet::new();
    let mut guest = 0;
    for row in &manifest {
        let spec = installed[row[0]];
        assert_eq!(format!("{:?}", spec.dispatch), row[3], "{}", row[0]);
        assert_eq!(row[4], "Evaluator.callPreparation");
        assert_eq!(
            preinstalled.contains(row[0]),
            matches!(row[2], "core" | "override-core"),
            "origin {}",
            row[0]
        );
        assert!(matches!(row[2], "core" | "override-core" | "stdlib"));
        match row[8] {
            "GuestEntries" => {
                guest += 1;
                assert_ne!(row[9], "-");
                for id in row[9].split(',') {
                    assert!(referred.insert(id));
                }
            }
            "NoLocalGuestEffect" => {
                assert_eq!(row[9], "-");
                assert!(row[10].len() > 30);
            }
            other => panic!("unknown disposition {other}"),
        }
    }
    for row in rows(include_str!("inventory/shared-effects.tsv"), 11) {
        assert_eq!(row[1], "@evaluator");
        assert!(referred.insert(row[0]));
    }
    assert!(referred.contains("Evaluator.callPreparation"));
    let entries = entries();
    let entries: Vec<_> = entries.iter().filter(|r| active(r[10])).collect();
    let declared: BTreeSet<String> = entries.iter().map(|r| r[0].into()).collect();
    assert_eq!(declared.len(), entries.len(), "duplicate entry ID");
    assert_eq!(referred, declared.iter().map(String::as_str).collect());
    let mut covered = BTreeSet::new();
    let mut descriptors = BTreeSet::new();
    let mut boundary = 0;
    let registry = super::fixtures::primary::descriptors();
    assert_eq!(
        registry.len(),
        registry.iter().map(|d| d.id).collect::<BTreeSet<_>>().len(),
        "duplicate descriptor ID"
    );
    let mut lifecycle = 0;
    for row in &entries {
        assert!(matches!(row[5], "Frame" | "Tail" | "Boundary"));
        if row[5] == "Boundary" {
            boundary += 1;
        }
        super::coverage::source::owner_exists(row[3], row[4]);
        assert!(row[7].contains("Middle") && row[7].contains("Twice"));
        if row[6] != "-" {
            covered.insert(row[0].to_owned());
            for id in row[6].split(',') {
                let descriptor = registry
                    .iter()
                    .find(|d| d.id == id)
                    .unwrap_or_else(|| panic!("unknown descriptor {id}"));
                assert_eq!(
                    descriptor.entry, row[0],
                    "descriptor attached to wrong role"
                );
                let required: BTreeSet<_> = row[7]
                    .split(',')
                    .map(|v| v.trim_start_matches("Retired").to_owned())
                    .collect();
                let actual: BTreeSet<_> = descriptor
                    .variants
                    .iter()
                    .map(|v| format!("{v:?}"))
                    .collect();
                assert_eq!(required, actual, "descriptor variants for {id}");
                assert_eq!(descriptor.boundary, row[5] == "Boundary");
                if descriptors.insert(id) {
                    lifecycle += super::fixtures::primary::execute(descriptor);
                }
            }
        }
    }
    catalog::reconcile_coverage(&declared, &covered, true).unwrap();
    assert_eq!(
        descriptors,
        registry.iter().map(|d| d.id).collect(),
        "unattached executable descriptor"
    );
    println!("LIFECYCLE obligations={lifecycle} (plus completion for every variant)");
    println!(
        "CATALOG hash={} installed={} manifest={} GuestEntries={} NoLocalGuestEffect={} Boundary={} entries={} descriptors={} historical=209 covered={}",
        cfg!(feature = "hash"),
        installed.len(),
        names.len(),
        guest,
        names.len() - guest,
        boundary,
        entries.len(),
        descriptors.len(),
        covered.len()
    );
}
