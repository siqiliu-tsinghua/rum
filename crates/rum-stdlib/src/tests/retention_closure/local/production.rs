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
    for (label, name) in [
        ("C01", "merged_option_exprs_for_symbol"),
        ("C02", "evaluate_runtime_option_expr"),
    ] {
        let f = resolve::function(&source, &format!("{}::{name}", regions::SESSION)).unwrap();
        let report = check(&index, f, &marks);
        fs::write(
            directory.join(format!("{label}-rejections.tsv")),
            report.text(),
        )
        .unwrap();
        rows.push_str(&format!(
            "{label}\tclean within reconciled domain\t{} rejections\n",
            report.rejected.len()
        ));
        if label == "C01" {
            let domain = index.option_domain.as_ref().unwrap();
            for mode in types::MODES {
                for descriptor in domain {
                    let instance = option_domain::specialize(
                        &index.signatures,
                        f,
                        &[protocol::nominal(mode), descriptor.clone()],
                        domain,
                    )
                    .unwrap();
                    let concrete = check_instance(&index, &instance, &marks);
                    rows.push_str(&format!(
                        "C01 M={mode} R={} hash={}\tclean\t{} rejections\n",
                        descriptor.spelling(),
                        cfg!(feature = "hash"),
                        concrete.rejected.len()
                    ));
                    assert!(concrete.rejected.is_empty(), "{}", concrete.text());
                }
            }
        }
        option_reports.push((label, report));
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
    for (label, report) in option_reports {
        assert_eq!(
            report.rejected.len(),
            0,
            "brief-544 predeclared {label} prediction; raw-container negative fixtures remain required\n{}",
            report.text()
        );
    }
    assert!(
        probes.iter().all(|r| r.rejected.is_empty()),
        "LeaseProbe538 was predicted clean, but its current syntax requires forbidden new admissions; see saved located reports. Do not weaken this acceptance check."
    );
}
