//! Stable TSV artifacts plus a compact frequency table. Full reasons stay in
//! rows.tsv; the reason table groups only variable path/identity suffixes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::{Census, Definition, Key, Root, Row};
use crate::tests::retention_closure::{model::Kind, source};

pub(super) fn class(reason: &str) -> &str {
    if reason.starts_with("path ") && reason.ends_with(" exact bindings") {
        "path <path> has <n> exact bindings"
    } else if reason.starts_with("not a resolved function definition:") {
        "not a resolved function definition: <target>"
    } else if reason.starts_with("ambiguous or conditional function ") {
        "ambiguous or conditional function <target>"
    } else {
        reason
    }
}

fn table_line(out: &mut String, fields: &[String]) {
    assert!(fields.iter().all(|s| !s.contains(['\n', '\r', '\t'])));
    writeln!(out, "{}", fields.join("\t")).unwrap();
}

fn row_fields(population: &str, row: &Row, root: String, witness: String) -> Vec<String> {
    let d = &row.diagnostic;
    vec![
        population.into(),
        root,
        row.definition.clone(),
        format!("{:?}", d.kind),
        d.site.file.clone(),
        d.site.start.0.to_string(),
        d.site.start.1.to_string(),
        d.site.end.0.to_string(),
        d.site.end.1.to_string(),
        d.target.clone().unwrap_or_else(|| "-".into()),
        witness,
        d.reason.clone(),
        class(&d.reason).into(),
    ]
}

const HEADER: &str = "population\troots\tdefinition\tkind\tfile\tstart_line\tstart_column\tend_line\tend_column\ttarget\twitness\treason\tclass\n";

pub(super) fn write(report: &Census, expected: &BTreeSet<Definition>, roots: &[Root]) -> PathBuf {
    let dir = source::workspace()
        .join("target/brief-533")
        .join(report.population);
    fs::create_dir_all(&dir).unwrap();
    let mut rows = report.rows.clone();
    rows.sort();
    let mut raw = HEADER.to_string();
    let mut grouped: BTreeMap<Key, Vec<&Row>> = BTreeMap::new();
    for row in &rows {
        table_line(
            &mut raw,
            &row_fields(
                report.population,
                row,
                row.root.clone(),
                row.diagnostic.witness.join(" -> "),
            ),
        );
        grouped.entry(row.key()).or_default().push(row);
    }
    fs::write(dir.join("rows.tsv"), raw).unwrap();
    let mut dedup = HEADER.to_string();
    let mut counts: BTreeMap<(Kind, String), (usize, usize)> = BTreeMap::new();
    let mut exact: BTreeMap<(Kind, String), (usize, usize)> = BTreeMap::new();
    for group in grouped.values() {
        let row = group[0];
        let origins: BTreeSet<_> = group.iter().map(|r| r.root.as_str()).collect();
        let witnesses: BTreeSet<_> = group
            .iter()
            .map(|r| format!("{}: {}", r.root, r.diagnostic.witness.join(" -> ")))
            .collect();
        table_line(
            &mut dedup,
            &row_fields(
                report.population,
                row,
                origins.into_iter().collect::<Vec<_>>().join(" | "),
                witnesses.into_iter().collect::<Vec<_>>().join(" | "),
            ),
        );
        for (map, reason) in [
            (&mut counts, class(&row.diagnostic.reason).to_string()),
            (&mut exact, row.diagnostic.reason.clone()),
        ] {
            let value = map.entry((row.diagnostic.kind, reason)).or_default();
            value.0 += group.len();
            value.1 += 1;
        }
    }
    fs::write(dir.join("deduplicated.tsv"), dedup).unwrap();
    let reason_table = |counts: BTreeMap<(Kind, String), (usize, usize)>| {
        let mut entries: Vec<_> = counts.into_iter().collect();
        entries.sort_by(|a, b| b.1.1.cmp(&a.1.1).then_with(|| a.0.cmp(&b.0)));
        let mut out = "population\tkind\traw\tdeduplicated\treason\n".to_string();
        for ((kind, reason), (raw, unique)) in entries {
            table_line(
                &mut out,
                &[
                    report.population.into(),
                    format!("{kind:?}"),
                    raw.to_string(),
                    unique.to_string(),
                    reason,
                ],
            );
        }
        out
    };
    let summary = reason_table(counts);
    fs::write(dir.join("reason-table.tsv"), &summary).unwrap();
    fs::write(dir.join("exact-reasons.tsv"), reason_table(exact)).unwrap();
    let mut manifest = "definition\tspan\n".to_string();
    for f in expected {
        table_line(&mut manifest, &[f.id.clone(), f.site.display()]);
    }
    fs::write(dir.join("expected-definitions.tsv"), manifest).unwrap();
    let mut visited = "definition\tspan\troots\n".to_string();
    for (f, roots) in &report.visited {
        table_line(
            &mut visited,
            &[
                f.id.clone(),
                f.site.display(),
                roots.iter().cloned().collect::<Vec<_>>().join(" | "),
            ],
        );
    }
    fs::write(dir.join("visited.tsv"), visited).unwrap();
    let mut attachments = "root\tspan\ttarget\treason\n".to_string();
    for root in roots {
        table_line(
            &mut attachments,
            &[
                root.name.clone(),
                root.site.display(),
                root.target
                    .clone()
                    .unwrap_or_else(|_| root.spelling.clone()),
                root.target.as_ref().err().cloned().unwrap_or_default(),
            ],
        );
    }
    fs::write(dir.join("roots.tsv"), attachments).unwrap();
    let mut per_root: BTreeMap<&String, (usize, usize, usize)> = BTreeMap::new();
    for group in grouped.values() {
        for row in group {
            per_root.entry(&row.root).or_default().0 += 1;
        }
        for root in group.iter().map(|r| &r.root).collect::<BTreeSet<_>>() {
            per_root.entry(root).or_default().1 += 1;
        }
    }
    for roots in report.visited.values() {
        for root in roots {
            per_root.entry(root).or_default().2 += 1;
        }
    }
    let mut incidence = "root\traw\tdeduplicated\tdefinitions\n".to_string();
    for root in roots {
        let (raw, unique, bodies) = per_root.get(&root.name).copied().unwrap_or_default();
        table_line(
            &mut incidence,
            &[
                root.name.clone(),
                raw.to_string(),
                unique.to_string(),
                bodies.to_string(),
            ],
        );
    }
    fs::write(dir.join("root-incidence.tsv"), incidence).unwrap();
    let mut first = HEADER.to_string();
    for row in &report.first {
        table_line(
            &mut first,
            &row_fields(
                report.population,
                row,
                row.root.clone(),
                row.diagnostic.witness.join(" -> "),
            ),
        );
    }
    fs::write(dir.join("first-failure.tsv"), first).unwrap();
    fs::write(
        dir.join("first-ok.txt"),
        report
            .first_ok
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    let mut edges = "definition\tspan\ttarget\n".to_string();
    for edge in &report.resolved {
        table_line(
            &mut edges,
            &[
                edge.definition.clone(),
                edge.site.display(),
                edge.target.clone(),
            ],
        );
    }
    fs::write(dir.join("resolved.tsv"), edges).unwrap();
    let totals = format!(
        "{} roots={} definitions={} raw={} deduplicated={} resolved_sites={}\nFIRST_FAILURE {} rejected={} returned_ok={} (effects::check per root; not edge frequency or production coverage)\n",
        report.population,
        report.roots.len(),
        report.visited.len(),
        rows.len(),
        grouped.len(),
        report.resolved.len(),
        report.population,
        report.first.len(),
        report.first_ok.len()
    );
    fs::write(dir.join("totals.txt"), &totals).unwrap();
    // Print once so concurrent test output cannot split a reason table.
    println!("{totals}{summary}");
    dir
}

pub(super) fn independent_audit(dir: &std::path::Path) {
    let workspace = source::workspace();
    let output = Command::new("python3")
        .arg(workspace.join("scripts/run-bounded.py"))
        .args(["--timeout", "120", "--", "python3"])
        .arg(workspace.join("crates/rum-stdlib/src/tests/retention_closure/census/audit.py"))
        .arg(&workspace)
        .arg(dir)
        .output()
        .unwrap();
    fs::write(
        dir.join("independent-audit.log"),
        [&output.stdout[..], &output.stderr[..]].concat(),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "independent census audit: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    print!("{}", String::from_utf8_lossy(&output.stdout));
}
