use crate::runtime::tests::compiled_library::CompiledLibrary;
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn option_reentry_compile_pass_precedes_pinned_compile_fail() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let scratch = workspace.join("target/brief-531/compile-proof");
    fs::create_dir_all(&scratch).unwrap();
    let library = CompiledLibrary::get();
    let deps = library.deps();
    let docs = include_str!("../reentry.md");
    for (fence, name, succeeds) in [
        ("```no_run\n", "control", true),
        ("```compile_fail,E0061\n", "missing-reentry", false),
    ] {
        let source = docs
            .split_once(fence)
            .unwrap()
            .1
            .split_once("```")
            .unwrap()
            .0;
        let fixture = scratch.join(format!("{name}.rs"));
        fs::write(&fixture, source).unwrap();
        // The repository runner bounds the entire compiler process group.
        let output = Command::new("python3")
            .arg(workspace.join("scripts/run-bounded.py"))
            .args([
                "--timeout",
                "60",
                "--",
                "rustc",
                "--edition=2024",
                "--crate-name",
                "option_reentry_proof",
                "--emit=metadata",
            ])
            .arg("--out-dir")
            .arg(&scratch)
            .arg("-L")
            .arg(format!("dependency={}", deps.display()))
            .arg("--extern")
            .arg(format!("rum_runtime={}", library.runtime().display()))
            .arg("--extern")
            .arg(format!("rum_core={}", library.core().display()))
            .arg(&fixture)
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        fs::write(scratch.join(format!("{name}.stderr")), &stderr).unwrap();
        fs::write(scratch.join(format!("{name}.stdout")), &output.stdout).unwrap();
        assert_eq!(output.status.success(), succeeds, "{name}: {stderr}");
        if !succeeds {
            assert!(
                stderr.contains("error[E0061]"),
                "pin the missing argument error: {stderr}"
            );
            assert_eq!(
                stderr.matches("error[E").count(),
                1,
                "no unrelated failure: {stderr}"
            );
        }
        println!("{name}: success={}\n{stderr}", output.status.success());
    }
}
