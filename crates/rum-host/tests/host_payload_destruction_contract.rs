use std::path::Path;
use std::process::Command;

#[test]
fn entropy_and_file_io_providers_declare_bounded_destruction_at_the_crate_edge() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    // The recorded cold ten-consumer run took 60.71s. Ten times that,
    // rounded up to a minute, gives 660s for the whole gate (each check also
    // has its own bound). See report-482-bounded-destruction-file-io.md.
    let output = Command::new("python3")
        .current_dir(repo)
        .args([
            "scripts/run-bounded.py",
            "--timeout",
            "660",
            "--",
            "python3",
            "scripts/check-host-payload-destruction.py",
        ])
        .env("CARGO", env!("CARGO"))
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .expect("the bounded external-consumer check should start");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "external destruction declaration check failed ({:?})\n{stdout}\n{stderr}",
        output.status.code(),
    );
    print!("{stdout}{stderr}");
}
