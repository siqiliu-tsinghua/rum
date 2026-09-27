//! Acceptance predictions live in target/brief-539/PROGRESS.md. Do not turn an
//! unexpected rejection into an expectation after measuring it.

use super::super::{regions, source};
use super::*;
use std::{fs, time::Instant};

#[test]
fn local_production_acceptance() {
    let discovery = Instant::now();
    let source = source::test_module(
        source::production(),
        "crates/rum-runtime/src/runtime/accounting_mode/tests/mod.rs",
        "rum_runtime::runtime::accounting_mode::tests",
    );
    assert!(source.errors.is_empty(), "{:?}", source.errors);
    let index = Interfaces::new(&source);
    let marks = Marks::contracts(
        contracts::read(include_str!("../inventory/local-contracts.tsv")).unwrap(),
    )
    .unwrap();
    let directory = source::workspace().join("target/brief-539");
    fs::create_dir_all(&directory).unwrap();
    let indexing = discovery.elapsed();
    let started = Instant::now();
    let mut option_reports = Vec::new();
    let mut rows = String::from("body\tpredicted\tmeasured\n");
    for (label, name, line) in [
        ("C01", "merged_option_exprs_for_symbol", 101),
        ("C02", "evaluate_runtime_option_expr", 259),
    ] {
        let f = resolve::function(&source, &format!("{}::{name}", regions::SESSION)).unwrap();
        let report = check(&index, f, &marks);
        fs::write(
            directory.join(format!("{label}-rejections.tsv")),
            report.text(),
        )
        .unwrap();
        rows.push_str(&format!(
            "{label}\treject\t{} rejections\n",
            report.rejected.len()
        ));
        option_reports.push((label, line, report));
    }
    let mut probes = Vec::new();
    for name in ["probe", "start", "drive", "resume"] {
        let f = resolve::function(
            &source,
            &format!("rum_runtime::runtime::accounting_mode::tests::{name}"),
        )
        .unwrap();
        let report = check(&index, f, &marks);
        fs::write(
            directory.join(format!("LeaseProbe538-{name}-rejections.tsv")),
            report.text(),
        )
        .unwrap();
        rows.push_str(&format!(
            "LeaseProbe538::{name}\tclean\t{} rejections\n",
            report.rejected.len()
        ));
        probes.push(report);
    }
    rows.push_str("production coverage\t0\t0\n");
    fs::write(directory.join("acceptance.tsv"), &rows).unwrap();
    let cost = format!(
        "LOCAL_INDEX_SECONDS={:.6}\nLOCAL_SIX_BODY_SECONDS={:.6}\nFIXTURE_MARKS_IN_PRODUCTION_RUN={}\nPRODUCTION_COVERAGE=0\n",
        indexing.as_secs_f64(),
        started.elapsed().as_secs_f64(),
        marks.count()
    );
    fs::write(directory.join("local-cost.txt"), &cost).unwrap();
    print!("{rows}{cost}");
    assert!(include_str!("../inventory/promoted.tsv").is_empty());
    for (label, line, report) in option_reports {
        assert!(
            report
                .rejected
                .iter()
                .any(|d| d.kind == Kind::RET001 && d.site.start.0 == line),
            "{label}: missing raw construction at {line}\n{}",
            report.text()
        );
    }
    assert!(
        probes.iter().all(|r| r.rejected.is_empty()),
        "LeaseProbe538 was predicted clean, but its current syntax requires forbidden new admissions; see saved located reports. Do not weaken this acceptance check."
    );
}
