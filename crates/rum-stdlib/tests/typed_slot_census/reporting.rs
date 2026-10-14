//! Source fingerprints and deterministic census report emission.
use super::{
    classify::{self, Report},
    discovery::feature,
    receipts::{RECEIPTS, SHAPES, receipt_id, rows, use_identity},
    scan::{self, Facts, Population},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

fn hashes(root: &Path) -> BTreeMap<String, String> {
    let output=Command::new("python3").arg(root.join("scripts/run-bounded.py")).args(["--timeout","900","--","python3","-c",
        "from pathlib import Path; import hashlib,sys; r=Path(sys.argv[1]); [(print(str(p.relative_to(r))+'\\t'+hashlib.sha256(p.read_bytes()).hexdigest())) for p in sorted((r/'crates').rglob('*.rs'))]"]).arg(root).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let (p, h) = line.split_once('\t').unwrap();
            (p.into(), h.into())
        })
        .collect()
}
fn save(dir: &Path, name: &str, header: &str, rows: impl IntoIterator<Item = String>) {
    fs::create_dir_all(dir).unwrap();
    let mut rows: Vec<_> = rows.into_iter().collect();
    rows.sort();
    fs::write(dir.join(name), format!("{header}\n{}\n", rows.join("\n"))).unwrap();
}

pub(super) fn emit(root: &Path, facts: &Facts, report: &Report) {
    let dir = root.join(format!(".temp/brief-608/census/{}", feature()));
    let hashes = hashes(root);
    let hash = |site: &scan::Site| {
        hashes
            .get(&site.file)
            .map(String::as_str)
            .unwrap_or("MISSING")
    };
    save(
        &dir,
        "frames.tsv",
        "id\tidentity\tkind\tinput\toutput\tfile\tline\tcolumn\tend_line\tend_column\tsha256\tname\tpopulation",
        report.frames.iter().map(|f| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                receipt_id("F", &f.identity),
                f.identity,
                f.kind,
                f.input,
                f.output,
                f.site.fields(),
                hash(&f.site),
                f.name,
                f.population.label()
            )
        }),
    );
    save(
        &dir,
        "definitions.tsv",
        "id\tidentity\towner\ttrait\tname\tpopulation\tfile\tline\tcolumn\tend_line\tend_column\tsha256\tsignature",
        facts.functions.iter().map(|f| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                receipt_id("Decoder", &f.identity),
                f.identity,
                f.owner,
                f.trait_name,
                f.name,
                f.population.label(),
                f.site.fields(),
                hash(&f.site),
                f.signature
            )
        }),
    );
    save(
        &dir,
        "macro-sites.tsv",
        "id\tidentity\tpopulation\tfile\tline\tcolumn\tend_line\tend_column\tsha256\tarm",
        facts.uses.iter().filter(|s| s.kind == "macro").map(|s| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                receipt_id("M", &use_identity(s)),
                use_identity(s),
                s.population.label(),
                s.site.fields(),
                hash(&s.site),
                s.receiver
            )
        }),
    );
    let decoder_names: BTreeSet<_> = rows(RECEIPTS)
        .into_iter()
        .filter(|r| r[0] == "Decoder")
        .filter_map(|r| r[4].rsplit_once("::").map(|(_, name)| name.to_owned()))
        .collect();
    let relevant = |s: &scan::Use| {
        classify::boundary_name(&s.name)
            || decoder_names.contains(&s.name)
            || s.shadowed
            || s.name == "<indirect>"
    };
    save(
        &dir,
        "live-calls.tsv",
        "id\tcaller\tkind\tname\ttarget\treceiver\tresolution\tresolved_definitions\tfile\tline\tcolumn\tend_line\tend_column\tsha256",
        report.calls.iter().filter(|c| relevant(&c.site)).map(|c| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                receipt_id(
                    if c.site.name == "insert_suspension_frame" {
                        "I"
                    } else {
                        "C"
                    },
                    &use_identity(&c.site)
                ),
                c.site.caller,
                c.site.kind,
                c.site.name,
                c.site.target,
                c.site.receiver,
                c.reason,
                c.targets.join(";"),
                c.site.site.fields(),
                hash(&c.site.site)
            )
        }),
    );
    let unresolved = report.unresolved.iter().cloned().chain(
        report
            .calls
            .iter()
            .filter(|c| relevant(&c.site) && c.targets.len() != 1)
            .map(|c| {
                format!(
                    "{}\t{}\t{}\t{}",
                    c.site.site.fields(),
                    c.site.caller,
                    c.site.target,
                    c.reason
                )
            }),
    );
    save(
        &dir,
        "unresolved.tsv",
        "source_site\tcaller_or_kind\ttarget\treason",
        unresolved,
    );
    save(
        &dir,
        "wire-provider-closure.tsv",
        "provider_root\tdefinition_or_target\tclassification\tfile\tline\tcolumn\tend_line\tend_column",
        report.closure.clone(),
    );
    save(
        &dir,
        "typed-registry.tsv",
        "id\tslot\tmode\tfile\tline\tcolumn\tend_line\tend_column\tsha256\tpopulation",
        report.registry.iter().map(|r| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                classify::stable("T", &r.slot),
                r.slot,
                r.mode,
                r.site.fields(),
                hash(&r.site),
                r.population.label()
            )
        }),
    );
    save(
        &dir,
        "heads.tsv",
        "id\tcaller\tpopulation\thead\tfile\tline\tcolumn\tend_line\tend_column\tsha256",
        facts.uses.iter().filter(|s| s.kind == "head").map(|s| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                classify::stable("H", &use_identity(s)),
                s.caller,
                s.population.label(),
                s.name,
                s.site.fields(),
                hash(&s.site)
            )
        }),
    );
    save(
        &dir,
        "legacy-test-users.tsv",
        "identity\tfile\tline\tcolumn\tend_line\tend_column\tkind\tname",
        facts
            .uses
            .iter()
            .filter(|s| {
                s.population == Population::TestSupport
                    && (classify::legacy_name(&s.name)
                        || s.name == "resume_or_suspend"
                        || s.name == "insert_suspension_frame")
            })
            .map(|s| {
                format!(
                    "{}\t{}\t{}\t{}",
                    use_identity(s),
                    s.site.fields(),
                    s.kind,
                    s.name
                )
            }),
    );
    save(
        &dir,
        "all-uses.tsv",
        "identity\tcaller\tkind\tname\ttarget\tpopulation\tfile\tline\tcolumn\tend_line\tend_column\tsha256",
        facts.uses.iter().map(|s| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                use_identity(s),
                s.caller,
                s.kind,
                s.name,
                s.target,
                s.population.label(),
                s.site.fields(),
                hash(&s.site)
            )
        }),
    );
    let receipts = rows(RECEIPTS);
    save(
        &dir,
        "legacy-types.tsv",
        "slot\tdecoder_id\tdefinition\tpopulation\tregistered_mode\tresolved_uses\tunresolved_same_name_candidates",
        rows(SHAPES)
            .into_iter()
            .filter(|r| r[2] == "Legacy")
            .filter_map(|shape| {
                let decoder = receipts
                    .iter()
                    .find(|r| r[0] == "Decoder" && r[1] == shape[5])?;
                let function = facts.functions.iter().find(|f| f.identity == decoder[4])?;
                let mode = report
                    .registry
                    .iter()
                    .find(|r| r.slot == shape[1])
                    .map(|r| r.mode.as_str())
                    .unwrap_or("unregistered until S33");
                let resolved = report
                    .calls
                    .iter()
                    .filter(|c| c.targets.iter().any(|t| t == decoder[4]))
                    .count();
                let unresolved = report
                    .calls
                    .iter()
                    .filter(|c| c.site.name == function.name && c.targets.len() != 1)
                    .count();
                Some(format!(
                    "{}\t{}\t{}\t{}\t{mode}\t{resolved}\t{unresolved}",
                    shape[1],
                    decoder[1],
                    function.identity,
                    function.population.label()
                ))
            }),
    );
    let by_use: BTreeMap<_, _> = facts.uses.iter().map(|s| (use_identity(s), s)).collect();
    for (kind, file) in [
        ("Whole", "whole-slot-consumers.tsv"),
        ("Nested", "nested-decoder-uses.tsv"),
    ] {
        save(
            &dir,
            file,
            "id\tidentity\tdecoder_ids\tkind\tpopulation\tfile\tline\tcolumn\tend_line\tend_column\tsha256",
            receipts.iter().filter(|r| r[0] == kind).map(|r| {
                let site = by_use
                    .get(r[4])
                    .expect("recorded decoder use must be rediscovered");
                format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    r[1],
                    r[4],
                    r[8],
                    site.kind,
                    site.population.label(),
                    site.site.fields(),
                    hash(&site.site)
                )
            }),
        );
    }
    fs::write(dir.join("transition-receipt.tsv"), RECEIPTS).unwrap();
}
