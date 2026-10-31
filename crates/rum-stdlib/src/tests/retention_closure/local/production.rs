//! Two local production bodies, with finite instance certificates attached.
use super::super::source;
use super::*;
use std::fs;

#[test]
fn local_production_acceptance() {
    let shared = promotion::shared_population();
    let source = &shared.source;
    assert!(source.errors.is_empty(), "{:?}", source.errors);
    let index = &shared.index;
    let marks = promotion::earn(source, index);
    let directory = source::workspace().join(".temp/brief-546/certificates");
    fs::create_dir_all(&directory).unwrap();
    let mut rows = String::from("body\tinstance\trejections\n");
    for (label, id) in certificates::roots() {
        let body = resolve::function(&source, &id).unwrap();
        for (instance, body) in certificates::instances(&index, body).unwrap() {
            let report = check_instance(&index, &body, &marks);
            rows.push_str(&format!(
                "{label}\t{};M={};R={};hash={}\t{}\n",
                instance.key,
                instance.mode,
                instance.reentry,
                cfg!(feature = "hash"),
                report.rejected.len()
            ));
            assert!(report.rejected.is_empty(), "{}", report.text());
        }
    }
    for name in ["probe", "start", "drive", "resume"] {
        let id = format!("rum_runtime::runtime::accounting_mode::tests::{name}");
        let report = check(&index, resolve::function(&source, &id).unwrap(), &marks);
        fs::write(
            directory.join(format!(
                "LeaseProbe538-{name}-hash-{}.tsv",
                cfg!(feature = "hash")
            )),
            report.text(),
        )
        .unwrap();
        rows.push_str(&format!(
            "LeaseProbe538::{name}\tsealed modes\t{}\n",
            report.rejected.len()
        ));
        assert!(report.rejected.is_empty(), "{}", report.text());
        assert!(
            index.earned.lookup(&index, &id).is_ok(),
            "probe must earn its own mark"
        );
    }
    assert_eq!(marks.count(), 3, "two C01 instances and one nongeneric C02");
    assert_eq!(index.earned.count(), 4, "probe bodies stay separate");
    assert!(
        marks.lookup(&index, &option_boundaries::c01()).is_err(),
        "no universal all-R mark"
    );
    rows.push_str("PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0\n");
    let cells = promotion::expected(&index, &marks).unwrap();
    let mut review = promotion::HEADER.to_string();
    for (label, id) in certificates::roots() {
        review.push_str(&format!(
            "{label}\tLocalCovered\t{id}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            resolve::function(&source, &id).unwrap().site.display(),
            if cfg!(feature = "hash") {
                &cells[&label]
            } else {
                "UNRECORDED"
            },
            if cfg!(feature = "hash") {
                "UNRECORDED"
            } else {
                &cells[&label]
            },
            certificates::PRECONDITIONS,
            certificates::STRENGTH,
            certificates::EXCLUSIONS
        ));
    }
    fs::write(
        directory.join(format!("promoted-hash-{}.tsv", cfg!(feature = "hash"))),
        review,
    )
    .unwrap();
    fs::write(
        directory.join(format!("acceptance-hash-{}.tsv", cfg!(feature = "hash"))),
        &rows,
    )
    .unwrap();
    print!("{rows}");
    promotion::reconcile(
        &source,
        &index,
        &marks,
        promotion::INVENTORY,
        promotion::OBLIGATIONS,
        promotion::CONSUMERS,
    )
    .unwrap();
}
