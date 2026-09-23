use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::tests::retention_closure::source;

#[test]
fn production_rejection_census_is_complete() {
    let source = source::production();
    assert!(source.errors.is_empty(), "{:?}", source.errors);
    let mut census = all(&source);
    let expected = definitions(&source);
    assert_eq!(
        expected.len(),
        source.functions.values().map(Vec::len).sum::<usize>()
    );
    let roots: Vec<_> = expected
        .iter()
        .map(|f| Root {
            name: format!("{}@{}", f.id, f.site.display()),
            site: f.site.clone(),
            target: Ok(f.id.clone()),
            spelling: f.id.clone(),
        })
        .collect();
    validate(&census, &expected, &roots).unwrap();
    assert!(census.visited.values().all(|rs| rs.len() == 1));
    let dir = report::write(&census, &expected, &roots);
    report::independent_audit(&dir);

    let removed = census.visited.pop_first().unwrap();
    assert_eq!(
        validate(&census, &expected, &roots).unwrap_err(),
        "census omitted or invented a production definition"
    );
    census.visited.insert(removed.0, removed.1);
    validate(&census, &expected, &roots).unwrap();
    baseline(&source);
}

#[test]
fn n_frontier_keeps_unresolved_roots() {
    let source = source::production();
    assert!(source.errors.is_empty(), "{:?}", source.errors);
    let roots = n_roots(&source);
    assert_eq!(roots.iter().filter(|r| r.target.is_err()).count(), 9);
    assert_eq!(
        roots
            .iter()
            .map(|r| r.name.split('@').next().unwrap())
            .collect::<BTreeSet<_>>()
            .len(),
        78
    );
    let mut census = frontier(&source, roots.clone());
    // Independently traverse the existing walker for each resolved attachment;
    // the new census's visited set is not its own expected population.
    let mut expected = BTreeSet::new();
    for root in &roots {
        if let Ok(id) = &root.target {
            for id in regions::frontier(&source, &[id.clone()]).bodies {
                let f = resolve::function(&source, &id).unwrap();
                expected.insert(Definition {
                    id,
                    site: f.site.clone(),
                });
            }
        }
    }
    validate(&census, &expected, &roots).unwrap();
    let dir = report::write(&census, &expected, &roots);
    report::independent_audit(&dir);

    let unresolved = roots.iter().find(|r| r.target.is_err()).unwrap();
    let row = census
        .rows
        .iter()
        .position(|r| r.root == unresolved.name)
        .unwrap();
    let removed = census.rows.remove(row);
    assert!(
        validate(&census, &expected, &roots)
            .unwrap_err()
            .contains("root vanished")
    );
    census.rows.insert(row, removed);
    let removed = census.roots.remove(&unresolved.name).unwrap();
    assert_eq!(
        validate(&census, &expected, &roots).unwrap_err(),
        "census omitted or invented a root attachment"
    );
    census.roots.insert(removed.name.clone(), removed);
    validate(&census, &expected, &roots).unwrap();
}

#[test]
fn conditional_ambiguous_and_missing_roots_are_located() {
    let source = source::fixture(&[(
        "probe",
        "probe.rs",
        "#[cfg(feature=\"one\")] fn root() { missing(); } #[cfg(not(feature=\"one\"))] fn root() { other(); } fn unique() {}",
    )]);
    let all = all(&source);
    assert_eq!(all.visited.len(), 3);
    assert_eq!(
        all.rows
            .iter()
            .filter(|r| r.diagnostic.reason.starts_with("ambiguous or conditional"))
            .count(),
        2
    );
    let f = &source.functions["fixture::probe::root"][0];
    let roots = vec![
        Root {
            name: "N:Conditional".into(),
            site: f.site.clone(),
            target: Ok(f.id.clone()),
            spelling: f.id.clone(),
        },
        Root {
            name: "N:Missing".into(),
            site: f.site.clone(),
            target: Ok("fixture::probe::missing".into()),
            spelling: "missing".into(),
        },
    ];
    let census = frontier(&source, roots.clone());
    validate(&census, &BTreeSet::new(), &roots).unwrap();
    assert_eq!(census.rows.len(), 2);
    assert!(
        census
            .rows
            .iter()
            .all(|r| r.diagnostic.site.file == "probe.rs")
    );
}

fn baseline(source: &Source) {
    let options = regions::frontier(
        source,
        &[
            format!("{}::merged_option_exprs_for_symbol", regions::SESSION),
            format!("{}::evaluate_runtime_option_expr", regions::SESSION),
        ],
    );
    // Exactly the three located baseline rows removed by A1. Payload method
    // rejections at options/core.rs:250 and :264 remain in the frontier.
    for (suffix, line, start, end) in [
        ("expr_helpers.rs", 22, 45, 57),
        ("options/core.rs", 250, 20, 36),
        ("options/core.rs", 264, 13, 29),
    ] {
        assert!(
            !options
                .rejected
                .iter()
                .any(|r| r.site.file.ends_with(suffix)
                    && r.site.start == (line, start)
                    && r.site.end == (line, end)),
            "constructor row remains at {suffix}:{line}:{start}-{end}"
        );
    }
    for line in [250, 264] {
        assert!(
            options
                .rejected
                .iter()
                .any(|r| r.site.file.ends_with("options/core.rs")
                    && r.site.start.0 == line
                    && r.reason.starts_with("method syntax"))
        );
    }
    let mut kinds = BTreeMap::new();
    for row in &options.rejected {
        *kinds.entry(row.kind).or_insert(0) += 1;
    }
    assert_eq!(
        options.rejected.len(),
        14,
        "STOP: brief-534 predicted constructor delta moved"
    );
    assert_eq!(
        kinds,
        BTreeMap::from([(Kind::RET001, 1), (Kind::RET002, 12), (Kind::RET003, 1)])
    );
    assert_eq!(
        regions::frontier(
            source,
            &[format!(
                "{}::merged_option_exprs_for_system_symbol",
                regions::SESSION
            )]
        )
        .rejected
        .len(),
        9,
        "STOP: PUBLIC_WRAPPER moved"
    );
    assert!(include_str!("../inventory/promoted.tsv").is_empty());
    println!(
        "ACCEPTANCE_BASELINE OPTION_FRONTIER=14 RET001=1 RET002=12 RET003=1 PUBLIC_WRAPPER=9 PRODUCTION_COVERAGE=0"
    );
}
