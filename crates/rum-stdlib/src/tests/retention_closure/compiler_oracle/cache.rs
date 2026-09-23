//! Reuse an oracle run when nothing it depends on has changed.
//!
//! The oracle compiles the crate once per resolution claim, which is what makes
//! it a real instrument and also what took the checker group from 8 s to 115 s.
//! The answer is not to check less. The fingerprint below covers every input a
//! run consumes, so an unchanged tree replays the recorded verdict and any
//! change at all re-runs the compiler.
//!
//! The claim files each test just wrote are part of the key on purpose: they
//! are the checker's own output, so a checker change that moves one resolution
//! invalidates the cache even when no production byte moved. Callers name their
//! claim files explicitly rather than globbing, because the harness writes its
//! own tables into the same directory and a glob would never match twice.

use std::fs;
use std::path::{Path, PathBuf};

use super::super::source;

fn feed(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

fn rust_sources(directory: &Path, into: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|e| panic!("read {}: {e}", directory.display()))
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_sources(&path, into);
        } else if path
            .extension()
            .is_some_and(|e| matches!(e.to_str(), Some("rs" | "py" | "tsv" | "txt" | "toml")))
        {
            into.push(path);
        }
    }
}

/// Everything a run consumes: the crate sources it compiles, the lockfile and
/// toolchain that decide how, the harness that drives it, the claim files the
/// caller just wrote, and the feature set.
pub(super) fn fingerprint(directory: &Path, claims: &[&str], features: &str) -> String {
    let workspace = source::workspace();
    let mut hash = 0xcbf29ce484222325u64;
    feed(&mut hash, features.as_bytes());

    let mut inputs = Vec::new();
    rust_sources(&workspace.join("crates"), &mut inputs);
    inputs.push(workspace.join("Cargo.lock"));
    inputs.push(workspace.join("Cargo.toml"));
    inputs.push(workspace.join("scripts/run-bounded.py"));
    inputs.push(workspace.join("rust-toolchain.toml"));
    for claim in claims {
        inputs.push(directory.join(claim));
    }

    for path in inputs {
        feed(&mut hash, path.to_string_lossy().as_bytes());
        // A claim file that has gone missing changes the key, which re-runs.
        feed(&mut hash, &fs::read(&path).unwrap_or_default());
    }
    format!("fnv1a64:{hash:016x}")
}

/// The recorded verdict, reusable only when the fingerprint matches. Returns
/// the harness output of the recorded run so a reused verdict still prints its
/// evidence. A missing or unreadable record re-runs rather than passing
/// quietly.
pub(super) fn reuse(directory: &Path, fingerprint: &str) -> Option<String> {
    let recorded = fs::read_to_string(directory.join("fingerprint")).ok()?;
    if recorded.trim() != fingerprint {
        return None;
    }
    let verdict = fs::read_to_string(directory.join("verdict.log")).ok()?;
    println!("ORACLE_REUSED {} {fingerprint}", directory.display());
    Some(verdict)
}

/// Recorded only after the run succeeded, so a failure is never cached.
pub(super) fn record(directory: &Path, fingerprint: &str, output: &str) {
    fs::write(directory.join("verdict.log"), output).unwrap();
    fs::write(directory.join("fingerprint"), format!("{fingerprint}\n")).unwrap();
}
