//! Exhaustive A2 claims; no sampling. Unique deprecation markers make rustc
//! name the selected method at its exact token, even across crate boundaries.

use super::super::{fixtures, methods};
use super::{cache, source};
use std::{fmt::Write, fs, process::Command};

#[test]
fn inherent_method_claims_match_rustc() {
    let workspace = source::workspace();
    let dir = workspace.join("target/brief-535/oracle-methods");
    fs::create_dir_all(&dir).unwrap();
    let mut out = "name\tpopulation\tdefinition\tinstance\tfile\tstart_line\tstart_column\tend_line\tend_column\tcaller_line\tcaller_column\ttarget\ttarget_file\ttarget_line\ttarget_column\tmethod\treceiver\tprovenance\tderefs\tautoref\torder\texclusions\tsignature_line\tsignature_column\tsignature_prefix\n".to_string();
    let mut total = 0;
    for (population, source) in [
        ("fixture", fixtures::program()),
        ("production", source::production().to_owned()),
    ] {
        assert!(source.errors.is_empty(), "{:?}", source.errors);
        let claims = methods::all(&source);
        println!("METHOD_CLAIMS {population}={}", claims.len());
        for ((_, instance), (caller, selection)) in claims {
            let site = &selection.method_site;
            let f = &selection.function;
            let paren = f.sig().paren_token.span.close().start();
            let prefix = if f.sig().inputs.is_empty() || f.sig().inputs.trailing_punct() {
                ""
            } else {
                ", "
            };
            writeln!(out, "method_{total}\t{population}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{}\t{}\t{}",
                caller.id, instance.display(), site.file, site.start.0, site.start.1, site.end.0, site.end.1,
                caller.site.start.0, caller.site.start.1, f.id, f.site.file, f.site.start.0, f.site.start.1,
                f.sig().ident, selection.receiver.exact.spelling(),
                format!("{}:{}:{}", selection.receiver.provenance.source.display(), selection.receiver.provenance.basis, selection.receiver.provenance.projections.join(" / ")),
                selection.derefs, selection.autoref,
                selection.order.iter().map(|t| t.spelling()).collect::<Vec<_>>().join(" -> "), selection.exclusions.join(" | "), paren.line, paren.column + 1, prefix).unwrap();
            total += 1;
        }
    }
    assert!(total > 0);
    fs::write(dir.join("methods.tsv"), out).unwrap();
    let features = if cfg!(feature = "hash") {
        "hash"
    } else {
        "no-default-features"
    };
    let fingerprint = cache::fingerprint(&dir, &["methods.tsv"], features);
    let stdout = match cache::reuse(&dir, &fingerprint) {
        Some(output) => output,
        None => {
            let output = Command::new("python3")
                .arg(workspace.join("scripts/run-bounded.py"))
                .args(["--timeout", "1200", "--", "python3"])
                .arg(workspace.join(
                    "crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/methods.py",
                ))
                .arg(&workspace)
                .arg(&dir)
                .arg(features)
                .output()
                .unwrap();
            fs::write(
                dir.join("harness.log"),
                [&output.stdout[..], &output.stderr[..]].concat(),
            )
            .unwrap();
            print!("{}", String::from_utf8_lossy(&output.stdout));
            assert!(
                output.status.success(),
                "full method oracle failed (claims={total}):\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let output = String::from_utf8_lossy(&output.stdout).into_owned();
            cache::record(&dir, &fingerprint, &output);
            output
        }
    };
    print!("{stdout}");
}
