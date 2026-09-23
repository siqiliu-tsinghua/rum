//! Rust-language claims about our own source, checked by the stable toolchain
//! that builds rum. This is NOT the Testing Policy's banned differential
//! testing of rum language behavior against other vendors' WL engines.
//!
//! All mutation/compilation happens in copies under target/brief-534. Nothing
//! here admits a Rust idiom, changes a retention rule, or promotes a region.

mod cache;
mod corpus;
mod exact;
mod sample;

use std::fs;
use std::process::Command;

use super::{regions, source};

#[test]
fn compiler_oracle_agrees_with_resolution_claims() {
    let source = source::production();
    assert!(source.errors.is_empty(), "{:?}", source.errors);
    let claims = source
        .functions
        .values()
        .flatten()
        .flat_map(|f| regions::definition_edges(&source, f).resolved)
        .collect();
    let workspace = source::workspace();
    let dir = workspace.join("target/brief-534/oracle-legacy");
    fs::create_dir_all(&dir).unwrap();
    sample::write(&source, &claims, &dir);
    let features = if cfg!(feature = "hash") {
        "hash"
    } else {
        "no-default-features"
    };
    let fingerprint = cache::fingerprint(
        &dir,
        &["resolution-claims.tsv", "type-claims.tsv"],
        features,
    );
    let stdout = match cache::reuse(&dir, &fingerprint) {
        Some(recorded) => recorded,
        None => {
            let output =
                Command::new("python3")
                    .arg(workspace.join("scripts/run-bounded.py"))
                    .args(["--timeout", "700", "--", "python3"])
                    .arg(workspace.join(
                        "crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/run.py",
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
            assert!(
                output.status.success(),
                "compiler oracle controls failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            cache::record(&dir, &fingerprint, &stdout);
            stdout
        }
    };
    print!("{stdout}");
    // Disagreements are retained findings, not permission to change admission
    // or quietly prune the sample. A missing technique/control is a gate error.
    let controls = fs::read_to_string(dir.join("controls.tsv")).unwrap();
    for required in ["type\tpass", "resolution\tpass", "negative\tpass"] {
        assert!(
            controls.lines().any(|l| l == required),
            "missing required control: {required}"
        );
    }
}
