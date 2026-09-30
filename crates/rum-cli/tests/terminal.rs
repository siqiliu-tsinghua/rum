//! Terminal entry and wait-scoped SIGINT, using isolated real pseudo-terminals.
#![cfg(unix)]

use std::process::Command;

fn run(case: &str) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scratch = root
        .join("../../.temp/brief-554/pty")
        .join(std::process::id().to_string());
    let output = Command::new("python3")
        .arg(root.join("tests/terminal/driver.py"))
        .arg(env!("CARGO_BIN_EXE_rum-cli"))
        .arg(case)
        .arg(scratch)
        .output()
        .expect("python3 is required for real PTY tests");
    assert!(
        output.status.success(),
        "{case}: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn masked_answer() {
    run("masked_answer");
}

#[test]
fn ordinary_answer() {
    run("ordinary_answer");
}

#[test]
fn masked_eof() {
    run("masked_eof");
}

#[test]
fn masked_io_error() {
    run("masked_io_error");
}

#[test]
fn interrupt() {
    run("interrupt");
}

#[test]
fn masked_interrupt() {
    run("masked_interrupt");
}

#[test]
fn primary_sigint() {
    run("primary_sigint");
}

#[test]
fn primary_sigint_after_answer() {
    run("primary_sigint_after_answer");
}

#[test]
fn evaluation_sigint_after_answer() {
    run("evaluation_sigint_after_answer");
}
